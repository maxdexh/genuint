pub mod raw;

use core::mem::ManuallyDrop;

use crate::{ToUint, uint};
use raw::TernRaw;

/// A [`Result`]-like type that only has ok or error instances, depending on a [`Uint`] condition.
///
/// If `Cond` is zero, then this struct is a `repr(transparent)` wrapper around `E`. Otherwise, it
/// is a `repr(transparent)` wrapper around `T`.
#[repr(transparent)]
pub struct TernRes<Cond: ToUint, T, E> {
    /// The underlying [`TernRaw`].
    pub raw: TernRaw<Cond, T, E>,
}
impl<C: ToUint, T, E> TernRes<C, T, E> {
    /// Whether instances of this type are ok, i.e. `to_bool::<C>()`
    pub const IS_OK: bool = uint::to_bool::<C>();
    /// Whether instances of this type are errors, i.e. `!to_bool::<C>()`
    pub const IS_ERR: bool = !Self::IS_OK;

    /// Shorthand for `Self { raw }`.
    pub const fn from_raw(raw: TernRaw<C, T, E>) -> Self {
        Self { raw }
    }
    /// Does the same thing as moving `self.raw`, but also works in `const` contexts.
    pub const fn into_raw(self) -> TernRaw<C, T, E> {
        // SAFETY: `self` is by value and this struct is ok to unwrap by read
        unsafe {
            crate::utils::destruct_read!(Self, { raw: raw }, self);
            raw
        }
    }
    /// Equivalent of [`Result::as_ref`].
    pub const fn as_ref(&self) -> TernRes<C, &T, &E> {
        TernRes::from_raw(raw::push_tcon_lt::<C, T, E, crate::tcon::TConLtRef>(
            &self.raw,
        ))
    }
    /// Equivalent of [`Result::as_mut`].
    pub const fn as_mut(&mut self) -> TernRes<C, &mut T, &mut E> {
        TernRes::from_raw(raw::push_tcon_lt::<C, T, E, crate::tcon::TConLtMut>(
            &mut self.raw,
        ))
    }
    /// Turns this result in to a regular [`Result`].
    #[allow(clippy::missing_errors_doc)]
    pub const fn into_result(self) -> Result<T, E> {
        raw::match_tern_raw!(C, self.into_raw(), |t| Ok(t), |f| Err(f))
    }

    /// Turns `T` into `Self` assuming `Self::IS_OK`
    ///
    /// # Panics
    /// If `Self::IS_ERR`
    pub const fn make_ok(ok: T) -> Self {
        Self::from_raw(raw::wrap_true::<C, _, _>(
            ok,
            "Call to `make_ok` on error `TernRes`",
        ))
    }

    /// Turns `T` into `Self` assuming `Self::IS_ERR`
    ///
    /// # Panics
    /// If `Self::IS_OK`
    pub const fn make_err(err: E) -> Self {
        Self::from_raw(raw::wrap_false::<C, _, _>(
            err,
            "Call to `make_err` on ok `TernRes`",
        ))
    }

    /// Turns this result in to a regular [`Result`], but wraps the variants in [`ManuallyDrop`].
    ///
    /// This may make it easier to destructure in `const` contexts when generics or [`Drop`] impls
    /// are involved.
    #[must_use = "This Result's variants are wrapped in ManuallyDrop and may need cleanup"]
    #[allow(clippy::missing_errors_doc)]
    pub const fn into_man_drop_result(self) -> Result<ManuallyDrop<T>, ManuallyDrop<E>> {
        raw::match_tern_raw!(
            C,
            self.into_raw(), //
            |t| Ok(ManuallyDrop::new(t)),
            |f| Err(ManuallyDrop::new(f))
        )
    }
    /// Equivalent of [`Result::unwrap`], but uses a generic message so it's usable in `const` and
    /// without [`Debug`] bounds.
    pub const fn unwrap(self) -> T {
        raw::expect_true::<C, _, _>(self.into_raw(), "Call to `unwrap` on error variant")
    }

    /// Equivalent of [`Result::unwrap_err`], but uses a generic message so it's usable in `const` and
    /// without [`Debug`] bounds.
    pub const fn unwrap_err(self) -> E {
        raw::expect_false::<C, _, _>(self.into_raw(), "Call to `unwrap_err` on ok variant")
    }
}

impl<C: ToUint, T> TernRes<C, T, T> {
    /// Creates a result where both variants have the same type.
    pub const fn new_trivial(x: T) -> Self {
        Self::from_raw(raw::push_tcon::<C, T, T, crate::tcon::TConTrivial<T>>(x))
    }
    /// Unwraps a result where both variants have the same type.
    pub const fn into_trivial(self) -> T {
        raw::pull_tcon::<C, T, T, crate::tcon::TConTrivial<T>>(self.into_raw())
    }
}
