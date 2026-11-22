//! Length projection, emptiness predicates, universal/existential quantifiers, and boundary/indexed element access for generic collection containers.
//!
//! Separate from contiguous memory slice views ([`crate::function::slice`]), capabilities are layered by container logical structure:
//! - **Length, emptiness, and quantifiers** ([`Len`], [`IsEmpty`], [`IsNonEmpty`], [`LenGt`], [`All`], [`Any`]):
//!   - `core`: `&[T]`, `&[T; N]` (with conditional `const` under `nightly`), plus length and emptiness for `&str`;
//!   - `alloc`: `&String` (length/emptiness), `&Vec<T>`, `&VecDeque<T>`, `&LinkedList<T>`,
//!     `&BinaryHeap<T>`, `&BTreeSet<T>`, `&BTreeMap<K, V>` (quantifiers receive `(&K, &V)`);
//!   - `std`: `&HashSet<T, S>`, `&HashMap<K, V, S>` (quantifiers receive `(&K, &V)`, with results independent of hash iteration order).
//! - **First and last element access** ([`TryFirst`], [`TryLast`], [`First`], [`Last`]):
//!   Provided only for containers with well-defined first/last semantics (`[T]`, `[T; N]`, `Vec`, `VecDeque`, `LinkedList`,
//!   `BTreeSet`, `BTreeMap`, where `BTreeMap` yields `(&K, &V)`); unordered hash containers do not provide first/last access.
//! - **Constant-index access** ([`TryGet`], [`Get`]):
//!   Provided for sequence containers supporting positional indexing (`[T]`, `[T; N]`, `Vec`, `VecDeque`).
//!
//! ```
//! use fy_refine_core::function::Function;
//! use fy_refine_core::function::collection::{All, Any, First, Get, IsNonEmpty, Last, Len, LenGt};
//! use fy_refine_core::function::predicate::IsPositive;
//! use fy_refine_core::subset::Subset;
//!
//! let nums = [1i32, 2, 3];
//! assert_eq!(Len::call(&nums), 3);
//! assert!(IsNonEmpty::call(&nums));
//! assert!(All::<IsPositive>::call(&nums));
//! assert!(Any::<IsPositive>::call(&[-1, 0, 5][..]));
//!
//! let non_empty = Subset::<IsNonEmpty, [i32]>::try_from_ref(&nums[..]).unwrap();
//! assert_eq!(*First::call(non_empty), 1);
//! assert_eq!(*Last::call(non_empty), 3);
//!
//! let len_gt_1 = Subset::<LenGt<1>, [i32]>::try_from_ref(&nums[..]).unwrap();
//! assert_eq!(*Get::<1>::call(len_gt_1), 2);
//! ```

use crate::function::SafeFunction;
use crate::function::constant::ConstUsize;
use crate::function::predicate::{GtConst, Not};
use crate::function::wiring::Then;
use crate::marker::{Phantom, SafeValue};
use crate::subset::Subset;

mod alloc_std_impls;
#[cfg(feature = "nightly")]
mod collection_nightly;
#[cfg(not(feature = "nightly"))]
mod collection_stable;

/// Extracts the element count or byte length (`usize`) of a collection container, slice, or string.
pub struct Len;

/// Checks whether a collection container, slice, or string is empty.
pub struct IsEmpty;

/// Checks whether a collection container, slice, or string is non-empty (transparent alias for [`Not<IsEmpty>`]).
pub type IsNonEmpty = Not<IsEmpty>;

/// Length predicate alias requiring strictly more than `I` elements (used to constrain [`Get`]).
pub type LenGt<const I: usize> = Then<Len, GtConst<ConstUsize<I>>>;

/// Universal quantifier predicate: returns `true` if and only if all elements in the container satisfy `P` (always `true` for empty containers).
///
/// For key-value maps (`BTreeMap`, `HashMap`), `P` receives `(&K, &V)` for each entry;
/// for single-element containers, `P` receives `&T`.
pub struct All<P>(Phantom<P>);

/// Existential quantifier predicate: returns `true` if and only if at least one element in the container satisfies `P` (always `false` for empty containers).
///
/// For key-value maps (`BTreeMap`, `HashMap`), `P` receives `(&K, &V)` for each entry;
/// for single-element containers, `P` receives `&T`.
pub struct Any<P>(Phantom<P>);

/// Fallible first-element projection: borrows the first element from an ordered or double-ended container (returns `None` if empty).
pub struct TryFirst;

/// Fallible last-element projection: borrows the last element from an ordered or double-ended container (returns `None` if empty).
pub struct TryLast;

/// Fallible indexed projection: borrows `Option<&T>` at compile-time constant index `I` from a sequence container, returning `None` if out of bounds.
pub struct TryGet<const I: usize>;

/// Infallible first-element extractor: borrows the first element directly from a refined container known to be non-empty (`&Subset<IsNonEmpty, _>`) with zero branches.
pub struct First;

/// Infallible last-element extractor: borrows the last element directly from a refined container known to be non-empty (`&Subset<IsNonEmpty, _>`) with zero branches.
pub struct Last;

/// Infallible indexed extractor: borrows `&T` directly from a refined sequence known to have length strictly greater than `I` (`&Subset<LenGt<I>, _>`) with zero bounds checks.
pub struct Get<const I: usize>;

// ==========================================
// SafeFunction Contract Guarantees (core)
// ==========================================

// SAFETY: Reads slice/str/array metadata; pure, guaranteed to terminate, and panic-free.
unsafe impl<T> SafeFunction<&[T]> for Len {}
unsafe impl SafeFunction<&str> for Len {}
unsafe impl<T, const N: usize> SafeFunction<&[T; N]> for Len {}

unsafe impl<T> SafeFunction<&[T]> for IsEmpty {}
unsafe impl SafeFunction<&str> for IsEmpty {}
unsafe impl<T, const N: usize> SafeFunction<&[T; N]> for IsEmpty {}

// SAFETY: Iterates over a finite-length collection invoking safe predicate P; short-circuiting, guaranteed to terminate, and free of destructive side effects.
unsafe impl<T, P> SafeFunction<&[T]> for All<P> where P: for<'b> SafeFunction<&'b T, Output = bool> {}
unsafe impl<T, const N: usize, P> SafeFunction<&[T; N]> for All<P> where
    P: for<'b> SafeFunction<&'b T, Output = bool>
{
}

unsafe impl<T, P> SafeFunction<&[T]> for Any<P> where P: for<'b> SafeFunction<&'b T, Output = bool> {}
unsafe impl<T, const N: usize, P> SafeFunction<&[T; N]> for Any<P> where
    P: for<'b> SafeFunction<&'b T, Output = bool>
{
}

// SAFETY: Bounds-checked first/last/indexed reads; pure, guaranteed to terminate, and panic-free.
unsafe impl<T> SafeFunction<&[T]> for TryFirst {}
unsafe impl<T, const N: usize> SafeFunction<&[T; N]> for TryFirst {}

unsafe impl<T> SafeFunction<&[T]> for TryLast {}
unsafe impl<T, const N: usize> SafeFunction<&[T; N]> for TryLast {}

unsafe impl<T, const I: usize> SafeFunction<&[T]> for TryGet<I> {}
unsafe impl<T, const N: usize, const I: usize> SafeFunction<&[T; N]> for TryGet<I> {}

// SAFETY: The strong invariant of the refined subset guarantees the container is non-empty or has length > I, so branchless reference access is safe and guaranteed to terminate.
unsafe impl<T: SafeValue> SafeFunction<&Subset<IsNonEmpty, [T]>> for First {}
unsafe impl<T: SafeValue, const N: usize> SafeFunction<&Subset<IsNonEmpty, [T; N]>> for First {}

unsafe impl<T: SafeValue> SafeFunction<&Subset<IsNonEmpty, [T]>> for Last {}
unsafe impl<T: SafeValue, const N: usize> SafeFunction<&Subset<IsNonEmpty, [T; N]>> for Last {}

unsafe impl<T: SafeValue, const I: usize> SafeFunction<&Subset<LenGt<I>, [T]>> for Get<I> {}
unsafe impl<T: SafeValue, const N: usize, const I: usize> SafeFunction<&Subset<LenGt<I>, [T; N]>>
    for Get<I>
{
}
