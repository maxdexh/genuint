use core::marker::PhantomData;

/// # Safety
/// Cannot be implemented by downstream crates
pub unsafe trait Array {
    type Item;
    type Length: crate::Uint;
}

mod arr_utils;

unsafe impl<A: Array> Array for core::mem::ManuallyDrop<A> {
    type Item = core::mem::ManuallyDrop<A::Item>;
    type Length = A::Length;
}
unsafe impl<A: Array> Array for core::mem::MaybeUninit<A> {
    type Item = core::mem::MaybeUninit<A::Item>;
    type Length = A::Length;
}
unsafe impl<A: Array> Array for ArrApi<A> {
    type Item = A::Item;
    type Length = A::Length;
}

pub use crate::internals::arr_reexports::*;

mod core_impl;
mod extra_impl;

#[cfg_attr(not(doc), repr(transparent))]
pub struct ArrApi<A: Array<Item = T>, T = <A as Array>::Item>(A, PhantomData<T>);

mod arr_vec;

#[cfg_attr(not(doc), repr(transparent))]
pub struct ArrVec<A: Array<Item = T>, T = <A as Array>::Item>(
    arr_vec::ArrVecDrop<A>,
    PhantomData<T>,
);

mod arr_deq;

#[repr(transparent)]
pub struct ArrDeq<A: Array<Item = T>, T = <A as Array>::Item>(
    arr_deq::ArrDeqDrop<A>,
    PhantomData<T>,
);

#[macro_export]
#[doc(hidden)]
macro_rules! __drop_items {
    [ $arr:expr ] => {{
        let mut __guard = $crate::__mac::ArrDrop($arr).enter();
        while __guard.has_next() {
            let _ = __guard.pop_next();
        }
        __guard.discard();
    }};
}
pub use __drop_items as drop_items;
