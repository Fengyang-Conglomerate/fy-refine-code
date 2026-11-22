//! Stable implementations of logical predicate evaluation.

use super::{And, Not, Or, Xor};
use crate::function::{Function, SafeFunction};

impl<Input, F: SafeFunction<Input, Output = bool>> Function<Input> for Not<F> {
    type Output = bool;
    #[inline]
    fn call(input: Input) -> bool {
        !F::call(input)
    }
}

impl<Input: Copy, L, R> Function<Input> for And<L, R>
where
    L: SafeFunction<Input, Output = bool>,
    R: SafeFunction<Input, Output = bool>,
{
    type Output = bool;
    #[inline]
    fn call(input: Input) -> bool {
        L::call(input) && R::call(input)
    }
}

impl<Input: Copy, L, R> Function<Input> for Or<L, R>
where
    L: SafeFunction<Input, Output = bool>,
    R: SafeFunction<Input, Output = bool>,
{
    type Output = bool;
    #[inline]
    fn call(input: Input) -> bool {
        L::call(input) || R::call(input)
    }
}

impl<Input: Copy, L, R> Function<Input> for Xor<L, R>
where
    L: SafeFunction<Input, Output = bool>,
    R: SafeFunction<Input, Output = bool>,
{
    type Output = bool;
    #[inline]
    fn call(input: Input) -> bool {
        let left = L::call(input);
        let right = R::call(input);
        left ^ right
    }
}
