use core::marker::PhantomData;

/// # Safety
/// Cannot be implemented by downstream crates
pub unsafe trait Array {
    type Item;
    type Length: crate::Uint;
}

unsafe impl<A: Array> Array for core::mem::ManuallyDrop<A> {
    type Item = core::mem::ManuallyDrop<A::Item>;
    type Length = A::Length;
}
unsafe impl<A: Array> Array for core::mem::MaybeUninit<A> {
    type Item = core::mem::MaybeUninit<A::Item>;
    type Length = A::Length;
}

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
