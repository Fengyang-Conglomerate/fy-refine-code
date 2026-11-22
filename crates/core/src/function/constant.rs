//! Compile-time primitive literal constant morphisms (nullary safe functions).
//!
//! Every constant type implements `SafeFunction<()>` and can be directly combined with operators such as `EqConst`, `LtConst`, `Range`, and `Always`.
//!
//! ```
//! use fy_refine_core::function::Function;
//! use fy_refine_core::function::constant::{ConstBool, ConstChar, ConstI32, ConstU8};
//! use fy_refine_core::function::predicate::EqConst;
//!
//! assert_eq!(ConstU8::<42>::call(()), 42);
//! assert_eq!(ConstI32::<-7>::call(()), -7);
//! assert!(ConstBool::<true>::call(()));
//! assert_eq!(ConstChar::<'a'>::call(()), 'a');
//! assert!(EqConst::<ConstU8<42>>::call(&42u8));
//! ```

#[cfg(feature = "nightly")]
mod constant_nightly;
#[cfg(not(feature = "nightly"))]
mod constant_stable;

#[cfg(feature = "nightly")]
pub use constant_nightly::{
    ConstBool, ConstChar, ConstI8, ConstI16, ConstI32, ConstI64, ConstI128, ConstIsize, ConstU8,
    ConstU16, ConstU32, ConstU64, ConstU128, ConstUsize,
};
#[cfg(not(feature = "nightly"))]
pub use constant_stable::{
    ConstBool, ConstChar, ConstI8, ConstI16, ConstI32, ConstI64, ConstI128, ConstIsize, ConstU8,
    ConstU16, ConstU32, ConstU64, ConstU128, ConstUsize,
};
