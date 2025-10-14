use core::marker::PhantomData;

use crate::{Uint, array::*, const_fmt, uint};

#[track_caller]
pub(crate) const fn arr_len<A: Array>() -> usize {
    const fn doit<N: Uint>() -> usize {
        let precalc = const {
            match uint::to_usize::<N>() {
                Some(n) => Ok(n),
                None => Err(const_fmt::fmt![
                    "Array length ",
                    PhantomData::<N>,
                    " exceeds the maximum value for a usize",
                ]),
            }
        };
        match precalc {
            Ok(n) => n,
            Err(err) => err.panic(),
        }
    }
    doit::<A::Length>()
}

/// Checks some invariants of an array type.
///
/// Note that the array's [`size_of`] is used by this function. If the array type in question
/// exceeds the maximum size for the architecture, this will by itself cause a
/// post-monomorphization error.
pub(crate) const fn arr_impl_ubcheck<A: Array>() {
    #[cfg(debug_assertions)]
    const {
        assert!(
            align_of::<A>() == align_of::<A::Item>(),
            "UB: Array alignment must be the same as that of item"
        );
        let item_size = size_of::<A::Item>();
        let arr_size = size_of::<A>();
        if let Some(arr_len) = uint::to_usize::<A::Length>() {
            let calc_size = arr_len.checked_mul(item_size);
            assert!(
                calc_size.is_some() && arr_size == calc_size.unwrap(),
                "UB: Array size must be equal to item size multiplied by length"
            )
        } else {
            assert!(
                item_size == 0 && arr_size == 0,
                "UB: Array with length exceeding usize::MAX must be ZST"
            )
        }
    }
}

/// # Panics
/// If `A::Length > usize::MAX`
///
/// # Safety
/// This operation is strictly the same as [`core::ptr::slice_from_raw_parts_mut`] with `ptr.cast()` as
/// the first argument and [`ArrApi::<A>::length()`] as the second.
///
/// Due to the guarantees made by [`Array`], this should generally mean that the returned pointer
/// is valid for the same operations as `ptr`. In particular, if `ptr` is valid for some operation
/// on `A::Length`  values of `A::Item` with array layout, then the returned pointer is valid for
/// that operation on the corresponding slice.
#[track_caller]
pub(crate) const fn unsize_raw_mut<A: Array>(ptr: *mut A) -> *mut [A::Item] {
    core::ptr::slice_from_raw_parts_mut(ptr.cast(), arr_len::<A>())
}

pub(crate) mod oversize {
    use super::*;
    use crate::{condty, uops, utils};

    pub type PopDigit<N> = uint::From<uops::_Shr<N, crate::consts::PtrWidth>>;

    #[utils::apply(uops::lazy)]
    pub type _DigitLenRec<N> = _DigitLen<PopDigit<N>>;
    #[utils::apply(uops::lazy)]
    pub type _DigitLen<N> = uops::If<
        N,
        uops::_Inc<_DigitLenRec<N>>, //
        uint::lit!(0),
    >;
    pub type DigitLen<N> = uint::From<_DigitLen<N>>;

    pub(crate) struct BigCounter<N: Uint> {
        /// # Safety
        /// Represents a number in base `usize::MAX + 1`.
        /// Little-endian, i.e. least significant digit at index 0.
        ///
        /// Must be less than or equal to N
        digits: CopyArr<usize, DigitLen<N>>,
    }
    impl<N: Uint> BigCounter<N> {
        pub const fn dec(&mut self) -> bool {
            if self.is_zero() {
                return false;
            }
            let mut digits = self.digits.as_mut_slice();
            while let [lsd, rest @ ..] = digits {
                digits = rest;
                let ovfl;
                (*lsd, ovfl) = lsd.overflowing_sub(1);
                if !ovfl {
                    break;
                }
            }
            true
        }
        pub const unsafe fn inc_unchecked(&mut self) {
            let mut digits = self.digits.as_mut_slice();
            while let [lsd, rest @ ..] = digits {
                digits = rest;
                let ovfl;
                (*lsd, ovfl) = lsd.overflowing_add(1);
                if !ovfl {
                    return;
                }
            }
        }
        pub const fn is_zero(&self) -> bool {
            let mut digits = self.digits.as_slice();
            while let &[ref rest @ .., last] = digits {
                digits = rest;
                if last != 0 {
                    return false;
                }
            }
            true
        }
        pub const fn zero() -> Self {
            Self {
                digits: CopyArr::of(0),
            }
        }
        pub const fn max() -> Self {
            Self {
                digits: const {
                    if uint::is_nonzero::<N>() {
                        BigCounter::<PopDigit<N>>::max()
                            .digits
                            .concat([uint::to_usize_overflowing::<N>().0])
                            .try_retype()
                            .unwrap()
                    } else {
                        CopyArr::of(0)
                    }
                },
            }
        }
    }

    /// # Safefty
    /// It must be safe to conceive the ZST `T` from nothing.
    const unsafe fn conjure_zst<T>() -> T {
        debug_assert!(const { size_of::<T>() == 0 });

        // SAFETY: By definition. Reading ZSTs from dangling is legal.
        unsafe { core::ptr::dangling::<T>().read() }
    }
    pub(crate) struct InstanceCounter<T, N: Uint> {
        /// # Safety
        /// - This type owns as many instances as indicated by the value represented by `digits`
        /// - `T` must be a ZST
        counter: BigCounter<N>,
        _p: PhantomData<T>,
    }
    impl<T, N: Uint> Drop for InstanceCounter<T, N> {
        fn drop(&mut self) {
            while self.pop().is_some() {}
        }
    }
    impl<T, N: Uint> InstanceCounter<T, N> {
        pub const fn full(arr: impl Array<Item = T, Length = N>) -> Self {
            assert!(size_of::<T>() == 0);
            core::mem::forget(arr);
            // SAFETY: Array of `N` instances was forgotten, so this is logically
            // equivalent to moving them into a new container.
            Self {
                counter: BigCounter::max(),
                _p: PhantomData,
            }
        }
        pub const fn empty() -> Self {
            assert!(size_of::<T>() == 0);
            // SAFETY: An empty container is trivially safe to create.
            Self {
                counter: BigCounter::zero(),
                _p: PhantomData,
            }
        }
        pub const fn pop(&mut self) -> Option<T> {
            match self.counter.dec() {
                // SAFETY: Counter was decremented, so creating one instance from nothing
                // is logically equivalent to moving it out of the container.
                true => Some(unsafe { conjure_zst() }),
                false => None,
            }
        }
        /// # Safety
        /// Counter must be smaller than `N`.
        pub const unsafe fn push_unchecked(&mut self, item: T) {
            // SAFETY:
            // - `inc_unchecked` is safe because the counter is smaller than `N`
            // - incrementing the instance count is safe because an instance was
            //   because it is logically equivalent to moving the forgotten
            //   instance into the container.
            unsafe {
                core::mem::forget(item);
                self.counter.inc_unchecked()
            }
        }
    }

    pub(crate) struct ArrBuilder<A: Array> {
        /// # Safety
        /// If BigCounter, then this is a container with up to N instances
        /// of T, where the value of the counter is the number of free slots.
        #[allow(clippy::complexity)]
        inner: condty::CondResult<
            PopDigit<A::Length>,                 // if N is oversized
            InstanceCounter<A::Item, A::Length>, // use a counter
            ArrVecApi<A>,                        // else a vec
        >,
    }
    impl<A: Array> ArrBuilder<A> {
        pub const fn new() -> Self {
            Self {
                inner: condty::ctx!(
                    |c| c.new_ok(InstanceCounter::empty()),
                    |c| c.new_err(ArrVecApi::new()), //
                ),
            }
        }
        /// # Safety
        /// This builder must have `A::Length` elements.
        pub unsafe fn into_full_unchecked(self) -> A {
            condty::ctx!(
                // SAFETY: The counter is maxed out, so this is logically equivalent
                // to moving the instances out of the container.
                |_| unsafe {
                    core::mem::forget(self);
                    conjure_zst()
                },
                |c| c.unwrap_err(self.inner).assert_full(),
            )
        }
        /// # Safety
        /// This builder must have fewer than `A::Length` elements.
        pub const unsafe fn push_unchecked(&mut self, item: A::Item) {
            let inner = self.inner.as_mut();
            condty::ctx!(
                // SAFETY:
                |c| unsafe { c.unwrap_ok(inner).push_unchecked(item) },
                |c| c.unwrap_err(inner).push(item), //
            )
        }
    }

    pub(crate) struct ArrConsumer<A: Array> {
        #[allow(clippy::complexity)]
        inner: condty::CondResult<
            PopDigit<A::Length>,                 // if Length is oversized
            InstanceCounter<A::Item, A::Length>, // use a counter
            ArrDeqApi<A>,                        // else a deque
        >,
    }
    impl<A: Array> ArrConsumer<A> {
        pub const fn new(arr: A) -> Self {
            Self {
                inner: condty::ctx!(
                    |c| c.new_ok(InstanceCounter::full(arr)),
                    |c| c.new_err(ArrDeqApi::new_full(arr)), //
                ),
            }
        }
        pub const fn next(&mut self) -> Option<A::Item> {
            let inner = self.inner.as_mut();
            condty::ctx!(
                |c| c.unwrap_ok(inner).pop(), //
                |c| c.unwrap_err(inner).pop_front(),
            )
        }
    }

    pub(crate) struct ArrRefConsumer<'a, T, N: Uint> {
        inner: condty::CondResult<
            PopDigit<N>,            // if oversized
            (BigCounter<N>, &'a T), // yield the same reference N times
            &'a [T],                // else yield from a slice
        >,
    }
    impl<'a, T, N: Uint> ArrRefConsumer<'a, T, N> {
        pub const fn new<A>(arr: &'a A) -> Self
        where
            A: Array<Item = T, Length = N>,
        {
            const { arr_impl_ubcheck::<A>() }

            Self {
                inner: condty::ctx!(
                    |c| c.new_ok((
                        BigCounter::max(),
                        // SAFETY: array length is nonzero, so this points to the first item.
                        unsafe { &*core::ptr::from_ref(arr).cast() },
                    )),
                    |c| c.new_err(arr_api::unsize_ref(arr)),
                ),
            }
        }
        pub const fn next(&mut self) -> Option<&'a T> {
            let inner = self.inner.as_mut();
            condty::ctx!(
                |c| {
                    let (count, r) = c.unwrap_ok(inner);
                    match count.is_zero() {
                        true => None,
                        false => Some(r),
                    }
                },
                |c| {
                    let inner = c.unwrap_err(inner);
                    match inner {
                        [] => None,
                        [next, rest @ ..] => {
                            *inner = rest;
                            Some(next)
                        }
                    }
                }
            )
        }
    }
}
