//! Core type property and safety marker protocols.
//!
//! Provides `unsafe` contract traits describing intrinsic data-type invariants along with
//! `unsafe`-free declarative macros:
//! - Value safety and base-unwrapping contract ([`SafeValue`], macro [`impl_safe_value!`](crate::impl_safe_value)): guarantees that logical data remains constant under shared borrows and supports recursive unwrapping to the innermost carrier type;
//! - Faithful cloning contract ([`SafeClone`], macro [`impl_safe_clone!`](crate::impl_safe_clone)): guarantees that cloning produces a copy logically equivalent to the original under all safe functions;
//! - Safe destruction contract ([`SafeDrop`], macro [`impl_safe_drop!`](crate::impl_safe_drop)): guarantees that dropping terminates cleanly without panicking or destructive side effects;
//! - Phantom marker alias ([`Phantom`]): zero-overhead, invariant in `T`, and transparent to the drop checker.

mod safe_clone;
mod safe_drop;
mod safe_value;

pub use safe_clone::SafeClone;
pub use safe_drop::SafeDrop;
pub use safe_value::SafeValue;

/// A zero-runtime-overhead phantom type marker that is invariant in `T` and transparent to the drop checker.
///
/// Backed by `core::marker::PhantomData<fn(T) -> T>`, it neither owns a physical instance of `T`
/// (never invokes `T`'s destructor) nor permits unintended covariant or contravariant subtyping on `T`.
pub type Phantom<T> = core::marker::PhantomData<fn(T) -> T>;
