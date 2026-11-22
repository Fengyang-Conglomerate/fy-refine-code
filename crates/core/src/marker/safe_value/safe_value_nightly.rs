//! Value safety and recursive base-unwrapping contract supporting optional compile-time (`const`) evaluation.

/// Promises that the logical data exposed by an instance of this type under a shared borrow is
/// strictly invariant, and declares the innermost underlying carrier type [`Base`](Self::Base)
/// obtained by recursively unwrapping nested refinement wrappers.
///
/// `const` implementations support unwrapping both at compile time and at runtime; non-`const`
/// implementations remain usable at runtime.
///
/// # Safety
/// Implementors must guarantee:
///
/// 1. **Predicate Invariance**:
///    If an instance satisfies a safe predicate `P(&x) == true`, then as long as `x` remains alive
///    and undergoes no exclusive write, re-evaluating `P(&x)` at any time, on any thread, and via
///    any shared borrow must always yield `true`, with no "predicate drift" caused by hidden
///    internal state changes.
/// 2. **No Observable Shared Mutation**:
///    The type must not contain primitives that permit mutating domain data through a shared borrow
///    (such as interior mutability via `Cell`, `RefCell`, `Mutex`, etc.).
/// 3. **Exemption for Lifetime Metadata**:
///    Lifetime-management metadata inside smart pointers (such as `Rc`/`Arc` reference counts or
///    heap allocator headers) constitutes low-level runtime bookkeeping and is completely invisible
///    to the domain payload observable through shared borrows. As long as the wrapped payload `T`
///    satisfies `SafeValue`, the smart pointer itself may satisfy `SafeValue`.
/// 4. **Recursive Coverage**:
///    This promise must apply recursively to all fields, pointers, and references held internally.
/// 5. **Unique Base Unwrapping & Transparent Layout Isomorphism**:
///    Except for [`Subset`](crate::subset::Subset) in this crate, every external type implementing
///    `SafeValue` must set `type Base = Self` and return `self` unchanged from `as_base`; external
///    types must never set `Base` to an inner field type (otherwise, bitwise move-out unwrapping in
///    `Subset::into_base` would bypass a custom `Drop` implementation or trigger undefined behavior).
///    `as_base` must be pure, guaranteed to terminate, and panic-free.
pub const unsafe trait SafeValue {
    /// The innermost underlying carrier type obtained by recursively stripping all nested refinement wrappers.
    type Base: ?Sized + SafeValue;

    /// Recursively unwraps to a shared reference to the innermost underlying value.
    fn as_base(&self) -> &Self::Base;
}
