//! Functions for [`TernRaw`].

use crate::{ToUint, tern::TernRaw, uint, utils};

/// Turns reference to [`TernRaw`] into [`TernRaw`] of reference.
pub const fn as_ref<C: ToUint, T, F>(tern: &TernRaw<C, T, F>) -> TernRaw<C, &T, &F> {
    // SAFETY: Same type under type map `X -> &'a X` for some 'a
    unsafe { utils::same_type_transmute!(&TernRaw<C, T, F>, TernRaw<C, &T, &F>, tern) }
}

/// Turns mutable reference to [`TernRaw`] into [`TernRaw`] of mutable reference.
pub const fn as_mut<C: ToUint, T, F>(tern: &mut TernRaw<C, T, F>) -> TernRaw<C, &mut T, &mut F> {
    // SAFETY: Same type under type map `X -> &'a mut X` for some 'a
    unsafe { utils::same_type_transmute!(&mut TernRaw<C, T, F>, TernRaw<C, &mut T, &mut F>, tern) }
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
