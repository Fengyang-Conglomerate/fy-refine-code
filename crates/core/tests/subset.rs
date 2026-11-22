//! Tests for the refinement subset type `Subset<P, T>`, safety contracts, recursive penetration, safe cloning, and slice/ASCII usage.
#![cfg_attr(feature = "nightly", feature(const_trait_impl))]

use core::cmp::Ordering;
use core::convert::Infallible;
use core::num::{NonZeroI32, NonZeroU8, NonZeroUsize, Saturating, Wrapping};
use core::ops::{RangeFull, RangeInclusive};
use core::time::Duration;

use fy_refine_core::cmp::{SafeEq, SafeOrd};
use fy_refine_core::function::collection::{
    Any, First, Get, IsEmpty, IsNonEmpty, Last, Len, LenGt, TryFirst, TryGet, TryLast,
};
use fy_refine_core::function::constant::{ConstI32, ConstU8, ConstUsize};
use fy_refine_core::function::predicate::{
    All, And, AnyOf, CharToU32, Closed, Eq, EqConst, ForkLt, IsAscii, IsAsciiAlphabetic,
    IsAsciiAlphanumeric, IsAsciiDigit, IsAsciiHexDigit, IsAsciiLowercase, IsAsciiUppercase,
    IsAsciiWhitespace, IsChar, IsEven, IsNonNegative, IsNonZero, IsPositive, IsUtf8, IsZero, Lt,
    LtConst, Not, Open, Or, Predicate, Range, U32ToChar,
};
use fy_refine_core::function::slice::{AsBytes, AsSlice, AsStr};
use fy_refine_core::function::wiring::{
    AsBase, AsInner, Cloned, Discard, Fst, IntoBase, IntoInner, Then, WithRef,
};
use fy_refine_core::function::{Function, SafeFunction};
use fy_refine_core::marker::{Phantom, SafeClone, SafeDrop, SafeValue};
use fy_refine_core::subset::Subset;
use fy_refine_core::{field_projector, impl_safe_clone, impl_safe_drop, impl_safe_value};

#[test]
fn builtin_primitives_and_extended_types_implement_contracts() {
    fn assert_value_drop<T: ?Sized + SafeValue + SafeDrop>() {}
    fn assert_sized_all<T: SafeValue + SafeDrop + SafeClone>() {}

    assert_sized_all::<()>();
    assert_sized_all::<bool>();
    assert_sized_all::<char>();
    assert_sized_all::<u8>();
    assert_sized_all::<u32>();
    assert_sized_all::<u64>();
    assert_sized_all::<usize>();
    assert_sized_all::<i8>();
    assert_sized_all::<i32>();
    assert_sized_all::<i64>();
    assert_sized_all::<isize>();
    assert_sized_all::<f32>();
    assert_sized_all::<f64>();

    assert_sized_all::<NonZeroU8>();
    assert_sized_all::<NonZeroI32>();
    assert_sized_all::<NonZeroUsize>();
    assert_sized_all::<Wrapping<u32>>();
    assert_sized_all::<Saturating<i64>>();

    assert_sized_all::<Ordering>();
    assert_sized_all::<Duration>();
    assert_sized_all::<Infallible>();
    assert_sized_all::<core::ops::Range<usize>>();
    assert_sized_all::<RangeInclusive<u32>>();
    assert_sized_all::<RangeFull>();

    // Dynamically sized types (DST slices)
    assert_value_drop::<str>();
    assert_value_drop::<[i32]>();

    // Composite containers
    assert_sized_all::<Option<i32>>();
    assert_sized_all::<Result<u32, i32>>();
    assert_sized_all::<[u8; 32]>();
    assert_sized_all::<(i32, u64)>();
    assert_sized_all::<&str>();
    assert_sized_all::<&[i32]>();
    assert_sized_all::<Phantom<i32>>();

    #[cfg(feature = "alloc")]
    {
        extern crate alloc;
        use alloc::borrow::Cow;
        use alloc::boxed::Box;
        use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};
        use alloc::rc::Rc;
        use alloc::string::String;
        use alloc::sync::Arc;
        use alloc::vec::Vec;
        use core::ops::ControlFlow;

        assert_sized_all::<String>();
        assert_sized_all::<ControlFlow<i32, String>>();
        assert_sized_all::<Box<i32>>();
        assert_sized_all::<Box<str>>();
        assert_sized_all::<Box<[u8]>>();
        assert_sized_all::<Vec<u8>>();
        assert_sized_all::<VecDeque<u16>>();
        assert_sized_all::<LinkedList<char>>();
        assert_sized_all::<BinaryHeap<i32>>();
        assert_sized_all::<BTreeSet<u64>>();
        assert_sized_all::<BTreeMap<String, i32>>();
        assert_sized_all::<Cow<'static, str>>();
        assert_sized_all::<Cow<'static, [u8]>>();
        assert_sized_all::<Rc<i32>>();
        assert_sized_all::<Rc<str>>();
        assert_sized_all::<Rc<[u8]>>();
        assert_sized_all::<Arc<i32>>();
        assert_sized_all::<Arc<str>>();
        assert_sized_all::<Arc<[u8]>>();
        assert_sized_all::<Arc<Subset<IsPositive, i32>>>();
    }

    #[cfg(feature = "std")]
    {
        use std::collections::{HashMap, HashSet};
        use std::hash::RandomState;

        assert_sized_all::<RandomState>();
        assert_sized_all::<HashSet<u64>>();
        assert_sized_all::<HashMap<String, i32>>();
    }
}

#[test]
fn subset_checked_construction_and_nested_base_penetration() {
    type PositiveI32 = Subset<IsPositive, i32>;

    assert!(PositiveI32::is_valid(&10));
    assert!(!PositiveI32::is_valid(&0));
    assert!(!PositiveI32::is_valid(&-5));

    let pos = PositiveI32::try_new(42).unwrap();
    Subset::assume_valid(&pos);
    assert_eq!(*pos, 42);
    assert_eq!(*Subset::as_inner(&pos), 42);
    assert_eq!(*Subset::as_base(&pos), 42);
    assert_eq!(Subset::into_inner(pos), 42);
    assert_eq!(PositiveI32::try_new(-3), Err(-3));

    // Compile-time const fn single-layer unwrapping
    const UNCHECKED: PositiveI32 = unsafe { PositiveI32::new_unchecked(7) };
    const INNER_REF: &i32 = Subset::as_inner(&UNCHECKED);
    const INNER_VAL: i32 = Subset::into_inner(UNCHECKED);
    assert_eq!(*INNER_REF, 7);
    assert_eq!(INNER_VAL, 7);

    // Two-layer nested Subset<IsEven, Subset<IsPositive, i32>>: distinguishing as_inner / into_inner from as_base / into_base
    struct IsEvenInner;
    impl<'a> Function<&'a PositiveI32> for IsEvenInner {
        type Output = bool;
        fn call(input: &'a PositiveI32) -> bool {
            **input % 2 == 0
        }
    }
    unsafe impl SafeFunction<&PositiveI32> for IsEvenInner {}

    type PositiveEvenI32 = Subset<IsEvenInner, PositiveI32>;
    assert_eq!(
        core::mem::size_of::<PositiveEvenI32>(),
        core::mem::size_of::<i32>()
    );

    let p1 = PositiveI32::try_new(8).unwrap();
    let pe = PositiveEvenI32::try_new(p1).unwrap();

    // Single-layer unwrap yields PositiveI32
    let inner_ref: &PositiveI32 = Subset::as_inner(&pe);
    assert_eq!(**inner_ref, 8);
    // Recursive penetration reaches the innermost base i32
    let base_ref: &i32 = Subset::as_base(&pe);
    assert_eq!(*base_ref, 8);

    let inner_owned: PositiveI32 = Subset::into_inner(pe);
    assert_eq!(*inner_owned, 8);

    let pe2 = PositiveEvenI32::try_new(inner_owned).unwrap();
    let base_owned: i32 = Subset::into_base(pe2);
    assert_eq!(base_owned, 8);

    // On failure, returns the direct inner PositiveI32
    let odd = PositiveI32::try_new(9).unwrap();
    let returned: PositiveI32 = PositiveEvenI32::try_new(odd).unwrap_err();
    assert_eq!(*returned, 9);
}

#[test]
fn safe_clone_preserves_invariants_and_private_state() {
    // 1. RangeInclusive's private exhausted flag must remain faithful after cloning
    struct IsExhaustedRange;
    impl<'a> Function<&'a RangeInclusive<u8>> for IsExhaustedRange {
        type Output = bool;
        fn call(r: &'a RangeInclusive<u8>) -> bool {
            r.is_empty()
        }
    }
    unsafe impl SafeFunction<&RangeInclusive<u8>> for IsExhaustedRange {}

    let mut range = 5u8..=5u8;
    assert_eq!(range.next(), Some(5));
    assert!(range.is_empty());

    type ExhaustedRange = Subset<IsExhaustedRange, RangeInclusive<u8>>;
    let wrapped = ExhaustedRange::try_new(range).unwrap();
    let cloned = wrapped.clone();
    assert!(cloned.is_empty());
    assert!(ExhaustedRange::is_valid(&cloned));

    // 2. Faithful SafeClone for collection containers and Cow
    #[cfg(feature = "alloc")]
    {
        extern crate alloc;
        use alloc::borrow::Cow;
        use alloc::collections::{BTreeMap, BTreeSet};

        let bset = Subset::<IsNonEmpty, BTreeSet<i32>>::try_new(BTreeSet::from([1, 2, 3])).unwrap();
        let bset_cloned = bset.clone();
        assert_eq!(*bset, *bset_cloned);

        let bmap = Subset::<IsNonEmpty, BTreeMap<&str, i32>>::try_new(BTreeMap::from([
            ("a", 1),
            ("b", 2),
        ]))
        .unwrap();
        let bmap_cloned = bmap.clone();
        assert_eq!(*bmap, *bmap_cloned);

        struct NonEmptyCowStr;
        impl<'a> Function<&'a Cow<'static, str>> for NonEmptyCowStr {
            type Output = bool;
            fn call(s: &'a Cow<'static, str>) -> bool {
                !s.is_empty()
            }
        }
        unsafe impl SafeFunction<&Cow<'static, str>> for NonEmptyCowStr {}

        let c1 = Subset::<NonEmptyCowStr, Cow<'static, str>>::try_new(Cow::Borrowed("fy")).unwrap();
        let c2 = c1.clone();
        assert_eq!(&**c1, &**c2);

        struct NonEmptyCowBytes;
        impl<'a> Function<&'a Cow<'static, [u8]>> for NonEmptyCowBytes {
            type Output = bool;
            fn call(s: &'a Cow<'static, [u8]>) -> bool {
                !s.is_empty()
            }
        }
        unsafe impl SafeFunction<&Cow<'static, [u8]>> for NonEmptyCowBytes {}

        let b1 = Subset::<NonEmptyCowBytes, Cow<'static, [u8]>>::try_new(Cow::Owned(vec![1, 2]))
            .unwrap();
        let b2 = b1.clone();
        assert_eq!(&**b1, &**b2);
    }

    #[cfg(feature = "std")]
    {
        use std::collections::{HashMap, HashSet};

        let hset = Subset::<IsNonEmpty, HashSet<i32>>::try_new(HashSet::from([10, 20])).unwrap();
        assert_eq!(*hset, *hset.clone());

        let hmap =
            Subset::<IsNonEmpty, HashMap<&str, i32>>::try_new(HashMap::from([("k", 7)])).unwrap();
        assert_eq!(*hmap, *hmap.clone());
    }
}

#[test]
fn subset_safe_cmp_and_wiring_penetration() {
    type PosI8 = Subset<IsPositive, i8>;
    type NonZeroU128 = Subset<IsNonZero, u128>;
    type PosI32 = Subset<IsPositive, i32>;
    type PosEq8 = Subset<Then<AsInner, EqConst<ConstI32<8>>>, PosI32>;

    // Cross-predicate and cross-base-integer Subset SafeEq / SafeOrd
    let a = PosI8::try_new(5).unwrap();
    let b = NonZeroU128::try_new(100).unwrap();
    let c = NonZeroU128::try_new(5).unwrap();
    assert!(SafeOrd::lt(&a, &b));
    assert!(SafeEq::eq(&a, &c));
    assert!(Lt::call((a, b)));
    assert!(Eq::call((a, c)));

    // Struct fields of Subset types can be compared directly via ForkLt
    struct Window {
        lo: PosI32,
        hi: PosI32,
    }
    struct GetLo;
    impl<'a> Function<&'a Window> for GetLo {
        type Output = &'a PosI32;
        fn call(w: &'a Window) -> &'a PosI32 {
            &w.lo
        }
    }
    unsafe impl SafeFunction<&Window> for GetLo {}

    struct GetHi;
    impl<'a> Function<&'a Window> for GetHi {
        type Output = &'a PosI32;
        fn call(w: &'a Window) -> &'a PosI32 {
            &w.hi
        }
    }
    unsafe impl SafeFunction<&Window> for GetHi {}

    let ok_win = Window {
        lo: PosI32::try_new(1).unwrap(),
        hi: PosI32::try_new(10).unwrap(),
    };
    let bad_win = Window {
        lo: PosI32::try_new(10).unwrap(),
        hi: PosI32::try_new(1).unwrap(),
    };
    assert!(ForkLt::<GetLo, GetHi>::call(&ok_win));
    assert!(!ForkLt::<GetLo, GetHi>::call(&bad_win));

    // SafeOrd comparison on DST Subset (&Subset<..., str>)
    type AsciiStr = Subset<Then<AsBytes, All<IsAscii>>, str>;
    let s1 = AsciiStr::try_from_ref("alpha").unwrap();
    let s2 = AsciiStr::try_from_ref("beta").unwrap();
    assert!(SafeOrd::lt(s1, s2));
    assert!(Lt::call((s1, s2)));

    // AsInner / IntoInner / AsBase / IntoBase wiring operators
    let p8 = PosI32::try_new(8).unwrap();
    let nested = PosEq8::try_new(p8).unwrap();
    assert_eq!(AsInner::call(&nested), &p8);
    assert_eq!(*AsBase::call(&nested), 8);
    assert_eq!(IntoInner::call(nested), p8);
    assert_eq!(IntoBase::call(nested), 8);

    // Zero-boilerplate penetration on nested DST via Then<AsBase, Then<AsBytes, Len>>
    type NestedAscii = Subset<Then<AsInner, Then<AsBytes, IsNonEmpty>>, AsciiStr>;
    let ns = NestedAscii::try_from_ref(s1).unwrap();
    assert_eq!(Then::<AsBase, Then<AsBytes, Len>>::call(ns), 5);
}

#[test]
fn dst_nested_subset_and_bcp47_like_predicates() {
    // 1. Nested DST: Deref and as_base penetration on Subset<P2, Subset<P1, str>>
    type AsciiStrPred = Then<AsBytes, All<IsAscii>>;
    type AsciiStr = Subset<AsciiStrPred, str>;

    struct NonEmptyAsciiStr;
    impl<'a> Function<&'a AsciiStr> for NonEmptyAsciiStr {
        type Output = bool;
        fn call(input: &'a AsciiStr) -> bool {
            !input.is_empty()
        }
    }
    unsafe impl SafeFunction<&AsciiStr> for NonEmptyAsciiStr {}

    type NonEmptyAscii = Subset<NonEmptyAsciiStr, AsciiStr>;

    let s1: &AsciiStr = AsciiStr::try_from_ref("hello").unwrap();
    Subset::assume_valid(s1);
    assert!(AsciiStr::try_from_ref("你好").is_none());
    let s2: &NonEmptyAscii = NonEmptyAscii::try_from_ref(s1).unwrap();

    // Automatic Deref penetration to invoke str methods
    assert_eq!(s2.len(), 5);
    assert_eq!(&s2[0..2], "he");
    // Explicit single-layer and full base borrowing
    let mid: &AsciiStr = Subset::as_inner(s2);
    let raw: &str = Subset::as_base(s2);
    assert_eq!(Subset::as_inner(mid), "hello");
    assert_eq!(raw, "hello");

    // 2. Grammar predicates modeled after fy-i18n-lang-tag (zero unsafe)
    // Script = 4ALPHA
    type ScriptPred = Then<AsBytes, And<Then<Len, EqConst<ConstUsize<4>>>, All<IsAsciiAlphabetic>>>;
    type ScriptStr = Subset<ScriptPred, str>;
    assert!(ScriptStr::try_from_ref("Latn").is_some());
    assert!(ScriptStr::try_from_ref("Hans").is_some());
    assert!(ScriptStr::try_from_ref("La1n").is_none());
    assert!(ScriptStr::try_from_ref("Lat").is_none());

    // Region = 2ALPHA / 3DIGIT
    type RegionPred = Then<
        AsBytes,
        Or<
            And<Then<Len, EqConst<ConstUsize<2>>>, All<IsAsciiAlphabetic>>,
            And<Then<Len, EqConst<ConstUsize<3>>>, All<IsAsciiDigit>>,
        >,
    >;
    type RegionStr = Subset<RegionPred, str>;
    assert!(RegionStr::try_from_ref("CN").is_some());
    assert!(RegionStr::try_from_ref("419").is_some());
    assert!(RegionStr::try_from_ref("C1").is_none());
    assert!(RegionStr::try_from_ref("USA").is_none());

    // Language = 2*3ALPHA / 5*8ALPHA
    type LangLenPred = AnyOf<(
        Range<Closed<ConstUsize<2>>, Closed<ConstUsize<3>>>,
        Range<Closed<ConstUsize<5>>, Closed<ConstUsize<8>>>,
    )>;
    type LanguagePred = Then<AsBytes, And<Then<Len, LangLenPred>, All<IsAsciiAlphabetic>>>;
    type LanguageStr = Subset<LanguagePred, str>;
    assert!(LanguageStr::try_from_ref("zh").is_some());
    assert!(LanguageStr::try_from_ref("yue").is_some());
    assert!(LanguageStr::try_from_ref("local").is_some());
    assert!(LanguageStr::try_from_ref("abcd").is_none()); // 4 chars invalid
    assert!(LanguageStr::try_from_ref("a").is_none()); // 1 char invalid
    assert!(LanguageStr::try_from_ref("abcdefghi").is_none()); // 9 chars invalid
}

#[test]
fn range_ascii_and_slice_predicates_comprehensive() {
    // Four explicit open/closed interval combinations and Not<Range<...>>
    type ClosedOpen = Subset<Range<Closed<ConstU8<10>>, Open<ConstU8<20>>>, u8>;
    assert!(ClosedOpen::try_new(10).is_ok());
    assert!(ClosedOpen::try_new(19).is_ok());
    assert_eq!(ClosedOpen::try_new(20), Err(20));
    assert_eq!(ClosedOpen::try_new(9), Err(9));

    type OpenClosed = Range<Open<ConstU8<10>>, Closed<ConstU8<20>>>;
    assert!(!OpenClosed::call(&10u8));
    assert!(OpenClosed::call(&11u8));
    assert!(OpenClosed::call(&20u8));
    assert!(!OpenClosed::call(&21u8));

    type StrictOpen = Range<Open<ConstU8<10>>, Open<ConstU8<20>>>;
    assert!(!StrictOpen::call(&10u8));
    assert!(StrictOpen::call(&15u8));
    assert!(!StrictOpen::call(&20u8));
    assert!(Not::<StrictOpen>::call(&10u8));
    assert!(!Not::<StrictOpen>::call(&15u8));

    // Flat union of multiple ranges and singletons (AnyOf)
    type IdentByte = AnyOf<(
        Range<Closed<ConstU8<b'a'>>, Closed<ConstU8<b'z'>>>,
        Range<Closed<ConstU8<b'0'>>, Closed<ConstU8<b'9'>>>,
        EqConst<ConstU8<b'_'>>,
        EqConst<ConstU8<b'-'>>,
    )>;
    assert!(IdentByte::call(&b'k'));
    assert!(IdentByte::call(&b'7'));
    assert!(IdentByte::call(&b'_'));
    assert!(IdentByte::call(&b'-'));
    assert!(!IdentByte::call(&b'!'));

    // Exhaustive check over all 256 bytes verifying exact equivalence with std ASCII classification
    for b in 0u8..=255 {
        assert_eq!(IsAscii::call(&b), b.is_ascii());
        assert_eq!(IsAsciiLowercase::call(&b), b.is_ascii_lowercase());
        assert_eq!(IsAsciiUppercase::call(&b), b.is_ascii_uppercase());
        assert_eq!(IsAsciiDigit::call(&b), b.is_ascii_digit());
        assert_eq!(IsAsciiAlphabetic::call(&b), b.is_ascii_alphabetic());
        assert_eq!(IsAsciiAlphanumeric::call(&b), b.is_ascii_alphanumeric());
        assert_eq!(IsAsciiHexDigit::call(&b), b.is_ascii_hexdigit());
        assert_eq!(IsAsciiWhitespace::call(&b), b.is_ascii_whitespace());
    }

    // Fallible raw slice projections (TryFirst / TryLast / TryGet) and quantifiers
    let arr = [1u8, 2, 3];
    assert_eq!(Len::call(&arr), 3);
    assert_eq!(AsSlice::call(&arr), &[1, 2, 3][..]);
    assert_eq!(TryGet::<1>::call(&arr), Some(&2));
    assert_eq!(TryGet::<5>::call(&arr), None);
    assert_eq!(TryFirst::call(&arr), Some(&1));
    assert_eq!(TryLast::call(&arr), Some(&3));
    assert!(!IsEmpty::call(&arr));
    assert!(IsNonEmpty::call(&arr));
    assert!(Any::<EqConst<ConstU8<2>>>::call(&arr));
    assert!(!Any::<EqConst<ConstU8<9>>>::call(&arr));

    // Infallible refined extraction (First / Last / Get, no Option, no bounds check)
    let non_empty_slice = Subset::<IsNonEmpty, [u8]>::try_from_ref(&arr[..]).unwrap();
    assert_eq!(*First::call(non_empty_slice), 1);
    assert_eq!(*Last::call(non_empty_slice), 3);

    let non_empty_arr = Subset::<IsNonEmpty, [u8; 3]>::try_new(arr).unwrap();
    assert_eq!(*First::call(&non_empty_arr), 1);
    assert_eq!(*Last::call(&non_empty_arr), 3);

    let len_gt_1 = Subset::<LenGt<1>, [u8]>::try_from_ref(&arr[..]).unwrap();
    assert_eq!(*Get::<1>::call(len_gt_1), 2);
    assert!(Subset::<LenGt<3>, [u8]>::try_from_ref(&arr[..]).is_none());

    // Nested refinement: non-empty and first byte is an ASCII letter (directly via Then<First, IsAsciiAlphabetic>)
    type StartsWithAlpha = Subset<Then<First, IsAsciiAlphabetic>, Subset<IsNonEmpty, [u8]>>;
    let alpha_bytes = Subset::<IsNonEmpty, [u8]>::try_from_ref(b"Abc").unwrap();
    let digit_bytes = Subset::<IsNonEmpty, [u8]>::try_from_ref(b"1bc").unwrap();
    assert!(StartsWithAlpha::try_from_ref(alpha_bytes).is_some());
    assert!(StartsWithAlpha::try_from_ref(digit_bytes).is_none());

    #[cfg(feature = "alloc")]
    {
        extern crate alloc;
        use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};
        use fy_refine_core::function::wiring::Snd;

        let non_empty_vec = Subset::<IsNonEmpty, Vec<u8>>::try_new(vec![10, 20, 30]).unwrap();
        assert_eq!(*First::call(&non_empty_vec), 10);
        assert_eq!(*Last::call(&non_empty_vec), 30);

        let vec_gt_2 = Subset::<LenGt<2>, Vec<u8>>::try_new(vec![10, 20, 30]).unwrap();
        assert_eq!(*Get::<2>::call(&vec_gt_2), 30);

        // Non-contiguous containers: VecDeque / LinkedList / BinaryHeap / BTreeSet / BTreeMap
        let mut dq = VecDeque::from([1i32, 2, 3]);
        dq.push_front(4);
        assert_eq!(Len::call(&dq), 4);
        assert!(IsNonEmpty::call(&dq));
        assert!(All::<IsPositive>::call(&dq));
        assert_eq!(TryFirst::call(&dq), Some(&4));
        assert_eq!(TryLast::call(&dq), Some(&3));
        assert_eq!(TryGet::<2>::call(&dq), Some(&2));
        assert_eq!(TryGet::<9>::call(&dq), None);
        let non_empty_dq = Subset::<IsNonEmpty, VecDeque<i32>>::try_new(dq.clone()).unwrap();
        assert_eq!(*First::call(&non_empty_dq), 4);
        assert_eq!(*Last::call(&non_empty_dq), 3);
        let gt_2_dq = Subset::<LenGt<2>, VecDeque<i32>>::try_new(dq.clone()).unwrap();
        assert_eq!(*Get::<2>::call(&gt_2_dq), 2);
        assert!(Subset::<All<IsPositive>, VecDeque<i32>>::try_new(dq).is_ok());

        let list = LinkedList::from([-1i32, 0, 5]);
        assert_eq!(Len::call(&list), 3);
        assert!(!All::<IsPositive>::call(&list));
        assert!(Any::<IsPositive>::call(&list));
        assert_eq!(TryFirst::call(&list), Some(&-1));
        assert_eq!(TryLast::call(&list), Some(&5));
        let non_empty_list = Subset::<IsNonEmpty, LinkedList<i32>>::try_new(list).unwrap();
        assert_eq!(*First::call(&non_empty_list), -1);
        assert_eq!(*Last::call(&non_empty_list), 5);

        let heap = BinaryHeap::from([1i32, 5, 3]);
        assert_eq!(Len::call(&heap), 3);
        assert!(All::<IsPositive>::call(&heap));

        let bset = BTreeSet::from([2i32, 4, 6]);
        assert_eq!(Len::call(&bset), 3);
        assert!(All::<IsPositive>::call(&bset));
        assert_eq!(TryFirst::call(&bset), Some(&2));
        assert_eq!(TryLast::call(&bset), Some(&6));
        let non_empty_bset = Subset::<IsNonEmpty, BTreeSet<i32>>::try_new(bset).unwrap();
        assert_eq!(*First::call(&non_empty_bset), 2);
        assert_eq!(*Last::call(&non_empty_bset), 6);

        let bmap = BTreeMap::from([("a", 1i32), ("b", 2), ("c", 3)]);
        assert_eq!(Len::call(&bmap), 3);
        assert!(All::<Then<Snd, IsPositive>>::call(&bmap));
        assert_eq!(TryFirst::call(&bmap), Some((&"a", &1)));
        assert_eq!(TryLast::call(&bmap), Some((&"c", &3)));
        let non_empty_bmap =
            Subset::<IsNonEmpty, BTreeMap<&str, i32>>::try_new(bmap.clone()).unwrap();
        assert_eq!(First::call(&non_empty_bmap), (&"a", &1));
        assert_eq!(Last::call(&non_empty_bmap), (&"c", &3));
        assert!(Subset::<All<Then<Snd, IsPositive>>, BTreeMap<&str, i32>>::try_new(bmap).is_ok());
    }

    #[cfg(feature = "std")]
    {
        use fy_refine_core::function::wiring::Snd;
        use std::collections::{HashMap, HashSet};

        let hset = HashSet::from([10i32, 20, 30]);
        assert_eq!(Len::call(&hset), 3);
        assert!(IsNonEmpty::call(&hset));
        assert!(All::<IsPositive>::call(&hset));
        assert!(Subset::<All<IsPositive>, HashSet<i32>>::try_new(hset).is_ok());

        let hmap = HashMap::from([("k1", 1i32), ("k2", -5)]);
        assert_eq!(Len::call(&hmap), 2);
        assert!(!All::<Then<Snd, IsPositive>>::call(&hmap));
        assert!(Any::<Then<Snd, IsPositive>>::call(&hmap));
        assert!(Subset::<Any<Then<Snd, IsPositive>>, HashMap<&str, i32>>::try_new(hmap).is_ok());
    }

    let empty: &[u8] = &[];
    assert!(IsEmpty::call(empty));
    assert!(!IsNonEmpty::call(empty));
    assert_eq!(TryFirst::call(empty), None);
    assert_eq!(TryLast::call(empty), None);
    assert!(Subset::<IsNonEmpty, [u8]>::try_from_ref(empty).is_none());
    assert!(All::<IsPositive>::call(empty));
    assert!(!Any::<IsPositive>::call(empty));

    // Additional existing type checks
    fn accepts_subset<P, T>(_: Phantom<Subset<P, T>>)
    where
        P: for<'a> Predicate<&'a T>,
        T: SafeValue,
    {
    }
    accepts_subset::<IsNonZero, usize>(core::marker::PhantomData);
    accepts_subset::<IsNonNegative, i64>(core::marker::PhantomData);
    accepts_subset::<IsZero, u8>(core::marker::PhantomData);
}

#[cfg(feature = "nightly")]
#[test]
fn nightly_const_subset_construction_and_penetration() {
    type Percent = Subset<Range<Closed<ConstU8<0>>, Closed<ConstU8<100>>>, u8>;
    type AsciiBytes = Subset<All<IsAscii>, [u8]>;

    const VALID_CHECK: bool = Percent::is_valid(&80);
    const OK_VAL: Result<Percent, u8> = Percent::try_new(80);
    const ERR_VAL: Result<Percent, u8> = Percent::try_new(120);
    const OK_BASE: u8 = match OK_VAL {
        Ok(v) => {
            Subset::assume_valid(&v);
            Subset::into_base(v)
        }
        Err(_) => 0,
    };
    const COPIED_AND_CMP: (u8, bool) = match (OK_VAL, Percent::try_new(90)) {
        (Ok(v), Ok(hi)) => (*AsBase::call(&v), Lt::call((v, hi))),
        _ => (0, false),
    };
    const OK_REF: Option<&AsciiBytes> = AsciiBytes::try_from_ref(b"abc");
    const BAD_REF: Option<&AsciiBytes> = AsciiBytes::try_from_ref(&[0xff]);
    const NON_EMPTY_FIRST: u8 = match Subset::<IsNonEmpty, [u8]>::try_from_ref(b"xyz") {
        Some(s) => *First::call(s),
        None => 0,
    };
    const NON_EMPTY_LAST: u8 = match Subset::<IsNonEmpty, [u8]>::try_from_ref(b"xyz") {
        Some(s) => *Last::call(s),
        None => 0,
    };
    const INDEXED_SECOND: u8 = match Subset::<LenGt<1>, [u8]>::try_from_ref(b"xyz") {
        Some(s) => *Get::<1>::call(s),
        None => 0,
    };

    const {
        assert!(VALID_CHECK);
    }
    assert!(OK_VAL.is_ok());
    assert_eq!(ERR_VAL, Err(120));
    assert_eq!(OK_BASE, 80);
    assert_eq!(COPIED_AND_CMP, (80, true));
    assert!(OK_REF.is_some());
    assert!(BAD_REF.is_none());
    assert_eq!(NON_EMPTY_FIRST, b'x');
    assert_eq!(NON_EMPTY_LAST, b'z');
    assert_eq!(INDEXED_SECOND, b'y');
}

#[test]
fn zero_unsafe_macros_and_relaxed_subset_comparisons() {
    type PosI32 = Subset<IsPositive, i32>;
    type EvenI32 = Subset<IsEven, i32>;

    // 1. impl_safe_value! / impl_safe_drop! / impl_safe_clone! support named, tuple, and unit structs as well as enums (zero unsafe)
    #[derive(Debug, PartialEq, Eq)]
    struct Window {
        lo: PosI32,
        hi: PosI32,
        tag: &'static str,
        flags: u32,
    }
    impl_safe_value!(Window { lo, hi, tag, flags });
    impl_safe_drop!(Window { lo, hi, tag, flags });
    impl_safe_clone!(Window { lo, hi, tag, flags });

    #[derive(Copy, Debug, PartialEq, Eq)]
    struct TupleBox(PosI32, u8);
    impl_safe_value!(TupleBox(a, b));
    impl_safe_drop!(TupleBox(a, b));
    impl_safe_clone!(TupleBox(a, b));

    #[derive(Copy, Debug, PartialEq, Eq)]
    struct EmptyMarker;
    impl_safe_value!(EmptyMarker);
    impl_safe_drop!(EmptyMarker);
    impl_safe_clone!(EmptyMarker);

    // Verify unit structs also support Fst / WithRef / Copy / Clone
    #[derive(Debug, PartialEq, Eq)]
    enum CustomPacket {
        Header { len: PosI32, tag: &'static str },
        Data(u8, u16),
        End,
    }
    impl_safe_value!(enum CustomPacket {
        Header { len, tag },
        Data(a, b),
        End,
    });
    impl_safe_drop!(enum CustomPacket {
        Header { len, tag },
        Data(a, b),
        End,
    });
    impl_safe_clone!(enum CustomPacket {
        Header { len, tag },
        Data(a, b),
        End,
    });

    // 2. field_projector! supports reference projection, Copy value projection, explicit lifetime references, and tuple indices (zero unsafe)
    field_projector!(GetWinLo: &Window => lo: &PosI32);
    field_projector!(GetWinHi: &Window => hi: &PosI32);
    field_projector!(GetWinTag: &Window => tag: &'static str);
    field_projector!(GetWinFlags: &Window => flags: u32);
    field_projector!(GetTupleFirst: &TupleBox => 0: &PosI32);

    type ValidWindow = Subset<And<ForkLt<GetWinLo, GetWinHi>, Then<GetWinFlags, IsEven>>, Window>;
    let w_ok = ValidWindow::try_new(Window {
        lo: PosI32::try_new(2).unwrap(),
        hi: PosI32::try_new(9).unwrap(),
        tag: "ok",
        flags: 4,
    })
    .unwrap();
    assert_eq!(**GetWinLo::call(&w_ok), 2);
    assert_eq!(**GetWinHi::call(&w_ok), 9);
    assert_eq!(GetWinTag::call(&w_ok), "ok");
    assert_eq!(GetWinFlags::call(&w_ok), 4);
    assert_eq!(w_ok.clone(), w_ok);
    assert!(WithRef::<Then<AsInner, Then<GetWinFlags, IsEven>>>::call(
        w_ok.clone()
    ));
    assert_eq!(Fst::call((w_ok.clone(), w_ok.clone())), w_ok);
    Discard::call(w_ok.clone());
    assert!(
        ValidWindow::try_new(Window {
            lo: PosI32::try_new(10).unwrap(),
            hi: PosI32::try_new(5).unwrap(),
            tag: "bad",
            flags: 4,
        })
        .is_err()
    );

    let tb = Subset::<Then<GetTupleFirst, GtConstPos>, TupleBox>::try_new(TupleBox(
        PosI32::try_new(5).unwrap(),
        1,
    ));
    struct GtConstPos;
    impl Function<PosI32> for GtConstPos {
        type Output = bool;
        fn call(v: PosI32) -> bool {
            *v > 3
        }
    }
    unsafe impl SafeFunction<PosI32> for GtConstPos {}
    impl<'a> Function<&'a PosI32> for GtConstPos {
        type Output = bool;
        fn call(v: &'a PosI32) -> bool {
            **v > 3
        }
    }
    unsafe impl SafeFunction<&PosI32> for GtConstPos {}
    let tb_ok = tb.unwrap();
    let tb_copy = tb_ok;
    assert_eq!(Cloned::call(&tb_copy), tb_ok);

    struct Yes;
    impl Function<&EmptyMarker> for Yes {
        type Output = bool;
        fn call(_: &EmptyMarker) -> bool {
            true
        }
    }
    unsafe impl SafeFunction<&EmptyMarker> for Yes {}
    let empty_sub = Subset::<Yes, EmptyMarker>::try_new(EmptyMarker).unwrap();
    let empty_copy = empty_sub;
    assert_eq!(Subset::into_base(Cloned::call(&empty_copy)), EmptyMarker);
    assert_eq!(Subset::into_base(empty_sub), EmptyMarker);

    struct PacketOk;
    impl Function<&CustomPacket> for PacketOk {
        type Output = bool;
        fn call(_: &CustomPacket) -> bool {
            true
        }
    }
    unsafe impl SafeFunction<&CustomPacket> for PacketOk {}
    let pkt = Subset::<PacketOk, CustomPacket>::try_new(CustomPacket::Header {
        len: PosI32::try_new(8).unwrap(),
        tag: "hdr",
    })
    .unwrap();
    assert_eq!(pkt.clone(), pkt);
    Discard::call(pkt);
    assert_eq!(CustomPacket::Data(1, 2).clone(), CustomPacket::Data(1, 2));
    assert_eq!(CustomPacket::End.clone(), CustomPacket::End);

    #[cfg(feature = "alloc")]
    {
        extern crate alloc;
        use alloc::string::String;

        #[derive(Debug, PartialEq, Eq)]
        enum CustomMsg {
            Text { body: String, id: PosI32 },
            Ping(u16),
            Quit,
        }
        impl_safe_value!(enum CustomMsg {
            Text { body, id },
            Ping(code),
            Quit,
        });
        impl_safe_drop!(enum CustomMsg {
            Text { body, id },
            Ping(code),
            Quit,
        });
        impl_safe_clone!(enum CustomMsg {
            Text { body, id },
            Ping(code),
            Quit,
        });

        struct MsgOk;
        impl Function<&CustomMsg> for MsgOk {
            type Output = bool;
            fn call(_: &CustomMsg) -> bool {
                true
            }
        }
        unsafe impl SafeFunction<&CustomMsg> for MsgOk {}

        let msg = Subset::<MsgOk, CustomMsg>::try_new(CustomMsg::Text {
            body: String::from("hello"),
            id: PosI32::try_new(1).unwrap(),
        })
        .unwrap();
        assert_eq!(msg.clone(), msg);
        Discard::call(msg);
    }

    // 3. Relaxed cross-predicate and cross-base PartialEq and PartialOrd
    let p4 = PosI32::try_new(4).unwrap();
    let e4 = EvenI32::try_new(4).unwrap();
    let e6 = EvenI32::try_new(6).unwrap();
    assert_eq!(p4, e4);
    assert!(p4 < e6);

    #[cfg(feature = "alloc")]
    {
        extern crate alloc;
        use alloc::string::String;

        let s_owned = Subset::<IsNonEmpty, String>::try_new(String::from("abc")).unwrap();
        let s_slice = Subset::<IsNonEmpty, str>::try_from_ref("abc").unwrap();
        assert_eq!(s_owned, *s_slice);
    }

    // 4. Single-sided reference SafeEq / SafeOrd and constant comparison predicates on Subset
    struct ConstPos5;
    impl Function<()> for ConstPos5 {
        type Output = PosI32;
        fn call((): ()) -> PosI32 {
            PosI32::try_new(5).unwrap()
        }
    }
    unsafe impl SafeFunction<()> for ConstPos5 {}

    type PosLt5 = Subset<LtConst<ConstPos5>, PosI32>;
    assert!(PosLt5::try_new(p4).is_ok());
    assert!(PosLt5::try_new(PosI32::try_new(5).unwrap()).is_err());
    assert!(SafeOrd::lt(&&p4, &e6));
    assert!(SafeEq::eq(&p4, &&e4));
}

#[cfg(feature = "nightly")]
#[test]
fn nightly_const_macros_work_in_const_eval() {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    struct ConstWin {
        lo: i32,
        hi: i32,
    }
    impl_safe_value!(const ConstWin { lo, hi });
    field_projector!(const WinLo: &ConstWin => lo: i32);
    field_projector!(const WinHi: &ConstWin => hi: i32);

    type ValidConstWin = Subset<ForkLt<WinLo, WinHi>, ConstWin>;
    const OK: Result<ValidConstWin, ConstWin> = ValidConstWin::try_new(ConstWin { lo: 1, hi: 2 });
    const BAD: Result<ValidConstWin, ConstWin> = ValidConstWin::try_new(ConstWin { lo: 5, hi: 2 });
    assert!(OK.is_ok());
    assert!(BAD.is_err());

    const CODE_OK: Result<Subset<IsChar, u32>, u32> = Subset::<IsChar, u32>::try_new(0x1F980);
    const CODE_BAD: Result<Subset<IsChar, u32>, u32> = Subset::<IsChar, u32>::try_new(0xD800);
    const CONVERTED_CHAR: char = match CODE_OK {
        Ok(c) => U32ToChar::call(c),
        Err(_) => '?',
    };
    const BACK_U32: u32 = Subset::into_inner(CharToU32::call('🦀'));
    const UTF8_VIEW: &str = match Subset::<All<IsAscii>, [u8]>::try_from_ref(b"const-ok") {
        Some(s) => AsStr::call(s),
        None => "",
    };
    assert!(CODE_BAD.is_err());
    assert_eq!(CONVERTED_CHAR, '🦀');
    assert_eq!(BACK_U32, 0x1F980);
    assert_eq!(UTF8_VIEW, "const-ok");
}

#[test]
fn unicode_scalar_utf8_and_slice_subset_safe_ord() {
    // 1. IsChar / U32ToChar / CharToU32 branch-free roundtrip and surrogate rejection
    for valid in [0x0000u32, b'A' as u32, 0xD7FF, 0xE000, 0x00E9, 0x10_FFFF] {
        let code = Subset::<IsChar, u32>::try_new(valid).unwrap();
        let ch = U32ToChar::call(&code);
        assert_eq!(U32ToChar::call(code), ch);
        let rt = CharToU32::call(&ch);
        assert_eq!(*rt, valid);
        assert_eq!(CharToU32::call(ch), rt);
    }
    for surrogate_or_overflow in [0xD800u32, 0xDBFF, 0xDFFF, 0x11_0000, u32::MAX] {
        assert!(Subset::<IsChar, u32>::try_new(surrogate_or_overflow).is_err());
    }

    // 2. IsUtf8 and AsStr zero-copy Result-free conversion to &str (supports All<IsAscii> and IsUtf8)
    let ascii = Subset::<All<IsAscii>, [u8]>::try_from_ref(b"hello").unwrap();
    assert_eq!(AsStr::call(ascii), "hello");

    let arr = Subset::<All<IsAscii>, [u8; 3]>::try_from_ref(b"abc").unwrap();
    assert_eq!(AsStr::call(arr), "abc");

    let utf8_slice = Subset::<IsUtf8, [u8]>::try_from_ref("café🦀".as_bytes()).unwrap();
    assert_eq!(AsStr::call(utf8_slice), "café🦀");

    let bad_utf8: &[u8] = &[0xff, 0xfe, 0x80];
    assert!(Subset::<IsUtf8, [u8]>::try_from_ref(bad_utf8).is_none());

    #[cfg(feature = "alloc")]
    {
        extern crate alloc;
        use alloc::vec::Vec;

        let ascii_vec = Subset::<All<IsAscii>, Vec<u8>>::try_new(b"vec".to_vec()).unwrap();
        assert_eq!(AsStr::call(&ascii_vec), "vec");

        let utf8_vec = Subset::<IsUtf8, Vec<u8>>::try_new("héllo🦀".as_bytes().to_vec()).unwrap();
        assert_eq!(AsStr::call(&utf8_vec), "héllo🦀");
    }

    // 3. Slice and array Subset SafeEq / SafeOrd comparisons
    let b1 = Subset::<IsNonEmpty, [u8]>::try_from_ref(b"ab").unwrap();
    let b2 = Subset::<LenGt<1>, [u8]>::try_from_ref(b"ac").unwrap();
    assert!(SafeOrd::lt(b1, b2));
    assert!(SafeEq::eq(b1, &b"ab"[..]));

    let a1 = Subset::<IsNonEmpty, [u8; 2]>::try_from_ref(b"ab").unwrap();
    let a2 = Subset::<IsNonEmpty, [u8; 2]>::try_from_ref(b"ac").unwrap();
    assert!(SafeOrd::lt(a1, a2));
}

fn assert_ord_all<A: ?Sized + SafeOrd<B>, B: ?Sized>(a: &A, b: &B, expected: Ordering) {
    assert_eq!(SafeOrd::cmp(a, b), expected);
    assert_eq!(SafeEq::eq(a, b), expected == Ordering::Equal);
    assert_eq!(SafeEq::ne(a, b), expected != Ordering::Equal);
    assert_eq!(SafeOrd::lt(a, b), expected == Ordering::Less);
    assert_eq!(SafeOrd::le(a, b), expected != Ordering::Greater);
    assert_eq!(SafeOrd::gt(a, b), expected == Ordering::Greater);
    assert_eq!(SafeOrd::ge(a, b), expected != Ordering::Less);
}

#[test]
fn subset_cmp_all_ops_std_traits_and_safe_value_as_base() {
    use core::borrow::Borrow;
    use core::fmt::Write;
    use core::hash::{Hash, Hasher};
    use core::marker::PhantomData;
    use core::ops::{ControlFlow, RangeFrom, RangeTo, RangeToInclusive};
    use fy_refine_core::function::wiring::Copied;
    use fy_refine_core::subset::{FiniteF32, FiniteF64, NotNaNF32, NotNaNF64};

    // 1. Complete comparisons across int / ref / slice / array / float Subsets (including single-sided refs and reversed scalar interop)
    type PosI32 = Subset<IsPositive, i32>;
    type EvenI32 = Subset<IsEven, i32>;
    type PosU64 = Subset<IsPositive, u64>;

    let p4 = PosI32::try_new(4).unwrap();
    let p5 = PosI32::try_new(5).unwrap();
    let e4 = EvenI32::try_new(4).unwrap();
    let e6 = EvenI32::try_new(6).unwrap();
    let pos_u64 = PosU64::try_new(5).unwrap();

    assert_ord_all(&p4, &e4, Ordering::Equal);
    assert_ord_all(&p4, &e6, Ordering::Less);
    assert_ord_all(&&p4, &e4, Ordering::Equal);
    assert_ord_all(&&p4, &e6, Ordering::Less);
    assert_ord_all(&p4, &4i32, Ordering::Equal);
    assert_ord_all(&p4, &6i32, Ordering::Less);
    assert_ord_all(&&p4, &6i32, Ordering::Less);
    assert_ord_all(&4i32, &p4, Ordering::Equal);
    assert_ord_all(&2i32, &p4, Ordering::Less);
    assert_ord_all(&2i32, &&p4, Ordering::Less);
    assert_ord_all(&&2i32, &p4, Ordering::Less);
    assert_ord_all(&6i32, &p4, Ordering::Greater);

    // Reference-carrier subset: SubsetCarrierEq/Ord for &T
    let four = 4i32;
    let ref_pos = Subset::<Then<Copied, IsPositive>, &i32>::try_new(&four).unwrap();
    assert_ord_all(&ref_pos, &4i32, Ordering::Equal);
    assert_ord_all(&ref_pos, &6i32, Ordering::Less);

    // Slice-carrier subset: Subset vs [u8] / &[u8] / Subset and reversed comparisons
    let b1 = Subset::<IsNonEmpty, [u8]>::try_from_ref(b"ab").unwrap();
    let b2 = Subset::<LenGt<1>, [u8]>::try_from_ref(b"ac").unwrap();
    assert_ord_all(b1, &b"ab"[..], Ordering::Equal);
    assert_ord_all(b1, &b"ac"[..], Ordering::Less);
    assert_ord_all(b1, &&b"ac"[..], Ordering::Less);
    assert_ord_all(b1, b2, Ordering::Less);
    assert_ord_all(&b1, b2, Ordering::Less);
    assert_ord_all(&b"ab"[..], b2, Ordering::Less);
    assert_ord_all(&b"ab"[..], &b2, Ordering::Less);
    assert_ord_all(&&b"ab"[..], b2, Ordering::Less);

    // Array-carrier subset: Subset vs [u8; 2] / &[u8; 2] / Subset and reversed comparisons
    let a1 = Subset::<IsNonEmpty, [u8; 2]>::try_new(*b"ab").unwrap();
    let a2 = Subset::<IsNonEmpty, [u8; 2]>::try_new(*b"ac").unwrap();
    assert_ord_all(&a1, b"ab", Ordering::Equal);
    assert_ord_all(&a1, b"ac", Ordering::Less);
    assert_ord_all(&a1, &b"ac", Ordering::Less);
    assert_ord_all(&a1, &a2, Ordering::Less);
    assert_ord_all(&&a1, &a2, Ordering::Less);
    assert_ord_all(b"ab", &a2, Ordering::Less);
    assert_ord_all(b"ab", &&a2, Ordering::Less);
    assert_ord_all(&b"ab", &a2, Ordering::Less);

    // Float-carrier subset interop, exact comparison boundary branches, and bidirectional scalar comparisons
    let nn32_15 = NotNaNF32::try_new(1.5).unwrap();
    let nn32_25 = NotNaNF32::try_new(2.5).unwrap();
    let fin32_25 = FiniteF32::try_new(nn32_25).unwrap();
    let nn64_15 = NotNaNF64::try_new(1.5).unwrap();
    let nn64_25 = NotNaNF64::try_new(2.5).unwrap();
    let fin64_25 = FiniteF64::try_new(nn64_25).unwrap();

    assert_ord_all(&nn32_15, &nn64_25, Ordering::Less);
    assert_ord_all(&nn32_25, &nn64_15, Ordering::Greater);
    assert_ord_all(&nn32_15, &nn64_15, Ordering::Equal);
    assert_ord_all(&nn64_15, &nn32_15, Ordering::Equal);
    assert_ord_all(&nn64_15, &fin64_25, Ordering::Less);
    assert_ord_all(&nn32_15, &fin32_25, Ordering::Less);
    assert_ord_all(&fin64_25, &nn32_15, Ordering::Greater);
    assert_ord_all(&nn64_15, &&nn32_15, Ordering::Equal);

    assert_ord_all(&nn32_15, &5i32, Ordering::Less);
    assert_ord_all(&nn32_15, &&5i32, Ordering::Less);
    assert_ord_all(&nn32_15, &5u64, Ordering::Less);
    assert_ord_all(&nn32_15, &&5u64, Ordering::Less);
    assert_ord_all(&nn32_15, &p4, Ordering::Less);
    assert_ord_all(&nn32_15, &pos_u64, Ordering::Less);
    assert_ord_all(&nn64_15, &5i32, Ordering::Less);
    assert_ord_all(&nn64_15, &&5i32, Ordering::Less);
    assert_ord_all(&nn64_15, &5u64, Ordering::Less);
    assert_ord_all(&nn64_15, &&5u64, Ordering::Less);
    assert_ord_all(&nn64_15, &p4, Ordering::Less);
    assert_ord_all(&nn64_15, &pos_u64, Ordering::Less);

    // Remaining cmp_non_nan_f64_u128 branches (f_int > n, f_floor < n, f_floor > n)
    let f_pow60 = NotNaNF64::try_new((1u128 << 60) as f64).unwrap();
    let f_25 = NotNaNF64::try_new(2.5).unwrap();
    assert_ord_all(&f_pow60, &1u128, Ordering::Greater);
    assert_ord_all(&nn64_15, &2u128, Ordering::Less);
    assert_ord_all(&f_25, &1u128, Ordering::Greater);
    assert_ord_all(&f_25, &2u128, Ordering::Greater);

    // 2. All forwarding trait methods in subset/std_traits.rs (AsRef, Borrow, Ord, Hash, Display, Debug)
    let as_r: &i32 = p4.as_ref();
    let brw: &i32 = p4.borrow();
    assert_eq!((*as_r, *brw), (4, 4));
    assert_eq!(core::cmp::Ord::cmp(&p4, &p4), Ordering::Equal);
    assert_eq!(core::cmp::Ord::cmp(&p4, &p5), Ordering::Less);

    struct SumHasher(u64);
    impl Hasher for SumHasher {
        fn finish(&self) -> u64 {
            self.0
        }
        fn write(&mut self, bytes: &[u8]) {
            for &b in bytes {
                self.0 = self.0.wrapping_mul(31).wrapping_add(u64::from(b));
            }
        }
    }
    let mut h1 = SumHasher(0);
    p4.hash(&mut h1);
    let mut h2 = SumHasher(0);
    4i32.hash(&mut h2);
    assert_eq!(h1.finish(), h2.finish());

    struct FmtBuf {
        buf: [u8; 64],
        len: usize,
    }
    impl FmtBuf {
        fn as_str(&self) -> &str {
            core::str::from_utf8(&self.buf[..self.len]).unwrap()
        }
    }
    impl Write for FmtBuf {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let bytes = s.as_bytes();
            let next = self.len + bytes.len();
            if next > self.buf.len() {
                return Err(core::fmt::Error);
            }
            self.buf[self.len..next].copy_from_slice(bytes);
            self.len = next;
            Ok(())
        }
    }
    let mut disp_buf = FmtBuf {
        buf: [0; 64],
        len: 0,
    };
    write!(&mut disp_buf, "{p4}").unwrap();
    assert_eq!(disp_buf.as_str(), "4");

    let mut dbg_buf = FmtBuf {
        buf: [0; 64],
        len: 0,
    };
    write!(&mut dbg_buf, "{p4:?}").unwrap();
    assert_eq!(dbg_buf.as_str(), "Subset(4)");

    // 3. Full SafeValue::as_base coverage across primitive, alloc, and std types plus missed slice/UTF-8 paths
    assert_eq!(<() as SafeValue>::as_base(&()), &());
    assert_eq!(<bool as SafeValue>::as_base(&true), &true);
    assert_eq!(<char as SafeValue>::as_base(&'z'), &'z');
    assert_eq!(<u8 as SafeValue>::as_base(&7), &7);
    assert_eq!(
        <NonZeroU8 as SafeValue>::as_base(&NonZeroU8::MIN),
        &NonZeroU8::MIN
    );
    assert_eq!(
        <Ordering as SafeValue>::as_base(&Ordering::Less),
        &Ordering::Less
    );
    assert_eq!(
        <Duration as SafeValue>::as_base(&Duration::ZERO),
        &Duration::ZERO
    );
    assert_eq!(<RangeFull as SafeValue>::as_base(&(..)), &(..));
    assert_eq!(<str as SafeValue>::as_base("fy"), "fy");
    assert_eq!(<&i32 as SafeValue>::as_base(&&9i32), &&9i32);
    assert_eq!(
        <PhantomData<i32> as SafeValue>::as_base(&PhantomData),
        &PhantomData::<i32>
    );
    assert_eq!(<[u8] as SafeValue>::as_base(&[1u8, 2][..]), &[1u8, 2]);
    assert_eq!(
        <Wrapping<u32> as SafeValue>::as_base(&Wrapping(3)),
        &Wrapping(3)
    );
    assert_eq!(
        <Saturating<i64> as SafeValue>::as_base(&Saturating(-1)),
        &Saturating(-1)
    );
    assert_eq!(<Option<i32> as SafeValue>::as_base(&Some(1)), &Some(1));
    assert_eq!(
        <Result<u8, i8> as SafeValue>::as_base(&Ok(2)),
        &Ok::<u8, i8>(2)
    );
    assert_eq!(
        <ControlFlow<u8, i8> as SafeValue>::as_base(&ControlFlow::Continue(3)),
        &ControlFlow::<u8, i8>::Continue(3)
    );
    assert_eq!(<[u8; 2] as SafeValue>::as_base(&[4, 5]), &[4, 5]);
    assert_eq!(
        <core::ops::Range<usize> as SafeValue>::as_base(&(1..3)),
        &(1..3)
    );
    assert_eq!(<RangeFrom<usize> as SafeValue>::as_base(&(1..)), &(1..));
    assert_eq!(<RangeTo<usize> as SafeValue>::as_base(&(..3)), &(..3));
    assert_eq!(
        <RangeInclusive<u32> as SafeValue>::as_base(&(1..=3)),
        &(1..=3)
    );
    assert_eq!(
        <RangeToInclusive<usize> as SafeValue>::as_base(&(..=3)),
        &(..=3)
    );
    assert_eq!(<(u8,) as SafeValue>::as_base(&(1,)), &(1,));
    assert_eq!(<(u8, u16) as SafeValue>::as_base(&(1, 2)), &(1, 2));

    assert!(IsUtf8::call(b"abc"));
    let utf8_arr = Subset::<IsUtf8, [u8; 3]>::try_new(*b"abc").unwrap();
    assert_eq!(AsStr::call(&utf8_arr), "abc");
    assert_eq!(TryGet::<1>::call(&[10u8, 20][..]), Some(&20));
    let arr_gt1 = Subset::<LenGt<1>, [u8; 3]>::try_new([1, 2, 3]).unwrap();
    assert_eq!(*Get::<1>::call(&arr_gt1), 2);

    #[cfg(feature = "alloc")]
    {
        extern crate alloc;
        use alloc::borrow::Cow;
        use alloc::boxed::Box;
        use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};
        use alloc::rc::Rc;
        use alloc::string::String;
        use alloc::sync::Arc;
        use alloc::vec::Vec;
        use fy_refine_core::function::wiring::Snd;

        assert_eq!(<String as SafeValue>::as_base(&String::from("a")), "a");
        assert_eq!(**(<Box<i32> as SafeValue>::as_base(&Box::new(1))), 1);
        assert_eq!(<Vec<u8> as SafeValue>::as_base(&vec![1]), &vec![1]);
        assert_eq!(
            <VecDeque<u8> as SafeValue>::as_base(&VecDeque::from([1])),
            &VecDeque::from([1])
        );
        assert_eq!(
            <LinkedList<u8> as SafeValue>::as_base(&LinkedList::from([1])),
            &LinkedList::from([1])
        );
        assert_eq!(
            (<BinaryHeap<u8> as SafeValue>::as_base(&BinaryHeap::from([1]))).peek(),
            Some(&1)
        );
        assert_eq!(
            <BTreeSet<u8> as SafeValue>::as_base(&BTreeSet::from([1])),
            &BTreeSet::from([1])
        );
        let bmap = BTreeMap::from([("a", 1i32), ("b", -2)]);
        assert_eq!(<BTreeMap<&str, i32> as SafeValue>::as_base(&bmap), &bmap);
        assert_eq!(
            <Cow<'static, str> as SafeValue>::as_base(&Cow::Borrowed("c")),
            "c"
        );
        assert_eq!(
            <Cow<'static, [u8]> as SafeValue>::as_base(&Cow::Borrowed(&[1u8][..])),
            &Cow::Borrowed(&[1u8][..])
        );
        assert_eq!(**<Rc<i32> as SafeValue>::as_base(&Rc::new(2)), 2);
        assert_eq!(**<Arc<i32> as SafeValue>::as_base(&Arc::new(3)), 3);

        let s_hi = String::from("hi");
        assert_eq!(AsBytes::call(&s_hi), b"hi");
        assert_eq!(Len::call(&s_hi), 2);
        assert!(!IsEmpty::call(&s_hi));
        let v_nums = vec![10u8, 20, 30];
        assert_eq!(AsSlice::call(&v_nums), &[10u8, 20, 30]);
        assert_eq!(TryFirst::call(&v_nums), Some(&10));
        assert_eq!(TryLast::call(&v_nums), Some(&30));
        assert_eq!(TryGet::<1>::call(&v_nums), Some(&20));
        assert!(Any::<Then<Snd, IsPositive>>::call(&bmap));
    }

    #[cfg(feature = "std")]
    {
        use std::collections::{HashMap, HashSet};
        use std::hash::RandomState;

        let rs = RandomState::new();
        let _ = <RandomState as SafeValue>::as_base(&rs);
        let hset = HashSet::from([10i32, 20]);
        assert_eq!(<HashSet<i32> as SafeValue>::as_base(&hset), &hset);
        let hmap = HashMap::from([("k", 1i32)]);
        assert_eq!(<HashMap<&str, i32> as SafeValue>::as_base(&hmap), &hmap);
        assert!(Any::<IsPositive>::call(&hset));
    }
}
