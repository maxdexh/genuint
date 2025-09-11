//! Functions that try to convert slices into [`Array`] types.
//!
//! This is equivalent to the [`TryFrom`] impls for builtin arrays from borrow and smart pointer slices.

use crate::array::{Array, func_mods::*};

macro_rules! decl_from_slice {
    ($ty:ident { $($mods:tt)* } ($name:ident, Result)) => {
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
