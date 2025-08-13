use crate::{
    Uint,
    array::{ArrApi, Array, extra::*},
};

pub struct IntoIter<T, N: Uint> {
    deq: CanonDeq<T, N>,
}

impl<T, N: Uint> Iterator for IntoIter<T, N> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.deq.pop_front()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.deq.len();
        (len, Some(len))
    }
}
impl<T, N: Uint> DoubleEndedIterator for IntoIter<T, N> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.deq.pop_back()
    }
}
impl<T, N: Uint> ExactSizeIterator for IntoIter<T, N> {}

impl<A: Array> IntoIterator for ArrApi<A> {
    type Item = A::Item;
    type IntoIter = IntoIter<A::Item, A::Length>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            deq: CanonDeq::full(self.retype()),
        }
    }
}

impl<'a, A: Array> IntoIterator for &'a ArrApi<A> {
    type Item = &'a A::Item;
    type IntoIter = <&'a [A::Item] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}
impl<'a, A: Array> IntoIterator for &'a mut ArrApi<A> {
    type Item = &'a mut A::Item;
    type IntoIter = <&'a mut [A::Item] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.as_mut_slice().iter_mut()
    }
}
