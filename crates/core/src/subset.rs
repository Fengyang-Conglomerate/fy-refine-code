//! Refined subset carrier type whose inner value satisfies a pure predicate invariant.
//!
//! Follows the `C-SMART-PTR` Rust API guideline: since [`Subset`] implements `Deref<Target = T>`,
//! all of its own operations are associated functions (`Subset::fn(this)`) to avoid shadowing methods on `T`.

use core::mem;
use core::ptr;

use crate::function::Predicate;
use crate::marker::Phantom;
pub use crate::marker::SafeValue;

#[cfg(feature = "nightly")]
mod cmp_impls_nightly;
#[cfg(not(feature = "nightly"))]
mod cmp_impls_stable;
mod marker_traits;
#[cfg(feature = "serde")]
mod serde_impls;
mod std_traits;

#[cfg(feature = "nightly")]
mod subset_nightly;
#[cfg(not(feature = "nightly"))]
mod subset_stable;

#[cfg(feature = "nightly")]
pub use cmp_impls_nightly::{
    FloatRhsEq, FloatRhsOrd, NonNaNFloatPredicate, SubsetCarrierEq, SubsetCarrierOrd,
    SubsetFloatRhsEq, SubsetFloatRhsOrd,
};
#[cfg(not(feature = "nightly"))]
pub use cmp_impls_stable::{
    FloatRhsEq, FloatRhsOrd, NonNaNFloatPredicate, SubsetCarrierEq, SubsetCarrierOrd,
    SubsetFloatRhsEq, SubsetFloatRhsOrd,
};

/// Non-`NaN` `f32` subset (finite numbers and `±infinity`, supporting strict total ordering via [`SafeOrd`](crate::cmp::SafeOrd)).
pub type NotNaNF32 = Subset<crate::function::predicate::IsNotNaN, f32>;

/// Non-`NaN` `f64` subset (finite numbers and `±infinity`, supporting strict total ordering via [`SafeOrd`](crate::cmp::SafeOrd)).
pub type NotNaNF64 = Subset<crate::function::predicate::IsNotNaN, f64>;

/// Finite `f32` subset (excluding `NaN` and `±infinity`, i.e., `Subset<IsFinite, NotNaNF32>`, supporting strict total ordering via [`SafeOrd`](crate::cmp::SafeOrd)).
pub type FiniteF32 = Subset<crate::function::predicate::IsFinite, NotNaNF32>;

/// Finite `f64` subset (excluding `NaN` and `±infinity`, i.e., `Subset<IsFinite, NotNaNF64>`, supporting strict total ordering via [`SafeOrd`](crate::cmp::SafeOrd)).
pub type FiniteF64 = Subset<crate::function::predicate::IsFinite, NotNaNF64>;

pub(super) mod float_cmp {
    use core::cmp::Ordering;

    #[inline(always)]
    pub const fn cmp_non_nan_f64(a: f64, b: f64) -> Ordering {
        if a < b {
            Ordering::Less
        } else if a > b {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }

    #[inline]
    pub const fn cmp_non_nan_f64_u128(f: f64, n: u128) -> Ordering {
        let bits = f.to_bits();
        let sign = (bits >> 63) != 0;
        let exp_bits = ((bits >> 52) & 0x7ff) as i32;
        let mant_bits = bits & 0x000f_ffff_ffff_ffff;

        if exp_bits == 0 && mant_bits == 0 {
            return if n == 0 {
                Ordering::Equal
            } else {
                Ordering::Less
            };
        }
        if sign {
            return Ordering::Less;
        }
        if exp_bits == 0x7ff {
            return Ordering::Greater;
        }
        if exp_bits < 1023 {
            return if n == 0 {
                Ordering::Greater
            } else {
                Ordering::Less
            };
        }

        let e = (exp_bits - 1023) as u32;
        if e >= 128 {
            return Ordering::Greater;
        }
        let sig = ((1u64 << 52) | mant_bits) as u128;
        if e >= 52 {
            let f_int = sig << (e - 52);
            if f_int < n {
                Ordering::Less
            } else if f_int > n {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        } else {
            let frac_bits = 52 - e;
            let f_floor = sig >> frac_bits;
            let f_rem = sig & ((1u128 << frac_bits) - 1);
            if f_floor < n {
                Ordering::Less
            } else if f_floor > n {
                Ordering::Greater
            } else if f_rem == 0 {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        }
    }

    #[inline]
    pub const fn cmp_non_nan_f64_i128(f: f64, n: i128) -> Ordering {
        if n >= 0 {
            cmp_non_nan_f64_u128(f, n as u128)
        } else {
            let abs_n = (n as u128).wrapping_neg();
            cmp_non_nan_f64_u128(-f, abs_n).reverse()
        }
    }
}

/// Wrapper representing carrier values of `T` satisfying predicate `P` (`P(&T) == true`).
///
/// - `P` must accept `&T` for any lifetime and lives strictly at the type level (zero runtime storage);
/// - `T` must satisfy [`SafeValue`] so validated predicates cannot drift under shared references;
/// - Natively supports dynamically sized types (`?Sized`), including `str` and `[T]`;
/// - Uses `#[repr(transparent)]` with identical memory layout to `T`.
#[repr(transparent)]
pub struct Subset<P, T: ?Sized>
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue,
{
    marker: Phantom<P>,
    value: T,
}

impl<P, T: ?Sized> Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue,
{
    /// Constructs a refined shared reference view without running `P` (unchecked path).
    ///
    /// # Safety
    /// Caller must ensure `value` strictly satisfies `P(&T)`.
    #[inline(always)]
    pub const unsafe fn from_ref_unchecked(value: &T) -> &Self {
        // SAFETY: `#[repr(transparent)]` guarantees identical layout; caller guarantees `P(&T)`.
        unsafe { &*(value as *const T as *const Self) }
    }

    /// Strips the outermost wrapper layer and borrows the immediate inner carrier.
    #[inline(always)]
    pub const fn as_inner(this: &Self) -> &T {
        &this.value
    }
}

impl<P, T> Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue,
{
    /// Constructs an owned refined value without running `P` (unchecked path).
    ///
    /// # Safety
    /// Caller must ensure `value` strictly satisfies `P(&T)`.
    #[inline(always)]
    pub const unsafe fn new_unchecked(value: T) -> Self {
        Self {
            marker: core::marker::PhantomData,
            value,
        }
    }

    /// Strips the outermost wrapper layer and moves out the immediate inner carrier.
    #[inline(always)]
    pub const fn into_inner(this: Self) -> T {
        // SAFETY: Reads the sole field before forgetting `this` to prevent double drop in `const fn`.
        let value = unsafe { ptr::read(&this.value) };
        mem::forget(this);
        value
    }

    /// Recursively penetrates all nested `Subset` layers and moves out the underlying base carrier.
    #[inline(always)]
    pub const fn into_base(this: Self) -> T::Base
    where
        T::Base: Sized,
    {
        debug_assert!(core::mem::size_of::<Self>() == core::mem::size_of::<T::Base>());
        debug_assert!(core::mem::align_of::<Self>() == core::mem::align_of::<T::Base>());
        // SAFETY: `#[repr(transparent)]` and `SafeValue` guarantee identical layout and ownership semantics
        // between `Self` and `T::Base`; bitwise read followed by `forget(this)` prevents double drop.
        let base = unsafe { ptr::read((&this as *const Self).cast::<T::Base>()) };
        mem::forget(this);
        base
    }
}
