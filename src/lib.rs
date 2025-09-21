//! TODO: Docs go here
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(any(test, doc, feature = "std")), no_std)]
#![cfg_attr(test, recursion_limit = "1024")]
#![warn(
    clippy::nursery,
    clippy::missing_panics_doc,
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    missing_docs,
    clippy::undocumented_unsafe_blocks
)]
#![allow(clippy::redundant_pub_crate, clippy::use_self)]
// TODO: doctests

#[cfg(feature = "alloc")]
extern crate alloc;

// Hidden implementation details
mod internals;
mod uimpl;

// Macro implementation details
#[doc(hidden)]
pub mod __mac;

// internal utils
mod const_fmt;
mod maxint;
mod utils;

// Public API
pub mod array;
pub mod condty;
pub mod consts;
pub mod ops;
pub mod small;
pub mod uint;

/// A type-level non-negative integer
///
/// See the [crate level documentation](crate).
///
/// It is guaranteed (including to unsafe code) that there is a one-to-one correspondence between
/// the non-negative integers and the set of types that implement this trait.
pub trait Uint: ToUint<ToUint = Self> + 'static + internals::UintSealed {}

/// A type that can be turned into a [`Uint`]
///
/// This is not only a conversion trait, but forms an important part in how most operations are
/// implemented. See the [`ops`] module.
pub trait ToUint {
    /// Performs the conversion to [`Uint`].
    type ToUint: Uint;
}

pub use uint::From as UintFrom;
