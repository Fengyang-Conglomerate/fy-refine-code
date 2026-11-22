//! Safe comparison predicate primitives and transparent combinator aliases.
//!
//! Organized in two layers:
//! 1. **Binary comparison primitives ([`Eq`], [`Ne`], [`Lt`], [`Le`], [`Gt`], [`Ge`])**:
//!    Directly accept a 2-tuple `(A, B)` and compare `A` with `B`;
//! 2. **Transparent combinator aliases ([`PairEq`]..[`PairGe`], [`ForkEq`]..[`ForkGe`], [`EqConst`]..[`GeConst`])**:
//!    All transparently composed from the 6 comparison primitives via `MapPair`, `Fork`, and `AttachUnit`, with zero redundant axioms and no `Copy` requirement.
//!
//! ```
//! use fy_refine_core::function::Function;
//! use fy_refine_core::function::constant::ConstI32;
//! use fy_refine_core::function::predicate::{Eq, EqConst, Lt};
//!
//! assert!(Lt::call((-1i8, u128::MAX)));
//! assert!(Eq::call((1u32, 1u32)));
//! assert!(EqConst::<ConstI32<10>>::call(10));
//! assert!(EqConst::<ConstI32<10>>::call(&10));
//! assert!(!EqConst::<ConstI32<10>>::call(20));
//! ```

use crate::cmp::{SafeEq, SafeOrd};
use crate::function::SafeFunction;
use crate::function::wiring::{AttachUnit, Fork, Id, MapPair, Then};
use crate::marker::SafeDrop;

#[cfg(feature = "nightly")]
mod cmp_nightly;
#[cfg(not(feature = "nightly"))]
mod cmp_stable;

// ==========================================
// 1. Binary Comparison Primitives (Input (A, B))
// ==========================================

/// Takes a 2-tuple `(A, B)` and checks equality (`==`).
pub struct Eq;
/// Takes a 2-tuple `(A, B)` and checks inequality (`!=`).
pub struct Ne;
/// Takes a 2-tuple `(A, B)` and checks whether the left side is strictly less than the right side (`<`).
pub struct Lt;
/// Takes a 2-tuple `(A, B)` and checks whether the left side is less than or equal to the right side (`<=`).
pub struct Le;
/// Takes a 2-tuple `(A, B)` and checks whether the left side is strictly greater than the right side (`>`).
pub struct Gt;
/// Takes a 2-tuple `(A, B)` and checks whether the left side is greater than or equal to the right side (`>=`).
pub struct Ge;

// ==========================================
// 2. Transparent Combinator Aliases (Pair*, Fork*, and *Const)
// ==========================================

/// Takes a 2-tuple `(A, B)`, computes `L` and `R` respectively, then compares for equality (`==`).
pub type PairEq<L, R> = Then<MapPair<L, R>, Eq>;
/// Takes a 2-tuple `(A, B)`, computes `L` and `R` respectively, then compares for inequality (`!=`).
pub type PairNe<L, R> = Then<MapPair<L, R>, Ne>;
/// Takes a 2-tuple `(A, B)`, computes `L` and `R` respectively, then compares for less-than (`<`).
pub type PairLt<L, R> = Then<MapPair<L, R>, Lt>;
/// Takes a 2-tuple `(A, B)`, computes `L` and `R` respectively, then compares for less-than-or-equal (`<=`).
pub type PairLe<L, R> = Then<MapPair<L, R>, Le>;
/// Takes a 2-tuple `(A, B)`, computes `L` and `R` respectively, then compares for greater-than (`>`).
pub type PairGt<L, R> = Then<MapPair<L, R>, Gt>;
/// Takes a 2-tuple `(A, B)`, computes `L` and `R` respectively, then compares for greater-than-or-equal (`>=`).
pub type PairGe<L, R> = Then<MapPair<L, R>, Ge>;

/// Takes a single `Copy` input, forks it to `L` and `R`, then compares for equality (`==`).
pub type ForkEq<L, R> = Then<Fork<L, R>, Eq>;
/// Takes a single `Copy` input, forks it to `L` and `R`, then compares for inequality (`!=`).
pub type ForkNe<L, R> = Then<Fork<L, R>, Ne>;
/// Takes a single `Copy` input, forks it to `L` and `R`, then compares for less-than (`<`).
pub type ForkLt<L, R> = Then<Fork<L, R>, Lt>;
/// Takes a single `Copy` input, forks it to `L` and `R`, then compares for less-than-or-equal (`<=`).
pub type ForkLe<L, R> = Then<Fork<L, R>, Le>;
/// Takes a single `Copy` input, forks it to `L` and `R`, then compares for greater-than (`>`).
pub type ForkGt<L, R> = Then<Fork<L, R>, Gt>;
/// Takes a single `Copy` input, forks it to `L` and `R`, then compares for greater-than-or-equal (`>=`).
pub type ForkGe<L, R> = Then<Fork<L, R>, Ge>;

/// Takes `A` (without requiring `Copy`) and checks equality (`==`) against constant morphism `C`.
pub type EqConst<C> = Then<AttachUnit, PairEq<Id, C>>;
/// Takes `A` (without requiring `Copy`) and checks inequality (`!=`) against constant morphism `C`.
pub type NeConst<C> = Then<AttachUnit, PairNe<Id, C>>;
/// Takes `A` (without requiring `Copy`) and checks whether the input is less than constant `C` (`<`).
pub type LtConst<C> = Then<AttachUnit, PairLt<Id, C>>;
/// Takes `A` (without requiring `Copy`) and checks whether the input is less than or equal to constant `C` (`<=`).
pub type LeConst<C> = Then<AttachUnit, PairLe<Id, C>>;
/// Takes `A` (without requiring `Copy`) and checks whether the input is greater than constant `C` (`>`).
pub type GtConst<C> = Then<AttachUnit, PairGt<Id, C>>;
/// Takes `A` (without requiring `Copy`) and checks whether the input is greater than or equal to constant `C` (`>=`).
pub type GeConst<C> = Then<AttachUnit, PairGe<Id, C>>;

// ==========================================
// 3. SafeFunction Contract Guarantees
// ==========================================

// SAFETY: SafeEq/SafeOrd promise that comparison is pure, total, panic-free, and guaranteed to terminate;
// A: SafeDrop and B: SafeDrop independently promise that dropping by-value inputs after comparison is benign, panic-free, and guaranteed to terminate.
unsafe impl<A: SafeEq<B> + SafeDrop, B: SafeDrop> SafeFunction<(A, B)> for Eq {}
unsafe impl<A: SafeEq<B> + SafeDrop, B: SafeDrop> SafeFunction<(A, B)> for Ne {}
unsafe impl<A: SafeOrd<B> + SafeDrop, B: SafeDrop> SafeFunction<(A, B)> for Lt {}
unsafe impl<A: SafeOrd<B> + SafeDrop, B: SafeDrop> SafeFunction<(A, B)> for Le {}
unsafe impl<A: SafeOrd<B> + SafeDrop, B: SafeDrop> SafeFunction<(A, B)> for Gt {}
unsafe impl<A: SafeOrd<B> + SafeDrop, B: SafeDrop> SafeFunction<(A, B)> for Ge {}
