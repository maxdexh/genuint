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
// FIXME: Function names
decl_ptr![
    Ref,
    typ! { inner => &$inner },
    doc = "&A",
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
    doc = "&mut A",
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
