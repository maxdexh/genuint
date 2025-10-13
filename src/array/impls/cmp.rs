use crate::{
    Uint,
    array::{ArrApi, Array},
};

impl<A, B> PartialEq<ArrApi<B>> for ArrApi<A>
where
    A: Array,
    B: Array,
    A::Item: PartialEq<B::Item>,
{
    fn eq(&self, other: &ArrApi<B>) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl<T, N: Uint, A> Eq for ArrApi<A>
where
    A: Array<Item = T, Length = N>,
    T: Eq,
{
}

impl<T, A, U> PartialEq<&[U]> for ArrApi<A>
where
    T: PartialEq<U>,
    A: Array<Item = T>,
{
    fn eq(&self, &other: &&[U]) -> bool {
        self.as_slice() == other
    }
}

impl<T, U, A> PartialEq<ArrApi<A>> for &[T]
where
    T: PartialEq<U>,
    A: Array<Item = U>,
{
    fn eq(&self, other: &ArrApi<A>) -> bool {
        *self == other.as_slice()
    }
}

impl<A, T, const N: usize> PartialEq<[T; N]> for ArrApi<A>
where
    A: Array,
    A::Item: PartialEq<T>,
    crate::consts::ConstUsize<N>: crate::ToUint<ToUint = A::Length>,
{
    fn eq(&self, other: &[T; N]) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl<A, T, const N: usize> PartialEq<ArrApi<A>> for [T; N]
where
    A: Array,
    T: PartialEq<A::Item>,
    crate::consts::ConstUsize<N>: crate::ToUint<ToUint = A::Length>,
{
    fn eq(&self, other: &ArrApi<A>) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<A> PartialOrd for ArrApi<A>
where
    A: Array<Item: PartialOrd>,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}
impl<A> Ord for ArrApi<A>
where
    A: Array<Item: Ord>,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}
