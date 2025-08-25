pub struct ConstUsize<const N: usize>;
pub struct ConstU128<const N: u128>;

macro_rules! generate {
    ($name:ident, $val:expr, $ty:ty) => {
        pub type $name = $ty;
        impl $crate::ToUint for ConstUsize<{ $val }> {
            type ToUint = $name;
        }
        impl $crate::ToUint for ConstU128<{ $val }> {
            type ToUint = $name;
        }
    };
}
generate!(_0, 0, crate::internals::O);
generate!(_1, 1, crate::internals::I);

macro_rules! bisect {
    ($name:ident, $val:expr, $half:ty, $parity:ty) => {
        generate! { $name, $val, $crate::internals::A<$half, $parity> }
    };
}
include!(concat!(env!("OUT_DIR"), "/consts.rs"));

/// [`usize::BITS`] as a [`Uint`](crate::Uint).
// NOTE: This implementation assumes that size_of::<usize>() <= 256, i.e. it assumes at
// most a 2048-bit platform (lol)
pub type UsizeBits = crate::ops::Shl<crate::uint::FromUsize<{ size_of::<usize>() }>, _3>;
const _: () = assert!(crate::uint::to_usize::<UsizeBits>().unwrap() == usize::BITS as usize);

/// [`usize::MAX`] as a [`Uint`](crate::Uint).
pub type UsizeMax = crate::ops::SatSub<crate::ops::Shl<_1, UsizeBits>, _1>;
