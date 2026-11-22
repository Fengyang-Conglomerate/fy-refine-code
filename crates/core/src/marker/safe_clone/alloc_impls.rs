//! `SafeClone` contract implementations for `alloc`-backed standard heap containers and borrow abstractions.

use super::SafeClone;
use crate::marker::SafeValue;
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};
use alloc::rc::Rc;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

// SAFETY: `String::clone` duplicates the exact UTF-8 byte sequence.
unsafe impl SafeClone for String {}

// SAFETY: `Box<T>::clone` clones the heap-allocated payload `T` directly, preserving full logical equivalence.
unsafe impl<T: SafeClone> SafeClone for Box<T> {}

// SAFETY: `Box<str>::clone` duplicates the exact UTF-8 byte sequence.
unsafe impl SafeClone for Box<str> {}

// SAFETY: `Box<[T]>::clone` invokes `T::clone` element-wise, preserving full logical equivalence.
unsafe impl<T: SafeClone> SafeClone for Box<[T]> {}

// SAFETY: Standard library `Clone` for `Vec`, `VecDeque`, and `LinkedList` invokes `T::clone` element-wise, preserving order and logical equivalence.
unsafe impl<T: SafeClone> SafeClone for Vec<T> {}
unsafe impl<T: SafeClone> SafeClone for VecDeque<T> {}
unsafe impl<T: SafeClone> SafeClone for LinkedList<T> {}

// SAFETY: Standard library `Clone` for `BinaryHeap`, `BTreeSet`, and `BTreeMap` directly copies the underlying array or B-tree node structure,
// calling only `Clone` on elements/keys/values without re-inserting via an untrusted `Ord`, faithfully preserving existing entries and topology.
unsafe impl<T: SafeClone> SafeClone for BinaryHeap<T> {}
unsafe impl<T: SafeClone> SafeClone for BTreeSet<T> {}
unsafe impl<K: SafeClone, V: SafeClone> SafeClone for BTreeMap<K, V> {}

// SAFETY: Implemented only for trusted standard borrowed targets (`str` and `[T]`);
// the `Borrowed` variant copies the reference and the `Owned` variant clones `String` / `Vec<T>` directly, isolating external custom `ToOwned`/`Borrow` implementations.
unsafe impl SafeClone for Cow<'_, str> {}
unsafe impl<T: SafeClone> SafeClone for Cow<'_, [T]> {}

// SAFETY: `Rc::clone` only increments the reference count and shares the same underlying payload; `T: SafeValue` guarantees a constant, identical read-only logical view.
unsafe impl<T: ?Sized + SafeValue> SafeClone for Rc<T> {}

// SAFETY: `Arc::clone` only atomically increments the reference count and shares the same underlying payload; `T: SafeValue` guarantees a constant, identical read-only logical view.
unsafe impl<T: ?Sized + SafeValue> SafeClone for Arc<T> {}
