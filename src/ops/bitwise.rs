use super::*;

// TODO: Missing stuff

#[apply(lazy)]
pub type BitAndL<L, R> = Tern<L, AppendL<BitAndL<Half<R>, H<L>>, AndSC<P<L>, P<R>>>, U0>;

#[apply(opaque)]
pub type BitAnd<L, R> = BitAndL<L, R>;

test_op! { test_bit_and: L R, BitAnd<L, R>, L & R }
