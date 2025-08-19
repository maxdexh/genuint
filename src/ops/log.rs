use super::*;

#[apply(lazy)]
pub type ILogUncheckedL<B, N> = Tern<
    //
    cmp::LtL<N, B>,
    U0,
    add::IncIfL<
        ILogUncheckedL<
            //
            B,
            divrem::DivUncheckedL<N, B>,
        >,
    >,
>;

#[apply(lazy)]
#[apply(test_op! test_ilog: N.ilog(B).into(), 2.., 1..)]
pub type ILogL<B, N> = Tern<
    // Check B > 1 and N > 0
    AndSC<H<B>, N>,
    ILogUncheckedL<B, N>,
    ILogL<B, N>,
>;

#[apply(opaque)]
pub type ILog<B, N> = ILogL<B, N>;

#[apply(opaque)]
#[apply(test_op! test_base_len: {
    let mut n = N;
    let mut r = 1;
    while n >= B {
        r += 1;
        n /= B;
    }
    r
}, 2.., 1..)]
pub type BaseLen<B, N> = Tern<N, add::IncIfL<ILogL<B, N>>, U1>;
