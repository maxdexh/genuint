use super::*;

pub type Eager<N> = uint::From<N>;

// Short-circuiting And
pub type AndSC<L, R> = Eager<If<L, R, _0>>;
// Short-circuiting Or
pub type OrSC<L, R> = Eager<If<L, _1, R>>;

pub type BitNot<N> = Eager<If<N, _0, _1>>;
pub type AndL<L, R> = If<L, R, _0>;
pub type Xor<L, R> = Eager<If<L, BitNot<R>, R>>;
pub type Xnor<L, R> = Eager<If<L, R, BitNot<R>>>;
pub type Xor3<A, B, C> = Eager<If<A, Xnor<B, C>, Xor<B, C>>>;

/// Eager version of `Half`.
pub type H<N> = Eager<Half<N>>;
/// Eager version of `Parity`.
pub type P<N> = Eager<Parity<Eager<N>>>;
