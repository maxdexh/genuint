//! implementation detail for some error messages

use crate::{
    array::{ArrApi, Array},
    maxint,
    utils::subslice,
};
use core::{marker::PhantomData, mem::ManuallyDrop};

/// ```rust_analyzer_prefer_brackets
/// fmt![]
/// ```
macro_rules! fmt {
    [ $($ex:expr),* $(,)? ] => {
        crate::const_fmt::ConstFmtWrap::new(
            crate::array::convert::retype::<_, crate::array::Arr<_, _>>([])
                $( .concat(crate::const_fmt::ConstFmtWrap::new($ex).fmt()) )*
        )
    };
}
pub(crate) use fmt;

/// ```rust_analyzer_prefer_brackets
/// panic_fmt![]
/// ```
macro_rules! panic_fmt {
    [ $($ex:expr),* $(,)? ] => {
        crate::const_fmt::fmt![$($ex),*].panic()
    };
}
pub(crate) use panic_fmt;

#[derive(Clone, Copy)]
pub(crate) enum ConstFmt<'a> {
    PrimUint(maxint::Umax),
    Str(&'a str),
    Concat(&'a [ConstFmt<'a>]),
}
impl<'a> ConstFmt<'a> {
    const fn write(self, mut out: &mut [u8]) -> Option<&mut [u8]> {
        match self {
            Self::PrimUint(mut n) => {
                if out.is_empty() {
                    return None;
                }
                let mut ran_out = false;
                while out.len() < maxint::umax_strlen(n) {
                    ran_out = true;
                    n /= 10;
                }
                out = maxint::umax_write(n, out);
                if ran_out {
                    debug_assert!(out.is_empty());
                    return None;
                }
            }
            Self::Str(s) => {
                let s = s.as_bytes();
                if out.len() < s.len() {
                    out.copy_from_slice(subslice![&s, _, out.len()]);
                    return None;
                }
                let s_out;
                (s_out, out) = out.split_at_mut(s.len());
                s_out.copy_from_slice(s);
            }
            Self::Concat(mut parts) => {
                while let [first, rem_parts @ ..] = parts {
                    parts = rem_parts;
                    if let Some(rem_out) = first.write(out) {
                        out = rem_out;
                    } else {
                        return None;
                    }
                }
            }
        }

        Some(out)
    }
}

#[repr(transparent)]
pub(crate) struct ConstFmtWrap<T>(ManuallyDrop<T>);
impl<T> ConstFmtWrap<T> {
    pub(crate) const fn new(t: T) -> Self {
        Self(ManuallyDrop::new(t))
    }
    pub(crate) const fn into_inner(self) -> T {
        ManuallyDrop::into_inner(self.0)
    }
}

impl<'a, A: Array<Item = ConstFmt<'a>>> ConstFmtWrap<A> {
    #[allow(dead_code)]
    pub(crate) const fn fmt(self) -> A {
        self.into_inner()
    }

    pub(crate) const fn panic(self) -> ! {
        do_panic_fmt(ConstFmt::Concat(ArrApi::new(self.fmt()).as_slice()))
    }
}
impl ConstFmtWrap<usize> {
    #[allow(dead_code)]
    pub(crate) const fn fmt(self) -> [ConstFmt<'static>; 1] {
        [ConstFmt::PrimUint(self.into_inner() as _)]
    }
}
impl<'a> ConstFmtWrap<&'a str> {
    #[allow(dead_code)]
    pub(crate) const fn fmt(self) -> [ConstFmt<'a>; 1] {
        [ConstFmt::Str(self.into_inner())]
    }
}
impl<'a> ConstFmtWrap<ConstFmt<'a>> {
    #[allow(dead_code)]
    pub(crate) const fn fmt(self) -> [ConstFmt<'a>; 1] {
        [self.into_inner()]
    }
}
impl<'a> ConstFmtWrap<&'a [ConstFmt<'a>]> {
    #[allow(dead_code)]
    pub(crate) const fn fmt(self) -> [ConstFmt<'a>; 1] {
        [ConstFmt::Concat(self.into_inner())]
    }
}
impl<'a, const N: usize> ConstFmtWrap<&'a [ConstFmt<'a>; N]> {
    #[allow(dead_code)]
    pub(crate) const fn fmt(self) -> [ConstFmt<'a>; 1] {
        [ConstFmt::Concat(self.into_inner())]
    }
}
impl<'a, A: Array<Item = ConstFmt<'a>>> ConstFmtWrap<ConstFmtWrap<A>> {
    #[allow(dead_code)]
    pub(crate) const fn fmt(self) -> A {
        self.into_inner().into_inner()
    }
}
impl<N: crate::ToUint> ConstFmtWrap<PhantomData<N>> {
    #[allow(dead_code)]
    pub(crate) const fn fmt(self) -> [ConstFmt<'static>; 1] {
        ConstFmtWrap::new(crate::uint::to_str::<N::ToUint>()).fmt()
    }
}

pub(crate) const fn do_panic_fmt(fmt: ConstFmt) -> ! {
    // Limit message size to 10KiB
    const MSG_SIZE: usize = 1 << 10;

    let mut buf = [0; MSG_SIZE];

    let result = if let Some(rest) = fmt.write(&mut buf) {
        let rest = rest.len();
        buf.split_at(buf.len() - rest).0
    } else {
        const ELLISPIS: &[u8] = b"...";
        buf.split_at_mut(MSG_SIZE - ELLISPIS.len())
            .1
            .copy_from_slice(ELLISPIS);
        buf.as_slice()
    };
    panic!(
        "{}",
        match core::str::from_utf8(result) {
            Ok(out) => out,
            Err(_) => unreachable!(),
        }
    )
}
