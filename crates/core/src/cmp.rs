//! Safe equality and safe total ordering comparison protocols supporting heterogeneous left- and right-hand operand types.

#[cfg(feature = "nightly")]
mod cmp_nightly;
#[cfg(not(feature = "nightly"))]
mod cmp_stable;

#[cfg(feature = "nightly")]
pub use cmp_nightly::{SafeEq, SafeOrd};
#[cfg(not(feature = "nightly"))]
pub use cmp_stable::{SafeEq, SafeOrd};

#[cfg(feature = "nightly")]
mod integers_nightly;
#[cfg(not(feature = "nightly"))]
mod integers_stable;
#[cfg(feature = "nightly")]
mod primitives_nightly;
#[cfg(not(feature = "nightly"))]
mod primitives_stable;
mod references;
#[cfg(feature = "nightly")]
mod slices_nightly;
#[cfg(not(feature = "nightly"))]
mod slices_stable;
