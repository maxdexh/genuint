//! Conversion functions for [`Array`] types.
//!
//! # `try_from_{, mut, ...}_slice`
//! Functions that try to convert slices into [`Array`] types.
//!
//! This is equivalent to the [`TryFrom`] impls for builtin arrays from borrow and smart pointer slices.
//!
//! # `retype_{ref, mut, ...}`
//! Functions that cast the [`Array`] type behind the input reference/smart pointer
//! to another [`Array`] type with the same item type and length.
//!
//! # `try_retype_{ref, mut, ...}`
//! Functions that try to case the [`Array`] type behind the input reference/smart pointer
//! to another [`Array`] type with the same item type.
//!
//! #### Errors
//! The conversion succeeds if the length is the same. Otherwise, the input is returned in a
//! [`CondResult`], even for references, since [`CondResult`] does not need extra space for a
//! discriminant, so there is no niche optimization benifit from using an option (or ZST error
//! type).
//!
//! # `unsize_{ref, mut, ...}`
//! Functions that convert [`Array`] types into slices.
//!
//! This is equivalent to implicit unsize coercion for builtin arrays.
//!
//! #### Panics
//! If the length of the input array exceeds `usize::MAX` (only possible for ZSTs)

use crate::{array::Array, condty::CondResult, ops, uint};

macro_rules! decl_ptr {
    (
        $name:ident
        $($input:tt)*
    ) => {
        decl_ptr! {
            @[$]
            $name
            $($input)*
        }
    };
    (
        @[$dollar:tt]
        $name:ident,
        typ! { $tparam:ident => $($typ:tt)* },
        doc = $docname:expr,
        into_raw = |$into_raw_par:pat_param| $into_raw:expr,
        from_raw = |$from_raw_par:pat_param| $from_raw:expr,
        modifiers! $modifiers:tt,
        fns {
            $($fn:ident: $impl:tt),* $(,)?
        }
        $(,)?
    ) => {
        macro_rules! $name {
            (typ, $dollar$tparam:ty) => { $($typ)* };
            (docname) => { $docname };
            (into_raw, $ptr:expr) => {{ let $into_raw_par = $ptr; $into_raw }};
            (from_raw, $ptr:expr) => {{ let $from_raw_par = $ptr; $from_raw }};
            $((fn $fn, $cb:ident) => { $cb! { $name $modifiers $impl } };)*
        }
        pub(crate) use $name;
    };
}
decl_ptr![
    Ref,
    typ! { inner => &$inner },
    doc = "`&A`",
    into_raw = |r| core::ptr::from_ref(r).cast_mut(),
    from_raw = |r| &*r,
    modifiers! { pub const },
    fns {
        retype: retype_ref,
        try_retype: try_retype_ref,
        unsize: unsize_ref,
        try_from_slice: (try_from_slice, Option),
    },
];
decl_ptr![
    RefMut,
    typ! { inner => &mut $inner },
    doc = "`&mut A`",
    into_raw = |r| core::ptr::from_mut(r),
    from_raw = |r| &mut *r,
    modifiers! {
        pub const
    },
    fns {
        retype: retype_mut,
        try_retype: try_retype_mut,
        unsize: unsize_mut,
        try_from_slice: (try_from_mut_slice, Option),
    },
];
decl_ptr![
    Box,
    typ! { inner => alloc::boxed::Box<$inner> },
    doc = "[`Box<A>`](std::boxed::Box)",
    into_raw = |r| alloc::boxed::Box::into_raw(r),
    from_raw = |r| alloc::boxed::Box::from_raw(r),
    modifiers! {
        #[cfg(feature = "alloc")]
        #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
        pub
    },
    fns {
        retype: retype_box,
        try_retype: try_retype_box,
        unsize: unsize_box,
        try_from_slice: (try_from_boxed_slice, Result),
    },
];
decl_ptr![
    Rc,
    typ! { inner => alloc::rc::Rc<$inner> },
    doc = "[`Rc<A>`](std::rc::Rc)",
    into_raw = |r| alloc::rc::Rc::into_raw(r).cast_mut(),
    from_raw = |r| alloc::rc::Rc::from_raw(r),
    modifiers! {
        #[cfg(feature = "alloc")]
        #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
        pub
    },
    fns {
        retype: retype_rc,
        try_retype: try_retype_rc,
        unsize: unsize_rc,
        try_from_slice: (try_from_rc_slice, Result),
    },
];
decl_ptr![
    Arc,
    typ! { inner => alloc::sync::Arc<$inner> },
    doc = "[`Arc<A>`](std::sync::Arc)",
    into_raw = |r| alloc::sync::Arc::into_raw(r).cast_mut(),
    from_raw = |r| alloc::sync::Arc::from_raw(r),
    modifiers! {
        #[cfg(feature = "alloc")]
        #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
        pub
    },
    fns {
        retype: retype_arc,
        try_retype: try_retype_arc,
        unsize: unsize_arc,
        try_from_slice: (try_from_arc_slice, Result),
    },
];

macro_rules! for_each_ptr {
    ($fn:ident, $cb:ident) => {
        Ref! { fn $fn, $cb }
        RefMut! { fn $fn, $cb }
        Box! { fn $fn, $cb }
        Rc! { fn $fn, $cb }
        Arc! { fn $fn, $cb }
    };
}
pub(crate) use for_each_ptr;

macro_rules! decl_from_slice {
    ($ty:ident { $($mods:tt)* } ($name:ident, Result)) => {
        #[doc = core::concat!("Converts from a slice for ", $ty!(docname), ".")]
        ///
        /// See the [module level documentation](self).
        ///
        /// # Errors
        /// If the lengths do not match, the input is returned.
        $($mods)* fn $name<A: Array>(slice: $ty!(typ, [A::Item])) -> Result<$ty!(typ, A), $ty!(typ, [A::Item])> {
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
    ($ty:ident { $($mods:tt)* } ($name:ident, Option)) => {
        #[doc = core::concat!("Converts from a slice for ", $ty!(docname), ".")]
        ///
        /// See the [module level documentation](self).
        ///
        /// # Errors
        /// If the lengths do not match, `None` is returned.
        $($mods)* fn $name<A: Array>(slice: $ty!(typ, [A::Item])) -> Option<$ty!(typ, A)> {
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
for_each_ptr!(try_from_slice, decl_from_slice);

macro_rules! decl_retype {
    ($ty:ident { $($mods:tt)* } $name:ident) => {
        #[doc = core::concat!("Retypes [`Array`]s for ", $ty!(docname), ".")]
        ///
        /// See the [module level documentation](self).
        $($mods)* fn $name<Src, Dst>(src: $ty!(typ, Src)) -> $ty!(typ, Dst)
        where
            Src: Array,
            Dst: Array<Item = Src::Item, Length = Src::Length>,
        {
            // SAFETY: N == Dst::Length, `Array` invariant
            unsafe { $ty!(from_raw, $ty!(into_raw, src).cast()) }
        }
    };
}
for_each_ptr!(retype, decl_retype);

macro_rules! decl_try_retype {
    ($ty:ident { $($mods:tt)* } $name:ident) => {
        #[doc = core::concat!("Attempts to retype [`Array`]s for ", $ty!(docname), ".")]
        ///
        /// See the [module level documentation](self).
        ///
        /// # Errors
        /// If the lengths do not match, the input is returned.
        $($mods)* fn $name<Src, Dst>(src: $ty!(typ, Src)) -> CondResult<ops::Eq<Src::Length, Dst::Length>, $ty!(typ, Dst), $ty!(typ, Src)>
        where
            Src: Array,
            Dst: Array<Item = Src::Item>,
        {
            match uint::to_bool::<ops::Eq<Src::Length, Dst::Length>>() {
                true => CondResult::make_ok(
                    // SAFETY: Src::Length == Dst::Length
                    unsafe { $ty!(from_raw, $ty!(into_raw, src).cast()) },
                ),
                false => CondResult::make_err(src),
            }
        }
    };
}
for_each_ptr!(try_retype, decl_try_retype);

macro_rules! decl_unsize {
    ($ty:ident { $($mods:tt)* } $name:ident) => {
        #[doc = core::concat!("Performs unsizing for ", $ty!(docname), ".")]
        ///
        /// See the [module level documentation](self).
        #[track_caller]
        $($mods)* fn $name<A: Array>(arr: $ty!(typ, A)) -> $ty!(typ, [A::Item]) {
            // SAFETY: `Array` to slice cast
            unsafe { $ty!(from_raw, crate::array::helper::unsize_raw_mut($ty!(into_raw, arr))) }
        }
    };
}
for_each_ptr!(unsize, decl_unsize);
