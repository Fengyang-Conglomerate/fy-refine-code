//! Stable topological wiring function implementations.

use super::{
    Always, AsBase, AsInner, AssocLeft, AssocRight, AttachUnit, Cloned, Copied, Discard, Dup, Fst,
    Id, IntoBase, IntoInner, MapPair, Snd, Swap, Then, WithRef,
};
use crate::function::{Function, Predicate, SafeFunction};
use crate::marker::{SafeClone, SafeValue};
use crate::subset::Subset;

impl<T> Function<T> for Id {
    type Output = T;

    #[inline(always)]
    fn call(input: T) -> T {
        input
    }
}

impl<I, F, G> Function<I> for Then<F, G>
where
    F: SafeFunction<I>,
    G: SafeFunction<F::Output>,
{
    type Output = G::Output;

    #[inline(always)]
    fn call(input: I) -> Self::Output {
        G::call(F::call(input))
    }
}

impl<A, B> Function<(A, B)> for Swap {
    type Output = (B, A);

    #[inline(always)]
    fn call((a, b): (A, B)) -> Self::Output {
        (b, a)
    }
}

impl<A, B, C> Function<(A, (B, C))> for AssocLeft {
    type Output = ((A, B), C);

    #[inline(always)]
    fn call((a, (b, c)): (A, (B, C))) -> Self::Output {
        ((a, b), c)
    }
}

impl<A, B, C> Function<((A, B), C)> for AssocRight {
    type Output = (A, (B, C));

    #[inline(always)]
    fn call(((a, b), c): ((A, B), C)) -> Self::Output {
        (a, (b, c))
    }
}

impl<A, B, F, G> Function<(A, B)> for MapPair<F, G>
where
    F: SafeFunction<A>,
    G: SafeFunction<B>,
{
    type Output = (F::Output, G::Output);

    #[inline(always)]
    fn call((a, b): (A, B)) -> Self::Output {
        (F::call(a), G::call(b))
    }
}

impl<A> Function<A> for AttachUnit {
    type Output = (A, ());

    #[inline(always)]
    fn call(input: A) -> (A, ()) {
        (input, ())
    }
}

impl<A, B> Function<(A, B)> for Fst {
    type Output = A;

    #[inline(always)]
    fn call((a, _): (A, B)) -> A {
        a
    }
}

impl<A, B> Function<(A, B)> for Snd {
    type Output = B;

    #[inline(always)]
    fn call((_, b): (A, B)) -> B {
        b
    }
}

impl<T: Copy> Function<T> for Dup {
    type Output = (T, T);

    #[inline(always)]
    fn call(input: T) -> (T, T) {
        (input, input)
    }
}

impl<T> Function<T> for Discard {
    type Output = ();

    #[inline(always)]
    fn call(_: T) {}
}

impl<Input, C: Function<()>> Function<Input> for Always<C> {
    type Output = C::Output;

    #[inline(always)]
    fn call(_: Input) -> Self::Output {
        C::call(())
    }
}

impl<'a, T: Copy> Function<&'a T> for Copied {
    type Output = T;

    #[inline(always)]
    fn call(input: &'a T) -> T {
        *input
    }
}

impl<'a, T: SafeClone> Function<&'a T> for Cloned {
    type Output = T;

    #[inline(always)]
    fn call(input: &'a T) -> T {
        input.clone()
    }
}

impl<T, O, F> Function<T> for WithRef<F>
where
    F: for<'a> SafeFunction<&'a T, Output = O>,
{
    type Output = O;

    #[inline(always)]
    fn call(input: T) -> O {
        F::call(&input)
    }
}

impl<'a, P, T: ?Sized> Function<&'a Subset<P, T>> for AsInner
where
    P: for<'b> Predicate<&'b T>,
    T: SafeValue,
{
    type Output = &'a T;

    #[inline(always)]
    fn call(input: &'a Subset<P, T>) -> &'a T {
        Subset::as_inner(input)
    }
}

impl<P, T> Function<Subset<P, T>> for IntoInner
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue,
{
    type Output = T;

    #[inline(always)]
    fn call(input: Subset<P, T>) -> T {
        Subset::into_inner(input)
    }
}

impl<'a, P, T: ?Sized> Function<&'a Subset<P, T>> for AsBase
where
    P: for<'b> Predicate<&'b T>,
    T: SafeValue,
{
    type Output = &'a T::Base;

    #[inline(always)]
    fn call(input: &'a Subset<P, T>) -> &'a T::Base {
        Subset::as_base(input)
    }
}

impl<P, T> Function<Subset<P, T>> for IntoBase
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue,
    T::Base: Sized,
{
    type Output = T::Base;

    #[inline(always)]
    fn call(input: Subset<P, T>) -> T::Base {
        Subset::into_base(input)
    }
}
