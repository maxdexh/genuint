use crate::Uint;
use crate::array::{helper::*, *};

impl<T, N: Uint, A> ArrApi<A>
where
    A: Array<Item = T, Length = N>,
{
    /// Returns a slice containing the entire array.
    ///
    /// In `const` contexts, this is the only way to do this.
    ///
    /// Equivalent of [`<[T; N]>::as_slice`](array::as_slice).
    ///
    /// # Panics
    /// If `N >= usize::MAX`.
    #[track_caller]
    pub const fn as_slice(&self) -> &[T] {
        // SAFETY: `Array` to slice cast
        unsafe { &*unsize_raw(self) }
    }

    /// Returns a mutable slice containing the entire array.
    ///
    /// In `const` contexts, this is the only way to do this.
    ///
    /// Equivalent of [`<[T; N]>::as_mut_slice`](array::as_mut_slice).
    ///
    /// # Panics
    /// If `N >= usize::MAX`.
    #[track_caller]
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: `Array` to slice cast
        unsafe { &mut *unsize_raw_mut(self) }
    }

    /// Equivalent of [`<[T; N]>::each_ref`](array::each_ref).
    pub const fn each_ref(&self) -> ArrApi<impl Array<Item = &T, Length = N> + Copy> {
        let mut out = ArrVecApi::<super::CopyArr<_, _>>::new();
        let mut this = self.as_slice();
        while let [first, rest @ ..] = this {
            out.push(first);
            this = rest;
        }
        out.assert_full()
    }

    /// Equivalent of [`<[T; N]>::each_mut`](array::each_mut).
    pub const fn each_mut(&mut self) -> ArrApi<impl Array<Item = &mut T, Length = N>> {
        let mut out = ArrVec::new();
        let mut this = self.as_mut_slice();
        while let [first, rest @ ..] = this {
            out.push(first);
            this = rest;
        }
        out.assert_full()
    }

    /// Equivalent of [`<[T; N]>::map`](array::map).
    pub fn map<F, U>(self, mut f: F) -> ArrApi<impl Array<Item = U, Length = N>>
    where
        F: FnMut(T) -> U,
    {
        let mut out = ArrVec::new();
        let mut inp = ArrDeqApi::new_full(self);
        while let Some(first) = inp.pop_front() {
            out.push(f(first));
        }
        out.assert_full()
    }
}

impl<A> Clone for ArrApi<A>
where
    A: Array<Item: Clone>,
{
    fn clone(&self) -> Self {
        self.each_ref().map(Clone::clone).retype()
    }
}
impl<A> Copy for ArrApi<A> where A: Array<Item: Copy> + Copy {}
impl<A> Default for ArrApi<A>
where
    A: Array<Item: Default>,
{
    fn default() -> Self {
        Self::from_fn(|_| Default::default())
    }
}
impl<A> type_const::DefaultConst for ArrApi<A>
where
    A: Array<Item: type_const::DefaultConst>,
{
    const DEFAULT: Self = Self::of_const::<type_const::DefaultOf<A::Item>>();
}
impl<A> core::fmt::Debug for ArrApi<A>
where
    A: Array<Item: core::fmt::Debug>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self.as_slice())
    }
}
impl<A> core::hash::Hash for ArrApi<A>
where
    A: Array<Item: core::hash::Hash>,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state)
    }
}
impl<T, A, I, O> core::ops::Index<I> for ArrApi<A>
where
    [T]: core::ops::Index<I, Output = O>,
    A: Array<Item = T>,
    O: ?Sized,
{
    type Output = O;
    fn index(&self, index: I) -> &Self::Output {
        &self.as_slice()[index]
    }
}
impl<T, A, I> core::ops::IndexMut<I> for ArrApi<A>
where
    [T]: core::ops::IndexMut<I>,
    A: Array<Item = T>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.as_mut_slice()[index]
    }
}
