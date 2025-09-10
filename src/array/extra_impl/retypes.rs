use crate::{
    Uint,
    array::{helper::*, *},
    ops, uint,
};

macro_rules! retype_fns {
    ($($const:ident)?, $Self:ty, $Dst:ty, ($infallible:ident, $try:ident) = |$self:ident| $conv_unchecked:expr,) => {
        /// Converts between equivalent array types.
        pub $($const)? fn $infallible<Dst>($self: $Self) -> $Dst
        where
            Dst: crate::array::Array<Item = T, Length = N>,
        {
            $conv_unchecked
        }

        /// Converts between equivalent array types.
        ///
        /// Returns the original array if the lengths do not match.
        pub $($const)? fn $try<Dst>(
            $self: $Self,
        ) -> crate::tern::TernRes<crate::ops::Eq<N, Dst::Length>, $Dst, $Self>
        where
            Dst: crate::array::Array<Item = T>,
        {
            match uint::to_bool::<ops::Eq<N, Dst::Length>>() {
                // SAFETY: Src::Length == Dst::Length
                true => {
                    crate::tern::TernRes::make_ok($conv_unchecked)
                },
                false => crate::tern::TernRes::make_err($self),
            }
        }
    };
}
macro_rules! smartptr_fns {
    (
        $Self:ty,
        $Dst:ty,
        $Slice:ty,
        ($retype:ident, $try_retype:ident) = |$self1:ident| $retype_doit:expr,
        $unsize:ident = |$self2:ident| $unsize_doit:expr,
        $from_slice:ident = |$srcpat:ident| $from_slice_doit:expr,
    ) => {
        retype_fns! {
            ,
            $Self,
            $Dst,
            ($retype, $try_retype) = |$self1| $retype_doit,
        }
        pub fn $unsize($self2: $Self) -> $Slice {
            $unsize_doit
        }
        pub fn $from_slice($srcpat: $Slice) -> Result<$Self, $Slice> {
            match crate::uint::to_usize::<A::Length>() {
                Some(arr_len) if arr_len == $srcpat.len() => Ok($from_slice_doit),
                _ => Err($srcpat),
            }
        }
    };
}

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, rc::Rc, sync::Arc};

impl<T, N: Uint, A: Array<Item = T, Length = N>> ArrApi<A> {
    #[cfg(feature = "alloc")]
    smartptr_fns![
        Box<Self>,
        Box<Dst>,
        Box<[T]>,
        // SAFETY: N == Dst::Length, `Array` invariant
        (retype_box, try_retype_box) = |self| unsafe { Box::from_raw(Box::into_raw(self).cast()) },
        // SAFETY: `Array` to slice cast
        unsize_box = |self| unsafe { Box::from_raw(unsize_raw_mut(Box::into_raw(self))) },
        // SAFETY:
        // - Pointer cast with same item and length.
        // - Ownership is transferred through into_raw followed by from_raw.
        // - This is the same as the `$ptr<[T; N]> as TryFrom<$ptr<[T]>>` impl
        from_boxed_slice = |slice| unsafe { Box::from_raw(Box::into_raw(slice).cast()) },
    ];
    #[cfg(feature = "alloc")]
    smartptr_fns![
        Rc<Self>,
        Rc<Dst>,
        Rc<[T]>,
        // SAFETY: N == Dst::Length, `Array` invariant
        (retype_rc, try_retype_rc) = |self| unsafe { Rc::from_raw(Rc::into_raw(self).cast()) },
        // SAFETY: `Array` to slice cast
        unsize_rc = |self| unsafe { Rc::from_raw(unsize_raw(Rc::into_raw(self))) },
        // SAFETY:
        // - Pointer cast with same item and length.
        // - Ownership is transferred through into_raw followed by from_raw.
        // - This is the same as the `$ptr<[T; N]> as TryFrom<$ptr<[T]>>` impl
        from_rc_slice = |slice| unsafe { Rc::from_raw(Rc::into_raw(slice).cast()) },
    ];
    #[cfg(feature = "alloc")]
    smartptr_fns![
        Arc<Self>,
        Arc<Dst>,
        Arc<[T]>,
        // SAFETY: N == Dst::Length, `Array` invariant
        (retype_arc, try_retype_arc) = |self| unsafe { Arc::from_raw(Arc::into_raw(self).cast()) },
        // SAFETY: `Array` to slice cast
        unsize_arc = |self| unsafe { Arc::from_raw(unsize_raw(Arc::into_raw(self))) },
        // SAFETY:
        // - Pointer cast with same item and length.
        // - Ownership is transferred through into_raw followed by from_raw.
        // - This is the same as the `$ptr<[T; N]> as TryFrom<$ptr<[T]>>` impl
        from_arc_slice = |slice| unsafe { Arc::from_raw(Arc::into_raw(slice).cast()) },
    ];

    retype_fns![
        const,
        &Self,
        &Dst,
        // SAFETY: N == Dst::Length, `Array` invariant
        (retype_ref, try_retype_ref) = |self| unsafe { &*core::ptr::from_ref(self).cast() },
    ];
    retype_fns![
        const,
        &mut Self,
        &mut Dst,
        // SAFETY: N == Dst::Length, `Array` invariant
        (retype_mut, try_retype_mut) = |self| unsafe { &mut *core::ptr::from_mut(self).cast() },
    ];
}
