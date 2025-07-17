use core::{marker::PhantomData, mem::MaybeUninit};

use super::{ArrApi, ArrDeq, Array, arr_utils::*};

#[repr(transparent)]
pub struct ArrDeqDrop<A: Array<Item = T>, T = <A as Array>::Item>(ArrDeqRepr<A>, PhantomData<T>);

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
    const fn phys_idx(&self, idx: usize) -> usize {
        phys_idx(repr!(self).head.wrapping_add(idx), Self::check_cap())
    }
    const fn phys_idx_back(&self, idx: usize) -> usize {
        let cap = Self::check_cap();
        phys_idx(repr!(self).head.wrapping_sub(idx).wrapping_add(cap), cap)
    }
    const fn update_head(&mut self, new_head: usize) -> usize {
        let head = repr!(self).head;
        repr!(self).head = new_head;
        head
    }
    const unsafe fn phys_read(&self, idx: usize) -> T {
        unsafe { repr!(self).arr.as_slice()[idx].assume_init_read() }
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
        let popped = self.update_head(self.phys_idx(1));
        repr!(self).len -= 1;
        Some(unsafe { self.phys_read(popped) })
    }
}
