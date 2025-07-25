#![cfg_attr(not(test), no_std)]
#![warn(clippy::nursery)]
#![allow(clippy::unit_arg)]

mod internals;
mod utils;

pub trait Uint: ToUint<ToUint = Self> + 'static + internals::UintSealed {}

pub trait ToUint {
    type ToUint: Uint;
}

pub mod array;
pub mod consts;
pub mod ops;
pub mod uint;

#[doc(hidden)]
pub mod __mac;
