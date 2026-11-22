//! Stable four-tier float-integer stratification, integer parity, and total divisibility implementations.

use super::{IsEven, IsFinite, IsInteger, IsMultipleOf, IsNaN, IsNotNaN};
use crate::function::branch::AndThen;
use crate::function::{Function, Predicate};
use crate::marker::SafeValue;
use crate::subset::{NonNaNFloatPredicate, Subset};

/// Carrier dispatch contract permitting [`Subset<P, Self>`] in [`IsFinite`] checks.
///
/// # Safety
/// Implementors must guarantee `is_finite` is pure, deterministic, panic-free, and terminating,
/// and may only implement this trait when precondition `P` statically excludes `NaN`
/// (so `!is_finite()` strictly means `±infinity`).
pub unsafe trait SubsetFiniteCarrier<P>: SafeValue {
    /// Returns `true` if the non-`NaN` inner float is finite.
    fn is_finite(&self) -> bool;
}

/// Carrier dispatch contract permitting [`Subset<P, Self>`] in [`IsInteger`] checks.
///
/// # Safety
/// Implementors must guarantee `is_integer` is pure, deterministic, panic-free, and terminating,
/// and that precondition `P` statically guarantees the carrier lies in the finite real domain
/// (neither `NaN` nor `±infinity`).
pub unsafe trait SubsetIntegerCarrier<P>: SafeValue {
    /// Returns `true` if the carrier guaranteed finite under `P` represents an exact mathematical integer.
    fn is_integer(&self) -> bool;
}

/// Auxiliary dispatch contract propagating finiteness proofs through nested [`Subset`] wrappers.
///
/// # Safety
/// Implementors must guarantee `is_integer_nested` is pure, deterministic, panic-free, and
/// terminating, and that `(POuter, PInner)` together statically guarantee a finite carrier.
pub unsafe trait NestedSubsetIntegerCarrier<POuter, PInner>: SafeValue {
    /// Returns `true` if the wrapped carrier proven finite by `POuter`/`PInner` is an exact mathematical integer.
    fn is_integer_nested(&self) -> bool;
}

/// Carrier dispatch contract permitting [`Subset<P, Self>`] in integer parity checks [`IsEven`] / [`IsOdd`](super::IsOdd).
///
/// # Safety
/// Implementors must guarantee `is_even` is pure, deterministic, panic-free, and terminating,
/// and that precondition `P` statically guarantees the carrier lies in the integer domain $\mathbb{Z}$.
pub unsafe trait SubsetEvenCarrier<P>: SafeValue {
    /// Returns `true` if the carrier holds an even integer under precondition `P`.
    fn is_even(&self) -> bool;
}

/// Auxiliary dispatch contract propagating integer-domain proofs across two [`Subset`] layers.
///
/// # Safety
/// Implementors must guarantee `is_even_nested2` is pure, deterministic, panic-free, and
/// terminating, and that `(P2, P1)` together statically guarantee the carrier lies in $\mathbb{Z}$.
pub unsafe trait NestedSubsetEvenCarrier2<P2, P1>: SafeValue {
    /// Returns `true` if the carrier proven to lie in $\mathbb{Z}$ across two predicate layers is even.
    fn is_even_nested2(&self) -> bool;
}

/// Auxiliary dispatch contract propagating integer-domain proofs across three or more [`Subset`] layers.
///
/// # Safety
/// Implementors must guarantee `is_even_nested3` is pure, deterministic, panic-free, and
/// terminating, and that `(P3, P2, P1)` together statically guarantee the carrier lies in $\mathbb{Z}$.
pub unsafe trait NestedSubsetEvenCarrier3<P3, P2, P1>: SafeValue {
    /// Returns `true` if the carrier proven to lie in $\mathbb{Z}$ across three outer predicate layers is even.
    fn is_even_nested3(&self) -> bool;
}

/// Determines via IEEE 754 bit patterns whether a guaranteed-finite `f32` is an exact mathematical integer (zero fractional part).
#[inline]
pub(super) const fn is_integer_finite_f32(x: f32) -> bool {
    let abs_bits = x.to_bits() & 0x7fff_ffff;
    if abs_bits == 0 {
        return true;
    }
    let exp = ((abs_bits >> 23) as i32) - 127;
    if exp < 0 {
        return false;
    }
    if exp >= 23 {
        return true;
    }
    (abs_bits & ((1u32 << (23 - exp)) - 1)) == 0
}

/// Determines via IEEE 754 bit patterns whether a guaranteed-finite `f64` is an exact mathematical integer (zero fractional part).
#[inline]
pub(super) const fn is_integer_finite_f64(x: f64) -> bool {
    let abs_bits = x.to_bits() & 0x7fff_ffff_ffff_ffff;
    if abs_bits == 0 {
        return true;
    }
    let exp = ((abs_bits >> 52) as i32) - 1023;
    if exp < 0 {
        return false;
    }
    if exp >= 52 {
        return true;
    }
    (abs_bits & ((1u64 << (52 - exp)) - 1)) == 0
}

/// Determines whether a finite `f32` already proven to satisfy [`is_integer_finite_f32`] is even.
#[inline]
pub(super) const fn is_even_integer_f32(x: f32) -> bool {
    let abs_bits = x.to_bits() & 0x7fff_ffff;
    if abs_bits == 0 {
        return true;
    }
    let exp = ((abs_bits >> 23) as i32) - 127;
    if exp <= 0 {
        return false;
    }
    if exp > 23 {
        return true;
    }
    ((abs_bits | (1u32 << 23)) & (1u32 << (23 - exp))) == 0
}

/// Determines whether a finite `f64` already proven to satisfy [`is_integer_finite_f64`] is even.
#[inline]
pub(super) const fn is_even_integer_f64(x: f64) -> bool {
    let abs_bits = x.to_bits() & 0x7fff_ffff_ffff_ffff;
    if abs_bits == 0 {
        return true;
    }
    let exp = ((abs_bits >> 52) as i32) - 1023;
    if exp <= 0 {
        return false;
    }
    if exp > 52 {
        return true;
    }
    ((abs_bits | (1u64 << 52)) & (1u64 << (52 - exp))) == 0
}

macro_rules! impl_unary_num_pred {
    ($pred:ident, $t:ty, |$x:ident| $body:expr) => {
        impl Function<$t> for $pred {
            type Output = bool;
            #[inline(always)]
            fn call($x: $t) -> bool {
                $body
            }
        }
        impl<'a> Function<&'a $t> for $pred {
            type Output = bool;
            #[inline(always)]
            fn call(x: &'a $t) -> bool {
                let $x = *x;
                $body
            }
        }
    };
}

macro_rules! impl_float_classify {
    ($(($t:ty, $is_int_fn:ident, $is_even_fn:ident)),* $(,)?) => {
        $(
            impl_unary_num_pred!(IsNaN, $t, |x| x.is_nan());

            // SAFETY: Precondition `P: NonNaNFloatPredicate` excludes `NaN`, so finiteness classification is pure, deterministic, panic-free, and terminating.
            unsafe impl<P: NonNaNFloatPredicate> SubsetFiniteCarrier<P> for $t {
                #[inline(always)]
                fn is_finite(&self) -> bool {
                    (*self).is_finite()
                }
            }

            // SAFETY: Precondition predicates explicitly narrow through `IsNotNaN` and `IsFinite` into finite reals; bitwise integer testing is pure and panic-free.
            unsafe impl SubsetIntegerCarrier<AndThen<IsNotNaN, IsFinite>> for $t {
                #[inline(always)]
                fn is_integer(&self) -> bool {
                    $is_int_fn(*self)
                }
            }

            // SAFETY: Narrowing via `IsNotNaN` followed by `AndThen<IsFinite, Q>` equally certifies the finite real domain.
            unsafe impl<Q> SubsetIntegerCarrier<AndThen<IsNotNaN, AndThen<IsFinite, Q>>> for $t {
                #[inline(always)]
                fn is_integer(&self) -> bool {
                    $is_int_fn(*self)
                }
            }

            // SAFETY: Inner layer proves `IsNotNaN` and direct outer layer proves `IsFinite`, jointly certifying the finite real domain.
            unsafe impl NestedSubsetIntegerCarrier<IsFinite, IsNotNaN> for $t {
                #[inline(always)]
                fn is_integer_nested(&self) -> bool {
                    $is_int_fn(*self)
                }
            }

            // SAFETY: Inner layer proves `IsNotNaN` and direct outer layer proves `AndThen<IsFinite, Q>`, jointly certifying the finite real domain.
            unsafe impl<Q> NestedSubsetIntegerCarrier<AndThen<IsFinite, Q>, IsNotNaN> for $t {
                #[inline(always)]
                fn is_integer_nested(&self) -> bool {
                    $is_int_fn(*self)
                }
            }

            // SAFETY: Inner `AndThen<IsNotNaN, IsFinite>` already certifies the finite real domain regardless of `POuter`.
            unsafe impl<POuter> NestedSubsetIntegerCarrier<POuter, AndThen<IsNotNaN, IsFinite>>
                for $t
            {
                #[inline(always)]
                fn is_integer_nested(&self) -> bool {
                    $is_int_fn(*self)
                }
            }

            // SAFETY: Precondition narrows step-by-step via `IsNotNaN` -> `IsFinite` -> `IsInteger` into the integer domain; parity testing is pure and panic-free.
            unsafe impl SubsetEvenCarrier<AndThen<IsNotNaN, AndThen<IsFinite, IsInteger>>> for $t {
                #[inline(always)]
                fn is_even(&self) -> bool {
                    $is_even_fn(*self)
                }
            }

            // SAFETY: Inner layer certifies the finite real domain and direct outer layer proves `IsInteger`, jointly certifying the integer domain.
            unsafe impl NestedSubsetEvenCarrier2<IsInteger, AndThen<IsNotNaN, IsFinite>> for $t {
                #[inline(always)]
                fn is_even_nested2(&self) -> bool {
                    $is_even_fn(*self)
                }
            }

            // SAFETY: Inner layer proves non-`NaN` and direct outer layer proves finiteness and integrality, jointly certifying the integer domain.
            unsafe impl NestedSubsetEvenCarrier2<AndThen<IsFinite, IsInteger>, IsNotNaN> for $t {
                #[inline(always)]
                fn is_even_nested2(&self) -> bool {
                    $is_even_fn(*self)
                }
            }

            // SAFETY: Inner composite predicate already certifies the integer domain regardless of `POuter`.
            unsafe impl<POuter>
                NestedSubsetEvenCarrier2<POuter, AndThen<IsNotNaN, AndThen<IsFinite, IsInteger>>>
                for $t
            {
                #[inline(always)]
                fn is_even_nested2(&self) -> bool {
                    $is_even_fn(*self)
                }
            }

            // SAFETY: Three layers sequentially prove `IsNotNaN`, `IsFinite`, and `IsInteger`, jointly certifying the integer domain.
            unsafe impl NestedSubsetEvenCarrier3<IsInteger, IsFinite, IsNotNaN> for $t {
                #[inline(always)]
                fn is_even_nested3(&self) -> bool {
                    $is_even_fn(*self)
                }
            }

            // SAFETY: Inner two layers already certify the integer domain; any outer narrowing `POuter` preserves the integer domain.
            unsafe impl<POuter>
                NestedSubsetEvenCarrier3<POuter, IsInteger, AndThen<IsNotNaN, IsFinite>> for $t
            {
                #[inline(always)]
                fn is_even_nested3(&self) -> bool {
                    $is_even_fn(*self)
                }
            }

            // SAFETY: Inner two layers already certify the integer domain via composite narrowing; any outer `POuter` preserves it.
            unsafe impl<POuter>
                NestedSubsetEvenCarrier3<POuter, AndThen<IsFinite, IsInteger>, IsNotNaN> for $t
            {
                #[inline(always)]
                fn is_even_nested3(&self) -> bool {
                    $is_even_fn(*self)
                }
            }
        )*
    };
}

macro_rules! multiple_body {
    (unsigned, $a:expr, $b:expr) => {
        if $b == 0 { $a == 0 } else { $a % $b == 0 }
    };
    (signed, $a:expr, $b:expr) => {
        if $b == 0 {
            $a == 0
        } else if $b == -1 {
            true
        } else {
            $a % $b == 0
        }
    };
}

macro_rules! impl_int_parity_and_multiple {
    ($kind:ident, $($t:ty),* $(,)?) => {
        $(
            impl_unary_num_pred!(IsEven, $t, |x| x % 2 == 0);

            // SAFETY: Primitive integer types intrinsically inhabit $\mathbb{Z}$, so any subset remains in the integer domain.
            unsafe impl<P> SubsetEvenCarrier<P> for $t {
                #[inline(always)]
                fn is_even(&self) -> bool {
                    (*self % 2) == 0
                }
            }

            // SAFETY: Nested subsets over primitive integer types remain in the integer domain $\mathbb{Z}$.
            unsafe impl<P2, P1> NestedSubsetEvenCarrier2<P2, P1> for $t {
                #[inline(always)]
                fn is_even_nested2(&self) -> bool {
                    (*self % 2) == 0
                }
            }

            // SAFETY: Nested subsets over primitive integer types remain in the integer domain $\mathbb{Z}$.
            unsafe impl<P3, P2, P1> NestedSubsetEvenCarrier3<P3, P2, P1> for $t {
                #[inline(always)]
                fn is_even_nested3(&self) -> bool {
                    (*self % 2) == 0
                }
            }

            impl Function<($t, $t)> for IsMultipleOf {
                type Output = bool;
                #[inline(always)]
                fn call((a, b): ($t, $t)) -> bool {
                    multiple_body!($kind, a, b)
                }
            }
            impl<'a> Function<(&'a $t, $t)> for IsMultipleOf {
                type Output = bool;
                #[inline(always)]
                fn call((a, b): (&'a $t, $t)) -> bool {
                    multiple_body!($kind, *a, b)
                }
            }
            impl<'a> Function<($t, &'a $t)> for IsMultipleOf {
                type Output = bool;
                #[inline(always)]
                fn call((a, b): ($t, &'a $t)) -> bool {
                    multiple_body!($kind, a, *b)
                }
            }
            impl<'a, 'b> Function<(&'a $t, &'b $t)> for IsMultipleOf {
                type Output = bool;
                #[inline(always)]
                fn call((a, b): (&'a $t, &'b $t)) -> bool {
                    multiple_body!($kind, *a, *b)
                }
            }
        )*
    };
}

impl_float_classify!(
    (f32, is_integer_finite_f32, is_even_integer_f32),
    (f64, is_integer_finite_f64, is_even_integer_f64),
);
impl_int_parity_and_multiple!(signed, i8, i16, i32, i64, i128, isize);
impl_int_parity_and_multiple!(unsigned, u8, u16, u32, u64, u128, usize);

// --- IsFinite recursive forwarding & impls ---

// SAFETY: Outer subset recursively forwards precondition `PInner` and inner finiteness proof down to the innermost carrier.
unsafe impl<POuter, PInner, TInner: ?Sized> SubsetFiniteCarrier<POuter> for Subset<PInner, TInner>
where
    PInner: for<'a> Predicate<&'a TInner>,
    TInner: SubsetFiniteCarrier<PInner>,
{
    #[inline(always)]
    fn is_finite(&self) -> bool {
        TInner::is_finite(Subset::as_inner(self))
    }
}

impl<P, T> Function<Subset<P, T>> for IsFinite
where
    P: for<'a> Predicate<&'a T>,
    T: SubsetFiniteCarrier<P>,
{
    type Output = bool;

    #[inline(always)]
    fn call(input: Subset<P, T>) -> bool {
        T::is_finite(Subset::as_inner(&input))
    }
}

impl<'a, P, T: ?Sized> Function<&'a Subset<P, T>> for IsFinite
where
    P: for<'b> Predicate<&'b T>,
    T: SubsetFiniteCarrier<P>,
{
    type Output = bool;

    #[inline(always)]
    fn call(input: &'a Subset<P, T>) -> bool {
        T::is_finite(Subset::as_inner(input))
    }
}

// --- IsInteger recursive forwarding & impls ---

// SAFETY: Propagates the finiteness proof jointly certified by outer predicate layers `(POuter, PInner)` to the inner carrier.
unsafe impl<POuter, PInner, TInner: ?Sized> SubsetIntegerCarrier<POuter> for Subset<PInner, TInner>
where
    PInner: for<'a> Predicate<&'a TInner>,
    TInner: NestedSubsetIntegerCarrier<POuter, PInner>,
{
    #[inline(always)]
    fn is_integer(&self) -> bool {
        TInner::is_integer_nested(Subset::as_inner(self))
    }
}

// SAFETY: Peels outer predicate layers while recursively forwarding the finiteness proof downward.
unsafe impl<POuter, PInner, P0, T0: ?Sized> NestedSubsetIntegerCarrier<POuter, PInner>
    for Subset<P0, T0>
where
    P0: for<'a> Predicate<&'a T0>,
    T0: NestedSubsetIntegerCarrier<PInner, P0>,
{
    #[inline(always)]
    fn is_integer_nested(&self) -> bool {
        T0::is_integer_nested(Subset::as_inner(self))
    }
}

impl<P, T> Function<Subset<P, T>> for IsInteger
where
    P: for<'a> Predicate<&'a T>,
    T: SubsetIntegerCarrier<P>,
{
    type Output = bool;

    #[inline(always)]
    fn call(input: Subset<P, T>) -> bool {
        T::is_integer(Subset::as_inner(&input))
    }
}

impl<'a, P, T: ?Sized> Function<&'a Subset<P, T>> for IsInteger
where
    P: for<'b> Predicate<&'b T>,
    T: SubsetIntegerCarrier<P>,
{
    type Output = bool;

    #[inline(always)]
    fn call(input: &'a Subset<P, T>) -> bool {
        T::is_integer(Subset::as_inner(input))
    }
}

// --- IsEven recursive forwarding & impls ---

// SAFETY: Propagates the integer-domain proof jointly certified by outer predicate layers `(P2, P1)` to the inner carrier.
unsafe impl<P2, P1, T1: ?Sized> SubsetEvenCarrier<P2> for Subset<P1, T1>
where
    P1: for<'a> Predicate<&'a T1>,
    T1: NestedSubsetEvenCarrier2<P2, P1>,
{
    #[inline(always)]
    fn is_even(&self) -> bool {
        T1::is_even_nested2(Subset::as_inner(self))
    }
}

// SAFETY: Propagates the integer-domain proof jointly certified by outer predicate layers `(P3, P2, P1)` to the inner carrier.
unsafe impl<P3, P2, P1, T1: ?Sized> NestedSubsetEvenCarrier2<P3, P2> for Subset<P1, T1>
where
    P1: for<'a> Predicate<&'a T1>,
    T1: NestedSubsetEvenCarrier3<P3, P2, P1>,
{
    #[inline(always)]
    fn is_even_nested2(&self) -> bool {
        T1::is_even_nested3(Subset::as_inner(self))
    }
}

// SAFETY: Shifts a sliding window across deep subset layers, forwarding the innermost three integer-domain proofs purely and without panics.
unsafe impl<P4, P3, P2, P1, T1: ?Sized> NestedSubsetEvenCarrier3<P4, P3, P2> for Subset<P1, T1>
where
    P1: for<'a> Predicate<&'a T1>,
    T1: NestedSubsetEvenCarrier3<P3, P2, P1>,
{
    #[inline(always)]
    fn is_even_nested3(&self) -> bool {
        T1::is_even_nested3(Subset::as_inner(self))
    }
}

impl<P, T> Function<Subset<P, T>> for IsEven
where
    P: for<'a> Predicate<&'a T>,
    T: SubsetEvenCarrier<P>,
{
    type Output = bool;

    #[inline(always)]
    fn call(input: Subset<P, T>) -> bool {
        T::is_even(Subset::as_inner(&input))
    }
}

impl<'a, P, T: ?Sized> Function<&'a Subset<P, T>> for IsEven
where
    P: for<'b> Predicate<&'b T>,
    T: SubsetEvenCarrier<P>,
{
    type Output = bool;

    #[inline(always)]
    fn call(input: &'a Subset<P, T>) -> bool {
        T::is_even(Subset::as_inner(input))
    }
}
