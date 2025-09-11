//! Functions that try to convert between equivalent [`Array`] types
//!
//! Each function in this module tries to cast the [`Array`] type behind the input reference/smart pointer
//! to another [`Array`] type with the same item type.
//!
//! The conversion succeeds if the length is the same.

use crate::{
    array::{Array, func_mods::*},
    ops,
    tern::TernRes,
    uint,
};

macro_rules! decl_try_retype {
    ($ty:ident { $($mods:tt)* } $name:ident) => {
        $($mods)* fn $name<Src, Dst>(src: $ty!(typ, Src)) -> TernRes<ops::Eq<Src::Length, Dst::Length>, $ty!(typ, Dst), $ty!(typ, Src)>
        where
            Src: Array,
            Dst: Array<Item = Src::Item>,
        {
            match uint::to_bool::<ops::Eq<Src::Length, Dst::Length>>() {
                true => TernRes::make_ok(
                    // SAFETY: Src::Length == Dst::Length
                    unsafe { $ty!(from_raw, $ty!(into_raw, src).cast()) },
                ),
                false => TernRes::make_err(src),
            }
        }
    };
}
for_each_ptr!(try_retype, decl_try_retype);
