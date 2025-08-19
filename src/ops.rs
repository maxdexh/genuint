use crate::{consts::*, internals, uint};
use generic_uint_proc::apply;

macro_rules! lazy {
    (
        $(())?
        $(#[$($attr:tt)*])*
        pub type $name:ident<$($param:ident $(= $def:ty)?),* $(,)?> = $val:ty;
    ) => {
        $(#[$($attr)*])*
        pub struct $name<$($param $(= $def)?),*>($($param),*);
        impl<$($param: $crate::ToUint),*> $crate::ToUint for $name<$($param),*> {
            type ToUint = $crate::uint::From<$val>;
        }
    };
}

macro_rules! __make_opaque {
    (, $out:ty) => {
        $out
    };
    ($pop:ident $($param:ident)*, $out:ty) => {
        $crate::internals::PrimitiveOp!(
            $crate::uint::From<$pop>,
            ::Opaque<
                $crate::ops::__make_opaque!($($param)*, $out)
            >
        )
    };
}
pub(crate) use __make_opaque;

macro_rules! opaque {
    (
        $(())?
        $(#[$($attr:tt)*])*
        pub type $name:ident<$($param:ident $(= $def:ty)?),* $(,)?> = $val:ty;
    ) => {
        $(#[$($attr)*])*
        pub type $name<$($param $(= $def)?),*> = $crate::uint::From<
            $crate::ops::__make_opaque!($($param)*, $val)
        >;
    };
}

macro_rules! test_op {
    (
        ($name:ident: $($args:tt)*)
        $(#[$($attr:tt)*])*
        $v:vis $kw:ident $tname:ident<$($param:ident $(= $def:ty)?),* $(,)?> $($rest:tt)*
    ) => {
        #[cfg(all(test, not(miri)))]
        $crate::ops::testing::test_op! { $name: $($param)*, $tname<$($param),*>, $($args)* }

        $(#[$($attr)*])*
        $v $kw $tname<$($param $(= $def)?),*> $($rest)*
    };
}

/// More efficient implementation of [`Div<N, U2>`].
///
/// This is currently a primitive operation.
// H(N) := N / 2
pub type Half<N> = internals::PrimitiveOp!(uint::From<N>, ::Half);

/// More efficient implementation of [`Rem<N, U2>`].
///
/// This is currently a primitive operation.
// P(N) := N % 2
pub type Parity<N> = internals::PrimitiveOp!(uint::From<N>, ::Parity);

/// More efficient implementation of `Add<Mul<N, U2>, Tern<P, U1, U0>>``
///
/// Equivalent to `2 * N + (P != 0) as _` in basic arithmetic or `(N << 1) | (P != 0) as _`
/// in bit manipulation. This operation is useful for building the output of operations
/// recursively bit-by-bit.
///
/// This is currently a primitive operation.
// Append(N, P) := 2 * N + if P { 1 } else { 0 }
pub type AppendBit<N, P> = internals::PrimitiveOp!(uint::From<N>, ::AppendAsBit<uint::From<P>>);

/// If-else/Ternary operation.
///
/// If the first argument is nonzero, evaluates to the second argument, otherwise to the third.
///
/// This will only access ("evaluate") [`ToUint::ToUint`](crate::ToUint::ToUint) for the required
/// argument. This means that this operation can be used for the exit condition of a recursive
/// operation (see examples below).
///
/// This is currently a primitive operation.
///
/// # Examples
/// Exiting from a recursive operation
/// ```
/// use generic_uint::{ToUint, ops, uint};
/// struct CountOnesL<N, Acc>(N, Acc);
/// impl<N: ToUint, Acc: ToUint> ToUint for CountOnesL<N, Acc> {
///     type ToUint = ops::Tern<
///         N,
///         CountOnesL<
///             ops::Half<N>,
///             ops::Add<Acc, ops::Parity<N>>,
///         >,
///         Acc,
///     >;
/// }
/// type CountOnes<N> = uint::From<CountOnesL<N, uint::FromU128<0>>>;
/// assert_eq!(
///     uint::to_u128::<CountOnes<uint::FromU128<0b1011101>>>(),
///     Some(5),
/// );
/// ```
//
// Tern(C, T, F) := if C { T } else { F }
pub type Tern<C, T, F> = uint::From<internals::PrimitiveOp!(uint::From<C>, ::Ternary<T, F>)>;

mod helper;
pub(crate) use helper::*;

mod satdec;

#[cfg(all(test, not(miri)))]
// We need this to iterate over ranges of uints in tests
pub type SatDecForTest<N> = uint::From<satdec::SatDecIfL<N>>;

#[cfg(all(test, not(miri)))]
mod testing;

mod bitmath;
pub use bitmath::{BitAnd, BitOr, BitXor, CountOnes};

mod log;
pub use log::{BaseLen, ILog};

mod add;
pub use add::Add;

mod mul;
pub use mul::Mul;

mod cmp;
pub use cmp::{Eq, Ge, Gt, Le, Lt, Max, Min, Ne};

mod sub;
pub use sub::{AbsDiff, SatSub};

mod divrem;
pub use divrem::{Div, Rem};

mod shift;
pub use shift::{Shl, Shr};

mod pow;
pub use pow::Pow;
