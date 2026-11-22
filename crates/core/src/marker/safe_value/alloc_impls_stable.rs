//! `SafeValue` contract implementations on stable Rust for `alloc`-backed standard heap containers and borrow abstractions.

use super::SafeValue;
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};
use alloc::rc::Rc;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

// SAFETY: The string buffer contents are immutable under a shared borrow.
unsafe impl SafeValue for String {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

// SAFETY: `Box` exclusively owns its heap data; when `T` satisfies `SafeValue`, the whole satisfies the contract.
unsafe impl<T: ?Sized + SafeValue> SafeValue for Box<T> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

// SAFETY: `Vec` exclusively owns its contiguous elements; when elements satisfy `SafeValue`, the whole satisfies the contract.
unsafe impl<T: SafeValue> SafeValue for Vec<T> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

// SAFETY: Sequence and ordered collection containers exclusively own all contained elements and key-value pairs.
unsafe impl<T: SafeValue> SafeValue for VecDeque<T> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

unsafe impl<T: SafeValue> SafeValue for LinkedList<T> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

unsafe impl<T: SafeValue> SafeValue for BinaryHeap<T> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

unsafe impl<T: SafeValue> SafeValue for BTreeSet<T> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

unsafe impl<K: SafeValue, V: SafeValue> SafeValue for BTreeMap<K, V> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

// SAFETY: Implemented only for trusted standard library `Cow<'a, str>` and `Cow<'a, [T]>`:
// their `Owned` types are fixed to `String` and `Vec<T>`, and standard `Borrow` is deterministic and pure,
// preventing external custom `ToOwned`/`Borrow` implementations from introducing state drift or non-deterministic views through `Deref`.
unsafe impl SafeValue for Cow<'_, str> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

unsafe impl<T: SafeValue + Clone> SafeValue for Cow<'_, [T]> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

// SAFETY: The `Rc` reference count is lifetime-management metadata and is completely transparent to read-only domain data.
unsafe impl<T: ?Sized + SafeValue> SafeValue for Rc<T> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

// SAFETY: The `Arc` atomic count is lifetime-management metadata and is completely transparent to read-only domain data.
unsafe impl<T: ?Sized + SafeValue> SafeValue for Arc<T> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}
