use crate::{Uint, array::CopyArr, ops};

mod capnum_utils;
use capnum_utils::*;

type Digit = usize;

mod arrlen {
    use super::Digit;
    use crate::{Uint, consts::*, ops, uint};
    use generic_uint_proc::apply;

    type DigitBitLen = ops::Shl<uint::FromUsize<{ size_of::<Digit>() }>, U3>;

    #[apply(ops::lazy)]
    pub type ArrLenL<N> = ops::Tern<
        N,
        ops::Inc<
            ArrLenL<
                ops::Shr<
                    //
                    N,
                    DigitBitLen,
                >,
            >,
        >,
        U0,
    >;

    pub trait ArrLenOf: Uint {
        type Value: Uint;
    }
    impl<N: Uint> ArrLenOf for N {
        type Value = uint::From<ArrLenL<N>>;
    }
    pub type ArrLen<N> = <N as ArrLenOf>::Value;

    #[test]
    fn test_arr_len() {
        fn arr_len<N: Uint>() -> usize {
            uint::to_usize::<ArrLen<N>>().unwrap()
        }
        assert_eq!(uint::to_usize::<DigitBitLen>(), Some(Digit::BITS as usize));

        assert_eq!(arr_len::<U0>(), 0);
        assert_eq!(arr_len::<U1>(), 1);
        type DigitMaxInc = ops::Shl<U1, DigitBitLen>;
        assert_eq!(arr_len::<ops::SatSub<DigitMaxInc, U1>>(), 1);
        assert_eq!(arr_len::<DigitMaxInc>(), 2);
    }
}

type DigitArrBase<N> = CopyArr<Digit, N>;
type DigitArr<N> = DigitArrBase<arrlen::ArrLen<N>>;

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

impl<N: Uint> CapUint<N> {
    const MAX: Self = Self::try_of_uint::<N>().unwrap();

    const fn from_digits(digits: DigitArr<N>) -> Self {
        assert!(cmp_same_len(digits.as_slice(), Self::MAX.digits.as_slice()).is_le());
        Self { digits }
    }
    const fn as_digits(&self) -> &[Digit] {
        self.digits.as_slice()
    }
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
}

impl<N: Uint> CapUint<N> {
    pub const fn try_of_uint<M: Uint>() -> Option<Self> {
        todo!()
    }
    pub const fn add_const<M: Uint>(self, rhs: CapUint<M>) -> CapUint<ops::Add<N, M>> {
        let mut out = DigitArrBase::of(0);
        add(self.as_digits(), rhs.as_digits(), out.as_mut_slice());
        CapUint::from_digits(out)
    }
    pub const fn cmp_const<M: Uint>(self, rhs: CapUint<M>) -> core::cmp::Ordering {
        cmp(self.as_digits(), rhs.as_digits())
    }
    pub const fn min_const<M: Uint>(self, rhs: CapUint<M>) -> CapUint<ops::Min<N, M>> {
        match self.cmp_const(rhs).is_lt() {
            true => self.try_resize(),
            false => rhs.try_resize(),
        }
        .unwrap()
    }
}
