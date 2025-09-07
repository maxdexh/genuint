use crate::{ToUint, Uint, array::Array};

pub struct U<N>(N);
pub struct I;
pub struct O;

pub(crate) type _0 = U<O>;
pub(crate) type _1 = U<I>;

pub trait ArraySealed {}

pub trait UintSealed: 'static {
    type __Ops: _Uint;
}

pub trait _Uint: _Arrays + 'static {
    const IS_NONZERO: bool;

    type Ternary<T: ToUint, F: ToUint>: ToUint;
    type Opaque<N: ToUint>: ToUint;

    type Half: Uint;
    type Parity: Uint;
    type AppendAsBit<B: Uint>: Uint;

    type _AsBit: _Bit;
}
pub(crate) type _PrimOps<N> = <N as UintSealed>::__Ops;
macro_rules! InternalOp {
    ($N:ty, $($item:tt)*) => {
        <crate::internals::_PrimOps<$N> as crate::internals::_Uint>$($item)*
    };
}
pub(crate) use InternalOp;
impl _Uint for O {
    const IS_NONZERO: bool = false;

    type Ternary<T: ToUint, F: ToUint> = F;
    type Opaque<N: ToUint> = N;

    type Half = _0;
    type Parity = _0;
    type AppendAsBit<B: Uint> = U<InternalOp!(B, ::_AsBit)>;

    type _AsBit = Self;
}
impl _Uint for I {
    const IS_NONZERO: bool = true;

    type Ternary<T: ToUint, F: ToUint> = T;
    type Opaque<N: ToUint> = N;

    type Half = _0;
    type Parity = _1;
    type AppendAsBit<B: Uint> = U<(Self, InternalOp!(B, ::_AsBit))>;

    type _AsBit = Self;
}
impl<H: _Pint, P: _Bit> _Uint for (H, P) {
    const IS_NONZERO: bool = true;

    type Ternary<T: ToUint, F: ToUint> = T;
    type Opaque<N: ToUint> = N;

    type Half = U<H>;
    type Parity = U<P>;
    type AppendAsBit<B: Uint> = U<(Self, InternalOp!(B, ::_AsBit))>;

    type _AsBit = I;
}

pub trait _Bit: _Uint<_AsBit = Self> {}
impl<N: _Uint<_AsBit = Self>> _Bit for N {}

pub trait _Pint: _Uint<_AsBit = I> {}
impl<N: _Uint<_AsBit = I>> _Pint for N {}

impl<N: _Uint> UintSealed for U<N> {
    type __Ops = N;
}
impl<N: _Uint> Uint for U<N> {}
impl<N: _Uint> ToUint for U<N> {
    type ToUint = Self;
}

/// # Safety
/// Internal API
pub trait ArrBound<T, N: Uint> {
    type Arr;
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ArrBisect<A, P>([A; 2], P);

/// Implementation detail of the different recursive array implementors that uses a sentinel `Bound`
/// type to distinguish between them.
#[repr(transparent)]
pub struct ArrImpl<Bound: ArrBound<T, N>, T, N: Uint>(Bound::Arr);

// SAFETY: repr(transparent); `Bound::Arr` was recursively constructed to be a valid implementor
unsafe impl<T, N: Uint, Bound: ArrBound<T, N>> Array for ArrImpl<Bound, T, N> {
    type Item = T;
    type Length = N;
}
impl<T, N: Uint, Bound: ArrBound<T, N>> ArraySealed for ArrImpl<Bound, T, N> {}

impl<T: Copy, N: Uint, Bound: ArrBound<T, N, Arr: Copy>> Copy for ArrImpl<Bound, T, N> {}
impl<T: Copy, N: Uint, Bound: ArrBound<T, N, Arr: Copy>> Clone for ArrImpl<Bound, T, N> {
    fn clone(&self) -> Self {
        *self
    }
}

crate::utils::expand! {
    {
        pub trait _Arrays {$(
            type $name<T: $bound>: $bound;
        )}
        // `_Arrays` is a supertrait of `_Uint`, so it can be implemented using seperate
        // impls for each `_Uint` implementor while still working with generics.
        impl _Arrays for O {$(
            type $name<T: $bound> = [T; 0];
        )}
        impl _Arrays for I {$(
            type $name<T: $bound> = [T; 1];
        )}
        impl<H: _Pint, P: _Bit> _Arrays for (H, P) {$(
            type $name<T: $bound> = ArrBisect<H::$name<T>, P::$name<T>>;
        )}

        // bound type sentinels
        mod bounds {$(
            pub struct $name;
            impl<T: $bound, N: crate::Uint> super::ArrBound<T, N> for $name {
                type Arr = <N::__Ops as super::_Arrays>::$name<T>;
            }
        )}

        pub mod array_types {
            use crate::{internals::*, array::*};
            $(
                $doc
                pub type $out<T, N> = ArrApi<ArrImpl<bounds::$name, T, N>>;
            )
        }
    }
    [
        {doc}
        // Seperate `$out` and `$name` so LSPs show the right docs (since `$name`
        // is used for and therefore spans multiple declarations)
        out
        name
        (bound)
    ]
    [
        {
            /// Implementation of `Array` for arbitrary `N: Uint`.
            ///
            /// Wrapped in [`ArrApi`].
        }
        Arr
        Arr
        (Sized)
    ]
    [
        {
            /// Implementation of `Array + Copy` for arbitrary `N: Uint`.
            ///
            /// Wrapped in [`ArrApi`].
        }
        CopyArr
        CopyArr
        (Copy)
    ]
}

pub trait _ArrApi {
    type Inner;
}
impl<A: Array> _ArrApi for crate::array::ArrApi<A> {
    type Inner = A;
}
