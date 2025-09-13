//! TODO: Docs go here

// TODO: Doc features
// TODO: doctests
#![cfg_attr(test, recursion_limit = "512")]
#![cfg_attr(not(any(test, doc, feature = "std")), no_std)]
#![warn(
    clippy::nursery,
    clippy::missing_panics_doc,
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    missing_docs,
    // enforce "SAFETY" comments
    clippy::undocumented_unsafe_blocks,
)]
#![allow(
    // use of pub(crate) over pub is for clarity
    clippy::redundant_pub_crate,
    // `Ok({ ... })`
    clippy::unit_arg,
)]

#[cfg(feature = "alloc")]
extern crate alloc;

/// Turns an integer literal into a [`Uint`].
///
/// If you have a small constant value that is not a literal, use [`uint::FromU128`].
///
/// # Examples
/// ```
/// #![recursion_limit = "1024"] // `lit!` doesn't recurse, the type is just long
///
/// use generic_uint::{lit, uint};
/// assert_eq!(uint::to_u128::<lit!(1)>(), Some(1));
/// assert_eq!(
///     uint::to_u128::<lit!(100000000000000000000000000000)>(),
///     Some(100000000000000000000000000000),
/// )
/// ```
#[macro_export]
macro_rules! lit {
    ($l:literal) => {
        $crate::__mac::__lit!(($l) $crate)
    };
}

mod internals;
mod utils;

/// A type-level non-negative integer.
///
/// See the [crate level documentation](crate).
///
/// It is guaranteed (including to unsafe code) that there is a one-to-one correspondence between
/// the non-negative integers and the set of types that implement this trait.
pub trait Uint: ToUint<ToUint = Self> + 'static + internals::UintSealed {}

/// A type that can be turned into a [`Uint`].
///
/// TODO: Something something lazy uints refer to ops docs
pub trait ToUint {
    #[allow(missing_docs)]
    type ToUint: Uint;
}

pub mod array;
pub mod consts;
pub mod ops;
pub mod tern;
pub mod tfun;
pub mod uint;

#[doc(hidden)]
pub mod __mac;

mod const_fmt;
mod maxint;
