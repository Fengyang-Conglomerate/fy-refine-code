//! Function protocol with optional compile-time evaluation support.

/// Type-level representation of a mapping from `Input` to `Output`, without automatic evaluation or input conversion.
///
/// `const` implementations can be called both at compile time and at runtime; non-`const` implementations remain callable at runtime.
/// Multiple inputs are represented via tuples; borrows are represented via reference types.
/// This trait alone does not guarantee purity, termination, or panic freedom.
///
/// ```
/// #![feature(const_trait_impl)]
/// use fy_refine_core::function::Function;
///
/// struct Identity;
/// const impl<T> Function<T> for Identity {
///     type Output = T;
///     fn call(input: T) -> T { input }
/// }
/// const VALUE: u8 = Identity::call(7);
/// assert_eq!(VALUE, 7);
///
/// struct Runtime;
/// impl Function<u8> for Runtime {
///     type Output = String;
///     fn call(input: u8) -> String { input.to_string() }
/// }
/// assert_eq!(Runtime::call(7), "7");
/// ```
pub const trait Function<Input> {
    /// The output type of the mapping.
    type Output;

    /// Applies the mapping to the given input; `const` implementations also support compile-time evaluation.
    fn call(input: Input) -> Self::Output;
}
