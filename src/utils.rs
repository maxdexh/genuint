pub const unsafe fn transmute<Src, Dst>(src: Src) -> Dst {
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

pub const fn reverse<T>(slice: &mut [T]) {
    let mut i = 0;
    while i < slice.len() / 2 {
        slice.swap(i, slice.len() - i - 1);
        i += 1;
    }
}

/// ```rust_analyzer_brace_infer
/// private_pub! {}
/// ```
macro_rules! private_pub {
    (
        mod $name:ident;
        $($items:tt)*
    ) => {
        mod $name {
            #[allow(unused_imports)]
            use super::*;
            $($items)*
        }
        #[allow(unused_imports)]
        pub(crate) use $name::*;
    };
}
pub(crate) use private_pub;
