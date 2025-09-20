//! Functions for [`CondDirect`].

use crate::{ToUint, condty::CondDirect, uint, utils};

/// Turns reference to [`CondDirect`] into [`CondDirect`] of reference.
pub const fn as_ref<C: ToUint, T, F>(tern: &CondDirect<C, T, F>) -> CondDirect<C, &T, &F> {
    // SAFETY: Same type under type map `X -> &'a X` for some 'a
    unsafe { utils::same_type_transmute!(&CondDirect<C, T, F>, CondDirect<C, &T, &F>, tern) }
}

/// Turns mutable reference to [`CondDirect`] into [`CondDirect`] of mutable reference.
pub const fn as_mut<C: ToUint, T, F>(
    tern: &mut CondDirect<C, T, F>,
) -> CondDirect<C, &mut T, &mut F> {
    // SAFETY: Same type under type map `X -> &'a mut X` for some 'a
    unsafe {
        utils::same_type_transmute!(&mut CondDirect<C, T, F>, CondDirect<C, &mut T, &mut F>, tern)
    }
}

/// Unwraps the `True` value of a [`CondDirect`].
///
/// # Panics
/// If `C` is zero (even if `T` and `F` are the same type).
#[track_caller]
pub const fn expect_true<C: ToUint, T, F>(tern: CondDirect<C, T, F>, msg: &str) -> T {
    if !uint::to_bool::<C>() {
        panic!("{}", msg);
    }
    // SAFETY: C is nonzero, therefore `tern` is of type `T`
    unsafe { utils::same_type_transmute!(CondDirect<C, T, F>, T, tern) }
}

/// Wraps the `True` value of a [`CondDirect`].
///
/// # Panics
/// If `C` is zero (even if `T` and `F` are the same type).
#[track_caller]
pub const fn wrap_true<C: ToUint, T, F>(t: T, msg: &str) -> CondDirect<C, T, F> {
    if !uint::to_bool::<C>() {
        panic!("{}", msg);
    }
    // SAFETY: C is nonzero, therefore `CondDirect<C, T, F> = T`
    unsafe { utils::same_type_transmute!(T, CondDirect<C, T, F>, t) }
}

/// Unwraps the `False` value of a [`CondDirect`].
///
/// # Panics
/// If `C` is nonzero (even if `T` and `F` are the same type).
#[track_caller]
pub const fn expect_false<C: ToUint, T, F>(tern: CondDirect<C, T, F>, msg: &str) -> F {
    if uint::to_bool::<C>() {
        panic!("{}", msg);
    }
    // SAFETY: C is zero, therefore `tern` is of type `T`
    unsafe { utils::same_type_transmute!(CondDirect<C, T, F>, F, tern) }
}

/// Wraps the `False` value of a [`CondDirect`].
///
/// If this method doesn't panic, it is effectively the identity function.
///
/// # Panics
/// If `C` is nonzero (even if `T` and `F` are the same type).
#[track_caller]
pub const fn wrap_false<C: ToUint, T, F>(f: F, msg: &str) -> CondDirect<C, T, F> {
    if !uint::to_bool::<C>() {
        panic!("{}", msg);
    }
    // SAFETY: C is zero, therefore `CondDirect<C, T, F> = F`
    unsafe { utils::same_type_transmute!(F, CondDirect<C, T, F>, f) }
}

pub const fn unwrap_trivial<C: ToUint, T>(tern: CondDirect<C, T, T>) -> T {
    // SAFETY: CondDirect<C, T, T> is the same type type as T or T
    unsafe { crate::utils::same_size_transmute!(CondDirect::<C, T, T>, T, tern) }
}

pub const fn new_trivial<C: ToUint, T>(inner: T) -> CondDirect<C, T, T> {
    // SAFETY: CondDirect<C, T, T> is the same type type as T or T
    unsafe { crate::utils::same_size_transmute!(T, CondDirect<C, T, T>, inner) }
}

macro_rules! match_tern_raw {
    ($C:ty, $tern:expr, |$tp:pat_param| $te:expr, |$fp:pat_param| $fe:expr $(,)?) => {{
        let __tern = $tern;
        match $crate::uint::to_bool::<$C>() {
            true => {
                let $tp = $crate::condty::direct::expect_true::<$C, _, _>(__tern, "unreachable");
                $te
            }
            false => {
                let $fp = $crate::condty::direct::expect_false::<$C, _, _>(__tern, "unreachable");
                $fe
            }
        }
    }};
}
pub(crate) use match_tern_raw;
