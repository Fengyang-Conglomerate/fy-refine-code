//! Predicates are safe functions returning `bool`, without prescribing how the input is borrowed.

use super::SafeFunction;

pub mod ascii;
mod cmp;
mod logic;
pub mod num;
pub mod range;
pub mod utf8;

pub use crate::function::bitwise::{HasAllBits, HasAnyBits, HasNoBits};
pub use crate::function::branch::{
    AndThen, Classify, ErrAnd, ErrOr, IsErr, IsNone, IsOk, IsSome, IsVariant, NoneOr, OkAnd, OkOr,
    OrElse, RefineIf, SomeAnd,
};
pub use crate::function::collection::{All, Any, IsEmpty, IsNonEmpty};
pub use crate::function::lookup::{InBitset, InSliceEq, InSortedRanges, InSortedSlice};
pub use ascii::{
    IsAscii, IsAsciiAlphabetic, IsAsciiAlphanumeric, IsAsciiDigit, IsAsciiHexDigit,
    IsAsciiLowercase, IsAsciiUppercase, IsAsciiWhitespace,
};
pub use cmp::{
    Eq, EqConst, ForkEq, ForkGe, ForkGt, ForkLe, ForkLt, ForkNe, Ge, GeConst, Gt, GtConst, Le,
    LeConst, Lt, LtConst, Ne, NeConst, PairEq, PairGe, PairGt, PairLe, PairLt, PairNe,
};
pub use logic::{AllOf, And, AnyOf, ConjunctionTuple, DisjunctionTuple, False, Not, Or, True, Xor};
pub use num::{
    ForkIsMultipleOf, IsEven, IsFinite, IsInfinite, IsInteger, IsMultipleOf, IsMultipleOfConst,
    IsNaN, IsNegative, IsNonNegative, IsNonPositive, IsNonZero, IsNotNaN, IsOdd, IsPositive,
    IsZero, PairIsMultipleOf,
};
pub use range::{Closed, LowerBound, Open, Range, UpperBound};
pub use utf8::{CharToU32, IsChar, IsUtf8, U32ToChar};

/// A safe function performing a boolean test on `Input`.
///
/// `Input` may be a value, reference, or tuple. Any type implementing `SafeFunction` with a `bool`
/// output automatically implements this trait without requiring a separate safety promise.
///
/// ```
/// use fy_refine_core::function::{Function, SafeFunction, Predicate};
///
/// struct IsZero;
/// impl Function<u8> for IsZero {
///     type Output = bool;
///     fn call(input: u8) -> bool { input == 0 }
/// }
/// // SAFETY: Integer equality is pure, total, terminating, and panic-free.
/// unsafe impl SafeFunction<u8> for IsZero {}
///
/// fn test<P: Predicate<u8>>(input: u8) -> bool { P::call(input) }
/// assert!(test::<IsZero>(0));
/// assert!(!test::<IsZero>(1));
/// ```
///
/// Implementing `Function` with a `bool` output is not sufficient to become a predicate; `SafeFunction` must also be implemented.
pub trait Predicate<Input>: SafeFunction<Input, Output = bool> {}

impl<Input, F: SafeFunction<Input, Output = bool>> Predicate<Input> for F {}
