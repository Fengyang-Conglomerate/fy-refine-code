//! Shared-reference forwarding implementations for safe comparison at runtime on stable Rust.

use super::super::{SafeEq, SafeOrd};
use core::cmp::Ordering;

// SAFETY: Forwards the references verbatim; results and equality axioms are inherited from the underlying implementation.
unsafe impl<A: ?Sized, B: ?Sized> SafeEq<&B> for &A
where
    A: SafeEq<B>,
{
    #[inline(always)]
    fn eq(&self, other: &&B) -> bool {
        <A as SafeEq<B>>::eq(*self, *other)
    }

    #[inline(always)]
    fn ne(&self, other: &&B) -> bool {
        <A as SafeEq<B>>::ne(*self, *other)
    }
}

// SAFETY: Each method delegates to its counterpart in the underlying safe comparison, preserving total order consistency.
unsafe impl<A: ?Sized, B: ?Sized> SafeOrd<&B> for &A
where
    A: SafeOrd<B>,
{
    #[inline(always)]
    fn cmp(&self, other: &&B) -> Ordering {
        <A as SafeOrd<B>>::cmp(*self, *other)
    }

    #[inline(always)]
    fn lt(&self, other: &&B) -> bool {
        <A as SafeOrd<B>>::lt(*self, *other)
    }

    #[inline(always)]
    fn le(&self, other: &&B) -> bool {
        <A as SafeOrd<B>>::le(*self, *other)
    }

    #[inline(always)]
    fn gt(&self, other: &&B) -> bool {
        <A as SafeOrd<B>>::gt(*self, *other)
    }

    #[inline(always)]
    fn ge(&self, other: &&B) -> bool {
        <A as SafeOrd<B>>::ge(*self, *other)
    }
}
