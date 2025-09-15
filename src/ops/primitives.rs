use super::*;

// Just show e.g. Half<N> = _Half<N> in the docs.
mod real {
    use super::*;
    #[apply(lazy)]
    pub type _Half<N> = InternalOp!(uint::From<N>, Half);
    #[apply(lazy)]
    pub type _Parity<N> = InternalOp!(uint::From<N>, Parity);
    #[apply(lazy)]
    pub type _AppendBit<N, P> = InternalOp!(uint::From<P>, AppendMeAsBit<uint::From<N>>);
    #[apply(lazy)]
    pub type _If<C, T, F> = InternalOp!(uint::From<C>, If<T, F>);
    #[apply(lazy)]
    pub type _Opaque<P, Out> = uint::From<InternalOp!(uint::From<P>, Opaque<Out>)>;
}

/// Halves the value of a number by removing one bit from the end.
///
/// Effectively a more efficient implementation of [`Div<N, _2>`] or [`Shr<N, _1>`].
/// This, together with [`Parity`] and [`If`], can be used to implement operations that
/// recursively destructure a number and accumulate a result.
///
/// See the [module level documentation](crate::ops) for details on how to combine
/// primitive operations.
pub type Half<N> = real::_Half<N>;

/// Calculates the parity of a number (whether it is even or odd) by getting its last bit.
///
/// Effectively a more efficient implementation of [`Rem<N, _2>`] or [`BitAnd<N, _1>`].
/// This, together with [`Half`] and [`If`], can be used to implement operations that
/// recursively destructure a number and accumulate a result.
///
/// See the [module level documentation](crate::ops) for details on how to combine
/// primitive operations.
pub type Parity<N> = real::_Parity<N>;

/// Adds a single bit to the end of a number.
///
/// Effectively a more efficient implementation of `Add<Mul<N, _2>, IsTruthy<P>>`,
/// or `BitOr<Shl<N, _1>, IsTruthy<P>>`. It is meant to be used for building the
/// output of an operation recursively bit-by-bit.
///
/// See the [module level documentation](crate::ops) for details on how to combine
/// primitive operations.
pub type AppendBit<N, P> = real::_AppendBit<N, P>;

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
pub type If<Cond, Then, Else> = real::_If<Cond, Then, Else>;

/// Makes `Out` opaque with respect to the value of a parameter `P`.
///
/// This operation just evaluates to the same value as `Out`, but only after
/// going through a projection via an internal associated [`Uint`] type on
/// [`P::ToUint`](ToUint).
///
/// See the [module level documentation](crate::ops) for details on opaqueness.
pub type Opaque<P, Out> = real::_Opaque<P, Out>;

#[test]
fn opaqueness_tests() {
    struct Wat<L, R, const CLAIM_EQ: bool>(L, R);
    trait HasMethod {
        const CONST: ();
    }
    // `Wat` has a trait const
    impl<L, R, const CLAIM_EQ: bool> HasMethod for Wat<L, R, CLAIM_EQ> {
        const CONST: () = assert!(!CLAIM_EQ);
    }
    // It also has an inherent method of the same name, but only if
    // L and R are the same type! Inherent methods are resolved first,
    // so this method is called if and only if the compiler can prove
    // that L = R.
    impl<L, const CLAIM_EQ: bool> Wat<L, L, CLAIM_EQ> {
        // Check that equality was claimed
        const CONST: () = assert!(CLAIM_EQ);
    }
    macro_rules! check_eq {
        ($lhs:ty, $rhs:ty) => {
            _ = Wat::<$lhs, $rhs, true>::CONST
        };
    }
    macro_rules! check_neq {
        ($lhs:ty, $rhs:ty) => {
            _ = Wat::<$lhs, $rhs, false>::CONST
        };
    }
    fn accept<A: ToUint, B: ToUint>() {
        // types that are provably the same
        check_eq!(uint::From<If<_1, A, B>>, uint::From<A>);
        check_eq!(uint::From<If<_0, A, B>>, uint::From<B>);
        check_eq!(uint::From<Opaque<_0, A>>, uint::From<A>);

        // types that are not provably the same
        check_neq!(uint::From<Opaque<B, A>>, uint::From<A>);
        check_neq!(uint::From<Opaque<B, A>>, Opaque<A, A>);
        check_neq!(uint::From<Half<AppendBit<_0, A>>>, _0);
    }
    accept::<_3, _7>();
}
