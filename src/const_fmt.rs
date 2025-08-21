use core::marker::PhantomData;

/// ```rust_analyzer_infer_bracket
/// concat_fmt![]
/// ```
macro_rules! concat_fmt {
    [ $($ex:expr),* $(,)? ] => {
        $crate::const_fmt::ConstFmtEnter([ $( $crate::const_fmt::ConstFmtEnter($ex).enter(), )* ])
    };
}
pub(crate) use concat_fmt;

/// ```rust_analyzer_infer_bracket
/// concat_fmt_if![]
/// ```
macro_rules! concat_fmt_if {
    [ $cond:expr $(, $ex:expr)* $(,)? ] => {
        if $cond {
            Some($crate::const_fmt::concat_fmt![$($ex),*])
        } else {
            None
        }
    };
}
pub(crate) use concat_fmt_if;

use crate::maxint;

pub(crate) struct ConstFmtEnter<T>(pub T);
impl<N: crate::ToUint> ConstFmtEnter<PhantomData<N>> {
    pub(crate) const fn enter(self) -> ConstFmt<'static> {
        ConstFmt::Str(crate::uint::to_str::<N::ToUint>())
    }
}
impl ConstFmtEnter<usize> {
    pub(crate) const fn enter(self) -> ConstFmt<'static> {
        ConstFmt::PrimUint(self.0 as _)
    }
}
#[expect(dead_code)] // not used atm
impl ConstFmtEnter<u128> {
    pub(crate) const fn enter(self) -> ConstFmt<'static> {
        ConstFmt::PrimUint(self.0 as _)
    }
}
impl<'a> ConstFmtEnter<&'a str> {
    pub(crate) const fn enter(self) -> ConstFmt<'a> {
        ConstFmt::Str(self.0)
    }
}
#[expect(dead_code)] // not used atm
impl<'a> ConstFmtEnter<&'a [ConstFmt<'a>]> {
    pub(crate) const fn enter(self) -> ConstFmt<'a> {
        ConstFmt::Concat(self.0)
    }
}
impl<'a, const N: usize> ConstFmtEnter<[ConstFmt<'a>; N]> {
    pub(crate) const fn enter(&self) -> ConstFmt<'_> {
        ConstFmt::Concat(&self.0)
    }

    #[cold]
    pub(crate) const fn panic(self) -> ! {
        const_fmt_panic(self.enter())
    }
}

#[derive(Clone, Copy)]
pub(crate) enum ConstFmt<'a> {
    Concat(&'a [ConstFmt<'a>]),
    PrimUint(crate::maxint::UMaxInt),
    Str(&'a str),
}

impl ConstFmt<'_> {
    const fn write(self, mut out: &mut [u8]) -> (&mut [u8], bool) {
        match self {
            Self::Concat(mut parts) => {
                while let [first, rem @ ..] = parts {
                    parts = rem;
                    out = match first.write(out) {
                        (it, false) => it,
                        t @ (_, true) => return t,
                    };
                }
            }
            ConstFmt::PrimUint(n) => {
                if out.len() < maxint::u_max_int_strlen(n) {
                    return (out, true);
                }
                out = maxint::u_max_int_write(n, out);
            }
            ConstFmt::Str(s) => {
                let s = s.as_bytes();
                if out.len() < s.len() {
                    let (_, s) = s.split_at(out.len());
                    out.copy_from_slice(s);
                    return (out.split_at_mut(out.len()).1, true);
                }
                let s_out;
                (s_out, out) = out.split_at_mut(s.len());
                s_out.copy_from_slice(s);
            }
        }
        (out, false)
    }
}

#[cold]
#[track_caller]
pub(crate) const fn const_fmt_panic(msg: ConstFmt) -> ! {
    // Limit message size to 10KiB
    const MSG_SIZE: usize = 1 << 10;

    let mut out = [0; MSG_SIZE];
    let out = out.as_mut_slice();

    let (rest, ran_out) = msg.write(out);
    let rest = rest.len();

    let out = if ran_out {
        const ELLISPIS: &[u8] = b"...";
        let (out, _) = out.split_at_mut(MSG_SIZE - rest.saturating_sub(ELLISPIS.len()));
        let (_, dots) = out.split_at_mut(out.len() - ELLISPIS.len());
        dots.copy_from_slice(ELLISPIS);
        out
    } else {
        let (out, _) = out.split_at_mut(out.len() - rest);
        out
    };
    panic!(
        "{}",
        match core::str::from_utf8(out) {
            Ok(out) => out,
            Err(_) => unreachable!(),
        }
    )
}
