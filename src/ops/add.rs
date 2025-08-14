use super::*;

#[apply(lazy)]
pub type IncIfL<N, C> = Tern<C, AppendL<IncIfL<H<N>, P<N>>, BitNot<P<N>>>, N>;

#[apply(lazy)]
pub type AddL<L, R, C = U0> = Tern<
    L,
    AppendL<AddL<H<L>, H<R>, Tern<P<L>, OrSC<P<R>, C>, AndSC<P<R>, C>>>, Xor3<P<L>, P<R>, C>>,
    IncIfL<R, C>,
>;

#[apply(opaque)]
pub type Add<L, R> = add::AddL<L, R>;

test_op! { test_add: L R, Add<L, R>, L + R }
