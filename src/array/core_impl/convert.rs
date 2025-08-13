use crate::array::{ArrApi, Array, extra::*};

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
// TODO: Reconsider these, since they are fallible and arrays don't actually implement deref
#[cfg(any())]
impl<A: Array> core::ops::Deref for ArrApi<A> {
    type Target = [A::Item];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
#[cfg(any())]
impl<A: Array> core::ops::DerefMut for ArrApi<A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<'a, T, A> TryFrom<&'a [T]> for &'a ArrApi<A>
where
    A: Array<Item = T>,
{
    type Error = TryFromSliceError;
    fn try_from(value: &'a [T]) -> Result<Self, Self::Error> {
        from_slice(value)
    }
}
impl<'a, T, A> TryFrom<&'a mut [T]> for &'a mut ArrApi<A>
where
    A: Array<Item = T>,
{
    type Error = TryFromSliceError;
    fn try_from(value: &'a mut [T]) -> Result<Self, Self::Error> {
        from_mut_slice(value)
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
            .map(ArrApi::into_arr)
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

macro_rules! tuple_impl {
    () => {};
    ($F:ident $($T:ident)*) => {
        tuple_impl!($($T)*);

        const _: () = {
            const COUNT: usize = 1 $(+ { stringify!($T); 1 })*;
            impl<A, $F> From<ArrApi<A>> for ($F, $($T),*)
            where
                A: Array<Item = $F, Length = crate::uint::FromUsize<COUNT>>,
            {
                fn from(value: ArrApi<A>) -> Self {
                    crate::array::extra::arr_convert::<_, [_; COUNT]>(value).into()
                }
            }
            impl<A, $F> From<($F, $($T),*)> for ArrApi<A>
            where
                A: Array<Item = $F, Length = crate::uint::FromUsize<COUNT>>
            {
                fn from(value: ($F, $($T),*)) -> Self {
                    crate::array::extra::arr_convert(<[_; COUNT]>::from(value))
                }
            }
        };
    };
}
tuple_impl! {
    T T T T
    T T T T
    T T T T
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
        use alloc::sync::Arc;

        let mut arc = Self::new_uninit_slice(arr_len::<A>());

        let buf = Arc::get_mut(&mut arc).unwrap();
        *from_mut_slice(buf).unwrap() = core::mem::MaybeUninit::new(value);

        unsafe { arc.assume_init() }
    }
}
#[cfg(feature = "alloc")]
impl<T, A> From<ArrApi<A>> for alloc::rc::Rc<[T]>
where
    A: Array<Item = T>,
{
    fn from(value: ArrApi<A>) -> Self {
        use alloc::rc::Rc;

        let mut arc = Self::new_uninit_slice(arr_len::<A>());

        let buf = Rc::get_mut(&mut arc).unwrap();
        *from_mut_slice(buf).unwrap() = core::mem::MaybeUninit::new(value);

        unsafe { arc.assume_init() }
    }
}
#[cfg(feature = "alloc")]
impl<T, A> From<ArrApi<A>> for alloc::boxed::Box<[T]>
where
    A: Array<Item = T>,
{
    fn from(value: ArrApi<A>) -> Self {
        let mut out = Self::new_uninit_slice(arr_len::<A>());
        *from_mut_slice(&mut out).unwrap() = core::mem::MaybeUninit::new(value);

        unsafe { out.assume_init() }
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
        if value.len() == arr_len::<A>() {
            Ok(unsafe { Self::from_raw(alloc::boxed::Box::into_raw(value).cast()) })
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
        if value.len() == arr_len::<A>() {
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
        if value.len() == arr_len::<A>() {
            value
                .into_boxed_slice()
                .try_into()
                .map_err(|_| unreachable!())
        } else {
            Err(value)
        }
    }
}
