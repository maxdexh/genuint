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

/// Turns an integer literal of arbitrary length into a [`Uint`].
///
/// If you have a small constant value that is not a literal, use [`uint::FromU128`].
///
/// # Examples
/// ```
/// use generic_uint::{lit, uint};
/// assert_eq!(uint::to_u128::<lit!(1)>(), Some(1));
/// ```
#[macro_export]
macro_rules! lit {
    ($l:literal) => {
        $crate::__mac::__lit!(($l) $crate)
    };
}

mod internals;
mod utils;

pub trait Uint: ToUint<ToUint = Self> + 'static + internals::UintSealed {}

pub trait ToUint {
    type ToUint: Uint;
}

pub trait UintEq<To: ToUint>: ToUint<ToUint = To::ToUint> {}
impl<To: ToUint, N: ToUint<ToUint = To::ToUint>> UintEq<To> for N {}

pub mod array;
pub mod capnum;
pub mod consts;
pub mod ops;
pub mod uint;

#[doc(hidden)]
pub mod __mac;

mod const_fmt;
mod maxint;
