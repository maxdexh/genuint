//! Functions for [`TernRaw`].

use crate::{
    ToUint,
    tern::TernRaw,
    tfun::{TFun, TFunLt},
    uint, utils,
};

/// Pulls an arbitrary [`TFun`] out of a [`TernRaw`].
///
/// This function is the inverse of [`push_tcon`].
///
/// For example, this function can turn `TernRaw<C, Option<T>, Option<F>>` into `Option<TernRaw<C, T, F>>`, or
/// `TernRaw<C, (T, T), (F, F)>` into `(TernRaw<C, T, F>, TernRaw<C, T, F>)`.
///
/// # Limitations
/// Since [`TFun::Apply`] requires being implemented for all `T: Sized`, type constructors with extra bounds
/// (for example `T -> &'a T` would require `T: 'a`) cannot be expressed by this. It is possible to
/// work around this by duplicating the definition of this struct
pub const fn pull_tcon<C: ToUint, T, F, Con: TFun>(
    tern: TernRaw<C, Con::Apply<T>, Con::Apply<F>>,
) -> Con::Apply<TernRaw<C, T, F>> {
    // SAFETY: Input and output are the same type
    unsafe {
        utils::same_type_transmute!(
            TernRaw::<C, Con::Apply<T>, Con::Apply<F>>,
            Con::Apply<TernRaw<C, T, F>>,
            tern,
        )
    }
}

/// Pushes an arbitrary [`TFun`] out of a [`TernRaw`].
///
/// This function is the inverse of [`pull_tcon`].
///
/// For example, this function can turn `Option<TernRaw<C, T, F>>` into `TernRaw<C, Option<T>, Option<F>>`, or
/// `(TernRaw<C, T, F>, TernRaw<C, T, F>)` into `TernRaw<C, (T, T), (F, F)>`.
///
/// # Limitations
/// Since [`TFun::Apply`] requires being implemented for all `T: Sized`, type constructors with extra bounds
/// (for example `T -> &'a T` would require `T: 'a`) cannot be expressed by this.
pub const fn push_tcon<C: ToUint, T, F, Con: TFun>(
    tern: Con::Apply<TernRaw<C, T, F>>,
) -> TernRaw<C, Con::Apply<T>, Con::Apply<F>> {
    // SAFETY: Input and output are the same type
    unsafe {
        utils::same_type_transmute!(
            Con::Apply<TernRaw<C, T, F>>,
            TernRaw::<C, Con::Apply<T>, Con::Apply<F>>,
            tern,
        )
    }
}

/// Like [`pull_tcon`], but for [`TFunLt`].
pub const fn pull_tcon_lt<'a, C: ToUint, T: 'a, F: 'a, Con: TFunLt<'a>>(
    tern: TernRaw<C, Con::Apply<T>, Con::Apply<F>>,
) -> Con::Apply<TernRaw<C, T, F>> {
    // SAFETY: Input and output are the same type
    unsafe {
        utils::same_type_transmute!(
            TernRaw::<C, Con::Apply<T>, Con::Apply<F>>,
            Con::Apply<TernRaw<C, T, F>>,
            tern,
        )
    }
}
/// Like [`push_tcon`], but for [`TFunLt`].
pub const fn push_tcon_lt<'a, C: ToUint, T: 'a, F: 'a, Con: TFunLt<'a>>(
    tern: Con::Apply<TernRaw<C, T, F>>,
) -> TernRaw<C, Con::Apply<T>, Con::Apply<F>> {
    // SAFETY: Input and output are the same type
    unsafe {
        utils::same_type_transmute!(
            Con::Apply<TernRaw<C, T, F>>,
            TernRaw::<C, Con::Apply<T>, Con::Apply<F>>,
            tern
        )
    }
}

/// Unwraps the `True` value of a [`TernRaw`].
///
/// # Panics
/// If `C` is zero (even if `T` and `F` are the same type).
#[track_caller]
pub const fn expect_true<C: ToUint, T, F>(tern: TernRaw<C, T, F>, msg: &str) -> T {
    if !uint::to_bool::<C>() {
        panic!("{}", msg);
    }
    // SAFETY: C is nonzero, therefore `tern` is of type `T`
    unsafe { utils::same_type_transmute!(TernRaw<C, T, F>, T, tern) }
}

/// Wraps the `True` value of a [`TernRaw`].
///
/// # Panics
/// If `C` is zero (even if `T` and `F` are the same type).
#[track_caller]
pub const fn wrap_true<C: ToUint, T, F>(t: T, msg: &str) -> TernRaw<C, T, F> {
    if !uint::to_bool::<C>() {
        panic!("{}", msg);
    }
    // SAFETY: C is nonzero, therefore `TernRaw<C, T, F> = T`
    unsafe { utils::same_type_transmute!(T, TernRaw<C, T, F>, t) }
}

/// Unwraps the `False` value of a [`TernRaw`].
///
/// # Panics
/// If `C` is nonzero (even if `T` and `F` are the same type).
#[track_caller]
pub const fn expect_false<C: ToUint, T, F>(tern: TernRaw<C, T, F>, msg: &str) -> F {
    if uint::to_bool::<C>() {
        panic!("{}", msg);
    }
    // SAFETY: C is zero, therefore `tern` is of type `T`
    unsafe { utils::same_type_transmute!(TernRaw<C, T, F>, F, tern) }
}

/// Wraps the `False` value of a [`TernRaw`].
///
/// # Panics
/// If `C` is nonzero (even if `T` and `F` are the same type).
#[track_caller]
pub const fn wrap_false<C: ToUint, T, F>(f: F, msg: &str) -> TernRaw<C, T, F> {
    if !uint::to_bool::<C>() {
        panic!("{}", msg);
    }
    // SAFETY: C is zero, therefore `TernRaw<C, T, F> = F`
    unsafe { utils::same_type_transmute!(F, TernRaw<C, T, F>, f) }
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
