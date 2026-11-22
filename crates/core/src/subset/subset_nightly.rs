//! Associated functions on `Subset` for checked construction, validation, and recursive base penetration with conditional `const` support on `nightly`.

use super::Subset;
use crate::function::{Function, Predicate};
use crate::marker::SafeValue;

impl<P, T: ?Sized> Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue,
{
    /// Checks whether the given shared reference satisfies predicate `P`; supports compile-time evaluation when `P` is a `const` predicate.
    #[inline(always)]
    pub const fn is_valid(value: &T) -> bool
    where
        P: for<'a> [const] Function<&'a T>,
    {
        P::call(value)
    }

    /// Performs checked construction from a shared reference; supports compile-time evaluation when `P` is a `const` predicate.
    #[inline]
    pub const fn try_from_ref(value: &T) -> Option<&Self>
    where
        P: for<'a> [const] Function<&'a T>,
    {
        if P::call(value) {
            // SAFETY: P::call(value) == true was just verified, and T: SafeValue guarantees no semantic drift.
            Some(unsafe { Self::from_ref_unchecked(value) })
        } else {
            None
        }
    }

    /// Recursively penetrates all nested `Subset` wrappers and borrows the innermost base carrier.
    #[inline(always)]
    pub const fn as_base(this: &Self) -> &T::Base
    where
        T: [const] SafeValue,
    {
        this.value.as_base()
    }

    /// Explicitly informs the optimizer (LLVM `llvm.assume`) of the subset invariant `P(this) == true`.
    ///
    /// Callable in `const` contexts when `P` supports compile-time evaluation.
    /// Recommended only on hot paths with cheap `O(1)` predicates; avoid calling with `O(n)` traversal predicates.
    #[inline(always)]
    pub const fn assume_valid(this: &Self)
    where
        P: for<'a> [const] Function<&'a T>,
    {
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
    /// Performs checked construction from an owned value; supports compile-time evaluation when `P` is a `const` predicate, and returns the original value on failure.
    #[inline]
    pub const fn try_new(value: T) -> Result<Self, T>
    where
        P: for<'a> [const] Function<&'a T>,
    {
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
const unsafe impl<P, T: ?Sized> SafeValue for Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    T: [const] SafeValue,
{
    type Base = T::Base;

    #[inline(always)]
    fn as_base(&self) -> &Self::Base {
        self.value.as_base()
    }
}
