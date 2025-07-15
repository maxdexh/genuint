use crate::{ToUint, Uint};

pub struct I;
pub struct O;
pub struct A<H, P>(H, P);

pub trait UintSealed: 'static {
    type __Ops: PrimOps;
}
pub trait PrimOps {
    const IS_NONZERO: bool;
    type Ternary<T: ToUint, F: ToUint>: ToUint;
    type Half: Uint;
    type Parity: Uint;
    type AsBit: UintBit;
    type AppendAsBit<B: Uint>: Uint;
}
macro_rules! Prim {
    ($Self:ty, $($item:tt)*) => {
        <<$Self as $crate::internals::UintSealed>::__Ops as $crate::internals::PrimOps>::$($item)*
    };
}
pub(crate) use Prim;

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
    type AppendAsBit<B: Uint> = Prim!(B, AsBit);
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
    type AppendAsBit<B: Uint> = A<Self, Prim!(B, AsBit)>;
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
    type AppendAsBit<B: Uint> = A<Self, Prim!(B, AsBit)>;
}
