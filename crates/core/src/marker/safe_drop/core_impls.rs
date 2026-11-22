//! `SafeDrop` contract implementations for primitive and core types in pure `core` environments.

use super::SafeDrop;
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

macro_rules! impl_safe_drop_primitive {
    ($($t:ty),* $(,)?) => {
        $(
            unsafe impl SafeDrop for $t {}
        )*
    };
}

// Primitive scalar and numeric types
impl_safe_drop_primitive!(
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
);

// Non-zero integer scalar types
impl_safe_drop_primitive!(
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
);

// Core enum, duration, and range-full marker types
impl_safe_drop_primitive!(Ordering, Duration, Infallible, RangeFull);

// Dropping references, raw pointers, and PhantomData is a no-op
unsafe impl<T: ?Sized> SafeDrop for &T {}
unsafe impl<T: ?Sized> SafeDrop for &mut T {}
unsafe impl<T: ?Sized> SafeDrop for *const T {}
unsafe impl<T: ?Sized> SafeDrop for *mut T {}
unsafe impl<T: ?Sized> SafeDrop for PhantomData<T> {}

// Dynamically sized slices (DSTs)
unsafe impl SafeDrop for str {}
unsafe impl<T: SafeDrop> SafeDrop for [T] {}

// Arithmetic wrapper types
unsafe impl<T: SafeDrop> SafeDrop for Wrapping<T> {}
unsafe impl<T: SafeDrop> SafeDrop for Saturating<T> {}

// Composite containers with recursive delegation
unsafe impl<T: SafeDrop> SafeDrop for Option<T> {}
unsafe impl<T: SafeDrop, E: SafeDrop> SafeDrop for Result<T, E> {}
unsafe impl<B: SafeDrop, C: SafeDrop> SafeDrop for ControlFlow<B, C> {}
unsafe impl<T: SafeDrop, const N: usize> SafeDrop for [T; N] {}

// Range types
unsafe impl<Idx: SafeDrop> SafeDrop for Range<Idx> {}
unsafe impl<Idx: SafeDrop> SafeDrop for RangeFrom<Idx> {}
unsafe impl<Idx: SafeDrop> SafeDrop for RangeTo<Idx> {}
unsafe impl<Idx: SafeDrop> SafeDrop for RangeInclusive<Idx> {}
unsafe impl<Idx: SafeDrop> SafeDrop for RangeToInclusive<Idx> {}

macro_rules! impl_safe_drop_tuple {
    ($($t:ident),+) => {
        unsafe impl<$($t: SafeDrop),+> SafeDrop for ($($t,)+) {}
    };
}

impl_safe_drop_tuple!(A);
impl_safe_drop_tuple!(A, B);
impl_safe_drop_tuple!(A, B, C);
impl_safe_drop_tuple!(A, B, C, D);
impl_safe_drop_tuple!(A, B, C, D, E);
impl_safe_drop_tuple!(A, B, C, D, E, F);
impl_safe_drop_tuple!(A, B, C, D, E, F, G);
impl_safe_drop_tuple!(A, B, C, D, E, F, G, H);
impl_safe_drop_tuple!(A, B, C, D, E, F, G, H, I);
impl_safe_drop_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_safe_drop_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_safe_drop_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
