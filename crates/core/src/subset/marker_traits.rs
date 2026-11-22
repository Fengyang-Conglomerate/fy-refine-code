//! Closed-loop implementations of crate safety contracts (`SafeDrop`, `SafeClone`) for `Subset`.

use super::Subset;
use crate::function::Predicate;
use crate::marker::{SafeClone, SafeDrop, SafeValue};

// SAFETY: The phantom marker is a zero-sized pure type with no destructor logic;
// when the inner carrier T satisfies SafeDrop, dropping Subset is benign, panic-free, and terminating.
unsafe impl<P, T: ?Sized> SafeDrop for Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue + SafeDrop,
{
}

// SAFETY: Subset::clone delegates solely to T::clone, which guarantees logical equivalence under SafeClone, so the cloned value necessarily satisfies predicate P.
unsafe impl<P, T> SafeClone for Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue + SafeClone,
{
}
