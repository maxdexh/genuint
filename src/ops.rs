//! Module defining the fundamental and other useful operations for [`Uint`]s.
//!
//! TODO: Something about general techniques, lazy uints, etc.

// TODO:
// - Optimize test speed; compare times against before lazification; analyze how types expand
// - Remove unnecessary intermediate ops that were only used to lazify

#[allow(unused)] // for docs
use crate::{ToUint, Uint};
use crate::{consts::*, internals::InternalOp, uint, utils::apply};

macro_rules! lazy {
    (
        $(())?
        $(#[$attr:meta])*
        pub type $name:ident<$($param:ident $(= $def:ty)?),* $(,)?> = $val:ty;
    ) => {
        $(#[$attr])*
        pub struct $name<$($param $(= $def)?),*>($($param),*);
        impl<$($param: $crate::ToUint),*> $crate::ToUint for $name<$($param),*> {
            type ToUint = $crate::uint::From<$val>;
        }
    };
    (
        ($mod:ident)
        $(#[$attr:meta])*
        $v:vis type $name:ident<$($param:ident $(= $def:ty)?),* $(,)?> = $val:ty;
    ) => {
        mod $mod {
            use super::*;
            $(#[$attr])*
            pub struct $name<$($param $(= $def)?),*>($($param),*);
            impl<$($param: $crate::ToUint),*> $crate::ToUint for $name<$($param),*> {
                type ToUint = $crate::uint::From<$val>;
            }
        }
        $v use $mod::*;
    };
}
pub(crate) use lazy;

macro_rules! __make_opaque {
    ($pop:ident $($param:ident)*, $out:ty) => {
        $crate::ops::Opaque<$pop, $crate::ops::__make_opaque!($($param)*, $out)>
    };
    (, $out:ty) => {
        $out
    };
}
pub(crate) use __make_opaque;

macro_rules! opaque {
    (
        $(())?
        $(#[$attr:meta])*
        pub type $name:ident<$($param:ident $(= $def:ty)?),* $(,)?> = $val:ty;
    ) => {
        $crate::ops::lazy! {
            $(#[$attr])*
            pub type $name<$($param $(= $def)?),*> = $crate::ops::__make_opaque!($($param)*, $val);
        }
    };
    (
        ($mod:ident::$non_opaque:ident)
        $(#[$attr:meta])*
        pub type $name:ident<$($param:ident),* $(,)?> = $val:ty;
    ) => {
        mod $mod {
            use super::*;
            #[$crate::utils::apply($crate::ops::lazy)]
            pub type $non_opaque<$($param),*> = $val;
        }
        pub(crate) use $mod::*;

        $crate::ops::opaque! {
            $(#[$attr])*
            pub type $name<$($param),*> = $non_opaque<$($param),*>;
        }
    };
}
pub(crate) use opaque;

macro_rules! test_op {
    (
        ($name:ident, $($args:tt)*)
        $(#[$($attr:tt)*])*
        $v:vis $kw:ident $tname:ident<$($param:ident $(= $def:ty)?),* $(,)?> $($rest:tt)*
    ) => {
        #[cfg(test)]
        crate::ops::testing::test_op! { $name: $($param)*, $tname<$($param),*>, $($args)* }

        $(#[$($attr)*])*
        $v $kw $tname<$($param $(= $def)?),*> $($rest)*
    };
}
pub(crate) use test_op;

mod primitives;
pub use primitives::{AppendBit, Half, If, Opaque, Parity};

mod helper;
pub(crate) use helper::*;

mod trivial;
pub use trivial::*;

mod testing;

mod bitmath;
pub use bitmath::*;

mod log;
pub use log::*;

mod add;
pub use add::*;

mod mul;
pub use mul::*;

mod cmp;
pub use cmp::*;

mod sub;
pub use sub::*;

mod divrem;
pub use divrem::*;

mod shift;
pub use shift::*;

mod pow;
pub use pow::*;
