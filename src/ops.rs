//! Module defining the fundamental and other useful operations for [`Uint`]s.
//!
//! TODO: Something about general techniques, lazy uints, etc.

// TODO:
// - Optimize test speed; compare times against before lazification; analyze how types expand

#[expect(unused)] // for docs
use crate::{ToUint, Uint};
use crate::{consts::*, internals::InternalOp, uint, utils::apply};

/// Helper macro to translate type alias syntax into lazy uint functions.
/// Visibility must be `pub`. This should only be used in non-public
/// modules or by other macros.
///
/// Input format:
/// ```compile_fail
/// #[apply(pub_lazy)]
/// pub type A<P1, P2, ...> = $Val;
/// ```
///
/// Output format:
/// ```compile_fail
/// #[apply(pub_lazy)]
/// pub struct A<P1, P2, ...>(P1, P2, ...);
/// impl<P1: ToUint, P2: ToUint, ...> ToUint for A<P1, P2, ...> {
///     type ToUint = uint::From<$Val>;
/// }
/// ```
macro_rules! pub_lazy {
    (
        $(())?
        $(#[$attr:meta])*
        pub type $Name:ident<$($P:ident $(= $Def:ty)?),* $(,)?> = $Val:ty;
    ) => {
        $(#[$attr])*
        pub struct $Name<$($P $(= $Def)?),*>($($P),*);
        impl<$($P: crate::ToUint),*> crate::ToUint for $Name<$($P),*> {
            type ToUint = crate::uint::From<$Val>;
        }
    };
}
pub(crate) use pub_lazy;

/// Like [`pub_lazy`], but creates the item in a private module and reexports it at the declared
/// visibility.
macro_rules! lazy {
    (
        ($mod:ident)
        $(#[$attr:meta])*
        $v:vis type $Name:ident<$($P:ident $(= $Def:ty)?),* $(,)?> = $Val:ty;
    ) => {
        mod $mod {
            use super::*;
            crate::ops::pub_lazy! {
                $(#[$attr])*
                pub type $Name<$($P $(= $Def)?),*> = $Val;
            }
        }
        $v use $mod::*;
    };
}
pub(crate) use lazy;

/// Variadic [`Opaque`]
macro_rules! VarOpaque {
    ($LazyBase:ident<$($P:ident),* $(,)?>) => {
        crate::ops::VarOpaque!(
            @$($P)*,
            $LazyBase<$($P),*>
        )
    };
    (@$P:ident $($Ps:ident)*, $Out:ty) => {
        crate::ops::Opaque<$P, crate::ops::VarOpaque!(@$($Ps)*, $Out)>
        // crate::ops::Opaque<crate::ops::VarOpaque!(@$($Ps)*, $P), $Out>
    };
    (@, $Out:ty) => {
        $Out
    };
}
pub(crate) use VarOpaque;

/// Like [`lazy`], but wraps the result in [`VarOpaque`].
/// For this, another [`lazy`] type `$LazyBase` is declared in the
/// module to holds the implementation to be wrapped by [`VarOpaque`].
///
/// Recursive implementations should use that name when recursing,
/// not the opaque wrapper.
///
/// Additionally, when an additional `pub(...)` visibility is passed
/// to the attribute, the non-opaque base type is exported at that
/// visibility, for internal use elsewhere.
macro_rules! opaque {
    (
        (pub $base_vis:tt $mod:ident::$LazyBase:ident)
        $($input:tt)*
    ) => {
        crate::ops::opaque! { ($mod::$LazyBase) $($input)* }

        #[allow(clippy::needless_pub_self)]
        pub $base_vis use $mod::$LazyBase;
    };
    (
        ($mod:ident::$LazyBase:ident)
        $(#[$attr:meta])*
        $v:vis type $Name:ident<$($P:ident $(= $Def:ty)?),* $(,)?> = $Val:ty;
    ) => {
        mod $mod {
            use super::*;
            crate::ops::pub_lazy! {
                pub type $LazyBase<$($P $(= $Def)?),*> = $Val;
            }
            crate::ops::pub_lazy! {
                $(#[$attr])*
                pub type $Name<$($P $(= $Def)?),*> = crate::ops::VarOpaque!($LazyBase<$($P),*>);
            }
        }
        $v use $mod::$Name;
    };
}
pub(crate) use opaque;

macro_rules! test_op {
    (
        ($test_name:ident, $($args:tt)*)
        $(#[$attr:meta])*
        $v:vis $kw:ident $TypeName:ident<$($P:ident $(= $Def:ty)?),* $(,)?> $($rest:tt)*
    ) => {
        #[cfg(test)]
        crate::ops::testing::test_op! { $test_name: $($P)*, $TypeName<$($P),*>, $($args)* }

        $(#[$attr])*
        $v $kw $TypeName<$($P $(= $Def)?),*> $($rest)*
    };
}
pub(crate) use test_op;

mod primitives;
pub use primitives::*;

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
