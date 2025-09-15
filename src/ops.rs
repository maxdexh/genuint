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

// TODO: annotate lazy instead with name of opaque
macro_rules! opaque {
    (
        $(())?
        $(#[$attr:meta])*
        pub type $name:ident<$($param:ident $(= $def:ty)?),* $(,)?> = $val:ty;
    ) => {
        #[$crate::utils::apply($crate::ops::lazy)]
        $(#[$attr])*
        pub type $name<$($param $(= $def)?),*> = $crate::ops::__make_opaque!($($param)*, $val);
    };
    (
        ($with_lazy:ident)
        $(#[$attr:meta])*
        pub type $name:ident<$($param:ident $(= $def:ty)?),* $(,)?> = $val:ty;
    ) => {
        #[$crate::utils::apply($crate::ops::lazy)]
        pub type $with_lazy<$($param),*> = $val;

        $crate::ops::opaque! {
            $(#[$attr])*
            pub type $name<$($param $(= $def)?),*> = $with_lazy<$($param),*>;
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

mod satdec;

mod testing;

mod bitmath;
pub use bitmath::{BitAnd, BitOr, BitXor, CountOnes};

mod log;
pub use log::{BaseLen, ILog};

mod add;
pub use add::Add;

mod mul;
pub use mul::Mul;

mod cmp;
pub use cmp::{Eq, Ge, Gt, Le, Lt, Max, Min, Ne};

mod sub;
pub use sub::{AbsDiff, SatSub};

mod divrem;
pub use divrem::{Div, Rem};

mod shift;
pub use shift::{Shl, Shr};

mod pow;
pub use pow::Pow;

#[cfg(test)]
#[doc(hidden)]
pub const fn _type_eq_chk<A, B>()
where
    core::iter::Once<A>: Iterator<Item = B>,
{
}
