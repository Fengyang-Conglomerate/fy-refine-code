//! Stable associated functions on `Subset` for checked construction, validation, and recursive base penetration.

use super::Subset;
use crate::function::Predicate;
use crate::marker::SafeValue;

impl<P, T: ?Sized> Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue,
{
    /// Checks whether the given shared reference satisfies predicate `P`.
    #[inline(always)]
    pub fn is_valid(value: &T) -> bool {
        P::call(value)
    }

    /// Performs checked construction from a shared reference; returns `Some(&Self)` with zero overhead if valid, or `None` otherwise.
    #[inline]
    pub fn try_from_ref(value: &T) -> Option<&Self> {
        if P::call(value) {
            // SAFETY: P::call(value) == true was just verified, and T: SafeValue guarantees no semantic drift.
            Some(unsafe { Self::from_ref_unchecked(value) })
        } else {
            None
        }
    }

    /// Recursively penetrates all nested `Subset` wrappers and borrows the innermost base carrier.
    #[inline(always)]
    pub fn as_base(this: &Self) -> &T::Base {
        this.value.as_base()
    }

    /// Explicitly informs the optimizer (LLVM `llvm.assume`) of the subset invariant `P(this) == true`.
    ///
    /// Allows callers to help the optimizer eliminate downstream bounds, division-by-zero, or unreachable-branch checks without writing `unsafe`.
    /// Recommended only on hot paths with cheap `O(1)` predicates (such as numeric bounds, non-zero, or fixed-length checks);
    /// avoid calling with `O(n)` looping/traversal predicates (such as `All<P>`), as the optimizer may be unable to eliminate the evaluation cost.
    #[inline(always)]
    pub fn assume_valid(this: &Self) {
        // SAFETY: The Subset<P, T> invariant always guarantees P::call(&this.value) == true,
        // and T: SafeValue ensures the predicate outcome cannot drift under a shared borrow.
        unsafe { core::hint::assert_unchecked(P::call(&this.value)) }
    }
}

impl<P, T> Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue,
{
    /// Performs checked construction from an owned value; returns `Ok(Self)` if valid, or returns the original value as `Err(value)` on failure.
    #[inline]
    pub fn try_new(value: T) -> Result<Self, T> {
        if P::call(&value) {
            // SAFETY: P::call(&value) == true was just verified, and T: SafeValue guarantees no semantic drift.
            Ok(unsafe { Self::new_unchecked(value) })
        } else {
            Err(value)
        }
    }
}

// SAFETY: The phantom marker occupies zero bytes and Subset is #[repr(transparent)];
// when inner T satisfies SafeValue, read-only logical data is immutable and the physical layout matches T::Base.
unsafe impl<P, T: ?Sized> SafeValue for Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue,
{
    type Base = T::Base;

    #[inline(always)]
    fn as_base(&self) -> &Self::Base {
        self.value.as_base()
    }
}
