use core::{mem::MaybeUninit, ops::Range, ptr::NonNull};

/// Performs the operation of writing the argument into a `repr(C)` union
/// of `Src` and `Dst` and reading out `Dst`.
pub const unsafe fn union_transmute<Src, Dst>(src: Src) -> Dst {
    use core::mem::ManuallyDrop;
    #[repr(C)]
    union Helper<Src, Dst> {
        src: ManuallyDrop<Src>,
        dst: ManuallyDrop<Dst>,
    }
    ManuallyDrop::into_inner(unsafe {
        Helper {
            src: ManuallyDrop::new(src),
        }
        .dst
    })
}

/// Transmutes types of the same size.
pub const unsafe fn exact_transmute<Src, Dst>(src: Src) -> Dst {
    debug_assert!(size_of::<Src>() == size_of::<Dst>());
    unsafe { core::mem::transmute_copy(&core::mem::ManuallyDrop::new(src)) }
}

pub const unsafe fn assume_init_slice<T>(slice: &[MaybeUninit<T>]) -> &[T] {
    unsafe { core::slice::from_raw_parts(slice.as_ptr().cast(), slice.len()) }
}
pub const unsafe fn assume_init_mut_slice<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
    unsafe { core::slice::from_raw_parts_mut(slice.as_mut_ptr().cast(), slice.len()) }
}
pub const unsafe fn subslice_nonnull<T>(
    slice: NonNull<[MaybeUninit<T>]>,
    Range { start, end }: Range<usize>,
) -> NonNull<[T]> {
    debug_assert!(start <= end);
    debug_assert!(end <= slice.len());
    NonNull::slice_from_raw_parts(unsafe { slice.cast().add(start) }, end - start)
}
