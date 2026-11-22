//! Unicode scalar value (`char`) and UTF-8 byte sequence refinement predicates and proof-carrying conversions.
//!
//! Provides:
//! - [`IsChar`]: transparent interval disjunction over `u32` checking Unicode scalar value validity (`0x0000..=0xD7FF` or `0xE000..=0x10FFFF`);
//! - [`U32ToChar`]: takes `Subset<IsChar, u32>` (or a shared reference) and returns `char` directly without `Option` or branching;
//! - [`CharToU32`]: lifts `char` (or `&char`) into `Subset<IsChar, u32>` at zero runtime cost;
//! - [`IsUtf8`]: checks whether a byte slice/array (or `&Vec<u8>` with `alloc`) is valid UTF-8.
//!
//! ```
//! use fy_refine_core::function::Function;
//! use fy_refine_core::function::predicate::{CharToU32, IsChar, IsUtf8, U32ToChar};
//! use fy_refine_core::subset::Subset;
//!
//! let code = Subset::<IsChar, u32>::try_new(0x1F980).unwrap();
//! assert_eq!(U32ToChar::call(code), '🦀');
//! assert!(Subset::<IsChar, u32>::try_new(0xD800).is_err());
//!
//! let roundtrip = CharToU32::call('🦀');
//! assert_eq!(*roundtrip, 0x1F980);
//!
//! assert!(IsUtf8::call("café🦀".as_bytes()));
//! assert!(!IsUtf8::call(&[0xff, 0xfe][..]));
//! ```

use super::{Closed, LeConst, Or, Range};
use crate::function::SafeFunction;
use crate::function::constant::ConstU32;
use crate::subset::Subset;

#[cfg(feature = "nightly")]
mod utf8_nightly;
#[cfg(not(feature = "nightly"))]
mod utf8_stable;

/// Checks whether a `u32` is a valid Unicode scalar value (`0x0000..=0xD7FF` or `0xE000..=0x10FFFF`).
pub type IsChar =
    Or<LeConst<ConstU32<0xD7FF>>, Range<Closed<ConstU32<0xE000>>, Closed<ConstU32<0x10FFFF>>>>;

/// Converts a proven `Subset<IsChar, u32>` directly into [`char`] without `Option` or runtime branching.
pub struct U32ToChar;

/// Lifts a [`char`] into `Subset<IsChar, u32>` with zero runtime checking overhead.
pub struct CharToU32;

/// Checks whether a byte sequence (`&[u8]`, `&[u8; N]`, or `&Vec<u8>` when `alloc` is enabled) is valid UTF-8.
pub struct IsUtf8;

// SAFETY: Subset<IsChar, u32> guarantees the scalar falls in a valid Unicode scalar range; pure, deterministic, terminating, and panic-free.
unsafe impl SafeFunction<Subset<IsChar, u32>> for U32ToChar {}
unsafe impl SafeFunction<&Subset<IsChar, u32>> for U32ToChar {}

// SAFETY: Rust's language validity invariant on `char` guarantees `c as u32` satisfies `IsChar`; pure, terminating, and panic-free.
unsafe impl SafeFunction<char> for CharToU32 {}
unsafe impl SafeFunction<&char> for CharToU32 {}

// SAFETY: UTF-8 validation is pure, deterministic, terminating, and panic-free.
unsafe impl SafeFunction<&[u8]> for IsUtf8 {}
unsafe impl<const N: usize> SafeFunction<&[u8; N]> for IsUtf8 {}

#[cfg(feature = "alloc")]
mod alloc_impls {
    use super::*;
    use crate::function::Function;
    use alloc::vec::Vec;

    impl<'a> Function<&'a Vec<u8>> for IsUtf8 {
        type Output = bool;

        #[inline(always)]
        fn call(input: &'a Vec<u8>) -> bool {
            core::str::from_utf8(input.as_slice()).is_ok()
        }
    }

    // SAFETY: Validates read-only slice view of Vec<u8> as UTF-8; pure, terminating, and panic-free.
    unsafe impl SafeFunction<&Vec<u8>> for IsUtf8 {}
}
