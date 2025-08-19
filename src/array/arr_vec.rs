use core::{
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
};

use crate::{Uint, array::extra::ImplArr, ops, uint};

use super::{ArrApi, ArrVec, Array, extra::arr_len};

#[repr(transparent)]
pub struct ArrVecDrop<A: Array<Item = T>, T = <A as Array>::Item>(ArrVecRepr<A>, PhantomData<T>);
impl<A: Array<Item = T>, T> Drop for ArrVecDrop<A, T> {
    fn drop(&mut self) {
        unsafe {
            let vec = &mut *(&raw mut *self).cast::<ArrVec<A>>();
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
impl<A: Array<Item = T>, T> ArrVec<A> {
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

    /// Creates an empty [`ArrVec`].
    ///
    /// # Examples
    /// ```
    /// use generic_uint::{array::*, uint};
    ///
    /// type A = Arr<i32, uint::FromU128<10>>;
    /// assert_eq!(ArrVec::<A>::new(), []);
    /// ```
    pub const fn new() -> Self {
        let repr = ArrVecRepr {
            arr: ArrApi::new(MaybeUninit::uninit()),
            len: 0,
        };
        // SAFETY: Anything has 0 initialized elements
        unsafe { Self::from_repr(repr) }
    }

    /// Creates a full [`ArrVec<A>`] from an instance of `A`.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// assert_eq!(ArrVec::full([1, 2, 3]), [1, 2, 3]);
    /// ```
    pub const fn full(full: A) -> Self {
        let repr = ArrVecRepr {
            arr: ArrApi::new(MaybeUninit::new(full)),
            len: arr_len::<A>(),
        };
        // SAFETY: We have a full array worth of elements
        unsafe { Self::from_repr(repr) }
    }

    /// Turns an [`ArrVec`] into a slice.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// let mut vec = ArrVec::full([1, 2, 3]);
    /// vec.pop();
    /// assert_eq!(vec.as_slice()[1..], [2]);
    /// ```
    pub const fn as_slice(&self) -> &[T] {
        let ArrVecRepr { ref arr, len } = repr!(self);
        let (arr, _) = arr.as_slice().split_at(len);
        unsafe { crate::utils::assume_init_slice(arr) }
    }

    /// Turns an [`ArrVec`] into a slice.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// let mut vec = ArrVec::full([1, 2, 3]);
    /// vec.as_mut_slice().reverse();
    /// assert_eq!(vec, [3, 2, 1]);
    /// ```
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        let ArrVecRepr { ref mut arr, len } = repr!(self);
        let (arr, _) = arr.as_mut_slice().split_at_mut(len);
        unsafe { crate::utils::assume_init_mut_slice(arr) }
    }

    /// Returns the length of the [`ArrVec`].
    ///
    /// The length is the number of known to be initialized elements.
    pub const fn len(&self) -> usize {
        repr!(self).len
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const fn capacity(&self) -> usize {
        arr_len::<A>()
    }

    pub const fn is_full(&self) -> bool {
        self.len() >= self.capacity()
    }

    #[track_caller]
    pub const fn into_full(self) -> A {
        if !self.is_full() {
            panic!("Call to `into_full` on non-full `ArrVec`");
        }
        unsafe { self.into_repr().arr.inner.assume_init() }
    }

    #[track_caller]
    pub const fn push(&mut self, item: T) {
        const_util::result::expect_ok(self.try_push(item), "Call to `push` on full `ArrVec`")
    }

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

    pub const fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let ArrVecRepr { arr, len } = &mut repr!(self);
        *len -= 1;
        Some(unsafe { arr.as_slice()[*len].assume_init_read() })
    }

    pub const fn into_parts(self) -> (ArrApi<MaybeUninit<A>>, usize) {
        let ArrVecRepr { len, arr } = self.into_repr();
        (arr, len)
    }

    /// # Safety
    /// The first `len` elements of `arr` must be initialized.
    pub const unsafe fn from_parts(arr: ArrApi<MaybeUninit<A>>, len: usize) -> Self {
        // SAFETY: The first `len` elements are initialized
        unsafe { Self::from_repr(ArrVecRepr { arr, len }) }
    }

    /// # Safety
    /// The first `new_len` elements of the backing array must be initialized,
    /// and `new_len <= self.capacity()`.
    pub const unsafe fn set_len(&mut self, new_len: usize) {
        repr!(self).len = new_len;
    }

    /// Transfers the elements into a contiguous [`ArrDeq`](crate::array::ArrDeq).
    pub const fn into_arr_deq(self) -> crate::array::ArrDeq<A> {
        use crate::array::*;

        let ArrVecRepr { len, arr } = self.into_repr();
        // SAFETY: `len` elements starting at index `0` are initialized.
        unsafe { ArrDeq::from_repr(arr_deq::ArrDeqRepr { arr, len, head: 0 }) }
    }
}

impl<T, N: Uint, A: Array<Item = T, Length = N>> ArrVec<A> {
    pub const fn grow<M: Uint>(self) -> ArrVec<ImplArr![T; ops::Max<A::Length, M>]> {
        const_util::result::unwrap_ok(self.try_grow())
    }

    pub const fn try_grow<M: Uint>(self) -> Result<ArrVec<ImplArr![T; M]>, Self> {
        if uint::cmp::<M, N>().is_ge() {
            let ArrVecRepr { len, arr } = self.into_repr();
            let repr = ArrVecRepr {
                arr: arr.resize_uninit(),
                len,
            };
            // SAFETY: new cap >= old cap, so we must still have `len` initialized elements.
            Ok(unsafe { ArrVec::from_repr(repr) })
        } else {
            Err(self)
        }
    }
}

mod core_impl;
