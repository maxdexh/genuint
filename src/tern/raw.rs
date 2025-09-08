use crate::{Uint, uint, utils};

/// A result-like structure that takes on the left `True` value if `Cond` is nonzero, or the right
/// `False` value otherwise.
///
/// The type depends only on `Cond`. If `Cond` is nonzero, then `TernRaw<Cond, T, F>` is literally
/// the same type as `T`. Otherwise it is the same type as `F`.
pub type TernRaw<Cond, True, False> = crate::internals::InternalOp!(Cond, ::TernAny<True, False>);

/// An arbitrary type map from [`Sized`] to [`Sized`].
pub trait TCon {
    type Apply<T>;
}
/// The identity [`TCon`].
pub struct TConIdent(());
impl TCon for TConIdent {
    type Apply<T> = T;
}
pub struct TConManuallyDrop(());
impl TCon for TConManuallyDrop {
    type Apply<T> = core::mem::ManuallyDrop<T>;
}
pub struct TConMaybeUninit(());
impl TCon for TConMaybeUninit {
    type Apply<T> = core::mem::MaybeUninit<T>;
}
pub struct TConOption(());
impl TCon for TConOption {
    type Apply<T> = Option<T>;
}
pub struct TConTrivial<U>(U);
impl<U> TCon for TConTrivial<U> {
    type Apply<T> = U;
}

/// Pulls an arbitrary [`TCon`] out of a [`TernRaw`].
///
/// This function is the inverse of [`push_tcon`].
///
/// For example, this function can turn `TernRaw<C, Option<T>, Option<F>>` into `Option<TernRaw<C, T, F>>`, or
/// `TernRaw<C, (T, T), (F, F)>` into `(TernRaw<C, T, F>, TernRaw<C, T, F>)`.
///
/// # Limitations
/// Since [`TCon::Apply`] requires being implemented for all `T: Sized`, type constructors with extra bounds
/// (for example `T -> &'a T` would require `T: 'a`) cannot be expressed by this.
pub const fn pull_tcon<C: Uint, T, F, Con: TCon>(
    tern: TernRaw<C, Con::Apply<T>, Con::Apply<F>>,
) -> Con::Apply<TernRaw<C, T, F>> {
    // SAFETY: Input and output are the same type
    unsafe { utils::same_type_transmute(tern) }
}

/// Pushes an arbitrary [`TCon`] out of a [`TernRaw`].
///
/// This function is the inverse of [`pull_tcon`].
///
/// For example, this function can turn `Option<TernRaw<C, T, F>>` into `TernRaw<C, Option<T>, Option<F>>`, or
/// `(TernRaw<C, T, F>, TernRaw<C, T, F>)` into `TernRaw<C, (T, T), (F, F)>`.
///
/// # Limitations
/// Since [`TCon::Apply`] requires being implemented for all `T: Sized`, type constructors with extra bounds
/// (for example `T -> &'a T` would require `T: 'a`) cannot be expressed by this.
pub const fn push_tcon<C: Uint, T, F, Con: TCon>(
    tern: Con::Apply<TernRaw<C, T, F>>,
) -> TernRaw<C, Con::Apply<T>, Con::Apply<F>> {
    // SAFETY: Input and output are the same type
    unsafe { utils::same_type_transmute(tern) }
}

pub trait TConLt<'a> {
    type Apply<T: 'a>;
}
pub struct TConLtRef;
impl<'a> TConLt<'a> for TConLtRef {
    type Apply<T: 'a> = &'a T;
}
pub struct TConLtMut;
impl<'a> TConLt<'a> for TConLtMut {
    type Apply<T: 'a> = &'a mut T;
}
pub const fn pull_tcon_lt<'a, C: Uint, T: 'a, F: 'a, Con: TConLt<'a>>(
    tern: TernRaw<C, Con::Apply<T>, Con::Apply<F>>,
) -> Con::Apply<TernRaw<C, T, F>> {
    // SAFETY: Input and output are the same type
    unsafe { utils::same_type_transmute(tern) }
}
pub const fn push_tcon_lt<'a, C: Uint, T: 'a, F: 'a, Con: TConLt<'a>>(
    tern: Con::Apply<TernRaw<C, T, F>>,
) -> TernRaw<C, Con::Apply<T>, Con::Apply<F>> {
    // SAFETY: Input and output are the same type
    unsafe { utils::same_type_transmute(tern) }
}

/// Unwraps the `True` value of a [`TernRaw`].
///
/// # Panics
/// If `C` is zero (even if `T` and `F` are the same type).
#[track_caller]
pub const fn expect_true<C: Uint, T, F>(tern: TernRaw<C, T, F>, msg: &str) -> T {
    if !uint::to_bool::<C>() {
        panic!("{}", msg);
    }
    // SAFETY: C is nonzero, therefore `tern` is of type `T`
    unsafe { utils::same_type_transmute(tern) }
}

/// Wraps the `True` value of a [`TernRaw`].
///
/// # Panics
/// If `C` is zero (even if `T` and `F` are the same type).
#[track_caller]
pub const fn wrap_true<C: Uint, T, F>(t: T, msg: &str) -> TernRaw<C, T, F> {
    if !uint::to_bool::<C>() {
        panic!("{}", msg);
    }
    // SAFETY: C is nonzero, therefore `TernRaw<C, T, F> = T`
    unsafe { utils::same_type_transmute(t) }
}

/// Unwraps the `False` value of a [`TernRaw`].
///
/// # Panics
/// If `C` is nonzero (even if `T` and `F` are the same type).
#[track_caller]
pub const fn expect_false<C: Uint, T, F>(tern: TernRaw<C, T, F>, msg: &str) -> F {
    if uint::to_bool::<C>() {
        panic!("{}", msg);
    }
    // SAFETY: C is zero, therefore `tern` is of type `T`
    unsafe { utils::same_type_transmute(tern) }
}

/// Wraps the `False` value of a [`TernRaw`].
///
/// # Panics
/// If `C` is nonzero (even if `T` and `F` are the same type).
#[track_caller]
pub const fn wrap_false<C: Uint, T, F>(f: F, msg: &str) -> TernRaw<C, T, F> {
    if !uint::to_bool::<C>() {
        panic!("{}", msg);
    }
    // SAFETY: C is zero, therefore `TernRaw<C, T, F> = F`
    unsafe { utils::same_type_transmute(f) }
}

macro_rules! match_tern_raw {
    ($C:ty, $tern:expr, |$tp:pat_param| $te:expr, |$fp:pat_param| $fe:expr $(,)?) => {{
        let __tern = $tern;
        match $crate::uint::to_bool::<$C>() {
            true => {
                let $tp = $crate::tern::raw::expect_true::<$C, _, _>(__tern, "unreachable");
                $te
            }
            false => {
                let $fp = $crate::tern::raw::expect_false::<$C, _, _>(__tern, "unreachable");
                $fe
            }
        }
    }};
}
pub(crate) use match_tern_raw;
