//! Numeric sign, zero, parity, divisibility, and four-tier binary-partitioning predicates across float and integer domains.
//!
//! Provides:
//! 1. **Four-tier binary-partition ladder**:
//!    - Representation layer (raw `f32` / `f64`): [`IsNaN`] and [`IsNotNaN`] (`= Not<IsNaN>`);
//!    - Extended reals (`NotNaNF32` / `NotNaNF64`): [`IsFinite`] and [`IsInfinite`] (`= Not<IsFinite>`);
//!    - Finite reals (`FiniteF32` / `FiniteF64`): [`IsInteger`];
//!    - Integer domain (all 12 primitive integer types plus `Subset<IsInteger, FiniteF32/64>`): [`IsEven`] and [`IsOdd`] (`= Not<IsEven>`).
//! 2. **Sign and zero checks (transparent comparison aliases against `ConstU8<0>`)**:
//!    [`IsZero`], [`IsNonZero`], [`IsPositive`], [`IsNegative`], [`IsNonNegative`], [`IsNonPositive`];
//! 3. **Total panic-free integer divisibility**:
//!    [`IsMultipleOf`], plus transparent aliases [`PairIsMultipleOf`], [`ForkIsMultipleOf`], [`IsMultipleOfConst`].
//!
//! # Complement Closure & Four-Tier Domain Stratification Contract
//! - **Tier 1 (IEEE 754 raw floats `f32` / `f64`)**: implements only [`IsNaN`] (and [`IsNotNaN`] = `Not<IsNaN>`),
//!   intentionally omitting [`SafeEq`](crate::cmp::SafeEq), [`SafeOrd`](crate::cmp::SafeOrd),
//!   sign, finiteness, and integrality predicates.
//! - **Tier 2 (Extended reals `NotNaNF32/64`)**: once `NaN` is excluded, carriers unlock exact
//!   mathematical total ordering and all ordering/sign predicates, together with [`IsFinite`] and
//!   [`IsInfinite`] (`= Not<IsFinite>`). "Not finite" means "infinite" with no `NaN` leakage.
//! - **Tier 3 (Finite reals `FiniteF32/64` = `Subset<IsFinite, NotNaNF32/64>`)**: once both `NaN`
//!   and `±infinity` are excluded, [`IsInteger`] is unlocked; its complement `Not<IsInteger>`
//!   strictly denotes finite numbers with non-zero fractional part.
//! - **Tier 4 (Integer domain $\mathbb{Z}$)**: [`IsEven`] and [`IsOdd`] (`= Not<IsEven>`) are unlocked
//!   only on primitive integers and finite float subsets certified by [`IsInteger`], preserving
//!   $\neg\text{Even}(n) \iff \text{Odd}(n)$ strictly without fractions or `NaN`.
//!
//! # Total Panic-Free Integer Divisibility ([`IsMultipleOf`])
//! Tests the mathematical proposition $\exists k \in \mathbb{Z},\; a = k \cdot b$ for `(a, b)`:
//! - When `b == 0`, evaluates to `a == 0` (never panics on division by zero);
//! - When signed `b == -1`, always returns `true` (never panics on `T::MIN % -1`);
//! - Otherwise returns `a % b == 0`.
//!
//! ```
//! use fy_refine_core::function::Function;
//! use fy_refine_core::function::constant::{ConstI32, ConstU8};
//! use fy_refine_core::function::predicate::{
//!     Closed, IsEven, IsFinite, IsInfinite, IsInteger, IsMultipleOf, IsMultipleOfConst,
//!     IsNonPositive, IsNonZero, IsNotNaN, IsOdd, IsPositive, IsZero, Range,
//! };
//! use fy_refine_core::subset::{FiniteF32, FiniteF64, NotNaNF32, NotNaNF64, Subset};
//!
//! assert!(IsZero::call(0i32));
//! assert!(IsPositive::call(5u64));
//! assert!(IsNonZero::call(&-100isize));
//! assert!(IsEven::call(42i32));
//! assert!(IsOdd::call(&-3i32));
//! assert!(!IsNotNaN::call(f32::NAN));
//!
//! // Progressive narrowing: Tier 1 -> 2 -> 3 -> 4
//! let nn_fin = NotNaNF64::try_new(1.5).unwrap();
//! let nn_inf = NotNaNF64::try_new(f64::INFINITY).unwrap();
//! assert!(IsFinite::call(nn_fin) && !IsInfinite::call(nn_fin));
//! assert!(!IsFinite::call(nn_inf) && IsInfinite::call(nn_inf));
//!
//! let fin_neg3 = FiniteF32::try_new(NotNaNF32::try_new(-3.0).unwrap()).unwrap();
//! let fin_frac = FiniteF32::try_new(NotNaNF32::try_new(0.5).unwrap()).unwrap();
//! assert!(IsInteger::call(fin_neg3));
//! assert!(!IsInteger::call(fin_frac));
//!
//! let int_f32 = Subset::<IsInteger, FiniteF32>::try_new(fin_neg3).unwrap();
//! assert!(IsOdd::call(&int_f32));
//! assert!(!IsEven::call(int_f32));
//!
//! assert!(IsMultipleOf::call((12i32, 4i32)));
//! assert!(IsMultipleOf::call((i8::MIN, -1i8)));
//! assert!(IsMultipleOf::call((0i32, 0i32)));
//! assert!(!IsMultipleOf::call((5i32, 0i32)));
//! assert!(IsMultipleOfConst::<ConstI32<3>>::call(&9i32));
//!
//! type NonPosF32 = Subset<IsNonPositive, NotNaNF32>;
//! assert!(NonPosF32::try_new(NotNaNF32::try_new(-2.5).unwrap()).is_ok());
//!
//! type UnitInterval = Subset<Range<Closed<ConstU8<0>>, Closed<ConstU8<1>>>, FiniteF64>;
//! assert!(UnitInterval::try_new(FiniteF64::try_new(NotNaNF64::try_new(0.5).unwrap()).unwrap()).is_ok());
//! ```

use super::{EqConst, GeConst, GtConst, LeConst, LtConst, NeConst, Not, Predicate};
use crate::function::SafeFunction;
use crate::function::constant::ConstU8;
use crate::function::wiring::{AttachUnit, Fork, Id, MapPair, Then};
use crate::marker::SafeDrop;
use crate::subset::Subset;

#[cfg(feature = "nightly")]
mod num_nightly;
#[cfg(not(feature = "nightly"))]
mod num_stable;

#[cfg(feature = "nightly")]
pub use num_nightly::{
    NestedSubsetEvenCarrier2, NestedSubsetEvenCarrier3, NestedSubsetIntegerCarrier,
    SubsetEvenCarrier, SubsetFiniteCarrier, SubsetIntegerCarrier,
};
#[cfg(not(feature = "nightly"))]
pub use num_stable::{
    NestedSubsetEvenCarrier2, NestedSubsetEvenCarrier3, NestedSubsetIntegerCarrier,
    SubsetEvenCarrier, SubsetFiniteCarrier, SubsetIntegerCarrier,
};

// ==========================================
// 1. Predicate Types & Transparent Combinator Aliases
// ==========================================

/// Checks whether a number equals 0 (`x == 0`), transparent alias for [`EqConst<ConstU8<0>>`].
pub type IsZero = EqConst<ConstU8<0>>;

/// Checks whether a number is non-zero (`x != 0`), transparent alias for [`NeConst<ConstU8<0>>`].
pub type IsNonZero = NeConst<ConstU8<0>>;

/// Checks whether a number is strictly positive (`x > 0`), transparent alias for [`GtConst<ConstU8<0>>`].
pub type IsPositive = GtConst<ConstU8<0>>;

/// Checks whether a number is strictly negative (`x < 0`), transparent alias for [`LtConst<ConstU8<0>>`].
pub type IsNegative = LtConst<ConstU8<0>>;

/// Checks whether a number is non-negative (`x >= 0`), transparent alias for [`GeConst<ConstU8<0>>`].
pub type IsNonNegative = GeConst<ConstU8<0>>;

/// Checks whether a number is non-positive (`x <= 0`), transparent alias for [`LeConst<ConstU8<0>>`].
pub type IsNonPositive = LeConst<ConstU8<0>>;

/// Checks whether a non-`NaN` floating-point number (`NotNaNF32` / `NotNaNF64` or outer subsets) is finite.
pub struct IsFinite;

/// Checks whether a non-`NaN` extended real is `+infinity` or `-infinity` (transparent alias for [`Not<IsFinite>`]).
pub type IsInfinite = Not<IsFinite>;

/// Checks whether a floating-point value is `NaN`.
pub struct IsNaN;

/// Checks whether a floating-point value is not `NaN` (includes finite values and infinities), transparent alias for [`Not<IsNaN>`].
pub type IsNotNaN = Not<IsNaN>;

/// Checks whether a finite floating-point number (`FiniteF32` / `FiniteF64` or outer subsets) is an exact mathematical integer (zero fractional part).
pub struct IsInteger;

/// Checks whether an integer-domain value is even (`x % 2 == 0`).
///
/// Accepts all 12 primitive integer types as well as refined subsets statically proven to lie in the integer domain
/// (such as `Subset<IsInteger, FiniteF32/64>` and outer nested subsets).
pub struct IsEven;

/// Checks whether an integer-domain value is odd (transparent alias for [`Not<IsEven>`]).
pub type IsOdd = Not<IsEven>;

/// Binary total integer divisibility predicate: checks whether the left operand is an integer multiple of the right (without division-by-zero or signed `MIN % -1` panics).
pub struct IsMultipleOf;

/// Takes `(A, B)`, applies `L` and `R` respectively, and checks whether the left result is a multiple of the right.
pub type PairIsMultipleOf<L, R> = Then<MapPair<L, R>, IsMultipleOf>;

/// Takes a single `Copy` value, forks to `L` and `R`, and checks whether `L`'s output is a multiple of `R`'s output.
pub type ForkIsMultipleOf<L, R> = Then<Fork<L, R>, IsMultipleOf>;

/// Unary constant divisibility predicate: checks whether the input is an integer multiple of constant morphism `C`.
pub type IsMultipleOfConst<C> = Then<AttachUnit, PairIsMultipleOf<Id, C>>;

// ==========================================
// 2. SafeFunction Contract Guarantees
// ==========================================

macro_rules! impl_pure_for_floats {
    ($($t:ty),* $(,)?) => {
        $(
            // SAFETY: Raw floats admit only NaN bit-pattern classification; pure, total, terminating, and panic-free.
            unsafe impl SafeFunction<$t> for IsNaN {}
            unsafe impl<'a> SafeFunction<&'a $t> for IsNaN {}
        )*
    };
}

macro_rules! impl_pure_for_ints_extra {
    ($($t:ty),* $(,)?) => {
        $(
            // SAFETY: Parity checks and total divisibility exclude division by zero and MIN % -1 overflow; pure, terminating, and panic-free.
            unsafe impl SafeFunction<$t> for IsEven {}
            unsafe impl<'a> SafeFunction<&'a $t> for IsEven {}

            unsafe impl SafeFunction<($t, $t)> for IsMultipleOf {}
            unsafe impl<'a> SafeFunction<(&'a $t, $t)> for IsMultipleOf {}
            unsafe impl<'a> SafeFunction<($t, &'a $t)> for IsMultipleOf {}
            unsafe impl<'a, 'b> SafeFunction<(&'a $t, &'b $t)> for IsMultipleOf {}
        )*
    };
}

impl_pure_for_floats!(f32, f64);
impl_pure_for_ints_extra!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize,
);

// SAFETY: `T: SubsetFiniteCarrier<P>` certifies the inner float already excludes `NaN` and `T: SafeDrop` ensures benign drop; finiteness classification is pure, deterministic, terminating, and panic-free.
unsafe impl<P, T> SafeFunction<Subset<P, T>> for IsFinite
where
    P: for<'a> Predicate<&'a T>,
    T: SubsetFiniteCarrier<P> + SafeDrop,
{
}

unsafe impl<P, T: ?Sized> SafeFunction<&Subset<P, T>> for IsFinite
where
    P: for<'b> Predicate<&'b T>,
    T: SubsetFiniteCarrier<P>,
{
}

// SAFETY: `T: SubsetIntegerCarrier<P>` statically guarantees a finite floating-point carrier (neither `NaN` nor infinite)
// and `T: SafeDrop` ensures benign drop; bitwise integer testing is pure, terminating, and panic-free.
unsafe impl<P, T> SafeFunction<Subset<P, T>> for IsInteger
where
    P: for<'a> Predicate<&'a T>,
    T: SubsetIntegerCarrier<P> + SafeDrop,
{
}

unsafe impl<P, T: ?Sized> SafeFunction<&Subset<P, T>> for IsInteger
where
    P: for<'b> Predicate<&'b T>,
    T: SubsetIntegerCarrier<P>,
{
}

// SAFETY: `T: SubsetEvenCarrier<P>` statically guarantees an integer-domain carrier (primitive integer or `IsInteger`-proven finite float)
// and `T: SafeDrop` ensures benign drop; parity testing is pure, total, panic-free, and terminating.
unsafe impl<P, T> SafeFunction<Subset<P, T>> for IsEven
where
    P: for<'a> Predicate<&'a T>,
    T: SubsetEvenCarrier<P> + SafeDrop,
{
}

// SAFETY: Same invariant as above; shared-reference form neither clones nor drops the carrier.
unsafe impl<P, T: ?Sized> SafeFunction<&Subset<P, T>> for IsEven
where
    P: for<'b> Predicate<&'b T>,
    T: SubsetEvenCarrier<P>,
{
}
