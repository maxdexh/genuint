//! Functions for [`CondDirect`].

use crate::{ToUint, condty::CondDirect, utils};

/// Unwraps a `True` instance a [`CondDirect`].
///
/// # Panics
/// If `C` is zero (even if `T` and `F` are the same type).
#[track_caller]
pub const fn unwrap_true<C: ToUint, T, F>(tern: CondDirect<C, T, F>) -> T {
    condty_ctx!(
        //
        |c| c.unwrap_true(tern),
        |_| panic!("Call to `unwrap_true` with false condition"),
        C,
    )
}

/// Creates a `True` instance of [`CondDirect`]
///
/// # Panics
/// If `C` is zero (even if `T` and `F` are the same type).
#[track_caller]
pub const fn new_true<C: ToUint, T, F>(value: T) -> CondDirect<C, T, F> {
    condty_ctx!(
        //
        |c| c.new_true(value),
        |_| panic!("Call to `new_true` with false condition"),
        C,
    )
}

/// Unwraps a `False` instance a [`CondDirect`].
///
/// # Panics
/// If `C` is nonzero (even if `T` and `F` are the same type).
#[track_caller]
pub const fn unwrap_false<C: ToUint, T, F>(tern: CondDirect<C, T, F>) -> F {
    condty_ctx!(
        //
        |_| panic!("Call to `unwrap_false` with true condition"),
        |c| c.unwrap_false(tern),
        C,
    )
}

/// Creates a `False` instance of [`CondDirect`]
///
/// # Panics
/// If `C` is nonzero (even if `T` and `F` are the same type).
#[track_caller]
pub const fn new_false<C: ToUint, T, F>(value: F) -> CondDirect<C, T, F> {
    condty_ctx!(
        //
        |_| panic!("Call to `new_false` with true condition"),
        |c| c.new_false(value),
        C,
    )
}

/// Turns reference to [`CondDirect`] into [`CondDirect`] of reference.
pub const fn as_ref<C: ToUint, T, F>(tern: &CondDirect<C, T, F>) -> CondDirect<C, &T, &F> {
    // SAFETY: Same type under type map `X -> &'a X` for some 'a
    unsafe { utils::same_type_transmute!(&CondDirect::<C, T, F>, CondDirect::<C, &T, &F>, tern) }
}

/// Turns mutable reference to [`CondDirect`] into [`CondDirect`] of mutable reference.
pub const fn as_mut<C: ToUint, T, F>(
    tern: &mut CondDirect<C, T, F>,
) -> CondDirect<C, &mut T, &mut F> {
    // SAFETY: Same type under type map `X -> &'a mut X` for some 'a
    unsafe {
        utils::same_type_transmute!(
            &mut CondDirect::<C, T, F>,
            CondDirect::<C, &mut T, &mut F>,
            tern
        )
    }
}

/// Turns `CondDirect<C, T, T>` into `T`
///
/// This function is effectively the identity function.
pub const fn unwrap_trivial<C: ToUint, T>(tern: CondDirect<C, T, T>) -> T {
    // SAFETY: CondDirect<C, T, T> is the same type type as T or T
    unsafe { crate::utils::same_type_transmute!(CondDirect::<C, T, T>, T, tern) }
}

/// Turns `T` into `CondDirect<C, T, T>`
///
/// This function is effectively the identity function.
pub const fn new_trivial<C: ToUint, T>(inner: T) -> CondDirect<C, T, T> {
    // SAFETY: CondDirect<C, T, T> is the same type type as T or T
    unsafe { crate::utils::same_type_transmute!(T, CondDirect::<C, T, T>, inner) }
}
