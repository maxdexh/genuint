use crate::array::*;

impl<A: Array, U> PartialEq<[U]> for ArrVec<A>
where
    A::Item: PartialEq<U>,
{
    fn eq(&self, other: &[U]) -> bool {
        self.as_slice() == other
    }
}

impl<A: Array, T> PartialEq<ArrVec<A>> for [T]
where
    T: PartialEq<A::Item>,
{
    fn eq(&self, other: &ArrVec<A>) -> bool {
        self == other.as_slice()
    }
}

impl<A: Array, U, const N: usize> PartialEq<[U; N]> for ArrVec<A>
where
    A::Item: PartialEq<U>,
{
    fn eq(&self, other: &[U; N]) -> bool {
        self.as_slice() == other
    }
}

impl<A: Array, T, const N: usize> PartialEq<ArrVec<A>> for [T; N]
where
    T: PartialEq<A::Item>,
{
    fn eq(&self, other: &ArrVec<A>) -> bool {
        self == other.as_slice()
    }
}

impl<A: Array> core::fmt::Debug for ArrVec<A>
where
    A::Item: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.as_slice())
    }
}
