#![recursion_limit = "512"]
#![cfg_attr(not(any(test, doc, feature = "std")), no_std)]
#![warn(
    clippy::nursery,
    clippy::undocumented_unsafe_blocks, // enforce "SAFETY" comments
)]
#![allow(
    clippy::fallible_impl_from, // intrusive for infallible unwraps
    clippy::redundant_pub_crate, // changing pub(crate) to pub reduces clarity
    clippy::unit_arg, // justice for `=> Ok({ ... })`
)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod internals;
mod utils;

pub trait Uint: ToUint<ToUint = Self> + 'static + internals::UintSealed {}

pub trait ToUint {
    type ToUint: Uint;
}

pub mod array;
pub mod capnum;
pub mod consts;
pub mod ops;
pub mod uint;

#[doc(hidden)]
pub mod __mac;

mod const_fmt;
mod maxint;
