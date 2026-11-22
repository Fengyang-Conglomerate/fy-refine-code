//! Nightly `const`-capable implementations of proof-carrying Unicode scalar conversions and UTF-8 validation.

use super::{CharToU32, IsChar, IsUtf8, U32ToChar};
use crate::function::Function;
use crate::subset::Subset;

const impl Function<Subset<IsChar, u32>> for U32ToChar {
    type Output = char;

    #[inline(always)]
    fn call(input: Subset<IsChar, u32>) -> char {
        let code = Subset::into_inner(input);
        // SAFETY: The strong invariant of `Subset<IsChar, u32>` guarantees `code` is a valid Unicode scalar value.
        unsafe { char::from_u32_unchecked(code) }
    }
}

const impl<'a> Function<&'a Subset<IsChar, u32>> for U32ToChar {
    type Output = char;

    #[inline(always)]
    fn call(input: &'a Subset<IsChar, u32>) -> char {
        let code = *Subset::as_inner(input);
        // SAFETY: The strong invariant of `Subset<IsChar, u32>` guarantees `code` is a valid Unicode scalar value.
        unsafe { char::from_u32_unchecked(code) }
    }
}

const impl Function<char> for CharToU32 {
    type Output = Subset<IsChar, u32>;

    #[inline(always)]
    fn call(input: char) -> Subset<IsChar, u32> {
        // SAFETY: Rust's `char` validity invariant is strictly equivalent to `IsChar`.
        unsafe { Subset::new_unchecked(input as u32) }
    }
}

const impl<'a> Function<&'a char> for CharToU32 {
    type Output = Subset<IsChar, u32>;

    #[inline(always)]
    fn call(input: &'a char) -> Subset<IsChar, u32> {
        // SAFETY: Rust's `char` validity invariant is strictly equivalent to `IsChar`.
        unsafe { Subset::new_unchecked(*input as u32) }
    }
}

const impl<'a> Function<&'a [u8]> for IsUtf8 {
    type Output = bool;

    #[inline(always)]
    fn call(input: &'a [u8]) -> bool {
        core::str::from_utf8(input).is_ok()
    }
}

const impl<'a, const N: usize> Function<&'a [u8; N]> for IsUtf8 {
    type Output = bool;

    #[inline(always)]
    fn call(input: &'a [u8; N]) -> bool {
        core::str::from_utf8(input.as_slice()).is_ok()
    }
}
