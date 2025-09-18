use super::*;

#[apply(lazy! unchecked_rec)]
type _ILogUncheckedNormRec<B, N> = _ILogUnchecked<
    B,
    // Normalize recursive argument
    uint::From<_DivUnchecked<N, B>>,
>;
#[apply(lazy! unchecked)]
type _ILogUnchecked<B, N> = If<
    //
    _Lt<N, B>,
    _0,
    _Inc<_ILogUncheckedNormRec<B, N>>,
>;

/// Type-level [`ilog`](u128::ilog).
///
/// # Errors
/// Using `B <= 1` or `N == 0` gives an "overflow while evaluating" error.
///
/// ```compile_fail
/// use genuint::{ops::ILog, uint, consts::*};
/// const _: fn(uint::From<ILog<_1, _0>>) = |_| {};
/// ```
#[apply(opaque! ilog::_ILog)]
#[apply(test_op!
    test_ilog,
    N.ilog(B).into(),
    2..,
    1..,
)]
pub type ILog<B, N> = If<
    // Check B > 1 and N > 0
    _And<_H<B>, N>,
    _ILogUnchecked<B, N>,
    // Recurse infinitely
    _ILog<B, N>,
>;

/// Type-level version of `N.to_string().len()` in base `B`
///
/// # Errors
/// Using `B <= 1` gives an "overflow while evaluating" error.
///
/// ```compile_fail
/// use genuint::{ops::BaseLen, uint, consts::*};
/// const _: fn(uint::From<BaseLen<_1, _0>>) = |_| {};
/// ```
#[apply(opaque! base_len::_BaseLen)]
#[apply(test_op! test_base_len, {
    let mut n = N;
    let mut r = 1;
    while n >= B {
        r += 1;
        n /= B;
    }
    r
}, 2..)]
pub type BaseLen<B, N> = If<
    _H<B>, // H<B> = 0 iff B <= 1
    If<
        N,
        // If B > 1 and N > 0, length in base B is just ILog + 1
        _Inc<_ILogUnchecked<B, N>>,
        // The length of 0 is 1
        _1,
    >,
    // Recurse infinitely
    _BaseLen<B, N>,
>;
