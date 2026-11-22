//! Logical connectives, boolean constant aliases, and flat multi-way combinator aliases for pure boolean functions.
//!
//! This module provides minimal orthogonal boolean logic primitives—[`Not`], [`And`], [`Or`], [`Xor`]—along with transparent combinator aliases:
//! - Boolean constants: [`True`] = `ConstBool<true>`, [`False`] = `ConstBool<false>`;
//! - Flat multi-way union/intersection: [`AnyOf`] (expands to nested `Or`), [`AllOf`] (expands to nested `And`).
//!
//! ## Extensional Semantics and Evaluation Order
//!
//! All sub-predicates satisfy [`SafeFunction`] (pure, total, guaranteed to terminate, and free of destructive side effects).
//! Therefore, the extensional truth value of [`And`], [`Or`], and [`Xor`] depends solely on the boolean results of both sub-predicates, **without promising a fixed evaluation order to callers**.
//! In the current default implementation, [`And`] and [`Or`] still short-circuit (`L && R` / `L || R`) to avoid unnecessary right-hand evaluation,
//! but no code should rely on any side-effect assumption that "the left side is always called before the right side".
//!
//! If you already have a `(bool, bool)` 2-tuple, simply compose `And<Fst, Snd>`, `Or<Fst, Snd>`, or `Xor<Fst, Snd>`;
//! no separate scalar boolean primitives are needed.
//!
//! ```
//! use fy_refine_core::function::{Function, SafeFunction};
//! use fy_refine_core::function::constant::ConstU8;
//! use fy_refine_core::function::predicate::{
//!     AllOf, And, AnyOf, EqConst, False, Not, Or, True, Xor,
//! };
//! use fy_refine_core::function::wiring::{Always, Fst, Snd};
//!
//! struct Data(bool);
//! struct Flag;
//! impl Function<&Data> for Flag {
//!     type Output = bool;
//!     fn call(input: &Data) -> bool { input.0 }
//! }
//! // SAFETY: Only reads a boolean field; pure, total, terminating, and panic-free.
//! unsafe impl SafeFunction<&Data> for Flag {}
//!
//! for flag in [false, true] {
//!     let data = Data(flag);
//!     assert_eq!(And::<Flag, Always<True>>::call(&data), flag);
//!     assert_eq!(Or::<Flag, Always<False>>::call(&data), flag);
//!     assert_eq!(Xor::<Flag, Always<True>>::call(&data), !flag);
//!     assert_eq!(Not::<Flag>::call(&data), !flag);
//!     assert_eq!(And::<Flag, Not<Always<False>>>::call(&data), flag);
//! }
//!
//! // (bool, bool) 2-tuples can be reduced directly with And<Fst, Snd> / Or<Fst, Snd> / Xor<Fst, Snd>
//! assert!(And::<Fst, Snd>::call((true, true)));
//! assert!(!And::<Fst, Snd>::call((true, false)));
//! assert!(Or::<Fst, Snd>::call((false, true)));
//! assert!(Xor::<Fst, Snd>::call((true, false)));
//!
//! // Flat multi-way union and intersection (transparently expands to Or / And)
//! type Vowel = AnyOf<(EqConst<ConstU8<b'a'>>, EqConst<ConstU8<b'e'>>, EqConst<ConstU8<b'i'>>)>;
//! assert!(Vowel::call(&b'e'));
//! assert!(!Vowel::call(&b'z'));
//! assert!(AllOf::<(Always<True>, Always<True>, Always<True>)>::call(&0u8));
//! ```

use crate::function::SafeFunction;
use crate::function::constant::ConstBool;
use crate::function::wiring::Always;
use crate::marker::Phantom;

#[cfg(feature = "nightly")]
mod logic_nightly;
#[cfg(not(feature = "nightly"))]
mod logic_stable;

/// Pure constant yielding `true` (transparent alias for `ConstBool<true>`, taking unit `()`).
pub type True = ConstBool<true>;

/// Pure constant yielding `false` (transparent alias for `ConstBool<false>`, taking unit `()`).
pub type False = ConstBool<false>;

/// Negates the boolean result of pure function `F`.
pub struct Not<F>(Phantom<F>);

/// Logical conjunction (AND): returns `true` if and only if both `L` and `R` evaluate to `true`.
///
/// Semantically order-independent; the current default implementation short-circuits (skipping `R` when `L` is `false`).
pub struct And<L, R>(Phantom<L>, Phantom<R>);

/// Logical disjunction (OR): returns `true` if and only if at least one of `L` or `R` evaluates to `true`.
///
/// Semantically order-independent; the current default implementation short-circuits (skipping `R` when `L` is `true`).
pub struct Or<L, R>(Phantom<L>, Phantom<R>);

/// Logical exclusive-OR (XOR): returns `true` if and only if `L` and `R` produce different boolean results.
///
/// Semantically order-independent; evaluates both sides and returns their boolean XOR.
pub struct Xor<L, R>(Phantom<L>, Phantom<R>);

/// Type-level normalization trait: transparently expands a predicate tuple `(P1, P2, ...)` into right-associated nested disjunction `Or<P1, Or<P2, ...>>`.
///
/// - Empty tuple `()` expands to `Always<False>` (empty disjunction is false);
/// - Single-element tuple `(P0,)` expands to `P0`;
/// - Multi-element tuples are supported up to 16 elements.
pub trait DisjunctionTuple {
    /// The normalized underlying disjunction predicate type.
    type Pred;
}

/// Type-level normalization trait: transparently expands a predicate tuple `(P1, P2, ...)` into right-associated nested conjunction `And<P1, And<P2, ...>>`.
///
/// - Empty tuple `()` expands to `Always<True>` (empty conjunction is true);
/// - Single-element tuple `(P0,)` expands to `P0`;
/// - Multi-element tuples are supported up to 16 elements.
pub trait ConjunctionTuple {
    /// The normalized underlying conjunction predicate type.
    type Pred;
}

/// Transparent alias for flat multi-way disjunction (finite union): expands tuple `(P1, P2, ...)` into `Or<P1, Or<P2, ...>>`.
pub type AnyOf<Tuple> = <Tuple as DisjunctionTuple>::Pred;

/// Transparent alias for flat multi-way conjunction (finite intersection): expands tuple `(P1, P2, ...)` into `And<P1, And<P2, ...>>`.
pub type AllOf<Tuple> = <Tuple as ConjunctionTuple>::Pred;

impl DisjunctionTuple for () {
    type Pred = Always<False>;
}

impl ConjunctionTuple for () {
    type Pred = Always<True>;
}

impl<P0> DisjunctionTuple for (P0,) {
    type Pred = P0;
}

impl<P0> ConjunctionTuple for (P0,) {
    type Pred = P0;
}

macro_rules! impl_logic_tuples {
    ($head:ident, $tail:ident) => {
        impl<$head, $tail> DisjunctionTuple for ($head, $tail) {
            type Pred = Or<$head, $tail>;
        }
        impl<$head, $tail> ConjunctionTuple for ($head, $tail) {
            type Pred = And<$head, $tail>;
        }
    };
    ($head:ident, $($rest:ident),+ $(,)?) => {
        impl<$head, $($rest),+> DisjunctionTuple for ($head, $($rest),+) {
            type Pred = Or<$head, <($($rest,)+) as DisjunctionTuple>::Pred>;
        }
        impl<$head, $($rest),+> ConjunctionTuple for ($head, $($rest),+) {
            type Pred = And<$head, <($($rest,)+) as ConjunctionTuple>::Pred>;
        }
        impl_logic_tuples!($($rest),+);
    };
}

impl_logic_tuples!(
    P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14, P15
);

// SAFETY: Input is consumed by a pure function, and boolean negation introduces neither side effects nor temporary drops.
unsafe impl<Input, F: SafeFunction<Input, Output = bool>> SafeFunction<Input> for Not<F> {}

// SAFETY: Both sides are pure safe functions and input is Copy; conjunction and short-circuit selection introduce no side effects and always terminate.
unsafe impl<Input: Copy, L, R> SafeFunction<Input> for And<L, R>
where
    L: SafeFunction<Input, Output = bool>,
    R: SafeFunction<Input, Output = bool>,
{
}

// SAFETY: Both sides are pure safe functions and input is Copy; disjunction and short-circuit selection introduce no side effects and always terminate.
unsafe impl<Input: Copy, L, R> SafeFunction<Input> for Or<L, R>
where
    L: SafeFunction<Input, Output = bool>,
    R: SafeFunction<Input, Output = bool>,
{
}

// SAFETY: Both sides are pure safe functions and input is Copy; boolean XOR introduces no side effects and always terminates.
unsafe impl<Input: Copy, L, R> SafeFunction<Input> for Xor<L, R>
where
    L: SafeFunction<Input, Output = bool>,
    R: SafeFunction<Input, Output = bool>,
{
}
