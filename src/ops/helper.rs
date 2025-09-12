use super::*;

pub type Norm<N> = uint::From<N>;

// Short-circuiting And
pub type AndSC<L, R> = Norm<Tern<L, R, _0>>;
// Short-circuiting Or
pub type OrSC<L, R> = Norm<Tern<L, _1, R>>;

pub type BitNot<N> = Norm<Tern<N, _0, _1>>;
pub type AndL<L, R> = Tern<L, R, _0>;
pub type Xor<L, R> = Norm<Tern<L, BitNot<R>, R>>;
pub type Xnor<L, R> = Norm<Tern<L, R, BitNot<R>>>;
pub type Xor3<A, B, C> = Norm<Tern<A, Xnor<B, C>, Xor<B, C>>>;

pub type H<N> = Norm<Half<N>>;
pub type P<N> = Norm<Parity<N>>;
