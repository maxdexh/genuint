#![cfg(test)]

pub(crate) type SatDec<N> = crate::uint::From<crate::ops::satdec::SatDecIfL<N>>;

// Test decrementing itself before we use it in all other tests.
#[test]
fn test_decrement() {
    macro_rules! tests {
        ($($val:literal)*) => {$(
            assert_eq!(
                crate::uint::to_u128::<crate::ops::testing::SatDec<crate::uint::FromU128<$val>>>(),
                Some(u128::saturating_sub($val, 1)),
            );
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
                type Hi = crate::uint::FromU128<{ crate::ops::testing::__expr_or!($($($($hi)?)?)?, 10) }>;
                const LO: u128 = crate::ops::testing::__expr_or!($($($($lo)?)?)?, 0);
                type Partial<$first: crate::Uint> = Self;

                const IS_LEAF: bool = true;

                #[allow(non_snake_case)]
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
            type Hi = crate::uint::FromU128<{ crate::ops::testing::__expr_or!($($($($hi)?)?)?, 10) }>;
            const LO: u128 = crate::ops::testing::__expr_or!($($($($lo)?)?)?, 0);
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

macro_rules! __expr_or {
    (, $def:expr) => {
        $def
    };
    ($val:expr, $_:expr) => {
        $val
    };
}
pub(crate) use __expr_or;

use crate::Uint;

pub trait Tests {
    type Hi: Uint;
    const LO: u128;
    type Partial<N: Uint>: Tests;

    const IS_LEAF: bool = false;
    fn leaf_test<N: Uint>() {}
}

pub fn run_tests<T: Tests>() {
    fn run_leaf<T: Tests>() {
        const { debug_assert!(T::IS_LEAF) }
        fn traverse<T: Tests, N: Uint>() {
            T::leaf_test::<N>();

            (const {
                let n = crate::uint::to_u128::<N>().unwrap();
                let lo = T::LO;

                if n > lo {
                    const fn next_traverse<T: Tests, N: Uint>() -> fn() {
                        traverse::<T, crate::ops::testing::SatDec<N>>
                    }
                    next_traverse::<T, N>()
                } else if n == lo {
                    || {}
                } else {
                    panic!("Unnecessary monomorphization")
                }
            })()
        }
        traverse::<T, T::Hi>();
    }
    fn run_non_leaf<T: Tests>() {
        const { debug_assert!(!T::IS_LEAF) }
        fn traverse<T: Tests, N: Uint>() {
            run_tests::<T::Partial<N>>();

            (const {
                let n = crate::uint::to_u128::<N>().unwrap();
                let lo = T::LO;

                if n > lo {
                    const fn next_traverse<T: Tests, N: Uint>() -> fn() {
                        traverse::<T, crate::ops::testing::SatDec<N>>
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

    (const {
        if T::IS_LEAF {
            const fn get_leaf_runner<T: Tests>() -> fn() {
                run_leaf::<T>
            }
            get_leaf_runner::<T>()
        } else {
            const fn get_non_leaf_runner<T: Tests>() -> fn() {
                run_non_leaf::<T>
            }
            get_non_leaf_runner::<T>()
        }
    })();
}
