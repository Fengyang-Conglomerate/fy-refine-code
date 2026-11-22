//! Safe comparison protocols supporting optional compile-time (`const`) evaluation; non-`const` implementations remain usable at runtime.

use core::cmp::Ordering;

/// Performs safe equality comparison between values of `Self` and `Rhs` without depending on standard library comparison traits.
///
/// Compares identical types by default, and may also be implemented across distinct types.
/// `const` implementations can be called both at compile time and at runtime.
///
/// # Safety
/// Both methods must be guaranteed to terminate without panicking, including any inner calls and
/// the destruction of temporary values.
/// Comparisons must obey mathematical equality: homogeneous comparisons must be reflexive, and
/// when reversed or chained comparisons are supported, symmetry and transitivity must hold.
/// The result of `ne` must be the logical complement of `eq`.
/// (When consuming comparisons drop owned inputs, drop safety is governed independently by [`SafeDrop`](crate::marker::SafeDrop).)
pub const unsafe trait SafeEq<Rhs: ?Sized = Self> {
    /// Returns `true` if `self` and `other` are equal.
    fn eq(&self, other: &Rhs) -> bool;

    /// Returns `true` if `self` and `other` are not equal, implemented directly rather than defaulting to `!self.eq(other)`.
    fn ne(&self, other: &Rhs) -> bool;
}

/// Performs safe total order comparison between values of `Self` and `Rhs`.
///
/// A `const` implementation requires a matching `const` [`SafeEq`] implementation; non-`const`
/// implementations do not require `const` support.
///
/// # Safety
/// All methods must be guaranteed to terminate without panicking, including any inner calls and
/// the destruction of temporary values.
/// Every pair of valid inputs must be comparable; all comparison results must satisfy the total
/// order axioms and remain consistent with [`SafeEq`].
/// `cmp` must return `Ordering::Equal` if and only if `eq` returns `true`; `lt`, `le`, `gt`, and
/// `ge` must match the less-than, less-than-or-equal, greater-than, and greater-than-or-equal
/// relations defined by `cmp`, respectively. When reversed or chained comparisons are supported,
/// implementations must also preserve consistency under operand reversal and transitivity.
pub const unsafe trait SafeOrd<Rhs: ?Sized = Self>: [const] SafeEq<Rhs> {
    /// Returns the [`Ordering`] between `self` and `other`.
    fn cmp(&self, other: &Rhs) -> Ordering;

    /// Returns `true` if `self` is strictly less than `other`.
    fn lt(&self, other: &Rhs) -> bool;

    /// Returns `true` if `self` is less than or equal to `other`.
    fn le(&self, other: &Rhs) -> bool;

    /// Returns `true` if `self` is strictly greater than `other`.
    fn gt(&self, other: &Rhs) -> bool;

    /// Returns `true` if `self` is greater than or equal to `other`.
    fn ge(&self, other: &Rhs) -> bool;
}
