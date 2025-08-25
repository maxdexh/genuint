macro_rules! meta_generate {
    ($($struct:ident $prim:ident,)*) => {
        $(pub struct $struct<const N: $prim>;)*

        macro_rules! generate {
            ($name:ident, $val:expr, $ty:ty) => {
                pub type $name = $ty;
                $(impl $crate::ToUint for $struct<{ $val }> {
                    type ToUint = $name;
                })*
            };
        }
    };
}
meta_generate! {
    ConstUsize usize,
    ConstU128 u128,
    ConstU64 u64,
    ConstU32 u32,
    ConstU16 u16,
}
pub struct ConstU8<const N: u8>;

macro_rules! generate_byte {
    ($name:ident, $val:expr, $ty:ty) => {
        generate!($name, $val, $ty);
        impl $crate::ToUint for ConstU8<{ $val }> {
            type ToUint = $name;
        }
    };
}

generate_byte!(_0, 0, crate::internals::O);
generate_byte!(_1, 1, crate::internals::I);

macro_rules! bisect {
    ($name:ident, $val:expr, $half:ty, $parity:ty, $cb:ident) => {
        $cb! { $name, $val, $crate::internals::A<$half, $parity> }
    };
}
include!(concat!(env!("OUT_DIR"), "/consts.rs"));

/// [`usize::BITS`] as a [`Uint`](crate::Uint).
pub type UsizeBits = crate::ops::Shl<crate::uint::FromUsize<{ size_of::<usize>() }>, _3>;
const _: () = assert!(crate::uint::to_usize::<UsizeBits>().unwrap() == usize::BITS as usize);

/// [`usize::MAX`] as a [`Uint`](crate::Uint).
pub type UsizeMax = crate::ops::SatSub<crate::ops::Shl<_1, UsizeBits>, _1>;

macro_rules! maxes {
    ($($name:ident $bits:ident $tnamefordocs:ident,)*) => {$(
        #[doc = concat!("[`", stringify!($tnamefordocs), "::MAX`], but as a [`Uint`](crate::Uint)")]
        pub type $name = crate::ops::SatSub<crate::ops::Shl<_1, $bits>, _1>;
    )*};
}
maxes! {
    U8Max _8 u8,
    U16Max _16 u16,
    U32Max _32 u32,
    U64Max _64 u64,
    U128Max _128 u128,
}
