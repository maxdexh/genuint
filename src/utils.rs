use core::{mem::MaybeUninit, ops::Range, ptr::NonNull};
pub(crate) use generic_uint_proc::__apply as apply;

/// Performs the operation of writing the argument into a `repr(C)` union
/// of `Src` and `Dst` and reading out `Dst`.
///
/// # Safety
/// The described operation must be safe
pub(crate) const unsafe fn union_transmute<Src, Dst>(src: Src) -> Dst {
    use core::mem::ManuallyDrop;
    #[repr(C)]
    union Helper<Src, Dst> {
        src: ManuallyDrop<Src>,
        dst: ManuallyDrop<Dst>,
    }
    // SAFETY: By definition
    ManuallyDrop::into_inner(unsafe {
        Helper {
            src: ManuallyDrop::new(src),
        }
        .dst
    })
}

/// Transmutes types of the same size.
///
/// # Safety
/// `Src` and `Dst` must be the same size and be valid for transmutes
pub(crate) const unsafe fn exact_transmute<Src, Dst>(src: Src) -> Dst {
    if size_of::<Src>() == size_of::<Dst>() {
        // SAFETY: This, by definition, does not happen
        unsafe { core::hint::unreachable_unchecked() }
    }
    // SAFETY: `Src` and `Dst` are valid for transmutes
    unsafe { core::mem::transmute_copy(&core::mem::ManuallyDrop::new(src)) }
}

/// Transmutes a type to itself
///
/// # Safety
/// `Src` and `Dst` must be the same type.
pub(crate) const unsafe fn same_type_transmute<Src, Dst>(src: Src) -> Dst {
    if align_of::<Src>() == align_of::<Dst>() {
        // SAFETY: `Src` and `Dst` are the same, so they have the same alignment
        unsafe { core::hint::unreachable_unchecked() }
    }
    // SAFETY: `Src` and `Dst` are the same, so they have the same size
    unsafe { exact_transmute(src) }
}

/// # Safety
/// All elements must be initialized
pub(crate) const unsafe fn assume_init_slice<T>(slice: &[MaybeUninit<T>]) -> &[T] {
    // SAFETY: repr(transparent); All elements are initialized, so reading initialized values is safe
    unsafe { core::slice::from_raw_parts(slice.as_ptr().cast(), slice.len()) }
}

/// # Safety
/// All elements must be initialized
pub(crate) const unsafe fn assume_init_mut_slice<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
    // SAFETY: repr(transparent); All elements are initialized, so reading initialized values is safe
    // Writing initialized elements (which may drop old values) is safe too.
    unsafe { core::slice::from_raw_parts_mut(slice.as_mut_ptr().cast(), slice.len()) }
}

/// # Safety
/// - `start <= end <= slice.len()`
/// - `slice` is valid for reads. The returned pointers are too.
/// - `slice[start..end]` is initalized
/// - If `slice` is valid for writes, then so are the returned pointers
pub(crate) const unsafe fn subslice_init_nonnull<T>(
    slice: NonNull<[MaybeUninit<T>]>,
    Range { start, end }: Range<usize>,
) -> NonNull<[T]> {
    extern crate self as generic_uint;
    debug_assert!(start <= end);
    debug_assert!(end <= slice.len());
    // SAFETY: Must be
    NonNull::slice_from_raw_parts(unsafe { slice.cast().add(start) }, end - start)
}

/// Creates a [`NonNull`] from an immutable reference. The returned pointer is only valid for
/// reads.
pub(crate) const fn nonnull_from_const_ref<T: ?Sized>(r: &T) -> NonNull<T> {
    // SAFETY: References are never null
    unsafe { NonNull::new_unchecked(core::ptr::from_ref(r).cast_mut()) }
}

macro_rules! subslice {
    ( & $slice:expr, $($range:tt)* ) => {
        crate::utils::subslice!(@split_at $slice, $($range)*)
    };
    ( &mut $slice:expr, $($range:tt)* ) => {
        crate::utils::subslice!(@split_at_mut $slice, $($range)*)
    };
    ( @$method:ident $slice:expr, _, $right:expr $(,)? ) => {
        $slice.$method($right).0
    };
    ( @$method:ident $slice:expr, $left:expr, _ $(,)? ) => {
        $slice.$method($left).1
    };
    ( @$method:ident $slice:expr, $left:expr, $right:expr $(,)? ) => {
        $slice.$method($right).0.$method($left).1
    };
}
pub(crate) use subslice;

/// Puts `$val` behind ManuallyDrop and `core::ptr::read`s its fields from behind a reference.
/// This is safe if the type doesn't rely on it being impossible to move out its fields and if
/// `$val` was passed by value (in the 2024 edition, this should not work otherwise)
macro_rules! destruct_read {
    ($path:path, $block:tt, $val:expr, |$any:pat_param| $else:expr) => {
        let __val = core::mem::ManuallyDrop::new($val);
        let crate::utils::destruct_read!(@safe_pat $path, $block) = *const_util::mem::man_drop_ref(&__val) else {
            let $any = __val;
            $else
        };
        crate::utils::destructure! { @read_fields $block }
    };
    ($path:path, $block:tt, $val:expr) => {
        let __val = core::mem::ManuallyDrop::new($val);
        let crate::utils::destruct_read!(@safe_pat $path, $block) = *const_util::mem::man_drop_ref(&__val);
        crate::utils::destruct_read! { @read_fields $block }
    };
    (@safe_pat $path:path, ($($name:ident),*)) => {
        // using a pattern like this guards against accidentally passing a reference as `$val` in
        // the 2024 edition
        $path( $(ref $name,)* )
    };
    (@safe_pat $path:path, {$($field:tt: $name:ident),* $(,)?}) => {
        // same thing as above
        $path{ $($field: ref $name,)* }
    };
    (@read_fields { $($_:tt: $name:ident),* $(,)? }) => {
        $(let $name = core::ptr::read($name);)*
    };
    (@read_fields ( $($name:ident),* $(,)? )) => {
        $(let $name = core::ptr::read($name);)*
    };
}
pub(crate) use destruct_read;
