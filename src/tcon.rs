use core::mem::{ManuallyDrop, MaybeUninit};

/// An arbitrary type map from [`Sized`] to [`Sized`].
///
/// See [`tern::raw::pull_tcon`](crate::tern::raw::pull_tcon) and [`tern::raw::push_tcon`](crate::tern::raw::push_tcon) for why this is useful.
pub trait TCon {
    /// Applies the type constructor to `T`.
    type Apply<T>;
}
/// The identity [`TCon`]. Maps `T` to `T`.
pub struct TConIdent(());
impl TCon for TConIdent {
    type Apply<T> = T;
}
/// Maps `T` to [`ManuallyDrop<T>`]
pub struct TConManuallyDrop(());
impl TCon for TConManuallyDrop {
    type Apply<T> = ManuallyDrop<T>;
}
/// Maps `T` to [`MaybeUninit<T>`]
pub struct TConMaybeUninit(());
impl TCon for TConMaybeUninit {
    type Apply<T> = MaybeUninit<T>;
}
/// Maps `T` to [`Option<T>`]
pub struct TConOption(());
impl TCon for TConOption {
    type Apply<T> = Option<T>;
}
/// Maps `T` to `U` for an unrelated `U`
pub struct TConTrivial<U>(U);
impl<U> TCon for TConTrivial<U> {
    type Apply<T> = U;
}

/// Like [`TCon`], but the generic associated type is bounded by a lifetime parameter.
/// This allows expressing type constructors such as [`T -> &'a T`](TConLtRef)
pub trait TConLt<'a> {
    /// Applies the type constructor to `T`.
    type Apply<T: 'a>;
}
/// Maps `T` to `&T`
pub struct TConLtRef(());
impl<'a> TConLt<'a> for TConLtRef {
    type Apply<T: 'a> = &'a T;
}
/// Maps `T` to `&mut T`
pub struct TConLtMut(());
impl<'a> TConLt<'a> for TConLtMut {
    type Apply<T: 'a> = &'a mut T;
}
