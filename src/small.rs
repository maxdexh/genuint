//! Small [`Uint`](crate::Uint) constants

macro_rules! new_alias {
    ($name:ident, $val:literal, $ty:ty) => {
        #[doc = core::concat!("`", $val, "` as a [`Uint`](crate::Uint)")]
        pub type $name = $ty;
        #[diagnostic::do_not_recommend]
        impl crate::ToUint for crate::consts::ConstU128<$val> {
            type ToUint = $name;
        }
        #[diagnostic::do_not_recommend]
        impl crate::ToUint for crate::consts::ConstUsize<$val> {
            type ToUint = $name;
        }
    };
}

new_alias!(U0, 0, crate::uimpl::_0);
new_alias!(U1, 1, crate::uimpl::_1);

macro_rules! bisect {
    ($name:ident, $val:expr, $half:ty, $parity:ty) => {
        new_alias! { $name, $val, crate::uimpl::_U<$half, $parity> }
    };
}
include!(concat!(env!("OUT_DIR"), "/consts.rs"));
