//! Tests for integer bitwise primitives and transparent flag/mask predicate aliases.
#![cfg_attr(feature = "nightly", feature(const_trait_impl))]

use fy_refine_core::function::bitwise::{
    BitAnd, BitAndConst, BitNot, BitOr, BitOrConst, BitXor, BitXorConst, CheckedShl, CheckedShr,
    ForkBitAnd, ForkBitOr, ForkBitXor, ForkCheckedShl, ForkCheckedShr, HasAllBits, HasAnyBits,
    HasNoBits, PairBitAnd, PairBitOr, PairBitXor, PairCheckedShl, PairCheckedShr, Shl, Shr,
};
use fy_refine_core::function::constant::{ConstU8, ConstU32, ConstUsize};
use fy_refine_core::function::predicate::ForkEq;
use fy_refine_core::function::wiring::Id;
use fy_refine_core::function::{Function, SafeFunction};
use fy_refine_core::marker::SafeValue;
use fy_refine_core::subset::Subset;

#[test]
fn bitwise_primitives_and_shifts() {
    assert_eq!(BitNot::call(0x0fu8), 0xf0);
    assert_eq!(BitNot::call(&0x0fu8), 0xf0);
    assert_eq!(BitNot::call(-1i32), 0);

    assert_eq!(BitAnd::call((0b1100u8, 0b1010u8)), 0b1000);
    assert_eq!(BitAnd::call((&0b1100u8, 0b1010u8)), 0b1000);
    assert_eq!(BitAnd::call((0b1100u8, &0b1010u8)), 0b1000);
    assert_eq!(BitAnd::call((&0b1100u8, &0b1010u8)), 0b1000);

    assert_eq!(BitOr::call((0b1100u8, 0b0011u8)), 0b1111);
    assert_eq!(BitOr::call((&0b1100u8, &0b0011u8)), 0b1111);

    assert_eq!(BitXor::call((0b1100u8, 0b1010u8)), 0b0110);
    assert_eq!(BitXor::call((&0b1100u8, &0b1010u8)), 0b0110);

    assert_eq!(CheckedShl::call((1u8, 7)), Some(128));
    assert_eq!(CheckedShl::call((1u8, 8)), None);
    assert_eq!(CheckedShl::call((&1u8, 2u32)), Some(4));
    assert_eq!(CheckedShl::call((1u8, &2u32)), Some(4));
    assert_eq!(CheckedShl::call((&1u32, &32u32)), None);

    assert_eq!(CheckedShr::call((128u8, 7)), Some(1));
    assert_eq!(CheckedShr::call((128u8, 8)), None);
    assert_eq!(CheckedShr::call((&8u8, 1u32)), Some(4));
    assert_eq!(CheckedShr::call((8u8, &1u32)), Some(4));
    assert_eq!(CheckedShr::call((&-8i32, &1u32)), Some(-4));

    assert_eq!(Shl::<3>::call(1u8), 8);
    assert_eq!(Shl::<3>::call(&1u8), 8);
    assert_eq!(Shr::<2>::call(16u32), 4);
    assert_eq!(Shr::<2>::call(&-16i32), -4);
}

#[test]
fn bitwise_const_aliases_and_flag_predicates() {
    assert_eq!(BitAndConst::<ConstU8<0x0f>>::call(&0xabu8), 0x0b);
    assert_eq!(BitOrConst::<ConstU8<0x0f>>::call(0xa0u8), 0xaf);
    assert_eq!(BitXorConst::<ConstU8<0xff>>::call(&0x0fu8), 0xf0);

    // Alignment constraint (4-byte aligned: (x & 3) == 0)
    type Aligned4 = Subset<HasNoBits<ConstUsize<3>>, usize>;
    assert!(Aligned4::try_new(0).is_ok());
    assert!(Aligned4::try_new(16).is_ok());
    assert!(Aligned4::try_new(17).is_err());

    // Evenness constraint and ASCII high-bit-zero constraint
    type EvenU32 = Subset<HasNoBits<ConstU32<1>>, u32>;
    assert!(EvenU32::try_new(42).is_ok());
    assert!(EvenU32::try_new(43).is_err());

    type AsciiHighBitZero = Subset<HasNoBits<ConstU8<0x80>>, u8>;
    for b in 0u8..=255 {
        assert_eq!(AsciiHighBitZero::is_valid(&b), b.is_ascii());
    }

    // All-bits-set and any-bit-set flag matching
    type HasReadWrite = Subset<HasAllBits<ConstU8<0b0110>>, u8>;
    assert!(HasReadWrite::try_new(0b1111).is_ok());
    assert!(HasReadWrite::try_new(0b0110).is_ok());
    assert!(HasReadWrite::try_new(0b0100).is_err());

    type HasEitherFlag = Subset<HasAnyBits<ConstU8<0b0110>>, u8>;
    assert!(HasEitherFlag::try_new(0b0100).is_ok());
    assert!(HasEitherFlag::try_new(0b0010).is_ok());
    assert!(HasEitherFlag::try_new(0b1001).is_err());

    // Symmetric transparent alias verification for PairBit* and ForkBit*
    assert_eq!(PairBitAnd::<Id, Id>::call((0b1100u8, 0b1010u8)), 0b1000);
    assert_eq!(PairBitOr::<Id, Id>::call((0b1100u8, 0b0011u8)), 0b1111);
    assert_eq!(PairBitXor::<Id, Id>::call((0b1100u8, 0b1010u8)), 0b0110);
    assert_eq!(PairCheckedShl::<Id, Id>::call((1u8, 3u32)), Some(8));
    assert_eq!(PairCheckedShr::<Id, Id>::call((8u8, 2u32)), Some(2));

    struct FlagPair {
        flags: u32,
        required: u32,
    }
    unsafe impl SafeValue for FlagPair {
        type Base = Self;
        fn as_base(&self) -> &Self {
            self
        }
    }
    struct GetFlags;
    impl<'a> Function<&'a FlagPair> for GetFlags {
        type Output = u32;
        fn call(input: &'a FlagPair) -> u32 {
            input.flags
        }
    }
    unsafe impl SafeFunction<&FlagPair> for GetFlags {}
    struct GetRequired;
    impl<'a> Function<&'a FlagPair> for GetRequired {
        type Output = u32;
        fn call(input: &'a FlagPair) -> u32 {
            input.required
        }
    }
    unsafe impl SafeFunction<&FlagPair> for GetRequired {}

    // (s.flags & s.required) == s.required
    type SatisfiedFlags = Subset<ForkEq<ForkBitAnd<GetFlags, GetRequired>, GetRequired>, FlagPair>;
    assert!(
        SatisfiedFlags::try_new(FlagPair {
            flags: 0b1110,
            required: 0b0110,
        })
        .is_ok()
    );
    assert!(
        SatisfiedFlags::try_new(FlagPair {
            flags: 0b1010,
            required: 0b0110,
        })
        .is_err()
    );

    assert_eq!(
        ForkBitOr::<GetFlags, GetRequired>::call(&FlagPair {
            flags: 0b1000,
            required: 0b0011,
        }),
        0b1011
    );
    assert_eq!(
        ForkBitXor::<GetFlags, GetRequired>::call(&FlagPair {
            flags: 0b1100,
            required: 0b1010,
        }),
        0b0110
    );
    assert_eq!(
        ForkCheckedShl::<GetFlags, GetRequired>::call(&FlagPair {
            flags: 1,
            required: 4,
        }),
        Some(16)
    );
    assert_eq!(
        ForkCheckedShr::<GetFlags, GetRequired>::call(&FlagPair {
            flags: 16,
            required: 2,
        }),
        Some(4)
    );
}

#[cfg(feature = "nightly")]
#[test]
fn nightly_const_bitwise_and_flags() {
    const NOT_V: u8 = BitNot::call(0x0fu8);
    const AND_V: u8 = BitAndConst::<ConstU8<0x0f>>::call(&0xabu8);
    const SHL_V: u32 = Shl::<4>::call(1u32);
    const ALIGNED: bool = HasNoBits::<ConstUsize<7>>::call(&64usize);
    const ALL_BITS: bool = HasAllBits::<ConstU8<0b0101>>::call(&0b1101u8);

    assert_eq!(NOT_V, 0xf0);
    assert_eq!(AND_V, 0x0b);
    assert_eq!(SHL_V, 16);
    const {
        assert!(ALIGNED);
        assert!(ALL_BITS);
    }
}
