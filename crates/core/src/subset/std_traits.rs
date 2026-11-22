//! Transparent delegation of core standard-library traits for `Subset`.
//!
//! Enforces foundational invariance and clone-fidelity guarantees:
//! - Read-only observation traits (`Deref`, `AsRef`, `Borrow`, comparison, hashing, formatting) transparently delegate to the underlying `T`;
//! - `Clone` is gated by [`SafeClone`], enabled only when `T: SafeClone` guarantees faithful cloning;
//! - `Copy` emerges automatically when `T: SafeValue + SafeClone + Copy`;
//! - `DerefMut` is intentionally never implemented.

use core::borrow::Borrow;
use core::cmp::Ordering;
use core::fmt::{self, Debug, Display};
use core::hash::{Hash, Hasher};
use core::ops::Deref;

use super::Subset;
use crate::function::Predicate;
use crate::marker::{SafeClone, SafeValue};

// ---------------- References and dereferencing (?Sized supported) ----------------

impl<P, T: ?Sized> Deref for Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue,
{
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<P, T: ?Sized> AsRef<T> for Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue,
{
    #[inline(always)]
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<P, T: ?Sized> Borrow<T> for Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue,
{
    #[inline(always)]
    fn borrow(&self) -> &T {
        &self.value
    }
}

// ---------------- Copy and Clone (gated by SafeClone fidelity) ----------------

impl<P, T> Clone for Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue + SafeClone,
{
    #[inline]
    fn clone(&self) -> Self {
        // SAFETY: The T: SafeClone contract guarantees that T::clone produces a copy logically
        // equivalent to self.value under all SafeFunction observations, so P(&T) == true still holds.
        unsafe { Subset::new_unchecked(self.value.clone()) }
    }
}

impl<P, T> Copy for Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue + SafeClone + Copy,
{
}

// ---------------- Comparison and hashing (?Sized and cross-predicate/cross-carrier supported) ----------------

impl<P, Q, T: ?Sized + PartialEq<U>, U: ?Sized> PartialEq<Subset<Q, U>> for Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    Q: for<'a> Predicate<&'a U>,
    T: SafeValue,
    U: SafeValue,
{
    #[inline]
    fn eq(&self, other: &Subset<Q, U>) -> bool {
        self.value == other.value
    }
}

impl<P, T: ?Sized + Eq> Eq for Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue,
{
}

impl<P, Q, T: ?Sized + PartialOrd<U>, U: ?Sized> PartialOrd<Subset<Q, U>> for Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    Q: for<'a> Predicate<&'a U>,
    T: SafeValue,
    U: SafeValue,
{
    #[inline]
    fn partial_cmp(&self, other: &Subset<Q, U>) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<P, T: ?Sized + Ord> Ord for Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl<P, T: ?Sized + Hash> Hash for Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

// ---------------- Formatting (?Sized supported) ----------------

impl<P, T: ?Sized + Display> Display for Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<P, T: ?Sized + Debug> Debug for Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Subset").field(&&self.value).finish()
    }
}
