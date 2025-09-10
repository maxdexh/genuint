use crate::array::{extra::*, *};

impl<T, A> AsRef<[T]> for ArrApi<A>
where
    A: Array<Item = T>,
{
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}
impl<T, A> AsMut<[T]> for ArrApi<A>
where
    A: Array<Item = T>,
{
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}
impl<T, A> core::borrow::Borrow<[T]> for ArrApi<A>
where
    A: Array<Item = T>,
{
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}
impl<T, A> core::borrow::BorrowMut<[T]> for ArrApi<A>
where
    A: Array<Item = T>,
{
    fn borrow_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}
impl<'a, T, A> TryFrom<&'a [T]> for &'a ArrApi<A>
where
    A: Array<Item = T>,
{
    type Error = TryFromSliceError;
    fn try_from(value: &'a [T]) -> Result<Self, Self::Error> {
        ArrApi::try_from_slice(value).ok_or(TryFromSliceError(()))
    }
}
impl<'a, T, A> TryFrom<&'a mut [T]> for &'a mut ArrApi<A>
where
    A: Array<Item = T>,
{
    type Error = TryFromSliceError;
    fn try_from(value: &'a mut [T]) -> Result<Self, Self::Error> {
        ArrApi::try_from_mut_slice(value).ok_or(TryFromSliceError(()))
    }
}
impl<T, A> TryFrom<&[T]> for ArrApi<A>
where
    A: Array<Item = T>,
    T: Copy,
{
    type Error = TryFromSliceError;
    fn try_from(value: &[T]) -> Result<Self, Self::Error> {
        <&crate::array::CopyArr<_, _>>::try_from(value)
            .copied()
            .map(ArrApi::retype)
    }
}
impl<T, A> TryFrom<&mut [T]> for ArrApi<A>
where
    A: Array<Item = T>,
    T: Copy,
{
    type Error = TryFromSliceError;
    fn try_from(value: &mut [T]) -> Result<Self, Self::Error> {
        (value as &[T]).try_into()
    }
}

#[cfg(feature = "alloc")]
impl<'a, T, A> From<&'a ArrApi<A>> for alloc::borrow::Cow<'a, [T]>
where
    A: Array<Item = T>,
    T: Clone,
{
    fn from(value: &'a ArrApi<A>) -> Self {
        value.as_slice().into()
    }
}

#[cfg(feature = "alloc")]
impl<'a, T, A> From<&'a ArrApi<A>> for alloc::vec::Vec<T>
where
    A: Array<Item = T>,
    T: Clone,
{
    fn from(value: &'a ArrApi<A>) -> Self {
        value.as_slice().into()
    }
}
#[cfg(feature = "alloc")]
impl<T, A> From<&mut ArrApi<A>> for alloc::vec::Vec<T>
where
    A: Array<Item = T>,
    T: Clone,
{
    fn from(value: &mut ArrApi<A>) -> Self {
        value.as_slice().into()
    }
}
#[cfg(feature = "alloc")]
impl<T, A> From<ArrApi<A>> for alloc::sync::Arc<[T]>
where
    A: Array<Item = T>,
{
    fn from(value: ArrApi<A>) -> Self {
        alloc::sync::Arc::new(value).unsize_arc()
    }
}
#[cfg(feature = "alloc")]
impl<T, A> From<ArrApi<A>> for alloc::rc::Rc<[T]>
where
    A: Array<Item = T>,
{
    fn from(value: ArrApi<A>) -> Self {
        alloc::rc::Rc::new(value).unsize_rc()
    }
}
#[cfg(feature = "alloc")]
impl<T, A> From<ArrApi<A>> for alloc::boxed::Box<[T]>
where
    A: Array<Item = T>,
{
    fn from(value: ArrApi<A>) -> Self {
        alloc::boxed::Box::new(value).unsize_box()
    }
}
#[cfg(feature = "alloc")]
impl<T, A> From<ArrApi<A>> for alloc::vec::Vec<T>
where
    A: Array<Item = T>,
{
    fn from(value: ArrApi<A>) -> Self {
        <[T]>::into_vec(value.into())
    }
}
#[cfg(feature = "alloc")]
impl<T, A> From<ArrApi<A>> for alloc::collections::VecDeque<T>
where
    A: Array<Item = T>,
{
    fn from(value: ArrApi<A>) -> Self {
        alloc::vec::Vec::from(value).into()
    }
}
#[cfg(feature = "std")]
impl<K, V, A> From<ArrApi<A>> for std::collections::HashMap<K, V>
where
    A: Array<Item = (K, V)>,
    K: core::hash::Hash + Eq,
{
    fn from(value: ArrApi<A>) -> Self {
        Self::from_iter(value)
    }
}
#[cfg(feature = "std")]
impl<T, A> From<ArrApi<A>> for std::collections::HashSet<T>
where
    A: Array<Item = T>,
    T: core::hash::Hash + Eq,
{
    fn from(value: ArrApi<A>) -> Self {
        Self::from_iter(value)
    }
}
#[cfg(feature = "alloc")]
impl<K, V, A> From<ArrApi<A>> for alloc::collections::BTreeMap<K, V>
where
    A: Array<Item = (K, V)>,
    K: Ord,
{
    fn from(value: ArrApi<A>) -> Self {
        Self::from_iter(value)
    }
}
#[cfg(feature = "alloc")]
impl<T, A> From<ArrApi<A>> for alloc::collections::BTreeSet<T>
where
    A: Array<Item = T>,
    T: Ord,
{
    fn from(value: ArrApi<A>) -> Self {
        Self::from_iter(value)
    }
}
#[cfg(feature = "alloc")]
impl<T, A> From<ArrApi<A>> for alloc::collections::BinaryHeap<T>
where
    A: Array<Item = T>,
    T: Ord,
{
    fn from(value: ArrApi<A>) -> Self {
        alloc::vec::Vec::from(value).into()
    }
}
#[cfg(feature = "alloc")]
impl<T, A> From<ArrApi<A>> for alloc::collections::LinkedList<T>
where
    A: Array<Item = T>,
    T: Ord,
{
    fn from(value: ArrApi<A>) -> Self {
        Self::from_iter(value)
    }
}
#[cfg(feature = "alloc")]
impl<T, A> TryFrom<alloc::boxed::Box<[T]>> for alloc::boxed::Box<ArrApi<A>>
where
    A: Array<Item = T>,
{
    type Error = alloc::boxed::Box<[T]>;
    fn try_from(value: alloc::boxed::Box<[T]>) -> Result<Self, Self::Error> {
        use alloc::boxed::Box;
        if value.len() == helper::arr_len::<A>() {
            // SAFETY: Length is correct; casting slices to arrays this way is valid
            Ok(unsafe { Box::from_raw(Box::into_raw(value).cast()) })
        } else {
            Err(value)
        }
    }
}
#[cfg(feature = "alloc")]
impl<T, A> TryFrom<alloc::vec::Vec<T>> for ArrApi<A>
where
    A: Array<Item = T>,
{
    type Error = alloc::vec::Vec<T>;
    fn try_from(mut value: alloc::vec::Vec<T>) -> Result<Self, Self::Error> {
        if value.len() == helper::arr_len::<A>() {
            // SAFETY: set_len(0) is always safe and effectively forgets the elements,
            // ensuring that the drop of `Vec` only frees the allocation.
            unsafe {
                value.set_len(0);
                Ok(core::ptr::read(value.as_ptr().cast()))
            }
        } else {
            Err(value)
        }
    }
}
#[cfg(feature = "alloc")]
impl<T, A> TryFrom<alloc::vec::Vec<T>> for alloc::boxed::Box<ArrApi<A>>
where
    A: Array<Item = T>,
{
    type Error = alloc::vec::Vec<T>;
    fn try_from(value: alloc::vec::Vec<T>) -> Result<Self, Self::Error> {
        if value.len() == helper::arr_len::<A>() {
            value
                .into_boxed_slice()
                .try_into()
                .map_err(|_| unreachable!())
        } else {
            Err(value)
        }
    }
}
