//! Shared-reference comparisons forward directly to the underlying pure comparison without copying referenced values.

#[cfg(feature = "nightly")]
mod references_nightly;
#[cfg(not(feature = "nightly"))]
mod references_stable;
