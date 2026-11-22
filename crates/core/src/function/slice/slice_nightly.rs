//! Nightly `const`-capable implementations of contiguous slice view projections and proof-carrying string conversions.

use super::{AsBytes, AsSlice, AsStr};
use crate::function::Function;
use crate::function::collection::All;
use crate::function::predicate::{IsAscii, IsUtf8};
use crate::subset::Subset;

const impl<'a> Function<&'a str> for AsBytes {
    type Output = &'a [u8];

    #[inline(always)]
    fn call(input: &'a str) -> &'a [u8] {
        input.as_bytes()
    }
}

const impl<'a, T, const N: usize> Function<&'a [T; N]> for AsSlice {
    type Output = &'a [T];

    #[inline(always)]
    fn call(input: &'a [T; N]) -> &'a [T] {
        input.as_slice()
    }
}

const impl<'a> Function<&'a Subset<All<IsAscii>, [u8]>> for AsStr {
    type Output = &'a str;

    #[inline(always)]
    fn call(input: &'a Subset<All<IsAscii>, [u8]>) -> &'a str {
        let bytes = Subset::as_inner(input);
        // SAFETY: The strong invariant of `Subset<All<IsAscii>, [u8]>` guarantees every byte is 7-bit ASCII (valid UTF-8).
        unsafe { core::str::from_utf8_unchecked(bytes) }
    }
}

const impl<'a, const N: usize> Function<&'a Subset<All<IsAscii>, [u8; N]>> for AsStr {
    type Output = &'a str;

    #[inline(always)]
    fn call(input: &'a Subset<All<IsAscii>, [u8; N]>) -> &'a str {
        let bytes = Subset::as_inner(input).as_slice();
        // SAFETY: The strong invariant of `Subset<All<IsAscii>, [u8; N]>` guarantees every byte is 7-bit ASCII (valid UTF-8).
        unsafe { core::str::from_utf8_unchecked(bytes) }
    }
}

const impl<'a> Function<&'a Subset<IsUtf8, [u8]>> for AsStr {
    type Output = &'a str;

    #[inline(always)]
    fn call(input: &'a Subset<IsUtf8, [u8]>) -> &'a str {
        let bytes = Subset::as_inner(input);
        // SAFETY: The strong invariant of `Subset<IsUtf8, [u8]>` guarantees the byte sequence is valid UTF-8.
        unsafe { core::str::from_utf8_unchecked(bytes) }
    }
}

const impl<'a, const N: usize> Function<&'a Subset<IsUtf8, [u8; N]>> for AsStr {
    type Output = &'a str;

    #[inline(always)]
    fn call(input: &'a Subset<IsUtf8, [u8; N]>) -> &'a str {
        let bytes = Subset::as_inner(input).as_slice();
        // SAFETY: The strong invariant of `Subset<IsUtf8, [u8; N]>` guarantees the byte sequence is valid UTF-8.
        unsafe { core::str::from_utf8_unchecked(bytes) }
    }
}
