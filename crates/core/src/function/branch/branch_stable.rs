//! Stable branch matching, pattern elimination, and sum-type bridge implementations.

use super::{
    BranchOpt, BranchOptPure, BranchRes, BranchResPure, Classify, IfElse, IsErr, IsNone, IsOk,
    IsSome, IsVariant, OptAsRef, ResAsRef, ResErr, ResOk, UnwrapErr, UnwrapOk, UnwrapSome,
    UnwrapVariant, ZipOpt, ZipRes,
};
use crate::function::predicate::{Not, Predicate};
use crate::function::{Function, SafeFunction};
use crate::marker::SafeValue;
use crate::subset::Subset;

impl<T, C, O, S, N> Function<(Option<T>, C)> for BranchOpt<S, N>
where
    S: SafeFunction<(T, C), Output = O>,
    N: SafeFunction<C, Output = O>,
{
    type Output = O;

    #[inline]
    fn call((option, context): (Option<T>, C)) -> O {
        match option {
            Some(value) => S::call((value, context)),
            None => N::call(context),
        }
    }
}

impl<T, E, C, O, S, N> Function<(Result<T, E>, C)> for BranchRes<S, N>
where
    S: SafeFunction<(T, C), Output = O>,
    N: SafeFunction<(E, C), Output = O>,
{
    type Output = O;

    #[inline]
    fn call((result, context): (Result<T, E>, C)) -> O {
        match result {
            Ok(value) => S::call((value, context)),
            Err(err) => N::call((err, context)),
        }
    }
}

impl<T, O, S, N> Function<Option<T>> for BranchOptPure<S, N>
where
    S: SafeFunction<T, Output = O>,
    N: SafeFunction<(), Output = O>,
{
    type Output = O;

    #[inline]
    fn call(option: Option<T>) -> O {
        match option {
            Some(value) => S::call(value),
            None => N::call(()),
        }
    }
}

impl<T, E, O, S, N> Function<Result<T, E>> for BranchResPure<S, N>
where
    S: SafeFunction<T, Output = O>,
    N: SafeFunction<E, Output = O>,
{
    type Output = O;

    #[inline]
    fn call(result: Result<T, E>) -> O {
        match result {
            Ok(value) => S::call(value),
            Err(err) => N::call(err),
        }
    }
}

impl<I: Copy, O, C, T, E> Function<I> for IfElse<C, T, E>
where
    C: SafeFunction<I, Output = bool>,
    T: SafeFunction<I, Output = O>,
    E: SafeFunction<I, Output = O>,
{
    type Output = O;

    #[inline]
    fn call(input: I) -> O {
        if C::call(input) {
            T::call(input)
        } else {
            E::call(input)
        }
    }
}

impl<'a, P, T: ?Sized> Function<&'a T> for Classify<P>
where
    P: for<'b> Predicate<&'b T> + 'a,
    T: SafeValue,
{
    type Output = Result<&'a Subset<P, T>, &'a Subset<Not<P>, T>>;

    #[inline(always)]
    fn call(input: &'a T) -> Self::Output {
        if P::call(input) {
            // SAFETY: Just verified `P::call(input) == true`, and `T: SafeValue` prevents drift.
            Ok(unsafe { Subset::from_ref_unchecked(input) })
        } else {
            // SAFETY: `P` is a pure total predicate, so `P::call(input) == false` is equivalent to `Not<P>::call(input) == true`.
            Err(unsafe { Subset::from_ref_unchecked(input) })
        }
    }
}

impl<'a, T> Function<&'a Option<T>> for OptAsRef {
    type Output = Option<&'a T>;

    #[inline(always)]
    fn call(input: &'a Option<T>) -> Option<&'a T> {
        input.as_ref()
    }
}

impl<'a, T, E> Function<&'a Result<T, E>> for ResAsRef {
    type Output = Result<&'a T, &'a E>;

    #[inline(always)]
    fn call(input: &'a Result<T, E>) -> Result<&'a T, &'a E> {
        input.as_ref()
    }
}

impl<T, E> Function<Result<T, E>> for ResOk {
    type Output = Option<T>;

    #[inline(always)]
    fn call(input: Result<T, E>) -> Option<T> {
        input.ok()
    }
}

impl<'a, T, E> Function<&'a Result<T, E>> for ResOk {
    type Output = Option<&'a T>;

    #[inline(always)]
    fn call(input: &'a Result<T, E>) -> Option<&'a T> {
        input.as_ref().ok()
    }
}

impl<T, E> Function<Result<T, E>> for ResErr {
    type Output = Option<E>;

    #[inline(always)]
    fn call(input: Result<T, E>) -> Option<E> {
        input.err()
    }
}

impl<'a, T, E> Function<&'a Result<T, E>> for ResErr {
    type Output = Option<&'a E>;

    #[inline(always)]
    fn call(input: &'a Result<T, E>) -> Option<&'a E> {
        input.as_ref().err()
    }
}

impl<A, B> Function<(Option<A>, Option<B>)> for ZipOpt {
    type Output = Option<(A, B)>;

    #[inline(always)]
    fn call((a, b): (Option<A>, Option<B>)) -> Option<(A, B)> {
        match (a, b) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        }
    }
}

impl<A, B, E> Function<(Result<A, E>, Result<B, E>)> for ZipRes {
    type Output = Result<(A, B), E>;

    #[inline(always)]
    fn call((a, b): (Result<A, E>, Result<B, E>)) -> Result<(A, B), E> {
        match (a, b) {
            (Ok(a), Ok(b)) => Ok((a, b)),
            (Err(e), _) => Err(e),
            (Ok(_), Err(e)) => Err(e),
        }
    }
}

impl<T> Function<Option<T>> for IsSome {
    type Output = bool;

    #[inline(always)]
    fn call(input: Option<T>) -> bool {
        input.is_some()
    }
}

impl<'a, T> Function<&'a Option<T>> for IsSome {
    type Output = bool;

    #[inline(always)]
    fn call(input: &'a Option<T>) -> bool {
        input.is_some()
    }
}

impl<T> Function<Option<T>> for IsNone {
    type Output = bool;

    #[inline(always)]
    fn call(input: Option<T>) -> bool {
        input.is_none()
    }
}

impl<'a, T> Function<&'a Option<T>> for IsNone {
    type Output = bool;

    #[inline(always)]
    fn call(input: &'a Option<T>) -> bool {
        input.is_none()
    }
}

impl<T, E> Function<Result<T, E>> for IsOk {
    type Output = bool;

    #[inline(always)]
    fn call(input: Result<T, E>) -> bool {
        input.is_ok()
    }
}

impl<'a, T, E> Function<&'a Result<T, E>> for IsOk {
    type Output = bool;

    #[inline(always)]
    fn call(input: &'a Result<T, E>) -> bool {
        input.is_ok()
    }
}

impl<T, E> Function<Result<T, E>> for IsErr {
    type Output = bool;

    #[inline(always)]
    fn call(input: Result<T, E>) -> bool {
        input.is_err()
    }
}

impl<'a, T, E> Function<&'a Result<T, E>> for IsErr {
    type Output = bool;

    #[inline(always)]
    fn call(input: &'a Result<T, E>) -> bool {
        input.is_err()
    }
}

impl<T: SafeValue> Function<Subset<IsSome, Option<T>>> for UnwrapSome {
    type Output = T;

    #[inline(always)]
    fn call(input: Subset<IsSome, Option<T>>) -> T {
        match Subset::into_inner(input) {
            Some(value) => value,
            // SAFETY: The Subset<IsSome, Option<T>> invariant guarantees the inner value is Some.
            None => unsafe { core::hint::unreachable_unchecked() },
        }
    }
}

impl<'a, T: SafeValue> Function<&'a Subset<IsSome, Option<T>>> for UnwrapSome {
    type Output = &'a T;

    #[inline(always)]
    fn call(input: &'a Subset<IsSome, Option<T>>) -> &'a T {
        match Subset::as_inner(input) {
            Some(value) => value,
            // SAFETY: The Subset<IsSome, Option<T>> invariant guarantees the inner value is Some.
            None => unsafe { core::hint::unreachable_unchecked() },
        }
    }
}

impl<T: SafeValue, E: SafeValue> Function<Subset<IsOk, Result<T, E>>> for UnwrapOk {
    type Output = T;

    #[inline(always)]
    fn call(input: Subset<IsOk, Result<T, E>>) -> T {
        match Subset::into_inner(input) {
            Ok(value) => value,
            // SAFETY: The Subset<IsOk, Result<T, E>> invariant guarantees the inner value is Ok.
            Err(_) => unsafe { core::hint::unreachable_unchecked() },
        }
    }
}

impl<'a, T: SafeValue, E: SafeValue> Function<&'a Subset<IsOk, Result<T, E>>> for UnwrapOk {
    type Output = &'a T;

    #[inline(always)]
    fn call(input: &'a Subset<IsOk, Result<T, E>>) -> &'a T {
        match Subset::as_inner(input) {
            Ok(value) => value,
            // SAFETY: The Subset<IsOk, Result<T, E>> invariant guarantees the inner value is Ok.
            Err(_) => unsafe { core::hint::unreachable_unchecked() },
        }
    }
}

impl<T: SafeValue, E: SafeValue> Function<Subset<IsErr, Result<T, E>>> for UnwrapErr {
    type Output = E;

    #[inline(always)]
    fn call(input: Subset<IsErr, Result<T, E>>) -> E {
        match Subset::into_inner(input) {
            // SAFETY: The Subset<IsErr, Result<T, E>> invariant guarantees the inner value is Err.
            Ok(_) => unsafe { core::hint::unreachable_unchecked() },
            Err(err) => err,
        }
    }
}

impl<'a, T: SafeValue, E: SafeValue> Function<&'a Subset<IsErr, Result<T, E>>> for UnwrapErr {
    type Output = &'a E;

    #[inline(always)]
    fn call(input: &'a Subset<IsErr, Result<T, E>>) -> &'a E {
        match Subset::as_inner(input) {
            // SAFETY: The Subset<IsErr, Result<T, E>> invariant guarantees the inner value is Err.
            Ok(_) => unsafe { core::hint::unreachable_unchecked() },
            Err(err) => err,
        }
    }
}

impl<'a, T, V, Payload> Function<&'a Subset<IsVariant<V>, T>> for UnwrapVariant<V>
where
    T: ?Sized + SafeValue,
    IsVariant<V>: for<'b> Predicate<&'b T>,
    V: SafeFunction<&'a T, Output = Option<Payload>>,
{
    type Output = Payload;

    #[inline(always)]
    fn call(input: &'a Subset<IsVariant<V>, T>) -> Payload {
        match V::call(Subset::as_inner(input)) {
            Some(payload) => payload,
            // SAFETY: The Subset<IsVariant<V>, T> invariant guarantees V(&T) returns Some.
            None => unsafe { core::hint::unreachable_unchecked() },
        }
    }
}
