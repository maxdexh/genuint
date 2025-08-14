use super::*;

#[apply(lazy)]
pub type TernL<C, T, F> = Tern<C, T, F>;

#[apply(lazy)]
pub type AppendL<N, P> = AppendBit<N, P>;

// Short-circuiting And
pub type AndSC<L, R> = Tern<L, R, U0>;
// Short-circuiting Or
pub type OrSC<L, R> = Tern<L, U1, R>;

pub type BitNot<N> = Tern<N, U0, U1>;
pub type AndL<L, R> = TernL<L, R, U0>;
pub type Xor<L, R> = Tern<L, BitNot<R>, R>;
pub type Xnor<L, R> = Tern<L, R, BitNot<R>>;
pub type Xor3<A, B, C> = Tern<A, Xnor<B, C>, Xor<B, C>>;

pub use super::{Half as H, Parity as P};
