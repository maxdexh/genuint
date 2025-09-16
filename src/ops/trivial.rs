use super::*;

/// Evaluates to [`_1`] if `N` is zero, else [`_0`].
#[apply(opaque! pub(crate) is_zero::_IsZero)]
pub type IsZero<N> = If<N, _0, _1>;

/// Evaluates to [`_0`] if `N` is zero, else [`_1`].
#[apply(opaque! is_truthy::_IsTruthy)]
pub type IsTruthy<N> = If<N, _1, _0>;
