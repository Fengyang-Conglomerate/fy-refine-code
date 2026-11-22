//! Stable implementations of `core` slice, `str`, and fixed-size array collection operations.

use super::{
    All, Any, First, Get, IsEmpty, IsNonEmpty, Last, Len, LenGt, TryFirst, TryGet, TryLast,
};
use crate::function::{Function, SafeFunction};
use crate::marker::SafeValue;
use crate::subset::Subset;

impl<'a, T> Function<&'a [T]> for Len {
    type Output = usize;

    #[inline(always)]
    fn call(input: &'a [T]) -> usize {
        input.len()
    }
}

impl<'a> Function<&'a str> for Len {
    type Output = usize;

    #[inline(always)]
    fn call(input: &'a str) -> usize {
        input.len()
    }
}

impl<'a, T, const N: usize> Function<&'a [T; N]> for Len {
    type Output = usize;

    #[inline(always)]
    fn call(_: &'a [T; N]) -> usize {
        N
    }
}

impl<'a, T> Function<&'a [T]> for IsEmpty {
    type Output = bool;

    #[inline(always)]
    fn call(input: &'a [T]) -> bool {
        input.is_empty()
    }
}

impl<'a> Function<&'a str> for IsEmpty {
    type Output = bool;

    #[inline(always)]
    fn call(input: &'a str) -> bool {
        input.is_empty()
    }
}

impl<'a, T, const N: usize> Function<&'a [T; N]> for IsEmpty {
    type Output = bool;

    #[inline(always)]
    fn call(_: &'a [T; N]) -> bool {
        N == 0
    }
}

impl<'a, T, P> Function<&'a [T]> for All<P>
where
    P: for<'b> SafeFunction<&'b T, Output = bool>,
{
    type Output = bool;

    #[inline]
    fn call(input: &'a [T]) -> bool {
        let mut i = 0;
        while i < input.len() {
            if !P::call(&input[i]) {
                return false;
            }
            i += 1;
        }
        true
    }
}

impl<'a, T, const N: usize, P> Function<&'a [T; N]> for All<P>
where
    P: for<'b> SafeFunction<&'b T, Output = bool>,
{
    type Output = bool;

    #[inline]
    fn call(input: &'a [T; N]) -> bool {
        All::<P>::call(input.as_slice())
    }
}

impl<'a, T, P> Function<&'a [T]> for Any<P>
where
    P: for<'b> SafeFunction<&'b T, Output = bool>,
{
    type Output = bool;

    #[inline]
    fn call(input: &'a [T]) -> bool {
        let mut i = 0;
        while i < input.len() {
            if P::call(&input[i]) {
                return true;
            }
            i += 1;
        }
        false
    }
}

impl<'a, T, const N: usize, P> Function<&'a [T; N]> for Any<P>
where
    P: for<'b> SafeFunction<&'b T, Output = bool>,
{
    type Output = bool;

    #[inline]
    fn call(input: &'a [T; N]) -> bool {
        Any::<P>::call(input.as_slice())
    }
}

impl<'a, T> Function<&'a [T]> for TryFirst {
    type Output = Option<&'a T>;

    #[inline(always)]
    fn call(input: &'a [T]) -> Option<&'a T> {
        input.first()
    }
}

impl<'a, T, const N: usize> Function<&'a [T; N]> for TryFirst {
    type Output = Option<&'a T>;

    #[inline(always)]
    fn call(input: &'a [T; N]) -> Option<&'a T> {
        input.first()
    }
}

impl<'a, T> Function<&'a [T]> for TryLast {
    type Output = Option<&'a T>;

    #[inline(always)]
    fn call(input: &'a [T]) -> Option<&'a T> {
        input.last()
    }
}

impl<'a, T, const N: usize> Function<&'a [T; N]> for TryLast {
    type Output = Option<&'a T>;

    #[inline(always)]
    fn call(input: &'a [T; N]) -> Option<&'a T> {
        input.last()
    }
}

impl<'a, T, const I: usize> Function<&'a [T]> for TryGet<I> {
    type Output = Option<&'a T>;

    #[inline(always)]
    fn call(input: &'a [T]) -> Option<&'a T> {
        input.get(I)
    }
}

impl<'a, T, const N: usize, const I: usize> Function<&'a [T; N]> for TryGet<I> {
    type Output = Option<&'a T>;

    #[inline(always)]
    fn call(input: &'a [T; N]) -> Option<&'a T> {
        input.get(I)
    }
}

impl<'a, T: SafeValue> Function<&'a Subset<IsNonEmpty, [T]>> for First {
    type Output = &'a T;

    #[inline(always)]
    fn call(input: &'a Subset<IsNonEmpty, [T]>) -> &'a T {
        let slice = Subset::as_inner(input);
        // SAFETY: The strong invariant of Subset<IsNonEmpty, [T]> guarantees slice.len() >= 1.
        unsafe { slice.get_unchecked(0) }
    }
}

impl<'a, T: SafeValue, const N: usize> Function<&'a Subset<IsNonEmpty, [T; N]>> for First {
    type Output = &'a T;

    #[inline(always)]
    fn call(input: &'a Subset<IsNonEmpty, [T; N]>) -> &'a T {
        let slice = Subset::as_inner(input).as_slice();
        // SAFETY: The strong invariant of Subset<IsNonEmpty, [T; N]> guarantees N >= 1.
        unsafe { slice.get_unchecked(0) }
    }
}

impl<'a, T: SafeValue> Function<&'a Subset<IsNonEmpty, [T]>> for Last {
    type Output = &'a T;

    #[inline(always)]
    fn call(input: &'a Subset<IsNonEmpty, [T]>) -> &'a T {
        let slice = Subset::as_inner(input);
        // SAFETY: The strong invariant of Subset<IsNonEmpty, [T]> guarantees slice.len() >= 1.
        unsafe { slice.get_unchecked(slice.len() - 1) }
    }
}

impl<'a, T: SafeValue, const N: usize> Function<&'a Subset<IsNonEmpty, [T; N]>> for Last {
    type Output = &'a T;

    #[inline(always)]
    fn call(input: &'a Subset<IsNonEmpty, [T; N]>) -> &'a T {
        let slice = Subset::as_inner(input).as_slice();
        // SAFETY: The strong invariant of Subset<IsNonEmpty, [T; N]> guarantees N >= 1.
        unsafe { slice.get_unchecked(N - 1) }
    }
}

impl<'a, T: SafeValue, const I: usize> Function<&'a Subset<LenGt<I>, [T]>> for Get<I> {
    type Output = &'a T;

    #[inline(always)]
    fn call(input: &'a Subset<LenGt<I>, [T]>) -> &'a T {
        let slice = Subset::as_inner(input);
        // SAFETY: The strong invariant of Subset<LenGt<I>, [T]> guarantees slice.len() > I.
        unsafe { slice.get_unchecked(I) }
    }
}

impl<'a, T: SafeValue, const N: usize, const I: usize> Function<&'a Subset<LenGt<I>, [T; N]>>
    for Get<I>
{
    type Output = &'a T;

    #[inline(always)]
    fn call(input: &'a Subset<LenGt<I>, [T; N]>) -> &'a T {
        let slice = Subset::as_inner(input).as_slice();
        // SAFETY: The strong invariant of Subset<LenGt<I>, [T; N]> guarantees N > I.
        unsafe { slice.get_unchecked(I) }
    }
}
