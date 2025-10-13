use core::marker::PhantomData;

use crate::{ToUint, condty::*, uint, utils};

pub use Result;

// SAFETY INVARIANT: `COND = uint::is_nonzero::<C>()`
// Conversely, the existence of an instance of this type proves the above statement!
pub struct CtxCond<C, const COND: bool> {
    _p: PhantomData<C>,
}

pub const fn hold<C: crate::ToUint>() -> Result<CtxCond<C, true>, CtxCond<C, false>> {
    if uint::is_nonzero::<C>() {
        Ok(CtxCond::<C, true> { _p: PhantomData })
    } else {
        Err(CtxCond::<C, false> { _p: PhantomData })
    }
}

impl<C: ToUint> CtxCond<C, true> {
    #[inline(always)]
    pub const fn new_true<T, F>(&self, t: T) -> CondDirect<C, T, F> {
        // SAFETY: C is nonzero, so `CondDirect<C, T, F> = T`
        unsafe { utils::same_type_transmute!(T, CondDirect::<C, T, F>, t) }
    }
    #[inline(always)]
    pub const fn new_ok<T, E>(&self, ok: T) -> CondResult<C, T, E> {
        CondResult {
            direct: self.new_true(ok),
        }
    }
    #[inline(always)]
    pub const fn new_some<T>(&self, some: T) -> CondOption<C, T> {
        CondOption {
            direct: self.new_true(some),
        }
    }
    #[inline(always)]
    pub const fn unwrap_true<T, F>(&self, t: CondDirect<C, T, F>) -> T {
        // SAFETY: C is nonzero, so `CondDirect<C, T, F> = T`
        unsafe { utils::same_type_transmute!(CondDirect::<C, T, F>, T, t) }
    }
    #[inline(always)]
    pub const fn unwrap_ok<T, E>(&self, ok: CondResult<C, T, E>) -> T {
        self.unwrap_true(ok.into_direct())
    }
    #[inline(always)]
    pub const fn unwrap_some<T>(&self, some: CondOption<C, T>) -> T {
        self.unwrap_true(some.into_direct())
    }
}
impl<C: ToUint> CtxCond<C, false> {
    #[inline(always)]
    pub const fn new_false<T, F>(&self, f: F) -> CondDirect<C, T, F> {
        // SAFETY: C is zero, so `CondDirect<C, T, F> = F`
        unsafe { utils::same_type_transmute!(F, CondDirect::<C, T, F>, f) }
    }
    #[inline(always)]
    pub const fn new_err<T, E>(&self, err: E) -> CondResult<C, T, E> {
        CondResult {
            direct: self.new_false(err),
        }
    }
    #[inline(always)]
    pub const fn new_none<T>(&self) -> CondOption<C, T> {
        CondOption {
            direct: self.new_false(()),
        }
    }
    #[inline(always)]
    pub const fn unwrap_false<T, F>(&self, f: CondDirect<C, T, F>) -> F {
        // SAFETY: C is zero, so `CondDirect<C, T, F> = F`
        unsafe { utils::same_type_transmute!(CondDirect::<C, T, F>, F, f) }
    }
    #[inline(always)]
    pub const fn unwrap_err<T, E>(&self, err: CondResult<C, T, E>) -> E {
        self.unwrap_false(err.into_direct())
    }
    #[inline(always)]
    pub const fn drop_none<T>(&self, none: CondOption<C, T>) {
        self.unwrap_false(none.into_direct())
    }
}
