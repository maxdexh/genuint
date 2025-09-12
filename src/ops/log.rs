use super::*;

#[apply(lazy)]
pub type ILogUncheckedL<B, N> = Tern<
    //
    cmp::LtL<N, B>,
    _0,
    add::IncIfL<
        ILogUncheckedL<
            //
            B,
            divrem::DivUncheckedL<N, B>,
        >,
    >,
>;

#[apply(lazy)]
pub type ILogL<B, N> = Tern<
    // Check B > 1 and N > 0
    AndSC<H<B>, N>,
    ILogUncheckedL<B, N>,
    ILogL<B, N>,
>;

/// Type-level [`u128::ilog`].
///
/// # Errors
/// Using `B <= 1` or `N == 0` gives an "overflow while evaluating" error.
///
/// ```compile_fail
/// use generic_uint::{ops::ILog, uint, consts::*};
/// const _: fn(uint::From<ILog<_1, _0>>) = |_| {};
/// ```
#[apply(opaque)]
#[apply(test_op!
    test_ilog,
    N.ilog(B).into(),
    2..,
    1..,
)]
pub type ILog<B, N> = ILogL<B, N>;

#[apply(lazy)]
pub type BaseLenL<B, N> = Tern<H<B>, Tern<N, add::IncIfL<ILogL<B, N>>, _1>, BaseLenL<B, N>>;

/// Type-level version of `N.to_string().len()` in base `B`
///
/// # Errors
/// Using `B <= 1` gives an "overflow while evaluating" error.
///
/// ```compile_fail
/// use generic_uint::{ops::BaseLen, uint, consts::*};
/// const _: fn(uint::From<BaseLen<_1, _0>>) = |_| {};
/// ```
#[apply(opaque)]
#[apply(test_op! test_base_len, {
    let mut n = N;
    let mut r = 1;
    while n >= B {
        r += 1;
        n /= B;
    }
    r
}, 2..)]
pub type BaseLen<B, N> = BaseLenL<B, N>;
