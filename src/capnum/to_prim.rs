use crate::{
    Uint,
    capnum::{CapUint, Digit, capnum_utils::*},
    maxint::UMaxInt,
};

const fn to_umaxint_overflowing(mut digits: &[Digit]) -> (UMaxInt, bool) {
    let mut buf = [0u8; size_of::<UMaxInt>().next_multiple_of(size_of::<Digit>())];
    let mut out = buf.as_mut_slice();
    while let Some((first_out, rem_out)) = out.split_at_mut_checked(size_of::<Digit>()) {
        out = rem_out;

        let last_digit;
        (digits, last_digit) = pop_or_zero(digits);
        first_out.copy_from_slice(&last_digit.to_le_bytes());
    }
    let (actual, mut leftover) = buf.split_at(size_of::<UMaxInt>());

    let overflow = 'check_leftover: {
        while let [rem @ .., last] = leftover {
            leftover = rem;
            if *last != 0 {
                break 'check_leftover true;
            }
        }
        false
    } || !is_zero(digits);

    let wrapped = UMaxInt::from_le_bytes(match crate::array::ArrApi::try_from_slice(actual) {
        Ok(ok) => ok.inner,
        Err(_) => unreachable!(),
    });

    (wrapped, overflow)
}

macro_rules! generate_methods {
    ($($normal:ident $overflowing:ident $prim:ty,)*) => {$(
        #[doc = concat!("Converts the number to a ", stringify!($prim), ", wrapping around if necessary and returning whether any wrapping occurred.")]
        pub const fn $overflowing(self) -> ($prim, bool) {
            let (u, o) = to_umaxint_overflowing(self.as_digits());
            (u as _, u > <$prim>::MAX as UMaxInt || o)
        }

        #[doc = concat!("Converts the number to a ", stringify!($prim), ", returning [`None`] if it doesn't fit.")]
        pub const fn $normal(self) -> Option<$prim> {
            match self.$overflowing() {
                (n, false) => Some(n),
                (_, true) => None,
            }
        }
    )*};
}

impl<N: Uint> CapUint<N> {
    generate_methods! {
        to_usize to_usize_overflowing usize,
        to_u8 to_u8_overflowing u8,
        to_u16 to_u16_overflowing u16,
        to_u32 to_u32_overflowing u32,
        to_u64 to_u64_overflowing u64,
        to_u128 to_u128_overflowing u128,
    }
}

#[test]
fn try_some_idk() {
    use crate::{consts, lit, uint};

    fn doit<N: Uint>() {
        assert_eq!(
            CapUint::<N>::MAX.to_usize_overflowing(),
            uint::to_usize_overflowing::<N>()
        )
    }

    doit::<consts::UsizeMax>();
    doit::<lit!(0x123451234512345123451234512345)>();
    doit::<lit!(123451234512345123451234512345)>();
}
