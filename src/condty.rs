//! Types conditional on a [`Uint`](crate::Uint)
//!
//! This module provides conditional types that depend on whether a `Uint` is zero.
//! This is not "conditional typing" in the Haskell sense, but a simple type projection.

pub mod direct;

use core::mem::ManuallyDrop;

use crate::{ToUint, uint};

/// Direct conditional type based on a [`Uint`](crate::Uint).
///
/// "Direct" in this context refers to the fact that the ternary is implemented as a
/// type alias to an internal associated type on `Uint`, i.e. it is not newtype wrapped
/// (unlike [`CondResult`]).
/// The type of `CondDirect<Cond, True, False>` depends directly on `Cond`. If `Cond` is nonzero,
/// then `CondDirect<Cond, T, F>` is exactly the same type as `T`. Otherwise it is the same type as `F`.
///
/// As a consequence any generic `TFun<CondDirect<C, T, F>>` is exactly the same type as `TFun<T>` or
/// `TFun<F>` and therefore is valid to transmute given a known `C` (which can be runtime checked)
/// or `T = F` (which may follow from other invariants, such as [`Uint`](crate::Uint) uniqueness.
/// This applies even to types with unspecified layout such as `TFun<X> = Vec<X>` or type
/// projections like `TFun<X> = <X as Tr>::Assoc`.
///
/// This type's disadvantage compared to [`CondResult`] are the usual use cases for a newtype wrapper:
/// It is not possible to use impls of `T` and `F` if `C` is generic, it does not play nicely with
/// type inferrence (especially of `C`) and it can't have methods. Its "methods" are defined as free
/// standing functions in the [`direct`] module.
pub type CondDirect<Cond, True, False> =
    crate::internals::CondDirect<<Cond as ToUint>::ToUint, True, False>;

/// A [`Result`]-like type that only has ok or error instances, depending on a [`ToUint`] condition.
///
/// This struct is implemented as a `repr(transparent)` newtype wrapper for [`CondDirect`].
/// If `Cond` is zero, then this struct is a `repr(transparent)` wrapper around `E`. Otherwise, it
/// is a `repr(transparent)` wrapper around `T`.
#[repr(transparent)]
pub struct CondResult<Cond: ToUint, T, E> {
    /// The underlying [`CondDirect`].
    pub direct: CondDirect<Cond, T, E>,
}
impl<C: ToUint, T, E> CondResult<C, T, E> {
    /// Whether instances of this type are ok, i.e. `to_bool::<C>()`
    pub const IS_OK: bool = uint::to_bool::<C>();
    /// Whether instances of this type are errors, i.e. `!to_bool::<C>()`
    pub const IS_ERR: bool = !Self::IS_OK;

    /// Shorthand for `Self { direct }`.
    pub const fn from_direct(direct: CondDirect<C, T, E>) -> Self {
        Self { direct }
    }

    /// Does the same thing as moving `self.raw`, but also works in `const` contexts.
    pub const fn into_raw(self) -> CondDirect<C, T, E> {
        // SAFETY: `self` is by value and this struct is ok to unwrap by read
        unsafe {
            crate::utils::destruct_read!(Self, { direct: raw }, self);
            raw
        }
    }
    /// Equivalent of [`Result::as_ref`].
    pub const fn as_ref(&self) -> CondResult<C, &T, &E> {
        CondResult::from_direct(direct::as_ref::<C, _, _>(&self.direct))
    }
    /// Equivalent of [`Result::as_mut`].
    pub const fn as_mut(&mut self) -> CondResult<C, &mut T, &mut E> {
        CondResult::from_direct(direct::as_mut::<C, _, _>(&mut self.direct))
    }
    /// Turns this result in to a regular [`Result`].
    #[allow(clippy::missing_errors_doc)]
    pub const fn into_result(self) -> Result<T, E> {
        direct::match_tern_raw!(C, self.into_raw(), |t| Ok(t), |f| Err(f))
    }

    /// Turns `T` into `Self` assuming `Self::IS_OK`
    ///
    /// # Panics
    /// If `Self::IS_ERR`
    pub const fn make_ok(ok: T) -> Self {
        Self::from_direct(direct::wrap_true::<C, _, _>(
            ok,
            "Call to `make_ok` on error `CondResult`",
        ))
    }

    /// Turns `T` into `Self` assuming `Self::IS_ERR`
    ///
    /// # Panics
    /// If `Self::IS_OK`
    pub const fn make_err(err: E) -> Self {
        Self::from_direct(direct::wrap_false::<C, _, _>(
            err,
            "Call to `make_err` on ok `CondResult`",
        ))
    }

    /// Wraps the variants of this result in [`ManuallyDrop`].
    ///
    /// This may make it easier to destructure [`Self::into_result`] in `const` contexts when generics or
    /// [`Drop`] impls are involved.
    #[must_use = "The variants of this result are wrapped in ManuallyDrop and may need to be dropped"]
    #[allow(clippy::missing_errors_doc)]
    pub const fn into_manual_drop(self) -> CondResult<C, ManuallyDrop<T>, ManuallyDrop<E>> {
        // SAFETY: repr(transparent)
        unsafe {
            crate::utils::same_size_transmute!(
                CondResult::<C, T, E>,
                CondResult::<C, ManuallyDrop<T>, ManuallyDrop<E>>,
                self
            )
        }
    }
    /// Equivalent of [`Result::unwrap`], but uses a generic message so it's usable in `const` and
    /// without [`Debug`] bounds.
    pub const fn unwrap(self) -> T {
        direct::expect_true::<C, _, _>(self.into_raw(), "Call to `unwrap` on error variant")
    }

    /// Equivalent of [`Result::unwrap_err`], but uses a generic message so it's usable in `const` and
    /// without [`Debug`] bounds.
    pub const fn unwrap_err(self) -> E {
        direct::expect_false::<C, _, _>(self.into_raw(), "Call to `unwrap_err` on ok variant")
    }
}

impl<C: ToUint, T> CondResult<C, T, T> {
    /// Creates a result where both variants have the same type.
    pub const fn new_trivial(x: T) -> Self {
        Self::from_direct(
            // SAFETY: CondDirect<C, T, T> is the same type as T or T, so it is T
            unsafe { crate::utils::same_size_transmute!(T, CondDirect::<C, T, T>, x) },
        )
    }
    /// Unwraps a result where both variants have the same type.
    pub const fn unwrap_trivial(self) -> T {
        // SAFETY: CondDirect<C, T, T> is the same type as T or T, so it is T
        unsafe { crate::utils::same_size_transmute!(CondDirect::<C, T, T>, T, self.into_raw()) }
    }
}

pub struct CondOption<C: ToUint, T> {
    raw: CondDirect<C, T, ()>,
}
