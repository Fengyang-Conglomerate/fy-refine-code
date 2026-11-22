//! Mathematical-value comparison for primitive integers supporting `const` evaluation on nightly;
//! bit-width selection is resolved at compile time without widening everything to 128 bits.
//!
//! Also provides mixed value-and-reference comparisons (`&A` vs `B` and `A` vs `&B`),
//! non-overlapping with two-sided reference forwarding (`&A` vs `&B`), allowing refinement predicates
//! to compare borrowed inputs directly against constant values.

use super::{SafeEq, SafeOrd};
use core::cmp::Ordering;

// Same-signedness pairs widen to the wider type; mixed-signedness pairs cast only on the non-negative branch to a sufficiently wide unsigned type.
macro_rules! widened {
    ($left:expr, $right:expr, $a:ty, $b:ty, $wide_a:ty, $wide_b:ty, $op:tt) => {
        if <$a>::BITS >= <$b>::BITS {
            ($left as $wide_a) $op ($right as $wide_a)
        } else {
            ($left as $wide_b) $op ($right as $wide_b)
        }
    };
}

// The relation between a negative signed integer and any unsigned integer is determined without casting.
macro_rules! negative_result {
    (signed_unsigned, ==) => {
        false
    };
    (signed_unsigned, !=) => {
        true
    };
    (signed_unsigned, <) => {
        true
    };
    (signed_unsigned, <=) => {
        true
    };
    (signed_unsigned, >) => {
        false
    };
    (signed_unsigned, >=) => {
        false
    };
    (unsigned_signed, ==) => {
        false
    };
    (unsigned_signed, !=) => {
        true
    };
    (unsigned_signed, <) => {
        false
    };
    (unsigned_signed, <=) => {
        false
    };
    (unsigned_signed, >) => {
        true
    };
    (unsigned_signed, >=) => {
        true
    };
}

macro_rules! compare {
    (same, $left:expr, $right:expr, $a:ty, $b:ty, $wide_a:ty, $wide_b:ty, $op:tt) => {
        widened!($left, $right, $a, $b, $wide_a, $wide_b, $op)
    };
    (signed_unsigned, $left:expr, $right:expr, $a:ty, $b:ty, $wide_a:ty, $wide_b:ty, $op:tt) => {
        if $left < 0 {
            negative_result!(signed_unsigned, $op)
        } else {
            widened!($left, $right, $a, $b, $wide_a, $wide_b, $op)
        }
    };
    (unsigned_signed, $left:expr, $right:expr, $a:ty, $b:ty, $wide_a:ty, $wide_b:ty, $op:tt) => {
        if $right < 0 {
            negative_result!(unsigned_signed, $op)
        } else {
            widened!($left, $right, $a, $b, $wide_a, $wide_b, $op)
        }
    };
}

macro_rules! pair {
    ($kind:ident, $a:ty, $b:ty, $wide_a:ty, $wide_b:ty) => {
        // SAFETY: Negative values are handled first, followed by lossless widening comparisons; primitive integer operations are pure, total, and terminating.
        const unsafe impl SafeEq<$b> for $a {
            #[inline(always)]
            fn eq(&self, other: &$b) -> bool {
                compare!($kind, *self, *other, $a, $b, $wide_a, $wide_b, ==)
            }
            #[inline(always)]
            fn ne(&self, other: &$b) -> bool {
                compare!($kind, *self, *other, $a, $b, $wide_a, $wide_b, !=)
            }
        }
        // SAFETY: All seven methods compare the same mathematical integer values, satisfy total ordering and consistency with equality, and cannot overflow or truncate.
        const unsafe impl SafeOrd<$b> for $a {
            #[inline(always)]
            fn cmp(&self, other: &$b) -> Ordering {
                if compare!($kind, *self, *other, $a, $b, $wide_a, $wide_b, <) {
                    Ordering::Less
                } else if compare!($kind, *self, *other, $a, $b, $wide_a, $wide_b, >) {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            }
            #[inline(always)]
            fn lt(&self, other: &$b) -> bool {
                compare!($kind, *self, *other, $a, $b, $wide_a, $wide_b, <)
            }
            #[inline(always)]
            fn le(&self, other: &$b) -> bool {
                compare!($kind, *self, *other, $a, $b, $wide_a, $wide_b, <=)
            }
            #[inline(always)]
            fn gt(&self, other: &$b) -> bool {
                compare!($kind, *self, *other, $a, $b, $wide_a, $wide_b, >)
            }
            #[inline(always)]
            fn ge(&self, other: &$b) -> bool {
                compare!($kind, *self, *other, $a, $b, $wide_a, $wide_b, >=)
            }
        }

        // SAFETY: Dereferencing the left operand forwards to the safe primitive integer comparison for the same pair, preserving all equality and total ordering axioms.
        const unsafe impl SafeEq<$b> for &$a {
            #[inline(always)]
            fn eq(&self, other: &$b) -> bool {
                <$a as SafeEq<$b>>::eq(*self, other)
            }
            #[inline(always)]
            fn ne(&self, other: &$b) -> bool {
                <$a as SafeEq<$b>>::ne(*self, other)
            }
        }
        const unsafe impl SafeOrd<$b> for &$a {
            #[inline(always)]
            fn cmp(&self, other: &$b) -> Ordering {
                <$a as SafeOrd<$b>>::cmp(*self, other)
            }
            #[inline(always)]
            fn lt(&self, other: &$b) -> bool {
                <$a as SafeOrd<$b>>::lt(*self, other)
            }
            #[inline(always)]
            fn le(&self, other: &$b) -> bool {
                <$a as SafeOrd<$b>>::le(*self, other)
            }
            #[inline(always)]
            fn gt(&self, other: &$b) -> bool {
                <$a as SafeOrd<$b>>::gt(*self, other)
            }
            #[inline(always)]
            fn ge(&self, other: &$b) -> bool {
                <$a as SafeOrd<$b>>::ge(*self, other)
            }
        }

        // SAFETY: Dereferencing the right operand forwards to the safe primitive integer comparison for the same pair, preserving all equality and total ordering axioms.
        const unsafe impl SafeEq<&$b> for $a {
            #[inline(always)]
            fn eq(&self, other: &&$b) -> bool {
                <$a as SafeEq<$b>>::eq(self, *other)
            }
            #[inline(always)]
            fn ne(&self, other: &&$b) -> bool {
                <$a as SafeEq<$b>>::ne(self, *other)
            }
        }
        const unsafe impl SafeOrd<&$b> for $a {
            #[inline(always)]
            fn cmp(&self, other: &&$b) -> Ordering {
                <$a as SafeOrd<$b>>::cmp(self, *other)
            }
            #[inline(always)]
            fn lt(&self, other: &&$b) -> bool {
                <$a as SafeOrd<$b>>::lt(self, *other)
            }
            #[inline(always)]
            fn le(&self, other: &&$b) -> bool {
                <$a as SafeOrd<$b>>::le(self, *other)
            }
            #[inline(always)]
            fn gt(&self, other: &&$b) -> bool {
                <$a as SafeOrd<$b>>::gt(self, *other)
            }
            #[inline(always)]
            fn ge(&self, other: &&$b) -> bool {
                <$a as SafeOrd<$b>>::ge(self, *other)
            }
        }
    };
}

macro_rules! signed_row {
    ($a:ty, $unsigned_a:ty) => {
        pair!(same, $a, i8, $a, i8);
        pair!(same, $a, i16, $a, i16);
        pair!(same, $a, i32, $a, i32);
        pair!(same, $a, i64, $a, i64);
        pair!(same, $a, i128, $a, i128);
        pair!(same, $a, isize, $a, isize);
        pair!(signed_unsigned, $a, u8, $unsigned_a, u8);
        pair!(signed_unsigned, $a, u16, $unsigned_a, u16);
        pair!(signed_unsigned, $a, u32, $unsigned_a, u32);
        pair!(signed_unsigned, $a, u64, $unsigned_a, u64);
        pair!(signed_unsigned, $a, u128, $unsigned_a, u128);
        pair!(signed_unsigned, $a, usize, $unsigned_a, usize);
    };
}

macro_rules! unsigned_row {
    ($a:ty) => {
        pair!(same, $a, u8, $a, u8);
        pair!(same, $a, u16, $a, u16);
        pair!(same, $a, u32, $a, u32);
        pair!(same, $a, u64, $a, u64);
        pair!(same, $a, u128, $a, u128);
        pair!(same, $a, usize, $a, usize);
        pair!(unsigned_signed, $a, i8, $a, u8);
        pair!(unsigned_signed, $a, i16, $a, u16);
        pair!(unsigned_signed, $a, i32, $a, u32);
        pair!(unsigned_signed, $a, i64, $a, u64);
        pair!(unsigned_signed, $a, i128, $a, u128);
        pair!(unsigned_signed, $a, isize, $a, usize);
    };
}

signed_row!(i8, u8);
signed_row!(i16, u16);
signed_row!(i32, u32);
signed_row!(i64, u64);
signed_row!(i128, u128);
signed_row!(isize, usize);
unsigned_row!(u8);
unsigned_row!(u16);
unsigned_row!(u32);
unsigned_row!(u64);
unsigned_row!(u128);
unsigned_row!(usize);
