//! Integer bitwise primitives and transparent combinator aliases for flag/mask predicates.
//!
//! Organized in two layers:
//! 1. **Minimal orthogonal bitwise primitives** (covering all native integers `u8..u128`, `usize`, `i8..i128`, `isize`, supporting values and references, strictly panic-free):
//!    - Bitwise logic: [`BitNot`], [`BitAnd`], [`BitOr`], [`BitXor`];
//!    - Total checked shifts: [`CheckedShl`], [`CheckedShr`] (returns `None` when out of bounds);
//!    - Compile-time statically bounds-checked fixed shifts: [`Shl`], [`Shr`] (statically forbids out-of-bounds shifts via `const { assert!(N < T::BITS) }`).
//! 2. **Transparent combinator aliases** (zero additional primitives):
//!    - Paired and forked bitwise operations: [`PairBitAnd`]..[`PairCheckedShr`], [`ForkBitAnd`]..[`ForkCheckedShr`];
//!    - Constant bitwise operations: [`BitAndConst`], [`BitOrConst`], [`BitXorConst`];
//!    - Flag and mask predicates: [`HasAllBits`], [`HasAnyBits`], [`HasNoBits`].
//!
//! ```
//! use fy_refine_core::function::Function;
//! use fy_refine_core::function::bitwise::{
//!     BitAnd, BitAndConst, BitNot, BitOr, BitXor, CheckedShl, CheckedShr, HasAllBits, HasAnyBits,
//!     HasNoBits, Shl, Shr,
//! };
//! use fy_refine_core::function::constant::{ConstU8, ConstUsize};
//! use fy_refine_core::subset::Subset;
//!
//! assert_eq!(BitNot::call(0u8), 0xff);
//! assert_eq!(BitAnd::call((0b1100u8, 0b1010u8)), 0b1000);
//! assert_eq!(BitOr::call((&0b1100u8, 0b0011u8)), 0b1111);
//! assert_eq!(BitXor::call((0b1100u8, &0b1010u8)), 0b0110);
//! assert_eq!(Shl::<2>::call(1u8), 4);
//! assert_eq!(Shr::<1>::call(&8u8), 4);
//! assert_eq!(CheckedShl::call((1u8, 8)), None);
//! assert_eq!(CheckedShr::call((8u8, 1)), Some(4));
//!
//! // 4-byte alignment check: (x & 3) == 0
//! type Aligned4 = Subset<HasNoBits<ConstUsize<3>>, usize>;
//! assert!(Aligned4::try_new(16).is_ok());
//! assert!(Aligned4::try_new(18).is_err());
//!
//! // Flag bits check: (flags & 0b0101) == 0b0101
//! type ReadyFlags = Subset<HasAllBits<ConstU8<0b0101>>, u8>;
//! assert!(ReadyFlags::try_new(0b1101).is_ok());
//! assert!(ReadyFlags::try_new(0b0100).is_err());
//! assert!(HasAnyBits::<ConstU8<0b0101>>::call(&0b0100u8));
//! assert_eq!(BitAndConst::<ConstU8<0x0f>>::call(&0xabu8), 0x0b);
//! ```

use crate::function::SafeFunction;
use crate::function::predicate::{EqConst, IsNonZero, IsZero};
use crate::function::wiring::{AttachUnit, Fork, Id, MapPair, Then};

#[cfg(feature = "nightly")]
mod bitwise_nightly;
#[cfg(not(feature = "nightly"))]
mod bitwise_stable;

// ==========================================
// 1. Primitive Bitwise Operations
// ==========================================

/// Unary bitwise NOT (`!x`).
pub struct BitNot;

/// Binary bitwise AND (`a & b`, taking a 2-tuple `(A, B)`).
pub struct BitAnd;

/// Binary bitwise OR (`a | b`, taking a 2-tuple `(A, B)`).
pub struct BitOr;

/// Binary bitwise XOR (`a ^ b`, taking a 2-tuple `(A, B)`).
pub struct BitXor;

/// Total checked left shift (takes `(T, u32)` or reference combinations; returns `Option<T>`, yielding `None` when shift amount `>= T::BITS`).
pub struct CheckedShl;

/// Total checked right shift (takes `(T, u32)` or reference combinations; returns `Option<T>`, yielding `None` when shift amount `>= T::BITS`).
pub struct CheckedShr;

/// Compile-time statically bounds-checked fixed left shift (`x << N`, statically enforcing `N < T::BITS` at compile time with zero runtime branches and guaranteed panic-free).
pub struct Shl<const N: u32>;

/// Compile-time statically bounds-checked fixed right shift (`x >> N`, statically enforcing `N < T::BITS` at compile time with zero runtime branches and guaranteed panic-free).
pub struct Shr<const N: u32>;

// ==========================================
// 2. Transparent Combinator Aliases (Zero Redundant Primitives)
// ==========================================

/// Takes a 2-tuple `(A, B)`, computes `L` and `R` respectively, then applies bitwise AND (`&`).
pub type PairBitAnd<L, R> = Then<MapPair<L, R>, BitAnd>;

/// Takes a 2-tuple `(A, B)`, computes `L` and `R` respectively, then applies bitwise OR (`|`).
pub type PairBitOr<L, R> = Then<MapPair<L, R>, BitOr>;

/// Takes a 2-tuple `(A, B)`, computes `L` and `R` respectively, then applies bitwise XOR (`^`).
pub type PairBitXor<L, R> = Then<MapPair<L, R>, BitXor>;

/// Takes a 2-tuple `(A, B)`, computes `L` and `R` respectively, then performs a checked left shift.
pub type PairCheckedShl<L, R> = Then<MapPair<L, R>, CheckedShl>;

/// Takes a 2-tuple `(A, B)`, computes `L` and `R` respectively, then performs a checked right shift.
pub type PairCheckedShr<L, R> = Then<MapPair<L, R>, CheckedShr>;

/// Takes a single `Copy` input, forks it to `L` and `R`, then applies bitwise AND (`&`).
pub type ForkBitAnd<L, R> = Then<Fork<L, R>, BitAnd>;

/// Takes a single `Copy` input, forks it to `L` and `R`, then applies bitwise OR (`|`).
pub type ForkBitOr<L, R> = Then<Fork<L, R>, BitOr>;

/// Takes a single `Copy` input, forks it to `L` and `R`, then applies bitwise XOR (`^`).
pub type ForkBitXor<L, R> = Then<Fork<L, R>, BitXor>;

/// Takes a single `Copy` input, forks it to `L` and `R`, then performs a checked left shift.
pub type ForkCheckedShl<L, R> = Then<Fork<L, R>, CheckedShl>;

/// Takes a single `Copy` input, forks it to `L` and `R`, then performs a checked right shift.
pub type ForkCheckedShr<L, R> = Then<Fork<L, R>, CheckedShr>;

/// Unary constant bitwise AND: `x & C`.
pub type BitAndConst<C> = Then<AttachUnit, PairBitAnd<Id, C>>;

/// Unary constant bitwise OR: `x | C`.
pub type BitOrConst<C> = Then<AttachUnit, PairBitOr<Id, C>>;

/// Unary constant bitwise XOR: `x ^ C`.
pub type BitXorConst<C> = Then<AttachUnit, PairBitXor<Id, C>>;

/// All-bits-set mask predicate: `(x & Mask) == Mask` (input has all bits specified in `Mask` set).
pub type HasAllBits<Mask> = Then<BitAndConst<Mask>, EqConst<Mask>>;

/// Any-bit-set mask predicate: `(x & Mask) != 0` (input has at least one bit specified in `Mask` set).
pub type HasAnyBits<Mask> = Then<BitAndConst<Mask>, IsNonZero>;

/// No-bits-set mask predicate: `(x & Mask) == 0` (input has none of the bits specified in `Mask` set; useful for alignment, parity, or high-bit checks).
pub type HasNoBits<Mask> = Then<BitAndConst<Mask>, IsZero>;

// ==========================================
// 3. SafeFunction Contract Guarantees
// ==========================================

macro_rules! impl_safe_bitwise_for_ints {
    ($($ty:ty),* $(,)?) => {
        $(
            // SAFETY: Native integer bitwise NOT, bitwise logic, and checked shifts are pure, deterministic, guaranteed to terminate, and panic-free.
            unsafe impl SafeFunction<$ty> for BitNot {}
            unsafe impl SafeFunction<&$ty> for BitNot {}

            unsafe impl SafeFunction<($ty, $ty)> for BitAnd {}
            unsafe impl SafeFunction<(&$ty, $ty)> for BitAnd {}
            unsafe impl SafeFunction<($ty, &$ty)> for BitAnd {}
            unsafe impl SafeFunction<(&$ty, &$ty)> for BitAnd {}

            unsafe impl SafeFunction<($ty, $ty)> for BitOr {}
            unsafe impl SafeFunction<(&$ty, $ty)> for BitOr {}
            unsafe impl SafeFunction<($ty, &$ty)> for BitOr {}
            unsafe impl SafeFunction<(&$ty, &$ty)> for BitOr {}

            unsafe impl SafeFunction<($ty, $ty)> for BitXor {}
            unsafe impl SafeFunction<(&$ty, $ty)> for BitXor {}
            unsafe impl SafeFunction<($ty, &$ty)> for BitXor {}
            unsafe impl SafeFunction<(&$ty, &$ty)> for BitXor {}

            unsafe impl SafeFunction<($ty, u32)> for CheckedShl {}
            unsafe impl SafeFunction<(&$ty, u32)> for CheckedShl {}
            unsafe impl SafeFunction<($ty, &u32)> for CheckedShl {}
            unsafe impl SafeFunction<(&$ty, &u32)> for CheckedShl {}

            unsafe impl SafeFunction<($ty, u32)> for CheckedShr {}
            unsafe impl SafeFunction<(&$ty, u32)> for CheckedShr {}
            unsafe impl SafeFunction<($ty, &u32)> for CheckedShr {}
            unsafe impl SafeFunction<(&$ty, &u32)> for CheckedShr {}

            // SAFETY: Shl/Shr statically guarantee N < T::BITS via a const block in the call body, so compiled code can never panic at runtime.
            unsafe impl<const N: u32> SafeFunction<$ty> for Shl<N> {}
            unsafe impl<const N: u32> SafeFunction<&$ty> for Shl<N> {}

            unsafe impl<const N: u32> SafeFunction<$ty> for Shr<N> {}
            unsafe impl<const N: u32> SafeFunction<&$ty> for Shr<N> {}
        )*
    };
}

impl_safe_bitwise_for_ints!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);
