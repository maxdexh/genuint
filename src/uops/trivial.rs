use super::*;

/// Checks if `N` is zero.
pub type IsZero<N> = If<N, _0, _1>;

/// Checks if `N` is nonzero.
pub type IsNonzero<N> = If<N, _1, _0>;
