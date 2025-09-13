//! Type functions
//!
//! A type function is a type that implements some trait that looks like this:
//! ```
//! # trait SomeBoundIn {}
//! # trait SomeBoundOut {}
//! trait TFun {
//!     type Apply<T: SomeBoundIn>: SomeBoundOut;
//! }
//! ```
//!
//! Currently only used for [`tern::raw::pull_tcon`](crate::tern::raw::pull_tcon) and [`tern::raw::push_tcon`](crate::tern::raw::push_tcon).

use core::mem::{ManuallyDrop, MaybeUninit};

/// An arbitrary type map from [`Sized`] to [`Sized`].
///
/// See  for why this is useful.
pub trait TFun {
    /// Applies the type constructor to `T`.
    type Apply<T>;
}
/// The identity [`TFun`]. Maps `T` to `T`.
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
pub struct TFunOption(());
impl TFun for TFunOption {
    type Apply<T> = Option<T>;
}
/// Maps `T` to `U` for an unrelated `U`
pub struct TFunTrivial<U>(U);
impl<U> TFun for TFunTrivial<U> {
    type Apply<T> = U;
}

/// Like [`TFun`], but the generic associated type is bounded by a lifetime parameter.
/// This allows expressing type constructors such as [`T -> &'a T`](TFunLtRef)
pub trait TFunLt<'a> {
    /// Applies the type constructor to `T`.
    type Apply<T: 'a>: 'a;
}
/// Maps `T` to `&T`
pub struct TFunLtRef(());
impl<'a> TFunLt<'a> for TFunLtRef {
    type Apply<T: 'a> = &'a T;
}
/// Maps `T` to `&mut T`
pub struct TFunLtMut(());
impl<'a> TFunLt<'a> for TFunLtMut {
    type Apply<T: 'a> = &'a mut T;
}
