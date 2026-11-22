//! Nightly `const`-capable safe comparison protocol implementations for `Subset` (including non-`NaN` float total ordering and exact cross-integer comparisons).

use core::cmp::Ordering;

use super::Subset;
use super::float_cmp::{cmp_non_nan_f64, cmp_non_nan_f64_i128, cmp_non_nan_f64_u128};
use crate::cmp::{SafeEq, SafeOrd};
use crate::function::Predicate;
use crate::function::branch::AndThen;
use crate::function::predicate::IsNotNaN;
use crate::marker::SafeValue;

/// Marker contract statically certifying that a predicate strictly excludes `NaN` on floating-point carriers (`f32`/`f64`).
///
/// # Safety
/// Implementors must guarantee `!x.is_nan()` holds for every floating-point value accepted by the predicate.
pub unsafe trait NonNaNFloatPredicate {}

// SAFETY: `IsNotNaN` and `AndThen<IsNotNaN, Q>` both strictly reject `NaN` on `f32`/`f64`.
unsafe impl NonNaNFloatPredicate for IsNotNaN {}
unsafe impl<Q> NonNaNFloatPredicate for AndThen<IsNotNaN, Q> {}

/// Dispatches safe equality comparisons for `Subset<P, Self>` across eligible carrier types.
///
/// # Safety
/// Implementations must satisfy reflexivity, symmetry, transitivity, purity, and termination as required by [`SafeEq`].
pub const unsafe trait SubsetCarrierEq<P, Rhs: ?Sized> {
    /// Returns `true` if `self` and `other` are mathematically equal.
    fn eq(&self, other: &Rhs) -> bool;

    /// Returns `true` if `self` and `other` are mathematically unequal.
    fn ne(&self, other: &Rhs) -> bool;
}

/// Dispatches safe total-order comparisons for `Subset<P, Self>` across eligible carrier types.
///
/// # Safety
/// Implementations must satisfy total order axioms and remain strictly consistent with [`SubsetCarrierEq`].
pub const unsafe trait SubsetCarrierOrd<P, Rhs: ?Sized>:
    [const] SubsetCarrierEq<P, Rhs>
{
    /// Returns the total ordering between `self` and `other`.
    fn cmp(&self, other: &Rhs) -> Ordering;
    /// Returns `true` if `self < other`.
    fn lt(&self, other: &Rhs) -> bool;
    /// Returns `true` if `self <= other`.
    fn le(&self, other: &Rhs) -> bool;
    /// Returns `true` if `self > other`.
    fn gt(&self, other: &Rhs) -> bool;
    /// Returns `true` if `self >= other`.
    fn ge(&self, other: &Rhs) -> bool;
}

/// Right-hand-side equality contract accepting a certified non-`NaN` float on the left.
///
/// # Safety
/// `left` is guaranteed non-`NaN`; implementations must compare exact mathematical values purely and terminate.
pub const unsafe trait FloatRhsEq<Float> {
    /// Evaluates `left == self`.
    fn eq_float(&self, left: Float) -> bool;
    /// Evaluates `left != self`.
    fn ne_float(&self, left: Float) -> bool;
}

/// Right-hand-side total-order contract accepting a certified non-`NaN` float on the left.
///
/// # Safety
/// `left` is guaranteed non-`NaN`; implementations must satisfy strict total ordering consistent with [`FloatRhsEq`].
pub const unsafe trait FloatRhsOrd<Float>: [const] FloatRhsEq<Float> {
    /// Returns the exact mathematical ordering of `left` relative to `self`.
    fn cmp_float(&self, left: Float) -> Ordering;
}

/// Underlying carrier equality contract for `Subset<Q, Self>` compared against a non-`NaN` float on the left.
///
/// # Safety
/// `left` is guaranteed non-`NaN`; when `Self` is a floating-point type, `Q` must likewise exclude `NaN`.
pub const unsafe trait SubsetFloatRhsEq<Q, Float> {
    /// Evaluates `left == self`.
    fn eq_float(&self, left: Float) -> bool;
    /// Evaluates `left != self`.
    fn ne_float(&self, left: Float) -> bool;
}

/// Underlying carrier total-order contract for `Subset<Q, Self>` compared against a non-`NaN` float on the left.
///
/// # Safety
/// `left` is guaranteed non-`NaN`; when `Self` is a floating-point type, `Q` must likewise exclude `NaN`.
pub const unsafe trait SubsetFloatRhsOrd<Q, Float>:
    [const] SubsetFloatRhsEq<Q, Float>
{
    /// Returns the exact mathematical ordering of `left` relative to `self`.
    fn cmp_float(&self, left: Float) -> Ordering;
}

// ==========================================
// 1. Non-`NaN` Float & Exact Integer Right-Hand Sides
// ==========================================

macro_rules! impl_subset_float_rhs_float {
    ($rhs:ty) => {
        // SAFETY: `Q: NonNaNFloatPredicate` and the left-hand contract ensure both operands are non-NaN;
        // numeric comparison yields total order with `-0.0 == +0.0`, zero panics, and guaranteed termination.
        const unsafe impl<Q: NonNaNFloatPredicate> SubsetFloatRhsEq<Q, f32> for $rhs {
            #[inline(always)]
            fn eq_float(&self, left: f32) -> bool {
                matches!(cmp_non_nan_f64(left as f64, *self as f64), Ordering::Equal)
            }
            #[inline(always)]
            fn ne_float(&self, left: f32) -> bool {
                !matches!(cmp_non_nan_f64(left as f64, *self as f64), Ordering::Equal)
            }
        }

        const unsafe impl<Q: NonNaNFloatPredicate> SubsetFloatRhsOrd<Q, f32> for $rhs {
            #[inline(always)]
            fn cmp_float(&self, left: f32) -> Ordering {
                cmp_non_nan_f64(left as f64, *self as f64)
            }
        }

        const unsafe impl<Q: NonNaNFloatPredicate> SubsetFloatRhsEq<Q, f64> for $rhs {
            #[inline(always)]
            fn eq_float(&self, left: f64) -> bool {
                matches!(cmp_non_nan_f64(left, *self as f64), Ordering::Equal)
            }
            #[inline(always)]
            fn ne_float(&self, left: f64) -> bool {
                !matches!(cmp_non_nan_f64(left, *self as f64), Ordering::Equal)
            }
        }

        const unsafe impl<Q: NonNaNFloatPredicate> SubsetFloatRhsOrd<Q, f64> for $rhs {
            #[inline(always)]
            fn cmp_float(&self, left: f64) -> Ordering {
                cmp_non_nan_f64(left, *self as f64)
            }
        }
    };
}

impl_subset_float_rhs_float!(f32);
impl_subset_float_rhs_float!(f64);

macro_rules! impl_float_rhs_int {
    ($cmp_fn:ident, $wide:ty, $($int:ty),* $(,)?) => {
        $(
            // SAFETY: Compares IEEE 754 bit fields directly against lossless 128-bit integer representations without overflow or rounding.
            const unsafe impl FloatRhsEq<f32> for $int {
                #[inline(always)]
                fn eq_float(&self, left: f32) -> bool {
                    matches!($cmp_fn(left as f64, *self as $wide), Ordering::Equal)
                }
                #[inline(always)]
                fn ne_float(&self, left: f32) -> bool {
                    !matches!($cmp_fn(left as f64, *self as $wide), Ordering::Equal)
                }
            }

            const unsafe impl FloatRhsOrd<f32> for $int {
                #[inline(always)]
                fn cmp_float(&self, left: f32) -> Ordering {
                    $cmp_fn(left as f64, *self as $wide)
                }
            }

            const unsafe impl FloatRhsEq<f64> for $int {
                #[inline(always)]
                fn eq_float(&self, left: f64) -> bool {
                    matches!($cmp_fn(left, *self as $wide), Ordering::Equal)
                }
                #[inline(always)]
                fn ne_float(&self, left: f64) -> bool {
                    !matches!($cmp_fn(left, *self as $wide), Ordering::Equal)
                }
            }

            const unsafe impl FloatRhsOrd<f64> for $int {
                #[inline(always)]
                fn cmp_float(&self, left: f64) -> Ordering {
                    $cmp_fn(left, *self as $wide)
                }
            }

            const unsafe impl<Q> SubsetFloatRhsEq<Q, f32> for $int {
                #[inline(always)]
                fn eq_float(&self, left: f32) -> bool {
                    <Self as FloatRhsEq<f32>>::eq_float(self, left)
                }
                #[inline(always)]
                fn ne_float(&self, left: f32) -> bool {
                    <Self as FloatRhsEq<f32>>::ne_float(self, left)
                }
            }

            const unsafe impl<Q> SubsetFloatRhsOrd<Q, f32> for $int {
                #[inline(always)]
                fn cmp_float(&self, left: f32) -> Ordering {
                    <Self as FloatRhsOrd<f32>>::cmp_float(self, left)
                }
            }

            const unsafe impl<Q> SubsetFloatRhsEq<Q, f64> for $int {
                #[inline(always)]
                fn eq_float(&self, left: f64) -> bool {
                    <Self as FloatRhsEq<f64>>::eq_float(self, left)
                }
                #[inline(always)]
                fn ne_float(&self, left: f64) -> bool {
                    <Self as FloatRhsEq<f64>>::ne_float(self, left)
                }
            }

            const unsafe impl<Q> SubsetFloatRhsOrd<Q, f64> for $int {
                #[inline(always)]
                fn cmp_float(&self, left: f64) -> Ordering {
                    <Self as FloatRhsOrd<f64>>::cmp_float(self, left)
                }
            }
        )*
    };
}

impl_float_rhs_int!(cmp_non_nan_f64_u128, u128, u8, u16, u32, u64, u128, usize);
impl_float_rhs_int!(cmp_non_nan_f64_i128, i128, i8, i16, i32, i64, i128, isize);

// SAFETY: Transparent forwarding through references and nested Subset layers.
const unsafe impl<Float, Rhs: ?Sized + [const] FloatRhsEq<Float>> FloatRhsEq<Float> for &Rhs {
    #[inline(always)]
    fn eq_float(&self, left: Float) -> bool {
        <Rhs as FloatRhsEq<Float>>::eq_float(*self, left)
    }
    #[inline(always)]
    fn ne_float(&self, left: Float) -> bool {
        <Rhs as FloatRhsEq<Float>>::ne_float(*self, left)
    }
}

const unsafe impl<Float, Rhs: ?Sized + [const] FloatRhsOrd<Float>> FloatRhsOrd<Float> for &Rhs {
    #[inline(always)]
    fn cmp_float(&self, left: Float) -> Ordering {
        <Rhs as FloatRhsOrd<Float>>::cmp_float(*self, left)
    }
}

const unsafe impl<Q, B: ?Sized, Float> FloatRhsEq<Float> for Subset<Q, B>
where
    Q: for<'a> Predicate<&'a B>,
    B: SafeValue + [const] SubsetFloatRhsEq<Q, Float>,
{
    #[inline(always)]
    fn eq_float(&self, left: Float) -> bool {
        <B as SubsetFloatRhsEq<Q, Float>>::eq_float(&self.value, left)
    }
    #[inline(always)]
    fn ne_float(&self, left: Float) -> bool {
        <B as SubsetFloatRhsEq<Q, Float>>::ne_float(&self.value, left)
    }
}

const unsafe impl<Q, B: ?Sized, Float> FloatRhsOrd<Float> for Subset<Q, B>
where
    Q: for<'a> Predicate<&'a B>,
    B: SafeValue + [const] SubsetFloatRhsOrd<Q, Float>,
{
    #[inline(always)]
    fn cmp_float(&self, left: Float) -> Ordering {
        <B as SubsetFloatRhsOrd<Q, Float>>::cmp_float(&self.value, left)
    }
}

const unsafe impl<OuterQ, InnerQ, InnerB: ?Sized, Float> SubsetFloatRhsEq<OuterQ, Float>
    for Subset<InnerQ, InnerB>
where
    InnerQ: for<'a> Predicate<&'a InnerB>,
    InnerB: SafeValue + [const] SubsetFloatRhsEq<InnerQ, Float>,
{
    #[inline(always)]
    fn eq_float(&self, left: Float) -> bool {
        <InnerB as SubsetFloatRhsEq<InnerQ, Float>>::eq_float(&self.value, left)
    }
    #[inline(always)]
    fn ne_float(&self, left: Float) -> bool {
        <InnerB as SubsetFloatRhsEq<InnerQ, Float>>::ne_float(&self.value, left)
    }
}

const unsafe impl<OuterQ, InnerQ, InnerB: ?Sized, Float> SubsetFloatRhsOrd<OuterQ, Float>
    for Subset<InnerQ, InnerB>
where
    InnerQ: for<'a> Predicate<&'a InnerB>,
    InnerB: SafeValue + [const] SubsetFloatRhsOrd<InnerQ, Float>,
{
    #[inline(always)]
    fn cmp_float(&self, left: Float) -> Ordering {
        <InnerB as SubsetFloatRhsOrd<InnerQ, Float>>::cmp_float(&self.value, left)
    }
}

// ==========================================
// 2. Carrier Dispatch Implementations
// ==========================================

macro_rules! impl_non_float_carrier {
    ($($carrier:ty),* $(,)?) => {
        $(
            // SAFETY: Forwards directly to the underlying SafeEq/SafeOrd implementation.
            const unsafe impl<P, Rhs: ?Sized> SubsetCarrierEq<P, Rhs> for $carrier
            where
                $carrier: [const] SafeEq<Rhs>,
            {
                #[inline(always)]
                fn eq(&self, other: &Rhs) -> bool {
                    <$carrier as SafeEq<Rhs>>::eq(self, other)
                }
                #[inline(always)]
                fn ne(&self, other: &Rhs) -> bool {
                    <$carrier as SafeEq<Rhs>>::ne(self, other)
                }
            }

            const unsafe impl<P, Rhs: ?Sized> SubsetCarrierOrd<P, Rhs> for $carrier
            where
                $carrier: [const] SafeOrd<Rhs>,
            {
                #[inline(always)]
                fn cmp(&self, other: &Rhs) -> Ordering {
                    <$carrier as SafeOrd<Rhs>>::cmp(self, other)
                }
                #[inline(always)]
                fn lt(&self, other: &Rhs) -> bool {
                    <$carrier as SafeOrd<Rhs>>::lt(self, other)
                }
                #[inline(always)]
                fn le(&self, other: &Rhs) -> bool {
                    <$carrier as SafeOrd<Rhs>>::le(self, other)
                }
                #[inline(always)]
                fn gt(&self, other: &Rhs) -> bool {
                    <$carrier as SafeOrd<Rhs>>::gt(self, other)
                }
                #[inline(always)]
                fn ge(&self, other: &Rhs) -> bool {
                    <$carrier as SafeOrd<Rhs>>::ge(self, other)
                }
            }
        )*
    };
}

impl_non_float_carrier!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, bool, char, str
);

// SAFETY: Slices, arrays, shared references, and nested Subsets forward to their own SafeEq/SafeOrd contracts.
const unsafe impl<P, T, Rhs: ?Sized> SubsetCarrierEq<P, Rhs> for [T]
where
    [T]: [const] SafeEq<Rhs>,
{
    #[inline(always)]
    fn eq(&self, other: &Rhs) -> bool {
        <[T] as SafeEq<Rhs>>::eq(self, other)
    }
    #[inline(always)]
    fn ne(&self, other: &Rhs) -> bool {
        <[T] as SafeEq<Rhs>>::ne(self, other)
    }
}

const unsafe impl<P, T, Rhs: ?Sized> SubsetCarrierOrd<P, Rhs> for [T]
where
    [T]: [const] SafeOrd<Rhs>,
{
    #[inline(always)]
    fn cmp(&self, other: &Rhs) -> Ordering {
        <[T] as SafeOrd<Rhs>>::cmp(self, other)
    }
    #[inline(always)]
    fn lt(&self, other: &Rhs) -> bool {
        <[T] as SafeOrd<Rhs>>::lt(self, other)
    }
    #[inline(always)]
    fn le(&self, other: &Rhs) -> bool {
        <[T] as SafeOrd<Rhs>>::le(self, other)
    }
    #[inline(always)]
    fn gt(&self, other: &Rhs) -> bool {
        <[T] as SafeOrd<Rhs>>::gt(self, other)
    }
    #[inline(always)]
    fn ge(&self, other: &Rhs) -> bool {
        <[T] as SafeOrd<Rhs>>::ge(self, other)
    }
}

const unsafe impl<P, T, const N: usize, Rhs: ?Sized> SubsetCarrierEq<P, Rhs> for [T; N]
where
    [T; N]: [const] SafeEq<Rhs>,
{
    #[inline(always)]
    fn eq(&self, other: &Rhs) -> bool {
        <[T; N] as SafeEq<Rhs>>::eq(self, other)
    }
    #[inline(always)]
    fn ne(&self, other: &Rhs) -> bool {
        <[T; N] as SafeEq<Rhs>>::ne(self, other)
    }
}

const unsafe impl<P, T, const N: usize, Rhs: ?Sized> SubsetCarrierOrd<P, Rhs> for [T; N]
where
    [T; N]: [const] SafeOrd<Rhs>,
{
    #[inline(always)]
    fn cmp(&self, other: &Rhs) -> Ordering {
        <[T; N] as SafeOrd<Rhs>>::cmp(self, other)
    }
    #[inline(always)]
    fn lt(&self, other: &Rhs) -> bool {
        <[T; N] as SafeOrd<Rhs>>::lt(self, other)
    }
    #[inline(always)]
    fn le(&self, other: &Rhs) -> bool {
        <[T; N] as SafeOrd<Rhs>>::le(self, other)
    }
    #[inline(always)]
    fn gt(&self, other: &Rhs) -> bool {
        <[T; N] as SafeOrd<Rhs>>::gt(self, other)
    }
    #[inline(always)]
    fn ge(&self, other: &Rhs) -> bool {
        <[T; N] as SafeOrd<Rhs>>::ge(self, other)
    }
}

const unsafe impl<P, T: ?Sized, Rhs: ?Sized> SubsetCarrierEq<P, Rhs> for &T
where
    Self: [const] SafeEq<Rhs>,
{
    #[inline(always)]
    fn eq(&self, other: &Rhs) -> bool {
        <Self as SafeEq<Rhs>>::eq(self, other)
    }
    #[inline(always)]
    fn ne(&self, other: &Rhs) -> bool {
        <Self as SafeEq<Rhs>>::ne(self, other)
    }
}

const unsafe impl<P, T: ?Sized, Rhs: ?Sized> SubsetCarrierOrd<P, Rhs> for &T
where
    Self: [const] SafeOrd<Rhs>,
{
    #[inline(always)]
    fn cmp(&self, other: &Rhs) -> Ordering {
        <Self as SafeOrd<Rhs>>::cmp(self, other)
    }
    #[inline(always)]
    fn lt(&self, other: &Rhs) -> bool {
        <Self as SafeOrd<Rhs>>::lt(self, other)
    }
    #[inline(always)]
    fn le(&self, other: &Rhs) -> bool {
        <Self as SafeOrd<Rhs>>::le(self, other)
    }
    #[inline(always)]
    fn gt(&self, other: &Rhs) -> bool {
        <Self as SafeOrd<Rhs>>::gt(self, other)
    }
    #[inline(always)]
    fn ge(&self, other: &Rhs) -> bool {
        <Self as SafeOrd<Rhs>>::ge(self, other)
    }
}

const unsafe impl<P, InnerP, InnerA: ?Sized, Rhs: ?Sized> SubsetCarrierEq<P, Rhs>
    for Subset<InnerP, InnerA>
where
    InnerP: for<'a> Predicate<&'a InnerA>,
    InnerA: SafeValue,
    Self: [const] SafeEq<Rhs>,
{
    #[inline(always)]
    fn eq(&self, other: &Rhs) -> bool {
        <Self as SafeEq<Rhs>>::eq(self, other)
    }
    #[inline(always)]
    fn ne(&self, other: &Rhs) -> bool {
        <Self as SafeEq<Rhs>>::ne(self, other)
    }
}

const unsafe impl<P, InnerP, InnerA: ?Sized, Rhs: ?Sized> SubsetCarrierOrd<P, Rhs>
    for Subset<InnerP, InnerA>
where
    InnerP: for<'a> Predicate<&'a InnerA>,
    InnerA: SafeValue,
    Self: [const] SafeOrd<Rhs>,
{
    #[inline(always)]
    fn cmp(&self, other: &Rhs) -> Ordering {
        <Self as SafeOrd<Rhs>>::cmp(self, other)
    }
    #[inline(always)]
    fn lt(&self, other: &Rhs) -> bool {
        <Self as SafeOrd<Rhs>>::lt(self, other)
    }
    #[inline(always)]
    fn le(&self, other: &Rhs) -> bool {
        <Self as SafeOrd<Rhs>>::le(self, other)
    }
    #[inline(always)]
    fn gt(&self, other: &Rhs) -> bool {
        <Self as SafeOrd<Rhs>>::gt(self, other)
    }
    #[inline(always)]
    fn ge(&self, other: &Rhs) -> bool {
        <Self as SafeOrd<Rhs>>::ge(self, other)
    }
}

macro_rules! impl_float_carrier {
    ($($float:ty),* $(,)?) => {
        $(
            // SAFETY: `P: NonNaNFloatPredicate` guarantees non-NaN LHS; `Rhs` guarantees non-NaN float or exact integer.
            const unsafe impl<P: NonNaNFloatPredicate, Rhs: ?Sized + [const] FloatRhsEq<$float>>
                SubsetCarrierEq<P, Rhs> for $float
            {
                #[inline(always)]
                fn eq(&self, other: &Rhs) -> bool {
                    <Rhs as FloatRhsEq<$float>>::eq_float(other, *self)
                }
                #[inline(always)]
                fn ne(&self, other: &Rhs) -> bool {
                    <Rhs as FloatRhsEq<$float>>::ne_float(other, *self)
                }
            }

            const unsafe impl<P: NonNaNFloatPredicate, Rhs: ?Sized + [const] FloatRhsOrd<$float>>
                SubsetCarrierOrd<P, Rhs> for $float
            {
                #[inline(always)]
                fn cmp(&self, other: &Rhs) -> Ordering {
                    <Rhs as FloatRhsOrd<$float>>::cmp_float(other, *self)
                }
                #[inline(always)]
                fn lt(&self, other: &Rhs) -> bool {
                    matches!(
                        <Rhs as FloatRhsOrd<$float>>::cmp_float(other, *self),
                        Ordering::Less
                    )
                }
                #[inline(always)]
                fn le(&self, other: &Rhs) -> bool {
                    !matches!(
                        <Rhs as FloatRhsOrd<$float>>::cmp_float(other, *self),
                        Ordering::Greater
                    )
                }
                #[inline(always)]
                fn gt(&self, other: &Rhs) -> bool {
                    matches!(
                        <Rhs as FloatRhsOrd<$float>>::cmp_float(other, *self),
                        Ordering::Greater
                    )
                }
                #[inline(always)]
                fn ge(&self, other: &Rhs) -> bool {
                    !matches!(
                        <Rhs as FloatRhsOrd<$float>>::cmp_float(other, *self),
                        Ordering::Less
                    )
                }
            }
        )*
    };
}

impl_float_carrier!(f32, f64);

// ==========================================
// 3. Public `SafeEq` / `SafeOrd` for `Subset`, `&Subset`, and Scalars/Slices
// ==========================================

// SAFETY: Dispatches through `SubsetCarrierEq`/`Ord`, giving non-NaN floats strict total order and exact cross-integer comparisons.
const unsafe impl<P, A: ?Sized, Rhs: ?Sized> SafeEq<Rhs> for Subset<P, A>
where
    P: for<'a> Predicate<&'a A>,
    A: SafeValue + [const] SubsetCarrierEq<P, Rhs>,
{
    #[inline(always)]
    fn eq(&self, other: &Rhs) -> bool {
        <A as SubsetCarrierEq<P, Rhs>>::eq(&self.value, other)
    }

    #[inline(always)]
    fn ne(&self, other: &Rhs) -> bool {
        <A as SubsetCarrierEq<P, Rhs>>::ne(&self.value, other)
    }
}

const unsafe impl<P, A: ?Sized, Rhs: ?Sized> SafeOrd<Rhs> for Subset<P, A>
where
    P: for<'a> Predicate<&'a A>,
    A: SafeValue + [const] SubsetCarrierOrd<P, Rhs>,
{
    #[inline(always)]
    fn cmp(&self, other: &Rhs) -> Ordering {
        <A as SubsetCarrierOrd<P, Rhs>>::cmp(&self.value, other)
    }

    #[inline(always)]
    fn lt(&self, other: &Rhs) -> bool {
        <A as SubsetCarrierOrd<P, Rhs>>::lt(&self.value, other)
    }

    #[inline(always)]
    fn le(&self, other: &Rhs) -> bool {
        <A as SubsetCarrierOrd<P, Rhs>>::le(&self.value, other)
    }

    #[inline(always)]
    fn gt(&self, other: &Rhs) -> bool {
        <A as SubsetCarrierOrd<P, Rhs>>::gt(&self.value, other)
    }

    #[inline(always)]
    fn ge(&self, other: &Rhs) -> bool {
        <A as SubsetCarrierOrd<P, Rhs>>::ge(&self.value, other)
    }
}

// SAFETY: Left-hand dereference forwards to the underlying Subset comparison pair.
const unsafe impl<P, Q, A: ?Sized, B: ?Sized> SafeEq<Subset<Q, B>> for &Subset<P, A>
where
    P: for<'a> Predicate<&'a A>,
    Q: for<'a> Predicate<&'a B>,
    A: SafeValue,
    B: SafeValue,
    Subset<P, A>: [const] SafeEq<Subset<Q, B>>,
{
    #[inline(always)]
    fn eq(&self, other: &Subset<Q, B>) -> bool {
        <Subset<P, A> as SafeEq<Subset<Q, B>>>::eq(*self, other)
    }

    #[inline(always)]
    fn ne(&self, other: &Subset<Q, B>) -> bool {
        <Subset<P, A> as SafeEq<Subset<Q, B>>>::ne(*self, other)
    }
}

const unsafe impl<P, Q, A: ?Sized, B: ?Sized> SafeOrd<Subset<Q, B>> for &Subset<P, A>
where
    P: for<'a> Predicate<&'a A>,
    Q: for<'a> Predicate<&'a B>,
    A: SafeValue,
    B: SafeValue,
    Subset<P, A>: [const] SafeOrd<Subset<Q, B>>,
{
    #[inline(always)]
    fn cmp(&self, other: &Subset<Q, B>) -> Ordering {
        <Subset<P, A> as SafeOrd<Subset<Q, B>>>::cmp(*self, other)
    }

    #[inline(always)]
    fn lt(&self, other: &Subset<Q, B>) -> bool {
        <Subset<P, A> as SafeOrd<Subset<Q, B>>>::lt(*self, other)
    }

    #[inline(always)]
    fn le(&self, other: &Subset<Q, B>) -> bool {
        <Subset<P, A> as SafeOrd<Subset<Q, B>>>::le(*self, other)
    }

    #[inline(always)]
    fn gt(&self, other: &Subset<Q, B>) -> bool {
        <Subset<P, A> as SafeOrd<Subset<Q, B>>>::gt(*self, other)
    }

    #[inline(always)]
    fn ge(&self, other: &Subset<Q, B>) -> bool {
        <Subset<P, A> as SafeOrd<Subset<Q, B>>>::ge(*self, other)
    }
}

macro_rules! impl_subset_scalar_interop {
    ($($scalar:ty),* $(,)?) => {
        $(
            // SAFETY: Forwards dereferenced `&Subset<P, A>` to `Subset<P, A>: SafeEq/SafeOrd<$scalar>`.
            const unsafe impl<P, A: ?Sized> SafeEq<$scalar> for &Subset<P, A>
            where
                P: for<'a> Predicate<&'a A>,
                A: SafeValue,
                Subset<P, A>: [const] SafeEq<$scalar>,
            {
                #[inline(always)]
                fn eq(&self, other: &$scalar) -> bool {
                    <Subset<P, A> as SafeEq<$scalar>>::eq(*self, other)
                }
                #[inline(always)]
                fn ne(&self, other: &$scalar) -> bool {
                    <Subset<P, A> as SafeEq<$scalar>>::ne(*self, other)
                }
            }

            const unsafe impl<P, A: ?Sized> SafeOrd<$scalar> for &Subset<P, A>
            where
                P: for<'a> Predicate<&'a A>,
                A: SafeValue,
                Subset<P, A>: [const] SafeOrd<$scalar>,
            {
                #[inline(always)]
                fn cmp(&self, other: &$scalar) -> Ordering {
                    <Subset<P, A> as SafeOrd<$scalar>>::cmp(*self, other)
                }
                #[inline(always)]
                fn lt(&self, other: &$scalar) -> bool {
                    <Subset<P, A> as SafeOrd<$scalar>>::lt(*self, other)
                }
                #[inline(always)]
                fn le(&self, other: &$scalar) -> bool {
                    <Subset<P, A> as SafeOrd<$scalar>>::le(*self, other)
                }
                #[inline(always)]
                fn gt(&self, other: &$scalar) -> bool {
                    <Subset<P, A> as SafeOrd<$scalar>>::gt(*self, other)
                }
                #[inline(always)]
                fn ge(&self, other: &$scalar) -> bool {
                    <Subset<P, A> as SafeOrd<$scalar>>::ge(*self, other)
                }
            }

            // SAFETY: Uses symmetry and antisymmetry to forward scalar LHS comparisons to `Subset<Q, B>`.
            const unsafe impl<Q, B: ?Sized> SafeEq<Subset<Q, B>> for $scalar
            where
                Q: for<'a> Predicate<&'a B>,
                B: SafeValue,
                Subset<Q, B>: [const] SafeEq<$scalar>,
            {
                #[inline(always)]
                fn eq(&self, other: &Subset<Q, B>) -> bool {
                    <Subset<Q, B> as SafeEq<$scalar>>::eq(other, self)
                }
                #[inline(always)]
                fn ne(&self, other: &Subset<Q, B>) -> bool {
                    <Subset<Q, B> as SafeEq<$scalar>>::ne(other, self)
                }
            }

            const unsafe impl<Q, B: ?Sized> SafeOrd<Subset<Q, B>> for $scalar
            where
                Q: for<'a> Predicate<&'a B>,
                B: SafeValue,
                Subset<Q, B>: [const] SafeOrd<$scalar>,
            {
                #[inline(always)]
                fn cmp(&self, other: &Subset<Q, B>) -> Ordering {
                    <Subset<Q, B> as SafeOrd<$scalar>>::cmp(other, self).reverse()
                }
                #[inline(always)]
                fn lt(&self, other: &Subset<Q, B>) -> bool {
                    <Subset<Q, B> as SafeOrd<$scalar>>::gt(other, self)
                }
                #[inline(always)]
                fn le(&self, other: &Subset<Q, B>) -> bool {
                    <Subset<Q, B> as SafeOrd<$scalar>>::ge(other, self)
                }
                #[inline(always)]
                fn gt(&self, other: &Subset<Q, B>) -> bool {
                    <Subset<Q, B> as SafeOrd<$scalar>>::lt(other, self)
                }
                #[inline(always)]
                fn ge(&self, other: &Subset<Q, B>) -> bool {
                    <Subset<Q, B> as SafeOrd<$scalar>>::le(other, self)
                }
            }

            const unsafe impl<Q, B: ?Sized> SafeEq<&Subset<Q, B>> for $scalar
            where
                Q: for<'a> Predicate<&'a B>,
                B: SafeValue,
                Subset<Q, B>: [const] SafeEq<$scalar>,
            {
                #[inline(always)]
                fn eq(&self, other: &&Subset<Q, B>) -> bool {
                    <Subset<Q, B> as SafeEq<$scalar>>::eq(*other, self)
                }
                #[inline(always)]
                fn ne(&self, other: &&Subset<Q, B>) -> bool {
                    <Subset<Q, B> as SafeEq<$scalar>>::ne(*other, self)
                }
            }

            const unsafe impl<Q, B: ?Sized> SafeOrd<&Subset<Q, B>> for $scalar
            where
                Q: for<'a> Predicate<&'a B>,
                B: SafeValue,
                Subset<Q, B>: [const] SafeOrd<$scalar>,
            {
                #[inline(always)]
                fn cmp(&self, other: &&Subset<Q, B>) -> Ordering {
                    <Subset<Q, B> as SafeOrd<$scalar>>::cmp(*other, self).reverse()
                }
                #[inline(always)]
                fn lt(&self, other: &&Subset<Q, B>) -> bool {
                    <Subset<Q, B> as SafeOrd<$scalar>>::gt(*other, self)
                }
                #[inline(always)]
                fn le(&self, other: &&Subset<Q, B>) -> bool {
                    <Subset<Q, B> as SafeOrd<$scalar>>::ge(*other, self)
                }
                #[inline(always)]
                fn gt(&self, other: &&Subset<Q, B>) -> bool {
                    <Subset<Q, B> as SafeOrd<$scalar>>::lt(*other, self)
                }
                #[inline(always)]
                fn ge(&self, other: &&Subset<Q, B>) -> bool {
                    <Subset<Q, B> as SafeOrd<$scalar>>::le(*other, self)
                }
            }

            const unsafe impl<Q, B: ?Sized> SafeEq<Subset<Q, B>> for &$scalar
            where
                Q: for<'a> Predicate<&'a B>,
                B: SafeValue,
                Subset<Q, B>: [const] SafeEq<$scalar>,
            {
                #[inline(always)]
                fn eq(&self, other: &Subset<Q, B>) -> bool {
                    <Subset<Q, B> as SafeEq<$scalar>>::eq(other, *self)
                }
                #[inline(always)]
                fn ne(&self, other: &Subset<Q, B>) -> bool {
                    <Subset<Q, B> as SafeEq<$scalar>>::ne(other, *self)
                }
            }

            const unsafe impl<Q, B: ?Sized> SafeOrd<Subset<Q, B>> for &$scalar
            where
                Q: for<'a> Predicate<&'a B>,
                B: SafeValue,
                Subset<Q, B>: [const] SafeOrd<$scalar>,
            {
                #[inline(always)]
                fn cmp(&self, other: &Subset<Q, B>) -> Ordering {
                    <Subset<Q, B> as SafeOrd<$scalar>>::cmp(other, *self).reverse()
                }
                #[inline(always)]
                fn lt(&self, other: &Subset<Q, B>) -> bool {
                    <Subset<Q, B> as SafeOrd<$scalar>>::gt(other, *self)
                }
                #[inline(always)]
                fn le(&self, other: &Subset<Q, B>) -> bool {
                    <Subset<Q, B> as SafeOrd<$scalar>>::ge(other, *self)
                }
                #[inline(always)]
                fn gt(&self, other: &Subset<Q, B>) -> bool {
                    <Subset<Q, B> as SafeOrd<$scalar>>::lt(other, *self)
                }
                #[inline(always)]
                fn ge(&self, other: &Subset<Q, B>) -> bool {
                    <Subset<Q, B> as SafeOrd<$scalar>>::le(other, *self)
                }
            }
        )*
    };
}

impl_subset_scalar_interop!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, bool, char, str
);

// SAFETY: Interoperability between slices and `Subset<Q, [U]>` (including nested slices).
const unsafe impl<T, Q, B: ?Sized> SafeEq<Subset<Q, B>> for [T]
where
    Q: for<'a> Predicate<&'a B>,
    B: SafeValue,
    [T]: [const] SafeEq<B>,
{
    #[inline(always)]
    fn eq(&self, other: &Subset<Q, B>) -> bool {
        <[T] as SafeEq<B>>::eq(self, &other.value)
    }
    #[inline(always)]
    fn ne(&self, other: &Subset<Q, B>) -> bool {
        <[T] as SafeEq<B>>::ne(self, &other.value)
    }
}

const unsafe impl<T, Q, B: ?Sized> SafeOrd<Subset<Q, B>> for [T]
where
    Q: for<'a> Predicate<&'a B>,
    B: SafeValue,
    [T]: [const] SafeOrd<B>,
{
    #[inline(always)]
    fn cmp(&self, other: &Subset<Q, B>) -> Ordering {
        <[T] as SafeOrd<B>>::cmp(self, &other.value)
    }
    #[inline(always)]
    fn lt(&self, other: &Subset<Q, B>) -> bool {
        <[T] as SafeOrd<B>>::lt(self, &other.value)
    }
    #[inline(always)]
    fn le(&self, other: &Subset<Q, B>) -> bool {
        <[T] as SafeOrd<B>>::le(self, &other.value)
    }
    #[inline(always)]
    fn gt(&self, other: &Subset<Q, B>) -> bool {
        <[T] as SafeOrd<B>>::gt(self, &other.value)
    }
    #[inline(always)]
    fn ge(&self, other: &Subset<Q, B>) -> bool {
        <[T] as SafeOrd<B>>::ge(self, &other.value)
    }
}

const unsafe impl<T, Q, B: ?Sized> SafeEq<&Subset<Q, B>> for [T]
where
    Q: for<'a> Predicate<&'a B>,
    B: SafeValue,
    [T]: [const] SafeEq<B>,
{
    #[inline(always)]
    fn eq(&self, other: &&Subset<Q, B>) -> bool {
        <[T] as SafeEq<B>>::eq(self, &other.value)
    }
    #[inline(always)]
    fn ne(&self, other: &&Subset<Q, B>) -> bool {
        <[T] as SafeEq<B>>::ne(self, &other.value)
    }
}

const unsafe impl<T, Q, B: ?Sized> SafeOrd<&Subset<Q, B>> for [T]
where
    Q: for<'a> Predicate<&'a B>,
    B: SafeValue,
    [T]: [const] SafeOrd<B>,
{
    #[inline(always)]
    fn cmp(&self, other: &&Subset<Q, B>) -> Ordering {
        <[T] as SafeOrd<B>>::cmp(self, &other.value)
    }
    #[inline(always)]
    fn lt(&self, other: &&Subset<Q, B>) -> bool {
        <[T] as SafeOrd<B>>::lt(self, &other.value)
    }
    #[inline(always)]
    fn le(&self, other: &&Subset<Q, B>) -> bool {
        <[T] as SafeOrd<B>>::le(self, &other.value)
    }
    #[inline(always)]
    fn gt(&self, other: &&Subset<Q, B>) -> bool {
        <[T] as SafeOrd<B>>::gt(self, &other.value)
    }
    #[inline(always)]
    fn ge(&self, other: &&Subset<Q, B>) -> bool {
        <[T] as SafeOrd<B>>::ge(self, &other.value)
    }
}

const unsafe impl<T, Q, B: ?Sized> SafeEq<Subset<Q, B>> for &[T]
where
    Q: for<'a> Predicate<&'a B>,
    B: SafeValue,
    [T]: [const] SafeEq<B>,
{
    #[inline(always)]
    fn eq(&self, other: &Subset<Q, B>) -> bool {
        <[T] as SafeEq<B>>::eq(*self, &other.value)
    }
    #[inline(always)]
    fn ne(&self, other: &Subset<Q, B>) -> bool {
        <[T] as SafeEq<B>>::ne(*self, &other.value)
    }
}

const unsafe impl<T, Q, B: ?Sized> SafeOrd<Subset<Q, B>> for &[T]
where
    Q: for<'a> Predicate<&'a B>,
    B: SafeValue,
    [T]: [const] SafeOrd<B>,
{
    #[inline(always)]
    fn cmp(&self, other: &Subset<Q, B>) -> Ordering {
        <[T] as SafeOrd<B>>::cmp(*self, &other.value)
    }
    #[inline(always)]
    fn lt(&self, other: &Subset<Q, B>) -> bool {
        <[T] as SafeOrd<B>>::lt(*self, &other.value)
    }
    #[inline(always)]
    fn le(&self, other: &Subset<Q, B>) -> bool {
        <[T] as SafeOrd<B>>::le(*self, &other.value)
    }
    #[inline(always)]
    fn gt(&self, other: &Subset<Q, B>) -> bool {
        <[T] as SafeOrd<B>>::gt(*self, &other.value)
    }
    #[inline(always)]
    fn ge(&self, other: &Subset<Q, B>) -> bool {
        <[T] as SafeOrd<B>>::ge(*self, &other.value)
    }
}

// SAFETY: Interoperability between fixed-size arrays and `Subset<Q, [U; N]>`.
const unsafe impl<T, const N: usize, Q, B: ?Sized> SafeEq<Subset<Q, B>> for [T; N]
where
    Q: for<'a> Predicate<&'a B>,
    B: SafeValue,
    [T; N]: [const] SafeEq<B>,
{
    #[inline(always)]
    fn eq(&self, other: &Subset<Q, B>) -> bool {
        <[T; N] as SafeEq<B>>::eq(self, &other.value)
    }
    #[inline(always)]
    fn ne(&self, other: &Subset<Q, B>) -> bool {
        <[T; N] as SafeEq<B>>::ne(self, &other.value)
    }
}

const unsafe impl<T, const N: usize, Q, B: ?Sized> SafeOrd<Subset<Q, B>> for [T; N]
where
    Q: for<'a> Predicate<&'a B>,
    B: SafeValue,
    [T; N]: [const] SafeOrd<B>,
{
    #[inline(always)]
    fn cmp(&self, other: &Subset<Q, B>) -> Ordering {
        <[T; N] as SafeOrd<B>>::cmp(self, &other.value)
    }
    #[inline(always)]
    fn lt(&self, other: &Subset<Q, B>) -> bool {
        <[T; N] as SafeOrd<B>>::lt(self, &other.value)
    }
    #[inline(always)]
    fn le(&self, other: &Subset<Q, B>) -> bool {
        <[T; N] as SafeOrd<B>>::le(self, &other.value)
    }
    #[inline(always)]
    fn gt(&self, other: &Subset<Q, B>) -> bool {
        <[T; N] as SafeOrd<B>>::gt(self, &other.value)
    }
    #[inline(always)]
    fn ge(&self, other: &Subset<Q, B>) -> bool {
        <[T; N] as SafeOrd<B>>::ge(self, &other.value)
    }
}

const unsafe impl<T, const N: usize, Q, B: ?Sized> SafeEq<&Subset<Q, B>> for [T; N]
where
    Q: for<'a> Predicate<&'a B>,
    B: SafeValue,
    [T; N]: [const] SafeEq<B>,
{
    #[inline(always)]
    fn eq(&self, other: &&Subset<Q, B>) -> bool {
        <[T; N] as SafeEq<B>>::eq(self, &other.value)
    }
    #[inline(always)]
    fn ne(&self, other: &&Subset<Q, B>) -> bool {
        <[T; N] as SafeEq<B>>::ne(self, &other.value)
    }
}

const unsafe impl<T, const N: usize, Q, B: ?Sized> SafeOrd<&Subset<Q, B>> for [T; N]
where
    Q: for<'a> Predicate<&'a B>,
    B: SafeValue,
    [T; N]: [const] SafeOrd<B>,
{
    #[inline(always)]
    fn cmp(&self, other: &&Subset<Q, B>) -> Ordering {
        <[T; N] as SafeOrd<B>>::cmp(self, &other.value)
    }
    #[inline(always)]
    fn lt(&self, other: &&Subset<Q, B>) -> bool {
        <[T; N] as SafeOrd<B>>::lt(self, &other.value)
    }
    #[inline(always)]
    fn le(&self, other: &&Subset<Q, B>) -> bool {
        <[T; N] as SafeOrd<B>>::le(self, &other.value)
    }
    #[inline(always)]
    fn gt(&self, other: &&Subset<Q, B>) -> bool {
        <[T; N] as SafeOrd<B>>::gt(self, &other.value)
    }
    #[inline(always)]
    fn ge(&self, other: &&Subset<Q, B>) -> bool {
        <[T; N] as SafeOrd<B>>::ge(self, &other.value)
    }
}

const unsafe impl<T, const N: usize, Q, B: ?Sized> SafeEq<Subset<Q, B>> for &[T; N]
where
    Q: for<'a> Predicate<&'a B>,
    B: SafeValue,
    [T; N]: [const] SafeEq<B>,
{
    #[inline(always)]
    fn eq(&self, other: &Subset<Q, B>) -> bool {
        <[T; N] as SafeEq<B>>::eq(*self, &other.value)
    }
    #[inline(always)]
    fn ne(&self, other: &Subset<Q, B>) -> bool {
        <[T; N] as SafeEq<B>>::ne(*self, &other.value)
    }
}

const unsafe impl<T, const N: usize, Q, B: ?Sized> SafeOrd<Subset<Q, B>> for &[T; N]
where
    Q: for<'a> Predicate<&'a B>,
    B: SafeValue,
    [T; N]: [const] SafeOrd<B>,
{
    #[inline(always)]
    fn cmp(&self, other: &Subset<Q, B>) -> Ordering {
        <[T; N] as SafeOrd<B>>::cmp(*self, &other.value)
    }
    #[inline(always)]
    fn lt(&self, other: &Subset<Q, B>) -> bool {
        <[T; N] as SafeOrd<B>>::lt(*self, &other.value)
    }
    #[inline(always)]
    fn le(&self, other: &Subset<Q, B>) -> bool {
        <[T; N] as SafeOrd<B>>::le(*self, &other.value)
    }
    #[inline(always)]
    fn gt(&self, other: &Subset<Q, B>) -> bool {
        <[T; N] as SafeOrd<B>>::gt(*self, &other.value)
    }
    #[inline(always)]
    fn ge(&self, other: &Subset<Q, B>) -> bool {
        <[T; N] as SafeOrd<B>>::ge(*self, &other.value)
    }
}
