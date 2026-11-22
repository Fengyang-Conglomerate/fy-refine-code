//! Safe equality and safe total ordering implementations for primitive `bool` and Unicode scalar `char` on stable Rust.

use super::{SafeEq, SafeOrd};
use core::cmp::Ordering;

macro_rules! impl_primitive_cmp {
    ($t:ty) => {
        // SAFETY: Primitive scalar comparisons are pure, total, panic-free, and satisfy a total order (`false < true`; `char` ordered by Unicode code point).
        unsafe impl SafeEq for $t {
            #[inline(always)]
            fn eq(&self, other: &$t) -> bool {
                *self == *other
            }
            #[inline(always)]
            fn ne(&self, other: &$t) -> bool {
                *self != *other
            }
        }

        unsafe impl SafeOrd for $t {
            #[inline(always)]
            fn cmp(&self, other: &$t) -> Ordering {
                if *self < *other {
                    Ordering::Less
                } else if *self > *other {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            }
            #[inline(always)]
            fn lt(&self, other: &$t) -> bool {
                *self < *other
            }
            #[inline(always)]
            fn le(&self, other: &$t) -> bool {
                *self <= *other
            }
            #[inline(always)]
            fn gt(&self, other: &$t) -> bool {
                *self > *other
            }
            #[inline(always)]
            fn ge(&self, other: &$t) -> bool {
                *self >= *other
            }
        }

        // SAFETY: Dereferencing the left operand forwards to the underlying primitive scalar comparison.
        unsafe impl SafeEq<$t> for &$t {
            #[inline(always)]
            fn eq(&self, other: &$t) -> bool {
                <$t as SafeEq>::eq(*self, other)
            }
            #[inline(always)]
            fn ne(&self, other: &$t) -> bool {
                <$t as SafeEq>::ne(*self, other)
            }
        }

        unsafe impl SafeOrd<$t> for &$t {
            #[inline(always)]
            fn cmp(&self, other: &$t) -> Ordering {
                <$t as SafeOrd>::cmp(*self, other)
            }
            #[inline(always)]
            fn lt(&self, other: &$t) -> bool {
                <$t as SafeOrd>::lt(*self, other)
            }
            #[inline(always)]
            fn le(&self, other: &$t) -> bool {
                <$t as SafeOrd>::le(*self, other)
            }
            #[inline(always)]
            fn gt(&self, other: &$t) -> bool {
                <$t as SafeOrd>::gt(*self, other)
            }
            #[inline(always)]
            fn ge(&self, other: &$t) -> bool {
                <$t as SafeOrd>::ge(*self, other)
            }
        }

        // SAFETY: Dereferencing the right operand forwards to the underlying primitive scalar comparison.
        unsafe impl SafeEq<&$t> for $t {
            #[inline(always)]
            fn eq(&self, other: &&$t) -> bool {
                <$t as SafeEq>::eq(self, *other)
            }
            #[inline(always)]
            fn ne(&self, other: &&$t) -> bool {
                <$t as SafeEq>::ne(self, *other)
            }
        }

        unsafe impl SafeOrd<&$t> for $t {
            #[inline(always)]
            fn cmp(&self, other: &&$t) -> Ordering {
                <$t as SafeOrd>::cmp(self, *other)
            }
            #[inline(always)]
            fn lt(&self, other: &&$t) -> bool {
                <$t as SafeOrd>::lt(self, *other)
            }
            #[inline(always)]
            fn le(&self, other: &&$t) -> bool {
                <$t as SafeOrd>::le(self, *other)
            }
            #[inline(always)]
            fn gt(&self, other: &&$t) -> bool {
                <$t as SafeOrd>::gt(self, *other)
            }
            #[inline(always)]
            fn ge(&self, other: &&$t) -> bool {
                <$t as SafeOrd>::ge(self, *other)
            }
        }
    };
}

impl_primitive_cmp!(bool);
impl_primitive_cmp!(char);
