use core::{
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
};

use crate::{Uint, array::extra::ImplArr, const_fmt, ops, uint};

use super::{ArrApi, ArrVecApi, Array, extra::arr_len};

#[repr(transparent)]
pub struct ArrVecDrop<A: Array<Item = T>, T = <A as Array>::Item>(ArrVecRepr<A>, PhantomData<T>);
impl<A: Array<Item = T>, T> Drop for ArrVecDrop<A, T> {
    fn drop(&mut self) {
        // SAFETY: repr(transparent); elements are behind MaybeUninit they will only be dropped
        // once. We are only dropping the initialized part of the array.
        unsafe {
            let vec = &mut *(&raw mut *self).cast::<ArrVecApi<A>>();
            core::ptr::drop_in_place(vec.as_mut_slice())
        }
    }
}

pub struct ArrVecRepr<A: Array> {
    pub len: usize,
    pub arr: ArrApi<MaybeUninit<A>>,
}

macro_rules! repr {
    ($self:expr) => {
        $self.0.0
    };
}

// TODO: Capacity panics
impl<A: Array<Item = T>, T> ArrVecApi<A> {
    /// # Safety
    /// `repr.arr[..repr.len]` must be initialized.
    pub(crate) const unsafe fn from_repr(repr: ArrVecRepr<A>) -> Self {
        Self(ArrVecDrop(repr, PhantomData), PhantomData)
    }

    const fn into_repr(self) -> ArrVecRepr<A> {
        let this = ManuallyDrop::new(self);
        let repr = &repr!(const_util::mem::man_drop_ref(&this));
        // SAFETY: Known safe way of destructuring in `const fn`
        unsafe { core::ptr::read(repr) }
    }

    /// Creates an empty [`ArrVecApi`].
    ///
    /// # Examples
    /// ```
    /// use generic_uint::{array::*, uint};
    ///
    /// type A = Arr<i32, uint::FromU128<10>>;
    /// assert_eq!(ArrVecApi::<A>::new(), []);
    /// ```
    pub const fn new() -> Self {
        let repr = ArrVecRepr {
            arr: ArrApi::new(MaybeUninit::uninit()),
            len: 0,
        };
        // SAFETY: Anything has 0 initialized elements
        unsafe { Self::from_repr(repr) }
    }

    /// Creates a full [`ArrVecApi<A>`] from an instance of `A`.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// assert_eq!(ArrVecApi::full([1, 2, 3]), [1, 2, 3]);
    /// ```
    pub const fn full(full: A) -> Self {
        let repr = ArrVecRepr {
            arr: ArrApi::new(MaybeUninit::new(full)),
            len: arr_len::<A>(),
        };
        // SAFETY: We have a full array worth of elements
        unsafe { Self::from_repr(repr) }
    }

    /// Turns an [`ArrVecApi`] into a slice.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// let mut vec = ArrVecApi::full([1, 2, 3]);
    /// vec.pop();
    /// assert_eq!(vec.as_slice()[1..], [2]);
    /// ```
    pub const fn as_slice(&self) -> &[T] {
        let ArrVecRepr { ref arr, len } = repr!(self);
        let (arr, _) = arr.as_slice().split_at(len);
        // SAFETY: The first `len` elements are initialized by invariant
        unsafe { crate::utils::assume_init_slice(arr) }
    }

    /// Turns an [`ArrVecApi`] into a slice.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// let mut vec = ArrVecApi::full([1, 2, 3]);
    /// vec.as_mut_slice().reverse();
    /// assert_eq!(vec, [3, 2, 1]);
    /// ```
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        let ArrVecRepr { ref mut arr, len } = repr!(self);
        let (arr, _) = arr.as_mut_slice().split_at_mut(len);
        // SAFETY: The first `len` elements are initialized by invariant
        unsafe { crate::utils::assume_init_mut_slice(arr) }
    }

    /// Returns the length of the [`ArrVecApi`].
    ///
    /// The length is the number of elements known to be initialized.
    pub const fn len(&self) -> usize {
        repr!(self).len
    }

    /// Returns [`self.len() == 0`](Self::len).
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns [`ArrApi::<A>::length`]
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// assert_eq!(ArrVecApi::<[i32; 20]>::new().capacity(), 20);
    /// ```
    pub const fn capacity(&self) -> usize {
        arr_len::<A>()
    }

    /// Checks whether this vec is full.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// assert_eq!(ArrVecApi::<[i32; 20]>::new().is_full(), false);
    /// assert_eq!(ArrVecApi::full([1; 20]).is_full(), true);
    /// ```
    pub const fn is_full(&self) -> bool {
        self.len() >= self.capacity()
    }

    /// Moves the elements of this vec into a full array.
    ///
    /// # Panics
    /// If [`!self.is_full()`](Self::is_full)
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// let mut vec = ArrVecApi::<[i32; 2]>::new();
    /// vec.push(1);
    /// vec.push(2);
    /// let [a, b] = vec.into_full();
    /// assert_eq!((a, b), (1, 2));
    /// ```
    #[track_caller]
    pub const fn into_full(self) -> A {
        if self.is_full() {
            // SAFETY: The vec is full, hence all elements of the backing array are initialized
            unsafe { self.into_repr().arr.inner.assume_init() }
        } else {
            const_fmt::concat_fmt![
                "Call to `into_full` on `ArrVecApi` with length ",
                self.len(),
                " out of ",
                self.capacity()
            ]
            .panic();
        }
    }

    /// Discards an empty deque by asserting that it is empty and using
    /// [`core::mem::forget`] if it is.
    ///
    /// See the info about the [Drop implementation](crate::array::ArrVecApi#drop-implementation).
    /// ```
    /// use generic_uint::array::*;
    /// const fn works_in_const<A: Array<Item = i32>>(arr: A) -> i32 {
    ///     let mut vec = ArrVecApi::full(arr);
    ///     let mut sum = 0;
    ///     while let Some(item) = vec.pop() {
    ///         sum += item;
    ///     }
    ///     vec.assert_empty();
    ///     sum
    /// }
    /// assert_eq!(works_in_const([1; 20]), 20);
    /// ```
    #[track_caller]
    pub const fn assert_empty(self) {
        if !self.is_empty() {
            const_fmt::concat_fmt![
                "Call to `assert_empty` on `ArrVecApi` with length ",
                self.len()
            ]
            .panic()
        }
        core::mem::forget(self);
    }

    /// Equivalent of [`Vec::push`].
    ///
    /// # Panics
    /// If this vec is full.
    ///
    /// ```
    /// use generic_uint::array::*;
    ///
    /// let mut vec = ArrVecApi::<[_; 20]>::new();
    /// vec.push(1);
    /// vec.push(2);
    /// assert_eq!(vec, [1, 2]);
    /// ```
    #[track_caller]
    pub const fn push(&mut self, item: T) {
        const_util::result::expect_ok(self.try_push(item), "Call to `push` on full `ArrVecApi`")
    }

    /// Like [`push`](Self::push), but returns [`Err`] on full vecs.
    ///
    /// ```
    /// use generic_uint::array::*;
    ///
    /// let mut vec = ArrVecApi::<[_; 2]>::new();
    /// assert_eq!(vec.try_push(1), Ok(()));
    /// assert_eq!(vec.try_push(2), Ok(()));
    /// assert_eq!(vec.try_push(3), Err(3));
    /// assert_eq!(vec, [1, 2]);
    /// ```
    pub const fn try_push(&mut self, item: T) -> Result<(), T> {
        match self.is_full() {
            true => Err(item),
            false => Ok({
                let ArrVecRepr { arr, len } = &mut repr!(self);
                arr.as_mut_slice()[*len].write(item);
                *len += 1;
            }),
        }
    }

    /// Equivalent of [`Vec::pop`].
    ///
    /// # Examples
    /// ```
    /// use generic_uint::{array::*, consts::*};
    ///
    /// assert_eq!(
    ///     ArrVecApi::<[i32; 20]>::new().pop(),
    ///     None
    /// );
    /// assert_eq!(
    ///     ArrVecApi::full(Arr::<_, U20>::from_fn(|i| i)).pop(),
    ///     Some(19)
    /// );
    /// ```
    pub const fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let ArrVecRepr { arr, len } = &mut repr!(self);
        *len -= 1;
        // SAFETY: This is the last time we remember that the element at this index was initialized
        // (due to being in 0..old_len). After this, the element is treated as uninit and not
        // read from until overwritten
        Some(unsafe { arr.as_slice()[*len].assume_init_read() })
    }

    /// Equivalent of [`Vec::into_raw_parts`].
    ///
    /// Turns this array into its components.
    /// The first `len` elements of the array are guaranteed to be initialized.
    ///
    pub const fn into_parts(self) -> (ArrApi<MaybeUninit<A>>, usize) {
        let ArrVecRepr { len, arr } = self.into_repr();
        (arr, len)
    }

    /// Equivalent of [`Vec::from_raw_parts`].
    ///
    /// Creates a vec from its components.
    ///
    /// # Safety
    /// The first `len` elements of `arr` must be initialized.
    pub const unsafe fn from_parts(arr: ArrApi<MaybeUninit<A>>, len: usize) -> Self {
        // SAFETY: The first `len` elements are initialized
        unsafe { Self::from_repr(ArrVecRepr { arr, len }) }
    }

    /// Sets the length of this vec.
    ///
    /// # Safety
    /// The first `new_len` elements of the backing array must be initialized,
    /// and `new_len <= self.capacity()`.
    pub const unsafe fn set_len(&mut self, new_len: usize) {
        repr!(self).len = new_len;
    }

    /// Transfers the elements from this vec into a contiguous [`ArrDeqApi`](crate::array::ArrDeqApi).
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// let mut vec = ArrVecApi::<[_; 20]>::new();
    /// vec.push(1);
    /// vec.push(2);
    /// let mut deq = vec.into_arr_deq();
    /// assert_eq!(deq.as_slices(), ([1, 2].as_slice(), [].as_slice()));
    /// ```
    pub const fn into_arr_deq(self) -> crate::array::ArrDeqApi<A> {
        use crate::array::*;

        let ArrVecRepr { len, arr } = self.into_repr();
        // SAFETY: `len` elements starting at index `0` are initialized.
        unsafe { ArrDeqApi::from_repr(arr_deq::ArrDeqRepr { arr, len, head: 0 }) }
    }
}

impl<T, N: Uint, A: Array<Item = T, Length = N>> ArrVecApi<A> {
    pub const fn grow<M: Uint>(self) -> ArrVecApi<ImplArr![T; ops::Max<A::Length, M>]> {
        const_util::result::unwrap_ok(self.try_grow())
    }

    pub const fn try_grow<M: Uint>(self) -> Result<ArrVecApi<ImplArr![T; M]>, Self> {
        if uint::cmp::<M, N>().is_ge() {
            let ArrVecRepr { len, arr } = self.into_repr();
            let repr = ArrVecRepr {
                arr: arr.resize_uninit(),
                len,
            };
            // SAFETY: new cap >= old cap, so we must still have `len` initialized elements.
            Ok(unsafe { ArrVecApi::from_repr(repr) })
        } else {
            Err(self)
        }
    }
}

mod core_impl;
