use crate::{ToUint, Uint, consts, maxint::UMaxInt, ops};

pub type From<N> = <N as ToUint>::ToUint;
pub type FromUsize<const N: usize> = From<crate::consts::ConstUsize<N>>;
pub type FromU128<const N: u128> = From<crate::consts::ConstU128<N>>;

const fn to_u_max_int_overflowing<N: Uint>() -> (UMaxInt, bool) {
    const {
        if to_bool::<N>() {
            let (h, o1) = to_u_max_int_overflowing::<ops::Half<N>>();
            let (t, o2) = h.overflowing_mul(2);
            let (n, o3) = t.overflowing_add(to_bool::<ops::Parity<N>>() as _);
            (n, o1 || o2 || o3)
        } else {
            (0, false)
        }
    }
}

pub const fn to_bool<N: ToUint>() -> bool {
    crate::internals::PrimitiveOp!(N::ToUint, ::IS_NONZERO)
}
pub const fn to_str<N: ToUint>() -> &'static str {
    const fn to_byte_str_naive<N: Uint>() -> &'static [u8] {
        struct ConcatBytes<N>(N);
        impl<N: Uint> type_const::Const for ConcatBytes<N> {
            type Type = &'static [&'static [u8]];
            const VALUE: Self::Type = &[
                doit::<ops::Div<N, consts::U10>>(),
                &[b'0' + to_usize::<ops::Rem<N, consts::U10>>().unwrap() as u8],
            ];
        }
        const fn doit<N: Uint>() -> &'static [u8] {
            const {
                if to_bool::<N>() {
                    const_util::concat::concat_bytes::<ConcatBytes<N>>()
                } else {
                    b""
                }
            }
        }
        match doit::<N>() {
            b"" => b"0",
            val => val,
        }
    }

    const fn doit<N: Uint>() -> &'static str {
        const {
            let fast_eval = const {
                const MAXLEN: usize = crate::maxint::u_max_int_strlen(UMaxInt::MAX);

                if let (n, false) = to_u_max_int_overflowing::<N>() {
                    let len = crate::maxint::u_max_int_strlen(n);
                    let mut out = [0; MAXLEN];
                    crate::maxint::u_max_int_write(n, &mut out);
                    Some((&{ out }, len))
                } else {
                    None
                }
            };
            let byte_str = match fast_eval {
                Some((out, len)) => out.split_at(len).0,
                None => to_byte_str_naive::<N>(),
            };
            match core::str::from_utf8(byte_str) {
                Ok(s) => s,
                Err(_) => unreachable!(),
            }
        }
    }

    doit::<N::ToUint>()
}

pub const fn to_usize_overflowing<N: ToUint>() -> (usize, bool) {
    let (n, o1) = to_u_max_int_overflowing::<N::ToUint>();
    (n as _, o1 || n > usize::MAX as UMaxInt)
}
pub const fn to_usize<N: ToUint>() -> Option<usize> {
    match to_usize_overflowing::<N>() {
        (n, false) => Some(n),
        (_, true) => None,
    }
}
pub const fn to_u128_overflowing<N: ToUint>() -> (u128, bool) {
    let (n, o1) = to_u_max_int_overflowing::<N::ToUint>();
    (n as _, o1 || n > u128::MAX as UMaxInt)
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
        if !to_bool::<L>() {
            match to_bool::<R>() {
                true => Ordering::Less,
                false => Ordering::Equal,
            }
        } else {
            match cmp::<ops::Half<L>, ops::Half<R>>() {
                it @ (Ordering::Less | Ordering::Greater) => it,
                Ordering::Equal => match (to_bool::<ops::Parity<L>>(), to_bool::<ops::Parity<R>>())
                {
                    (true, true) | (false, false) => Ordering::Equal,
                    (true, false) => Ordering::Greater,
                    (false, true) => Ordering::Less,
                },
            }
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
        // SAFETY: Src == Dst, uniqueness of `Uint`s and type projection
        // imply that this is a transmute from a type to itself
        Ok(unsafe { crate::utils::exact_transmute(src) })
    } else {
        Err(src)
    }
}
