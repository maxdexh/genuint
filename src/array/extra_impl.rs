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
    ///
    ///
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
    pub const fn as_inner(&self) -> &A {
        &self.0
    }

    /// Gets a mutable reference to the wrapped array of this [`ArrApi`].
    ///
    /// This method is primarily useful when dealing with [`ManuallyDrop`] or [`MaybeUninit`] inside of an [`ArrApi`].
    pub const fn as_mut_inner(&mut self) -> &mut A {
        &mut self.0
    }

    // Changes the type of the inner array to one with the same item and length.
    //
    //
    pub const fn retype<Dst: Array<Item = T, Length = N>>(self) -> ArrApi<Dst> {
        retype(self)
    }

    // Tries to change the type of the inner array to one with the same item and length.
    //
    // If the lengths are the same, the operation is successful. Otherwise, the original
    // array is returned.
    pub const fn try_retype<Dst: Array<Item = T>>(self) -> Result<ArrApi<Dst>, Self> {
        match crate::uint::cmp::<N, Dst::Length>().is_eq() {
            true => Ok(unsafe { self.retype_unchecked() }),
            false => Err(self),
        }
    }

    /// # Safety
    /// `Self::Length == Dst::Length`
    pub const unsafe fn retype_unchecked<Dst: Array<Item = T>>(self) -> ArrApi<Dst> {
        unsafe { retype_unchecked(self) }
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
        from_slice(slice)
    }

    pub const fn from_mut_slice(slice: &mut [T]) -> Result<&mut Self, TryFromSliceError> {
        from_mut_slice(slice)
    }

    pub const fn into_manually_drop(self) -> ArrApi<ManuallyDrop<A>> {
        ArrApi::new(ManuallyDrop::new(self.into_inner()))
    }

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
        // SAFETY: min(N, I) + saturating_sub(N, I) = N
        let ArrConcat(lhs, rhs): ArrConcat<
            ManuallyDrop<CanonArr<T, _>>,
            ManuallyDrop<CanonArr<T, _>>,
        > = unsafe { retype_unchecked(ManuallyDrop::new(self)) };

        (ManuallyDrop::into_inner(lhs), ManuallyDrop::into_inner(rhs))
    }

    pub const fn concat<Rhs>(self, rhs: Rhs) -> ArrApi<ImplArr![T; ops::Add<N, Rhs::Length>]>
    where
        Rhs: Array<Item = T>,
    {
        retype::<_, CanonArr<_, _>>(ArrConcat(self, rhs))
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
