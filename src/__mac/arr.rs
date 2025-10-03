use crate::array::*;

#[repr(transparent)]
pub struct ArrDrop<T>(pub T);

impl<A: Array> ArrDrop<A> {
    pub const fn enter(self) -> ArrDrop<ArrVecApi<A>> {
        // SAFETY: repr(transparent)
        unsafe {
            ArrDrop(ArrVecApi::new_full(crate::utils::union_transmute!(
                ArrDrop<A>,
                A,
                self,
            )))
        }
    }
}
impl<A: Array> ArrDrop<ArrVecApi<A>> {
    pub const fn enter(self) -> Self {
        self
    }
    pub const fn needs_drop(&self) -> bool {
        const { core::mem::needs_drop::<A::Item>() }
    }
    pub const fn pop_next(&mut self) -> Option<A::Item> {
        self.0.pop()
    }
    pub const fn discard(self) {
        debug_assert!(!self.needs_drop() || self.0.is_empty());
        core::mem::forget(self);
    }
}
impl<A: Array> ArrDrop<ArrDeqApi<A>> {
    pub const fn enter(self) -> Self {
        self
    }
    pub const fn needs_drop(&self) -> bool {
        const { core::mem::needs_drop::<A::Item>() }
    }
    pub const fn pop_next(&mut self) -> Option<A::Item> {
        self.0.pop_front()
    }
    pub const fn discard(self) {
        debug_assert!(!self.needs_drop() || self.0.is_empty());
        core::mem::forget(self);
    }
}
