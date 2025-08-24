use crate::capnum::Digit;

pub(crate) const fn pop_or_zero(n: &[Digit]) -> (&[Digit], Digit) {
    match n {
        [rest @ .., last] => (rest, *last),
        [] => (n, 0),
    }
}
pub(crate) const fn pop_first_or_zero(n: &[Digit]) -> (Digit, &[Digit]) {
    match n {
        [first, rest @ ..] => (*first, rest),
        [] => (0, n),
    }
}
pub(crate) const fn add(mut lhs: &[Digit], mut rhs: &[Digit], mut out: &mut [Digit]) {
    let mut carry = false;
    while !lhs.is_empty() || !rhs.is_empty() || carry {
        let [rest_out @ .., last_out] = out else {
            unreachable!()
        };
        out = rest_out;

        let (last_lhs, last_rhs);
        (lhs, last_lhs) = pop_or_zero(lhs);
        (rhs, last_rhs) = pop_or_zero(rhs);

        let (sum, o1) = last_lhs.overflowing_add(last_rhs);
        let (sum, o2) = sum.overflowing_add(carry as _);
        *last_out = sum;
        carry = o1 || o2;
    }
}
pub(crate) const fn cmp_same_len(mut lhs: &[Digit], mut rhs: &[Digit]) -> core::cmp::Ordering {
    use core::cmp::Ordering;

    while !lhs.is_empty() || !rhs.is_empty() {
        let (long_first, short_first);
        (long_first, lhs) = pop_first_or_zero(lhs);
        (short_first, rhs) = pop_first_or_zero(rhs);

        if long_first > short_first {
            return Ordering::Greater;
        } else if long_first < short_first {
            return Ordering::Less;
        }
    }

    Ordering::Equal
}
pub(crate) const fn is_zero(mut digits: &[Digit]) -> bool {
    while let [rem @ .., pop] = digits {
        digits = rem;
        if *pop != 0 {
            return false;
        }
    }
    true
}
pub(crate) const fn cmp(lhs: &[Digit], rhs: &[Digit]) -> core::cmp::Ordering {
    use core::cmp::Ordering;

    const fn has_extra(long: &[Digit], short: &[Digit]) -> bool {
        debug_assert!(long.len() >= short.len());
        let (_, long_extra) = long.split_at(long.len() - short.len());
        !is_zero(long_extra)
    }

    #[allow(clippy::collapsible_if)]
    if lhs.len() < rhs.len() {
        if has_extra(rhs, lhs) {
            return Ordering::Less;
        }
    } else if lhs.len() > rhs.len() {
        if has_extra(lhs, rhs) {
            return Ordering::Greater;
        }
    }

    cmp_same_len(lhs, rhs)
}
