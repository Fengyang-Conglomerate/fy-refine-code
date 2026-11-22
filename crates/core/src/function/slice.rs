//! Contiguous slice view projections and proof-carrying string conversions.
//!
//! Generic collection length, emptiness, quantifiers, and first/last/indexed access live in [`crate::function::collection`];
//! this module provides zero-copy view conversions relying on contiguous memory layout ([`AsBytes`], [`AsSlice`], [`AsStr`]).
//!
//! ```
//! use fy_refine_core::function::Function;
//! use fy_refine_core::function::collection::{All, Len};
//! use fy_refine_core::function::constant::ConstUsize;
//! use fy_refine_core::function::predicate::{And, EqConst, IsAscii, IsAsciiAlphabetic, IsUtf8};
//! use fy_refine_core::function::slice::{AsBytes, AsStr};
//! use fy_refine_core::function::wiring::Then;
//! use fy_refine_core::subset::Subset;
//!
//! // Express a 4-letter ASCII script code (such as "Latn")
//! type ScriptPred = Then<AsBytes, And<Then<Len, EqConst<ConstUsize<4>>>, All<IsAsciiAlphabetic>>>;
//! type ScriptStr = Subset<ScriptPred, str>;
//! assert!(ScriptStr::try_from_ref("Latn").is_some());
//! assert!(ScriptStr::try_from_ref("La1n").is_none());
//!
//! // Byte slices carrying ASCII / UTF-8 proofs convert directly to &str in O(1) without Result
//! let ascii_bytes = Subset::<All<IsAscii>, [u8]>::try_from_ref(b"refine").unwrap();
//! assert_eq!(AsStr::call(ascii_bytes), "refine");
//!
//! let utf8_bytes = Subset::<IsUtf8, [u8]>::try_from_ref("café🦀".as_bytes()).unwrap();
//! assert_eq!(AsStr::call(utf8_bytes), "café🦀");
//! ```

use crate::function::SafeFunction;
use crate::function::collection::All;
use crate::function::predicate::{IsAscii, IsUtf8};
use crate::subset::Subset;

#[cfg(feature = "nightly")]
mod slice_nightly;
#[cfg(not(feature = "nightly"))]
mod slice_stable;

/// Projects a string slice (or `&String` when `alloc` is enabled) to a read-only byte slice `&[u8]`.
pub struct AsBytes;

/// Projects a fixed-length array reference (or `&Vec<T>` when `alloc` is enabled) to a read-only slice `&[T]`.
pub struct AsSlice;

/// Converts a byte slice/array (or `&Subset<..., Vec<u8>>` with `alloc`) carrying an `All<IsAscii>` or `IsUtf8` proof
/// into `&str` in `O(1)` zero-copy without returning `Result`.
pub struct AsStr;

// SAFETY: Zero-copy contiguous memory view conversions; pure, guaranteed to terminate, and panic-free.
unsafe impl SafeFunction<&str> for AsBytes {}
unsafe impl<T, const N: usize> SafeFunction<&[T; N]> for AsSlice {}

// SAFETY: ASCII is a strict subset of UTF-8; both `All<IsAscii>` and `IsUtf8` guarantee valid UTF-8,
// and `SafeValue` prevents interior mutation, making conversion pure, deterministic, terminating, and panic-free.
unsafe impl SafeFunction<&Subset<All<IsAscii>, [u8]>> for AsStr {}
unsafe impl<const N: usize> SafeFunction<&Subset<All<IsAscii>, [u8; N]>> for AsStr {}
unsafe impl SafeFunction<&Subset<IsUtf8, [u8]>> for AsStr {}
unsafe impl<const N: usize> SafeFunction<&Subset<IsUtf8, [u8; N]>> for AsStr {}

#[cfg(feature = "alloc")]
mod alloc_impls {
    use super::*;
    use crate::function::Function;
    use alloc::string::String;
    use alloc::vec::Vec;

    impl<'a> Function<&'a String> for AsBytes {
        type Output = &'a [u8];

        #[inline(always)]
        fn call(input: &'a String) -> &'a [u8] {
            input.as_bytes()
        }
    }

    impl<'a, T> Function<&'a Vec<T>> for AsSlice {
        type Output = &'a [T];

        #[inline(always)]
        fn call(input: &'a Vec<T>) -> &'a [T] {
            input.as_slice()
        }
    }

    impl<'a> Function<&'a Subset<All<IsAscii>, Vec<u8>>> for AsStr {
        type Output = &'a str;

        #[inline(always)]
        fn call(input: &'a Subset<All<IsAscii>, Vec<u8>>) -> &'a str {
            let bytes = Subset::as_inner(input).as_slice();
            // SAFETY: The strong invariant of `Subset<All<IsAscii>, Vec<u8>>` guarantees every byte is 7-bit ASCII (valid UTF-8).
            unsafe { core::str::from_utf8_unchecked(bytes) }
        }
    }

    impl<'a> Function<&'a Subset<IsUtf8, Vec<u8>>> for AsStr {
        type Output = &'a str;

        #[inline(always)]
        fn call(input: &'a Subset<IsUtf8, Vec<u8>>) -> &'a str {
            let bytes = Subset::as_inner(input).as_slice();
            // SAFETY: The strong invariant of `Subset<IsUtf8, Vec<u8>>` guarantees the byte sequence is valid UTF-8.
            unsafe { core::str::from_utf8_unchecked(bytes) }
        }
    }

    // SAFETY: Reads read-only slice view of standard library contiguous buffers; pure, guaranteed to terminate, and panic-free.
    unsafe impl SafeFunction<&String> for AsBytes {}
    unsafe impl<T> SafeFunction<&Vec<T>> for AsSlice {}
    unsafe impl SafeFunction<&Subset<All<IsAscii>, Vec<u8>>> for AsStr {}
    unsafe impl SafeFunction<&Subset<IsUtf8, Vec<u8>>> for AsStr {}
}
