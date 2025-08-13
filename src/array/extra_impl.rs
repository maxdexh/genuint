use core::mem::{ManuallyDrop, MaybeUninit};

use crate::{Uint, ops};

use crate::array::{ArrApi, ArrVec, Array, extra::*};

// Helpers
impl<T, N: Uint, A> ArrApi<A>
where
    A: Array<Item = T, Length = N>,
{
    pub(crate) const fn length() -> usize {
        arr_len::<Self>()
    }
}

impl<T, N: Uint, A> ArrApi<A>
where
    A: Array<Item = T, Length = N>,
{
    /// Creates a new [`ArrApi`] to wrap the given array.
    pub const fn new(inner: A) -> Self {
        Self(inner, core::marker::PhantomData)
    }

    /// Returns the wrapped array of this [`ArrApi`].
    ///
    /// For types that are always wrapped in [`ArrApi`] (such as [`Arr`](crate::array::Arr)),
    /// the return type of this method can be named explicitly using [`ArrApiInner<Self>`](crate::array::extra::ArrApiInner).
    ///
    /// This method is primarily useful when dealing with [`ManuallyDrop`] or [`MaybeUninit`] inside of an [`ArrApi`].
    pub const fn into_inner(self) -> A {
        // SAFETY: This is a known safe way of destructuring in a `const fn`
        unsafe {
            let this = core::mem::ManuallyDrop::new(self);
            let inner = &const_util::mem::man_drop_ref(&this).0;
            core::ptr::read(inner)
        }
    }

    /// Gets a reference to the wrapped array of this [`ArrApi`].
    ///
    /// This method is primarily useful when dealing with [`ManuallyDrop`] or [`MaybeUninit`] inside of an [`ArrApi`].
    ///
    /// # Examples
    /// ```
    /// ```
    pub const fn as_inner(&self) -> &A {
        &self.0
    }

    /// Gets a mutable reference to the wrapped array of this [`ArrApi`].
    ///
    /// This method is primarily useful when dealing with [`ManuallyDrop`] or [`MaybeUninit`] inside of an [`ArrApi`].
    ///
    /// # Examples
    /// Writing into a [`ManuallyDrop<Arr<_, _>>`] wrapped in an [`ArrApi`]:
    /// ```
    /// use core::mem::ManuallyDrop;
    /// use generic_uint::array::*;
    /// let mut arr = ArrApi::new(ManuallyDrop::new([1, 2, 3, 4]));
    /// **arr.as_mut_inner() = [1; 4];
    /// assert_eq!(arr, [ManuallyDrop::new(1); 4]);
    /// ```
    pub const fn as_mut_inner(&mut self) -> &mut A {
        &mut self.0
    }

    /// Converts into an array with the same item and length.
    ///
    /// # Examples
    /// Retyping [`Arr`](crate::array::Arr) to [`CopyArr`](crate::array::CopyArr):
    /// ```
    /// use generic_uint::{array::*, consts::*};
    /// let arr = Arr::<_, U5>::from_fn(|i| i * i);
    /// let into_arrd: CopyArr<_, _> = arr.into_arr();
    /// let into_arrd_copy = into_arrd;
    /// assert_eq!(into_arrd_copy, into_arrd);
    /// ```
    ///
    /// Converting a small `ArrApi` into a builtin array:
    /// ```
    /// use generic_uint::{array::*, consts::*};
    /// let arr = Arr::from_fn(|i| i * i);
    /// let builtin_arr: [_; 5] = arr.into_arr();
    /// assert_eq!(arr, builtin_arr);
    /// ```
    ///
    /// Transposing [`ManuallyDrop`] or [`MaybeUninit`]:
    /// ```
    /// use generic_uint::array::*;
    /// use core::mem::ManuallyDrop;
    /// let arr: ArrApi<ManuallyDrop<[i32; 3]>> = ArrApi::new(ManuallyDrop::new([1, 2, 3]));
    /// let transposed: ArrApi<[ManuallyDrop<i32>; 3]> = arr.into_arr();
    /// assert_eq!(arr, transposed);
    /// let back: ArrApi<ManuallyDrop<[i32; 3]>> = transposed.into_arr();
    /// assert_eq!(arr, back);
    /// ```
    pub const fn into_arr<Dst: Array<Item = T, Length = N>>(self) -> Dst {
        arr_convert(self)
    }

    /// Tries to convert into an array with the same item and length.
    ///
    /// If the lengths are the same, the operation is successful. Otherwise, the original
    /// array is returned.
    pub const fn try_into_arr<Dst: Array<Item = T>>(self) -> Result<Dst, Self> {
        match crate::uint::cmp::<N, Dst::Length>().is_eq() {
            true => Ok(unsafe { into_arr_unchecked(self) }),
            false => Err(self),
        }
    }

    /// [`core::array::from_fn`], but as a method.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::{array::*, consts::*};
    /// let arr = Arr::<_, U4>::from_fn(|i| i * i);
    /// assert_eq!(arr, [0, 1, 4, 9]);
    /// ```
    pub fn from_fn<F: FnMut(usize) -> T>(mut f: F) -> Self {
        let mut out = ArrVec::new();
        while !out.is_full() {
            out.push(f(out.len()));
        }
        out.into_full()
    }

    /// The same as `ArrApi::new(from).into_arr::<Self>()`.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    /// let arr = Arr::from_arr([1, 2, 3]);
    /// assert_eq!(arr, [1, 2, 3]);
    /// ```
    pub const fn from_arr<Src: Array<Item = T, Length = N>>(from: Src) -> Self {
        arr_convert(from)
    }

    /// Equivalent to `[x; N]` with `x` of a copyable type.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::{array::*, consts::*};
    /// let arr = Arr::<_, U4>::of_copy(1);
    /// assert_eq!(arr, [1; 4]);
    /// ```
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

    /// Equivalent to `[CONST; N]` (or `[const { expr }; N]`).
    ///
    /// # Examples
    /// ```
    /// extern crate type_const;
    /// struct EmptyVecConst<T>(T);
    /// impl<T> type_const::Const for EmptyVecConst<T> {
    ///     type Type = Vec<T>;
    ///     const VALUE: Self::Type = Vec::new();
    /// }
    ///
    /// use generic_uint::{array::*, consts::*};
    /// assert_eq!(
    ///     Arr::<_, U4>::of_const::<EmptyVecConst<i32>>(),
    ///     [const { Vec::<i32>::new() }; 4],
    /// )
    /// ```
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

    /// Like `<&[T] as TryInto<&[T; N]>>::try_into`, but as a const method.
    pub const fn try_from_slice(slice: &[T]) -> Result<&Self, TryFromSliceError> {
        from_slice(slice)
    }

    /// Like `<&mut [T] as TryInto<&mut [T; N]>>::try_into`, but as a const method.
    pub const fn try_from_mut_slice(slice: &mut [T]) -> Result<&mut Self, TryFromSliceError> {
        from_mut_slice(slice)
    }

    /// Wraps the wrapped array in [`ManuallyDrop`].
    ///
    /// Because of the [`Array`] implementation for [`ManuallyDrop<impl Array>`],
    /// the returned value implements [`Array<Item = ManuallyDrop<T>>`].
    pub const fn into_manually_drop(self) -> ArrApi<ManuallyDrop<A>> {
        ArrApi::new(ManuallyDrop::new(self.into_inner()))
    }

    /// Wraps the wrapped array in [`ManuallyDrop`].
    ///
    /// Because of the [`Array`] implementation for [`MaybeUninit<impl Array>`],
    /// the returned value implements [`Array<Item = MaybeUninit<T>>`].
    pub const fn into_maybe_uninit(self) -> ArrApi<MaybeUninit<A>> {
        ArrApi::new(MaybeUninit::new(self.into_inner()))
    }

    /// Splits an owned array at a [`Uint`] position.
    ///
    /// The output is a pair of arrays with lengths `min(N, I)` and `saturating_sub(N, I)`.
    /// Because the sum of these operations can be proven to always be `N`, this never loses any
    /// elements. As a result, the method behaves as follows:
    /// - If `I <= N`, returns arrays with lengths `I` and `N - I`.
    /// - If `I >= N`, returns arrays with lengths `N` and `0`.
    /// - Using [`concat`](Self::concat) on the split arrays gives back the original.
    #[allow(clippy::type_complexity, reason = "Not much we can do here")]
    pub const fn split_at_uint<I: Uint>(
        self,
    ) -> (
        ArrApi<ImplArr![T; ops::Min<N, I>]>,
        ArrApi<ImplArr![T; ops::SatSub<N, I>]>,
    ) {
        #[repr(C)]
        struct ArrSplit<N: Uint, I: Uint, T> {
            lhs: ManuallyDrop<CanonArr<T, ops::Min<N, I>>>,
            rhs: ManuallyDrop<CanonArr<T, ops::SatSub<N, I>>>,
        }
        // SAFETY: repr(C), ManuallyDrop is repr(transparent), min(N, I) + saturating_sub(N, I) = N
        unsafe impl<N: Uint, I: Uint, T> Array for ArrSplit<N, I, T> {
            type Item = ManuallyDrop<T>;
            type Length = N;
        }
        let ArrSplit { lhs, rhs } = arr_convert(ManuallyDrop::new(self));
        (ManuallyDrop::into_inner(lhs), ManuallyDrop::into_inner(rhs))
    }

    /// Concatenates two [`Array`]s.
    pub const fn concat<Rhs>(self, rhs: Rhs) -> ArrApi<ImplArr![T; ops::Add<N, Rhs::Length>]>
    where
        Rhs: Array<Item = T>,
    {
        #[repr(C)]
        struct ArrConcat<A, B>(pub A, pub B);
        // SAFETY: repr(C)
        unsafe impl<T, A: Array<Item = T>, B: Array<Item = T>> Array for ArrConcat<A, B> {
            type Item = T;
            type Length = crate::ops::Add<A::Length, B::Length>;
        }
        into_canon(ArrConcat(self, rhs))
    }

    pub const fn flatten(self) -> ArrApi<ImplArr![T::Item; ops::Mul<N, T::Length>]>
    where
        T: Array,
    {
        #[repr(transparent)]
        struct ArrFlatten<A>(A);
        // SAFETY: Nested arrays are equivalent to flattened arrays with product size
        unsafe impl<A: Array<Item = T>, T: Array> Array for ArrFlatten<A> {
            type Item = T::Item;
            type Length = ops::Mul<A::Length, T::Length>;
        }
        into_canon(ArrFlatten(self))
    }
}

impl<T, N: Uint, A> ArrApi<A>
where
    A: Array<Item = MaybeUninit<T>, Length = N>,
{
    pub const fn resize_uninit<M: Uint>(self) -> ArrApi<MaybeUninit<ImplArr![T; M]>> {
        unsafe { crate::utils::union_transmute::<_, ArrApi<MaybeUninit<CanonArr<_, _>>>>(self) }
    }
}
