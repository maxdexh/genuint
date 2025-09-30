use super::*;

/// Checks if `N` is zero.
pub type IsFalsy<N> = If<N, _0, _1>;

/// Checks if `N` is nonzero.
pub type IsTruthy<N> = If<N, _1, _0>;
