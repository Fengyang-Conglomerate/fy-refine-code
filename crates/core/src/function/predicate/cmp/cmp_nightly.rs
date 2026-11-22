//! Conditional `const` implementations of comparison predicates.

use super::{Eq, Ge, Gt, Le, Lt, Ne};
use crate::cmp::{SafeEq, SafeOrd};
use crate::function::Function;

const impl<A, B> Function<(A, B)> for Eq
where
    A: [const] SafeEq<B> + [const] core::marker::Destruct,
    B: [const] core::marker::Destruct,
{
    type Output = bool;

    #[inline(always)]
    fn call((left, right): (A, B)) -> bool {
        <A as SafeEq<B>>::eq(&left, &right)
    }
}

const impl<A, B> Function<(A, B)> for Ne
where
    A: [const] SafeEq<B> + [const] core::marker::Destruct,
    B: [const] core::marker::Destruct,
{
    type Output = bool;

    #[inline(always)]
    fn call((left, right): (A, B)) -> bool {
        <A as SafeEq<B>>::ne(&left, &right)
    }
}

const impl<A, B> Function<(A, B)> for Lt
where
    A: [const] SafeOrd<B> + [const] core::marker::Destruct,
    B: [const] core::marker::Destruct,
{
    type Output = bool;

    #[inline(always)]
    fn call((left, right): (A, B)) -> bool {
        <A as SafeOrd<B>>::lt(&left, &right)
    }
}

const impl<A, B> Function<(A, B)> for Le
where
    A: [const] SafeOrd<B> + [const] core::marker::Destruct,
    B: [const] core::marker::Destruct,
{
    type Output = bool;

    #[inline(always)]
    fn call((left, right): (A, B)) -> bool {
        <A as SafeOrd<B>>::le(&left, &right)
    }
}

const impl<A, B> Function<(A, B)> for Gt
where
    A: [const] SafeOrd<B> + [const] core::marker::Destruct,
    B: [const] core::marker::Destruct,
{
    type Output = bool;

    #[inline(always)]
    fn call((left, right): (A, B)) -> bool {
        <A as SafeOrd<B>>::gt(&left, &right)
    }
}

const impl<A, B> Function<(A, B)> for Ge
where
    A: [const] SafeOrd<B> + [const] core::marker::Destruct,
    B: [const] core::marker::Destruct,
{
    type Output = bool;

    #[inline(always)]
    fn call((left, right): (A, B)) -> bool {
        <A as SafeOrd<B>>::ge(&left, &right)
    }
}
