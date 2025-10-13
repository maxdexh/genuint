//! Utilities related to [`ToUint`] implementors.

use core::cmp::Ordering;

use crate::{ToUint, Uint, maxint::Umax, small, uint, uops};

/// Alias for [`ToUint::ToUint`].
pub type From<N> = <N as ToUint>::ToUint;

/// Turns an integer literal into a [`Uint`].
///
/// If you have a small constant value that is not a literal, use [`uint::FromU128`].
///
/// # Examples
/// ```
/// #![recursion_limit = "1024"] // `lit!` doesn't recurse, the type is just long
///
/// use genuint::uint;
/// assert_eq!(uint::to_u128::<uint::lit!(1)>(), Some(1));
/// assert_eq!(
///     uint::to_u128::<uint::lit!(100000000000000000000000000000)>(),
///     Some(100000000000000000000000000000),
/// )
/// ```
#[macro_export]
#[doc(hidden)]
macro_rules! __lit {
    ($l:literal) => {
        $crate::uint::From<$crate::__mac::proc::__lit!(
            ($l)
            ($crate::uops::PushBit)
            ($crate::small::_0)
            ($crate::small::_1)
        )>
    };
}
pub use __lit as lit;

const fn to_umax_overflowing<N: Uint>() -> (Umax, bool) {
    const {
        if is_nonzero::<N>() {
            let (h, o1) = to_umax_overflowing::<uint::From<uops::PopBit<N>>>();
            let (t, o2) = h.overflowing_mul(2);
            let (n, o3) = t.overflowing_add(is_nonzero::<uops::LastBit<N>>() as _);
            (n, o1 || o2 || o3)
        } else {
            (0, false)
        }
    }
}
const fn to_umax<N: Uint>() -> Option<Umax> {
    match to_umax_overflowing::<N>() {
        (n, false) => Some(n),
        (_, true) => None,
    }
}

/// Returns whether a [`Uint`] is nonzero.
pub const fn is_nonzero<N: ToUint>() -> bool {
    crate::internals::InternalOp!(N::ToUint, IS_NONZERO)
}
/// Returns whether a [`Uint`] is zero.
pub const fn is_zero<N: ToUint>() -> bool {
    !is_nonzero::<N>()
}

/// Returns the decimal representation of a [`Uint`] for arbitrarily large `N`.
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
                        uops::Div<N, small::_10>,
                    >,
                >(),
                &[b'0' + to_usize::<uops::Rem<N, small::_10>>().unwrap() as u8],
            ];
        }
        const fn doit<N: Uint>() -> &'static [u8] {
            const {
                if is_nonzero::<N>() {
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

                if let Some(n) = to_umax::<N>() {
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
pub const fn cmp<L: ToUint, R: ToUint>() -> Ordering {
    const fn doit<L: Uint, R: Uint>() -> Ordering {
        const {
            if !is_nonzero::<L>() {
                match is_nonzero::<R>() {
                    true => Ordering::Less,
                    false => Ordering::Equal,
                }
            } else {
                match doit::<From<uops::PopBit<L>>, From<uops::PopBit<R>>>() {
                    it @ (Ordering::Less | Ordering::Greater) => it,
                    Ordering::Equal => {
                        match (
                            is_nonzero::<uops::LastBit<L>>(),
                            is_nonzero::<uops::LastBit<R>>(),
                        ) {
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

const fn cmp_umax<Lhs: Uint>(rhs: Umax) -> Ordering {
    if let Some(lhs) = to_umax::<Lhs>() {
        if lhs < rhs {
            Ordering::Less
        } else if lhs == rhs {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    } else {
        Ordering::Greater
    }
}

/// Compares a [`Uint`] (lhs) to a [`u128`] (rhs).
pub const fn cmp_u128<Lhs: ToUint>(rhs: u128) -> Ordering {
    cmp_umax::<Lhs::ToUint>(rhs as _)
}

/// Compares a [`Uint`] (lhs) to a [`usize`] (rhs).
pub const fn cmp_usize<Lhs: ToUint>(rhs: usize) -> Ordering {
    cmp_umax::<Lhs::ToUint>(rhs as _)
}
