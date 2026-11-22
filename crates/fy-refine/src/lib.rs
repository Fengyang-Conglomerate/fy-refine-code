#![cfg_attr(not(feature = "std"), no_std)]

//! `fy-refine`: unified facade crate for Fengyang Conglomerate refinement types.
//!
//! Re-exports all safety contracts, function combinators, predicate building blocks, and the refined subset type [`Subset`] from [`fy_refine_core`],
//! as well as the zero-dependency derive macros [`SafeValue`], [`SafeDrop`], [`SafeClone`], and [`Projectors`].
//!
//! ```
//! use fy_refine::Subset;
//! use fy_refine::function::constant::ConstU8;
//! use fy_refine::function::predicate::{Closed, Range};
//!
//! type U8In0To100 = Subset<Range<Closed<ConstU8<0>>, Closed<ConstU8<100>>>, u8>;
//!
//! #[derive(Debug, PartialEq, Eq)]
//! struct Percentage(U8In0To100);
//!
//! let ok = Percentage(U8In0To100::try_new(80).unwrap());
//! assert_eq!(*ok.0, 80);
//! assert_eq!(U8In0To100::try_new(101), Err(101));
//! ```

pub use fy_refine_core as core;
pub use fy_refine_core::*;
pub use fy_refine_macro::{Projectors, SafeClone, SafeDrop, SafeValue};
