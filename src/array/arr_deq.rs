use core::{
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
    ops::Range,
    ptr::NonNull,
};

use crate::{const_fmt, utils};

use super::{ArrApi, ArrDeqApi, Array, extra::*};

#[repr(transparent)]
pub struct ArrDeqDrop<A: Array<Item = T>, T = <A as Array>::Item>(ArrDeqRepr<A>, PhantomData<T>);
impl<A: Array<Item = T>, T> Drop for ArrDeqDrop<A, T> {
    fn drop(&mut self) {
        // SAFETY: repr(transparent); elements are behind MaybeUninit they will only be dropped
        // once. We are only dropping the initialized parts of the array.
        unsafe {
            let deq = &mut *(&raw mut *self).cast::<ArrDeqApi<A>>();
            let (lhs, rhs) = deq.as_mut_slices();
            core::ptr::drop_in_place(lhs);
            core::ptr::drop_in_place(rhs);
        }
    }
}

pub struct ArrDeqRepr<A: Array> {
    pub head: usize,
    pub len: usize,
    pub arr: ArrApi<MaybeUninit<A>>,
}
macro_rules! repr {
    ($self:expr) => {
        $self.0.0
    };
}

const fn wrapping_idx(logical: usize, cap: usize) -> usize {
    debug_assert!(logical == 0 || logical < 2 * cap);
    let phys = if logical >= cap {
        logical - cap
    } else {
        logical
    };
    debug_assert!(phys == 0 || phys < cap);
    phys
}

const fn phys_idx_of(idx: usize, head: usize, cap: usize) -> usize {
    // TODO: Overflow explain
    wrapping_idx(head.wrapping_add(idx), cap)
}
const fn slice_ranges(head: usize, len: usize, cap: usize) -> (Range<usize>, Range<usize>) {
    debug_assert!(head <= cap);
    debug_assert!(len <= cap);
    if len == 0 {
        (0..0, 0..0)
    } else {
        let after_head = cap - head;
        if after_head >= len {
            (head..head + len, 0..0)
        } else {
            let tail_len = len - after_head;
            (head..cap, 0..tail_len)
        }
    }
}

/// Combined implementation akin to `VecDeque::as_(mut)_slices`.
///
/// # Safety
/// - `len <= buf.len()`
/// - `head <= buf.len()`
/// - `buf` is valid for reads. The returned pointers are too.
/// - `len` elements starting at `head` and wrapping around the end are initialized
/// - if `buf` is valid for writes, then so are the returned pointers
const unsafe fn as_nonnull_slices<T>(
    buf: NonNull<[MaybeUninit<T>]>,
    head: usize,
    len: usize,
) -> (NonNull<[T]>, NonNull<[T]>) {
    debug_assert!(len <= buf.len());
    debug_assert!(head <= buf.len());
    let (lhs, rhs) = slice_ranges(head, len, buf.len());
    // SAFETY: `slice_ranges` always returns ranges in the initialized parts of the deque.
    unsafe {
        (
            utils::subslice_init_nonnull(buf, lhs),
            utils::subslice_init_nonnull(buf, rhs),
        )
    }
}

impl<A: Array<Item = T>, T> ArrDeqApi<A> {
    const fn get_cap() -> usize {
        arr_len::<A>()
    }
    const fn phys_idx_of(&self, idx: usize) -> usize {
        phys_idx_of(idx, repr!(self).head, self.capacity())
    }
    const fn tail(&self) -> usize {
        self.phys_idx_of(self.len())
    }
    const fn phys_idx_before_head(&self, idx: usize) -> usize {
        let cap = self.capacity();
        wrapping_idx(repr!(self).head.wrapping_sub(idx).wrapping_add(cap), cap)
    }

    /// # Safety
    /// The element at `self.arr[idx]` must be initialized and never used again
    /// until overwritten (including drops)
    const unsafe fn phys_read(&self, idx: usize) -> T {
        // SAFETY: `idx` is initialized and never used again
        unsafe { repr!(self).arr.as_slice()[idx].assume_init_read() }
    }
}

// TODO: Capacity panics
impl<A: Array<Item = T>, T> ArrDeqApi<A> {
    const fn into_repr(self) -> ArrDeqRepr<A> {
        let this = ManuallyDrop::new(self);
        let repr = &repr!(const_util::mem::man_drop_ref(&this));
        // SAFETY: Known safe way of destructuring in `const fn`
        unsafe { core::ptr::read(repr) }
    }

    pub(crate) const unsafe fn from_repr(repr: ArrDeqRepr<A>) -> Self {
        Self(ArrDeqDrop(repr, PhantomData), PhantomData)
    }

    /// Creates a new empty [`ArrDeqApi`].
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// assert_eq!(ArrDeqApi::<[i32; 20]>::new(), []);
    /// ```
    pub const fn new() -> Self {
        // SAFETY: 0 elements are initialized
        unsafe {
            Self::from_repr(ArrDeqRepr {
                arr: ArrApi::new(MaybeUninit::uninit()),
                head: 0,
                len: 0,
            })
        }
    }

    /// Creates a full [`ArrDeqApi<A>`] from an instance of `A`.
    ///
    /// The resulting deque is contiguous.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// assert_eq!(ArrDeqApi::new_full([1; 20]), [1; 20]);
    /// ```
    pub const fn new_full(full: A) -> Self {
        // SAFETY: All elements are initialized because we have a fully initialized array
        unsafe {
            Self::from_repr(ArrDeqRepr {
                arr: ArrApi::new(MaybeUninit::new(full)),
                head: 0,
                len: arr_len::<A>(),
            })
        }
    }

    /// Returns [`ArrApi::<A>::length`]
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// assert_eq!(ArrDeqApi::<[i32; 20]>::new().capacity(), 20);
    /// ```
    pub const fn capacity(&self) -> usize {
        Self::get_cap()
    }

    /// Returns the current number of elements in this deque.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// assert_eq!(ArrDeqApi::<[i32; 20]>::new().len(), 0);
    /// assert_eq!(ArrDeqApi::new_full([1; 20]).len(), 20);
    /// ```
    pub const fn len(&self) -> usize {
        repr!(self).len
    }

    /// Checks whether this deque is empty.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// assert_eq!(ArrDeqApi::<[i32; 20]>::new().is_empty(), true);
    /// assert_eq!(ArrDeqApi::new_full([1; 20]).is_empty(), false);
    /// ```
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Checks whether this deque is full.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// assert_eq!(ArrDeqApi::<[i32; 20]>::new().is_full(), false);
    /// assert_eq!(ArrDeqApi::new_full([1; 20]).is_full(), true);
    /// ```
    pub const fn is_full(&self) -> bool {
        self.len() >= self.capacity()
    }

    /// Equivalent of [`VecDeque::pop_front`](std::collections::VecDeque::pop_front).
    ///
    /// # Examples
    /// ```
    /// use generic_uint::{array::*, consts::*};
    ///
    /// assert_eq!(
    ///     ArrDeqApi::<[i32; 20]>::new().pop_front(),
    ///     None
    /// );
    /// assert_eq!(
    ///     ArrDeqApi::new_full(Arr::<_, U20>::from_fn(|i| i)).pop_front(),
    ///     Some(0)
    /// );
    /// ```
    pub const fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let new_head = self.phys_idx_of(1);
        let ArrDeqRepr { head, len, arr: _ } = &mut repr!(self);
        *len -= 1;
        let old_head = core::mem::replace(head, new_head);
        // SAFETY: This is the last time we remember that the element at this index was initialized.
        // After this, the element is treated as uninit and not read from until overwritten
        Some(unsafe { self.phys_read(old_head) })
    }

    /// Equivalent of [`VecDeque::pop_back`](std::collections::VecDeque::pop_back).
    ///
    /// # Examples
    /// ```
    /// use generic_uint::{array::*, consts::*};
    ///
    /// assert_eq!(
    ///     ArrDeqApi::<[i32; 20]>::new().pop_back(),
    ///     None
    /// );
    /// assert_eq!(
    ///     ArrDeqApi::new_full(Arr::<_, U20>::from_fn(|i| i)).pop_back(),
    ///     Some(19)
    /// );
    /// ```
    pub const fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        repr!(self).len -= 1;
        // SAFETY: This is the last time we remember that the element at this index was initialized.
        // After this, the element is treated as uninit and not read from until overwritten
        Some(unsafe { self.phys_read(self.tail()) })
    }

    /// Equivalent of [`VecDeque::push_front`](std::collections::VecDeque::push_front).
    ///
    /// # Panics
    /// If this deque is full.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// let mut deq = ArrDeqApi::<[_; 20]>::new();
    /// deq.push_front(1);
    /// deq.push_front(2);
    /// assert_eq!(deq, [2, 1]);
    /// ```
    #[track_caller]
    pub const fn push_front(&mut self, item: T) {
        const_util::result::expect_ok(
            self.try_push_front(item),
            "Call to `push_front` on full `ArrDeqApi`",
        )
    }

    /// Equivalent of [`VecDeque::push_back`](std::collections::VecDeque::push_back).
    ///
    /// # Panics
    /// If this deque is full.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// let mut deq = ArrDeqApi::<[_; 20]>::new();
    /// deq.push_back(1);
    /// deq.push_back(2);
    /// assert_eq!(deq, [1, 2]);
    /// ```
    #[track_caller]
    pub const fn push_back(&mut self, item: T) {
        const_util::result::expect_ok(
            self.try_push_back(item),
            "Call to `push_back` on full `ArrDeqApi`",
        )
    }

    /// Like [`push_front`](Self::push_front), but returns [`Err`] on full deques.
    ///
    /// ```
    /// use generic_uint::array::*;
    ///
    /// let mut deq = ArrDeqApi::<[_; 2]>::new();
    /// assert_eq!(deq.try_push_front(1), Ok(()));
    /// assert_eq!(deq.try_push_front(2), Ok(()));
    /// assert_eq!(deq.try_push_front(3), Err(3));
    /// assert_eq!(deq, [2, 1]);
    /// ```
    pub const fn try_push_front(&mut self, item: T) -> Result<(), T> {
        match self.is_full() {
            true => Err(item),
            false => Ok({
                let new_head = self.phys_idx_before_head(1);
                let ArrDeqRepr { head, len, arr } = &mut repr!(self);
                arr.as_mut_slice()[new_head].write(item);
                *head = new_head;
                *len += 1;
            }),
        }
    }

    /// Like [`push_back`](Self::push_back), but returns [`Err`] on full deques.
    ///
    /// ```
    /// use generic_uint::array::*;
    ///
    /// let mut deq = ArrDeqApi::<[_; 2]>::new();
    /// assert_eq!(deq.try_push_back(1), Ok(()));
    /// assert_eq!(deq.try_push_back(2), Ok(()));
    /// assert_eq!(deq.try_push_back(3), Err(3));
    /// assert_eq!(deq, [1, 2]);
    /// ```
    pub const fn try_push_back(&mut self, item: T) -> Result<(), T> {
        match self.is_full() {
            true => Err(item),
            false => Ok({
                let tail = self.tail();
                let ArrDeqRepr { head: _, len, arr } = &mut repr!(self);
                arr.as_mut_slice()[tail].write(item);
                *len += 1;
            }),
        }
    }

    /// Returns a reference to the elements as a pair of slices.
    ///
    /// If this deque is contiguous, then the right slice will be empty.
    /// The left slice is empty only when the entire deque is empty.
    ///
    /// Note that the exact distribution between left and right are not guaranteed
    /// unless the length of this deque is 1 or less, or this deque is contiguous
    /// according to an API.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// let mut deq = ArrDeqApi::<[_; 20]>::new();
    /// deq.push_front(1);
    /// deq.push_back(2);
    /// let (lhs, rhs) = deq.as_slices();
    /// assert_eq!(lhs.iter().chain(rhs).next(), Some(&1));
    /// assert_eq!(lhs.iter().chain(rhs).next_back(), Some(&2));
    /// ```
    pub const fn as_slices(&self) -> (&[T], &[T]) {
        let ArrDeqRepr { head, len, ref arr } = repr!(self);
        // SAFETY: The invariants of `as_nonnull_slices` are such that it it safe to call with the
        // fields of a `ArrDeqApi`. The returned pointers are valid for reads.
        unsafe {
            let (lhs, rhs) = as_nonnull_slices(
                crate::utils::nonnull_from_const_ref(arr.as_slice()),
                head,
                len,
            );
            (lhs.as_ref(), rhs.as_ref())
        }
    }

    /// Returns a mutable reference to the elements as a pair of slices.
    ///
    /// If this deque is contiguous, then the right slice will be empty.
    /// The left slice is empty only when the entire deque is empty.
    ///
    /// Note that the exact distribution between left and right are not guaranteed
    /// unless the length of this deque is 1 or less, or this deque is contiguous
    /// according to an API.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// let mut deq = ArrDeqApi::<[_; 20]>::new();
    /// deq.push_front(1);
    /// deq.push_back(2);
    /// let (lhs, rhs) = deq.as_mut_slices();
    /// assert_eq!(lhs.iter_mut().chain(&mut *rhs).next(), Some(&mut 1));
    /// assert_eq!(lhs.iter_mut().chain(rhs).next_back(), Some(&mut 2));
    /// ```
    pub const fn as_mut_slices(&mut self) -> (&mut [T], &mut [T]) {
        let ArrDeqRepr {
            head,
            len,
            ref mut arr,
        } = repr!(self);
        // SAFETY: The invariants of `as_nonnull_slices` are such that it it safe to call with the
        // fields of a `ArrDeqApi`. The returned pointers are valid for reads and writes because
        // the slice is.
        unsafe {
            let (mut lhs, mut rhs) = as_nonnull_slices(
                NonNull::new_unchecked(arr.as_mut_slice()), //
                head,
                len,
            );
            (lhs.as_mut(), rhs.as_mut())
        }
    }

    /// Rotates the underlying array to make the initialized part contiguous.
    /// Also returns a mutable slice of the now contiguous elements.
    ///
    /// After calling this method, [`Self::as_slices`] and [`Self::as_mut_slices`] are guaranteed
    /// to return an empty slice as the right tuple element.
    ///
    /// ```
    /// use generic_uint::array::*;
    /// let mut deq = ArrDeqApi::<[i32; 20]>::new();
    /// for i in 0..3 {
    ///     deq.push_back(i);
    ///     deq.push_front(10 - i);
    /// }
    /// assert_eq!(deq.make_contiguous(), [8, 9, 10, 0, 1, 2]);
    /// assert_eq!(deq.as_slices(), ([8, 9, 10, 0, 1, 2].as_slice(), [].as_slice()));
    /// ```
    #[inline]
    pub const fn make_contiguous(&mut self) -> &mut [T] {
        const fn rotate_left<T>(slice: &mut [T], dist: usize) {
            const fn reverse<T>(slice: &mut [T]) {
                let mut i = 0;
                while i < slice.len() / 2 {
                    slice.swap(i, slice.len() - i - 1);
                    i += 1;
                }
            }
            let (lhs, rhs) = slice.split_at_mut(dist);
            // EFGHIJKLMN^ABCD
            reverse(lhs);
            // NMLKJIHGFE^ABCD
            reverse(rhs);
            // NMLKJIHGFE^DBCA
            reverse(slice);
            // ABCDEFGHIJ^KLMN
        }

        let ArrDeqRepr { head, len: _, arr } = &mut repr!(self);
        rotate_left(arr.as_mut_slice(), *head);
        *head = 0;
        self.as_mut_slices().0
    }

    /// Transfers the elements into an [`ArrVecApi`](crate::array::ArrVecApi).
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// let mut deq = ArrDeqApi::<[_; 20]>::new();
    /// deq.push_front(2);
    /// deq.push_front(1);
    /// let mut vec = deq.into_arr_vec();
    /// assert_eq!(vec.as_slice(), [1, 2]);
    /// ```
    pub const fn into_arr_vec(mut self) -> crate::array::ArrVecApi<A> {
        use crate::array::*;

        self.make_contiguous();
        let ArrDeqRepr { len, arr, head: _ } = self.into_repr();
        // SAFETY: `head == 0`, so the first `len` elements are initialized.
        unsafe { ArrVecApi::from_uninit_parts(arr, len) }
    }

    /// Makes this deque contiguous and then returns the elements as a full array.
    ///
    /// # Panics
    /// If [`!self.is_full()`](Self::is_full)
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// let mut deq = ArrDeqApi::<[i32; 2]>::new();
    /// deq.push_back(2);
    /// deq.push_front(1);
    /// let [a, b] = deq.assert_full();
    /// assert_eq!((a, b), (1, 2));
    /// ```
    #[track_caller]
    pub const fn assert_full(mut self) -> A {
        if self.is_full() {
            self.make_contiguous();
            // SAFETY: The deque is full, hence all elements of the backing array are initialized
            unsafe { self.into_repr().arr.inner.assume_init() }
        } else {
            const_fmt::panic_fmt![
                "Call to `assert_full` on `ArrDeqApi` with length ",
                self.len(),
                " out of ",
                self.capacity()
            ]
        }
    }

    /// Discards an empty deque by asserting that it is empty and using
    /// [`core::mem::forget`] if it is.
    ///
    /// See the info about the [Drop implementation](crate::array::ArrVecApi#drop-implementation).
    /// ```
    /// use generic_uint::array::*;
    /// const fn works_in_const<A: Array<Item = i32>>(arr: A) -> i32 {
    ///     let mut deq = ArrDeqApi::new_full(arr);
    ///     let mut sum = 0;
    ///     while let Some(item) = deq.pop_front() {
    ///         sum += item;
    ///     }
    ///     deq.assert_empty();
    ///     sum
    /// }
    /// assert_eq!(works_in_const([1; 20]), 20);
    /// ```
    #[track_caller]
    pub const fn assert_empty(self) {
        if !self.is_empty() {
            const_fmt::panic_fmt![
                "Call to `assert_empty` on `ArrDeqApi` with length ",
                self.len()
            ]
        }
        core::mem::forget(self);
    }
}

mod core_impl;
