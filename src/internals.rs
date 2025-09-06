use crate::{ToUint, Uint, array::Array};

pub struct U<N>(N);
pub struct I;
pub struct O;
pub struct A<H, P>(H, P);

pub(crate) type _0 = U<O>;
pub(crate) type _1 = U<I>;

pub trait UintSealed: 'static {
    type __Ops: _Uint;
}

pub trait _Uint: Arrays + 'static {
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
    type AppendAsBit<B: Uint> = U<A<Self, InternalOp!(B, ::_AsBit)>>;

    type _AsBit = Self;
}
impl<H: _Pint, P: _Bit> _Uint for A<H, P> {
    const IS_NONZERO: bool = true;

    type Ternary<T: ToUint, F: ToUint> = T;
    type Opaque<N: ToUint> = N;

    type Half = U<H>;
    type Parity = U<P>;
    type AppendAsBit<B: Uint> = U<A<Self, InternalOp!(B, ::_AsBit)>>;

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

#[repr(transparent)]
pub struct ArrImpl<T, N: Uint, Bound: ArrBound<T, N>>(Bound::Arr);
// SAFETY: repr(transparent); `Bound::Arr` was recursively constructed to be a valid implementor
unsafe impl<T, N: Uint, Bound: ArrBound<T, N>> Array for ArrImpl<T, N, Bound> {
    type Item = T;
    type Length = N;
}
impl<T: Copy, N: Uint, Bound: ArrBound<T, N, Arr: Copy>> Copy for ArrImpl<T, N, Bound> {}
impl<T: Copy, N: Uint, Bound: ArrBound<T, N, Arr: Copy>> Clone for ArrImpl<T, N, Bound> {
    fn clone(&self) -> Self {
        *self
    }
}

macro_rules! gen_arr_internals {
    (
        $($name:ident[$($($bound:tt)+)?])*
    ) => {
        pub trait Arrays {
            $(type $name<T: $($($bound)+)?> $(: $($bound)+)?;)*
        }
        impl Arrays for O {
            $(type $name<T: $($($bound)+)?> = [T; 0];)*
        }
        impl Arrays for I {
            $(type $name<T: $($($bound)+)?> = [T; 1];)*
        }
        impl<H: _Pint, P: _Bit> Arrays for A<H, P> {
            $(type $name<T: $($($bound)+)?> = ArrBisect<H::$name<T>, P::$name<T>>;)*
        }
        mod bounds {
            $(pub struct $name;)*
        }
        $(impl<T: $($($bound)+)?, N: Uint> ArrBound<T, N> for bounds::$name {
            type Arr = <N::__Ops as Arrays>::$name<T>;
        })*

        pub mod arr_reexports {
            $(pub type $name<T, N> = crate::array::ArrApi<super::ArrImpl<T, N, super::bounds::$name>>;)*
        }
    };
}

gen_arr_internals! {
    Arr[Sized]
    CopyArr[Copy]
}

pub trait _ArrApi {
    type Inner;
}
impl<A: Array> _ArrApi for crate::array::ArrApi<A> {
    type Inner = A;
}
