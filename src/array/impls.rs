use core::mem::{ManuallyDrop, MaybeUninit};

use crate::condty::CondResult;
use crate::{Uint, ops, uint, utils};

use crate::array::{helper::*, *};
use crate::internals::ArraySealed;

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

// SAFETY: repr(transparent), CondDirect<C, O, E> is O if C else E
unsafe impl<C: Uint, T, O, E> Array for crate::condty::CondResult<C, O, E>
where
    O: Array<Item = T>,
    E: Array<Item = T>,
{
    type Item = T;
    type Length = uint::From<crate::ops::If<C, O::Length, E::Length>>;
}
impl<C: Uint, T, O, E> ArraySealed for crate::condty::CondResult<C, O, E>
where
    O: Array<Item = T>,
    E: Array<Item = T>,
{
}

mod cmp;
mod convert;
mod core_impl;
mod iter;
mod tuple_convert;

impl<A, B> Concat<A, B> {
    /// Returns the fields of this struct as a pair of arrays wrapped in
    /// [`ManuallyDrop`].
    ///
    /// May make it easier to destructure the result in `const` contexts.
    ///
    /// Note that the result of this method is not an [`Array`] as it only fulfills the layout
    /// invariants. This is purely a convenience methods for destructuring.
    #[must_use = "The values returned by this function are wrapped in ManuallyDrop and may need to be dropped"]
    pub const fn into_manual_drop(self) -> Concat<ManuallyDrop<A>, ManuallyDrop<B>> {
        // SAFETY: `self` is passed by value and can be destructed by read
        unsafe {
            crate::utils::destruct_read!(Self, (lhs, rhs), self);
            Concat(ManuallyDrop::new(lhs), ManuallyDrop::new(rhs))
        }
    }
}

impl<A> Flatten<A> {
    /// Returns the field of this struct.
    pub const fn into_inner(self) -> A {
        // SAFETY: `self` is passed by value and can be destructed by read
        unsafe {
            crate::utils::destruct_read!(Self, (inner), self);
            inner
        }
    }
}

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
    /// inside of an [`ArrApi`] when moving out of [`inner`](Self::inner) is
    /// not possible, e.g. in `const` contexts with generics.
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
        // SAFETY: `Array` layout guarantees
        unsafe { utils::same_size_transmute!(Self, Dst, self) }
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
        if uint::to_bool::<ops::Eq<N, Dst::Length>>() {
            // SAFETY: `Array` layout guarantees
            crate::condty::CondResult::new_ok(unsafe {
                utils::same_size_transmute!(Self, Dst, self)
            })
        } else {
            crate::condty::CondResult::new_err(self)
        }
    }

    /// [`core::array::from_fn`], but as a method.
    ///
    /// # Examples
    /// ```rust
    /// use genuint::{array::*, small::*};
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

    /// The same as `ArrApi::new(from).retype::<Self>()`.
    ///
    /// # Examples
    /// ```rust
    /// use genuint::array::*;
    /// let arr = Arr::retype_from([1, 2, 3]);
    /// assert_eq!(arr, [1, 2, 3]);
    /// ```
    pub const fn retype_from<Src: Array<Item = T, Length = N>>(from: Src) -> Self {
        ArrApi::new(from).retype()
    }

    /// Equivalent to `[x; N]` with `x` of a copyable type.
    ///
    /// # Examples
    /// ```
    /// use genuint::{array::*, small::*};
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

    /// Moves the items from another array of [`MaybeUninit<T>`] items.
    ///
    /// If the input array is larger than this array, the extra items will be forgotten.
    /// If the input array is smaller, the missing items will be left uninitialized.
    ///
    /// This method is defined on `ArrApi<A>` and creates `ArrApi<MaybeUninit<A>>`.
    /// This is so that [`Arr::resize_uninit_from`] infers the return type to
    /// `ArrApi<MaybeUninit<ArrInner<_, _>>>`.
    pub const fn resize_uninit_from<B>(arr: B) -> ArrApi<MaybeUninit<A>>
    where
        B: Array<Item = MaybeUninit<T>>,
    {
        // SAFETY: M := B::Length
        // - if M >= N, then transmuting through a union forgets `M - N` elements,
        //   which is always safe.
        // - if M <= N, then transmuting through a union fills the rest of the array with
        //   uninitialized memory, which is valid in this context.
        unsafe { utils::union_transmute!(B, ArrApi<MaybeUninit<A>>, arr) }
    }

    /// Tries to turn the array into a builtin `[T; M]` array of the same size.
    ///
    /// # Errors
    /// If `Self::Length != M`.
    ///
    /// # Examples
    /// ```
    /// use genuint::{array::*, small::*};
    ///
    /// let arr = Arr::<_, _3>::from_fn(|i| i); // type inference
    /// assert_eq!(arr.try_into_builtin_arr::<2>(), Err(arr));
    /// assert_eq!(arr.try_into_builtin_arr::<3>(), Ok([0, 1, 2]));
    /// assert_eq!(arr.try_into_builtin_arr::<4>(), Err(arr));
    /// ```
    pub const fn try_into_builtin_arr<const M: usize>(self) -> Result<[T; M], Self> {
        if len_is::<Self>(M) {
            // SAFETY: `Array` invariant
            Ok(unsafe { utils::same_size_transmute!(Self, [T; M], self) })
        } else {
            Err(self)
        }
    }
}
