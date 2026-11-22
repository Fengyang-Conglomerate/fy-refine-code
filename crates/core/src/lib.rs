#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "nightly", feature(const_trait_impl, const_destruct))]

//! Core library for fy-refine: strong-invariant refined subsets, safety contracts, pure function topology, and predicate building blocks.

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod cmp;
pub mod function;
pub mod marker;
pub mod subset;

pub use cmp::{SafeEq, SafeOrd};
pub use function::{Function, Predicate, SafeFunction};
pub use marker::{Phantom, SafeClone, SafeDrop, SafeValue};
pub use subset::{FiniteF32, FiniteF64, NotNaNF32, NotNaNF64, Subset};
