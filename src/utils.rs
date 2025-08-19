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
