//! `SafeDrop` contract implementations for `alloc`-backed standard heap containers and borrow abstractions.

use super::SafeDrop;
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};
use alloc::rc::Rc;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

unsafe impl SafeDrop for String {}
unsafe impl<T: SafeDrop + ?Sized> SafeDrop for Box<T> {}
unsafe impl<T: SafeDrop> SafeDrop for Vec<T> {}
unsafe impl<T: SafeDrop> SafeDrop for VecDeque<T> {}
unsafe impl<T: SafeDrop> SafeDrop for LinkedList<T> {}
unsafe impl<T: SafeDrop> SafeDrop for BinaryHeap<T> {}
unsafe impl<T: SafeDrop> SafeDrop for BTreeSet<T> {}
unsafe impl<K: SafeDrop, V: SafeDrop> SafeDrop for BTreeMap<K, V> {}

// SAFETY: Implemented only for trusted standard borrowed targets (`str` and `[T]`);
// the `Borrowed` variant has a no-op drop, and the `Owned` variant drops identically to `String` or `Vec<T>`.
unsafe impl SafeDrop for Cow<'_, str> {}
unsafe impl<T: SafeDrop + Clone> SafeDrop for Cow<'_, [T]> {}

// SAFETY: Dropping `Rc` only decrements the reference count; when the count reaches zero, it recursively drops the inner `T`, behaving equivalently to dropping `T`.
unsafe impl<T: SafeDrop + ?Sized> SafeDrop for Rc<T> {}

// SAFETY: Dropping `Arc` only atomically decrements the reference count; when the count reaches zero, it recursively drops the inner `T`, behaving equivalently to dropping `T`.
unsafe impl<T: SafeDrop + ?Sized> SafeDrop for Arc<T> {}
