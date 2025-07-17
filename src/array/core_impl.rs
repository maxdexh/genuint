use crate::Uint;

use super::{ArrApi, ArrDeq, ArrVec, Array, arr_utils::*};

impl<A: Array<Item = T, Length = N>, T, N: Uint> ArrApi<A> {
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        let len = self.check_len();
        unsafe { core::slice::from_raw_parts_mut(&mut *(&raw mut *self).cast(), len) }
    }
    pub const fn as_slice(&self) -> &[T] {
        let len = self.check_len();
        unsafe { core::slice::from_raw_parts(&*(&raw const *self).cast(), len) }
    }
    pub const fn each_mut(&mut self) -> impl Array<Item = &mut T, Length = N> {
        let mut out = CanonVec::new();
        let mut this = self.as_mut_slice();
        while let [first, rest @ ..] = this {
            out.push_alt(first);
            this = rest;
        }
        out.into_full()
    }
    pub const fn each_ref(&self) -> impl Array<Item = &T, Length = N> + Copy {
        let mut out = ArrVec::<super::CopyArr<_, _>>::new();
        let mut this = self.as_slice();
        while let [first, rest @ ..] = this {
            out.push_alt(first);
            this = rest;
        }
        out.into_full()
    }
    pub fn map<F, U>(self, mut f: F) -> impl Array<Item = U, Length = N>
    where
        F: FnMut(T) -> U,
    {
        let mut out = CanonVec::new();
        let mut inp = ArrDeq::full(self);
        while let Some(first) = inp.pop_front() {
            out.push_alt(f(first));
        }
        out.into_full()
    }
}
