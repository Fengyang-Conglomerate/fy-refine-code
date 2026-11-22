//! Conditional branching, sum-type pattern elimination, proof-carrying refinement branching, and custom enum variant combinators.
//!
//! Provides:
//! 1. **Pure branch-elimination and proof-partitioning primitives**:
//!    [`BranchOpt`], [`BranchRes`], [`BranchOptPure`], [`BranchResPure`], [`IfElse`], [`Classify`];
//! 2. **Sum-type borrow, conversion, and zipping bridges**: [`OptAsRef`], [`ResAsRef`], [`ResOk`], [`ResErr`], [`ZipOpt`], [`ZipRes`];
//! 3. **Sum-type discriminant predicates and refined branchless unwrappers**: [`IsSome`], [`IsNone`], [`IsOk`], [`IsErr`],
//!    [`UnwrapSome`], [`UnwrapOk`], [`UnwrapErr`], [`UnwrapVariant`];
//! 4. **Transparent combinator aliases & zero-`unsafe` declarative macro**: [`MatchOpt`], [`MatchRes`], [`MatchOptPure`], [`MatchResPure`],
//!    [`RefineIf`], [`AndThen`], [`OrElse`],
//!    convenience predicates such as [`SomeAnd`] / [`NoneOr`] / [`OkAnd`] / [`ErrOr`] / [`SomeElseFalse`],
//!    and custom-enum combinators [`IsVariant`], [`IfLetElse`], [`IfLetElseFalse`], [`IfLetElseTrue`],
//!    together with [`variant_projector!`](crate::variant_projector).
//!
//! ```
//! use fy_refine_core::function::Function;
//! use fy_refine_core::function::branch::{BranchOpt, BranchOptPure, MatchOpt, OptAsRef};
//! use fy_refine_core::function::predicate::{False, IsPositive};
//! use fy_refine_core::function::wiring::{AttachUnit, Then};
//!
//! // Zero-copy validation of &Option<i32> via OptAsRef and BranchOptPure
//! type PositiveOpt = Then<OptAsRef, BranchOptPure<IsPositive, False>>;
//! assert!(PositiveOpt::call(&Some(42)));
//! assert!(!PositiveOpt::call(&Some(-1)));
//! assert!(!PositiveOpt::call(&None::<i32>));
//!
//! // Eliminating Option via AttachUnit and MatchOpt
//! struct ReadVal;
//! impl Function<(u32, ())> for ReadVal {
//!     type Output = u32;
//!     fn call((val, ()): (u32, ())) -> u32 { val }
//! }
//! unsafe impl fy_refine_core::function::SafeFunction<(u32, ())> for ReadVal {}
//!
//! struct DefaultVal;
//! impl Function<()> for DefaultVal {
//!     type Output = u32;
//!     fn call(_: ()) -> u32 { 0 }
//! }
//! unsafe impl fy_refine_core::function::SafeFunction<()> for DefaultVal {}
//!
//! type SafeUnwrap = MatchOpt<AttachUnit, ReadVal, DefaultVal>;
//! assert_eq!(SafeUnwrap::call(Some(42)), 42);
//! assert_eq!(SafeUnwrap::call(None), 0);
//!
//! // Pure branch elimination directly on (Option<T>, Context)
//! type DirectBranch = BranchOpt<ReadVal, DefaultVal>;
//! assert_eq!(DirectBranch::call((Some(100), ())), 100);
//! assert_eq!(DirectBranch::call((None, ())), 0);
//! ```

use crate::function::SafeFunction;
use crate::function::predicate::{False, Predicate, True};
use crate::function::wiring::{Always, Fst, PipeWith, Then};
use crate::marker::{Phantom, SafeDrop, SafeValue};
use crate::subset::Subset;

#[cfg(feature = "nightly")]
mod branch_nightly;
#[cfg(not(feature = "nightly"))]
mod branch_stable;

// ==========================================
// 1. Pure branch-elimination primitives
// ==========================================

/// Pure branch elimination for `(Option<T>, Context)`:
/// - If `Some(value)`, evaluates `SomeBranch((value, context))`;
/// - If `None`, evaluates `NoneBranch(context)`.
pub struct BranchOpt<SomeBranch, NoneBranch>(Phantom<SomeBranch>, Phantom<NoneBranch>);

/// Pure branch elimination for `(Result<T, E>, Context)`:
/// - If `Ok(value)`, evaluates `OkBranch((value, context))`;
/// - If `Err(err)`, evaluates `ErrBranch((err, context))`.
pub struct BranchRes<OkBranch, ErrBranch>(Phantom<OkBranch>, Phantom<ErrBranch>);

/// Pure branch elimination for a standalone `Option<T>` (without context):
/// - If `Some(value)`, evaluates `SomeBranch(value)`;
/// - If `None`, evaluates `NoneBranch(())`.
pub struct BranchOptPure<SomeBranch, NoneBranch>(Phantom<SomeBranch>, Phantom<NoneBranch>);

/// Pure branch elimination for a standalone `Result<T, E>` (without context):
/// - If `Ok(value)`, evaluates `OkBranch(value)`;
/// - If `Err(err)`, evaluates `ErrBranch(err)`.
pub struct BranchResPure<OkBranch, ErrBranch>(Phantom<OkBranch>, Phantom<ErrBranch>);

/// Short-circuiting boolean conditional branch:
/// - First evaluates `Cond(input)` on `Input: Copy`;
/// - If `true`, evaluates only `ThenBranch(input)`;
/// - If `false`, evaluates only `ElseBranch(input)`.
pub struct IfElse<Cond, ThenBranch, ElseBranch>(
    Phantom<Cond>,
    Phantom<ThenBranch>,
    Phantom<ElseBranch>,
);

/// Proof-partitioning refiner: evaluates predicate `P` on `&'a T` (supporting `T: ?Sized`),
/// zero-copy returning `Ok(&'a Subset<P, T>)` when `true` and `Err(&'a Subset<Not<P>, T>)` when `false`.
pub struct Classify<P>(Phantom<P>);

// ==========================================
// 2. Sum-type borrow, conversion, zipping, and discriminant primitives
// ==========================================

/// Zero-copy projection from `&Option<T>` to `Option<&T>`.
pub struct OptAsRef;

/// Zero-copy projection from `&Result<T, E>` to `Result<&T, &E>`.
pub struct ResAsRef;

/// Converts `Result<T, E>` into `Option<T>` (dropping the `Err` payload; requires `E: SafeDrop`),
/// or zero-copy projects `&Result<T, E>` into `Option<&T>`.
pub struct ResOk;

/// Converts `Result<T, E>` into `Option<E>` (dropping the `Ok` payload; requires `T: SafeDrop`),
/// or zero-copy projects `&Result<T, E>` into `Option<&E>`.
pub struct ResErr;

/// Zips a pair of options `(Option<A>, Option<B>)` into `Option<(A, B)>`;
/// if only one side is `Some`, its payload is dropped, so both payload types must satisfy [`SafeDrop`].
pub struct ZipOpt;

/// Zips a pair of results `(Result<A, E>, Result<B, E>)` into `Result<(A, B), E>` (left-biased error semantics):
/// - `(Ok(a), Ok(b)) => Ok((a, b))`;
/// - `(Err(e), _) => Err(e)` (drops the right side, requiring `B, E: SafeDrop`);
/// - `(Ok(_), Err(e)) => Err(e)` (drops the left side, requiring `A: SafeDrop`).
pub struct ZipRes;

/// Predicate testing whether an `Option<T>` or `&Option<T>` is `Some`.
pub struct IsSome;

/// Predicate testing whether an `Option<T>` or `&Option<T>` is `None`.
pub struct IsNone;

/// Predicate testing whether a `Result<T, E>` or `&Result<T, E>` is `Ok`.
pub struct IsOk;

/// Predicate testing whether a `Result<T, E>` or `&Result<T, E>` is `Err`.
pub struct IsErr;

/// Infallible branchless unwrap: extracts `T` or `&T` directly from `Subset<IsSome, Option<T>>` or its shared reference.
pub struct UnwrapSome;

/// Infallible branchless unwrap: extracts `T` or `&T` directly from `Subset<IsOk, Result<T, E>>` or its shared reference.
pub struct UnwrapOk;

/// Infallible branchless unwrap: extracts `E` or `&E` directly from `Subset<IsErr, Result<T, E>>` or its shared reference.
pub struct UnwrapErr;

/// Infallible variant extractor: branchlessly extracts the target variant payload from `&Subset<IsVariant<V>, T>` without `Option` wrapping.
///
/// By providing a projection `V` (`SafeFunction<&T>`) returning `Option<Payload>` for a custom enum variant,
/// this operator pairs with [`IsVariant<V>`] to enable safe branchless extraction.
/// Implemented strictly for `&Subset<IsVariant<V>, T>` to preserve soundness, ensuring extraction invokes the exact same `&T` projection as the refinement predicate.
pub struct UnwrapVariant<V>(Phantom<V>);

// ==========================================
// 3. Transparent combinator aliases (zero redundant primitives)
// ==========================================

/// Evaluates `Target` to produce a standalone `Option<T>`, then dispatches to `BranchOptPure<SomeBranch, NoneBranch>`.
pub type MatchOptPure<Target, SomeBranch, NoneBranch> =
    Then<Target, BranchOptPure<SomeBranch, NoneBranch>>;

/// Evaluates `Target` to produce a standalone `Result<T, E>`, then dispatches to `BranchResPure<OkBranch, ErrBranch>`.
pub type MatchResPure<Target, OkBranch, ErrBranch> =
    Then<Target, BranchResPure<OkBranch, ErrBranch>>;

/// Evaluates a standalone `Option<T>`: delegates `Some(v)` to `SomeBranch(v)`, or returns `false` on `None`.
pub type SomeAnd<SomeBranch> = BranchOptPure<SomeBranch, False>;

/// Evaluates a standalone `Option<T>`: delegates `Some(v)` to `SomeBranch(v)`, or returns `true` on `None`.
pub type NoneOr<SomeBranch> = BranchOptPure<SomeBranch, True>;

/// Evaluates a standalone `Result<T, E>`: delegates `Ok(v)` to `OkBranch(v)`, or returns `false` on `Err` (requires `E: SafeDrop`).
pub type OkAnd<OkBranch> = BranchResPure<OkBranch, Always<False>>;

/// Evaluates a standalone `Result<T, E>`: delegates `Ok(v)` to `OkBranch(v)`, or returns `true` on `Err` (requires `E: SafeDrop`).
pub type ErrOr<OkBranch> = BranchResPure<OkBranch, Always<True>>;

/// Evaluates a standalone `Result<T, E>`: delegates `Err(e)` to `ErrBranch(e)`, or returns `false` on `Ok` (requires `T: SafeDrop`).
pub type ErrAnd<ErrBranch> = BranchResPure<Always<False>, ErrBranch>;

/// Evaluates a standalone `Result<T, E>`: delegates `Err(e)` to `ErrBranch(e)`, or returns `true` on `Ok` (requires `T: SafeDrop`).
pub type OkOr<ErrBranch> = BranchResPure<Always<True>, ErrBranch>;

/// Evaluates `Target` to produce `(Option<T>, Context)`, then dispatches to `BranchOpt<SomeBranch, NoneBranch>`.
pub type MatchOpt<Target, SomeBranch, NoneBranch> = Then<Target, BranchOpt<SomeBranch, NoneBranch>>;

/// Evaluates `Target` to produce `(Result<T, E>, Context)`, then dispatches to `BranchRes<OkBranch, ErrBranch>`.
pub type MatchRes<Target, OkBranch, ErrBranch> = Then<Target, BranchRes<OkBranch, ErrBranch>>;

/// Delegates `Some` to `SomeBranch`, otherwise returns `false`.
pub type SomeElseFalse<Target, SomeBranch> = MatchOpt<Target, SomeBranch, Always<False>>;

/// Delegates `Some` to `SomeBranch`, otherwise returns `true`.
pub type SomeElseTrue<Target, SomeBranch> = MatchOpt<Target, SomeBranch, Always<True>>;

/// Delegates `None` to `NoneBranch`, otherwise returns `false`.
pub type NoneElseFalse<Target, NoneBranch> = MatchOpt<Target, Always<False>, NoneBranch>;

/// Delegates `None` to `NoneBranch`, otherwise returns `true`.
pub type NoneElseTrue<Target, NoneBranch> = MatchOpt<Target, Always<True>, NoneBranch>;

/// Delegates `Ok` to `OkBranch`, otherwise returns `false`.
pub type OkElseFalse<Target, OkBranch> = MatchRes<Target, OkBranch, Always<False>>;

/// Delegates `Ok` to `OkBranch`, otherwise returns `true`.
pub type OkElseTrue<Target, OkBranch> = MatchRes<Target, OkBranch, Always<True>>;

/// Delegates `Err` to `ErrBranch`, otherwise returns `false`.
pub type ErrElseFalse<Target, ErrBranch> = MatchRes<Target, Always<False>, ErrBranch>;

/// Delegates `Err` to `ErrBranch`, otherwise returns `true`.
pub type ErrElseTrue<Target, ErrBranch> = MatchRes<Target, Always<True>, ErrBranch>;

/// Proof-carrying conditional branch: evaluates `P` on `&'a T`,
/// passing `&'a Subset<P, T>` to `ThenBranch` on `true` and `&'a Subset<Not<P>, T>` to `ElseBranch` on `false`.
pub type RefineIf<P, ThenBranch, ElseBranch> =
    Then<Classify<P>, BranchResPure<ThenBranch, ElseBranch>>;

/// Proof-carrying dependent conjunction: verifies `P(&T)`, then evaluates `Q` on `&Subset<P, T>`, returning `false` if `P` fails.
pub type AndThen<P, Q> = RefineIf<P, Q, Always<False>>;

/// Proof-carrying dependent disjunction: verifies `P(&T)`, returning `true` immediately if it holds, or evaluates `Q` on `&Subset<Not<P>, T>` otherwise.
pub type OrElse<P, Q> = RefineIf<P, Always<True>, Q>;

/// Custom enum variant predicate: returns `true` if variant projection `V` yields `Some(_)`, or `false` otherwise.
pub type IsVariant<V> = Then<V, IsSome>;

/// Custom enum `if let` conditional branch (supports chained `else if let`):
/// - Applies variant projection `V(I) -> Option<Payload>` on `I: Copy + SafeDrop`;
/// - On `Some(payload)`, passes `payload` to `ThenFn`;
/// - On `None`, passes the original input `I` to `ElseFn` (which may nest another `IfLetElse`).
pub type IfLetElse<V, ThenFn, ElseFn> = PipeWith<V, BranchOpt<Then<Fst, ThenFn>, ElseFn>>;

/// Custom enum variant predicate shorthand: applies `ThenFn` to the payload when variant `V` matches, otherwise returns `false`.
pub type IfLetElseFalse<V, ThenFn> = IfLetElse<V, ThenFn, Always<False>>;

/// Custom enum variant predicate shorthand: applies `ThenFn` to the payload when variant `V` matches, otherwise returns `true`.
pub type IfLetElseTrue<V, ThenFn> = IfLetElse<V, ThenFn, Always<True>>;

// ==========================================
// 4. SafeFunction contract guarantees
// ==========================================

// SAFETY: Pure branch consumes ownership without implicit drops; panic-free and terminating.
unsafe impl<T, C, O, S, N> SafeFunction<(Option<T>, C)> for BranchOpt<S, N>
where
    S: SafeFunction<(T, C), Output = O>,
    N: SafeFunction<C, Output = O>,
{
}

// SAFETY: Pure branch consumes ownership without implicit drops; panic-free and terminating.
unsafe impl<T, E, C, O, S, N> SafeFunction<(Result<T, E>, C)> for BranchRes<S, N>
where
    S: SafeFunction<(T, C), Output = O>,
    N: SafeFunction<(E, C), Output = O>,
{
}

// SAFETY: Pure branch consumes ownership without implicit drops; panic-free and terminating.
unsafe impl<T, O, S, N> SafeFunction<Option<T>> for BranchOptPure<S, N>
where
    S: SafeFunction<T, Output = O>,
    N: SafeFunction<(), Output = O>,
{
}

// SAFETY: Pure branch consumes ownership without implicit drops; panic-free and terminating.
unsafe impl<T, E, O, S, N> SafeFunction<Result<T, E>> for BranchResPure<S, N>
where
    S: SafeFunction<T, Output = O>,
    N: SafeFunction<E, Output = O>,
{
}

// SAFETY: Condition and both branches are safe functions; with Copy input, short-circuit selection is terminating and free of destructive side effects.
unsafe impl<I: Copy, O, C, T, E> SafeFunction<I> for IfElse<C, T, E>
where
    C: SafeFunction<I, Output = bool>,
    T: SafeFunction<I, Output = O>,
    E: SafeFunction<I, Output = O>,
{
}

// SAFETY: `P` is a safe predicate (pure, deterministic, terminating, panic-free); the `#[repr(transparent)]` reference cast adds zero overhead or side effects.
unsafe impl<'a, P, T: ?Sized> SafeFunction<&'a T> for Classify<P>
where
    P: for<'b> Predicate<&'b T> + 'a,
    T: SafeValue,
{
}

// SAFETY: Only borrows the inner enum payload; side-effect-free, panic-free, and terminating.
unsafe impl<T> SafeFunction<&Option<T>> for OptAsRef {}

// SAFETY: Only borrows the inner enum payload; side-effect-free, panic-free, and terminating.
unsafe impl<T, E> SafeFunction<&Result<T, E>> for ResAsRef {}

// SAFETY: Owned form drops E on Err (governed by SafeDrop); borrowed form is zero-copy with no drops.
unsafe impl<T, E: SafeDrop> SafeFunction<Result<T, E>> for ResOk {}
unsafe impl<T, E> SafeFunction<&Result<T, E>> for ResOk {}

// SAFETY: Owned form drops T on Ok (governed by SafeDrop); borrowed form is zero-copy with no drops.
unsafe impl<T: SafeDrop, E> SafeFunction<Result<T, E>> for ResErr {}
unsafe impl<T, E> SafeFunction<&Result<T, E>> for ResErr {}

// SAFETY: Both payloads satisfy SafeDrop, so dropping the remaining value when one side is None is safe and terminating.
unsafe impl<A: SafeDrop, B: SafeDrop> SafeFunction<(Option<A>, Option<B>)> for ZipOpt {}

// SAFETY: All potentially dropped payloads satisfy SafeDrop; left-biased error reduction is safe and terminating.
unsafe impl<A: SafeDrop, B: SafeDrop, E: SafeDrop> SafeFunction<(Result<A, E>, Result<B, E>)>
    for ZipRes
{
}

// SAFETY: Inspects enum discriminant; owned forms are bounded by SafeDrop, and borrowed forms perform no drops.
unsafe impl<T: SafeDrop> SafeFunction<Option<T>> for IsSome {}
unsafe impl<T> SafeFunction<&Option<T>> for IsSome {}
unsafe impl<T: SafeDrop> SafeFunction<Option<T>> for IsNone {}
unsafe impl<T> SafeFunction<&Option<T>> for IsNone {}

unsafe impl<T: SafeDrop, E: SafeDrop> SafeFunction<Result<T, E>> for IsOk {}
unsafe impl<T, E> SafeFunction<&Result<T, E>> for IsOk {}
unsafe impl<T: SafeDrop, E: SafeDrop> SafeFunction<Result<T, E>> for IsErr {}
unsafe impl<T, E> SafeFunction<&Result<T, E>> for IsErr {}

// SAFETY: The Subset<IsSome/IsOk/IsErr, _> invariant guarantees the target variant holds; branchless extraction is safe and terminating.
unsafe impl<T: SafeValue> SafeFunction<Subset<IsSome, Option<T>>> for UnwrapSome {}
unsafe impl<T: SafeValue> SafeFunction<&Subset<IsSome, Option<T>>> for UnwrapSome {}

unsafe impl<T: SafeValue, E: SafeValue> SafeFunction<Subset<IsOk, Result<T, E>>> for UnwrapOk {}
unsafe impl<T: SafeValue, E: SafeValue> SafeFunction<&Subset<IsOk, Result<T, E>>> for UnwrapOk {}

unsafe impl<T: SafeValue, E: SafeValue> SafeFunction<Subset<IsErr, Result<T, E>>> for UnwrapErr {}
unsafe impl<T: SafeValue, E: SafeValue> SafeFunction<&Subset<IsErr, Result<T, E>>> for UnwrapErr {}

// SAFETY: The Subset<IsVariant<V>, T> invariant guarantees V(&T) returns Some, and V is a pure safe function.
unsafe impl<'a, T, V, Payload> SafeFunction<&'a Subset<IsVariant<V>, T>> for UnwrapVariant<V>
where
    T: ?Sized + SafeValue,
    IsVariant<V>: for<'b> Predicate<&'b T>,
    V: SafeFunction<&'a T, Output = Option<Payload>>,
{
}

/// Generates a custom enum variant projection function verified by compile-time pure pattern matching and exact type checking, avoiding manual `unsafe impl SafeFunction`.
///
/// Internally uses only `match *input` for side-effect-free pattern destructuring and static exact-type verification,
/// never triggering `Deref` or any custom method calls; works directly with [`IsVariant`], [`IfLetElse`], and [`UnwrapVariant`].
///
/// Supported variant forms:
/// - Named fields by reference: `variant_projector!(pub AsClickRef: &Event => Click { x: &i32, y: &i32 });`
/// - Named fields by `Copy` value: `variant_projector!(pub AsClick: &Event => Click { x: i32, y: i32 });`
/// - Single-payload tuple variant by reference or `Copy` value: `variant_projector!(pub AsText: &Event => Text(&String));`, `variant_projector!(pub AsCode: &Event => Code(u32));`
/// - Multi-payload tuple variant (with field binding names): `variant_projector!(pub AsPairRef: &Event => Pair(a: &i32, b: &i32));`, `variant_projector!(pub AsPair: &Event => Pair(a: i32, b: i32));`
/// - Unit variant without payload: `variant_projector!(pub AsQuit: &Event => Quit);` (outputs `Option<()>`)
/// - Under `nightly` + `#![feature(const_trait_impl)]`, prefix `const` to emit a `const impl`:
///   `variant_projector!(pub const AsQuit: &Event => Quit);`
#[macro_export]
macro_rules! variant_projector {
    (@impl_named [$($mode:tt)*] $(#[$meta:meta])* $vis:vis $name:ident : &$enum_ty:ty => $variant:ident { $($field:ident : &$field_ty:ty),+ $(,)? }) => {
        $(#[$meta])*
        $vis struct $name;

        #[allow(unused_parens)]
        $($mode)* impl<'a> $crate::function::Function<&'a $enum_ty> for $name {
            type Output = Option<($(&'a $field_ty),+)>;

            #[inline(always)]
            fn call(input: &'a $enum_ty) -> Self::Output {
                trait _ExactType<T: ?Sized> {}
                impl<T: ?Sized> _ExactType<T> for T {}
                const fn _check<'r, U: ?Sized + _ExactType<V>, V: ?Sized>(_: &'r U) {}
                type _Enum = $enum_ty;

                match *input {
                    _Enum::$variant { $($field: ref $field),+ } => {
                        $(_check::<_, $field_ty>($field);)+
                        Some(($($field),+))
                    }
                    _ => None,
                }
            }
        }

        // SAFETY: Borrows variant fields by offset via pure pattern destructuring without triggering Deref or any method calls; pure, terminating, and panic-free.
        unsafe impl<'a> $crate::function::SafeFunction<&'a $enum_ty> for $name {}
    };
    (@impl_named_copy [$($mode:tt)*] $(#[$meta:meta])* $vis:vis $name:ident : &$enum_ty:ty => $variant:ident { $($field:ident : $field_ty:ty),+ $(,)? }) => {
        $(#[$meta])*
        $vis struct $name;

        #[allow(unused_parens)]
        $($mode)* impl<'a> $crate::function::Function<&'a $enum_ty> for $name {
            type Output = Option<($($field_ty),+)>;

            #[inline(always)]
            fn call(input: &'a $enum_ty) -> Self::Output {
                trait _ExactType<T: ?Sized> {}
                impl<T: ?Sized> _ExactType<T> for T {}
                const fn _check<'r, U: ?Sized + _ExactType<V>, V: ?Sized>(_: &'r U) {}
                type _Enum = $enum_ty;

                match *input {
                    _Enum::$variant { $($field: ref $field),+ } => {
                        $(_check::<_, $field_ty>($field);)+
                        Some(($(*$field),+))
                    }
                    _ => None,
                }
            }
        }

        // SAFETY: Copies Copy variant fields by offset via pure pattern destructuring without triggering Deref or any method calls; pure, terminating, and panic-free.
        unsafe impl<'a> $crate::function::SafeFunction<&'a $enum_ty> for $name {}
    };
    (@impl_tuple_named [$($mode:tt)*] $(#[$meta:meta])* $vis:vis $name:ident : &$enum_ty:ty => $variant:ident ( $($field:ident : &$field_ty:ty),+ $(,)? )) => {
        $(#[$meta])*
        $vis struct $name;

        #[allow(unused_parens)]
        $($mode)* impl<'a> $crate::function::Function<&'a $enum_ty> for $name {
            type Output = Option<($(&'a $field_ty),+)>;

            #[inline(always)]
            fn call(input: &'a $enum_ty) -> Self::Output {
                trait _ExactType<T: ?Sized> {}
                impl<T: ?Sized> _ExactType<T> for T {}
                const fn _check<'r, U: ?Sized + _ExactType<V>, V: ?Sized>(_: &'r U) {}
                type _Enum = $enum_ty;

                match *input {
                    _Enum::$variant($(ref $field),+) => {
                        $(_check::<_, $field_ty>($field);)+
                        Some(($($field),+))
                    }
                    _ => None,
                }
            }
        }

        // SAFETY: Borrows variant payloads by offset via pure pattern destructuring without triggering Deref or any method calls; pure, terminating, and panic-free.
        unsafe impl<'a> $crate::function::SafeFunction<&'a $enum_ty> for $name {}
    };
    (@impl_tuple_named_copy [$($mode:tt)*] $(#[$meta:meta])* $vis:vis $name:ident : &$enum_ty:ty => $variant:ident ( $($field:ident : $field_ty:ty),+ $(,)? )) => {
        $(#[$meta])*
        $vis struct $name;

        #[allow(unused_parens)]
        $($mode)* impl<'a> $crate::function::Function<&'a $enum_ty> for $name {
            type Output = Option<($($field_ty),+)>;

            #[inline(always)]
            fn call(input: &'a $enum_ty) -> Self::Output {
                trait _ExactType<T: ?Sized> {}
                impl<T: ?Sized> _ExactType<T> for T {}
                const fn _check<'r, U: ?Sized + _ExactType<V>, V: ?Sized>(_: &'r U) {}
                type _Enum = $enum_ty;

                match *input {
                    _Enum::$variant($(ref $field),+) => {
                        $(_check::<_, $field_ty>($field);)+
                        Some(($(*$field),+))
                    }
                    _ => None,
                }
            }
        }

        // SAFETY: Copies Copy variant payloads by offset via pure pattern destructuring without triggering Deref or any method calls; pure, terminating, and panic-free.
        unsafe impl<'a> $crate::function::SafeFunction<&'a $enum_ty> for $name {}
    };
    (@impl_tuple_single_lt [$($mode:tt)*] $(#[$meta:meta])* $vis:vis $name:ident : &$enum_ty:ty => $variant:ident ( &$lt:lifetime $field_ty:ty )) => {
        $(#[$meta])*
        $vis struct $name;

        $($mode)* impl<'a> $crate::function::Function<&'a $enum_ty> for $name {
            type Output = Option<&$lt $field_ty>;

            #[inline(always)]
            fn call(input: &'a $enum_ty) -> Self::Output {
                trait _ExactType<T: ?Sized> {}
                impl<T: ?Sized> _ExactType<T> for T {}
                const fn _check<'r, U: ?Sized + _ExactType<V>, V: ?Sized>(_: &'r U) {}
                type _Enum = $enum_ty;

                match *input {
                    _Enum::$variant(ref payload) => {
                        _check::<_, &$lt $field_ty>(payload);
                        Some(*payload)
                    }
                    _ => None,
                }
            }
        }

        // SAFETY: Copies a Copy reference payload by offset via pure pattern destructuring without triggering Deref or any method calls; pure, terminating, and panic-free.
        unsafe impl<'a> $crate::function::SafeFunction<&'a $enum_ty> for $name {}
    };
    (@impl_tuple_single_ref [$($mode:tt)*] $(#[$meta:meta])* $vis:vis $name:ident : &$enum_ty:ty => $variant:ident ( &$field_ty:ty )) => {
        $(#[$meta])*
        $vis struct $name;

        $($mode)* impl<'a> $crate::function::Function<&'a $enum_ty> for $name {
            type Output = Option<&'a $field_ty>;

            #[inline(always)]
            fn call(input: &'a $enum_ty) -> Self::Output {
                trait _ExactType<T: ?Sized> {}
                impl<T: ?Sized> _ExactType<T> for T {}
                const fn _check<'r, U: ?Sized + _ExactType<V>, V: ?Sized>(_: &'r U) {}
                type _Enum = $enum_ty;

                match *input {
                    _Enum::$variant(ref payload) => {
                        _check::<_, $field_ty>(payload);
                        Some(payload)
                    }
                    _ => None,
                }
            }
        }

        // SAFETY: Borrows the variant payload by offset via pure pattern destructuring without triggering Deref or any method calls; pure, terminating, and panic-free.
        unsafe impl<'a> $crate::function::SafeFunction<&'a $enum_ty> for $name {}
    };
    (@impl_tuple_single_copy [$($mode:tt)*] $(#[$meta:meta])* $vis:vis $name:ident : &$enum_ty:ty => $variant:ident ( $field_ty:ty )) => {
        $(#[$meta])*
        $vis struct $name;

        $($mode)* impl<'a> $crate::function::Function<&'a $enum_ty> for $name {
            type Output = Option<$field_ty>;

            #[inline(always)]
            fn call(input: &'a $enum_ty) -> Self::Output {
                trait _ExactType<T: ?Sized> {}
                impl<T: ?Sized> _ExactType<T> for T {}
                const fn _check<'r, U: ?Sized + _ExactType<V>, V: ?Sized>(_: &'r U) {}
                type _Enum = $enum_ty;

                match *input {
                    _Enum::$variant(ref payload) => {
                        _check::<_, $field_ty>(payload);
                        Some(*payload)
                    }
                    _ => None,
                }
            }
        }

        // SAFETY: Copies a Copy variant payload by offset via pure pattern destructuring without triggering Deref or any method calls; pure, terminating, and panic-free.
        unsafe impl<'a> $crate::function::SafeFunction<&'a $enum_ty> for $name {}
    };
    (@impl_unit [$($mode:tt)*] $(#[$meta:meta])* $vis:vis $name:ident : &$enum_ty:ty => $variant:ident) => {
        $(#[$meta])*
        $vis struct $name;

        $($mode)* impl<'a> $crate::function::Function<&'a $enum_ty> for $name {
            type Output = Option<()>;

            #[inline(always)]
            fn call(input: &'a $enum_ty) -> Self::Output {
                type _Enum = $enum_ty;

                match *input {
                    _Enum::$variant => Some(()),
                    _ => None,
                }
            }
        }

        // SAFETY: Inspects unit variant discriminant via pure pattern matching without dereference hijacking or method calls; pure, terminating, and panic-free.
        unsafe impl<'a> $crate::function::SafeFunction<&'a $enum_ty> for $name {}
    };

    ($(#[$meta:meta])* $vis:vis const $name:ident : &$enum_ty:ty => $variant:ident { $($field:ident : &$field_ty:ty),+ $(,)? } $(;)?) => {
        $crate::variant_projector!(@impl_named [const] $(#[$meta])* $vis $name : &$enum_ty => $variant { $($field : &$field_ty),+ });
    };
    ($(#[$meta:meta])* $vis:vis $name:ident : &$enum_ty:ty => $variant:ident { $($field:ident : &$field_ty:ty),+ $(,)? } $(;)?) => {
        $crate::variant_projector!(@impl_named [] $(#[$meta])* $vis $name : &$enum_ty => $variant { $($field : &$field_ty),+ });
    };
    ($(#[$meta:meta])* $vis:vis const $name:ident : &$enum_ty:ty => $variant:ident { $($field:ident : $field_ty:ty),+ $(,)? } $(;)?) => {
        $crate::variant_projector!(@impl_named_copy [const] $(#[$meta])* $vis $name : &$enum_ty => $variant { $($field : $field_ty),+ });
    };
    ($(#[$meta:meta])* $vis:vis $name:ident : &$enum_ty:ty => $variant:ident { $($field:ident : $field_ty:ty),+ $(,)? } $(;)?) => {
        $crate::variant_projector!(@impl_named_copy [] $(#[$meta])* $vis $name : &$enum_ty => $variant { $($field : $field_ty),+ });
    };
    ($(#[$meta:meta])* $vis:vis const $name:ident : &$enum_ty:ty => $variant:ident ( $($field:ident : &$field_ty:ty),+ $(,)? ) $(;)?) => {
        $crate::variant_projector!(@impl_tuple_named [const] $(#[$meta])* $vis $name : &$enum_ty => $variant ( $($field : &$field_ty),+ ));
    };
    ($(#[$meta:meta])* $vis:vis $name:ident : &$enum_ty:ty => $variant:ident ( $($field:ident : &$field_ty:ty),+ $(,)? ) $(;)?) => {
        $crate::variant_projector!(@impl_tuple_named [] $(#[$meta])* $vis $name : &$enum_ty => $variant ( $($field : &$field_ty),+ ));
    };
    ($(#[$meta:meta])* $vis:vis const $name:ident : &$enum_ty:ty => $variant:ident ( $($field:ident : $field_ty:ty),+ $(,)? ) $(;)?) => {
        $crate::variant_projector!(@impl_tuple_named_copy [const] $(#[$meta])* $vis $name : &$enum_ty => $variant ( $($field : $field_ty),+ ));
    };
    ($(#[$meta:meta])* $vis:vis $name:ident : &$enum_ty:ty => $variant:ident ( $($field:ident : $field_ty:ty),+ $(,)? ) $(;)?) => {
        $crate::variant_projector!(@impl_tuple_named_copy [] $(#[$meta])* $vis $name : &$enum_ty => $variant ( $($field : $field_ty),+ ));
    };
    ($(#[$meta:meta])* $vis:vis const $name:ident : &$enum_ty:ty => $variant:ident ( &$lt:lifetime $field_ty:ty ) $(;)?) => {
        $crate::variant_projector!(@impl_tuple_single_lt [const] $(#[$meta])* $vis $name : &$enum_ty => $variant ( &$lt $field_ty ));
    };
    ($(#[$meta:meta])* $vis:vis $name:ident : &$enum_ty:ty => $variant:ident ( &$lt:lifetime $field_ty:ty ) $(;)?) => {
        $crate::variant_projector!(@impl_tuple_single_lt [] $(#[$meta])* $vis $name : &$enum_ty => $variant ( &$lt $field_ty ));
    };
    ($(#[$meta:meta])* $vis:vis const $name:ident : &$enum_ty:ty => $variant:ident ( &$field_ty:ty ) $(;)?) => {
        $crate::variant_projector!(@impl_tuple_single_ref [const] $(#[$meta])* $vis $name : &$enum_ty => $variant ( &$field_ty ));
    };
    ($(#[$meta:meta])* $vis:vis $name:ident : &$enum_ty:ty => $variant:ident ( &$field_ty:ty ) $(;)?) => {
        $crate::variant_projector!(@impl_tuple_single_ref [] $(#[$meta])* $vis $name : &$enum_ty => $variant ( &$field_ty ));
    };
    ($(#[$meta:meta])* $vis:vis const $name:ident : &$enum_ty:ty => $variant:ident ( $field_ty:ty ) $(;)?) => {
        $crate::variant_projector!(@impl_tuple_single_copy [const] $(#[$meta])* $vis $name : &$enum_ty => $variant ( $field_ty ));
    };
    ($(#[$meta:meta])* $vis:vis $name:ident : &$enum_ty:ty => $variant:ident ( $field_ty:ty ) $(;)?) => {
        $crate::variant_projector!(@impl_tuple_single_copy [] $(#[$meta])* $vis $name : &$enum_ty => $variant ( $field_ty ));
    };
    ($(#[$meta:meta])* $vis:vis const $name:ident : &$enum_ty:ty => $variant:ident $(;)?) => {
        $crate::variant_projector!(@impl_unit [const] $(#[$meta])* $vis $name : &$enum_ty => $variant);
    };
    ($(#[$meta:meta])* $vis:vis $name:ident : &$enum_ty:ty => $variant:ident $(;)?) => {
        $crate::variant_projector!(@impl_unit [] $(#[$meta])* $vis $name : &$enum_ty => $variant);
    };
}
