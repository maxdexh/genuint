//! Module with operations for [`Uint`]s.
//!
//! # Laziness
//! Operations implemented as a struct implementing [`ToUint`] are called lazy. They are lazy
//! in the sense that they will only be evaluated once the compiler "evaluates" the associated
//! type projection [`<Op<...> as ToUint>::ToUint`](ToUint::ToUint), generally through use of
//! [`uint::From`].
//!
//! All operations in this module are lazy. In order to get a [`Uint`] from them, e.g. for use
//! with [arrays](crate::array), one has to use [`uint::From`] to evaluate them.
//!
//! Lazy operations are in contrast to type aliases, e.g. `type Inc<N> = uint::From<Add<N, _1>>`,
//! which directly expand at the usage site, though they can still be lazy if they expand to
//! a lazy operation and don't convert anything to a [`Uint`].
//!
//! # Primitive operations
//! Operations that are implemented through a dedicated associated type are called primitive.
//!
//! Currently, there are the following nontrivial primitive operations. Their associated types
//! are not public API.
//! - [`PopBit<N>`] removes the last bit of [`N::ToUint`](ToUint).
//!     - Equivalent expr: `N.to_uint() / 2`
//! - [`LastBit<N>`] gets the last bit of [`N::ToUint`](ToUint).
//!     - Equivalent expr: `N.to_uint() % 2`
//! - [`PushBit<N, B>`] pushes [`B`](ToUint) as a bit to the end of [`N`](ToUint)
//!     - Equivalent expr: `2 * N.to_uint() + (B.to_uint() != 0) as _`
//! - [`If<C, T, F>`] evaluates to [`T::ToUint`](ToUint) if `C` is nonzero, otherwise
//!   to [`F::ToUint`](ToUint). Only the necessary [`ToUint::ToUint`] projection is evaluated.
//!     - Equivalent expr: `if C != 0 { T.to_uint() } else { F.to_uint() }`
//!
//! These primitives, together with [`ToUint`] implementations based on them (and [`uint::From`]),
//! are sufficient for a [Turing-complete](https://en.wikipedia.org/wiki/Turing_completeness)
//! system, and all other operations in this module are just implemented on top of them. The way to do this is
//! described in the following sections.
//!
//! # Recursion
//! The way to implement an operation where the output requires looking at the entire number is to
//! do it recursively. However, regular type aliases do not support recursion, see error E0391
//! "cycle detected when expanding type alias".
//!
//! Instead, one has to go through [`ToUint`] to make the operation "lazy", as in its value is only
//! computed when it is projected to [`ToUint::ToUint`]. For example, consider the following
//! implementation of [`BitAnd`]:
//! ```
//! use genuint::{ToUint, small::*, uops::*, uint};
//! // MyBitAnd is a struct implementing ToUint, i.e. a lazy operation
//! pub struct MyBitAnd<L, R>(L, R);
//! impl<L: ToUint, R: ToUint> ToUint for MyBitAnd<L, R> {
//!     type ToUint = uint::From<If<
//!         L,
//!         // take the bitand of the previous bits and append the and of the last bit
//!         PushBit<
//!             MyBitAnd<PopBit<L>, PopBit<R>>,
//!             If<LastBit<L>, LastBit<R>, _0>, // boolean AND
//!         >,
//!         _0, // 0 & R = 0
//!     >>;
//! }
//! fn check_input<L: ToUint, R: ToUint>() {
//!     assert_eq!(
//!         uint::to_u128::<MyBitAnd<L, R>>().unwrap(),
//!         uint::to_u128::<L>().unwrap() & uint::to_u128::<R>().unwrap(),
//!     )
//! }
//! check_input::<_3, _5>();
//! check_input::<_59, _122>();
//! check_input::<uint::lit!(0b10101000110111111), uint::lit!(0b11110111011111)>()
//! ```
//! Because `MyBitAnd` is [`ToUint`] here and [`If`] works by only evaluating
//! [`ToUint::ToUint`] for the branch that is needed for the output, this will
//! safely exit when `L` becomes 0.
//!
//! #### Evaluating recursive arguments
//! Because [`PopBit`] is itself lazy, the above definition of `MyBitAnd` will
//! result in the arguments to `MyBitAnd` accumulating `PopBit<PopBit<...>>`
//! for every recusive step. This can be fixed by applying [`uint::From`] to
//! the recursive arguments; e.g. in the example above, it is preferrable to
//! use `MyBitAnd<uint::From<PopBit<L>>, uint::From<PopBit<R>>>`.
//!
//! Evaluating recursive arguments is almost always  beneficial for compile times.
//! Note that if the recursive arguments are nontrivial to calculate or might themselves
//! result in infinite loops when normalized, they can be refactored out into a seperate
//! lazy operation. As an example of this, [`ILog`] uses division in its recursive argument
//! and therefore
//!
//! # Opaqueness
//! Note: This section is only relevant if the operation in question is public API or when
//! experiencing weird recursion limit errors from normalization of large inputs.
//!
//! The reason this is useful is that because types are heavily normalized
//! by the compiler, it is easy to accidentally leak implementation details about
//! them in a public API, which would make them impossible to normalize in the future,
//! as someone could rely on them behaving a certain way in generic contexts.
//! An example of this would be `LastBit<PushBit<N, B>> = B` where the arguments are generic.
//!
//! Furthermore, when using things like `uint::From<Min<UsizeMax, N>>` where `N` is generic,
//! the compiler might try to normalize the entire recursive `Min` operation, which may cause
//! spurious "overflow while ..." errors.
//!
//! These things can be guarded against using [`Opaque`]. `Opaque<P, Out>` always evaluates
//! to `Out`, but only after projecting through an internal associated type of `P`, like
//!`<P as Uint>::_Opaque<Out>`.
//!
//! This means that the compiler can only determine the value of [`Opaque<P, Out>`]
//! after it has determined the value of `P`, and it cannot do any normalization
//! specific to the implementation of `Out::ToUint` before that.
//!
//! The way to use this when implementing a public operation `Op<A, B>` is as follows:
//! - The actual implementation is moved to a seperate lazy operation `OpImpl<A, B>`. Recursive
//!   evaluations use `OpImpl` rather than `Op`.
//! - `Op` should be a lazy operation that evaluates to `uint::From<Opaque<A, Opaque<B, OpImpl<A, B>>>>`
//!
//! # Complete example implementation of [`BitAnd`]
//! ```
//! use genuint::{ToUint, small::*, uops::*, uint};
//! pub struct _MyBitAnd<L, R>(L, R); // hide this in a private module
//! impl<L: ToUint, R: ToUint> ToUint for _MyBitAnd<L, R> {
//!     type ToUint = uint::From<If<
//!         L,
//!         // take the bitand of the previous bits and append the and of the last bit
//!         PushBit<
//!             _MyBitAnd<
//!                 uint::From<PopBit<L>>,
//!                 uint::From<PopBit<R>>,
//!             >,
//!             If<LastBit<L>, LastBit<R>, _0>, // boolean AND
//!         >,
//!         _0, // 0 & R = 0
//!     >>;
//! }
//! pub type MyBitAnd<L, R> = Opaque<L, Opaque<R, _MyBitAnd<L, R>>>;
//! fn check_input<L: ToUint, R: ToUint>() {
//!     assert_eq!( // works fully generically!
//!         uint::to_u128::<MyBitAnd<L, R>>().unwrap(),
//!         uint::to_u128::<L>().unwrap() & uint::to_u128::<R>().unwrap(),
//!     )
//! }
//! check_input::<_3, _5>();
//! check_input::<_59, _122>();
//! check_input::<uint::lit!(0b10101000110111111), uint::lit!(0b11110111011111)>()
//! ```

#[expect(unused_imports)] // for docs
use crate::{ToUint, Uint};
use crate::{internals::InternalOp, small::*, uint, utils::apply};

macro_rules! lazy_impl {
    (
        $(())?
        type $Name:ident<$($P:ident $(= $_:ty)?),* $(,)?> = $Val:ty;
    ) => {
        impl<$($P: crate::ToUint),*> crate::ToUint for $Name<$($P),*> {
            #[doc(hidden)]
            type ToUint = crate::uint::From<$Val>;
        }
    };
    (
        $(())?
        $(#[$attr:meta])*
        type $Name:ident<$($P:ident: $Bound:path $(= $_:ty)?),* $(,)?>: $OutBound:path = $Val:ty;
    ) => {
        $(#[$attr])*
        impl<$($P: $Bound),*> $OutBound for $Name<$($P),*> {
            #[doc(hidden)]
            type ToUint = crate::uint::From<$Val>;
        }
    };
}
pub(crate) use lazy_impl;

/// Input format:
/// ```compile_fail
/// #[apply(lazy)]
/// pub type A<P1, P2, ...> = $Val;
/// ```
///
/// Output format:
/// ```compile_fail
/// #[apply(lazy)]
/// pub struct A<P1, P2, ...>(P1, P2, ...);
/// impl<P1: ToUint, P2: ToUint, ...> ToUint for A<P1, P2, ...> {
///     type ToUint = uint::From<$Val>;
/// }
/// ```
macro_rules! lazy {
    (
        $(())?
        $(#[$attr:meta])*
        pub type $Name:ident<$($P:ident $(= $Def:ty)?),* $(,)?> = $Val:ty;
    ) => {
        $(#[$attr])*
        pub struct $Name<$($P $(= $Def)?),*>($($P),*);
        crate::uops::lazy_impl! {
            type $Name<$($P),*> = $Val;
        }
    };
}
pub(crate) use lazy;

/// Variadic [`Opaque`]
macro_rules! VarOpaque {
    ($($LazyBase:ident)::+<$($P:ident),* $(,)?>) => {
        crate::uops::VarOpaque!(
            @$($P)*,
            $($LazyBase)::+<$($P),*>
        )
    };
    (@$P:ident $($Ps:ident)*, $Out:ty) => {
        crate::uops::Opaque<$P, crate::uops::VarOpaque!(@$($Ps)*, $Out)>
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
        ()
        $(#[$attr:meta])*
        pub type $Name:ident<$($P:ident $(: $Bound:path)? $(= $Def:ty)?),* $(,)?> $(: $OutBound:path)? = $LazyBase:ident;
    ) => {
        #[cfg(test)]
        #[allow(unused)] // Ensure that LazyBase is spanned for LSP
        const _: () = { use $LazyBase; };
        crate::uops::lazy! {
            $(#[$attr])*
            pub type $Name<$($P $(: $Bound)? $(= $Def)?),*> $(: $OutBound)? = crate::uops::VarOpaque!($LazyBase<$($P),*>);
        }
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
        crate::uops::testing::test_op! { $test_name: $($P)*, $TypeName<$($P),*>, $($args)* }

        $(#[$attr])*
        $v $kw $TypeName<$($P $(= $Def)?),*> $($rest)*
    };
}
pub(crate) use test_op;

macro_rules! base_case {
    (
        (0 == $CheckZero:ty => $IfZero:ty)
        $(#[$attr:meta])*
        $v:vis type $Name:ident<$($P:ident $(: $Bound:path)? $(= $Def:ty)?),* $(,)?> $(: $OutBound:path)? = $Val:ty;
    ) => {
        $(#[$attr])*
        $v type $Name<$($P $(: $Bound)? $(= $Def)?),*> $(: $OutBound)? = crate::uops::If<
            $CheckZero,
            $Val,
            $IfZero,
        >;
    };
}

mod primitives;
pub use primitives::{If, LastBit, Opaque, PopBit, PushBit};

mod helper;
pub(crate) use helper::*;

mod trivial;
pub use trivial::{IsFalsy, IsTruthy};

mod testing;

mod bitmath;
pub use bitmath::{BitAnd, BitOr, BitXor, CountOnes};

mod log;
pub use log::{BaseLen, ILog};

mod add;
pub use add::Add;
pub(crate) use add::*;

mod mul;
pub use mul::Mul;
pub(crate) use mul::*;

mod cmp;
pub(crate) use cmp::*;
pub use cmp::{Eq, Ge, Gt, Le, Lt, Max, Min, Ne};

mod sub;
pub(crate) use sub::*;
pub use sub::{AbsDiff, SatSub};

mod divrem;
pub(crate) use divrem::*;
pub use divrem::{Div, Rem};

mod shift;
pub use shift::{Shl, Shr};

mod pow;
pub use pow::Pow;
