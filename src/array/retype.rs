//! Functions that convert between equivalent [`Array`] types
//!
//! Each function in this module casts the [`Array`] type behind the input reference/smart pointer
//! to another [`Array`] type with the same item type and length.

use crate::array::{Array, func_mods::*};

macro_rules! decl_retype {
    ($ty:ident { $($mods:tt)* } $name:ident) => {
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
