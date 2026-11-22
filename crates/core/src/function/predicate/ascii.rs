//! Transparent combinator aliases for ASCII byte classification predicates.
//!
//! Built transparently by combining `ConstU8` with [`Range`] / [`AnyOf`] / `Or` / `LeConst`,
//! naturally preserving extensional subset inclusion (e.g., `IsAsciiLowercase ⊆ IsAsciiAlphabetic ⊆ IsAsciiAlphanumeric ⊆ IsAscii`).
//!
//! ```
//! use fy_refine_core::function::Function;
//! use fy_refine_core::function::predicate::{
//!     IsAscii, IsAsciiAlphabetic, IsAsciiAlphanumeric, IsAsciiDigit, IsAsciiHexDigit,
//!     IsAsciiLowercase, IsAsciiUppercase, IsAsciiWhitespace,
//! };
//!
//! assert!(IsAscii::call(&0x7fu8));
//! assert!(!IsAscii::call(&0x80u8));
//! assert!(IsAsciiLowercase::call(&b'z'));
//! assert!(IsAsciiUppercase::call(&b'A'));
//! assert!(IsAsciiDigit::call(&b'9'));
//! assert!(IsAsciiAlphabetic::call(&b'K'));
//! assert!(IsAsciiAlphanumeric::call(&b'7'));
//! assert!(IsAsciiHexDigit::call(&b'f'));
//! assert!(!IsAsciiHexDigit::call(&b'g'));
//! assert!(IsAsciiWhitespace::call(&b' '));
//! assert!(IsAsciiWhitespace::call(&b'\n'));
//! ```

use crate::function::constant::ConstU8;

use super::{AnyOf, Closed, EqConst, LeConst, Or, Range};

/// Checks whether a byte lies within the 7-bit ASCII range (`0x00..=0x7f`).
pub type IsAscii = LeConst<ConstU8<0x7f>>;

/// Checks whether a byte is an ASCII lowercase letter (`b'a'..=b'z'`).
pub type IsAsciiLowercase = Range<Closed<ConstU8<b'a'>>, Closed<ConstU8<b'z'>>>;

/// Checks whether a byte is an ASCII uppercase letter (`b'A'..=b'Z'`).
pub type IsAsciiUppercase = Range<Closed<ConstU8<b'A'>>, Closed<ConstU8<b'Z'>>>;

/// Checks whether a byte is an ASCII decimal digit (`b'0'..=b'9'`).
pub type IsAsciiDigit = Range<Closed<ConstU8<b'0'>>, Closed<ConstU8<b'9'>>>;

/// Checks whether a byte is an ASCII letter (`b'a'..=b'z' || b'A'..=b'Z'`).
pub type IsAsciiAlphabetic = Or<IsAsciiLowercase, IsAsciiUppercase>;

/// Checks whether a byte is an ASCII alphanumeric character.
pub type IsAsciiAlphanumeric = Or<IsAsciiAlphabetic, IsAsciiDigit>;

/// Checks whether a byte is an ASCII hexadecimal digit (`0-9`, `a-f`, `A-F`).
pub type IsAsciiHexDigit = AnyOf<(
    IsAsciiDigit,
    Range<Closed<ConstU8<b'a'>>, Closed<ConstU8<b'f'>>>,
    Range<Closed<ConstU8<b'A'>>, Closed<ConstU8<b'F'>>>,
)>;

/// Checks whether a byte is a standard ASCII whitespace character (` `, `\t`, `\n`, `\x0C`, `\r`).
pub type IsAsciiWhitespace = AnyOf<(
    EqConst<ConstU8<b' '>>,
    EqConst<ConstU8<b'\t'>>,
    EqConst<ConstU8<b'\n'>>,
    EqConst<ConstU8<0x0C>>,
    EqConst<ConstU8<b'\r'>>,
)>;
