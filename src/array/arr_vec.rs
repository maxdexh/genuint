use core::{
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
};

mod core_impl;

use crate::{
    Uint,
    array::{ArrApi, ArrVec, ArrVecApi, Array, helper::*},
    const_fmt, ops, uint,
};

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

impl<A: Array<Item = T, Length = N>, T, N: Uint> ArrVecApi<A> {
    /// Creates a vector from its components.
    ///
    /// Equivalent of [`Vec::from_raw_parts`].
    ///
    /// # Safety
    /// The first `len` elements of `arr`, i.e. `arr[..len]`, must be initialized.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    /// use core::mem::MaybeUninit;
    ///
    /// let mut arr = ArrApi::new(MaybeUninit::<[_; 3]>::uninit());
    /// arr[0].write(1);
    /// // SAFETY: The first element of `arr` is initialized
    /// let vec = unsafe { ArrVecApi::from_uninit_parts(arr, 1) };
    /// assert_eq!(vec, [1]);
    /// ```
    pub const unsafe fn from_uninit_parts(arr: ArrApi<MaybeUninit<A>>, len: usize) -> Self {
        let repr = ArrVecRepr { arr, len };
        Self(ArrVecDrop(repr, PhantomData), PhantomData)
    }

    /// Creates a vector from a backing array.
    ///
    /// The initial length of the vector will be zero. This method has the same effect as
    /// [`from_uninit_parts`](Self::from_uninit_parts) when combined with [`set_len`](Self::set_len).
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    /// use core::mem::MaybeUninit;
    ///
    /// let mut arr = ArrApi::new(MaybeUninit::<[_; 3]>::uninit());
    /// arr[0].write(1);
    /// let vec = ArrVecApi::from_uninit_array(arr);
    /// assert_eq!(vec, []); // length is always 0
    /// ```
    pub const fn from_uninit_array(arr: ArrApi<MaybeUninit<A>>) -> Self {
        // SAFETY: arr[0..0] is empty and thus initialized
        unsafe { Self::from_uninit_parts(arr, 0) }
    }

    /// Turns the vector into its components.
    ///
    /// The first `len` elements of the array are guaranteed to be initialized.
    ///
    /// Equivalent of [`Vec::into_raw_parts`].
    #[must_use = "The initialized elements of the array may need to be dropped"]
    pub const fn into_uninit_parts(self) -> (ArrApi<MaybeUninit<A>>, usize) {
        let ArrVecRepr { len, arr } = {
            let this = ManuallyDrop::new(self);
            let repr = &repr!(const_util::mem::man_drop_ref(&this));
            // SAFETY: Known safe way of destructuring in `const fn`
            unsafe { core::ptr::read(repr) }
        };
        (arr, len)
    }

    /// Returns references to the vector's components.
    ///
    /// The first `len` elements of the array are guaranteed to be initialized.
    ///
    /// A mutable version of this method can be emulated using [`set_len(0)`](Self::set_len)
    /// and [`split_at_spare_mut`](Self::split_at_spare_mut).
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    /// use core::mem::MaybeUninit;
    ///
    /// let mut arr = ArrApi::new(MaybeUninit::<[_; 3]>::uninit());
    /// arr[0].write(1);
    /// let vec = ArrVecApi::from_uninit_array(arr);
    /// let arr = vec.as_uninit_array();
    /// assert!(vec.is_empty());
    /// assert_eq!(unsafe { arr[0].assume_init_read() }, 1);
    /// ```
    pub const fn as_uninit_array(&self) -> &ArrApi<MaybeUninit<A>> {
        // NOTE: This does not expose the internal representation of the array because
        // `Array` types can be converted via casts
        &repr!(self).arr
    }

    /// Creates an empty vector.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::{array::*, uint};
    ///
    /// type A = Arr<i32, uint::FromU128<10>>;
    /// assert_eq!(ArrVecApi::<A>::new(), []);
    /// ```
    pub const fn new() -> Self {
        Self::from_uninit_array(ArrApi::new(MaybeUninit::uninit()))
    }

    /// Creates a full vector from an instance of the underlying array.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// assert_eq!(ArrVecApi::new_full([1, 2, 3]), [1, 2, 3]);
    /// ```
    pub const fn new_full(arr: A) -> Self {
        let arr = ArrApi::new(MaybeUninit::new(arr));
        // SAFETY: `arr` is initialized and has `arr_len::<A>()` elements
        unsafe { Self::from_uninit_parts(arr, arr_len::<A>()) }
    }

    /// Returns the vector's elements as mutable slices.
    ///
    /// The tuple is divided into the initialized and uninitialized elements of the vector.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    /// use core::mem::MaybeUninit;
    ///
    /// let mut arr = ArrApi::new(MaybeUninit::<[_; 3]>::uninit());
    /// arr[1].write(2);
    /// let mut vec = ArrVecApi::from_uninit_array(arr);
    /// vec.push(1);
    /// let (init, spare) = vec.split_at_spare();
    /// assert_eq!(init, [1]);
    /// assert_eq!(unsafe { spare[0].assume_init_read() }, 2);
    /// ```
    pub const fn split_at_spare(&self) -> (&[T], &[MaybeUninit<T>]) {
        let ArrVecRepr { ref arr, len } = repr!(self);
        let (init, spare) = arr.as_slice().split_at(len);
        // SAFETY: The first `len` elements are initialized by invariant
        (unsafe { crate::utils::assume_init_slice(init) }, spare)
    }

    /// Returns the vector's elements as slices.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// let mut vec = ArrVecApi::new_full([1, 2, 3]);
    /// vec.pop();
    /// assert_eq!(vec.as_slice()[1..], [2]);
    /// ```
    pub const fn as_slice(&self) -> &[T] {
        self.split_at_spare().0
    }

    pub const fn spare_capacity(&self) -> &[MaybeUninit<T>] {
        self.split_at_spare().1
    }

    /// Returns the vector's elements as mutable slices.
    ///
    /// The tuple is divided into the initialized and uninitialized elements of the vector.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    /// use core::mem::MaybeUninit;
    ///
    /// let mut arr = ArrApi::new(MaybeUninit::<[_; 3]>::uninit());
    /// arr[1].write(3);
    /// let mut vec = ArrVecApi::from_uninit_array(arr);
    /// vec.push(0);
    ///
    /// let (init, spare) = vec.split_at_spare_mut() else { unreachable!() };
    /// assert_eq!((init.len(), spare.len()), (1, 2));
    /// init[0] = 1;
    /// spare[1].write(2);
    /// spare.reverse();
    ///
    /// unsafe { vec.set_len(3) }
    /// assert_eq!(vec, [1, 2, 3]);
    /// ```
    pub const fn split_at_spare_mut(&mut self) -> (&mut [T], &mut [MaybeUninit<T>]) {
        let ArrVecRepr { ref mut arr, len } = repr!(self);
        let (init, spare) = arr.as_mut_slice().split_at_mut(len);
        // SAFETY: The first `len` elements are initialized by invariant
        (unsafe { crate::utils::assume_init_mut_slice(init) }, spare)
    }

    pub const fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        self.split_at_spare_mut().1
    }

    /// Returns the vector's initialized elements as a mutable slice.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// let mut vec = ArrVecApi::new_full([1, 2, 3]);
    /// vec.as_mut_slice().reverse();
    /// assert_eq!(vec, [3, 2, 1]);
    /// ```
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        self.split_at_spare_mut().0
    }

    /// Returns the length of the vector.
    ///
    /// The length is the number of elements known to be initialized.
    pub const fn len(&self) -> usize {
        repr!(self).len
    }

    /// Checks whether the vector is empty, i.e. whether [`len`](Self::len) is zero.
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the length of the vector's backing array, i.e. [`ArrApi::<A>::length`].
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

    /// Checks whether the vector is full.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// assert_eq!(ArrVecApi::<[i32; 20]>::new().is_full(), false);
    /// assert_eq!(ArrVecApi::new_full([1; 20]).is_full(), true);
    /// ```
    pub const fn is_full(&self) -> bool {
        self.spare_len() == 0
    }

    /// Returns the number of elements that can be pushed into the vector until it is full.
    ///
    /// This is the same as the length of the uninitialized segment, i.e.
    /// `self.capacity() - self.len()`
    pub const fn spare_len(&self) -> usize {
        if self.capacity() < self.len() {
            // SAFETY: We cannot have more than `capacity` initialized elements by definition
            // and it is a safety invariant of this type that the first `self.len()` elements
            // are initialized
            unsafe { core::hint::unreachable_unchecked() }
        }
        self.capacity() - self.len()
    }

    /// Moves the elements of the vector into a full array.
    ///
    /// # Panics
    /// If the vector is full.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    ///
    /// let mut vec = ArrVecApi::<[i32; 2]>::new();
    /// vec.push(1);
    /// vec.push(2);
    /// let [a, b] = vec.assert_full();
    /// assert_eq!((a, b), (1, 2));
    /// ```
    #[track_caller]
    pub const fn assert_full(self) -> A {
        match self.is_full() {
            // SAFETY: The vec is full, hence all elements of the backing array are initialized
            true => unsafe { self.into_uninit_parts().0.inner.assume_init() },
            false => const_fmt::panic_fmt![
                "Call to `assert_full` on `ArrVecApi` with length ",
                self.len(),
                " out of ",
                self.capacity()
            ],
        }
    }

    /// Discards the vector by asserting that it is empty and using [`core::mem::forget`].
    ///
    /// See the info about the [Drop implementation](crate::array::ArrVecApi#drop-implementation).
    ///
    /// # Panics
    /// If the vector is non-empty.
    ///
    /// # Examples
    /// ```
    /// use generic_uint::array::*;
    /// const fn works_in_const<A: Array<Item = i32>>(arr: A) -> i32 {
    ///     let mut vec = ArrVecApi::new_full(arr);
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
        match self.is_empty() {
            true => core::mem::forget(self),
            false => const_fmt::panic_fmt![
                "Call to `assert_empty` on `ArrVecApi` with length ",
                self.len()
            ],
        }
    }

    /// Equivalent of [`Vec::push`].
    ///
    /// # Panics
    /// If the vector is full.
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
    ///     ArrVecApi::new_full(Arr::<_, _20>::from_fn(|i| i)).pop(),
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

    /// Sets the length of the vector.
    ///
    /// Equivalent of [`Vec::set_len`].
    ///
    /// # Safety
    /// Same as [`Vec::set_len`].
    pub const unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.capacity());
        repr!(self).len = new_len;
    }

    /// Transfers the elements from the vector into a contiguous [`ArrDeqApi`](crate::array::ArrDeqApi).
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

        let (arr, len) = self.into_uninit_parts();
        // SAFETY: `len` elements starting at index `0` are initialized.
        unsafe { ArrDeqApi::from_repr(arr_deq::ArrDeqRepr { arr, len, head: 0 }) }
    }

    #[allow(clippy::type_complexity)]
    pub const fn split_at_uint<I: Uint>(
        self,
    ) -> (
        ArrVecApi<ImplArr![T; ops::Min<N, I>]>,
        ArrVecApi<ImplArr![T; ops::SatSub<N, I>]>,
    ) {
        let (arr, len) = self.into_uninit_parts();
        const_util::destruct_tuple! { lhs, rhs in arr.split_at_uint::<I>() }

        let rlen = match uint::to_usize::<I>() {
            Some(i) => len.saturating_sub(i),
            None => 0,
        };
        let llen = len - rlen;
        // SAFETY: lhs has llen leading valid elements, rhs has rlen
        unsafe {
            (
                ArrVec::from_uninit_parts(lhs.into_arr(), llen),
                ArrVec::from_uninit_parts(rhs.into_arr(), rlen),
            )
        }
    }

    pub const fn split_off_at_uint<I: Uint>(
        &mut self,
    ) -> ArrVecApi<ImplArr![T; ops::SatSub<N, I>]> {
        let len = self.len();
        let i = match uint::to_usize::<I>() {
            Some(i) if i <= len => i,
            _ => return ArrVecApi::new(),
        };
        // SAFETY: `i < len`. After this, `len - i` elements are diwowned and valid
        unsafe { self.set_len(i) }

        let spare = ArrApi::from_slice(self.spare_capacity()).unwrap();
        // SAFETY: MaybeUninit copy, taking ownership of `len - i` valid, disowned elements
        unsafe { ArrVec::from_uninit_parts(core::ptr::read(spare), len - i) }
    }
}

impl<T, N: Uint, A: Array<Item = T, Length = N>> ArrVecApi<A> {
    pub const fn try_grow<M: Uint>(self) -> Result<ArrVecApi<ImplArr![T; M]>, Self> {
        if let Some(m) = uint::to_usize::<M>()
            && m >= self.len()
        {
            // TODO: How many memcpys does this compile to in debug mode?
            let (arr, len) = self.into_uninit_parts();
            // SAFETY: new cap >= len, so we must still have `len` initialized elements.
            Ok(unsafe {
                ArrVecApi::from_uninit_parts(crate::array::Arr::resize_uninit_from(arr), len)
            })
        } else {
            Err(self)
        }
    }
}
