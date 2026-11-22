//! Typed function protocol, combinator topology, constant morphisms, generic container quantifiers, slice projections, and predicate building blocks; supports optional compile-time evaluation on `nightly`.

#[cfg(feature = "nightly")]
mod function_nightly;
#[cfg(not(feature = "nightly"))]
mod function_stable;

pub mod bitwise;
pub mod branch;
pub mod collection;
pub mod constant;
pub mod lookup;
pub mod predicate;
pub mod slice;
pub mod wiring;

#[cfg(feature = "nightly")]
pub use function_nightly::Function;
#[cfg(not(feature = "nightly"))]
pub use function_stable::Function;

pub use predicate::Predicate;

/// Safety contract for functions that are total over their input domain, deterministically terminating, and transparent to external application state.
///
/// Safety does not forbid local mutable bindings, benign diagnostic/monitoring logs (e.g. `println!`, tracing),
/// performance counters, or benign local heap allocations, provided the function never produces side effects that violate refinement invariants or system logic.
/// Use and destruction of the returned value after delivery to the caller are governed by the caller and receiver contracts.
///
/// # Safety
/// For every valid input, the implementor must guarantee:
///
/// 1. **Termination & Panic-free**: every call completes in finitely many steps and never panics.
/// 2. **Determinism**: logically equivalent inputs always produce logically equivalent outputs, independent of external mutable state, randomness, clocks, or call history.
/// 3. **No Destructive Side Effect**: never mutates external application state and never mutates input data through shared borrows.
/// 4. **Transitivity**: these guarantees extend to all inner calls and temporary value drops executed during the call.
pub unsafe trait SafeFunction<Input>: Function<Input> {}
