#![cfg(test)]

#[test]
/// Make sure the test runner is actually testing anything, since it uses SatDec to traverse ranges.
fn test_satdec() {
    fn doit<const N: u128, V: Uint>()
    where
        crate::consts::ConstU128<N>: crate::ToUint<ToUint = V>,
    {
        assert_eq!(uint::to_u128::<SatDec<V>>(), Some(N.saturating_sub(1)),)
    }
    macro_rules! tests {
        ($($val:literal)*) => {$(
            doit::<$val, _>();
        )*};
    }
    tests! { 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 }
}

use crate::{
    Uint,
    consts::{_0, _1},
    uint,
};

pub(crate) type SatDec<N> = uint::From<crate::ops::satdec::SatDecIfL<N>>;
pub(crate) type DefaultHi = crate::consts::_10;
pub(crate) type DefaultLo = crate::consts::_0;

pub trait UintList {
    type IsEmpty: Uint;

    type First: Uint;
    type Rest: UintList;
}
impl UintList for () {
    type IsEmpty = _1;

    type First = _0;
    type Rest = Self;
}
impl<N: Uint, L: UintList> UintList for (N, L) {
    type IsEmpty = _0;

    type First = N;
    type Rest = L;
}
pub trait UintRanges {
    const EMPTY: bool;
    type FirstLo: Uint;
    type FirstHi: Uint;
    type Rest: UintRanges;

    type ReduceLeaf<L: TestLeaf>: TestLeaf;
}
impl UintRanges for () {
    const EMPTY: bool = true;
    type FirstLo = _0;
    type FirstHi = _0;
    type Rest = Self;
    type ReduceLeaf<T: TestLeaf> = T;
}
impl<L: Uint, H: Uint, R: UintRanges> UintRanges for ((L, H), R) {
    const EMPTY: bool = false;
    type FirstLo = L;
    type FirstHi = H;
    type Rest = R;
    type ReduceLeaf<T: TestLeaf> = ReducedLeaf<T>;
}
pub struct ReducedLeaf<T>(T);
impl<T: TestLeaf> TestLeaf for ReducedLeaf<T> {
    type Ranges = <T::Ranges as UintRanges>::Rest;
    fn leaf_test<L: UintList>() {
        fn traverse<T: TestLeaf, L: UintList, N: Uint>() {
            T::leaf_test::<(N, L)>();
            let next = const {
                match uint::cmp::<N, <T::Ranges as UintRanges>::FirstLo>() {
                    core::cmp::Ordering::Less => unreachable!(),
                    core::cmp::Ordering::Equal => || {},
                    core::cmp::Ordering::Greater => {
                        const fn next_traverse<T: TestLeaf, L: UintList, N: Uint>() -> fn() {
                            traverse::<T, L, SatDec<N>>
                        }
                        next_traverse::<T, L, N>()
                    }
                }
            };
            next()
        }
        traverse::<T, L, <T::Ranges as UintRanges>::FirstHi>()
    }
}
pub trait TestLeaf {
    type Ranges: UintRanges;
    fn leaf_test<L: UintList>();
}
pub fn run_test_leaf<L: TestLeaf>() {
    let reduced = const {
        if L::Ranges::EMPTY {
            const fn get_leaf<L: TestLeaf>() -> fn() {
                L::leaf_test::<()>
            }
            get_leaf::<L>()
        } else {
            const fn reduce_leaf<L: TestLeaf>() -> fn() {
                run_test_leaf::<<L::Ranges as UintRanges>::ReduceLeaf<L>>
            }
            reduce_leaf::<L>()
        }
    };
    reduced()
}

/// ```rust_analyzer_brace_infer
/// test_op! {}
/// ```
macro_rules! test_op {
    (
        $name:ident:
        $first:ident $($param:ident)*,
        $got:ty,
        $expect:expr
        $(, $($lo:literal)?..$(=$hi:literal)?)* $(,)?
    ) => {
        crate::ops::testing::test_op! {
            @shift
            $name
            [$first $($param)*],
            [$($param)* Extra],
            $got,
            $expect,
            [$($($lo)?..$(=$hi)?),*]
        }
    };
    (
        @shift
        $name:ident
        [$first:ident $($param:ident)*],
        [$fshifted:ident $($shifted:ident)*],
        $got:ty,
        $expect:expr,
        [$($($lo:literal)?..$(=$hi:literal)?),*]
    ) => {
        #[test]
        fn $name() {
            struct Leaf;
            impl crate::ops::testing::TestLeaf for Leaf {
                type Ranges = crate::ops::testing::make_ranges!(
                    [ $first $($param)* ]
                    $(( $($lo)?, $($hi)? ))*
                );
                fn leaf_test<L: crate::ops::testing::UintList>() {
                    Flattener::<L>::doit()
                }
            }
            struct Flattener<L>(L);
            impl<
                $first: crate::ops::testing::UintList<
                    Rest = $fshifted
                >
                $(, $param: crate::ops::testing::UintList<
                    Rest = $shifted
                >)*
                , Extra: crate::ops::testing::UintList
            > Flattener<$first> {
                fn doit() {
                    doit::<$first::First $(, $param::First)*>()
                }
            }
            #[expect(non_snake_case)]
            fn doit<$first: crate::Uint $(, $param: crate::Uint)*>() {
                let $first = crate::uint::to_u128::<$first>().unwrap();
                $(let $param = crate::uint::to_u128::<$param>().unwrap();)*
                assert_eq!(
                    crate::uint::to_u128::<$got>(),
                    Some($expect),
                    "params={:?}",
                    ($($param),*)
                );
            }

            crate::ops::testing::run_test_leaf::<Leaf>()
        }
    }
}
pub(crate) use test_op;
macro_rules! make_ranges {
    (
        []
    ) => {
        ()
    };
    (
        []
        $($rest:tt)+
    ) => {
        core::compile_error! { "Leftover ranges" }
    };
    (
        [ $_:ident $($rest:ident)* ]
        $( ($($lo:expr)?, $($hi:expr)?)  $($rest2:tt)* )?
    ) => {
        (
            (
                crate::ops::testing::make_ranges!(@bound DefaultLo $($($lo)?)? ),
                crate::ops::testing::make_ranges!(@bound DefaultHi $($($hi)?)? ),
            ),
            crate::ops::testing::make_ranges!([$($rest)*] $($($rest2)*)?),
        )
    };
    (
        @bound
        $default:ident
    ) => {
        crate::ops::testing::$default
    };
    (
        @bound
        $default:ident
        $ex:expr
    ) => {
        crate::uint::FromU128<{$ex}>
    };
}
pub(crate) use make_ranges;
