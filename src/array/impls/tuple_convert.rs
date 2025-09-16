macro_rules! tuple_impl {
    (
        $helpers:tt
        $($T:ident)*
    ) => {
        const _: () = {
            const COUNT_MAX: usize = 0 $(+ { _ = stringify!($T); 1 })*;
            tuple_impl! {
                $helpers
                $($T)*,
                COUNT_MAX
            }
        };
    };
    (
        $helpers:tt,
        $count:expr
    ) => {};
    (
        {
            $Alias:ident
            $item:item
            $item2:item
        }
        $F:ident $($T:ident)*,
        $count:expr
    ) => {
        tuple_impl! {
            { $Alias $item $item2 }
            $($T)*,
            $count - 1
        }
        const _: () = {
            const COUNT: usize = $count;
            type $Alias<$F> = ($F, $($T),*);
            $item
            $item2
        };
    };
}

use crate::array::*;
tuple_impl! {
    {
        Tuple
        impl<A, T> From<ArrApi<A>> for Tuple<T>
        where
            A: Array<Item = T, Length = crate::uint::From<crate::consts::ConstUsize<COUNT>>>,
        {
            fn from(value: ArrApi<A>) -> Self {
                crate::array::ArrApi::retype::<[_; COUNT]>(value).into()
            }
        }
        impl<A, T> From<Tuple<T>> for ArrApi<A>
        where
            A: Array<Item = T, Length = crate::uint::From<crate::consts::ConstUsize<COUNT>>>,
        {
            fn from(value: Tuple<T>) -> Self {
                crate::array::ArrApi::retype_from(<[_; COUNT]>::from(value))
            }
        }
    }
    T T T T
    T T T T
    T T T T
}
