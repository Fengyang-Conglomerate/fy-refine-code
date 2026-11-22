//! `SafeClone` contract implementations for primitive and core types in pure `core` environments.

use super::SafeClone;
use core::cmp::Ordering;
use core::convert::Infallible;
use core::marker::PhantomData;
use core::num::{
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize, NonZeroU8,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize, Saturating, Wrapping,
};
use core::ops::{
    ControlFlow, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};
use core::time::Duration;

macro_rules! impl_safe_clone_primitives {
    ($($t:ty),* $(,)?) => {
        $(
            // SAFETY: `Clone` for primitive scalar and core value types is a bit-for-bit copy, logically identical to the original value.
            unsafe impl SafeClone for $t {}
        )*
    };
}

impl_safe_clone_primitives!(
    (),
    bool,
    char,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    f32,
    f64,
    NonZeroU8,
    NonZeroU16,
    NonZeroU32,
    NonZeroU64,
    NonZeroU128,
    NonZeroUsize,
    NonZeroI8,
    NonZeroI16,
    NonZeroI32,
    NonZeroI64,
    NonZeroI128,
    NonZeroIsize,
    Ordering,
    Duration,
    Infallible,
    RangeFull,
);

// SAFETY: Cloning a shared reference duplicates the pointer to the same referent, preserving full logical equivalence.
unsafe impl<T: ?Sized> SafeClone for &T {}

// SAFETY: `PhantomData` is a zero-sized phantom marker.
unsafe impl<T: ?Sized> SafeClone for PhantomData<T> {}

// SAFETY: Standard library `Clone` delegates field-wise to inner `T::clone` (and faithfully preserves the internal exhausted flag of `RangeInclusive`),
// yielding full logical equivalence whenever the inner types satisfy `SafeClone`.
unsafe impl<T: SafeClone> SafeClone for Wrapping<T> {}
unsafe impl<T: SafeClone> SafeClone for Saturating<T> {}
unsafe impl<T: SafeClone> SafeClone for Option<T> {}
unsafe impl<T: SafeClone, E: SafeClone> SafeClone for Result<T, E> {}
unsafe impl<B: SafeClone, C: SafeClone> SafeClone for ControlFlow<B, C> {}
unsafe impl<Idx: SafeClone> SafeClone for Range<Idx> {}
unsafe impl<Idx: SafeClone> SafeClone for RangeFrom<Idx> {}
unsafe impl<Idx: SafeClone> SafeClone for RangeTo<Idx> {}
unsafe impl<Idx: SafeClone> SafeClone for RangeInclusive<Idx> {}
unsafe impl<Idx: SafeClone> SafeClone for RangeToInclusive<Idx> {}

macro_rules! impl_safe_clone_tuple {
    ($($t:ident),+) => {
        // SAFETY: Standard library `Clone` for tuples delegates field-wise to inner `SafeClone` implementations, preserving full logical equivalence.
        unsafe impl<$($t: SafeClone),+> SafeClone for ($($t,)+) {}
    };
}

impl_safe_clone_tuple!(A);
impl_safe_clone_tuple!(A, B);
impl_safe_clone_tuple!(A, B, C);
impl_safe_clone_tuple!(A, B, C, D);
impl_safe_clone_tuple!(A, B, C, D, E);
impl_safe_clone_tuple!(A, B, C, D, E, F);
impl_safe_clone_tuple!(A, B, C, D, E, F, G);
impl_safe_clone_tuple!(A, B, C, D, E, F, G, H);
impl_safe_clone_tuple!(A, B, C, D, E, F, G, H, I);
impl_safe_clone_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_safe_clone_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_safe_clone_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);

// SAFETY: Standard library `Clone` for fixed-size arrays invokes `T::clone` element-wise, preserving full logical equivalence.
unsafe impl<T: SafeClone, const N: usize> SafeClone for [T; N] {}
