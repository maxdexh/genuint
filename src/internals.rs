#![allow(clippy::use_self)]

use crate::{ToUint, Uint, array::Array};

// Alias so other modules do not have to refer to internals directly, which may change
pub(crate) type _0 = O;
pub(crate) type _1 = I;

pub struct I;
pub struct O;

pub struct A<H, P>(H, P);

pub trait ArraySealed {}

pub trait UintSealed: 'static {
    // Not public API
    #[doc(hidden)]
    type __Internals: _Uint;
}
pub type _Internals<N> = <N as UintSealed>::__Internals;
macro_rules! InternalOp {
    ($N:ty, $($item:tt)*) => {
        <crate::internals::_Internals<$N> as crate::internals::_Uint>$($item)*
    };
}
pub(crate) use InternalOp;

pub trait _Uint: _Arrays + 'static {
    const IS_NONZERO: bool;

    type TernRaw<T, F>;

    type Ternary<T: ToUint, F: ToUint>: ToUint;
    type Opaque<N: ToUint>: ToUint;

    type Half: Uint;
    type Parity: Uint;
    type AppendAsBit<B: Uint>: Uint;

    type _AsBit: _Bit;
}

pub trait TernRawImpl {
    type Tern<T, F>;
}
impl<C: ToUint> TernRawImpl for C {
    type Tern<T, F> = InternalOp!(C::ToUint, ::TernRaw<T, F>);
}
pub type TernRaw<C, T, F> = <C as TernRawImpl>::Tern<T, F>;

impl _Uint for O {
    const IS_NONZERO: bool = false;

    type TernRaw<T, F> = F;

    type Ternary<T: ToUint, F: ToUint> = F;
    type Opaque<N: ToUint> = N;

    type Half = _0;
    type Parity = _0;
    type AppendAsBit<B: Uint> = InternalOp!(B, ::_AsBit);

    type _AsBit = Self;
}
impl _Uint for I {
    const IS_NONZERO: bool = true;

    type TernRaw<T, F> = T;

    type Ternary<T: ToUint, F: ToUint> = T;
    type Opaque<N: ToUint> = N;

    type Half = _0;
    type Parity = _1;
    type AppendAsBit<B: Uint> = A<Self, InternalOp!(B, ::_AsBit)>;

    type _AsBit = Self;
}
impl<Pre: _Pint, Last: _Bit> _Uint for A<Pre, Last> {
    const IS_NONZERO: bool = true;

    type TernRaw<T, F> = T;

    type Ternary<T: ToUint, F: ToUint> = T;
    type Opaque<N: ToUint> = N;

    type Half = Pre;
    type Parity = Last;
    type AppendAsBit<B: Uint> = A<Self, InternalOp!(B, ::_AsBit)>;

    type _AsBit = I;
}

pub trait _Bit: _Uint<_AsBit = Self> {}
impl<N: _Uint<_AsBit = Self>> _Bit for N {}

pub trait _Pint: _Uint<_AsBit = I> {}
impl<N: _Uint<_AsBit = I>> _Pint for N {}

#[diagnostic::do_not_recommend]
impl<N: _Uint> UintSealed for N {
    type __Internals = N;
}
#[diagnostic::do_not_recommend]
impl<N: _Uint> Uint for N {}
#[diagnostic::do_not_recommend]
impl<N: _Uint> ToUint for N {
    type ToUint = Self;
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ArrBisect<A, P>(A, A, P);

macro_rules! gen_arr_internals {
    [
        $ArrsTrait:ident,
        [$(
            [
                $bound_name:ident,
                ($($bound:tt)*),

                $out_inner:ident,

                $doc:expr,
                $out:ident,
            ]
        ),* $(,)?],
        $wrap:ident,
    ] => {
        pub trait $ArrsTrait {$(
            type $bound_name<T: $($bound)*>: $($bound)*;
        )*}
        $(type $bound_name<T, N> = <_Internals<N> as crate::internals::$ArrsTrait>::$bound_name<T>;)*

        macro_rules! impl_body_zero { () => {$(
            type $bound_name<T: $($bound)*> = [T; 0];
        )*}}
        macro_rules! impl_body_one { () => {$(
            type $bound_name<T: $($bound)*> = [T; 1];
        )*}}
        macro_rules! impl_body_bisect { ($Pre:ident, $Pop:ident) => {$(
            type $bound_name<T: $($bound)*> = ArrBisect<Pre::$bound_name<T>, Pop::$bound_name<T>>;
        )*}}

        $(
            #[doc = core::concat!("The inner [`Array`] type of ", core::stringify!($out), ".")]
            #[cfg_attr(not(doc), repr(transparent))]
            pub struct $out_inner<T: $($bound)*, N: crate::Uint>($bound_name<T, N>);

            // SAFETY: repr(transparent), array was recursively constructed to be a valid implementor
            unsafe impl<T: $($bound)*, N: crate::Uint> Array for $out_inner<T, N> {
                type Item = T;
                type Length = N;
            }
            impl<T: $($bound)*, N: crate::Uint> ArraySealed for $out_inner<T, N> {}

            impl<T: $($bound)*, N: crate::Uint> Copy for $out_inner<T, N>
            where
                T: Copy,
                $bound_name<T, N>: Copy
            {
            }
            impl<T: $($bound)*, N: crate::Uint> Clone for $out_inner<T, N>
            where
                T: Copy,
                $bound_name<T, N>: Copy
            {
                fn clone(&self) -> Self {
                    *self
                }
            }

            #[doc = $doc]
            pub type $out<T, N> = $wrap<$out_inner<T, N>>;
        )*

        pub mod array_types { pub use super::{$($out_inner, $out),*}; }
    };
}
use crate::array::ArrApi;
gen_arr_internals![
    _Arrays,
    [
        [
            _Arr,
            (Sized),
            ArrInner,
            crate::utils::docexpr! {
                /// General [`Array`] implementation.
                ///
                /// See the [module level documentation](crate::array).
            },
            Arr,
        ],
        [
            _CopyArr,
            (Copy),
            CopyArrInner,
            crate::utils::docexpr! {
                /// [`Array`] implementation that implements [`Copy`] but requires `T: Copy`.
                ///
                /// See the [module level documentation](crate::array).
            },
            CopyArr,
        ],
    ],
    ArrApi,
];
impl _Arrays for O {
    impl_body_zero!();
}
impl _Arrays for I {
    impl_body_one!();
}
impl<Pre: _Pint, Pop: _Bit> _Arrays for A<Pre, Pop> {
    impl_body_bisect!(Pre, Pop);
}
