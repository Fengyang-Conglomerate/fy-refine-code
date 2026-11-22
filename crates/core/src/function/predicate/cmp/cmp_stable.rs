//! Stable implementations of comparison predicates.

use super::{Eq, Ge, Gt, Le, Lt, Ne};
use crate::cmp::{SafeEq, SafeOrd};
use crate::function::Function;

impl<A: SafeEq<B>, B> Function<(A, B)> for Eq {
    type Output = bool;

    #[inline(always)]
    fn call((left, right): (A, B)) -> bool {
        <A as SafeEq<B>>::eq(&left, &right)
    }
}

impl<A: SafeEq<B>, B> Function<(A, B)> for Ne {
    type Output = bool;

    #[inline(always)]
    fn call((left, right): (A, B)) -> bool {
        <A as SafeEq<B>>::ne(&left, &right)
    }
}

impl<A: SafeOrd<B>, B> Function<(A, B)> for Lt {
    type Output = bool;

    #[inline(always)]
    fn call((left, right): (A, B)) -> bool {
        <A as SafeOrd<B>>::lt(&left, &right)
    }
}

impl<A: SafeOrd<B>, B> Function<(A, B)> for Le {
    type Output = bool;

    #[inline(always)]
    fn call((left, right): (A, B)) -> bool {
        <A as SafeOrd<B>>::le(&left, &right)
    }
}

impl<A: SafeOrd<B>, B> Function<(A, B)> for Gt {
    type Output = bool;

    #[inline(always)]
    fn call((left, right): (A, B)) -> bool {
        <A as SafeOrd<B>>::gt(&left, &right)
    }
}

impl<A: SafeOrd<B>, B> Function<(A, B)> for Ge {
    type Output = bool;

    #[inline(always)]
    fn call((left, right): (A, B)) -> bool {
        <A as SafeOrd<B>>::ge(&left, &right)
    }
}
