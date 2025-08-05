use core::mem::MaybeUninit;

use crate::Uint;

use crate::array::{ArrApi, ArrVec, Array, TryFromSliceError, arr_utils::*};

impl<T, N: Uint, A> ArrApi<A>
where
    A: Array<Item = T, Length = N>,
{
    pub(crate) const fn length() -> usize {
        arr_len::<Self>()
    }
    pub const fn new(inner: A) -> Self {
        Self(inner, core::marker::PhantomData)
    }
    pub const fn into_inner(self) -> A {
        // SAFETY: This is a known safe way of destructuring in a `const fn`
        unsafe {
            core::ptr::read(&const_util::mem::man_drop_ref(&core::mem::ManuallyDrop::new(self)).0)
        }
    }
    pub const fn as_inner(&self) -> &A {
        &self.0
    }
    pub const fn as_mut_inner(&mut self) -> &mut A {
        &mut self.0
    }
    pub const fn into_arr<B: Array<Item = T, Length = N>>(self) -> B {
        arr_conv(self)
    }
    pub fn from_fn<F: FnMut(usize) -> T>(mut f: F) -> Self {
        let mut out = ArrVec::new();
        while !out.is_full() {
            out.push(f(out.len()));
        }
        out.into_full()
    }

    pub const fn of_copy(item: T) -> Self
    where
        T: Copy,
    {
        let mut out = ArrApi::new(MaybeUninit::uninit());
        let mut buf = out.as_mut_slice();
        while let [first, rest @ ..] = buf {
            *first = MaybeUninit::new(item);
            buf = rest;
        }
        unsafe { out.into_inner().assume_init() }
    }
    pub const fn of_const<C: type_const::Const<Type = T>>() -> Self {
        // There is nothing here that could panic, so we don't need a drop guard
        let mut out = ArrApi::new(MaybeUninit::uninit());
        let mut buf = out.as_mut_slice();
        while let [first, rest @ ..] = buf {
            *first = MaybeUninit::new(C::VALUE);
            buf = rest;
        }
        unsafe { out.into_inner().assume_init() }
    }

    pub const fn from_slice(slice: &[T]) -> Result<&Self, TryFromSliceError> {
        arr_from_slice(slice)
    }

    pub const fn from_mut_slice(slice: &mut [T]) -> Result<&mut Self, TryFromSliceError> {
        arr_from_mut_slice(slice)
    }
}
