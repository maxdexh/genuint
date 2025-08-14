use crate::{ToUint, Uint, array::Array};

pub struct I;
pub struct O;
pub struct A<H, P>(H, P);

pub trait UintSealed: 'static {
    type __Ops: PrimOps;
}
pub trait PrimOps: Arrays {
    const IS_NONZERO: bool;
    type Ternary<T: ToUint, F: ToUint>: ToUint;
    type Half: Uint;
    type Parity: Uint;
    type AsBit: UintBit;
    type AppendAsBit<B: Uint>: Uint;
}
macro_rules! PrimitiveOp {
    ($Self:ty, ::$($item:tt)*) => {
        <<$Self as $crate::internals::UintSealed>::__Ops as $crate::internals::PrimOps>::$($item)*
    };
}
pub(crate) use PrimitiveOp;

pub trait UintBit: Uint<__Ops: PrimOps<AsBit = Self>> {}
impl<N: Uint<__Ops: PrimOps<AsBit = Self>>> UintBit for N {}

pub trait UintPos: Uint<__Ops: PrimOps<AsBit = I>> {}
impl<N: Uint<__Ops: PrimOps<AsBit = I>>> UintPos for N {}

impl Uint for O {}
impl ToUint for O {
    type ToUint = Self;
}
impl UintSealed for O {
    type __Ops = Self;
}
impl PrimOps for O {
    const IS_NONZERO: bool = false;
    type Ternary<T: ToUint, F: ToUint> = F;
    type Half = Self;
    type Parity = Self;
    type AsBit = Self;
    type AppendAsBit<B: Uint> = PrimitiveOp!(B, ::AsBit);
}

impl Uint for I {}
impl ToUint for I {
    type ToUint = Self;
}
impl UintSealed for I {
    type __Ops = Self;
}
impl PrimOps for I {
    const IS_NONZERO: bool = true;
    type Ternary<T: ToUint, F: ToUint> = T;
    type Half = O;
    type Parity = Self;
    type AsBit = Self;
    type AppendAsBit<B: Uint> = A<Self, PrimitiveOp!(B, ::AsBit)>;
}

impl<H: UintPos, P: UintBit> UintSealed for A<H, P> {
    type __Ops = Self;
}
impl<H: UintPos, P: UintBit> ToUint for A<H, P> {
    type ToUint = Self;
}
impl<H: UintPos, P: UintBit> Uint for A<H, P> {}
impl<H: UintPos, P: UintBit> PrimOps for A<H, P> {
    const IS_NONZERO: bool = true;
    type Ternary<T: ToUint, F: ToUint> = T;
    type Half = H;
    type Parity = P;
    type AsBit = I;
    type AppendAsBit<B: Uint> = A<Self, PrimitiveOp!(B, ::AsBit)>;
}

/// # Safety
/// Internal API
pub unsafe trait ArrBound<T, N: Uint> {
    type Arr;
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ArrBisect<A, P>([A; 2], P);

#[repr(transparent)]
pub struct ArrImpl<T, N: Uint, Bound: ArrBound<T, N>>(Bound::Arr);
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
        impl<HA: Arrays, PA: Arrays, H: UintPos<__Ops = HA>, P: UintBit<__Ops = PA>> Arrays for A<H, P> {
            $(type $name<T: $($($bound)+)?> = ArrBisect<HA::$name<T>, PA::$name<T>>;)*
        }
        mod bounds {
            $(pub struct $name;)*
        }
        $(unsafe impl<T: $($($bound)+)?, N: Uint> ArrBound<T, N> for bounds::$name {
            type Arr = <N::__Ops as Arrays>::$name<T>;
        })*

        pub mod arr_reexports {
            $(pub type $name<T, N> = crate::array::ArrApi<super::ArrImpl<T, N, super::bounds::$name>>;)*
        }
    };
}

gen_arr_internals! {
    Arr[]
    CopyArr[Copy]
}

pub trait _ArrApi {
    type Inner;
}
impl<A: Array> _ArrApi for crate::array::ArrApi<A> {
    type Inner = A;
}
