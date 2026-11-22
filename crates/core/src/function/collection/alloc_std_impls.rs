//! Unified implementations of projections, quantifiers, and contracts for `alloc` and `std` collection containers.

#[cfg(feature = "alloc")]
mod alloc_impls {
    use super::super::{
        All, Any, First, Get, IsEmpty, IsNonEmpty, Last, Len, LenGt, TryFirst, TryGet, TryLast,
    };
    use crate::function::{Function, SafeFunction};
    use crate::marker::SafeValue;
    use crate::subset::Subset;
    use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};
    use alloc::string::String;
    use alloc::vec::Vec;

    macro_rules! impl_single_param_container {
        ($($ty:ident),* $(,)?) => {
            $(
                impl<'a, T> Function<&'a $ty<T>> for Len {
                    type Output = usize;
                    #[inline(always)]
                    fn call(input: &'a $ty<T>) -> usize {
                        input.len()
                    }
                }
                impl<'a, T> Function<&'a $ty<T>> for IsEmpty {
                    type Output = bool;
                    #[inline(always)]
                    fn call(input: &'a $ty<T>) -> bool {
                        input.is_empty()
                    }
                }
                impl<'a, T, P> Function<&'a $ty<T>> for All<P>
                where
                    P: for<'b> SafeFunction<&'b T, Output = bool>,
                {
                    type Output = bool;
                    #[inline]
                    fn call(input: &'a $ty<T>) -> bool {
                        input.iter().all(|item| P::call(item))
                    }
                }
                impl<'a, T, P> Function<&'a $ty<T>> for Any<P>
                where
                    P: for<'b> SafeFunction<&'b T, Output = bool>,
                {
                    type Output = bool;
                    #[inline]
                    fn call(input: &'a $ty<T>) -> bool {
                        input.iter().any(|item| P::call(item))
                    }
                }

                // SAFETY: Reads standard library collection length/emptiness (pure, terminating, panic-free); finite-length read-only traversal invokes safe predicate P.
                unsafe impl<T> SafeFunction<&$ty<T>> for Len {}
                unsafe impl<T> SafeFunction<&$ty<T>> for IsEmpty {}
                unsafe impl<T, P> SafeFunction<&$ty<T>> for All<P> where
                    P: for<'b> SafeFunction<&'b T, Output = bool>
                {
                }
                unsafe impl<T, P> SafeFunction<&$ty<T>> for Any<P> where
                    P: for<'b> SafeFunction<&'b T, Output = bool>
                {
                }
            )*
        };
    }

    impl_single_param_container!(Vec, VecDeque, LinkedList, BinaryHeap, BTreeSet);

    impl<'a> Function<&'a String> for Len {
        type Output = usize;
        #[inline(always)]
        fn call(input: &'a String) -> usize {
            input.len()
        }
    }
    impl<'a> Function<&'a String> for IsEmpty {
        type Output = bool;
        #[inline(always)]
        fn call(input: &'a String) -> bool {
            input.is_empty()
        }
    }
    // SAFETY: Reads string length and emptiness; pure, guaranteed to terminate, and panic-free.
    unsafe impl SafeFunction<&String> for Len {}
    unsafe impl SafeFunction<&String> for IsEmpty {}

    impl<'a, K, V> Function<&'a BTreeMap<K, V>> for Len {
        type Output = usize;
        #[inline(always)]
        fn call(input: &'a BTreeMap<K, V>) -> usize {
            input.len()
        }
    }
    impl<'a, K, V> Function<&'a BTreeMap<K, V>> for IsEmpty {
        type Output = bool;
        #[inline(always)]
        fn call(input: &'a BTreeMap<K, V>) -> bool {
            input.is_empty()
        }
    }
    impl<'a, K, V, P> Function<&'a BTreeMap<K, V>> for All<P>
    where
        P: for<'b> SafeFunction<(&'b K, &'b V), Output = bool>,
    {
        type Output = bool;
        #[inline]
        fn call(input: &'a BTreeMap<K, V>) -> bool {
            input.iter().all(|entry| P::call(entry))
        }
    }
    impl<'a, K, V, P> Function<&'a BTreeMap<K, V>> for Any<P>
    where
        P: for<'b> SafeFunction<(&'b K, &'b V), Output = bool>,
    {
        type Output = bool;
        #[inline]
        fn call(input: &'a BTreeMap<K, V>) -> bool {
            input.iter().any(|entry| P::call(entry))
        }
    }
    // SAFETY: Reads ordered map length and emptiness; finite-length read-only traversal invokes safe predicate P.
    unsafe impl<K, V> SafeFunction<&BTreeMap<K, V>> for Len {}
    unsafe impl<K, V> SafeFunction<&BTreeMap<K, V>> for IsEmpty {}
    unsafe impl<K, V, P> SafeFunction<&BTreeMap<K, V>> for All<P> where
        P: for<'b> SafeFunction<(&'b K, &'b V), Output = bool>
    {
    }
    unsafe impl<K, V, P> SafeFunction<&BTreeMap<K, V>> for Any<P> where
        P: for<'b> SafeFunction<(&'b K, &'b V), Output = bool>
    {
    }

    // --- First/last and indexed access ---

    impl<'a, T> Function<&'a Vec<T>> for TryFirst {
        type Output = Option<&'a T>;
        #[inline(always)]
        fn call(input: &'a Vec<T>) -> Option<&'a T> {
            input.first()
        }
    }
    impl<'a, T> Function<&'a Vec<T>> for TryLast {
        type Output = Option<&'a T>;
        #[inline(always)]
        fn call(input: &'a Vec<T>) -> Option<&'a T> {
            input.last()
        }
    }
    impl<'a, T, const I: usize> Function<&'a Vec<T>> for TryGet<I> {
        type Output = Option<&'a T>;
        #[inline(always)]
        fn call(input: &'a Vec<T>) -> Option<&'a T> {
            input.get(I)
        }
    }

    impl<'a, T> Function<&'a VecDeque<T>> for TryFirst {
        type Output = Option<&'a T>;
        #[inline(always)]
        fn call(input: &'a VecDeque<T>) -> Option<&'a T> {
            input.front()
        }
    }
    impl<'a, T> Function<&'a VecDeque<T>> for TryLast {
        type Output = Option<&'a T>;
        #[inline(always)]
        fn call(input: &'a VecDeque<T>) -> Option<&'a T> {
            input.back()
        }
    }
    impl<'a, T, const I: usize> Function<&'a VecDeque<T>> for TryGet<I> {
        type Output = Option<&'a T>;
        #[inline(always)]
        fn call(input: &'a VecDeque<T>) -> Option<&'a T> {
            input.get(I)
        }
    }

    impl<'a, T> Function<&'a LinkedList<T>> for TryFirst {
        type Output = Option<&'a T>;
        #[inline(always)]
        fn call(input: &'a LinkedList<T>) -> Option<&'a T> {
            input.front()
        }
    }
    impl<'a, T> Function<&'a LinkedList<T>> for TryLast {
        type Output = Option<&'a T>;
        #[inline(always)]
        fn call(input: &'a LinkedList<T>) -> Option<&'a T> {
            input.back()
        }
    }

    impl<'a, T> Function<&'a BTreeSet<T>> for TryFirst {
        type Output = Option<&'a T>;
        #[inline(always)]
        fn call(input: &'a BTreeSet<T>) -> Option<&'a T> {
            input.iter().next()
        }
    }
    impl<'a, T> Function<&'a BTreeSet<T>> for TryLast {
        type Output = Option<&'a T>;
        #[inline(always)]
        fn call(input: &'a BTreeSet<T>) -> Option<&'a T> {
            input.iter().next_back()
        }
    }

    impl<'a, K, V> Function<&'a BTreeMap<K, V>> for TryFirst {
        type Output = Option<(&'a K, &'a V)>;
        #[inline(always)]
        fn call(input: &'a BTreeMap<K, V>) -> Option<(&'a K, &'a V)> {
            input.iter().next()
        }
    }
    impl<'a, K, V> Function<&'a BTreeMap<K, V>> for TryLast {
        type Output = Option<(&'a K, &'a V)>;
        #[inline(always)]
        fn call(input: &'a BTreeMap<K, V>) -> Option<(&'a K, &'a V)> {
            input.iter().next_back()
        }
    }

    // SAFETY: Read-only borrowing of first/last elements and bounds-checked indexing on ordered/double-ended containers; pure, deterministic, guaranteed to terminate, and panic-free.
    unsafe impl<T> SafeFunction<&Vec<T>> for TryFirst {}
    unsafe impl<T> SafeFunction<&VecDeque<T>> for TryFirst {}
    unsafe impl<T> SafeFunction<&LinkedList<T>> for TryFirst {}
    unsafe impl<T> SafeFunction<&BTreeSet<T>> for TryFirst {}
    unsafe impl<K, V> SafeFunction<&BTreeMap<K, V>> for TryFirst {}

    unsafe impl<T> SafeFunction<&Vec<T>> for TryLast {}
    unsafe impl<T> SafeFunction<&VecDeque<T>> for TryLast {}
    unsafe impl<T> SafeFunction<&LinkedList<T>> for TryLast {}
    unsafe impl<T> SafeFunction<&BTreeSet<T>> for TryLast {}
    unsafe impl<K, V> SafeFunction<&BTreeMap<K, V>> for TryLast {}

    unsafe impl<T, const I: usize> SafeFunction<&Vec<T>> for TryGet<I> {}
    unsafe impl<T, const I: usize> SafeFunction<&VecDeque<T>> for TryGet<I> {}

    impl<'a, T: SafeValue> Function<&'a Subset<IsNonEmpty, Vec<T>>> for First {
        type Output = &'a T;
        #[inline(always)]
        fn call(input: &'a Subset<IsNonEmpty, Vec<T>>) -> &'a T {
            let slice = Subset::as_inner(input).as_slice();
            // SAFETY: The strong invariant of Subset<IsNonEmpty, Vec<T>> guarantees len >= 1.
            unsafe { slice.get_unchecked(0) }
        }
    }
    impl<'a, T: SafeValue> Function<&'a Subset<IsNonEmpty, Vec<T>>> for Last {
        type Output = &'a T;
        #[inline(always)]
        fn call(input: &'a Subset<IsNonEmpty, Vec<T>>) -> &'a T {
            let slice = Subset::as_inner(input).as_slice();
            // SAFETY: The strong invariant of Subset<IsNonEmpty, Vec<T>> guarantees len >= 1.
            unsafe { slice.get_unchecked(slice.len() - 1) }
        }
    }
    impl<'a, T: SafeValue, const I: usize> Function<&'a Subset<LenGt<I>, Vec<T>>> for Get<I> {
        type Output = &'a T;
        #[inline(always)]
        fn call(input: &'a Subset<LenGt<I>, Vec<T>>) -> &'a T {
            let slice = Subset::as_inner(input).as_slice();
            // SAFETY: The strong invariant of Subset<LenGt<I>, Vec<T>> guarantees len > I.
            unsafe { slice.get_unchecked(I) }
        }
    }

    impl<'a, T: SafeValue> Function<&'a Subset<IsNonEmpty, VecDeque<T>>> for First {
        type Output = &'a T;
        #[inline(always)]
        fn call(input: &'a Subset<IsNonEmpty, VecDeque<T>>) -> &'a T {
            match Subset::as_inner(input).front() {
                Some(v) => v,
                // SAFETY: The strong invariant of Subset<IsNonEmpty, VecDeque<T>> guarantees non-emptiness.
                None => unsafe { core::hint::unreachable_unchecked() },
            }
        }
    }
    impl<'a, T: SafeValue> Function<&'a Subset<IsNonEmpty, VecDeque<T>>> for Last {
        type Output = &'a T;
        #[inline(always)]
        fn call(input: &'a Subset<IsNonEmpty, VecDeque<T>>) -> &'a T {
            match Subset::as_inner(input).back() {
                Some(v) => v,
                // SAFETY: The strong invariant of Subset<IsNonEmpty, VecDeque<T>> guarantees non-emptiness.
                None => unsafe { core::hint::unreachable_unchecked() },
            }
        }
    }
    impl<'a, T: SafeValue, const I: usize> Function<&'a Subset<LenGt<I>, VecDeque<T>>> for Get<I> {
        type Output = &'a T;
        #[inline(always)]
        fn call(input: &'a Subset<LenGt<I>, VecDeque<T>>) -> &'a T {
            match Subset::as_inner(input).get(I) {
                Some(v) => v,
                // SAFETY: The strong invariant of Subset<LenGt<I>, VecDeque<T>> guarantees len > I.
                None => unsafe { core::hint::unreachable_unchecked() },
            }
        }
    }

    impl<'a, T: SafeValue> Function<&'a Subset<IsNonEmpty, LinkedList<T>>> for First {
        type Output = &'a T;
        #[inline(always)]
        fn call(input: &'a Subset<IsNonEmpty, LinkedList<T>>) -> &'a T {
            match Subset::as_inner(input).front() {
                Some(v) => v,
                // SAFETY: The strong invariant of Subset<IsNonEmpty, LinkedList<T>> guarantees non-emptiness.
                None => unsafe { core::hint::unreachable_unchecked() },
            }
        }
    }
    impl<'a, T: SafeValue> Function<&'a Subset<IsNonEmpty, LinkedList<T>>> for Last {
        type Output = &'a T;
        #[inline(always)]
        fn call(input: &'a Subset<IsNonEmpty, LinkedList<T>>) -> &'a T {
            match Subset::as_inner(input).back() {
                Some(v) => v,
                // SAFETY: The strong invariant of Subset<IsNonEmpty, LinkedList<T>> guarantees non-emptiness.
                None => unsafe { core::hint::unreachable_unchecked() },
            }
        }
    }

    impl<'a, T: SafeValue> Function<&'a Subset<IsNonEmpty, BTreeSet<T>>> for First {
        type Output = &'a T;
        #[inline(always)]
        fn call(input: &'a Subset<IsNonEmpty, BTreeSet<T>>) -> &'a T {
            match Subset::as_inner(input).iter().next() {
                Some(v) => v,
                // SAFETY: The strong invariant of Subset<IsNonEmpty, BTreeSet<T>> guarantees non-emptiness.
                None => unsafe { core::hint::unreachable_unchecked() },
            }
        }
    }
    impl<'a, T: SafeValue> Function<&'a Subset<IsNonEmpty, BTreeSet<T>>> for Last {
        type Output = &'a T;
        #[inline(always)]
        fn call(input: &'a Subset<IsNonEmpty, BTreeSet<T>>) -> &'a T {
            match Subset::as_inner(input).iter().next_back() {
                Some(v) => v,
                // SAFETY: The strong invariant of Subset<IsNonEmpty, BTreeSet<T>> guarantees non-emptiness.
                None => unsafe { core::hint::unreachable_unchecked() },
            }
        }
    }

    impl<'a, K: SafeValue, V: SafeValue> Function<&'a Subset<IsNonEmpty, BTreeMap<K, V>>> for First {
        type Output = (&'a K, &'a V);
        #[inline(always)]
        fn call(input: &'a Subset<IsNonEmpty, BTreeMap<K, V>>) -> (&'a K, &'a V) {
            match Subset::as_inner(input).iter().next() {
                Some(entry) => entry,
                // SAFETY: The strong invariant of Subset<IsNonEmpty, BTreeMap<K, V>> guarantees non-emptiness.
                None => unsafe { core::hint::unreachable_unchecked() },
            }
        }
    }
    impl<'a, K: SafeValue, V: SafeValue> Function<&'a Subset<IsNonEmpty, BTreeMap<K, V>>> for Last {
        type Output = (&'a K, &'a V);
        #[inline(always)]
        fn call(input: &'a Subset<IsNonEmpty, BTreeMap<K, V>>) -> (&'a K, &'a V) {
            match Subset::as_inner(input).iter().next_back() {
                Some(entry) => entry,
                // SAFETY: The strong invariant of Subset<IsNonEmpty, BTreeMap<K, V>> guarantees non-emptiness.
                None => unsafe { core::hint::unreachable_unchecked() },
            }
        }
    }

    // SAFETY: The strong invariants of Subset<IsNonEmpty / LenGt<I>, _> guarantee that the target element exists.
    unsafe impl<T: SafeValue> SafeFunction<&Subset<IsNonEmpty, Vec<T>>> for First {}
    unsafe impl<T: SafeValue> SafeFunction<&Subset<IsNonEmpty, VecDeque<T>>> for First {}
    unsafe impl<T: SafeValue> SafeFunction<&Subset<IsNonEmpty, LinkedList<T>>> for First {}
    unsafe impl<T: SafeValue> SafeFunction<&Subset<IsNonEmpty, BTreeSet<T>>> for First {}
    unsafe impl<K: SafeValue, V: SafeValue> SafeFunction<&Subset<IsNonEmpty, BTreeMap<K, V>>>
        for First
    {
    }

    unsafe impl<T: SafeValue> SafeFunction<&Subset<IsNonEmpty, Vec<T>>> for Last {}
    unsafe impl<T: SafeValue> SafeFunction<&Subset<IsNonEmpty, VecDeque<T>>> for Last {}
    unsafe impl<T: SafeValue> SafeFunction<&Subset<IsNonEmpty, LinkedList<T>>> for Last {}
    unsafe impl<T: SafeValue> SafeFunction<&Subset<IsNonEmpty, BTreeSet<T>>> for Last {}
    unsafe impl<K: SafeValue, V: SafeValue> SafeFunction<&Subset<IsNonEmpty, BTreeMap<K, V>>> for Last {}

    unsafe impl<T: SafeValue, const I: usize> SafeFunction<&Subset<LenGt<I>, Vec<T>>> for Get<I> {}
    unsafe impl<T: SafeValue, const I: usize> SafeFunction<&Subset<LenGt<I>, VecDeque<T>>> for Get<I> {}
}

#[cfg(feature = "std")]
mod std_impls {
    use super::super::{All, Any, IsEmpty, Len};
    use crate::function::{Function, SafeFunction};
    use std::collections::{HashMap, HashSet};

    impl<'a, T, S> Function<&'a HashSet<T, S>> for Len {
        type Output = usize;
        #[inline(always)]
        fn call(input: &'a HashSet<T, S>) -> usize {
            input.len()
        }
    }
    impl<'a, T, S> Function<&'a HashSet<T, S>> for IsEmpty {
        type Output = bool;
        #[inline(always)]
        fn call(input: &'a HashSet<T, S>) -> bool {
            input.is_empty()
        }
    }
    impl<'a, T, S, P> Function<&'a HashSet<T, S>> for All<P>
    where
        P: for<'b> SafeFunction<&'b T, Output = bool>,
    {
        type Output = bool;
        #[inline]
        fn call(input: &'a HashSet<T, S>) -> bool {
            input.iter().all(|item| P::call(item))
        }
    }
    impl<'a, T, S, P> Function<&'a HashSet<T, S>> for Any<P>
    where
        P: for<'b> SafeFunction<&'b T, Output = bool>,
    {
        type Output = bool;
        #[inline]
        fn call(input: &'a HashSet<T, S>) -> bool {
            input.iter().any(|item| P::call(item))
        }
    }

    impl<'a, K, V, S> Function<&'a HashMap<K, V, S>> for Len {
        type Output = usize;
        #[inline(always)]
        fn call(input: &'a HashMap<K, V, S>) -> usize {
            input.len()
        }
    }
    impl<'a, K, V, S> Function<&'a HashMap<K, V, S>> for IsEmpty {
        type Output = bool;
        #[inline(always)]
        fn call(input: &'a HashMap<K, V, S>) -> bool {
            input.is_empty()
        }
    }
    impl<'a, K, V, S, P> Function<&'a HashMap<K, V, S>> for All<P>
    where
        P: for<'b> SafeFunction<(&'b K, &'b V), Output = bool>,
    {
        type Output = bool;
        #[inline]
        fn call(input: &'a HashMap<K, V, S>) -> bool {
            input.iter().all(|entry| P::call(entry))
        }
    }
    impl<'a, K, V, S, P> Function<&'a HashMap<K, V, S>> for Any<P>
    where
        P: for<'b> SafeFunction<(&'b K, &'b V), Output = bool>,
    {
        type Output = bool;
        #[inline]
        fn call(input: &'a HashMap<K, V, S>) -> bool {
            input.iter().any(|entry| P::call(entry))
        }
    }

    // SAFETY: Reads hash collection element count (pure, terminating, panic-free); universal and existential quantifiers over a pure safe predicate are independent of traversal order.
    unsafe impl<T, S> SafeFunction<&HashSet<T, S>> for Len {}
    unsafe impl<K, V, S> SafeFunction<&HashMap<K, V, S>> for Len {}
    unsafe impl<T, S> SafeFunction<&HashSet<T, S>> for IsEmpty {}
    unsafe impl<K, V, S> SafeFunction<&HashMap<K, V, S>> for IsEmpty {}
    unsafe impl<T, S, P> SafeFunction<&HashSet<T, S>> for All<P> where
        P: for<'b> SafeFunction<&'b T, Output = bool>
    {
    }
    unsafe impl<K, V, S, P> SafeFunction<&HashMap<K, V, S>> for All<P> where
        P: for<'b> SafeFunction<(&'b K, &'b V), Output = bool>
    {
    }
    unsafe impl<T, S, P> SafeFunction<&HashSet<T, S>> for Any<P> where
        P: for<'b> SafeFunction<&'b T, Output = bool>
    {
    }
    unsafe impl<K, V, S, P> SafeFunction<&HashMap<K, V, S>> for Any<P> where
        P: for<'b> SafeFunction<(&'b K, &'b V), Output = bool>
    {
    }
}
