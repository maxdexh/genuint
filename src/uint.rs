use crate::{ToUint, Uint, ops};

pub type From<N> = <N as ToUint>::ToUint;
pub type FromUsize<const N: usize> = From<crate::consts::ConstUsize<N>>;
pub type FromU128<const N: u128> = From<crate::consts::ConstU128<N>>;

pub const fn to_bool<N: ToUint>() -> bool {
    crate::internals::Prim!(N::ToUint, IS_NONZERO)
}
pub const fn to_str<N: ToUint>() -> &'static str {
    todo!()
}

macro_rules! to_u_ovfl {
    ($N:ty, $as:ty) => {{
        const fn doit<N: $crate::Uint>() -> ($as, bool) {
            const {
                if $crate::uint::to_bool::<N>() {
                    let (h, o1) = doit::<$crate::ops::Half<N>>();
                    let (t, o2) = h.overflowing_mul(2);
                    let (n, o3) =
                        t.overflowing_add($crate::uint::to_bool::<$crate::ops::Parity<N>>() as _);
                    (n, o1 || o2 || o3)
                } else {
                    (0, false)
                }
            }
        }
        doit::<$N>()
    }};
}
pub const fn to_usize_overflowing<N: ToUint>() -> (usize, bool) {
    to_u_ovfl!(N::ToUint, usize)
}
pub const fn to_usize<N: ToUint>() -> Option<usize> {
    match to_usize_overflowing::<N>() {
        (n, false) => Some(n),
        (_, true) => None,
    }
}
pub const fn to_u128_overflowing<N: ToUint>() -> (u128, bool) {
    to_u_ovfl!(N::ToUint, u128)
}
pub const fn to_u128<N: ToUint>() -> Option<u128> {
    match to_u128_overflowing::<N>() {
        (n, false) => Some(n),
        (_, true) => None,
    }
}
pub const fn cmp<L: ToUint, R: ToUint>() -> core::cmp::Ordering {
    use core::cmp::Ordering;
    const {
        match cmp::<ops::Half<L>, ops::Half<R>>() {
            it @ (Ordering::Less | Ordering::Greater) => it,
            Ordering::Equal => match (to_bool::<ops::Parity<L>>(), to_bool::<ops::Parity<R>>()) {
                (true, true) | (false, false) => Ordering::Equal,
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
            },
        }
    }
}

pub trait TCon {
    type Apply<N: Uint>;
}
pub const fn transform_eq<C: TCon, Src: Uint, Dst: Uint>(
    src: C::Apply<Src>,
) -> Result<C::Apply<Dst>, C::Apply<Src>> {
    if cmp::<Src, Dst>().is_eq() {
        Ok(unsafe { crate::utils::exact_transmute(src) })
    } else {
        Err(src)
    }
}
