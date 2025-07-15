#[test]
fn test_decrement() {
    macro_rules! tests {
        ($($val:literal)*) => {$(
            assert_eq!(
                crate::uint::to_u128::<$crate::ops::SatDec<$crate::uint::FromU128<$val>>>(),
                Some({ let val: u128 = $val; val }.saturating_sub(1)),
            );
        )*};
    }
    tests! { 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 }
}

macro_rules! test_op {
    (
        $name:ident:
        $($param:ident)*,
        $got:ty,
        $expect:expr $(,)?
    ) => {
        #[test]
        fn $name() {
            #[allow(non_snake_case)]
            fn callback<$($param: $crate::Uint),*>() {
                $(let $param = $crate::uint::to_u128::<$param>().unwrap();)*
                assert_eq!(
                    $crate::uint::to_u128::<$got>(),
                    Some($expect),
                );
            }
            $crate::ops::testing::__test_op_inner! {
                $($param)*,
                callback
            }
        }
    };
}
pub(crate) use test_op;

macro_rules! __test_op_inner {
    (
        $first:ident $($param:ident)*,
        $callback:ident
    ) => {{
        #[allow(non_snake_case)]
        fn $first<$($param: $crate::Uint,)*>() {
            fn cumul<$first: $crate::Uint, $($param: $crate::Uint),*>(max: u128) {
                if max != 0 {
                    cumul::<$crate::ops::SatDec<$first>, $($param),*>(max - 1);
                }
                $callback::<$first, $($param),*>()
            }
            cumul::<$crate::consts::U10, $($param,)*>(10)
        }
        $crate::ops::testing::__test_op_inner! {
            $($param)*,
            $first
        }
    }};
    (
        ,
        $callback:ident
    ) => {{
        $callback()
    }}
}
pub(crate) use __test_op_inner;
