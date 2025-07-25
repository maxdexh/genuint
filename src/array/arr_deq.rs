use core::{marker::PhantomData, mem::MaybeUninit};

use super::{ArrApi, ArrDeq, Array, arr_utils::*};

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
        check_len::<A>()
    }
    const fn phys_idx_of(&self, idx: usize) -> usize {
        phys_idx(repr!(self).head.wrapping_add(idx), Self::check_cap())
    }
    const fn phys_idx_of_back(&self, idx: usize) -> usize {
        let cap = Self::check_cap();
        phys_idx(repr!(self).head.wrapping_sub(idx).wrapping_add(cap), cap)
    }
    const unsafe fn phys_read(&self, idx: usize) -> T {
        unsafe { repr!(self).arr.as_slice()[idx].assume_init_read() }
    }
    const unsafe fn virt_read(&self, idx: usize) -> T {
        unsafe { self.phys_read(self.phys_idx_of(idx)) }
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
                len: check_len::<A>(),
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
}
