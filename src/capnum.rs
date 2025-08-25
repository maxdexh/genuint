use crate::{Uint, array::CopyArr, capnum::digits::PopLastDigit, ops, uint};

mod capnum_utils;
use capnum_utils::*;

type Digit = usize;

#[cfg_attr(not(test), allow(unused))] // allow items only used in tests
mod digits {
    use super::Digit;
    use crate::Uint;
    use crate::{consts::*, ops, ops::lazy, uint, utils::apply};

    pub use uint::to_usize as to_digit;
    pub use uint::to_usize_overflowing as to_digit_overflowing;

    pub type DigitBits = UsizeBits;
    pub type DigitMax = UsizeMax;
    pub type DigitBase = ops::Add<DigitMax, _1>;

    /// Gets a number without the last digit, i.e. Div<N, DigitBase>.
    /// Since our base is a power of two, this is the same as bit shifting like this.
    pub type PopLastDigit<N> = ops::Shr<N, DigitBits>;

    /// Gets the last Digit of a Uint
    pub const fn to_digit_wrapping<N: Uint>() -> Digit {
        // Since our base is a power of two, this wrapping conversion will leave
        // exactly the last digit, since every other bit represents a multiple of our base
        // and therefore contributes 0 to the wrapping conversion.
        //
        // We use the wrapping conversion here over `to_digit<BitAnd<N, DigitMax>>()`
        // because its current implementation calculates all digits as intermediate values.
        to_digit_overflowing::<N>().0
    }

    #[cfg(test)]
    const _: () = {
        assert!(uint::to_usize::<DigitBits>().unwrap() == Digit::BITS as usize);

        assert!(matches!(to_digit_overflowing::<DigitBase>(), (0, true)));

        let digit_max = to_digit::<DigitMax>().unwrap();
        assert!(digit_max == Digit::MAX);

        // our base is a power of two
        assert!(digit_max.wrapping_add(1) == 0);
        assert!(digit_max.count_zeros() == 0);
    };

    #[apply(lazy)]
    pub type ArrLenL<N> = ops::Tern<
        N,
        ops::Inc<
            ArrLenL<
                ops::Shr<
                    //
                    N,
                    DigitBits,
                >,
            >,
        >,
        _0,
    >;

    pub type DigitArrLen<N> = uint::From<ArrLenL<N>>;

    #[test]
    fn test_arr_len() {
        fn digit_arr_len<N: crate::Uint>() -> usize {
            uint::to_usize::<DigitArrLen<N>>().unwrap()
        }
        assert_eq!(uint::to_usize::<DigitBits>(), Some(Digit::BITS as usize));

        assert_eq!(digit_arr_len::<_0>(), 0);
        assert_eq!(digit_arr_len::<_1>(), 1);
        assert_eq!(digit_arr_len::<DigitMax>(), 1);
        assert_eq!(digit_arr_len::<ops::Add<DigitMax, _1>>(), 2);
    }
}

type DigitArrBase<N> = CopyArr<Digit, N>;
type DigitArr<N> = DigitArrBase<digits::DigitArrLen<N>>;

/// Holds a number that is from `0` to `N` (inclusive).
pub struct CapUint<N: Uint> {
    digits: DigitArr<N>,
}

impl<N: Uint> Clone for CapUint<N> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<N: Uint> Copy for CapUint<N> {}
impl<N: Uint> Default for CapUint<N> {
    fn default() -> Self {
        Self::ZERO
    }
}

impl<N: Uint> CapUint<N> {
    const unsafe fn from_digits_unchecked(digits: DigitArr<N>) -> Self {
        Self { digits }
    }

    /// The zero instance of the uint type.
    pub const ZERO: Self =
        // SAFETY: 0 <= N
        unsafe { Self::from_digits_unchecked(DigitArrBase::of(0)) };

    pub const MAX: Self = {
        // Break the const eval cycle using a const fn
        const fn max<N: Uint>() -> CapUint<N> {
            CapUint::<N>::MAX
        }

        match uint::to_bool::<N>() {
            // SAFETY:
            // By inductively assuming that max::<M>() == M for M < N, we get that
            // this concats the digits of N / DigitBase with N % DigitBase, giving
            // max::<N>() == DigitBase * (N / DigitBase) + N % DigitBase == N.
            true => unsafe {
                let prefix = max::<PopLastDigit<N>>().digits;
                let last = digits::to_digit_wrapping::<N>();
                Self::from_digits_unchecked(prefix.concat([last]).assert_len().into_arr())
            },
            // max::<0>() == 0
            false => Self::ZERO,
        }
    };

    const fn from_digits(digits: DigitArr<N>) -> Self {
        assert!(cmp_same_len(digits.as_slice(), Self::MAX.digits.as_slice()).is_le());
        Self { digits }
    }

    const fn as_digits(&self) -> &[Digit] {
        self.digits.as_slice()
    }
}

impl<N: Uint> CapUint<N> {
    pub const fn try_resize<M: Uint>(self) -> Option<CapUint<M>> {
        let new_len = DigitArr::<M>::length();

        let (truncated, _) = self
            .as_digits()
            .split_at(self.as_digits().len().saturating_sub(new_len));

        if !is_zero(truncated) {
            return None;
        }

        Some(CapUint::from_digits(
            self.digits.resize_with_fill(0).into_arr(),
        ))
    }

    pub const fn resize_overflowing<M: Uint>(self) -> (CapUint<M>, bool) {
        if self.cmp(CapUint::<M>::MAX).is_gt() {
            todo!() // TODO: modulo
        } else {
            (self.try_resize().unwrap(), false)
        }
    }

    pub const fn add<M: Uint>(self, rhs: CapUint<M>) -> CapUint<ops::Add<N, M>> {
        let mut out = DigitArrBase::of(0);
        add(self.as_digits(), rhs.as_digits(), out.as_mut_slice());
        CapUint::from_digits(out)
    }

    pub const fn cmp<M: Uint>(self, rhs: CapUint<M>) -> core::cmp::Ordering {
        cmp(self.as_digits(), rhs.as_digits())
    }

    pub const fn min<M: Uint>(self, rhs: CapUint<M>) -> CapUint<ops::Min<N, M>> {
        match self.cmp(rhs).is_lt() {
            true => self.try_resize(),
            false => rhs.try_resize(),
        }
        .unwrap()
    }
}

mod to_prim;
