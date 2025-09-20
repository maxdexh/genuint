use crate::array::{ArrDeqApi, Array};

impl<A: Array, U> PartialEq<[U]> for ArrDeqApi<A>
where
    A::Item: PartialEq<U>,
{
    fn eq(&self, other: &[U]) -> bool {
        self.len() == other.len() && {
            let (lhs, rhs) = self.as_slices();
            let (lhso, rhso) = other.split_at(lhs.len());
            lhs == lhso && rhs == rhso
        }
    }
}

impl<A: Array, T> PartialEq<ArrDeqApi<A>> for [T]
where
    T: PartialEq<A::Item>,
{
    fn eq(&self, other: &ArrDeqApi<A>) -> bool {
        self.len() == other.len() && {
            let (lhs, rhs) = other.as_slices();
            let (lhso, rhso) = self.split_at(lhs.len());
            lhso == lhs && rhso == rhs
        }
    }
}

impl<A: Array, U, const N: usize> PartialEq<[U; N]> for ArrDeqApi<A>
where
    A::Item: PartialEq<U>,
{
    fn eq(&self, other: &[U; N]) -> bool {
        self == other.as_slice()
    }
}

impl<A: Array, T, const N: usize> PartialEq<ArrDeqApi<A>> for [T; N]
where
    T: PartialEq<A::Item>,
{
    fn eq(&self, other: &ArrDeqApi<A>) -> bool {
        self.as_slice() == other
    }
}

impl<A: Array> core::fmt::Debug for ArrDeqApi<A>
where
    A::Item: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (lhs, rhs) = self.as_slices();
        f.debug_list().entries(lhs).entries(rhs).finish()
    }
}

impl<A: Array> Default for ArrDeqApi<A> {
    fn default() -> Self {
        Self::new()
    }
}
