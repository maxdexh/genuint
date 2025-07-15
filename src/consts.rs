pub struct ConstUsize<const N: usize>;
pub struct ConstU128<const N: u128>;

macro_rules! generate {
    ($name:ident, $val:expr, $ty:ty) => {
        pub type $name = $ty;
        impl $crate::ToUint for ConstUsize<{ $val }> {
            type ToUint = $name;
        }
        impl $crate::ToUint for ConstU128<{ $val }> {
            type ToUint = $name;
        }
    };
}
generate!(U0, 0, crate::internals::O);
generate!(U1, 1, crate::internals::I);

macro_rules! bisect {
    ($name:ident, $val:expr, $half:ty, $parity:ty) => {
        generate! { $name, $val, $crate::internals::A<$half, $parity> }
    };
}
macro_rules! big {
    ([ $($out:tt)* ]) => {
        $($out)*
    };
    ([ $($out:tt)* ] $first:ident $($bits:ident)*) => {
        big!([ $crate::internals::A<$($out)*, $first> ] $($bits)*)
    };
    ( $first:ident $($bits:ident)* ) => {
        big!( [$first] $($bits)* )
    };
}
include!(concat!(env!("OUT_DIR"), "/consts.rs"));
