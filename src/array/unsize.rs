//! Functions that convert [`Array`] types into slices.
//!
//! This is equivalent to implicit unsize coercion for builtin arrays.
//!
//! # Panics
//! Every function in this module panics if the length of the input array exceeds `usize::MAX`
//! (only possible for ZSTs)

use crate::array::{Array, func_mods::*};

macro_rules! decl_unsize {
    ($ty:ident { $($mods:tt)* } $name:ident) => {
        #[doc = core::concat!("Performs the conversion for ", $ty!(docname), ".")]
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
