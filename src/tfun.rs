//! Type functions
//!
//! A type function is a type that implements some trait that looks like this:
//! ```
//! # trait SomeBoundIn {}
//! # trait SomeBoundOut {}
//! trait TCon {
//!     type Apply<T: SomeBoundIn>: SomeBoundOut;
//! }
//! ```
//!
//! Currently only used for [`tern::raw::pull_tcon`](crate::tern::raw::pull_tcon) and [`tern::raw::push_tcon`](crate::tern::raw::push_tcon).

use core::mem::{ManuallyDrop, MaybeUninit};

use crate::ToUint;

/// An arbitrary type map from [`Sized`] to [`Sized`].
///
/// See  for why this is useful.
pub trait TFun {
    /// Applies the type constructor to `T`.
    type Apply<T>;
}
/// The identity [`TCon`]. Maps `T` to `T`.
pub struct TFunIdent(());
impl TFun for TFunIdent {
    type Apply<T> = T;
}
/// Maps `T` to [`ManuallyDrop<T>`]
pub struct TFunManuallyDrop(());
impl TFun for TFunManuallyDrop {
    type Apply<T> = ManuallyDrop<T>;
}
/// Maps `T` to [`MaybeUninit<T>`]
pub struct TFunMaybeUninit(());
impl TFun for TFunMaybeUninit {
    type Apply<T> = MaybeUninit<T>;
}
/// Maps `T` to [`Option<T>`]
pub struct TConOption(());
impl TFun for TConOption {
    type Apply<T> = Option<T>;
}
/// Maps `T` to `U` for an unrelated `U`
pub struct TConTrivial<U>(U);
impl<U> TFun for TConTrivial<U> {
    type Apply<T> = U;
}

/// Like [`TFun`], but the generic associated type is bounded by a lifetime parameter.
/// This allows expressing type constructors such as [`T -> &'a T`](TConLtRef)
pub trait TFunLt<'a> {
    /// Applies the type constructor to `T`.
    type Apply<T: 'a>: 'a;
}
/// Maps `T` to `&T`
pub struct TConLtRef(());
impl<'a> TFunLt<'a> for TConLtRef {
    type Apply<T: 'a> = &'a T;
}
/// Maps `T` to `&mut T`
pub struct TConLtMut(());
impl<'a> TFunLt<'a> for TConLtMut {
    type Apply<T: 'a> = &'a mut T;
}

/// A type function from [`ToUint`] to itself.
pub trait UintFn {
    /// Applies the type constructor to `N`.
    type Apply<N: ToUint>: ToUint;
}
