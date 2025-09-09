use core::mem::{ManuallyDrop, MaybeUninit};
use core::ptr;

use crate::tern::TernRes;
use crate::{Uint, ops, uint, utils};

use crate::array::{convert::*, extra::*, helper::*, *};

// FIXME: Capacity panics need to be documented
// TODO: Replace Self -> ImplArr with B -> Self
impl<T, N: Uint, A> ArrApi<A>
where
    A: Array<Item = T, Length = N>,
{
    /// Alias for `Self { inner }`
    pub const fn new(inner: A) -> Self {
        Self { inner }
    }

    pub const fn from_inner_ref(inner: &A) -> &Self {
        // SAFETY: repr(transparent)
        unsafe { &*ptr::from_ref(inner).cast() }
    }

    pub const fn from_inner_mut(inner: &mut A) -> &mut Self {
        // SAFETY: repr(transparent)
        unsafe { &mut *ptr::from_mut(inner).cast() }
    }

    /// Returns the length that arrays of this type have.
    ///
    /// # Panics
    /// If the length of this type exceeds [`usize::MAX`].
    #[track_caller]
    pub const fn length() -> usize {
        arr_len::<Self>()
    }

    /// Returns the wrapped array of this [`ArrApi`].
    ///
    /// For types that are always wrapped in [`ArrApi`] (such as [`Arr`]),
    /// the return type of this method can be named explicitly using [`ArrApiInner<Self>`](crate::array::extra::ArrApiInner).
    ///
    /// This method is primarily useful when dealing with [`ManuallyDrop`] or [`MaybeUninit`] inside of an [`ArrApi`].
    pub const fn into_inner(self) -> A {
        // SAFETY: `self` is passed by value and can be destructed by read
        unsafe {
            crate::utils::destruct_read!(Self, { inner: inner }, self);
            inner
        }
    }

    /// Converts into an array with the same item and length.
    ///
    /// # Examples
    /// Retyping [`Arr`] to [`CopyArr`]:
    /// ```
    /// use generic_uint::{array::*, consts::*};
    /// let arr = Arr::<_, _5>::from_fn(|i| i * i);
    /// let converted: CopyArr<_, _> = arr.into_arr();
    /// let converted_copy = converted;
    /// assert_eq!(converted, converted_copy);
    /// ```
    ///
    /// Converting a small `ArrApi` into a builtin array:
    /// ```
    /// use generic_uint::{array::*, consts::*};
    /// let arr = Arr::from_fn(|i| i * i);
    /// let builtin_arr: [_; 5] = arr.into_arr();
    /// assert_eq!(arr, builtin_arr);
    /// ```
    pub const fn retype<Dst: Array<Item = T, Length = N>>(self) -> Dst {
        retype(self)
    }

    /// Tries to convert into an array with the same item and length.
    ///
    /// If the lengths are the same, the operation is successful. Otherwise, the original
    /// array is returned.
    ///
    /// If you are having trouble destructuring the returned [`Result`] in a const fn, consider using
    /// functions from [`const_util::result`] or going through [`ManuallyDrop`] first.
    pub const fn try_retype<Dst: Array<Item = T>>(
        self,
    ) -> TernRes<ops::Eq<N, Dst::Length>, Dst, Self> {
        try_retype(self)
    }

    /// [`core::array::from_fn`], but as a method.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::{array::*, consts::*};
    /// let arr = Arr::<_, _4>::from_fn(|i| i * i);
    /// assert_eq!(arr, [0, 1, 4, 9]);
    /// ```
    pub fn from_fn<F: FnMut(usize) -> T>(mut f: F) -> Self {
        let mut out = ArrVecApi::new();
        while !out.is_full() {
            out.push(f(out.len()));
        }
        out.assert_full()
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
        retype(from)
    }

    /// Equivalent to `[x; N]` with `x` of a copyable type.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::{array::*, consts::*};
    /// let arr = Arr::<_, _4>::of(1);
    /// assert_eq!(arr, [1; 4]);
    /// ```
    pub const fn of(item: T) -> Self
    where
        T: Copy,
    {
        let mut out = ArrApi::new(MaybeUninit::uninit());
        init_fill(out.as_mut_slice(), item);
        // SAFETY: All elements have been initialized
        unsafe { out.inner.assume_init() }
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
    ///     Arr::<_, _4>::of_const::<EmptyVecConst<i32>>(),
    ///     [const { Vec::<i32>::new() }; 4],
    /// )
    /// ```
    pub const fn of_const<C: type_const::Const<Type = T>>() -> Self {
        let mut out = ArrApi::new(MaybeUninit::uninit());
        init_fill_const::<C>(out.as_mut_slice());
        // SAFETY: All elements have been initialized
        unsafe { out.inner.assume_init() }
    }

    /// Like `<&[T] as TryInto<&[T; N]>>::try_into`, but as a const method.
    pub const fn from_slice(slice: &[T]) -> Option<&Self> {
        from_ref_slice(slice)
    }

    /// Like `<&mut [T] as TryInto<&mut [T; N]>>::try_into`, but as a const method.
    pub const fn from_mut_slice(slice: &mut [T]) -> Option<&mut Self> {
        from_mut_slice(slice)
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
        let (lhs, rhs) = self
            .try_retype::<Concat<Arr<_, _>, Arr<_, _>>>()
            .unwrap()
            .into_man_drop();
        (ManuallyDrop::into_inner(lhs), ManuallyDrop::into_inner(rhs))
    }

    /// Concatenates two [`Array`]s.
    pub const fn concat<Rhs>(self, rhs: Rhs) -> ArrApi<Concat<A, Rhs>>
    where
        Rhs: Array<Item = T>,
    {
        ArrApi::new(Concat(self.into_inner(), rhs))
    }

    pub const fn flatten(self) -> ArrApi<Flatten<A>>
    where
        T: Array,
    {
        ArrApi::new(Flatten(self.into_inner()))
    }

    /// Tries to turn the array into a builtin `[T; M]` array of the same size.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::{array::*, consts::*};
    ///
    /// let arr = Arr::<_, _3>::from_fn(|i| i); // type inference
    /// assert_eq!(arr.try_into_builtin_arr::<2>(), Err(arr));
    /// assert_eq!(arr.try_into_builtin_arr::<3>(), Ok([0, 1, 2]));
    /// assert_eq!(arr.try_into_builtin_arr::<4>(), Err(arr));
    /// ```
    pub const fn try_into_builtin_arr<const M: usize>(self) -> Result<[T; M], Self> {
        if let Some(n) = uint::to_usize::<N>()
            && n == M
        {
            // SAFETY: M == N
            Ok(unsafe { arr_to_builtin_unchecked(self) })
        } else {
            Err(self)
        }
    }

    /// Resizes the array.
    ///
    /// If the new length is larger than the old length, the remaining elements will be filled with
    /// `item`. Otherwise, the array will be truncated and the extra elements discarded.
    pub(crate) const fn resize_with_fill<M: Uint>(self, item: T) -> ArrApi<ImplArr![T; M]>
    where
        T: Copy,
    {
        let mut out = ArrApi::new(MaybeUninit::new(self)).resize_uninit();
        if let Some((_, uninit)) = out.as_mut_slice().split_at_mut_checked(Self::length()) {
            init_fill(uninit, item);
        }
        // SAFETY: The first `Self::legnth()` items are already init. `init_fill` inits the rest.
        unsafe { ArrApi::new(out.inner.assume_init()) }
    }
}

impl<T, N: Uint, A> ArrApi<A>
where
    A: Array<Item = MaybeUninit<T>, Length = N>,
{
    pub const fn uninit() -> Self {
        // SAFETY: Shortcut for the already safe `arr_convert(MaybeUninit::uninit())`
        #[allow(clippy::uninit_assumed_init)]
        unsafe {
            MaybeUninit::uninit().assume_init()
        }
    }

    pub const fn resize_uninit<M: Uint>(
        self,
    ) -> ArrApi<MaybeUninit<impl Array<Item = T, Length = M>>> {
        // SAFETY:
        // - if N >= M, then transmuting through a union forgets `N - M` elements,
        //   which is always safe.
        // - if N <= M, then transmuting through a union fills the rest of the array with
        //   uninitialized memory, which is valid in this context.
        unsafe {
            utils::union_transmute::<
                Self, //
                ArrApi<MaybeUninit<Arr<T, M>>>,
            >(self)
        }
    }
}
