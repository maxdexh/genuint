use core::{marker::PhantomData, mem::MaybeUninit};

use super::{ArrApi, Array};

#[repr(transparent)]
pub struct ArrDeqDrop<A: Array<Item = T>, T = <A as Array>::Item>(ArrDeqRepr<A>, PhantomData<T>);

pub struct ArrDeqRepr<A: Array> {
    arr: ArrApi<MaybeUninit<A>>,
    head: usize,
    len: usize,
}
