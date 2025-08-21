use crate::array::*;

#[repr(transparent)]
pub struct ArrDrop<T>(pub T);

impl<A: Array> ArrDrop<A> {
    pub const fn enter(self) -> ArrDrop<ArrVecApi<A>> {
        // SAFETY: repr(transparent)
        let inner: A = unsafe { crate::utils::exact_transmute(self) };
        ArrDrop(ArrVecApi::full(inner))
    }
}
impl<A: Array> ArrDrop<ArrVecApi<A>> {
    pub const fn enter(self) -> Self {
        self
    }
    pub const fn has_next(&self) -> bool {
        !self.0.is_empty()
    }
    pub const fn pop_next(&mut self) -> A::Item {
        self.0.pop().unwrap()
    }
    pub const fn discard(self) {
        debug_assert!(self.0.is_empty());
        core::mem::forget(self);
    }
}
impl<A: Array> ArrDrop<ArrDeqApi<A>> {
    pub const fn enter(self) -> Self {
        self
    }
    pub const fn has_next(&self) -> bool {
        !self.0.is_empty()
    }
    pub const fn pop_next(&mut self) -> A::Item {
        self.0.pop_front().unwrap()
    }
    pub const fn discard(self) {
        debug_assert!(self.0.is_empty());
        core::mem::forget(self);
    }
}
