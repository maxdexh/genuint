use super::*;

/// Halves the value of a [`Uint`] by removing one bit from the end.
///
/// Effectively a more efficient implementation of [`Div<N, _2>`].
///
/// See the [module level documentation](crate::ops) for details on how to combine
/// primitive operations.
pub type Half<N> = _Half<N>;
#[apply(lazy)]
pub type _Half<N> = InternalOp!(uint::From<N>, Half);

/// More efficient implementation of [`Rem<N, _2>`].
///
/// See the [module level documentation](crate::ops) for details on how to combine
/// primitive operations.
pub type Parity<N> = _Parity<N>;
#[apply(lazy)]
pub type _Parity<N> = InternalOp!(uint::From<N>, Parity);

/// Adds a single bit to the end of a number.
///
/// Effectively a more efficient implementation of `Add<Mul<N, _2>, IsTruthy<P>>`,
/// or `BitOr<Shl<N, _1>, IsTruthy<P>>`; both of these operations use this one
/// internally. It is meant to be used for building the output of an operation recursively
/// bit-by-bit.
///
/// See the [module level documentation](crate::ops) for details on how to combine
/// primitive operations.
pub type AppendBit<N, P> = _AppendBit<N, P>;
#[apply(lazy)]
pub type _AppendBit<N, P> = InternalOp!(uint::From<P>, AppendMeAsBit<uint::From<N>>);

/// If-else/Ternary operation.
///
/// If `Cond` is truthy, then `uint::From<If<Cond, Then, Else>>` is the same as
/// `uint::From<Then>`. Otherwise, it is the same as `uint::From<Else>`.
/// Only the resulting argument has its [`ToUint::ToUint`] implementation accessed,
/// i.e. the other branch is not evaluated and thus cannot lead to cycles. This allows
/// breaking out of recursively implemented operations.
///
/// See the [module level documentation](crate::ops) for details on how to combine
/// primitive operations.
///
/// # Opaqueness
/// This operation is not opaque in `Then` and `Else`. If `Cond` is known, then
/// the compiler might normalize this to `Then` or `Else`.
pub type If<Cond, Then, Else> = _If<Cond, Then, Else>;
#[apply(lazy)]
pub type _If<C, T, F> = InternalOp!(uint::From<C>, If<T, F>);

/// Makes `Out` opaque with respect to the value of a parameter `P`.
///
/// Wrapping a public operation `<Op<A1, A2, ...> as ToUint>::ToUint = Val` as
/// `Opaque<A1, Opaque<A2, Opaque<..., Op<A1, A2, ...>>>>` has the following effects:
/// - It will cause the compiler to not recursively normalize operations with one
///   (large) constant and one generic input, like
///
/// See the [module level documentation](crate::ops) for details on opaqueness.
pub type Opaque<P, Out> = _Opaque<P, Out>;
#[apply(lazy)]
pub type _Opaque<P, Out> = uint::From<InternalOp!(uint::From<P>, Opaque<Out>)>;
