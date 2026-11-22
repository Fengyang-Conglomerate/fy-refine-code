//! `SafeDrop` contract implementations for `std` hash collections and hash builders.

use super::SafeDrop;
use std::collections::{HashMap, HashSet};
use std::hash::RandomState;

// SAFETY: `RandomState` has no custom destructor.
unsafe impl SafeDrop for RandomState {}

// SAFETY: Dropping `HashSet` only deallocates the hash table buckets and drops each element and the hash builder.
unsafe impl<T: SafeDrop, S: SafeDrop> SafeDrop for HashSet<T, S> {}

// SAFETY: Dropping `HashMap` only deallocates the hash table buckets and drops each key-value pair and the hash builder.
unsafe impl<K: SafeDrop, V: SafeDrop, S: SafeDrop> SafeDrop for HashMap<K, V, S> {}
