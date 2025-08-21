#![allow(clippy::absurd_extreme_comparisons)]

pub(crate) trait SelectMaxInt<const B: bool> {
    type Output;
}
impl SelectMaxInt<false> for () {
    type Output = u128;
}
impl SelectMaxInt<true> for () {
    type Output = usize;
}
const IS_USIZE: bool = size_of::<usize>() > size_of::<u128>();
pub(crate) type UMaxInt = <() as SelectMaxInt<IS_USIZE>>::Output;

pub(crate) const fn u_max_int_strlen(n: UMaxInt) -> usize {
    if n == 0 { 1 } else { n.ilog10() as usize + 1 }
}
pub(crate) const fn u_max_int_write(n: UMaxInt, out: &mut [u8]) -> &mut [u8] {
    let (mut n_out, out) = out.split_at_mut(u_max_int_strlen(n));
    let mut r = n;
    while let [rem @ .., last] = n_out {
        n_out = rem;
        *last = b'0' + (r % 10) as u8;
        r /= 10;
        if r == 0 {
            break;
        }
    }
    debug_assert!(r == 0);
    out
}
