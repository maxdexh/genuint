use core::{marker::PhantomData, mem::MaybeUninit};

use super::{ArrApi, ArrDeq, Array, extra::*};

#[repr(transparent)]
pub struct ArrDeqDrop<A: Array<Item = T>, T = <A as Array>::Item>(ArrDeqRepr<A>, PhantomData<T>);
impl<A: Array<Item = T>, T> Drop for ArrDeqDrop<A, T> {
    fn drop(&mut self) {
        unsafe {
            let deq = &mut *(&raw mut *self).cast::<ArrDeq<A>>();
            let (lhs, rhs) = deq.as_mut_slices();
            core::ptr::drop_in_place(lhs);
            core::ptr::drop_in_place(rhs);
        }
    }
}

pub struct ArrDeqRepr<A: Array> {
    head: usize,
    len: usize,
    arr: ArrApi<MaybeUninit<A>>,
}
macro_rules! repr {
    ($self:expr) => {
        $self.0.0
    };
}

impl<A: Array<Item = T>, T> ArrDeq<A> {
    const fn check_cap() -> usize {
        arr_len::<A>()
    }
    const fn phys_idx_of(&self, idx: usize) -> usize {
        wrapping_idx(self.head().wrapping_add(idx), Self::check_cap())
    }
    const fn phys_idx_of_back(&self, idx: usize) -> usize {
        let cap = Self::check_cap();
        wrapping_idx(self.head().wrapping_sub(idx).wrapping_add(cap), cap)
    }
    const unsafe fn phys_read(&self, idx: usize) -> T {
        unsafe { repr!(self).arr.as_slice()[idx].assume_init_read() }
    }
    const unsafe fn virt_read(&self, idx: usize) -> T {
        unsafe { self.phys_read(self.phys_idx_of(idx)) }
    }
    const fn head(&self) -> usize {
        repr!(self).head
    }
}

impl<A: Array<Item = T>, T> ArrDeq<A> {
    const unsafe fn from_repr(repr: ArrDeqRepr<A>) -> Self {
        Self(ArrDeqDrop(repr, PhantomData), PhantomData)
    }
    pub const fn new() -> Self {
        unsafe {
            Self::from_repr(ArrDeqRepr {
                arr: ArrApi::new(MaybeUninit::uninit()),
                head: 0,
                len: 0,
            })
        }
    }
    pub const fn full(full: A) -> Self {
        unsafe {
            Self::from_repr(ArrDeqRepr {
                arr: ArrApi::new(MaybeUninit::new(full)),
                head: 0,
                len: arr_len::<A>(),
            })
        }
    }
    pub const fn len(&self) -> usize {
        repr!(self).len
    }
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub const fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        repr!(self).len -= 1;
        let popped = repr!(self).head;
        repr!(self).head = self.phys_idx_of(1);
        Some(unsafe { self.phys_read(popped) })
    }
    pub const fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        repr!(self).len -= 1;
        Some(unsafe { self.virt_read(repr!(self).len) })
    }
    pub const fn as_mut_slices(&mut self) -> (&mut [T], &mut [T]) {
        todo!()
    }
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
            // EFGHIJKLMNABCD
            reverse(lhs);
            // NMLKJIHGFEABCD
            reverse(rhs);
            // NMLKJIHGFEDBCA
            reverse(slice);
            // ABCDEFGHIJKLMN
        }

        let ArrDeqRepr { head, len: _, arr } = &mut repr!(self);
        rotate_left(arr.as_mut_slice(), *head);
        *head = 0;
        self.as_mut_slices().0
    }
}
