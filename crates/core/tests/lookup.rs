//! Tests for static lookup tables (`InSliceEq`, `InSortedSlice`, `InSortedRanges`) and static bitsets (`InBitset`).
#![cfg_attr(feature = "nightly", feature(const_trait_impl))]

use fy_refine_core::function::lookup::{
    InBitset, InSliceEq, InSortedRanges, InSortedSlice, bitset_u8_from_slice,
    is_sorted_disjoint_ranges, is_strictly_sorted,
};
use fy_refine_core::function::predicate::{And, Not};
use fy_refine_core::function::{Function, SafeFunction};
use fy_refine_core::static_table;
use fy_refine_core::subset::Subset;

#[test]
fn static_slice_sorted_slice_and_sorted_ranges_lookup() {
    struct SmallExclusions;
    impl Function<()> for SmallExclusions {
        type Output = &'static [&'static str];
        fn call((): ()) -> Self::Output {
            &["Qabx", "Qaaa"] // Intentionally unsorted to test linear-scan InSliceEq
        }
    }
    unsafe impl SafeFunction<()> for SmallExclusions {}

    struct ScriptTable;
    impl Function<()> for ScriptTable {
        type Output = &'static [&'static str];
        fn call((): ()) -> Self::Output {
            &["Arab", "Cyrl", "Hans", "Hant", "Latn"]
        }
    }
    unsafe impl SafeFunction<()> for ScriptTable {}

    struct ScriptRangeTable;
    impl Function<()> for ScriptRangeTable {
        type Output = &'static [(&'static str, &'static str)];
        fn call((): ()) -> Self::Output {
            &[
                ("Arab", "Arab"),
                ("Hans", "Hant"),
                ("Latn", "Latn"),
                ("Qaaa", "Qabx"),
            ]
        }
    }
    unsafe impl SafeFunction<()> for ScriptRangeTable {}

    // Validation helper functions
    assert!(is_strictly_sorted(ScriptTable::call(())));
    assert!(!is_strictly_sorted(SmallExclusions::call(())));
    assert!(!is_strictly_sorted(&["Arab", "Arab"]));

    assert!(is_sorted_disjoint_ranges(ScriptRangeTable::call(())));
    assert!(!is_sorted_disjoint_ranges(&[("Latn", "Arab")]));
    assert!(!is_sorted_disjoint_ranges(&[
        ("Hans", "Latn"),
        ("Latn", "Zzzz")
    ]));

    // 1. Linear-scan table (requires only SafeEq)
    assert!(InSliceEq::<SmallExclusions>::call("Qaaa"));
    assert!(InSliceEq::<SmallExclusions>::call("Qabx"));
    assert!(!InSliceEq::<SmallExclusions>::call("Latn"));

    // 2. Binary search over a sorted static slice
    type KnownScript = Subset<InSortedSlice<ScriptTable>, str>;
    assert!(KnownScript::try_from_ref("Arab").is_some());
    assert!(KnownScript::try_from_ref("Hans").is_some());
    assert!(KnownScript::try_from_ref("Latn").is_some());
    assert!(KnownScript::try_from_ref("Xxxx").is_none());

    // 3. Binary search over sorted closed ranges combined with an exclusion table (simulating lang-tag registry checks)
    type AllowedScript =
        Subset<And<InSortedRanges<ScriptRangeTable>, Not<InSliceEq<SmallExclusions>>>, str>;
    assert!(AllowedScript::try_from_ref("Hans").is_some());
    assert!(AllowedScript::try_from_ref("Qabc").is_some());
    assert!(AllowedScript::try_from_ref("Qaaa").is_none()); // Excluded
    assert!(AllowedScript::try_from_ref("Qabx").is_none()); // Excluded
    assert!(AllowedScript::try_from_ref("Qaby").is_none()); // Out of range

    // 4. Numeric sorted slice and range table (supporting values and references)
    struct Ports;
    impl Function<()> for Ports {
        type Output = &'static [u16];
        fn call((): ()) -> Self::Output {
            &[22, 80, 443, 8080]
        }
    }
    unsafe impl SafeFunction<()> for Ports {}
    type KnownPort = Subset<InSortedSlice<Ports>, u16>;
    assert!(KnownPort::try_new(443).is_ok());
    assert!(KnownPort::try_new(8000).is_err());
    assert!(InSortedSlice::<Ports>::call(80u16));
}

#[test]
fn static_bitset_lookup() {
    // 1. 256-bit byte bitset (exhaustive check over all 256 bytes)
    const WS_BITS: [u64; 4] = bitset_u8_from_slice(b" \t\n\x0C\r");
    let rt_bits = bitset_u8_from_slice(b" \t\n\x0C\r");
    assert_eq!(rt_bits, WS_BITS);
    struct WsBits;
    impl Function<()> for WsBits {
        type Output = &'static [u64];
        fn call((): ()) -> Self::Output {
            &WS_BITS
        }
    }
    unsafe impl SafeFunction<()> for WsBits {}

    for b in 0u8..=255 {
        assert_eq!(InBitset::<WsBits>::call(b), b.is_ascii_whitespace());
        assert_eq!(InBitset::<WsBits>::call(&b), b.is_ascii_whitespace());
    }

    type WsByte = Subset<InBitset<WsBits>, u8>;
    assert!(WsByte::try_new(b'\t').is_ok());
    assert!(WsByte::try_new(b'x').is_err());

    // 2. Arbitrary-size static bitset + out-of-bounds indices safely return false (supports u16 / u32 / usize)
    const SMALL_BITS: [u64; 2] = [(1u64 << 0) | (1u64 << 63), (1u64 << 0) | (1u64 << 10)];
    struct SmallBits;
    impl Function<()> for SmallBits {
        type Output = &'static [u64];
        fn call((): ()) -> Self::Output {
            &SMALL_BITS
        }
    }
    unsafe impl SafeFunction<()> for SmallBits {}

    assert!(InBitset::<SmallBits>::call(0usize));
    assert!(InBitset::<SmallBits>::call(63u32));
    assert!(InBitset::<SmallBits>::call(64u16));
    assert!(InBitset::<SmallBits>::call(74usize));
    assert!(!InBitset::<SmallBits>::call(1usize));
    assert!(!InBitset::<SmallBits>::call(128usize)); // Out of bounds safely returns false
    assert!(!InBitset::<SmallBits>::call(u32::MAX)); // Large out-of-bounds value safely returns false
}

#[test]
fn static_table_macro_zero_unsafe() {
    static_table!(MacroScripts: &'static [&'static str] = &["Arab", "Cyrl", "Hans", "Latn"]);
    static_table!(MacroRanges: &'static [(&'static str, &'static str)] = &[("Arab", "Arab"), ("Hans", "Hant")]);
    const BITS: [u64; 4] = bitset_u8_from_slice(b"abc");
    static_table!(MacroBits: &'static [u64] = &BITS);

    type Script = Subset<InSortedSlice<MacroScripts>, str>;
    assert!(Script::try_from_ref("Hans").is_some());
    assert!(Script::try_from_ref("Zzzz").is_none());

    assert!(InSortedRanges::<MacroRanges>::call("Hant"));
    assert!(!InSortedRanges::<MacroRanges>::call("Latn"));

    type AbcByte = Subset<InBitset<MacroBits>, u8>;
    assert!(AbcByte::try_new(b'b').is_ok());
    assert!(AbcByte::try_new(b'd').is_err());
}

#[cfg(feature = "nightly")]
#[test]
fn nightly_const_static_table_and_validators() {
    static_table!(const ConstPorts: &'static [u16] = &[22, 80, 443]);
    static_table!(const ConstRanges: &'static [(u8, u8)] = &[(1, 3), (5, 7)]);

    const SORTED: bool = is_strictly_sorted(ConstPorts::call(()));
    const RANGES_OK: bool = is_sorted_disjoint_ranges(ConstRanges::call(()));
    const HAS_443: bool = InSortedSlice::<ConstPorts>::call(443u16);
    const IN_RANGE: bool = InSortedRanges::<ConstRanges>::call(&6u8);

    const {
        assert!(SORTED);
        assert!(RANGES_OK);
        assert!(HAS_443);
        assert!(IN_RANGE);
    }
}
