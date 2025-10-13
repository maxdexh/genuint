use crate::{Uint, array::*};

// TODO: Implement as a double ended buffer, implement APIs like as_slice, override default impls
pub struct IntoIter<A: Array> {
    pub(crate) deq: ArrDeqApi<A>,
}

impl<T, N: Uint, A> Iterator for IntoIter<A>
where
    A: Array<Item = T, Length = N>,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.deq.pop_front()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.deq.len();
        (len, Some(len))
    }
}
impl<T, N: Uint, A> DoubleEndedIterator for IntoIter<A>
where
    A: Array<Item = T, Length = N>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.deq.pop_back()
    }
}
impl<T, N: Uint, A> ExactSizeIterator for IntoIter<A> where A: Array<Item = T, Length = N> {}

#[doc = doc_no_oversized!()]
impl<A: Array> IntoIterator for ArrApi<A> {
    type Item = A::Item;
    type IntoIter = IntoIter<Self>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            deq: ArrDeqApi::new_full(self),
        }
    }
}
impl<A: Array> IntoIterator for ArrVecApi<A> {
    type Item = A::Item;
    type IntoIter = IntoIter<A>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            deq: self.into_deque(),
        }
    }
}

pub struct IntoIterDeq<A: Array> {
    pub(crate) deq: ArrDeqApi<A>,
}
impl<A: Array> Iterator for IntoIterDeq<A> {
    type Item = A::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.deq.pop_front()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.deq.len();
        (len, Some(len))
    }
}
impl<T, N: Uint, A> DoubleEndedIterator for IntoIterDeq<A>
where
    A: Array<Item = T, Length = N>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.deq.pop_back()
    }
}
impl<T, N: Uint, A> ExactSizeIterator for IntoIterDeq<A> where A: Array<Item = T, Length = N> {}
impl<A: Array> IntoIterator for ArrDeqApi<A> {
    type Item = A::Item;
    type IntoIter = IntoIterDeq<A>;
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter { deq: self }
    }
}
