//! Module defining the fundamental and other useful operations for [`Uint`]s.
//!
//! TODO: Something about general techniques, lazy uints, etc.

// TODO:
// - Optimize test speed; compare times against before lazification; analyze how types expand
// - Remove unnecessary intermediate ops that were only used to lazify

#[allow(unused)] // for docs
use crate::Uint;
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

/// Makes `Out` opaque with respect to a paramter `P`.
///
/// See the [module level documentation](self) for more details.
///
/// Note the order of the arguments! It is chosen to be this way so that when nesting multiple
/// `Opaque`s (for multiple parameters), the output type is always at the end, which is a useful
/// property to have when looking at the type through error messages or an lsp.
// TODO: Try a type function based approach for opaqueness, i.e. have a
// GAT ApplyOpaquely<F: TFunUint> = F::Apply<Self>
pub type Opaque<P, Out> = _Opaque<P, Out>;

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

/// More efficient implementation of [`Div<N, _2>`].
///
/// This is currently a primitive operation. When diretly passed to [`uint::From`], the
/// compiler will probably normalize it to a type projection on an internal associated
/// type of [`Uint`].
// H(N) := N / 2
pub type Half<N> = _Half<N>;

/// More efficient implementation of [`Rem<N, _2>`].
///
/// This is currently a primitive operation.
// P(N) := N % 2
pub type Parity<N> = _Parity<N>;

/// More efficient implementation of `Add<Mul<N, _2>, Tern<P, _1, _0>>`.
///
/// Equivalent to `2 * N + (P != 0) as _` in basic arithmetic or `(N << 1) | (P != 0) as _`
/// in bit manipulation. This operation is useful for building the output of operations
/// recursively bit-by-bit.
///
/// This is currently a primitive operation. When diretly passed to [`uint::From`], the
/// compiler will probably normalize it to a type projection on an internal associated
/// type of [`Uint`].
pub type AppendBit<N, P> = _AppendBit<N, P>;

/// If-else/Ternary operation.
///
/// If the first argument is nonzero, evaluates to the second argument, otherwise to the third.
///
/// This will only access ("evaluate") [`ToUint::ToUint`](crate::ToUint::ToUint) for the required
/// argument. This means that this operation can be used for the exit condition of a recursive
/// operation (see examples below).
///
/// This is currently a primitive operation. When diretly passed to [`uint::From`], the
/// compiler will probably normalize it to a type projection on an internal associated
/// type of [`Uint`].
///
/// # Examples
/// Exiting from a recursive operation
/// ```
/// use generic_uint::{ToUint, ops, uint, lit};
/// struct CountOnes<N, Acc = lit!(0)>(N, Acc);
/// impl<N: ToUint, Acc: ToUint> ToUint for CountOnes<N, Acc> {
///     type ToUint = uint::From<ops::Tern<
///         N,
///         CountOnes<
///             uint::From<ops::Half<N>>,
///             ops::Add<Acc, ops::Parity<N>>,
///         >,
///         Acc,
///     >>;
/// }
/// assert_eq!(
///     uint::to_u128::<CountOnes<lit!(0b1010101010101010101010)>>(),
///     Some(11),
/// );
/// ```
//
// Tern(C, T, F) := if C { T } else { F }
pub type Tern<C, T, F> = _Tern<C, T, F>;

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
