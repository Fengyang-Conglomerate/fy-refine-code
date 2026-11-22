//! Tests for numeric signs, zero checks, parity, divisibility, and floating-point domain stratification (classification, non-`NaN` total order, and exact cross-integer comparisons).
#![cfg_attr(feature = "nightly", feature(const_trait_impl))]

use core::cmp::Ordering;
use fy_refine_core::cmp::{SafeEq, SafeOrd};
use fy_refine_core::function::Function;
use fy_refine_core::function::branch::AndThen;
use fy_refine_core::function::constant::{ConstI32, ConstU8};
use fy_refine_core::function::predicate::{
    Closed, ForkIsMultipleOf, IsEven, IsFinite, IsInfinite, IsInteger, IsMultipleOf,
    IsMultipleOfConst, IsNaN, IsNegative, IsNonNegative, IsNonPositive, IsNonZero, IsNotNaN, IsOdd,
    IsPositive, IsZero, Lt, PairIsMultipleOf, Predicate, Range,
};
use fy_refine_core::function::wiring::Id;
use fy_refine_core::subset::{FiniteF32, FiniteF64, NotNaNF32, NotNaNF64, Subset};

#[test]
fn signed_integers_predicate() {
    // Zero check
    assert!(IsZero::call(0i32));
    assert!(!IsZero::call(1i32));
    assert!(!IsZero::call(-1i32));

    // Non-zero check
    assert!(!IsNonZero::call(0i32));
    assert!(IsNonZero::call(10i32));
    assert!(IsNonZero::call(-10i32));

    // Positive (> 0)
    assert!(IsPositive::call(1i32));
    assert!(!IsPositive::call(0i32));
    assert!(!IsPositive::call(-1i32));

    // Negative (< 0)
    assert!(IsNegative::call(-1i32));
    assert!(!IsNegative::call(0i32));
    assert!(!IsNegative::call(1i32));

    // Non-negative (>= 0)
    assert!(IsNonNegative::call(0i32));
    assert!(IsNonNegative::call(100i32));
    assert!(!IsNonNegative::call(-1i32));

    // Non-positive (<= 0)
    assert!(IsNonPositive::call(0i32));
    assert!(IsNonPositive::call(-100i32));
    assert!(!IsNonPositive::call(1i32));
}

#[test]
fn unsigned_integers_predicate() {
    // Zero and non-zero
    assert!(IsZero::call(0u64));
    assert!(!IsZero::call(1u64));
    assert!(IsNonZero::call(42u64));
    assert!(!IsNonZero::call(0u64));

    // Positive
    assert!(IsPositive::call(1u64));
    assert!(!IsPositive::call(0u64));

    // Negative (always false for unsigned)
    assert!(!IsNegative::call(0u64));
    assert!(!IsNegative::call(100u64));

    // Non-negative (always true for unsigned)
    assert!(IsNonNegative::call(0u64));
    assert!(IsNonNegative::call(100u64));

    // Non-positive (equivalent to == 0 for unsigned)
    assert!(IsNonPositive::call(0u64));
    assert!(!IsNonPositive::call(1u64));
}

fn finite_f32(x: f32) -> Result<FiniteF32, f32> {
    match NotNaNF32::try_new(x) {
        Ok(nn) => match FiniteF32::try_new(nn) {
            Ok(fin) => Ok(fin),
            Err(rejected_nn) => Err(NotNaNF32::into_inner(rejected_nn)),
        },
        Err(raw) => Err(raw),
    }
}

fn finite_f64(x: f64) -> Result<FiniteF64, f64> {
    match NotNaNF64::try_new(x) {
        Ok(nn) => match FiniteF64::try_new(nn) {
            Ok(fin) => Ok(fin),
            Err(rejected_nn) => Err(NotNaNF64::into_inner(rejected_nn)),
        },
        Err(raw) => Err(raw),
    }
}

#[test]
fn refined_float_domain_predicates() {
    // Raw f32/f64 only expose NaN binary representation classification; sign and range ordering require narrowing into NotNaN/Finite domains first.
    type NonPosF32 = Subset<IsNonPositive, NotNaNF32>;
    type ZeroF64 = Subset<IsZero, NotNaNF64>;
    type NonZeroFinite = Subset<IsNonZero, FiniteF64>;
    type UnitInterval = Subset<Range<Closed<ConstU8<0>>, Closed<ConstU8<1>>>, FiniteF64>;

    assert!(NonPosF32::try_new(NotNaNF32::try_new(0.0).unwrap()).is_ok());
    assert!(NonPosF32::try_new(NotNaNF32::try_new(-0.0).unwrap()).is_ok());
    assert!(NonPosF32::try_new(NotNaNF32::try_new(-3.25).unwrap()).is_ok());
    assert!(NonPosF32::try_new(NotNaNF32::try_new(f32::NEG_INFINITY).unwrap()).is_ok());
    assert!(NonPosF32::try_new(NotNaNF32::try_new(0.125).unwrap()).is_err());

    assert!(ZeroF64::try_new(NotNaNF64::try_new(0.0).unwrap()).is_ok());
    assert!(ZeroF64::try_new(NotNaNF64::try_new(-0.0).unwrap()).is_ok());
    assert!(ZeroF64::try_new(NotNaNF64::try_new(1e-12).unwrap()).is_err());

    assert!(NonZeroFinite::try_new(finite_f64(1.5).unwrap()).is_ok());
    assert!(NonZeroFinite::try_new(finite_f64(0.0).unwrap()).is_err());
    assert!(NonZeroFinite::try_new(finite_f64(-0.0).unwrap()).is_err());

    assert!(UnitInterval::try_new(finite_f64(0.0).unwrap()).is_ok());
    assert!(UnitInterval::try_new(finite_f64(-0.0).unwrap()).is_ok());
    assert!(UnitInterval::try_new(finite_f64(0.5).unwrap()).is_ok());
    assert!(UnitInterval::try_new(finite_f64(1.0).unwrap()).is_ok());
    assert!(UnitInterval::try_new(finite_f64(-0.01).unwrap()).is_err());
    assert!(UnitInterval::try_new(finite_f64(1.01).unwrap()).is_err());

    let fpos = finite_f64(2.5).unwrap();
    let fneg = finite_f64(-2.5).unwrap();
    assert!(IsPositive::call(&fpos));
    assert!(IsNonNegative::call(&fpos));
    assert!(!IsPositive::call(&fneg));
    assert!(IsNegative::call(&fneg));
    assert!(!IsNonNegative::call(&fneg));
}

#[test]
fn refined_float_safe_ord_and_cross_integer_exact_cmp() {
    let neg_inf = NotNaNF64::try_new(f64::NEG_INFINITY).unwrap();
    let neg_half = finite_f64(-0.5).unwrap();
    let neg_zero = finite_f64(-0.0).unwrap();
    let pos_zero = NotNaNF64::try_new(0.0).unwrap();
    let half_f32 = finite_f32(0.5).unwrap();
    let half_f64 = finite_f64(0.5).unwrap();
    let pos_inf_f32 = NotNaNF32::try_new(f32::INFINITY).unwrap();
    let pos_inf_f64 = NotNaNF64::try_new(f64::INFINITY).unwrap();

    // +0.0 and -0.0 are equal by real-number value, preserving strict total ordering
    assert!(SafeEq::eq(&neg_zero, &pos_zero));
    assert_eq!(SafeOrd::cmp(&neg_zero, &pos_zero), Ordering::Equal);
    assert!(SafeOrd::lt(&neg_inf, &neg_half));
    assert!(SafeOrd::lt(&neg_half, &neg_zero));
    assert!(SafeOrd::lt(&pos_zero, &half_f32));
    assert!(SafeEq::eq(&half_f32, &half_f64));
    assert!(SafeEq::eq(&pos_inf_f32, &pos_inf_f64));
    assert!(SafeOrd::lt(&half_f64, &pos_inf_f32));

    // Nested float refinements preserve SafeOrd
    type PosFinite = Subset<IsPositive, FiniteF64>;
    let p_half = PosFinite::try_new(half_f64).unwrap();
    assert!(SafeEq::eq(&p_half, &half_f32));
    assert!(SafeOrd::gt(&pos_inf_f64, &p_half));

    // Exact cross-type integer comparisons (large integers and fractional boundaries)
    assert!(SafeEq::eq(&neg_zero, &0u8));
    assert!(SafeEq::eq(&0i128, &neg_zero));
    assert!(SafeOrd::gt(&half_f64, &0u8));
    assert!(SafeOrd::lt(&half_f64, &1u8));
    assert!(SafeOrd::gt(&neg_half, &-1i32));
    assert!(SafeOrd::lt(&neg_half, &0i32));
    assert!(Lt::call((&0u8, &half_f64)));

    // Verify 2^53 + 1 is not collapsed onto (1u64 << 53) as f64 by lossy integer-to-float conversion
    let pow53 = (1u64 << 53) as f64;
    let f_pow53 = finite_f64(pow53).unwrap();
    assert!(SafeEq::eq(&f_pow53, &(1u64 << 53)));
    assert!(SafeOrd::lt(&f_pow53, &((1u64 << 53) + 1)));
    assert!(SafeOrd::gt(&((1u64 << 53) + 1), &f_pow53));

    // Extreme integers i128::MIN (-2^127) and u128::MAX
    let neg_pow127 = finite_f64(-(2.0f64.powi(127))).unwrap();
    assert!(SafeEq::eq(&neg_pow127, &i128::MIN));
    assert!(SafeOrd::lt(&neg_inf, &i128::MIN));

    let large_f64 = finite_f64(1e39).unwrap();
    assert!(SafeOrd::gt(&large_f64, &u128::MAX));
    assert!(SafeOrd::gt(&pos_inf_f64, &u128::MAX));
    let pos_pow127 = finite_f64(2.0f64.powi(127)).unwrap();
    assert!(SafeOrd::lt(&pos_pow127, &u128::MAX));
}

#[test]
fn borrowed_and_subset_compatibility() {
    // Verify borrowed calls
    let val = 42i32;
    assert!(IsPositive::call(&val));
    assert!(IsNonZero::call(&val));
    assert!(!IsZero::call(&val));

    // Verify HRTB predicate bound P: for<'a> Predicate<&'a T>
    fn assert_predicate_borrow<P, T>()
    where
        P: for<'a> Predicate<&'a T>,
    {
    }

    assert_predicate_borrow::<IsPositive, i32>();
    assert_predicate_borrow::<IsZero, usize>();
    assert_predicate_borrow::<IsNonZero, u64>();
    assert_predicate_borrow::<IsNegative, i64>();
    assert_predicate_borrow::<IsNonNegative, isize>();
    assert_predicate_borrow::<IsNonPositive, i16>();
    assert_predicate_borrow::<IsNonPositive, NotNaNF32>();
    assert_predicate_borrow::<IsPositive, FiniteF64>();

    // Native i32 already implements SafeValue and directly serves as a valid Subset carrier
    type PositiveInt = Subset<IsPositive, i32>;
    assert_eq!(
        core::mem::size_of::<PositiveInt>(),
        core::mem::size_of::<i32>()
    );
}

#[cfg(feature = "nightly")]
#[test]
fn compile_time_const_predicates() {
    type NonPosF32 = Subset<IsNonPositive, NotNaNF32>;
    type UnitF64 = Subset<Range<Closed<ConstU8<0>>, Closed<ConstU8<1>>>, FiniteF64>;

    const fn try_const_finite_f64(x: f64) -> Option<FiniteF64> {
        match NotNaNF64::try_new(x) {
            Ok(nn) => match FiniteF64::try_new(nn) {
                Ok(fin) => Some(fin),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }

    const ZERO_CHECK: bool = IsZero::call(0i32);
    const POS_CHECK: bool = IsPositive::call(100u32);
    const NEG_CHECK: bool = IsNegative::call(-50i64);
    const NON_NEG_CHECK: bool = IsNonNegative::call(0u8);
    const FINITE_CHECK: bool = match NotNaNF64::try_new(1.5f64) {
        Ok(nn) => IsFinite::call(nn) && !IsInfinite::call(nn),
        Err(_) => false,
    };
    const NAN_CHECK: bool = IsNaN::call(f32::NAN);
    const EVEN_CHECK: bool = IsEven::call(-4i32);
    const MULT_CHECK: bool = IsMultipleOf::call((i8::MIN, -1i8));
    const MULT_ZERO: bool = IsMultipleOf::call((0u8, 0u8));
    const NON_POS_F32_OK: bool = match NotNaNF32::try_new(-1.5) {
        Ok(v) => NonPosF32::try_new(v).is_ok(),
        Err(_) => false,
    };
    const UNIT_F64_OK: bool = match try_const_finite_f64(0.75) {
        Some(v) => UnitF64::try_new(v).is_ok(),
        None => false,
    };
    const EXACT_F64_INT_CMP: bool = match try_const_finite_f64(0.5) {
        Some(v) => SafeOrd::gt(&v, &0u8) && SafeOrd::lt(&v, &1u8),
        None => false,
    };

    const {
        assert!(ZERO_CHECK);
        assert!(POS_CHECK);
        assert!(NEG_CHECK);
        assert!(NON_NEG_CHECK);
        assert!(FINITE_CHECK);
        assert!(NAN_CHECK);
        assert!(EVEN_CHECK);
        assert!(MULT_CHECK);
        assert!(MULT_ZERO);
        assert!(NON_POS_F32_OK);
        assert!(UNIT_F64_OK);
        assert!(EXACT_F64_INT_CMP);
    }
}

#[test]
fn float_classification_and_integer_parity_multiple_predicates() {
    // 1. Floating-point binary stratification: NaN -> NotNaN -> Finite/Infinite
    assert!(IsNaN::call(f32::NAN));
    assert!(IsNaN::call(&f64::NAN));
    assert!(!IsNaN::call(0.0f64));
    assert!(!IsNaN::call(f32::INFINITY));

    assert!(IsNotNaN::call(0.0f64));
    assert!(IsNotNaN::call(&f32::INFINITY));
    assert!(!IsNotNaN::call(f32::NAN));

    let nn_zero = NotNaNF32::try_new(0.0).unwrap();
    let nn_neg = NotNaNF64::try_new(-42.5).unwrap();
    let nn_pos_inf = NotNaNF32::try_new(f32::INFINITY).unwrap();
    let nn_neg_inf = NotNaNF64::try_new(f64::NEG_INFINITY).unwrap();
    let nn_one = NotNaNF32::try_new(1.0).unwrap();

    assert!(IsFinite::call(nn_zero));
    assert!(IsFinite::call(&nn_neg));
    assert!(!IsFinite::call(nn_pos_inf));
    assert!(!IsFinite::call(&nn_neg_inf));

    assert!(IsInfinite::call(nn_pos_inf));
    assert!(IsInfinite::call(&nn_neg_inf));
    assert!(!IsInfinite::call(nn_one));

    assert!(finite_f64(1.5).is_ok());
    assert!(finite_f64(f64::INFINITY).is_err());
    assert!(finite_f64(f64::NAN).is_err());

    // 2. Integer parity
    assert!(IsEven::call(0i32));
    assert!(IsEven::call(&-4i8));
    assert!(IsEven::call(i8::MIN));
    assert!(!IsEven::call(3u64));
    assert!(IsOdd::call(-3i32));
    assert!(IsOdd::call(&u128::MAX));
    assert!(!IsOdd::call(0usize));

    // 3. Total panic-free integer divisibility (no division-by-zero or MIN % -1 panics)
    assert!(IsMultipleOf::call((12i32, 4i32)));
    assert!(IsMultipleOf::call((&-12i32, 4i32)));
    assert!(IsMultipleOf::call((12i32, &-4i32)));
    assert!(!IsMultipleOf::call((&13i32, &4i32)));
    assert!(IsMultipleOf::call((0i32, 0i32)));
    assert!(!IsMultipleOf::call((5i32, 0i32)));
    assert!(!IsMultipleOf::call((-5i32, 0i32)));
    assert!(IsMultipleOf::call((0u64, 0u64)));
    assert!(!IsMultipleOf::call((7u64, 0u64)));
    assert!(IsMultipleOf::call((i8::MIN, -1i8)));
    assert!(IsMultipleOf::call((i32::MIN, -1i32)));
    assert!(IsMultipleOf::call((i128::MIN, -1i128)));
    assert!(IsMultipleOf::call((isize::MIN, -1isize)));

    assert!(PairIsMultipleOf::<Id, Id>::call((15u32, 5u32)));
    assert!(ForkIsMultipleOf::<Id, Id>::call(0i32));
    assert!(ForkIsMultipleOf::<Id, Id>::call(-7i32));

    type Mult3 = Subset<IsMultipleOfConst<ConstI32<3>>, i32>;
    assert!(Mult3::try_new(9).is_ok());
    assert!(Mult3::try_new(-6).is_ok());
    assert!(Mult3::try_new(10).is_err());
}

#[test]
fn float_is_integer_domain_and_parity() {
    // 1. `IsInteger` exclusively accepts finite subsets (`FiniteF32` / `FiniteF64` and outer subsets)
    for v in [0.0f32, -0.0, 1.0, -42.0, 2.0, -3.0, (1u32 << 23) as f32] {
        let fin = finite_f32(v).unwrap();
        assert!(IsInteger::call(fin), "expected {v} to be integer");
        assert!(IsInteger::call(&fin));
    }
    for v in [0.5f32, -0.1, f32::MIN_POSITIVE] {
        let fin = finite_f32(v).unwrap();
        assert!(!IsInteger::call(fin), "expected {v} not to be integer");
        assert!(!IsInteger::call(&fin));
    }

    for v in [
        0.0f64,
        -0.0,
        1.0,
        -42.0,
        ((1u64 << 52) + 1) as f64,
        (1u64 << 53) as f64,
    ] {
        let fin = finite_f64(v).unwrap();
        assert!(IsInteger::call(fin));
        assert!(IsInteger::call(&fin));
    }
    for v in [0.5f64, -0.1, 1e-300] {
        let fin = finite_f64(v).unwrap();
        assert!(!IsInteger::call(fin));
    }

    // Outer ordering predicates on `FiniteF32` (such as `PosFinite32`) still accept `IsInteger`
    type PosFinite32 = Subset<IsPositive, FiniteF32>;
    let pos_fin_two = PosFinite32::try_new(finite_f32(2.0).unwrap()).unwrap();
    let pos_fin_half = PosFinite32::try_new(finite_f32(0.5).unwrap()).unwrap();
    assert!(IsInteger::call(pos_fin_two));
    assert!(!IsInteger::call(&pos_fin_half));

    // Composite dependent-refinement paths (`AndThen`) also forward `IsInteger` directly
    type PosFiniteFlatF32 = Subset<AndThen<IsNotNaN, AndThen<IsFinite, IsPositive>>, f32>;
    let flat_two = PosFiniteFlatF32::try_new(2.0).unwrap();
    let flat_half = PosFiniteFlatF32::try_new(0.5).unwrap();
    assert!(IsInteger::call(flat_two));
    assert!(IsInteger::call(&flat_two));
    assert!(!IsInteger::call(flat_half));

    type PosFiniteOnNotNaNF64 = Subset<AndThen<IsFinite, IsPositive>, NotNaNF64>;
    let nn_two = PosFiniteOnNotNaNF64::try_new(NotNaNF64::try_new(2.0).unwrap()).unwrap();
    let nn_half = PosFiniteOnNotNaNF64::try_new(NotNaNF64::try_new(0.5).unwrap()).unwrap();
    assert!(IsInteger::call(nn_two));
    assert!(!IsInteger::call(&nn_half));

    // 2. `Subset<IsInteger, FiniteF32/64>` provides `SafeEq` / `SafeOrd` and unlocks `IsEven` / `IsOdd`
    type IntF32 = Subset<IsInteger, FiniteF32>;
    type IntF64 = Subset<IsInteger, FiniteF64>;

    let neg_zero_f32 = IntF32::try_new(finite_f32(-0.0).unwrap()).unwrap();
    let pos_zero_f32 = IntF32::try_new(finite_f32(0.0).unwrap()).unwrap();
    let three_f32 = IntF32::try_new(finite_f32(3.0).unwrap()).unwrap();
    let neg_four_f32 = IntF32::try_new(finite_f32(-4.0).unwrap()).unwrap();
    assert!(SafeEq::eq(&neg_zero_f32, &pos_zero_f32));
    assert!(SafeEq::eq(&neg_zero_f32, &0u8));
    assert!(SafeOrd::lt(&neg_four_f32, &neg_zero_f32));
    assert!(SafeOrd::lt(&pos_zero_f32, &three_f32));
    assert!(SafeOrd::gt(&three_f32, &2u16));

    for even in [
        0.0f32,
        -0.0,
        2.0,
        -4.0,
        (1u32 << 23) as f32,
        (1u32 << 24) as f32,
    ] {
        let sub = IntF32::try_new(finite_f32(even).unwrap()).unwrap();
        assert!(IsEven::call(sub), "expected {even} to be even");
        assert!(IsEven::call(&sub));
        assert!(!IsOdd::call(sub));
    }
    for odd in [1.0f32, -1.0, -3.0, ((1u32 << 23) + 1) as f32] {
        let sub = IntF32::try_new(finite_f32(odd).unwrap()).unwrap();
        assert!(IsOdd::call(sub), "expected {odd} to be odd");
        assert!(IsOdd::call(&sub));
        assert!(!IsEven::call(sub));
    }

    for even in [
        0.0f64,
        -0.0,
        2.0,
        -100.0,
        (1u64 << 52) as f64,
        (1u64 << 53) as f64,
    ] {
        let sub = IntF64::try_new(finite_f64(even).unwrap()).unwrap();
        assert!(IsEven::call(sub));
        assert!(!IsOdd::call(&sub));
    }
    for odd in [1.0f64, -7.0, ((1u64 << 52) + 1) as f64] {
        let sub = IntF64::try_new(finite_f64(odd).unwrap()).unwrap();
        assert!(IsOdd::call(sub));
        assert!(!IsEven::call(&sub));
    }

    // 3. `IsEven` / `IsOdd` over nested integer-domain `Subset`s (including `Subset<IsPositive, Subset<IsInteger, FiniteF32>>` and `Subset<IsPositive, i32>`)
    type PosI32 = Subset<IsPositive, i32>;
    type PosIntF32 = Subset<IsPositive, IntF32>;

    let p4 = PosI32::try_new(4).unwrap();
    let p5 = PosI32::try_new(5).unwrap();
    assert!(IsEven::call(&p4) && !IsOdd::call(p4));
    assert!(IsOdd::call(&p5) && !IsEven::call(p5));

    let pf2 = PosIntF32::try_new(IntF32::try_new(finite_f32(2.0).unwrap()).unwrap()).unwrap();
    let pf3 = PosIntF32::try_new(three_f32).unwrap();
    assert!(IsEven::call(pf2) && IsOdd::call(&pf3));

    // 4. Cover remaining composite IsFinite / IsInteger / IsEven / IsOdd paths on nested integer and float Subsets
    type PosNotNaNF32 = Subset<IsPositive, NotNaNF32>;
    let pos_nn = PosNotNaNF32::try_new(NotNaNF32::try_new(4.0).unwrap()).unwrap();
    let pos_nn_inf = PosNotNaNF32::try_new(NotNaNF32::try_new(f32::INFINITY).unwrap()).unwrap();
    assert!(IsFinite::call(pos_nn) && IsFinite::call(&pos_nn));
    assert!(!IsFinite::call(pos_nn_inf));

    type FlatFiniteF32 = Subset<AndThen<IsNotNaN, IsFinite>, f32>;
    let ff2 = FlatFiniteF32::try_new(2.0).unwrap();
    let ff_half = FlatFiniteF32::try_new(0.5).unwrap();
    assert!(IsInteger::call(ff2) && !IsInteger::call(ff_half));

    type PosOverFlatFiniteF32 = Subset<IsPositive, FlatFiniteF32>;
    let pff2 = PosOverFlatFiniteF32::try_new(ff2).unwrap();
    assert!(IsInteger::call(pff2) && IsInteger::call(&pff2));

    type FlatIntF32 = Subset<AndThen<IsNotNaN, AndThen<IsFinite, IsInteger>>, f32>;
    let flat_even = FlatIntF32::try_new(-4.0).unwrap();
    let flat_odd = FlatIntF32::try_new(3.0).unwrap();
    assert!(IsEven::call(flat_even) && IsOdd::call(&flat_odd));

    type IntOverFlatFinite = Subset<IsInteger, FlatFiniteF32>;
    let iof = IntOverFlatFinite::try_new(ff2).unwrap();
    assert!(IsEven::call(iof) && IsEven::call(&iof));

    type FiniteIntOnNotNaN = Subset<AndThen<IsFinite, IsInteger>, NotNaNF32>;
    let fin_nn = FiniteIntOnNotNaN::try_new(NotNaNF32::try_new(6.0).unwrap()).unwrap();
    assert!(IsEven::call(fin_nn) && IsEven::call(&fin_nn));

    type PosOverFlatInt = Subset<IsPositive, FlatIntF32>;
    let pfi = PosOverFlatInt::try_new(FlatIntF32::try_new(8.0).unwrap()).unwrap();
    assert!(IsEven::call(pfi) && IsEven::call(&pfi));

    type PosOverIntOverFlatFinite = Subset<IsPositive, IntOverFlatFinite>;
    let poiof = PosOverIntOverFlatFinite::try_new(iof).unwrap();
    assert!(IsEven::call(poiof) && IsEven::call(&poiof));

    type PosOverFiniteIntOnNotNaN = Subset<IsPositive, FiniteIntOnNotNaN>;
    let pofnn = PosOverFiniteIntOnNotNaN::try_new(fin_nn).unwrap();
    assert!(IsEven::call(pofnn) && IsEven::call(&pofnn));

    type FourLayerIntF32 = Subset<IsNonZero, PosIntF32>;
    let nz4 = FourLayerIntF32::try_new(pf2).unwrap();
    assert!(IsEven::call(nz4) && IsEven::call(&nz4));

    type TwoLayerI32 = Subset<IsNonZero, PosI32>;
    type ThreeLayerI32 = Subset<IsNonNegative, TwoLayerI32>;
    let tl2 = TwoLayerI32::try_new(p4).unwrap();
    let tl3 = ThreeLayerI32::try_new(tl2).unwrap();
    assert!(IsEven::call(tl2) && IsEven::call(&tl2));
    assert!(IsEven::call(tl3) && IsEven::call(&tl3));
}

#[cfg(feature = "nightly")]
#[test]
fn compile_time_const_float_integer_and_parity() {
    type IntF32 = Subset<IsInteger, FiniteF32>;
    type PosIntF32 = Subset<IsPositive, IntF32>;

    const fn try_const_finite_f32(x: f32) -> Option<FiniteF32> {
        match NotNaNF32::try_new(x) {
            Ok(nn) => match FiniteF32::try_new(nn) {
                Ok(fin) => Some(fin),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }
    const fn try_const_finite_f64(x: f64) -> Option<FiniteF64> {
        match NotNaNF64::try_new(x) {
            Ok(nn) => match FiniteF64::try_new(nn) {
                Ok(fin) => Some(fin),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }

    const INT_TRUE: bool = match (
        try_const_finite_f32(-42.0),
        try_const_finite_f64(((1u64 << 52) + 1) as f64),
    ) {
        (Some(a), Some(b)) => IsInteger::call(a) && IsInteger::call(b),
        _ => false,
    };
    const INT_FALSE: bool = match try_const_finite_f32(0.5) {
        Some(v) => IsInteger::call(v),
        None => true,
    };
    const SUBSET_EVEN: bool = match try_const_finite_f32((1u32 << 23) as f32) {
        Some(fin) => match IntF32::try_new(fin) {
            Ok(v) => IsEven::call(&v) && !IsOdd::call(v),
            Err(_) => false,
        },
        None => false,
    };
    const SUBSET_ODD: bool = match try_const_finite_f32(((1u32 << 23) + 1) as f32) {
        Some(fin) => match IntF32::try_new(fin) {
            Ok(v) => IsOdd::call(&v) && !IsEven::call(v),
            Err(_) => false,
        },
        None => false,
    };
    const NESTED_POS_ODD: bool = match try_const_finite_f32(5.0) {
        Some(fin) => match IntF32::try_new(fin) {
            Ok(v) => match PosIntF32::try_new(v) {
                Ok(pv) => IsOdd::call(&pv) && SafeOrd::gt(&pv, &4u8),
                Err(_) => false,
            },
            Err(_) => false,
        },
        None => false,
    };

    const {
        assert!(INT_TRUE);
        assert!(!INT_FALSE);
        assert!(SUBSET_EVEN);
        assert!(SUBSET_ODD);
        assert!(NESTED_POS_ODD);
    }
}
