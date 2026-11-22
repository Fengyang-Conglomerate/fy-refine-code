//! Static lookup table and static bitset membership predicates (pure data-query primitives with zero automatic optimization logic).
//!
//! Provides four membership predicates over static tables and the zero-`unsafe` static table macro [`static_table!`](crate::static_table), all `no_std`-compatible and supporting conditional `const` under `nightly`:
//! 1. [`InSliceEq`]: Linear search in a static slice `&'static [U]` using [`SafeEq`] (requires only an equivalence relation, not a total order);
//! 2. [`InSortedSlice`]: Binary search in a strictly ascending static slice `&'static [U]` using [`SafeOrd`] (requires a total order, `O(log n)`);
//! 3. [`InSortedRanges`]: Binary search in an ascending disjoint closed-interval table `&'static [(U, U)]` using [`SafeOrd`] (requires a total order, `O(log k)`);
//! 4. [`InBitset`]: Bitwise membership test of an index in a static bitset `&'static [u64]` (`O(1)`, returns `false` when out of bounds).
//!
//! ```
//! use fy_refine_core::function::{Function, SafeFunction};
//! use fy_refine_core::function::lookup::{
//!     InBitset, InSliceEq, InSortedRanges, InSortedSlice, bitset_u8_from_slice,
//!     is_sorted_disjoint_ranges, is_strictly_sorted,
//! };
//! use fy_refine_core::subset::Subset;
//!
//! // 1. Binary search in a sorted static slice
//! struct Scripts;
//! impl Function<()> for Scripts {
//!     type Output = &'static [&'static str];
//!     fn call((): ()) -> Self::Output { &["Arab", "Cyrl", "Hans", "Hant", "Latn"] }
//! }
//! unsafe impl SafeFunction<()> for Scripts {}
//! assert!(is_strictly_sorted(Scripts::call(())));
//!
//! type KnownScript = Subset<InSortedSlice<Scripts>, str>;
//! assert!(KnownScript::try_from_ref("Hans").is_some());
//! assert!(KnownScript::try_from_ref("Xxxx").is_none());
//!
//! // 2. Static bitset LUT lookup (O(1))
//! const WS_BITS: [u64; 4] = bitset_u8_from_slice(b" \t\n\x0C\r");
//! struct WsTable;
//! impl Function<()> for WsTable {
//!     type Output = &'static [u64];
//!     fn call((): ()) -> Self::Output { &WS_BITS }
//! }
//! unsafe impl SafeFunction<()> for WsTable {}
//! assert!(InBitset::<WsTable>::call(b' '));
//! assert!(!InBitset::<WsTable>::call(b'a'));
//! ```

use crate::cmp::{SafeEq, SafeOrd};
use crate::function::SafeFunction;
use crate::marker::{Phantom, SafeDrop};

#[cfg(feature = "nightly")]
mod lookup_nightly;
#[cfg(not(feature = "nightly"))]
mod lookup_stable;

#[cfg(feature = "nightly")]
pub use lookup_nightly::{is_sorted_disjoint_ranges, is_strictly_sorted};
#[cfg(not(feature = "nightly"))]
pub use lookup_stable::{is_sorted_disjoint_ranges, is_strictly_sorted};

/// Linearly searches a static slice `&'static [U]` using [`SafeEq`] to check whether the input is present (requires only equivalence, with no ordering assumptions).
pub struct InSliceEq<Table>(Phantom<Table>);

/// Binary-searches an ascending static slice `&'static [U]` using [`SafeOrd`] to check whether the input is present (`O(log n)`).
///
/// Can be paired with [`is_strictly_sorted`] to verify that the static table is strictly ascending.
pub struct InSortedSlice<Table>(Phantom<Table>);

/// Binary-searches an ascending disjoint closed-interval static table `&'static [(U, U)]` using [`SafeOrd`] to check whether the input falls within any interval (`O(log k)`).
///
/// A singleton value can be represented as `(v, v)`; can be paired with [`is_sorted_disjoint_ranges`] to verify that the static table is sorted and mutually disjoint.
pub struct InSortedRanges<Table>(Phantom<Table>);

/// Performs an `O(1)` bit test in a static bitset `&'static [u64]` (safely returns `false` for out-of-bounds indices).
///
/// Supports `u8`, `u16`, `u32`, `usize`, and their shared references as input.
pub struct InBitset<Table>(Phantom<Table>);

/// Tests whether bit `index` is `1` in the static bitset slice `words`; safely returns `false` when out of bounds.
#[inline(always)]
pub(crate) const fn test_bitset(words: &[u64], index: usize) -> bool {
    let word_idx = index >> 6;
    if word_idx < words.len() {
        ((words[word_idx] >> (index & 63)) & 1) != 0
    } else {
        false
    }
}

/// Encodes a static byte list into a 256-bit bitset `[u64; 4]` (`const fn`, for building byte lookup tables used with [`InBitset`]).
pub const fn bitset_u8_from_slice(bytes: &[u8]) -> [u64; 4] {
    let mut mask = [0u64; 4];
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i] as usize;
        mask[b >> 6] |= 1u64 << (b & 63);
        i += 1;
    }
    mask
}

/// Defines a nullary safe function returning a static constant lookup table, avoiding manual `unsafe impl SafeFunction<()>` in downstream code.
///
/// Internally binds the expression to `const TABLE: $ty` and statically verifies `$ty: SafeValue`,
/// ensuring at compile time that constant evaluation succeeds and the value is pure, deterministic, guaranteed to terminate, and free of interior mutability.
///
/// - Default form: `static_table!(pub Scripts: &'static [&'static str] = &["Arab", "Latn"]);`
/// - Under `nightly` + `#![feature(const_trait_impl)]`, prefix with `const` to generate a `const impl`:
///   `static_table!(pub const Scripts: &'static [&'static str] = &["Arab", "Latn"]);`
#[macro_export]
macro_rules! static_table {
    ($(#[$meta:meta])* $vis:vis const $name:ident : $ty:ty = $expr:expr $(;)?) => {
        $(#[$meta])*
        $vis struct $name;

        const impl $crate::function::Function<()> for $name {
            type Output = $ty;

            #[inline(always)]
            fn call((): ()) -> Self::Output {
                const TABLE: $ty = {
                    struct _AssertSafeValue<T: ?Sized + $crate::marker::SafeValue>(
                        core::marker::PhantomData<T>,
                    );
                    let _: _AssertSafeValue<$ty> = _AssertSafeValue(core::marker::PhantomData);
                    $expr
                };
                TABLE
            }
        }

        // SAFETY: TABLE is a compile-time evaluated static constant satisfying SafeValue, so returning it is constant, pure, guaranteed to terminate, and panic-free.
        unsafe impl $crate::function::SafeFunction<()> for $name {}
    };
    ($(#[$meta:meta])* $vis:vis $name:ident : $ty:ty = $expr:expr $(;)?) => {
        $(#[$meta])*
        $vis struct $name;

        impl $crate::function::Function<()> for $name {
            type Output = $ty;

            #[inline(always)]
            fn call((): ()) -> Self::Output {
                const TABLE: $ty = {
                    struct _AssertSafeValue<T: ?Sized + $crate::marker::SafeValue>(
                        core::marker::PhantomData<T>,
                    );
                    let _: _AssertSafeValue<$ty> = _AssertSafeValue(core::marker::PhantomData);
                    $expr
                };
                TABLE
            }
        }

        // SAFETY: TABLE is a compile-time evaluated static constant satisfying SafeValue, so returning it is constant, pure, guaranteed to terminate, and panic-free.
        unsafe impl $crate::function::SafeFunction<()> for $name {}
    };
}

// ==========================================
// SafeFunction Contract Guarantees
// ==========================================

// SAFETY: Table is a safe nullary function, SafeEq guarantees pure, total, and terminating comparison, and T: SafeDrop guarantees benign drop of the consumed input; finite-length traversal always terminates.
unsafe impl<T, Table, U: 'static> SafeFunction<T> for InSliceEq<Table>
where
    Table: SafeFunction<(), Output = &'static [U]>,
    T: SafeEq<U> + SafeDrop,
{
}

// SAFETY: Table is a safe nullary function, SafeOrd guarantees pure, total, and terminating comparison, and T: SafeDrop guarantees benign drop of the consumed input; the binary-search loop strictly converges.
unsafe impl<T, Table, U: 'static> SafeFunction<T> for InSortedSlice<Table>
where
    Table: SafeFunction<(), Output = &'static [U]>,
    T: SafeOrd<U> + SafeDrop,
{
}

// SAFETY: Same as above; the binary-search loop strictly narrows the search interval on every iteration, guaranteeing termination without panic.
unsafe impl<T, Table, U: 'static> SafeFunction<T> for InSortedRanges<Table>
where
    Table: SafeFunction<(), Output = &'static [(U, U)]>,
    T: SafeOrd<U> + SafeDrop,
{
}

macro_rules! impl_safe_in_bitset {
    ($($ty:ty),* $(,)?) => {
        $(
            // SAFETY: Table is a safe nullary function, and bounds-checked bit testing is pure, deterministic, guaranteed to terminate, and panic-free.
            unsafe impl<Table> SafeFunction<$ty> for InBitset<Table>
            where
                Table: SafeFunction<(), Output = &'static [u64]>,
            {
            }

            unsafe impl<Table> SafeFunction<&$ty> for InBitset<Table>
            where
                Table: SafeFunction<(), Output = &'static [u64]>,
            {
            }
        )*
    };
}

impl_safe_in_bitset!(u8, u16, u32, usize);
