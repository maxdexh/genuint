use crate::{Uint, array::*};

macro_rules! from_slice_fn {
    (
        $($const:ident)?,
        $ty:ident,
        $doc_slice:expr,
        Result,
        $Result:ident,
        $from_slice:ident,
    ) => {
        #[doc = $doc_slice]
        pub $($const)? fn $from_slice(slice: $ty!(typ, [T])) -> $Result<$ty!(typ, Self), $ty!(typ, [T])> {
            match crate::uint::to_usize::<A::Length>() {
                Some(arr_len) if arr_len == slice.len() => Ok(
                    // SAFETY:
                    // - Pointer cast with same item and length.
                    // - Ownership is transferred through into_raw followed by from_raw.
                    // - This is the same as the `$ptr<[T; N]> as TryFrom<$ptr<[T]>>` impl
                    unsafe { $ty!(from_raw, $ty!(into_raw, slice).cast()) }
                ),
                _ => Err(slice),
            }
        }
    };
    (
        $($const:ident)?,
        $ty:ident,
        $doc_slice:expr,
        Option,
        $Option:ident,
        $from_slice:ident,
    ) => {
        #[doc = $doc_slice]
        pub $($const)? fn $from_slice(slice: $ty!(typ, [T])) -> $Option<$ty!(typ, Self)> {
            match crate::uint::to_usize::<A::Length>() {
                // SAFETY:
                // - Pointer cast with same item and length.
                // - Ownership is transferred through into_raw followed by from_raw.
                // - This is the same as the `$ptr<[T; N]> as TryFrom<$ptr<[T]>>` impl
                Some(arr_len) if arr_len == slice.len() => Some(
                    // SAFETY:
                    // - Pointer cast with same item and length.
                    // - Ownership is transferred through into_raw followed by from_raw.
                    // - This is the same as the `$ptr<[T; N]> as TryFrom<$ptr<[T]>>` impl
                    unsafe { $ty!(from_raw, $ty!(into_raw, slice).cast()) }
                ),
                _ => None,
            }
        }
    };
}
macro_rules! ptr_fns {
    (
        [$($const:ident {})?],
        $ty:ident,
        $doc_from_inner:expr,
        $from_inner:ident,
        $doc_retype:expr,
        $retype:ident,
        $doc_try_retype:expr,
        $try_retype:ident,
        $doc_unsize:expr,
        $unsize:ident,
        $doc_slice:expr,
        $from_slice_out:ident,
        $from_slice:ident,
    ) => {
        #[doc = $doc_from_inner]
        pub $($const)? fn $from_inner<Dst>(inner: $ty!(typ, A)) -> $ty!(typ, Self) {
            // SAFETY: repr(transparent)
            unsafe { $ty!(from_raw, $ty!(into_raw, inner).cast()) }
        }
        #[doc = $doc_retype]
        pub $($const)? fn $retype<Dst>(self: $ty!(typ, Self)) -> $ty!(typ, Dst)
        where
            Dst: crate::array::Array<Item = T, Length = N>,
        {
            // SAFETY: N == Dst::Length, `Array` invariant
            unsafe { $ty!(from_raw, $ty!(into_raw, self).cast()) }
        }
        #[doc = $doc_try_retype]
        pub $($const)? fn $try_retype<Dst>(
            self: $ty!(typ, Self),
        ) -> crate::tern::TernRes<crate::ops::Eq<N, Dst::Length>, $ty!(typ, Dst), $ty!(typ, Self)>
        where
            Dst: crate::array::Array<Item = T>,
        {
            match crate::uint::to_bool::<crate::ops::Eq<N, Dst::Length>>() {
                true => crate::tern::TernRes::make_ok(
                    // SAFETY: Src::Length == Dst::Length
                    unsafe { $ty!(from_raw, $ty!(into_raw, self).cast()) },
                ),
                false => crate::tern::TernRes::make_err(self),
            }
        }
        #[doc = $doc_unsize]
        #[track_caller]
        pub $($const)? fn $unsize(self: $ty!(typ, Self)) -> $ty!(typ, [T]) {
            // SAFETY: `Array` to slice cast
            unsafe { $ty!(from_raw, crate::array::helper::unsize_raw_mut($ty!(into_raw, self))) }
        }
        from_slice_fn![
            $($const)?,
            $ty,
            $doc_slice,
            $from_slice_out,
            $from_slice_out,
            $from_slice,
        ];
    };
}
macro_rules! docexpr {
    ($(#[doc = $doc:expr])*) => {
        core::concat!($($doc, "\n"),*)
    };
}

macro_rules! Ref {
    (typ, $inner:ty) => {
        &$inner
    };
    (from_raw, $ptr:expr) => {
        &*$ptr
    };
    (into_raw, $ptr:expr) => {
        core::ptr::from_ref($ptr).cast_mut()
    };
}
macro_rules! RefMut {
    (typ, $inner:ty) => {
        &mut $inner
    };
    (from_raw, $ptr:expr) => {
        &mut *$ptr
    };
    (into_raw, $ptr:expr) => {
        core::ptr::from_mut($ptr)
    };
}
#[cfg(feature = "alloc")]
macro_rules! Box {
    (typ, $inner:ty) => {
        Box<$inner>
    };
    (from_raw, $ptr:expr) => {
        Box::from_raw($ptr)
    };
    (into_raw, $ptr:expr) => {
        Box::into_raw($ptr)
    };
}
#[cfg(feature = "alloc")]
macro_rules! Rc {
    (typ, $inner:ty) => {
        Rc<$inner>
    };
    (from_raw, $ptr:expr) => {
        Rc::from_raw($ptr)
    };
    (into_raw, $ptr:expr) => {
        Rc::into_raw($ptr).cast_mut()
    };
}
#[cfg(feature = "alloc")]
macro_rules! Arc {
    (typ, $inner:ty) => {
        Arc<$inner>
    };
    (from_raw, $ptr:expr) => {
        Arc::from_raw($ptr)
    };
    (into_raw, $ptr:expr) => {
        Arc::into_raw($ptr).cast_mut()
    };
}

impl<T, N: Uint, A> ArrApi<A>
where
    A: crate::array::Array<Item = T, Length = N>,
{
    ptr_fns![
        [const {}],
        Ref,
        docexpr! {
            /// Casts `&A` to `&Self`.
        },
        from_inner_ref,
        docexpr! {
            /// Casts the array type behind this reference into another [`Array`] type with
            /// the same item type and length.
        },
        retype_ref,
        docexpr! {
            /// Tries to cast the array type behind this reference into another [`Array`] type
            /// with the same item type.
            ///
            /// The conversion succeeds if the length is the same.
        },
        try_retype_ref,
        docexpr! {
            /// Returns a slice containing the entire array.
            ///
            /// In `const` contexts, this is the only way to do this.
            ///
            /// Equivalent of [`<[T; N]>::as_slice`](array::as_slice).
            ///
            /// # Panics
            /// If `N >= usize::MAX`.
        },
        as_slice,
        docexpr! {
            /// Like [`<&[T; N]>::try_from`](primitive.slice.html#impl-TryFrom%3C%26%5BT%5D%3E-for-%26%5BT;+N%5D), but as a const method.
        },
        Option,
        try_from_slice,
    ];

    ptr_fns![
        [const {}],
        RefMut,
        docexpr! {
            /// Casts `&mut A` to `&mut Self`.
        },
        from_inner_mut,
        docexpr! {
            /// Casts the array type behind this reference into another [`Array`] type with
            /// the same item type and length.
        },
        retype_mut,
        docexpr! {
            /// Tries to cast the array type behind this reference into another [`Array`] type
            /// with the same item type.
            ///
            /// The conversion succeeds if the length is the same.
        },
        try_retype_mut,
        docexpr! {
            /// Returns a mutable slice containing the entire array.
            ///
            /// In `const` contexts, this is the only way to do this.
            ///
            /// Equivalent of [`<[T; N]>::as_mut_slice`](array::as_mut_slice).
            ///
            /// # Panics
            /// If `N >= usize::MAX`.
        },
        as_mut_slice,
        docexpr! {
            /// Like [`<&mut [T; N]>::try_from`](primitive.slice.html#impl-TryFrom%3C%26mut+%5BT%5D%3E-for-%26mut+%5BT;+N%5D), but as a const method.
        },
        Option,
        try_from_mut_slice,
    ];
}

#[cfg(feature = "alloc")]
mod smartptrs {
    use super::*;
    use alloc::{boxed::Box, rc::Rc, sync::Arc};

    macro_rules! generic_link {
        ($what:expr, $inner:expr) => {
            core::concat!("[`", $what, "<", $inner, ">`](", $what, ")")
        };
    }
    macro_rules! from_inner_doc {
        ($what:expr) => {
            core::concat!(
                "Casts ",
                generic_link!($what, "A"),
                " to ",
                generic_link!($what, "Self"),
                "."
            )
        };
    }
    macro_rules! retype_doc {
        ($what:expr) => {
            core::concat!(
                "Casts ",
                generic_link!($what, "Self"),
                " to a different ",
                generic_link!($what, "impl Array"),
                " type with the same item type and length.",
            )
        };
    }
    macro_rules! try_retype_doc {
        ($what:expr) => {
            core::concat!(
                "Tries to cast ",
                generic_link!($what, "Self"),
                " to a different ",
                generic_link!($what, "impl Array"),
                " type with the same item type.",
                "\n\n",
                "The operation succeeds if the lengths are the same.",
            )
        };
    }
    macro_rules! from_slice_doc {
        ($what:expr) => {
            core::concat!(
                "Tries to convert from ",
                generic_link!($what, "[T]"),
                " to ",
                generic_link!($what, "Self"),
                ".\n\n",
                "The conversion succeeds if the runtime length of the slice matches the static length of the array.",
            )
        };
    }
    macro_rules! unsize_doc {
        ($what:expr) => {
            core::concat!(
                "Converts from ",
                generic_link!($what, "Self"),
                " into ",
                $what,
                generic_link!($what, "[T]"),
                "\n\n",
                "This is equivalent to the implicit unsize coercion for ",
                generic_link!($what, "[T; N]"),
                ".\n\n",
                "# Panics\nIf `Self::Length > usize::MAX` (only relevant for ZSTs)."
            )
        };
    }

    impl<T, N: Uint, A> ArrApi<A>
    where
        A: Array<Item = T, Length = N>,
    {
        ptr_fns![
            [],
            Box,
            from_inner_doc!("Box"),
            from_inner_box,
            retype_doc!("Box"),
            retype_box,
            try_retype_doc!("Box"),
            try_retype_box,
            unsize_doc!("Box"),
            unsize_box,
            from_slice_doc!("Box"),
            Result,
            try_from_boxed_slice,
        ];
        ptr_fns![
            [],
            Rc,
            from_inner_doc!("Rc"),
            from_inner_rc,
            retype_doc!("Rc"),
            retype_rc,
            try_retype_doc!("Rc"),
            try_retype_rc,
            unsize_doc!("Rc"),
            unsize_rc,
            from_slice_doc!("Rc"),
            Result,
            try_from_rc_slice,
        ];
        ptr_fns![
            [],
            Arc,
            from_inner_doc!("Arc"),
            from_inner_arc,
            retype_doc!("Arc"),
            retype_arc,
            try_retype_doc!("Arc"),
            try_retype_arc,
            unsize_doc!("Arc"),
            unsize_arc,
            from_slice_doc!("Arc"),
            Result,
            try_from_arc_slice,
        ];
    }
}
