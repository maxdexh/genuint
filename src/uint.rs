//! Utilities related to [`ToUint`] implementors.

use crate::{
    ToUint, Uint,
    consts::{self, ConstU128, ConstUsize},
    maxint::Umax,
    ops, uint,
};

/// Alias for [`ToUint::ToUint`].
pub type From<N> = <N as ToUint>::ToUint;
/// Composition of [`From`] with [`ConstUsize`].
pub type FromUsize<const N: usize> = From<ConstUsize<N>>;
/// Composition of [`From`] with [`ConstU128`].
pub type FromU128<const N: u128> = From<ConstU128<N>>;

const fn to_umax_overflowing<N: Uint>() -> (Umax, bool) {
    // NOTE: This does unnecessary work. We only need to look at the last UMaxInt::BITS.
    // It might be worth it to perform a BitAnd on the `Uint` before plugging it in here.
    const {
        if to_bool::<N>() {
            let (h, o1) = to_umax_overflowing::<uint::From<ops::Half<N>>>();
            let (t, o2) = h.overflowing_mul(2);
            let (n, o3) = t.overflowing_add(to_bool::<ops::Parity<N>>() as _);
            (n, o1 || o2 || o3)
        } else {
            (0, false)
        }
    }
}

/// Returns whether `N::ToUint` is nonzero.
pub const fn to_bool<N: ToUint>() -> bool {
    crate::internals::InternalOp!(N::ToUint, ::IS_NONZERO)
}
/// Returns the decimal representation of `N::ToUint` for arbitrarily large `N`.
pub const fn to_str<N: ToUint>() -> &'static str {
    const fn to_byte_str_naive<N: Uint>() -> &'static [u8] {
        struct ConcatBytes<N>(N);
        impl<N: Uint> type_const::Const for ConcatBytes<N> {
            type Type = &'static [&'static [u8]];
            const VALUE: Self::Type = &[
                // Recursively append the last digit
                doit::<
                    uint::From<
                        // Pop a digit
                        ops::Div<N, consts::_10>,
                    >,
                >(),
                &[b'0' + to_usize::<ops::Rem<N, consts::_10>>().unwrap() as u8],
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

    // try to stringify the primitive representation if there is any
    const fn shortcut_umax<N: Uint>() -> &'static str {
        const {
            let fast_eval = const {
                const MAXLEN: usize = crate::maxint::umax_strlen(Umax::MAX);

                if let (n, false) = to_umax_overflowing::<N>() {
                    let len = crate::maxint::umax_strlen(n);
                    let mut out = [0; MAXLEN];
                    crate::maxint::umax_write(n, &mut out);
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

    shortcut_umax::<N::ToUint>()
}

/// Converts `N::ToUint` to a `usize` with overflow and reutrns whether any wrapping
/// occurred.
pub const fn to_usize_overflowing<N: ToUint>() -> (usize, bool) {
    let (n, o1) = to_umax_overflowing::<N::ToUint>();
    (n as _, o1 || n > usize::MAX as Umax)
}

/// Converts `N::ToUint` to a `usize` or returns `None` if it doesn't fit.
pub const fn to_usize<N: ToUint>() -> Option<usize> {
    match to_usize_overflowing::<N>() {
        (n, false) => Some(n),
        (_, true) => None,
    }
}

/// Converts `N::ToUint` to a `u128` with overflow and reutrns whether any wrapping
/// occurred.
pub const fn to_u128_overflowing<N: ToUint>() -> (u128, bool) {
    let (n, o1) = to_umax_overflowing::<N::ToUint>();
    (n as _, o1 || n > u128::MAX as Umax)
}

/// Converts `N::ToUint` to a `u128` or returns `None` if it doesn't fit.
pub const fn to_u128<N: ToUint>() -> Option<u128> {
    match to_u128_overflowing::<N>() {
        (n, false) => Some(n),
        (_, true) => None,
    }
}

/// Compares `L::ToUint` and `R::Uint`.
///
/// If this function returns [`Equal`](core::cmp::Ordering::Equal), it is guaranteed that
/// `L::ToUint` and `R::ToUint` are exactly the same type.
pub const fn cmp<L: ToUint, R: ToUint>() -> core::cmp::Ordering {
    use core::cmp::Ordering;
    const fn doit<L: Uint, R: Uint>() -> Ordering {
        const {
            if !to_bool::<L>() {
                match to_bool::<R>() {
                    true => Ordering::Less,
                    false => Ordering::Equal,
                }
            } else {
                match doit::<From<ops::Half<L>>, From<ops::Half<R>>>() {
                    it @ (Ordering::Less | Ordering::Greater) => it,
                    Ordering::Equal => {
                        match (to_bool::<ops::Parity<L>>(), to_bool::<ops::Parity<R>>()) {
                            (true, true) | (false, false) => Ordering::Equal,
                            (true, false) => Ordering::Greater,
                            (false, true) => Ordering::Less,
                        }
                    }
                }
            }
        }
    }
    doit::<L::ToUint, R::ToUint>()
}
