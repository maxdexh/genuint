//! Types conditional on a [`Uint`](crate::Uint)
//!
//! This module provides conditional types that depend on whether a `Uint` is zero.

macro_rules! condty_ctx {
    (|$ctxt:pat_param| $true:expr, |$ctxf:pat_param| $false:expr $(, $($C:ty $(,)?)?)?) => {{
        match $crate::__mac::cond::hold::<$($($C)?)?>() {
            Ok($ctxt) => $true,
            Err($ctxf) => $false,
        }
    }};
}
pub(crate) use condty_ctx;

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

/// A [`Result`]-like wrapper for [`CondDirect`]
///
/// If `Cond` is nonzero, instances of this type are always `Ok` instances with inner type `T`,
/// otherwise they are always `Err` instances with inner type `E`.
///
/// This type is a [`repr(transparent)`](https://doc.rust-lang.org/reference/type-layout.html#r-layout.repr.transparent)
/// wrapper around the corresponding instance kind. This means that `Ok` instances have the same layout as
/// `T` and `Err` instances have the same layout as `E`. No space is required to store the instance
/// kind.
#[repr(transparent)]
pub struct CondResult<Cond: ToUint, T, E> {
    /// The underlying [`CondDirect`]. The struct is `repr(transparent)` around this
    /// field.
    pub direct: CondDirect<Cond, T, E>,
}
impl<C: ToUint, T, E> CondResult<C, T, E> {
    /// Turns this result into its wrapped [`CondDirect`] by moving out of
    /// [`self.direct`](Self::direct).
    ///
    /// Also works in const contexts, even when generics or drop impls are involved.
    pub const fn into_direct(self) -> CondDirect<C, T, E> {
        // SAFETY: repr(transparent)
        unsafe {
            crate::utils::union_transmute!(
                CondResult::<C, T, E>, //
                CondDirect::<C, T, E>,
                self,
            )
        }
    }

    /// Whether instances of this type are `Ok`
    pub const IS_OK: bool = uint::is_nonzero::<C>();

    /// Whether instances of this type are `Err`
    pub const IS_ERR: bool = !Self::IS_OK;

    /// Whether this result is `Ok`
    pub const fn is_ok(&self) -> bool {
        uint::is_nonzero::<C>()
    }

    /// Whether this result is `Err`
    pub const fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /// Shorthand for `Self { direct }`.
    pub const fn from_direct(direct: CondDirect<C, T, E>) -> Self {
        Self { direct }
    }

    /// Equivalent of [`Result::as_ref`].
    pub const fn as_ref(&self) -> CondResult<C, &T, &E> {
        CondResult::from_direct(direct::as_ref::<C, _, _>(&self.direct))
    }

    /// Equivalent of [`Result::as_mut`].
    pub const fn as_mut(&mut self) -> CondResult<C, &mut T, &mut E> {
        CondResult::from_direct(direct::as_mut::<C, _, _>(&mut self.direct))
    }

    /// Turns this result into a regular builtin [`Result`].
    #[expect(clippy::missing_errors_doc)]
    pub const fn into_builtin(self) -> Result<T, E> {
        condty_ctx!(
            //
            |c| Ok(c.unwrap_ok(self)),
            |c| Err(c.unwrap_err(self))
        )
    }

    /// Creates an `Ok` instance, assuming [`Self::IS_OK`]
    ///
    /// # Panics
    /// If [`Self::IS_ERR`]
    #[track_caller]
    pub const fn new_ok(ok: T) -> Self {
        condty_ctx!(
            //
            |c| c.new_ok(ok),
            |_| panic!("Call to `new_ok` on Err type")
        )
    }

    /// Equivalent of [`Result::unwrap`], but `const` and without the [`Debug`] bound.
    ///
    /// # Panics
    /// If [`Self::IS_ERR`]
    pub const fn unwrap(self) -> T {
        condty_ctx!(
            //
            |c| c.unwrap_ok(self),
            |_| panic!("Call to `unwrap` on Err type")
        )
    }

    /// Creates an `Err` instance, assuming [`Self::IS_ERR`]
    ///
    /// # Panics
    /// If [`Self::IS_OK`]
    #[track_caller]
    pub const fn new_err(err: E) -> Self {
        condty_ctx!(
            //
            |_| panic!("Call to `new_err` on Ok type"),
            |c| c.new_err(err),
        )
    }

    /// Equivalent of [`Result::unwrap_err`], but `const` and without the [`Debug`] bound.
    ///
    /// # Panics
    /// If [`Self::IS_OK`]
    pub const fn unwrap_err(self) -> E {
        condty_ctx!(
            //
            |_| panic!("Call to `unwrap_err` on Ok type"),
            |c| c.unwrap_err(self),
        )
    }

    /// Wraps the content of this result in [`ManuallyDrop`].
    ///
    /// This may make it easier to destructure [`Self::into_builtin`] in `const` contexts when generics or
    /// [`Drop`] impls are involved.
    #[must_use = "The content of this result are wrapped in ManuallyDrop and may need to be dropped"]
    #[allow(clippy::missing_errors_doc)]
    pub const fn into_manual_drop(self) -> CondResult<C, ManuallyDrop<T>, ManuallyDrop<E>> {
        condty_ctx!(
            //
            |c| c.new_ok(ManuallyDrop::new(c.unwrap_ok(self))),
            |c| c.new_err(ManuallyDrop::new(c.unwrap_err(self))),
        )
    }
}

impl<C: ToUint, T> CondResult<C, T, T> {
    /// Creates a result where both instance kinds have the same type.
    pub const fn new_trivial(inner: T) -> Self {
        Self::from_direct(direct::new_trivial::<C, _>(inner))
    }

    /// Unwraps a result where both instance kinds have the same type.
    pub const fn unwrap_trivial(self) -> T {
        direct::unwrap_trivial::<C, _>(self.into_direct())
    }
}

/// An [`Option`]-like wrapper for [`CondDirect`]
///
/// This struct is a `repr(transparent)` newtype wrapper for [`CondDirect<Cond, T, ()>`].
/// If `Cond` is zero, then this struct is a `repr(transparent)` wrapper around `E`.
/// Otherwise, it is a `repr(transparent)` wrapper around `()`.
#[repr(transparent)]
pub struct CondOption<Cond: ToUint, T> {
    /// The underlying [`CondDirect`]. The struct is `repr(transparent)` around this
    /// field.
    pub direct: CondDirect<Cond, T, ()>,
}

impl<C: ToUint, T> CondOption<C, T> {
    /// Turns this option into its wrapped [`CondDirect`] by moving out of
    /// [`self.direct`](Self::direct).
    ///
    /// Also works in const contexts, even when generics or drop impls are involved.
    pub const fn into_direct(self) -> CondDirect<C, T, ()> {
        // SAFETY: repr(transparent)
        unsafe {
            crate::utils::union_transmute!(
                CondOption::<C, T>, //
                CondDirect::<C, T, ()>,
                self,
            )
        }
    }

    /// Whether instances of this type are `Some`
    pub const IS_SOME: bool = uint::is_nonzero::<C>();

    /// Whether instances of this type are `None`
    pub const IS_NONE: bool = !Self::IS_SOME;

    /// Whether this option is `Some`
    pub const fn is_some(&self) -> bool {
        Self::IS_SOME
    }

    /// Whether this result is `None`
    pub const fn is_none(&self) -> bool {
        uint::is_nonzero::<C>()
    }

    /// Shorthand for `Self { direct }`.
    pub const fn from_direct(direct: CondDirect<C, T, ()>) -> Self {
        Self { direct }
    }

    /// Turns this option into a regular builtin [`Option`].
    pub const fn into_builtin(self) -> Option<T> {
        condty_ctx!(
            //
            |c| Some(c.unwrap_some(self)),
            |c| {
                c.drop_none(self);
                None
            },
        )
    }

    /// Wraps the inner type of this option in [`ManuallyDrop`].
    ///
    /// This may make it easier to do pattern matching after converting via [`Self::into_builtin`].
    pub const fn into_manual_drop(self) -> CondOption<C, ManuallyDrop<T>> {
        condty_ctx!(
            |c| c.new_some(ManuallyDrop::new(c.unwrap_some(self))),
            |c| {
                c.drop_none(self);
                c.new_none()
            }
        )
    }

    /// Equivalent of [`Option::as_ref`]
    pub const fn as_ref(&self) -> CondOption<C, &T> {
        condty_ctx!(
            //
            |c| c.new_some(c.unwrap_true(direct::as_ref::<C, _, _>(&self.direct))),
            |c| c.new_none(),
        )
    }

    /// Equivalent of [`Option::as_mut`]
    pub const fn as_mut(&mut self) -> CondOption<C, &mut T> {
        condty_ctx!(
            //
            |c| c.new_some(c.unwrap_true(direct::as_mut::<C, _, _>(&mut self.direct))),
            |c| c.new_none(),
        )
    }

    /// Equivalent of [`Option::unwrap`]
    ///
    /// # Panics
    /// If [`Self::IS_NONE`]
    pub const fn unwrap(self) -> T {
        condty_ctx!(
            //
            |c| c.unwrap_some(self),
            |_| panic!("Call to `unwrap` on None instance"),
        )
    }

    /// Discards the value in a `const` context, assuming that [`Self::IS_NONE`]
    ///
    /// # Panics
    /// If [`Self::IS_SOME`]
    #[track_caller]
    pub const fn assert_none(self) {
        condty_ctx!(
            //
            |_| panic!("Call to `assert_none` on Some instance"),
            |c| c.drop_none(self),
        )
    }
}
