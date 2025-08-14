use super::*;
// Notation: Cond!(C) := if C { 1 } else { 0 }
//           Cond!(C) = Cond!(D)  iff  (C iff D)
//           Cond!(C) = 1  iff  C

#[apply(lazy)]
// HL := H(L), PL := P(L), HR := H(R), PR := P(R)
//
// Eq(L, R) := Cond!(L == R)
pub type EqL<L, R> = Tern<
    L,
    //     L = R
    // iff 2 * HL + PL = 2 * HR + PR
    // iff PL = PR  and  HL = HR
    // iff Xnor(PL, PR) = 1  and  Eq(HL, HR) = 1
    // iff And(Xnor(PL, PR), Eq(HL, HR)) = 1
    AndL<Xnor<P<L>, P<R>>, EqL<H<R>, H<L>>>,
    // case L = 0:  0 = R  iff  (if R { 0 } else { 1 }) = 1
    TernL<R, U0, U1>,
>;

#[apply(opaque)]
pub type Eq<L, R> = EqL<L, R>;
#[apply(opaque)]
pub type Ne<L, R> = BitNot<Eq<L, R>>;

test_op! { test_eq: L R, Eq<L, R>, (L == R) as _ }

#[apply(lazy)]
// HL := H(L), PL := P(L), HR := H(R), PR := P(R)
//
// LtByLast(L, R) := Cond!(HL = HR and PL = 0 and PR = 1)
pub type LtByLastL<L, R> = AndSC<
    Tern<P<L>, U0, P<R>>, // Cond!(not PL and PR)
    EqL<H<L>, H<R>>,      // Cond!(HL = HR)
>;

#[apply(lazy)]
// HL := H(L), PL := P(L), HR := H(R), PR := P(R)
//
// Lt(L, R) := Cond!(L < R)
pub type LtL<L, R> = AndSC<
    R,
    TernL<
        L,
        //     L < R
        // iff 2 * HL + PL < 2 * HR + PR
        // iff HL < HR or HL = HR and PL = 0 and PR = 1
        // iff Lt(HL, HR) = 1 or LtByLastL(L, R) = 1
        TernL<LtL<H<L>, H<R>>, U1, LtByLastL<L, R>>,
        // 0 < R  because R = 0 is handled by the initial And
        U1,
    >,
>;

#[apply(opaque)]
pub type Lt<L, R> = LtL<L, R>;
pub type Gt<L, R> = Lt<R, L>;

test_op! { test_cmp: L R, Lt<L, R>, (L < R) as _ }

#[apply(opaque)]
pub type Ge<L, R> = BitNot<Lt<L, R>>;
pub type Le<L, R> = Ge<L, R>;

#[apply(opaque)]
pub type Min<L, R> = Tern<Lt<L, R>, R, L>;

#[apply(opaque)]
pub type Max<L, R> = Tern<Lt<L, R>, L, R>;
