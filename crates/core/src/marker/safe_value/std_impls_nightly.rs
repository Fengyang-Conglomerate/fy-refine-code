//! `SafeValue` contract implementations supporting `const SafeValue` on nightly for `std` hash collections and hash builders.

use super::SafeValue;
use std::collections::{HashMap, HashSet};
use std::hash::RandomState;

// SAFETY: `RandomState` holds only immutable hash seeds and has no interior mutability.
const unsafe impl SafeValue for RandomState {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

// SAFETY: `HashSet` exclusively owns its elements and hash builder, so its logical element set remains constant under a shared borrow.
const unsafe impl<T: SafeValue, S: SafeValue> SafeValue for HashSet<T, S> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

// SAFETY: `HashMap` exclusively owns its key-value pairs and hash builder, so its logical mapping remains constant under a shared borrow.
const unsafe impl<K: SafeValue, V: SafeValue, S: SafeValue> SafeValue for HashMap<K, V, S> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}
