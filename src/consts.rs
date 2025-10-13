//! Various [`Uint`] constants
//!
//! Note that some types in this module require a high recursion limit.

#[allow(unused_imports)] // for docs
use crate::{ToUint, Uint};
use crate::{small::*, uint, uops};

/// Holds a const [`u128`]
///
/// Implements [`ToUint`] for [`small`](crate::small) values.
pub struct ConstU128<const N: u128>;

/// Holds a const [`usize`]
///
/// Implements [`ToUint`] for [`small`](crate::small) values.
pub struct ConstUsize<const N: usize>;

/// Holds a const [`bool`]
///
/// Implements [`ToUint`], using seperate impls for `true` and `false`.
pub struct ConstBool<const B: bool>;
impl ToUint for ConstBool<true> {
    type ToUint = _1;
}
impl ToUint for ConstBool<false> {
    type ToUint = _0;
}

/// [`usize::BITS`] as a [`Uint`]
pub type PtrWidth = uint::From<uops::Shl<ConstUsize<{ size_of::<usize>() }>, _3>>;

/// [`usize::MAX`] as a [`Uint`]
pub type UsizeMax = uint::From<uops::SatSub<uops::Shl<_1, PtrWidth>, _1>>;

/// [`isize::MAX`] as a [`Uint`]
pub type IsizeMax = uint::From<uops::PopBit<UsizeMax>>;

#[test]
fn test_usize_max() {
    assert_eq!(uint::to_usize::<PtrWidth>(), Some(usize::BITS as usize));
    assert_eq!(uint::to_usize::<UsizeMax>(), Some(usize::MAX));
    assert_eq!(uint::to_usize::<IsizeMax>(), Some(isize::MAX as usize));
}

macro_rules! gen_maxes {
    [
        $([$name:ident, $bits:ty, $prim:ty $(,)? ],)*
    ] => {
        $(
            #[doc = concat!("[`", stringify!($prim), "::MAX`] as a [`Uint`]")]
            pub type $name = uint::From<
                crate::uops::_DecUnchecked<
                    crate::uops::Shl<_1, $bits>
                >
            >;
        )*
        #[test]
        fn test_generated_maxes() {
            $(assert_eq!(
                uint::to_u128::<$name>(),
                Some(<$prim>::MAX as u128),
            );)*
        }
    };
}
gen_maxes![
    [I8Max, uint::lit!(7), i8],
    [U8Max, uint::lit!(8), u8],
    [I16Max, uint::lit!(15), i16],
    [U16Max, uint::lit!(16), u16],
    [I32Max, uint::lit!(31), i32],
    [U32Max, uint::lit!(32), u32],
    [I64Max, uint::lit!(63), i64],
    [U64Max, uint::lit!(64), u64],
    [I128Max, uint::lit!(127), i128],
    [U128Max, uint::lit!(128), u128],
];
