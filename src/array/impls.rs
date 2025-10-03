use core::mem::MaybeUninit;

use crate::{
    Uint,
    array::{helper::*, *},
    condty::CondResult,
    internals::ArraySealed,
    ops,
};

// SAFETY: By definition
unsafe impl<T, const N: usize> Array for [T; N]
where
    crate::consts::ConstUsize<N>: crate::ToUint,
{
    type Item = T;
    type Length = crate::uint::From<crate::consts::ConstUsize<N>>;
}
impl<T, const N: usize> ArraySealed for [T; N] where crate::consts::ConstUsize<N>: crate::ToUint {}

// SAFETY: MaybeUninit<[T; N]> is equivalent to [MaybeUninit<T>; N]
unsafe impl<A: Array> Array for core::mem::MaybeUninit<A> {
    type Item = core::mem::MaybeUninit<A::Item>;
    type Length = A::Length;
}
impl<A: Array> ArraySealed for core::mem::MaybeUninit<A> {}

// SAFETY: repr(transparent)
unsafe impl<A: Array> Array for ArrApi<A> {
    type Item = A::Item;
    type Length = A::Length;
}
impl<A: Array> ArraySealed for ArrApi<A> {}

// SAFETY: `repr(C)` results in the arrays being placed next to each other in memory
// in accordance with array layout
unsafe impl<T, A: Array<Item = T>, B: Array<Item = T>> Array for Concat<A, B> {
    type Item = T;
    type Length = crate::uint::From<crate::ops::Add<A::Length, B::Length>>;
}
impl<T, A: Array<Item = T>, B: Array<Item = T>> ArraySealed for Concat<A, B> {}

// SAFETY: repr(transparent), `[[T; M]; N]` is equivalent to `[T; M * N]`
unsafe impl<A: Array<Item = B>, B: Array> Array for Flatten<A> {
    type Item = B::Item;
    type Length = crate::uint::From<crate::ops::Mul<A::Length, B::Length>>;
}
impl<A: Array<Item = B>, B: Array> ArraySealed for Flatten<A> {}

mod base;
mod cmp;
mod convert_impl;
mod core_impl;
mod iter;
mod tuple_convert;

impl<T, N: Uint, A> ArrApi<A>
where
    A: Array<Item = T, Length = N>,
{
    /// Alias for `Self { inner }`
    pub const fn new(inner: A) -> Self {
        Self { inner }
    }

    /// Returns the length that arrays of this type have.
    ///
    /// # Panics
    /// If the length of this array exceeds [`usize::MAX`].
    #[track_caller]
    pub const fn length() -> usize {
        arr_len::<Self>()
    }

    /// Returns the wrapped array of this [`ArrApi`].
    ///
    /// This method is primarily useful when dealing with nested [`ArrApi`]s
    /// or extracting a type with its own api (such as builtin arrays or [`Concat`])
    /// inside of an [`ArrApi`] when moving out of [`self.inner`](Self::inner) is
    /// not possible, e.g. in `const` contexts with generics.
    pub const fn into_inner(self) -> A {
        self.retype()
    }

    /// [`core::array::from_fn`], but as a method.
    ///
    /// # Panics
    /// If `N > usize::MAX`. The generating function is not called in this case.
    ///
    /// # Examples
    /// ```
    /// use genuint::{array::*, small::*};
    /// let arr = Arr::<_, _4>::from_fn(|i| i * i);
    /// assert_eq!(arr, [0, 1, 4, 9]);
    /// ```
    #[track_caller]
    pub fn from_fn<F: FnMut(usize) -> T>(mut f: F) -> Self {
        let mut out = ArrVecApi::new();
        while !out.is_full() {
            out.push(f(out.len()));
        }
        out.assert_full()
    }

    /// Converts into an array with the same item and length.
    ///
    /// # Examples
    /// Retyping [`Arr`] to [`CopyArr`]:
    /// ```
    /// use genuint::{array::*, small::*};
    /// let arr = Arr::<_, _5>::from_fn(|i| i * i);
    /// let converted: CopyArr<_, _> = arr.retype();
    /// let converted_copy = converted;
    /// assert_eq!(converted, converted_copy);
    /// ```
    ///
    /// Converting a small `ArrApi` into a builtin array:
    /// ```
    /// use genuint::{array::*, consts::*};
    /// let arr = Arr::from_fn(|i| i * i);
    /// let builtin_arr: [_; 5] = arr.retype();
    /// assert_eq!(arr, builtin_arr);
    /// ```
    pub const fn retype<Dst>(self) -> Dst
    where
        Dst: Array<Item = T, Length = N>,
    {
        convert::retype(self)
    }

    /// Tries to convert into an array with the same item and length.
    ///
    /// If the lengths are the same, the operation is successful. Otherwise, the original
    /// array is returned.
    ///
    /// If you are having trouble destructuring the returned [`Result`] in a const fn, consider using
    /// functions from [`const_util::result`].
    pub const fn try_retype<Dst: Array<Item = T>>(
        self,
    ) -> CondResult<ops::Eq<N, Dst::Length>, Dst, Self> {
        convert::try_retype(self)
    }

    /// Concatenates the inner array with `rhs` via [`Concat`].
    pub const fn concat<Rhs>(self, rhs: Rhs) -> ArrApi<Concat<A, Rhs>>
    where
        Rhs: Array<Item = T>,
    {
        ArrApi::new(Concat(self.into_inner(), rhs))
    }

    /// Flattens the inner array via [`Flatten`].
    pub const fn flatten(self) -> ArrApi<Flatten<A>>
    where
        T: Array,
    {
        ArrApi::new(Flatten(self.into_inner()))
    }

    /// Creates an array of uninit.
    ///
    /// This method is defined on `ArrApi<A>` and creates `ArrApi<MaybeUninit<A>>`.
    /// This is so that [`Arr::uninit()`] infers the return type to
    /// `ArrApi<MaybeUninit<ArrInner<_, _>>>`.
    pub const fn uninit() -> ArrApi<MaybeUninit<A>> {
        ArrApi::new(MaybeUninit::uninit())
    }
}
