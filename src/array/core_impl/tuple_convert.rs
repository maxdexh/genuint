macro_rules! tuple_impl {
    (
        $out:tt $($T:ident)*
    ) => {
        const _: () = {
            const COUNT_MAX: usize = 0 $(+ { let _ = stringify!($T); 1 })*;
            tuple_impl! {
                @
                $out $($T)*,
                COUNT_MAX,
            }
        };
    };
    (
        @
        $out:tt,
        $_:expr,
        $($types:tt $count:tt)*
    ) => {
        crate::utils::expand! {
            [ tuple count ]
            $out
            $( [$types $count] )*
        }
    };
    (
        @
        $out:tt $F:ident $($T:ident)*,
        $count:expr,
        $($types:tt)*
    ) => {
        tuple_impl! {
            @
            $out $($T)*,
            $count - 1,
            $($types)* ($F, $($T,)*) ($count)
        }
    };
}
tuple_impl! {
    {$(
        const _: () = {
            use crate::array::{ArrApi, Array};

            const COUNT: usize = $count;
            impl<A, T> From<ArrApi<A>> for $tuple
            where
                A: Array<Item = T, Length = crate::uint::FromUsize<COUNT>>,
            {
                fn from(value: ArrApi<A>) -> Self {
                    crate::array::helper::arr_convert::<_, [_; COUNT]>(value).into()
                }
            }
            impl<A, T> From<$tuple> for ArrApi<A>
            where
                A: Array<Item = T, Length = crate::uint::FromUsize<COUNT>>
            {
                fn from(value: $tuple) -> Self {
                    crate::array::helper::arr_convert(<[_; COUNT]>::from(value))
                }
            }
        };
    )}
    T T T T
    T T T T
    T T T T
}
