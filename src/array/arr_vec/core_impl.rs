use crate::array::*;

impl<A: Array, U> PartialEq<[U]> for ArrVecApi<A>
where
    A::Item: PartialEq<U>,
{
    fn eq(&self, other: &[U]) -> bool {
        self.as_slice() == other
    }
}

impl<A: Array, T> PartialEq<ArrVecApi<A>> for [T]
where
    T: PartialEq<A::Item>,
{
    fn eq(&self, other: &ArrVecApi<A>) -> bool {
        self == other.as_slice()
    }
}

impl<A: Array, U, const N: usize> PartialEq<[U; N]> for ArrVecApi<A>
where
    A::Item: PartialEq<U>,
{
    fn eq(&self, other: &[U; N]) -> bool {
        self.as_slice() == other
    }
}

impl<A: Array, T, const N: usize> PartialEq<ArrVecApi<A>> for [T; N]
where
    T: PartialEq<A::Item>,
{
    fn eq(&self, other: &ArrVecApi<A>) -> bool {
        self == other.as_slice()
    }
}

impl<A: Array> core::fmt::Debug for ArrVecApi<A>
where
    A::Item: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.as_slice())
    }
}

impl<A: Array> Default for ArrVecApi<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Array<Item = T>, T> FromIterator<T> for ArrVecApi<A> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut this = Self::new();
        this.extend(iter);
        this
    }
}

impl<A: Array<Item = T>, T> Extend<T> for ArrVecApi<A> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let mut iter = iter.into_iter();
        while !self.is_full()
            && let Some(next) = iter.next()
        {
            self.push(next);
        }
    }
}
