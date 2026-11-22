//! Boundary tests for comparison protocols and comparison predicates.
#![cfg_attr(feature = "nightly", feature(const_trait_impl))]

use core::cmp::Ordering;
use fy_refine_core::cmp::{SafeEq, SafeOrd};
use fy_refine_core::function::constant::{ConstBool, ConstChar, ConstI32, ConstU8};
use fy_refine_core::function::predicate::{
    Eq, EqConst, ForkEq, ForkGe, ForkGt, ForkLe, ForkLt, ForkNe, Ge, GeConst, Gt, GtConst, Le,
    LeConst, Lt, LtConst, Ne, NeConst, PairEq, PairLt,
};
use fy_refine_core::function::wiring::Id;
use fy_refine_core::function::{Function, SafeFunction};
use fy_refine_core::marker::SafeDrop;
use fy_refine_core::subset::Subset;

fn verify<A: SafeOrd<B>, B: SafeOrd<A>>(a: A, b: B, expected: Ordering) {
    assert_eq!(SafeOrd::cmp(&a, &b), expected);
    assert_eq!(SafeOrd::cmp(&b, &a), expected.reverse());
    assert_eq!(SafeEq::eq(&a, &b), expected == Ordering::Equal);
    assert_eq!(SafeEq::ne(&a, &b), expected != Ordering::Equal);
    assert_eq!(SafeOrd::lt(&a, &b), expected == Ordering::Less);
    assert_eq!(SafeOrd::le(&a, &b), expected != Ordering::Greater);
    assert_eq!(SafeOrd::gt(&a, &b), expected == Ordering::Greater);
    assert_eq!(SafeOrd::ge(&a, &b), expected != Ordering::Less);
}

fn reference(a: (bool, u128), b: (bool, u128)) -> Ordering {
    match (a.0, b.0) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (true, true) => Ord::cmp(&b.1, &a.1),
        (false, false) => Ord::cmp(&a.1, &b.1),
    }
}
macro_rules! value {
    (signed, $v:expr) => {
        ($v < 0, ($v as i128).unsigned_abs())
    };
    (unsigned, $v:expr) => {
        (false, $v as u128)
    };
}
macro_rules! samples {
    (signed, $t:ty) => {
        [<$t>::MIN, -1, 0, 1, <$t>::MAX]
    };
    (unsigned, $t:ty) => {
        [0, 1, <$t>::MAX / 2, <$t>::MAX - 1, <$t>::MAX]
    };
}
macro_rules! row {
    ($kind:ident, $a:ty; $($other:ident $b:ty),*) => { $(
        for a in samples!($kind, $a) {
            for b in samples!($other, $b) {
                verify(a, b, reference(value!($kind, a), value!($other, b)));
            }
        }
    )* };
}
macro_rules! matrix {
    ($($kind:ident $a:ty),*) => { $(
        row!($kind, $a;
            signed i8, signed i16, signed i32, signed i64, signed i128, signed isize,
            unsigned u8, unsigned u16, unsigned u32, unsigned u64, unsigned u128, unsigned usize);
    )* };
}

#[test]
fn all_integer_pairs() {
    matrix!(signed i8, signed i16, signed i32, signed i64, signed i128, signed isize,
        unsigned u8, unsigned u16, unsigned u32, unsigned u64, unsigned u128, unsigned usize);
}

#[test]
fn exhaustive_i8_u8() {
    for a in i8::MIN..=i8::MAX {
        for b in u8::MIN..=u8::MAX {
            verify(a, b, Ord::cmp(&(a as i16), &(b as i16)));
        }
    }
}

#[test]
fn single_sided_ref_and_primitive_slices_cmp() {
    // Single-sided reference-to-value comparison
    assert!(LtConst::<ConstU8<10>>::call(&5u8));
    assert!(LtConst::<ConstU8<10>>::call(&-1i8));
    assert!(!LtConst::<ConstU8<10>>::call(&10u8));
    assert!(EqConst::<ConstI32<42>>::call(&42u64));
    assert!(SafeOrd::lt(&&5u8, &10i32));
    assert!(SafeOrd::gt(&10i32, &&5u8));

    // Safe comparisons for bool and char
    assert!(EqConst::<ConstBool<true>>::call(&true));
    assert!(LtConst::<ConstBool<true>>::call(&false));
    assert!(EqConst::<ConstChar<'z'>>::call(&'z'));
    assert!(LtConst::<ConstChar<'z'>>::call(&'a'));

    // Safe comparisons and single-sided reference forwarding for str / [T] / [T; N]
    assert!(SafeOrd::lt(&"abc", &"abd"));
    assert!(SafeEq::eq(&"hello", &"hello"));
    assert!(SafeEq::eq(&&"hello", &"hello"));
    assert!(SafeEq::eq(&"hello", &&"hello"));
    assert!(SafeOrd::lt(&&"abc", &"abd"));
    assert!(SafeOrd::lt(&"abc", &&"abd"));
    assert!(SafeOrd::lt(&[1u8, 2][..], &[1u8, 3][..]));
    assert!(SafeOrd::lt(&[1u8, 2][..], &[1u8, 2, 0][..]));
    assert!(SafeEq::eq(&&[1u8, 2][..], &[1u8, 2][..]));
    assert!(SafeOrd::lt(&[1u8, 2][..], &&[1u8, 3][..]));

    struct ConstTag;
    impl Function<()> for ConstTag {
        type Output = &'static str;
        fn call((): ()) -> &'static str {
            "ok"
        }
    }
    unsafe impl SafeFunction<()> for ConstTag {}
    type OkStaticStr = Subset<EqConst<ConstTag>, &'static str>;
    assert!(OkStaticStr::try_new("ok").is_ok());
    assert!(OkStaticStr::try_new("no").is_err());

    // Cross-carrier and single-sided reference SafeEq / SafeOrd for fixed-size arrays
    let arr_a: [i8; 3] = [-1, 2, 3];
    let arr_b: [u128; 3] = [0, 2, 3];
    let arr_c: [u16; 3] = [1, 2, 3];
    assert!(SafeOrd::lt(&arr_a, &arr_b));
    assert!(SafeOrd::lt(&&arr_a, &arr_b));
    assert!(SafeOrd::gt(&arr_b, &&arr_a));
    assert!(SafeEq::eq(&[1i8, 2, 3], &arr_c));
    assert!(SafeEq::eq(&&[1i8, 2, 3], &arr_c));
    assert!(SafeEq::eq(&arr_c, &&[1i8, 2, 3]));

    struct ConstArr;
    impl Function<()> for ConstArr {
        type Output = [u16; 3];
        fn call((): ()) -> [u16; 3] {
            [1, 2, 3]
        }
    }
    unsafe impl SafeFunction<()> for ConstArr {}
    assert!(EqConst::<ConstArr>::call(&[1i8, 2, 3]));
    assert!(LtConst::<ConstArr>::call(&[1i8, 2, 2]));
}

struct Record(i8, u128);
struct Left;
struct Right;
impl<'a> Function<&'a Record> for Left {
    type Output = &'a i8;
    fn call(input: &'a Record) -> Self::Output {
        &input.0
    }
}
impl<'a> Function<&'a Record> for Right {
    type Output = &'a u128;
    fn call(input: &'a Record) -> Self::Output {
        &input.1
    }
}
// SAFETY: Only borrows a field; no side effects, panics, or non-termination.
unsafe impl SafeFunction<&Record> for Left {}
// SAFETY: Same as above.
unsafe impl SafeFunction<&Record> for Right {}

#[test]
fn borrowed_non_copy_input() {
    for (a, b) in [(-1, u128::MAX), (0, 0), (1, 0)] {
        let input = Record(a, b);
        let order = reference((a < 0, (a as i128).unsigned_abs()), (false, b));
        assert_eq!(
            ForkEq::<Left, Right>::call(&input),
            order == Ordering::Equal
        );
        assert_eq!(
            ForkNe::<Left, Right>::call(&input),
            order != Ordering::Equal
        );
        assert_eq!(ForkLt::<Left, Right>::call(&input), order == Ordering::Less);
        assert_eq!(
            ForkLe::<Left, Right>::call(&input),
            order != Ordering::Greater
        );
        assert_eq!(
            ForkGt::<Left, Right>::call(&input),
            order == Ordering::Greater
        );
        assert_eq!(ForkGe::<Left, Right>::call(&input), order != Ordering::Less);
    }
}

struct RuntimeValue(u8);
// SAFETY: Directly compares integers, satisfying pure equality laws.
unsafe impl SafeEq for RuntimeValue {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
    fn ne(&self, other: &Self) -> bool {
        self.0 != other.0
    }
}
struct Identity;
impl<'a> Function<&'a RuntimeValue> for Identity {
    type Output = &'a RuntimeValue;
    fn call(input: &'a RuntimeValue) -> Self::Output {
        input
    }
}
// SAFETY: Returns the shared reference verbatim; pure, total, and terminating.
unsafe impl SafeFunction<&RuntimeValue> for Identity {}

#[test]
fn ordinary_comparison_impl_remains_usable() {
    assert!(ForkEq::<Identity, Identity>::call(&RuntimeValue(1)));
    assert!(!ForkNe::<Identity, Identity>::call(&RuntimeValue(1)));
}

#[cfg(feature = "nightly")]
#[path = "comparison/compile_time.rs"]
mod compile_time;

struct Owned(Vec<u8>);
// SAFETY: Contains only Vec<u8>; destruction is benign, panic-free, and always terminates.
unsafe impl SafeDrop for Owned {}
// SAFETY: Comparison of Vec<u8> satisfies mathematical equality laws, is panic-free, and always terminates.
unsafe impl SafeEq for Owned {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
    fn ne(&self, other: &Self) -> bool {
        self.0 != other.0
    }
}
// SAFETY: Delegates directly to the total order of byte sequences, consistent with the equality relation above.
unsafe impl SafeOrd for Owned {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.0, &other.0)
    }
    fn lt(&self, other: &Self) -> bool {
        self.0 < other.0
    }
    fn le(&self, other: &Self) -> bool {
        self.0 <= other.0
    }
    fn gt(&self, other: &Self) -> bool {
        self.0 > other.0
    }
    fn ge(&self, other: &Self) -> bool {
        self.0 >= other.0
    }
}
struct Empty;
impl Function<()> for Empty {
    type Output = Owned;
    fn call(_: ()) -> Owned {
        Owned(Vec::new())
    }
}
// SAFETY: Always returns an empty sequence; no allocation, panics, or observable side effects.
unsafe impl SafeFunction<()> for Empty {}

#[test]
fn owned_non_copy_output() {
    fn check<P: fy_refine_core::function::Predicate<()>>() -> bool {
        P::call(())
    }
    assert!(check::<ForkEq<Empty, Empty>>());
    assert!(!check::<ForkNe<Empty, Empty>>());
    assert!(!check::<ForkLt<Empty, Empty>>());
    assert!(check::<ForkLe<Empty, Empty>>());
    assert!(!check::<ForkGt<Empty, Empty>>());
    assert!(check::<ForkGe<Empty, Empty>>());

    // Binary comparison operators Eq..Ge and PairEq / PairLt
    assert!(Eq::call((10, 10)));
    assert!(!Eq::call((10, 20)));
    assert!(Ne::call((10, 20)));
    assert!(!Ne::call((10, 10)));
    assert!(Lt::call((10, 20)));
    assert!(!Lt::call((20, 10)));
    assert!(Le::call((10, 10)));
    assert!(Le::call((10, 20)));
    assert!(!Le::call((20, 10)));
    assert!(Gt::call((20, 10)));
    assert!(!Gt::call((10, 20)));
    assert!(Ge::call((20, 20)));
    assert!(Ge::call((20, 10)));
    assert!(!Ge::call((10, 20)));
    assert!(PairEq::<Id, Id>::call((10, 10)));
    assert!(PairLt::<Id, Id>::call((-1i8, u128::MAX)));

    // Transparent constant-alias comparison operators EqConst..GeConst (including non-Copy owned Owned checks)
    assert!(EqConst::<Empty>::call(Owned(Vec::new())));
    assert!(!EqConst::<Empty>::call(Owned(vec![1])));
    assert!(GtConst::<Empty>::call(Owned(vec![1])));

    assert!(EqConst::<ConstI32<10>>::call(10));
    assert!(!EqConst::<ConstI32<10>>::call(20));
    assert!(NeConst::<ConstI32<10>>::call(20));
    assert!(!NeConst::<ConstI32<10>>::call(10));
    assert!(LtConst::<ConstI32<10>>::call(5));
    assert!(!LtConst::<ConstI32<10>>::call(10));
    assert!(LeConst::<ConstI32<10>>::call(10));
    assert!(LeConst::<ConstI32<10>>::call(5));
    assert!(!LeConst::<ConstI32<10>>::call(15));
    assert!(GtConst::<ConstI32<10>>::call(15));
    assert!(!GtConst::<ConstI32<10>>::call(10));
    assert!(GeConst::<ConstI32<10>>::call(10));
    assert!(GeConst::<ConstI32<10>>::call(15));
    assert!(!GeConst::<ConstI32<10>>::call(5));
}

fn verify_unsized<A: ?Sized + SafeOrd<B>, B: ?Sized + SafeOrd<A>>(
    a: &A,
    b: &B,
    expected: Ordering,
) {
    assert_eq!(SafeOrd::cmp(a, b), expected);
    assert_eq!(SafeOrd::cmp(b, a), expected.reverse());
    assert_eq!(SafeEq::eq(a, b), expected == Ordering::Equal);
    assert_eq!(SafeEq::ne(a, b), expected != Ordering::Equal);
    assert_eq!(SafeOrd::lt(a, b), expected == Ordering::Less);
    assert_eq!(SafeOrd::le(a, b), expected != Ordering::Greater);
    assert_eq!(SafeOrd::gt(a, b), expected == Ordering::Greater);
    assert_eq!(SafeOrd::ge(a, b), expected != Ordering::Less);
}

#[test]
fn complete_safe_cmp_ops_and_single_sided_refs() {
    // 1. Single-sided reference coverage across all 7 safe comparison methods on integers
    let v_u8_5 = 5u8;
    let v_i32_10 = 10i32;
    let v_i8_neg1 = -1i8;
    let v_u16_0 = 0u16;
    let v_i32_42 = 42i32;
    let r_u8_5 = &v_u8_5;
    let r_i32_10 = &v_i32_10;
    let r_i8_neg1 = &v_i8_neg1;
    let r_u16_0 = &v_u16_0;
    let r_i32_42 = &v_i32_42;
    verify(r_u8_5, v_i32_10, Ordering::Less);
    verify(v_u8_5, r_i32_10, Ordering::Less);
    verify(r_i8_neg1, v_u16_0, Ordering::Less);
    verify(v_i8_neg1, r_u16_0, Ordering::Less);
    verify(r_i32_42, v_i32_42, Ordering::Equal);

    // 2. bool and char, including single-sided references
    let v_false = false;
    let v_true = true;
    let r_false = &v_false;
    let r_true = &v_true;
    verify(v_false, v_true, Ordering::Less);
    verify(v_true, v_true, Ordering::Equal);
    verify(r_false, v_true, Ordering::Less);
    verify(v_false, r_true, Ordering::Less);

    let v_a = 'a';
    let v_z = 'z';
    let r_a = &v_a;
    let r_z = &v_z;
    verify(v_a, v_z, Ordering::Less);
    verify(v_a, v_a, Ordering::Equal);
    verify(r_a, v_z, Ordering::Less);
    verify(v_a, r_z, Ordering::Less);

    // 3. All methods for str and single-sided references
    verify_unsized::<str, str>("abc", "abd", Ordering::Less);
    verify_unsized::<str, str>("abc", "abc", Ordering::Equal);
    verify_unsized::<str, str>("ab", "abc", Ordering::Less);
    verify_unsized::<&str, str>(&"abc", "abd", Ordering::Less);
    verify_unsized::<&str, str>(&"abc", "abc", Ordering::Equal);

    // 4. All methods for [u8] and single-sided references (including mismatched-length SafeEq)
    verify_unsized::<[u8], [u8]>(b"ab", b"abc", Ordering::Less);
    verify_unsized::<[u8], [u8]>(b"ab", b"ac", Ordering::Less);
    verify_unsized::<[u8], [u8]>(b"ab", b"ab", Ordering::Equal);
    verify_unsized::<&[u8], [u8]>(&&b"ab"[..], b"abc", Ordering::Less);
    verify_unsized::<&[u8], [u8]>(&&b"ab"[..], b"ab", Ordering::Equal);

    // 5. All methods for [T; N] and single-sided references
    let arr_a = [-1i8, 2, 3];
    let arr_b = [0u16, 2, 3];
    let arr_eq_l = [1i8, 2, 3];
    let arr_eq_r = [1u16, 2, 3];
    let r_arr_a = &arr_a;
    let r_arr_b = &arr_b;
    let r_arr_eq_l = &arr_eq_l;
    verify(arr_a, arr_b, Ordering::Less);
    verify(arr_eq_l, arr_eq_r, Ordering::Equal);
    verify(r_arr_a, arr_b, Ordering::Less);
    verify(arr_a, r_arr_b, Ordering::Less);
    verify(r_arr_eq_l, arr_eq_r, Ordering::Equal);
}
