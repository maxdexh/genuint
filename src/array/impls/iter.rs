use crate::{Uint, array::*};

// TODO: There are some APIs missing here
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

/// Panics for `A::Length > usize::MAX`.
impl<A: Array> IntoIterator for ArrApi<A> {
    type Item = A::Item;
    type IntoIter = IntoIter<Self>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            deq: ArrDeqApi::new_full(self),
        }
    }
}
/// Panics for `A::Length > usize::MAX`.
impl<'a, A: Array> IntoIterator for &'a ArrApi<A> {
    type Item = &'a A::Item;
    type IntoIter = <&'a [A::Item] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}
/// Panics for `A::Length > usize::MAX`.
impl<'a, A: Array> IntoIterator for &'a mut ArrApi<A> {
    type Item = &'a mut A::Item;
    type IntoIter = <&'a mut [A::Item] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.as_mut_slice().iter_mut()
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
impl<'a, A: Array> IntoIterator for &'a ArrVecApi<A> {
    type Item = &'a A::Item;
    type IntoIter = <&'a [A::Item] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}
impl<'a, A: Array> IntoIterator for &'a mut ArrVecApi<A> {
    type Item = &'a mut A::Item;
    type IntoIter = <&'a mut [A::Item] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.as_mut_slice().iter_mut()
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
