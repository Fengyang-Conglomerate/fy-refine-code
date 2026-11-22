//! Stable function protocol.

/// Type-level representation of a mapping from `Input` to `Output`, without automatic evaluation or input conversion.
///
/// Multiple inputs are represented via tuples; borrows are represented via reference types.
/// This trait alone does not guarantee purity, termination, or panic freedom.
///
/// ```
/// use fy_refine_core::function::Function;
///
/// struct Sum;
/// impl Function<(u8, u8)> for Sum {
///     type Output = u16;
///     fn call((left, right): (u8, u8)) -> u16 {
///         u16::from(left) + u16::from(right)
///     }
/// }
/// assert_eq!(Sum::call((1, 2)), 3);
/// ```
pub trait Function<Input> {
    /// The output type of the mapping.
    type Output;

    /// Applies the mapping to the given input.
    fn call(input: Input) -> Self::Output;
}
