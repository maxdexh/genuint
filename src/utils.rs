use core::{mem::MaybeUninit, ops::Range, ptr::NonNull};

// Internal API

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
    debug_assert!(size_of::<Src>() == size_of::<Dst>());
    // SAFETY: `Src` and `Dst` are valid for transmutes
    unsafe { core::mem::transmute_copy(&core::mem::ManuallyDrop::new(src)) }
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
        $crate::utils::subslice!(@split_at $slice, $($range)*)
    };
    ( &mut $slice:expr, $($range:tt)* ) => {
        $crate::utils::subslice!(@split_at_mut $slice, $($range)*)
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

macro_rules! min {
    ($lhs:expr, $rhs:expr) => {{
        let __lhs = $lhs;
        let __rhs = $rhs;
        if __lhs < __rhs { __lhs } else { __rhs }
    }};
}
pub(crate) use min;
