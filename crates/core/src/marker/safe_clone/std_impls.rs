//! `SafeClone` contract implementations for `std` hash collections and hash builders.

use super::SafeClone;
use std::collections::{HashMap, HashSet};
use std::hash::RandomState;

// SAFETY: `RandomState::clone` duplicates the exact internal hash seeds.
unsafe impl SafeClone for RandomState {}

// SAFETY: `HashSet::clone` copies the underlying hash table layout directly and invokes `T::clone` and `S::clone` per entry,
// without re-inserting via an untrusted `Hash` or `Eq`, faithfully preserving entries and counts.
unsafe impl<T: SafeClone, S: SafeClone> SafeClone for HashSet<T, S> {}

// SAFETY: `HashMap::clone` copies the underlying hash table layout directly and invokes `K::clone`, `V::clone`, and `S::clone` per entry,
// without re-inserting via an untrusted `Hash` or `Eq`, faithfully preserving key-value entries and counts.
unsafe impl<K: SafeClone, V: SafeClone, S: SafeClone> SafeClone for HashMap<K, V, S> {}
