//! Logical predicates evaluable at compile time.
//!
//! ```
//! #![feature(const_trait_impl)]
//! use fy_refine_core::function::Function;
//! use fy_refine_core::function::predicate::{And, False, Not, Or, True, Xor};
//! use fy_refine_core::function::wiring::Always;
//! const VALUE: bool = And::<Always<True>, Not<Always<False>>>::call(());
//! const OTHER: bool = Xor::<Or<Always<False>, Always<True>>, Always<True>>::call(());
//! assert_eq!((VALUE, OTHER), (true, false));
//! ```

use super::{And, Not, Or, Xor};
use crate::function::{Function, SafeFunction};

const impl<Input, F> Function<Input> for Not<F>
where
    F: SafeFunction<Input, Output = bool> + [const] Function<Input, Output = bool>,
{
    type Output = bool;
    #[inline]
    fn call(input: Input) -> bool {
        !F::call(input)
    }
}

const impl<Input: Copy + [const] core::marker::Destruct, L, R> Function<Input> for And<L, R>
where
    L: SafeFunction<Input, Output = bool> + [const] Function<Input, Output = bool>,
    R: SafeFunction<Input, Output = bool> + [const] Function<Input, Output = bool>,
{
    type Output = bool;
    #[inline]
    fn call(input: Input) -> bool {
        L::call(input) && R::call(input)
    }
}

const impl<Input: Copy + [const] core::marker::Destruct, L, R> Function<Input> for Or<L, R>
where
    L: SafeFunction<Input, Output = bool> + [const] Function<Input, Output = bool>,
    R: SafeFunction<Input, Output = bool> + [const] Function<Input, Output = bool>,
{
    type Output = bool;
    #[inline]
    fn call(input: Input) -> bool {
        L::call(input) || R::call(input)
    }
}

const impl<Input: Copy + [const] core::marker::Destruct, L, R> Function<Input> for Xor<L, R>
where
    L: SafeFunction<Input, Output = bool> + [const] Function<Input, Output = bool>,
    R: SafeFunction<Input, Output = bool> + [const] Function<Input, Output = bool>,
{
    type Output = bool;
    #[inline]
    fn call(input: Input) -> bool {
        L::call(input) ^ R::call(input)
    }
}
