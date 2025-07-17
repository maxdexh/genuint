use crate::Uint;

use super::{ArrApi, Array, arr_utils::*};

impl<A: Array<Item = T>, T> ArrApi<A> {
    pub(crate) const fn check_len(&self) -> usize {
        check_len::<A>()
    }
}

impl<A: Array<Item = T, Length = N>, T, N: Uint> ArrApi<A> {
    pub const fn new(inner: A) -> Self {
        Self(inner, core::marker::PhantomData)
    }
    pub const fn into_inner(self) -> A {
        arr_conv(self)
    }
}
