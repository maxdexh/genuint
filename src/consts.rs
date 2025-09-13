//! [`Uint`](crate::Uint) contants.

// immediately invoked macro that generates the `new_alias` macro, which takes in
// a declaration name, a const value and a type to generate the type alias _N with
// value N as a `Uint`
macro_rules! generate_alias_cons {
    ($($struct:ident $prim:ident,)*) => {
        $(
            #[doc = core::concat!("Holds a const generic [`", core::stringify!($prim), "`].")]
            ///
            /// Implements [`ToUint`](crate::ToUint) for small values.
            pub struct $struct<const N: $prim>;
        )*

        macro_rules! new_alias {
            ($name:ident, $val:literal, $ty:ty) => {
                #[doc = core::concat!("Type-level `", $val, "`.")]
                pub type $name = crate::uint::From<$ty>;
                $(#[doc(hidden)] impl crate::ToUint for $struct<$val> {
                    type ToUint = $name;
                })*
            };
        }
    };
}
generate_alias_cons! {
    ConstUsize usize,
    ConstU128 u128,
    ConstU64 u64,
    ConstU32 u32,
    ConstU16 u16,
}
/// Holds a const generic [`u8`].
///
/// Implements [`ToUint`](crate::ToUint) for all individual values, but not generically.
/// That is to say that `ConstU8<N>: ToUint` for every `const N: u8` because there is an
/// implementation for every single one, but this does not typecheck:
/// ```compile_fail
/// use generic_uint::{consts::ConstU8, ToUint};
///
/// fn take_uint<N: ToUint>() {}
/// fn every_u8_is_to_uint<const N: u8>() {
///     take_uint::<ConstU8<N>>()
/// }
/// ```
pub struct ConstU8<const N: u8>;

macro_rules! new_byte_alias {
    ($name:ident, $val:literal, $ty:ty) => {
        new_alias!($name, $val, $ty);
        #[doc(hidden)]
        impl crate::ToUint for ConstU8<$val> {
            type ToUint = $name;
        }
    };
}

new_byte_alias!(_0, 0, crate::internals::_0);
new_byte_alias!(_1, 1, crate::internals::_1);

macro_rules! bisect {
    ($name:ident, $val:expr, $half:ty, $parity:ty, $cb:ident) => {
        $cb! { $name, $val, crate::ops::AppendBit<$half, $parity> }
    };
}
include!(concat!(env!("OUT_DIR"), "/consts.rs"));

/// [`usize::BITS`] as a [`Uint`](crate::Uint).
pub type UsizeBits = crate::ops::Shl<crate::uint::FromUsize<{ size_of::<usize>() }>, _3>;
const _: () = assert!(crate::uint::to_usize::<UsizeBits>().unwrap() == usize::BITS as usize);

/// [`usize::MAX`] as a [`Uint`](crate::Uint).
pub type UsizeMax = crate::ops::SatSub<crate::ops::Shl<_1, UsizeBits>, _1>;

macro_rules! gen_maxes {
    [
        $bitsmac:ident,
        $val:ty,
        $([$name:ident, $bits:ident, $tnamefordocs:ty $(,)? ],)*
    ] => {$(
        macro_rules! $bitsmac {
            () => ($bits)
        }
        #[cfg(test)]
        const _: () = {
            type _Bits = $bits;
        };
        #[doc = concat!("[`", stringify!($tnamefordocs), "::MAX`], but as a [`Uint`](crate::Uint)")]
        pub type $name = $val;
    )*};
}
gen_maxes![
    __bits,
    crate::ops::SatSub<crate::ops::Shl<_1, __bits!()>, _1>,
    [U8Max, _8, u8],
    [U16Max, _16, u16],
    [U32Max, _32, u32],
    [U64Max, _64, u64],
    [U128Max, _128, u128],
];
