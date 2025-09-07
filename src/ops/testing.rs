#![cfg(test)]

use crate::{Uint, consts::_0, uint};

pub(crate) type SatDec<N> = uint::From<crate::ops::satdec::SatDecIfL<N>>;

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

pub(crate) type DefaultHi = crate::consts::_100;
pub(crate) type DefaultLo = crate::consts::_0;

/// A type-level linked list of `Uint`s
pub(crate) trait UintList {
    const EMPTY: bool;
    type First: Uint;
    type Tail: UintList;
}
/// A type-level linked list of pairs of `UInt`s
/// representing an n-dimensional range of inputs
/// to be tested.
pub(crate) trait UintRanges {
    const EMPTY: bool;
    type FirstLo: Uint;
    type FirstHi: Uint;
    type Tail: UintRanges;

    /// Reduces a `Tests` type by running `run_tests::<(N, L)>`
    /// for N ranging from `FirstLo` to `FirstHi`. The reduced
    /// tests will hence require one less parameter than before.
    type ReduceTests<L: Tests<Ranges = Self>>: Tests<Ranges = Self::Tail>;
}
pub(crate) trait Tests {
    /// The n-dimensional input range
    type Ranges: UintRanges;
    fn run_tests<L: UintList>();
}

// The empty list must be cyclical for `Tail` and `ReduceTests` when recursing over it,
// so that we don't need to monomorphize infinitely many functions.
impl UintList for () {
    const EMPTY: bool = true;
    type First = _0;
    type Tail = Self;
}
impl UintRanges for () {
    const EMPTY: bool = true;
    type FirstLo = _0;
    type FirstHi = _0;
    type Tail = Self;
    type ReduceTests<T: Tests<Ranges = Self>> = T;
}

impl<N: Uint, L: UintList> UintList for (N, L) {
    const EMPTY: bool = false;
    type First = N;
    type Tail = L;
}
impl<L: Uint, H: Uint, R: UintRanges> UintRanges for ((L, H), R) {
    const EMPTY: bool = false;
    type FirstLo = L;
    type FirstHi = H;
    type Tail = R;
    type ReduceTests<T: Tests<Ranges = Self>> = ReduceTests<T>;
}

pub(crate) struct ReduceTests<T>(T);
/// Implement the functionality specified by `ReduceTest`.
impl<T: Tests> Tests for ReduceTests<T> {
    type Ranges = <T::Ranges as UintRanges>::Tail;
    fn run_tests<L: UintList>() {
        fn traverse<T: Tests, L: UintList, N: Uint>() {
            T::run_tests::<(N, L)>();
            let next = const {
                use core::cmp::Ordering;
                match uint::cmp::<N, <T::Ranges as UintRanges>::FirstLo>() {
                    Ordering::Greater => {
                        // Avoid monomorphizing functions with params below `Lo`. While not
                        // strictly guaranteed by the compiler, this saves us from
                        // instantiating some unused versions of `traverse`. Not needed for
                        // termination since we will always reach a cycle at SatDec<0> = 0.
                        const fn next_traverse<T: Tests, L: UintList, N: Uint>() -> fn() {
                            traverse::<T, L, SatDec<N>>
                        }
                        next_traverse::<T, L, N>()
                    }
                    Ordering::Less => unreachable!(),
                    Ordering::Equal => || {},
                }
            };
            next()
        }
        let checked = const {
            if uint::cmp::<<T::Ranges as UintRanges>::FirstLo, <T::Ranges as UintRanges>::FirstHi>()
                .is_le()
            {
                const fn get_traverse<T: Tests, L: UintList>() -> fn() {
                    traverse::<T, L, <T::Ranges as UintRanges>::FirstHi>
                }
                get_traverse::<T, L>()
            } else {
                || {}
            }
        };
        checked()
    }
}
/// Recursively apply `ReduceTest` until we have no parameters left.
pub(crate) fn run_tests_reduce_all<L: Tests>() {
    let reduced = const {
        if L::Ranges::EMPTY {
            // avoid unnecessary monomorphizations
            const fn get_leaf<L: Tests>() -> fn() {
                L::run_tests::<()>
            }
            get_leaf::<L>()
        } else {
            // no need to guard here. if the range is empty, ReduceTest<L> = L and we
            // just get the current instance of the function, which is already monomorphized
            run_tests_reduce_all::<<L::Ranges as UintRanges>::ReduceTests<L>>
        }
    };
    reduced()
}

/// ```rust_analyzer_prefer_braces
/// test_op! {}
/// ```
macro_rules! test_op {
    (
        $name:ident:
        $first:ident $($param:ident)*,
        $got:ty,
        $expect:expr
        $(, $( $range:tt )* )?
    ) => {
        crate::ops::testing::test_op! {
            @shift
            $name
            [$first $($param)*],
            // Shift the params left and add an extra param.
            [$($param)* __Extra],
            $got,
            $expect,
            [$(, $( $range )*)?]
        }
    };
    (
        @shift
        $name:ident
        [$first:ident $($param:ident)*],
        [$fshifted:ident $($shifted:ident)*],
        $got:ty,
        $expect:expr,
        [$($range:tt)*]
    ) => {
        #[test]
        fn $name() {
            struct Leaf;
            impl crate::ops::testing::Tests for Leaf {
                type Ranges = crate::ops::testing::test_op!(
                    @ranges
                    [ $first $($param)* ]
                    $($range)*
                );
                fn run_tests<L: crate::ops::testing::UintList>() {
                    Flattener::<L>::doit()
                }
            }
            struct Flattener<L>(L);
            impl<
                // Name a list using each param. The tail of the list
                // is the parameter after it. For the last parameter,
                // the tail doesn't matter, so use an extra dummy param.
                $first: crate::ops::testing::UintList<
                    Tail = $fshifted
                >
                $(, $param: crate::ops::testing::UintList<
                    Tail = $shifted
                >)*
                , __Extra: crate::ops::testing::UintList
            > Flattener<$first> {
                fn doit() {
                    // By generating code that has an explicit name for each
                    // tail list, we can now directly name all items of the
                    // list. As a bonus, we can use the dummy param to check
                    // that the input list has the correct length.
                    const {
                        debug_assert!(__Extra::EMPTY);
                        debug_assert!(!$first::EMPTY);
                        $(debug_assert!(!$param::EMPTY);)*
                    }
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

            crate::ops::testing::run_tests_reduce_all::<Leaf>()
        }
    };
    (
        @ranges
        []
        $(,)?
    ) => {
        ()
    };
    (
        @ranges
        []
        $($rest:tt)+
    ) => {
        core::compile_error! { core::concat!("Leftover ranges: ", stringify!($($rest)+)) }
    };
    (
        @ranges
        [ $_:ident $($rest:ident)* ]
    ) => {
        (
            (crate::ops::testing::DefaultLo, crate::ops::testing::DefaultHi),
            crate::ops::testing::test_op!(@ranges [$($rest)*]),
        )
    };
    (
        @ranges
        [ $_:ident $($rest:ident)* ]
        , ..
        $(, $($range_rest:tt)*)?
    ) => {
        (
            (crate::ops::testing::DefaultLo, crate::ops::testing::DefaultHi),
            crate::ops::testing::test_op!(@ranges [$($rest)*] $(, $($range_rest)*)?),
        )
    };
    (
        @ranges
        [ $_:ident $($rest:ident)* ]
        , $lo:tt..
        $(, $($range_rest:tt)*)?
    ) => {
        (
            (crate::ops::testing::test_op!(@bound $lo), crate::ops::testing::DefaultHi),
            crate::ops::testing::test_op!(@ranges [$($rest)*] $(, $($range_rest)*)?),
        )
    };
    (
        @ranges
        [ $_:ident $($rest:ident)* ]
        , ..=$hi:tt
        $(, $($range_rest:tt)*)?
    ) => {
        (
            (crate::ops::testing::DefaultLo, crate::ops::testing::test_op!(@bound $hi)),
            crate::ops::testing::test_op!(@ranges [$($rest)*] $(, $($range_rest)*)?),
        )
    };
    (
        @ranges
        [ $_:ident $($rest:ident)* ]
        , $lo:tt..=$hi:tt
        $(, $($range_rest:tt)*)?
    ) => {
        (
            (crate::ops::testing::test_op!(@bound $lo), crate::ops::testing::test_op!(@bound $hi)),
            crate::ops::testing::test_op!(@ranges [$($rest)*] $(, $($range_rest)*)?),
        )
    };
    (@bound $n:ty) => { $n };
    (@bound $n:expr) => { crate::uint::FromU128<{$n}> };
}
pub(crate) use test_op;
