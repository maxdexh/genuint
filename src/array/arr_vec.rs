use core::{marker::PhantomData, mem::MaybeUninit};

use super::{ArrApi, Array};

#[repr(transparent)]
pub struct ArrVecDrop<A: Array<Item = T>, T = <A as Array>::Item>(ArrVecRepr<A>, PhantomData<T>);

pub struct ArrVecRepr<A: Array> {
    arr: ArrApi<MaybeUninit<A>>,
    len: usize,
}
