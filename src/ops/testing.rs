#![cfg(test)]

use crate::{Uint, uint};

pub(crate) type SatDec<N> = uint::From<crate::ops::satdec::SatDecIfL<N>>;
pub(crate) const DEFAULT_LO: u128 = 0;
pub(crate) type DefaultHi = crate::consts::_10;

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

/// ```rust_analyzer_brace_infer
/// test_op! {}
/// ```
macro_rules! test_op {
    (
        $name:ident:
        $first:ident $($param:ident)*,
        $got:ty,
        $expect:expr
        $(,$(
            $($lo:literal)?..$(=$hi:literal)?
            $($rest:tt)*
        )?)?
    ) => {
        #[test]
        fn $name() {
            struct __Callback<$($param),*>($($param),*);
            impl<$($param: crate::Uint),*> crate::ops::testing::Tests for __Callback<$($param),*> {
                type Hi = crate::ops::testing::__hi_or_default![ $($($( $hi )?)?)? ];
                const LO: u128 = crate::ops::testing::__lo_or_default![ $($($( $lo )?)?)? ];
                type Partial<$first: crate::Uint> = Self;

                const IS_LEAF: bool = true;

                #[expect(non_snake_case)]
                fn leaf_test<$first: crate::Uint>() {
                    let $first = crate::uint::to_u128::<$first>().unwrap();
                    $(let $param = crate::uint::to_u128::<$param>().unwrap();)*
                    assert_eq!(
                        crate::uint::to_u128::<$got>(),
                        Some($expect),
                        "params={:?}",
                        ($($param),*)
                    );
                }
            }
            crate::ops::testing::__test_op_inner! {
                $($param)*,
                __Callback
                $($($($rest)*)?)?
            }
        }
    };
}
pub(crate) use test_op;

// TODO: optimize
macro_rules! __test_op_inner {
    (
        $first:ident $($param:ident)*,
        $callback:ident
        $(,$(
            $($lo:literal)?..$(=$hi:literal)?
            $($rest:tt)*
        )?)?
    ) => {{
        struct $first<$($param),*>($($param),*);
        impl<$($param: crate::Uint),*> crate::ops::testing::Tests for $first<$($param),*> {
            type Hi = crate::ops::testing::__hi_or_default![ $($($( $hi )?)?)? ];
            const LO: u128 = crate::ops::testing::__lo_or_default![ $($($( $lo )?)?)? ];
            type Partial<$first: crate::Uint> = $callback<$first, $($param),*>;
        }
        crate::ops::testing::__test_op_inner! {
            $($param)*,
            $first
            $($($($rest)*)?)?
        }
    }};
    (
        ,
        $callback:ident $(,)?
    ) => {{
        crate::ops::testing::run_tests::<$callback>()
    }}
}
pub(crate) use __test_op_inner;

macro_rules! __hi_or_default {
    ($hi:expr) => {
        crate::uint::FromU128<{ $hi }>
    };
    () => {
        crate::ops::testing::DefaultHi
    }
}
pub(crate) use __hi_or_default;
macro_rules! __lo_or_default {
    ($lo:expr) => {
        $lo
    };
    () => {
        crate::ops::testing::DEFAULT_LO
    };
}
pub(crate) use __lo_or_default;

pub trait Tests {
    type Hi: Uint;
    const LO: u128;
    type Partial<N: Uint>: Tests;

    const IS_LEAF: bool = false;
    fn leaf_test<N: Uint>() {}
}

pub fn run_tests<T: Tests>() {
    fn traverse<T: Tests, N: Uint>() {
        T::leaf_test::<N>();
        if !T::IS_LEAF {
            run_tests::<T::Partial<N>>()
        }

        (const {
            let n = uint::to_u128::<N>().unwrap();
            let lo = T::LO;

            if n > lo {
                const fn next_traverse<T: Tests, N: Uint>() -> fn() {
                    traverse::<T, SatDec<N>>
                }
                next_traverse::<T, N>()
            } else if n == lo {
                || {}
            } else {
                panic!("Unnecessary monomorphization")
            }
        })();
    }
    traverse::<T, T::Hi>();
}
