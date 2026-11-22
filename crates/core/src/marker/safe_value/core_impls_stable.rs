//! `SafeValue` contract implementations on stable Rust for primitive and core types in pure `core` environments.

use super::SafeValue;
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

macro_rules! impl_safe_value_one {
    ($t:ty) => {
        // SAFETY: Primitive numbers and pure core scalar values have no interior mutability; `Base` is `Self`.
        unsafe impl SafeValue for $t {
            type Base = Self;

            #[inline(always)]
            fn as_base(&self) -> &Self {
                self
            }
        }
    };
}

macro_rules! impl_safe_value_self {
    () => {};
    ($head:ty $(, $tail:ty)* $(,)?) => {
        impl_safe_value_one!($head);
        impl_safe_value_self!($($tail),*);
    };
}

impl_safe_value_self!(
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
    str,
);

// SAFETY: A shared reference points to immutable data and introduces no interior mutability; the reference itself is not a refinement wrapper.
unsafe impl<T: ?Sized + SafeValue> SafeValue for &T {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

// SAFETY: `PhantomData` holds no runtime data.
unsafe impl<T: ?Sized> SafeValue for PhantomData<T> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

// SAFETY: A `[T]` slice yields a constant read-only view whenever its elements satisfy `SafeValue`.
unsafe impl<T: SafeValue> SafeValue for [T] {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

// SAFETY: `Wrapping` and `Saturating` only wrap inner arithmetic operations and have no interior mutability.
unsafe impl<T: SafeValue> SafeValue for Wrapping<T> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

unsafe impl<T: SafeValue> SafeValue for Saturating<T> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

// SAFETY: Composite containers satisfy the contract whenever all of their member types satisfy `SafeValue`.
unsafe impl<T: SafeValue> SafeValue for Option<T> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

unsafe impl<T: SafeValue, E: SafeValue> SafeValue for Result<T, E> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

unsafe impl<B: SafeValue, C: SafeValue> SafeValue for ControlFlow<B, C> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

unsafe impl<T: SafeValue, const N: usize> SafeValue for [T; N] {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

// SAFETY: Range types store only bound scalar values.
unsafe impl<Idx: SafeValue> SafeValue for Range<Idx> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

unsafe impl<Idx: SafeValue> SafeValue for RangeFrom<Idx> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

unsafe impl<Idx: SafeValue> SafeValue for RangeTo<Idx> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

unsafe impl<Idx: SafeValue> SafeValue for RangeInclusive<Idx> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

unsafe impl<Idx: SafeValue> SafeValue for RangeToInclusive<Idx> {
    type Base = Self;

    #[inline(always)]
    fn as_base(&self) -> &Self {
        self
    }
}

macro_rules! impl_safe_value_tuple {
    ($($t:ident),+) => {
        // SAFETY: All tuple fields satisfy `SafeValue`, so the tuple as a whole satisfies `SafeValue`.
        unsafe impl<$($t: SafeValue),+> SafeValue for ($($t,)+) {
            type Base = Self;

            #[inline(always)]
            fn as_base(&self) -> &Self {
                self
            }
        }
    };
}

impl_safe_value_tuple!(A);
impl_safe_value_tuple!(A, B);
impl_safe_value_tuple!(A, B, C);
impl_safe_value_tuple!(A, B, C, D);
impl_safe_value_tuple!(A, B, C, D, E);
impl_safe_value_tuple!(A, B, C, D, E, F);
impl_safe_value_tuple!(A, B, C, D, E, F, G);
impl_safe_value_tuple!(A, B, C, D, E, F, G, H);
impl_safe_value_tuple!(A, B, C, D, E, F, G, H, I);
impl_safe_value_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_safe_value_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_safe_value_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
