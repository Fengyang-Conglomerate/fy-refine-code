//! Explicit open/closed endpoint markers and transparent combinator aliases for interval predicates.
//!
//! Endpoint markers [`Closed`] and [`Open`] explicitly declare left/right inclusivity and transparently normalize at compile time into a conjunction of two order comparisons
//! (`And<GeConst/GtConst<Lo>, LeConst/LtConst<Hi>>`), requiring zero new primitives and no extra interval-specific axioms:
//! - `Range<Closed<Lo>, Closed<Hi>>` → closed interval `[Lo, Hi]`
//! - `Range<Closed<Lo>, Open<Hi>>`   → half-open interval `[Lo, Hi)`
//! - `Range<Open<Lo>, Closed<Hi>>`   → half-open interval `(Lo, Hi]`
//! - `Range<Open<Lo>, Open<Hi>>`     → open interval `(Lo, Hi)`
//!
//! ```
//! use fy_refine_core::function::Function;
//! use fy_refine_core::function::constant::ConstU8;
//! use fy_refine_core::function::predicate::{Closed, Not, Open, Range};
//! use fy_refine_core::subset::Subset;
//!
//! type Percent = Subset<Range<Closed<ConstU8<0>>, Closed<ConstU8<100>>>, u8>;
//! assert!(Percent::try_new(0).is_ok());
//! assert!(Percent::try_new(100).is_ok());
//! assert_eq!(Percent::try_new(101), Err(101));
//!
//! type HalfOpen = Range<Closed<ConstU8<1>>, Open<ConstU8<5>>>;
//! assert!(HalfOpen::call(&1u8));
//! assert!(!HalfOpen::call(&5u8));
//!
//! type StrictOpen = Range<Open<ConstU8<1>>, Open<ConstU8<5>>>;
//! assert!(!StrictOpen::call(&1u8));
//! assert!(StrictOpen::call(&3u8));
//! assert!(!StrictOpen::call(&5u8));
//!
//! // For outside-interval checks, compose Not<Range<...>> directly
//! assert!(Not::<HalfOpen>::call(&5u8));
//! ```

use crate::marker::Phantom;

use super::{And, GeConst, GtConst, LeConst, LtConst};

/// Closed interval endpoint marker (includes the boundary value).
pub struct Closed<C>(Phantom<C>);

/// Open interval endpoint marker (excludes the boundary value).
pub struct Open<C>(Phantom<C>);

/// Type-level normalization trait: maps a lower-bound endpoint marker to its corresponding order comparison predicate ([`Closed`] → [`GeConst`], [`Open`] → [`GtConst`]).
pub trait LowerBound {
    /// Normalized lower-bound comparison predicate.
    type Pred;
}

impl<C> LowerBound for Closed<C> {
    type Pred = GeConst<C>;
}

impl<C> LowerBound for Open<C> {
    type Pred = GtConst<C>;
}

/// Type-level normalization trait: maps an upper-bound endpoint marker to its corresponding order comparison predicate ([`Closed`] → [`LeConst`], [`Open`] → [`LtConst`]).
pub trait UpperBound {
    /// Normalized upper-bound comparison predicate.
    type Pred;
}

impl<C> UpperBound for Closed<C> {
    type Pred = LeConst<C>;
}

impl<C> UpperBound for Open<C> {
    type Pred = LtConst<C>;
}

/// Transparent alias for an explicit open/closed interval predicate, normalizing at compile time to `And<<Lo as LowerBound>::Pred, <Hi as UpperBound>::Pred>`.
pub type Range<Lo, Hi> = And<<Lo as LowerBound>::Pred, <Hi as UpperBound>::Pred>;
