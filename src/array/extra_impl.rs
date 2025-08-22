use core::mem::{ManuallyDrop, MaybeUninit};

use crate::{Uint, ops, uint, utils};

use crate::array::{extra::*, *};

// Helpers
impl<T, N: Uint, A> ArrApi<A> where A: Array<Item = T, Length = N> {}

// TODO: Capacity panics
impl<T, N: Uint, A> ArrApi<A>
where
    A: Array<Item = T, Length = N>,
{
    /// Returns the length that arrays of this type have.
    ///
    /// # Panics
    /// If the length of this type exceeds [`usize::MAX`].
    #[track_caller]
    pub const fn length() -> usize {
        check_invariants!(Self);

        arr_len::<Self>()
    }

    /// Creates a new [`ArrApi`] to wrap the given array.
    ///
    /// ```
    /// use generic_uint::array::*;
    ///
    /// assert_eq!(ArrApi::new([1, 2, 3]), [1, 2, 3]);
    /// ```
    pub const fn new(inner: A) -> Self {
        check_invariants!(Self);

        Self { inner }
    }

    /// Returns the wrapped array of this [`ArrApi`].
    ///
    /// For types that are always wrapped in [`ArrApi`] (such as [`Arr`]),
    /// the return type of this method can be named explicitly using [`ArrApiInner<Self>`](crate::array::extra::ArrApiInner).
    ///
    /// This method is primarily useful when dealing with [`ManuallyDrop`] or [`MaybeUninit`] inside of an [`ArrApi`].
    pub const fn into_inner(self) -> A {
        check_invariants!(Self);

        // SAFETY: This is a known safe way of destructuring in a `const fn`
        unsafe {
            let this = core::mem::ManuallyDrop::new(self);
            let inner = &const_util::mem::man_drop_ref(&this).inner;
            core::ptr::read(inner)
        }
    }

    /// Converts into an array with the same item and length.
    ///
    /// # Examples
    /// Retyping [`Arr`] to [`CopyArr`]:
    /// ```
    /// use generic_uint::{array::*, consts::*};
    /// let arr = Arr::<_, U5>::from_fn(|i| i * i);
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
        check_invariants!(Self);

        arr_convert(self)
    }

    /// Tries to convert into an array with the same item and length.
    ///
    /// If the lengths are the same, the operation is successful. Otherwise, the original
    /// array is returned.
    ///
    /// If you are having trouble destructuring the returned [`Result`] in a const fn, consider using
    /// functions from [`const_util::result`] or going through [`ManuallyDrop`] first.
    pub const fn try_into_arr<Dst: Array<Item = T>>(self) -> Result<Dst, Self> {
        check_invariants!(Self, Dst);

        match crate::uint::cmp::<N, Dst::Length>().is_eq() {
            // SAFETY: N == Dst::Length
            true => Ok(unsafe { arr_convert_unchecked(self) }),
            false => Err(self),
        }
    }

    /// Asserts that the length of the array is equal to the paramter and returns an new array of the
    /// adjusted type.
    ///
    /// This method is intended to be used in branches where the length has been checked to take on
    /// a certain value at runtime. If the equality is already known to hold regardless of any
    /// branches at runtime, consider using [`Self::compile_assert_len_eq`].
    ///
    /// # Panics
    /// If the length of the array is not equal to `M`
    ///
    /// # Examples
    /// ```
    /// use generic_uint::{array::*};
    /// const fn takes_generic_array(arr: impl Array<Item = i32>) -> [i32; 3] {
    ///     if size_of_val(&arr) == 12 {
    ///         ArrApi::new(arr).assert_len_eq().into_arr() // type inference
    ///     } else {
    ///         drop_items!(arr);
    ///         [0; 3]
    ///     }
    /// }
    /// assert_eq!(takes_generic_array([1, 2, 3]), [1, 2, 3]);
    /// assert_eq!(takes_generic_array([1, 2]), [0; 3]);
    /// ```
    #[track_caller]
    pub const fn assert_len_eq<M: Uint>(self) -> ArrApi<ImplArr![T; M]> {
        check_invariants!(Self);

        assert_same_arr_len::<Self, Arr<T, M>>();
        // SAFETY: Length equality was checked
        unsafe { arr_convert_unchecked::<_, Arr<T, M>>(self) }
    }

    /// Asserts in a `const` block that the length of the array is equal to the paramter and returns
    /// an new array of the adjusted type.
    ///
    /// This causes a monomorphization failure at compile time if the assertion is incorrect. As
    /// error messages are confusing when this happens multiple levels deep, you should only use
    /// this method to catch your own errors!
    ///
    /// This method is intended to be used for conversions where the length of the array can be
    /// proven to have a certain value, particularly in generic context, and that this property
    /// always holds (see examples below).
    ///
    /// # Examples
    /// Simplifying the result of splitting and concatenating:
    /// ```
    /// use generic_uint::{Uint, array::*, ops};
    ///
    /// fn identity<N: Uint, I: Uint>(arr: Arr<i32, N>) -> Arr<i32, N> {
    ///     let (lhs, rhs) = arr.split_at_uint::<I>();
    ///     let arr = lhs.concat(rhs); // Length = Add<Min<N, I>, SatSub<N, I>>, simplifies to N
    ///     arr.compile_assert_len_eq().into_arr()
    /// }
    /// fn into_lhs<N: Uint, I: Uint>(arr: Arr<i32, ops::Add<N, I>>) -> Arr<i32, I> {
    ///     let (lhs, _) = arr.split_at_uint::<I>(); // Length = Min<Add<N, I>, I>, simplifies to I
    ///     lhs.compile_assert_len_eq().into_arr()
    /// }
    /// ```
    ///
    /// Asserting commutativity:
    /// ```
    /// use generic_uint::{Uint, array::*, ops};
    ///
    /// fn commutative<A: Uint, B: Uint>(arr: Arr<i32, ops::Add<A, B>>) -> Arr<i32, ops::Add<B, A>> {
    ///     arr.compile_assert_len_eq().into_arr()
    /// }
    /// ```
    #[track_caller]
    pub const fn compile_assert_len_eq<M: Uint>(self) -> ArrApi<ImplArr![T; M]> {
        check_invariants!(Self);

        const {
            assert_same_arr_len::<Self, Arr<T, M>>();
        }
        // SAFETY: Length equality was checked
        unsafe { arr_convert_unchecked::<_, Arr<_, _>>(self) }
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
        check_invariants!(Self);

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
        check_invariants!(Self);

        arr_convert(from)
    }

    /// Equivalent to `[x; N]` with `x` of a copyable type.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::{array::*, consts::*};
    /// let arr = Arr::<_, U4>::of(1);
    /// assert_eq!(arr, [1; 4]);
    /// ```
    pub const fn of(item: T) -> Self
    where
        T: Copy,
    {
        check_invariants!(Self);

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
    ///     Arr::<_, U4>::of_const::<EmptyVecConst<i32>>(),
    ///     [const { Vec::<i32>::new() }; 4],
    /// )
    /// ```
    pub const fn of_const<C: type_const::Const<Type = T>>() -> Self {
        check_invariants!(Self);

        let mut out = ArrApi::new(MaybeUninit::uninit());
        init_fill_const::<C>(out.as_mut_slice());
        // SAFETY: All elements have been initialized
        unsafe { out.inner.assume_init() }
    }

    /// Like `<&[T] as TryInto<&[T; N]>>::try_into`, but as a const method.
    pub const fn try_from_slice(slice: &[T]) -> Result<&Self, TryFromSliceError> {
        check_invariants!(Self);

        from_slice(slice)
    }

    /// Like `<&mut [T] as TryInto<&mut [T; N]>>::try_into`, but as a const method.
    pub const fn try_from_mut_slice(slice: &mut [T]) -> Result<&mut Self, TryFromSliceError> {
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
        check_invariants!(Self, ArrSplit<N, I, T>);

        #[repr(C)]
        struct ArrSplit<N: Uint, I: Uint, T> {
            lhs: ManuallyDrop<Arr<T, ops::Min<N, I>>>,
            rhs: ManuallyDrop<Arr<T, ops::SatSub<N, I>>>,
        }
        // SAFETY: repr(C), ManuallyDrop is repr(transparent), min(N, I) + saturating_sub(N, I) = N
        unsafe impl<N: Uint, I: Uint, T> Array for ArrSplit<N, I, T> {
            type Item = ManuallyDrop<T>;
            type Length = N;
        }
        let ArrSplit::<N, I, T> { lhs, rhs } = arr_convert(ManuallyDrop::new(self));
        (ManuallyDrop::into_inner(lhs), ManuallyDrop::into_inner(rhs))
    }

    /// Concatenates two [`Array`]s.
    pub const fn concat<Rhs>(self, rhs: Rhs) -> ArrApi<ImplArr![T; ops::Add<N, Rhs::Length>]>
    where
        Rhs: Array<Item = T>,
    {
        check_invariants!(Self, ArrConcat<Self, Rhs>);

        #[repr(C)]
        struct ArrConcat<A, B>(pub A, pub B);
        // SAFETY: repr(C)
        unsafe impl<T, A: Array<Item = T>, B: Array<Item = T>> Array for ArrConcat<A, B> {
            type Item = T;
            type Length = crate::ops::Add<A::Length, B::Length>;
        }
        Arr::from_arr(ArrConcat(self, rhs))
    }

    pub const fn flatten(self) -> ArrApi<ImplArr![T::Item; ops::Mul<N, T::Length>]>
    where
        T: Array,
    {
        check_invariants!(Self, ArrFlatten<A>);

        #[repr(transparent)]
        struct ArrFlatten<A>(A);
        // SAFETY: Nested arrays are equivalent to flattened arrays with product size
        unsafe impl<A: Array<Item = T>, T: Array> Array for ArrFlatten<A> {
            type Item = T::Item;
            type Length = ops::Mul<A::Length, T::Length>;
        }
        Arr::from_arr(ArrFlatten(self))
    }

    /// Tries to turn the array into a builtin `[T; M]` array of the same size.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::{array::*, consts::*};
    ///
    /// let arr = Arr::<_, U3>::from_fn(|i| i); // type inference
    /// assert_eq!(arr.try_into_builtin_arr::<2>(), Err(arr));
    /// assert_eq!(arr.try_into_builtin_arr::<3>(), Ok([0, 1, 2]));
    /// assert_eq!(arr.try_into_builtin_arr::<4>(), Err(arr));
    /// ```
    pub const fn try_into_builtin_arr<const M: usize>(self) -> Result<[T; M], Self> {
        check_invariants!(Self);

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
    pub const fn resize_uninit<M: Uint>(self) -> ArrApi<MaybeUninit<ImplArr![T; M]>> {
        check_invariants!(Self, Arr<T, M>, MaybeUninit<Arr<T, M>>);

        // SAFETY:
        // - if N >= M, then transmuting through a union forgets `N - M` elements,
        //   which is always safe.
        // - if N <= M, then transmuting through a union fills the rest of the array with
        //   uninitialized memory, which is valid in this context.
        unsafe {
            utils::union_transmute::<
                Self, //
                ArrApi<MaybeUninit<Arr<_, _>>>,
            >(self)
        }
    }
}
