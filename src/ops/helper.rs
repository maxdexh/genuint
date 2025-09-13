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

/// Eager version of `Half`.
pub type H<N> = Norm<Half<N>>;
/// Eager version of `Parity`.
pub type P<N> = Norm<Parity<Norm<N>>>;

#[apply(lazy)]
pub type _Opaque<P, Out> = uint::From<InternalOp!(uint::From<P>, ::Opaque<Out>)>;
#[apply(lazy)]
pub type _Half<N> = InternalOp!(uint::From<N>, ::Half);
#[apply(lazy)]
pub type _Parity<N> = InternalOp!(uint::From<N>, ::Parity);
#[apply(lazy)]
pub type _AppendBit<N, P> = InternalOp!(uint::From<N>, ::AppendAsBit<uint::From<P>>);
#[apply(lazy)]
pub type _Tern<C, T, F> = InternalOp!(uint::From<C>, ::Ternary<T, F>);
