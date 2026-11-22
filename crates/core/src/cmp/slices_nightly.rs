//! Safe equality and lexicographical total ordering implementations for slices, string slices, and fixed-size arrays (`[T]`, `str`, `[T; N]`) supporting `const` evaluation on nightly.

use super::{SafeEq, SafeOrd};
use core::cmp::Ordering;

// SAFETY: Compares lengths and then elements via `SafeEq`; the bounded loop always terminates without panicking and satisfies mathematical equality.
const unsafe impl<T, U> SafeEq<[U]> for [T]
where
    T: [const] SafeEq<U>,
{
    #[inline]
    fn eq(&self, other: &[U]) -> bool {
        if self.len() != other.len() {
            return false;
        }
        let mut i = 0;
        while i < self.len() {
            if <T as SafeEq<U>>::ne(&self[i], &other[i]) {
                return false;
            }
            i += 1;
        }
        true
    }

    #[inline]
    fn ne(&self, other: &[U]) -> bool {
        !<[T] as SafeEq<[U]>>::eq(self, other)
    }
}

// SAFETY: Performs element-wise lexicographical comparison delegating to element `SafeOrd`, satisfying total ordering and consistency with `SafeEq`.
const unsafe impl<T, U> SafeOrd<[U]> for [T]
where
    T: [const] SafeOrd<U>,
{
    #[inline]
    fn cmp(&self, other: &[U]) -> Ordering {
        let len = if self.len() < other.len() {
            self.len()
        } else {
            other.len()
        };
        let mut i = 0;
        while i < len {
            match <T as SafeOrd<U>>::cmp(&self[i], &other[i]) {
                Ordering::Equal => i += 1,
                non_eq => return non_eq,
            }
        }
        <usize as SafeOrd>::cmp(&self.len(), &other.len())
    }

    #[inline]
    fn lt(&self, other: &[U]) -> bool {
        matches!(<[T] as SafeOrd<[U]>>::cmp(self, other), Ordering::Less)
    }

    #[inline]
    fn le(&self, other: &[U]) -> bool {
        !matches!(<[T] as SafeOrd<[U]>>::cmp(self, other), Ordering::Greater)
    }

    #[inline]
    fn gt(&self, other: &[U]) -> bool {
        matches!(<[T] as SafeOrd<[U]>>::cmp(self, other), Ordering::Greater)
    }

    #[inline]
    fn ge(&self, other: &[U]) -> bool {
        !matches!(<[T] as SafeOrd<[U]>>::cmp(self, other), Ordering::Less)
    }
}

// SAFETY: Byte-sequence comparison of UTF-8 strings is strictly identical to lexicographical order by Unicode code points.
const unsafe impl SafeEq for str {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        <[u8] as SafeEq>::eq(self.as_bytes(), other.as_bytes())
    }

    #[inline]
    fn ne(&self, other: &str) -> bool {
        <[u8] as SafeEq>::ne(self.as_bytes(), other.as_bytes())
    }
}

const unsafe impl SafeOrd for str {
    #[inline]
    fn cmp(&self, other: &str) -> Ordering {
        <[u8] as SafeOrd>::cmp(self.as_bytes(), other.as_bytes())
    }

    #[inline]
    fn lt(&self, other: &str) -> bool {
        <[u8] as SafeOrd>::lt(self.as_bytes(), other.as_bytes())
    }

    #[inline]
    fn le(&self, other: &str) -> bool {
        <[u8] as SafeOrd>::le(self.as_bytes(), other.as_bytes())
    }

    #[inline]
    fn gt(&self, other: &str) -> bool {
        <[u8] as SafeOrd>::gt(self.as_bytes(), other.as_bytes())
    }

    #[inline]
    fn ge(&self, other: &str) -> bool {
        <[u8] as SafeOrd>::ge(self.as_bytes(), other.as_bytes())
    }
}

// SAFETY: Dereferencing the left operand forwards to the corresponding slice safe comparison, preserving equality and total ordering consistency.
const unsafe impl<T, U> SafeEq<[U]> for &[T]
where
    T: [const] SafeEq<U>,
{
    #[inline(always)]
    fn eq(&self, other: &[U]) -> bool {
        <[T] as SafeEq<[U]>>::eq(*self, other)
    }

    #[inline(always)]
    fn ne(&self, other: &[U]) -> bool {
        <[T] as SafeEq<[U]>>::ne(*self, other)
    }
}

const unsafe impl<T, U> SafeOrd<[U]> for &[T]
where
    T: [const] SafeOrd<U>,
{
    #[inline(always)]
    fn cmp(&self, other: &[U]) -> Ordering {
        <[T] as SafeOrd<[U]>>::cmp(*self, other)
    }

    #[inline(always)]
    fn lt(&self, other: &[U]) -> bool {
        <[T] as SafeOrd<[U]>>::lt(*self, other)
    }

    #[inline(always)]
    fn le(&self, other: &[U]) -> bool {
        <[T] as SafeOrd<[U]>>::le(*self, other)
    }

    #[inline(always)]
    fn gt(&self, other: &[U]) -> bool {
        <[T] as SafeOrd<[U]>>::gt(*self, other)
    }

    #[inline(always)]
    fn ge(&self, other: &[U]) -> bool {
        <[T] as SafeOrd<[U]>>::ge(*self, other)
    }
}

// SAFETY: Dereferencing the right operand forwards to the corresponding slice safe comparison, preserving equality and total ordering consistency.
const unsafe impl<T, U> SafeEq<&[U]> for [T]
where
    T: [const] SafeEq<U>,
{
    #[inline(always)]
    fn eq(&self, other: &&[U]) -> bool {
        <[T] as SafeEq<[U]>>::eq(self, *other)
    }

    #[inline(always)]
    fn ne(&self, other: &&[U]) -> bool {
        <[T] as SafeEq<[U]>>::ne(self, *other)
    }
}

const unsafe impl<T, U> SafeOrd<&[U]> for [T]
where
    T: [const] SafeOrd<U>,
{
    #[inline(always)]
    fn cmp(&self, other: &&[U]) -> Ordering {
        <[T] as SafeOrd<[U]>>::cmp(self, *other)
    }

    #[inline(always)]
    fn lt(&self, other: &&[U]) -> bool {
        <[T] as SafeOrd<[U]>>::lt(self, *other)
    }

    #[inline(always)]
    fn le(&self, other: &&[U]) -> bool {
        <[T] as SafeOrd<[U]>>::le(self, *other)
    }

    #[inline(always)]
    fn gt(&self, other: &&[U]) -> bool {
        <[T] as SafeOrd<[U]>>::gt(self, *other)
    }

    #[inline(always)]
    fn ge(&self, other: &&[U]) -> bool {
        <[T] as SafeOrd<[U]>>::ge(self, *other)
    }
}

// SAFETY: Dereferencing the left operand forwards to the safe string slice comparison.
const unsafe impl SafeEq<str> for &str {
    #[inline(always)]
    fn eq(&self, other: &str) -> bool {
        <str as SafeEq>::eq(*self, other)
    }

    #[inline(always)]
    fn ne(&self, other: &str) -> bool {
        <str as SafeEq>::ne(*self, other)
    }
}

const unsafe impl SafeOrd<str> for &str {
    #[inline(always)]
    fn cmp(&self, other: &str) -> Ordering {
        <str as SafeOrd>::cmp(*self, other)
    }

    #[inline(always)]
    fn lt(&self, other: &str) -> bool {
        <str as SafeOrd>::lt(*self, other)
    }

    #[inline(always)]
    fn le(&self, other: &str) -> bool {
        <str as SafeOrd>::le(*self, other)
    }

    #[inline(always)]
    fn gt(&self, other: &str) -> bool {
        <str as SafeOrd>::gt(*self, other)
    }

    #[inline(always)]
    fn ge(&self, other: &str) -> bool {
        <str as SafeOrd>::ge(*self, other)
    }
}

// SAFETY: Dereferencing the right operand forwards to the safe string slice comparison.
const unsafe impl SafeEq<&str> for str {
    #[inline(always)]
    fn eq(&self, other: &&str) -> bool {
        <str as SafeEq>::eq(self, *other)
    }

    #[inline(always)]
    fn ne(&self, other: &&str) -> bool {
        <str as SafeEq>::ne(self, *other)
    }
}

const unsafe impl SafeOrd<&str> for str {
    #[inline(always)]
    fn cmp(&self, other: &&str) -> Ordering {
        <str as SafeOrd>::cmp(self, *other)
    }

    #[inline(always)]
    fn lt(&self, other: &&str) -> bool {
        <str as SafeOrd>::lt(self, *other)
    }

    #[inline(always)]
    fn le(&self, other: &&str) -> bool {
        <str as SafeOrd>::le(self, *other)
    }

    #[inline(always)]
    fn gt(&self, other: &&str) -> bool {
        <str as SafeOrd>::gt(self, *other)
    }

    #[inline(always)]
    fn ge(&self, other: &&str) -> bool {
        <str as SafeOrd>::ge(self, *other)
    }
}

// SAFETY: Fixed-size array equality delegates directly to element-wise slice `SafeEq`; it is pure, terminating, panic-free, and satisfies mathematical equality.
const unsafe impl<T, U, const N: usize> SafeEq<[U; N]> for [T; N]
where
    T: [const] SafeEq<U>,
{
    #[inline(always)]
    fn eq(&self, other: &[U; N]) -> bool {
        <[T] as SafeEq<[U]>>::eq(self, other)
    }

    #[inline(always)]
    fn ne(&self, other: &[U; N]) -> bool {
        <[T] as SafeEq<[U]>>::ne(self, other)
    }
}

// SAFETY: Fixed-size array ordering delegates to slice lexicographical total ordering, remaining strictly consistent with `SafeEq`.
const unsafe impl<T, U, const N: usize> SafeOrd<[U; N]> for [T; N]
where
    T: [const] SafeOrd<U>,
{
    #[inline(always)]
    fn cmp(&self, other: &[U; N]) -> Ordering {
        <[T] as SafeOrd<[U]>>::cmp(self, other)
    }

    #[inline(always)]
    fn lt(&self, other: &[U; N]) -> bool {
        <[T] as SafeOrd<[U]>>::lt(self, other)
    }

    #[inline(always)]
    fn le(&self, other: &[U; N]) -> bool {
        <[T] as SafeOrd<[U]>>::le(self, other)
    }

    #[inline(always)]
    fn gt(&self, other: &[U; N]) -> bool {
        <[T] as SafeOrd<[U]>>::gt(self, other)
    }

    #[inline(always)]
    fn ge(&self, other: &[U; N]) -> bool {
        <[T] as SafeOrd<[U]>>::ge(self, other)
    }
}

// SAFETY: Dereferencing the left operand forwards to the corresponding fixed-size array safe comparison.
const unsafe impl<T, U, const N: usize> SafeEq<[U; N]> for &[T; N]
where
    T: [const] SafeEq<U>,
{
    #[inline(always)]
    fn eq(&self, other: &[U; N]) -> bool {
        <[T; N] as SafeEq<[U; N]>>::eq(*self, other)
    }

    #[inline(always)]
    fn ne(&self, other: &[U; N]) -> bool {
        <[T; N] as SafeEq<[U; N]>>::ne(*self, other)
    }
}

const unsafe impl<T, U, const N: usize> SafeOrd<[U; N]> for &[T; N]
where
    T: [const] SafeOrd<U>,
{
    #[inline(always)]
    fn cmp(&self, other: &[U; N]) -> Ordering {
        <[T; N] as SafeOrd<[U; N]>>::cmp(*self, other)
    }

    #[inline(always)]
    fn lt(&self, other: &[U; N]) -> bool {
        <[T; N] as SafeOrd<[U; N]>>::lt(*self, other)
    }

    #[inline(always)]
    fn le(&self, other: &[U; N]) -> bool {
        <[T; N] as SafeOrd<[U; N]>>::le(*self, other)
    }

    #[inline(always)]
    fn gt(&self, other: &[U; N]) -> bool {
        <[T; N] as SafeOrd<[U; N]>>::gt(*self, other)
    }

    #[inline(always)]
    fn ge(&self, other: &[U; N]) -> bool {
        <[T; N] as SafeOrd<[U; N]>>::ge(*self, other)
    }
}

// SAFETY: Dereferencing the right operand forwards to the corresponding fixed-size array safe comparison.
const unsafe impl<T, U, const N: usize> SafeEq<&[U; N]> for [T; N]
where
    T: [const] SafeEq<U>,
{
    #[inline(always)]
    fn eq(&self, other: &&[U; N]) -> bool {
        <[T; N] as SafeEq<[U; N]>>::eq(self, *other)
    }

    #[inline(always)]
    fn ne(&self, other: &&[U; N]) -> bool {
        <[T; N] as SafeEq<[U; N]>>::ne(self, *other)
    }
}

const unsafe impl<T, U, const N: usize> SafeOrd<&[U; N]> for [T; N]
where
    T: [const] SafeOrd<U>,
{
    #[inline(always)]
    fn cmp(&self, other: &&[U; N]) -> Ordering {
        <[T; N] as SafeOrd<[U; N]>>::cmp(self, *other)
    }

    #[inline(always)]
    fn lt(&self, other: &&[U; N]) -> bool {
        <[T; N] as SafeOrd<[U; N]>>::lt(self, *other)
    }

    #[inline(always)]
    fn le(&self, other: &&[U; N]) -> bool {
        <[T; N] as SafeOrd<[U; N]>>::le(self, *other)
    }

    #[inline(always)]
    fn gt(&self, other: &&[U; N]) -> bool {
        <[T; N] as SafeOrd<[U; N]>>::gt(self, *other)
    }

    #[inline(always)]
    fn ge(&self, other: &&[U; N]) -> bool {
        <[T; N] as SafeOrd<[U; N]>>::ge(self, *other)
    }
}
