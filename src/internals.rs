#![allow(clippy::use_self)]

use crate::{
    ToUint, Uint,
    array::Array,
    uimpl::{_0, _1, A},
};

// NOTE: items from this module with names starting with _,
// except the above, are not meant to be used from anywhere
// but this module. This includes associated items.

pub trait _Tern {
    type _Tern<T, F>;
}
impl<C: Uint> _Tern for C {
    type _Tern<T, F> = InternalOp!(C::ToUint, TernRaw<T, F>);
}
pub type TernRaw<C, T, F> = <C as _Tern>::_Tern<T, F>;

pub type _Internals<N> = <N as UintSealed>::__Uint;
macro_rules! InternalOp {
    ($N:ty, $($item:tt)*) => {
        <crate::internals::_Internals<$N> as crate::internals::_Uint>::$($item)*
    };
}
pub(crate) use InternalOp;

pub trait ArraySealed {}

// Map the intenral API to the public one using an
// undocumented associated type. Since _Uint is in
// a private module, __Internals cannot be used to
// access the operations.
pub trait UintSealed: 'static {
    // Not public API
    #[doc(hidden)]
    type __Uint: _Uint;
}
pub trait _Uint: _Arrays + 'static {
    const IS_NONZERO: bool;

    // This needs to evaluate directly to `T` or `F` because it is observable
    // for generic `T` and `F` (not that one could do anything else, since there
    // are no trait bounds)
    type TernRaw<T, F>;

    // These are exposed only through structs implementing ToUint, so we can
    // do the ToUint conversion on the result here directly. This has the
    // advantage of making errors more readable, since if this was `: ToUint`,
    // then `uint::From<If<C, T, F>>` would normalize to
    // <<< C as UintSealed>::__Uint
    //       as _Uint>::IfImpl<T, F>
    //       as ToUint>::ToUint
    // Converting to `Uint` here removes the final `ToUint` conversion.
    //
    // WARN: Do not project to _Uint early, since that would cause
    // `uint::From<If<_1, T, F>>` (T: Uint) to normalize to
    // `<T as UintSealed>::__Uint`, rather than `T`.
    type If<T: ToUint, F: ToUint>: Uint;
    type Opaque<N: ToUint>: Uint;

    // Opaque in all arguments, including `Self`. Thus it's safe to return
    // _Uint directly, to save some projections.
    // This makes errors more readable, e.g. `uint::From<Half<Half<N>>>`
    // (N: Uint) normalizes to
    // <<< N as UintSealed>::__Uint
    //       as _Uint>::Half
    //       as _Uint>::Half
    // Without having to project to _Uint for the second primitive operation.
    type Half: _Uint;
    type Parity: _Uint;
    type AppendMeAsBit<N: Uint>: _Uint;

    // AppendBit<N, P> has to project through N and P to make the operation
    // opaque with respect to both, so simply implementing with a helper
    // `_ToBit: _Bit` doesn't work, because `uint::From<Half<AppendBit<Const, P>>>`
    // would normalize to `Const`.
    type _DirectAppend<B: _Bit>: _Uint;
}

//
pub trait _Pint: _Uint {}
pub trait _Bit: _Uint {}

#[diagnostic::do_not_recommend]
impl<N: _Uint> UintSealed for N {
    type __Uint = N;
}
#[diagnostic::do_not_recommend]
impl<N: _Uint> Uint for N {}
#[diagnostic::do_not_recommend]
impl<N: _Uint> ToUint for N {
    type ToUint = Self;
}

// 0
impl _Bit for _0 {}
impl _Uint for _0 {
    const IS_NONZERO: bool = false;

    type TernRaw<T, F> = F;

    type If<T: ToUint, F: ToUint> = F::ToUint;
    type Opaque<N: ToUint> = N::ToUint;

    type Half = _0;
    type Parity = _0;

    type AppendMeAsBit<N: Uint> = InternalOp!(N, _DirectAppend<Self>);
    type _DirectAppend<B: _Bit> = B;
}

// 1
impl _Bit for _1 {}
impl _Pint for _1 {}
impl _Uint for _1 {
    const IS_NONZERO: bool = true;

    type TernRaw<T, F> = T;

    type If<T: ToUint, F: ToUint> = T::ToUint;
    type Opaque<N: ToUint> = N::ToUint;

    type Half = _0;
    type Parity = _1;

    type AppendMeAsBit<N: Uint> = InternalOp!(N, _DirectAppend<Self>);
    type _DirectAppend<B: _Bit> = A<Self, B>;
}

// 2 * N + B where N > 0, B <= 1. Together with 0 and 1, this covers
// all non-negative integers.
impl<Pre: _Pint, Last: _Bit> _Pint for A<Pre, Last> {}
impl<Pre: _Pint, Last: _Bit> _Uint for A<Pre, Last> {
    const IS_NONZERO: bool = true;

    type TernRaw<T, F> = T;

    type If<T: ToUint, F: ToUint> = T::ToUint;
    type Opaque<N: ToUint> = N::ToUint;

    type Half = Pre;
    type Parity = Last;

    type AppendMeAsBit<N: Uint> = InternalOp!(N, _DirectAppend<_1>);
    type _DirectAppend<B: _Bit> = A<Self, B>;
}

// Array internals. Expressed through a supertrait of _Uint so that it can be generated more
// easily, and extended by future special traits like `Freeze`.
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
impl _Arrays for _0 {
    impl_body_zero!();
}
impl _Arrays for _1 {
    impl_body_one!();
}
impl<Pre: _Pint, Pop: _Bit> _Arrays for A<Pre, Pop> {
    impl_body_bisect!(Pre, Pop);
}
