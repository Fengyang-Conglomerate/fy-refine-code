//! Categorical minimal orthogonal wiring primitives and transparent combinator aliases.
//!
//! Organized in two layers:
//! 1. **Minimal orthogonal primitives**: `Id`, `Then`, `Swap`, `AssocLeft`, `AssocRight`, `MapPair`, `AttachUnit`,
//!    `Fst`, `Snd`, `Dup`, `Discard`, `Always`, `Copied`, `Cloned`, `WithRef`,
//!    and subset penetration primitives `AsInner`, `IntoInner`, `AsBase`, `IntoBase`;
//! 2. **Transparent combinator aliases & zero-`unsafe` declarative macro**: `Fork`, `PipeWith`, `MapFst`, `MapSnd`,
//!    and the struct field projector macro [`field_projector!`](crate::field_projector), with zero redundant primitives.
//!
//! ```
//! use fy_refine_core::function::Function;
//! use fy_refine_core::function::wiring::{AssocLeft, Fork, Id, MapFst, PipeWith, Swap, Then};
//!
//! type Route = Then<AssocLeft, MapFst<Swap>>;
//! assert_eq!(Route::call((1, (2, 3))), ((2, 1), 3));
//!
//! struct Double;
//! impl Function<i32> for Double {
//!     type Output = i32;
//!     fn call(x: i32) -> i32 { x * 2 }
//! }
//! unsafe impl fy_refine_core::function::SafeFunction<i32> for Double {}
//!
//! struct SumPair;
//! impl Function<(i32, i32)> for SumPair {
//!     type Output = i32;
//!     fn call((a, b): (i32, i32)) -> i32 { a + b }
//! }
//! unsafe impl fy_refine_core::function::SafeFunction<(i32, i32)> for SumPair {}
//!
//! // Fork: x -> (Double(x), x)
//! assert_eq!(Fork::<Double, Id>::call(5), (10, 5));
//! // PipeWith: x -> SumPair((Double(x), x)) => 5*2 + 5 = 15
//! assert_eq!(PipeWith::<Double, SumPair>::call(5), 15);
//! ```

use crate::function::{Predicate, SafeFunction};
use crate::marker::{Phantom, SafeClone, SafeDrop, SafeValue};
use crate::subset::Subset;

#[cfg(feature = "nightly")]
mod wiring_nightly;
#[cfg(not(feature = "nightly"))]
mod wiring_stable;

// ==========================================
// 1. Minimal orthogonal wiring primitives
// ==========================================

/// Identity morphism: returns the input unchanged (`T -> T`).
pub struct Id;

/// Sequential composition: runs `F` on the input and passes its result to `G` (`I -> G(F(I))`).
pub struct Then<F, G>(Phantom<F>, Phantom<G>);

/// Pair swap morphism: `(A, B) -> (B, A)`.
pub struct Swap;

/// Left associator morphism: `(A, (B, C)) -> ((A, B), C)`.
pub struct AssocLeft;

/// Right associator morphism: `((A, B), C) -> (A, (B, C))`.
pub struct AssocRight;

/// Bifunctor product map: applies safe functions independently to each pair element `(A, B) -> (F(A), G(B))`.
pub struct MapPair<F, G>(Phantom<F>, Phantom<G>);

/// Right unit attachment (monoidal unitor): `A -> (A, ())`.
pub struct AttachUnit;

/// First projection: extracts the left element of a pair and drops the right element; the right element must satisfy [`SafeDrop`].
pub struct Fst;

/// Second projection: extracts the right element of a pair and drops the left element; the left element must satisfy [`SafeDrop`].
pub struct Snd;

/// Diagonal duplication morphism: bitwise-copies the input into a pair `T -> (T, T)`; the input must implement `Copy`.
pub struct Dup;

/// Discard operator: drops the input and returns `()`; the input must satisfy [`SafeDrop`].
pub struct Discard;

/// Constant function combinator (K combinator): drops any input and returns the result of nullary morphism `C`; the input must satisfy [`SafeDrop`].
pub struct Always<C>(Phantom<C>);

/// Shared-reference bitwise-copy bridge: `&T -> T` (`T: Copy`).
pub struct Copied;

/// Shared-reference safe-clone bridge: `&T -> T` (`T: SafeClone`).
pub struct Cloned;

/// Scoped-borrow evaluation bridge: takes an owned input `T`, borrows `&T` in a local scope for `F` to compute,
/// then safely drops `T` (requiring `T: SafeDrop`) and returns `F`'s output `O`.
pub struct WithRef<F>(Phantom<F>);

/// Single-layer subset shared-reference unwrap: `&Subset<P, T> -> &T` (supports `?Sized`).
pub struct AsInner;

/// Single-layer subset owned unwrap: `Subset<P, T> -> T`.
pub struct IntoInner;

/// Recursive subset shared-reference base penetration: `&Subset<P, T> -> &T::Base` (supports `?Sized`).
pub struct AsBase;

/// Recursive subset owned base penetration: `Subset<P, T> -> T::Base` (requires `T::Base: Sized`).
pub struct IntoBase;

// ==========================================
// 2. Transparent combinator aliases (zero redundant primitives)
// ==========================================

/// Product fan-out morphism: `I -> (F(I), G(I))` (`I: Copy`), equivalent to `Then<Dup, MapPair<F, G>>`.
pub type Fork<F, G> = Then<Dup, MapPair<F, G>>;

/// Dependent pipeline composition: `I -> G((F(I), I))` (`I: Copy`), equivalent to `Then<Fork<F, Id>, G>`.
pub type PipeWith<F, G> = Then<Fork<F, Id>, G>;

/// Maps only the left element of a pair: `(A, B) -> (F(A), B)`, equivalent to `MapPair<F, Id>`.
pub type MapFst<F> = MapPair<F, Id>;

/// Maps only the right element of a pair: `(A, B) -> (A, G(B))`, equivalent to `MapPair<Id, G>`.
pub type MapSnd<G> = MapPair<Id, G>;

// ==========================================
// 3. SafeFunction contract guarantees
// ==========================================

// SAFETY: Transfers ownership directly to the caller without implicit copying or dropping.
unsafe impl<T> SafeFunction<T> for Id {}

// SAFETY: Composes two safe functions, passing the intermediate value directly to the next stage; panic-free, terminating, and free of destructive side effects.
unsafe impl<I, F: SafeFunction<I>, G: SafeFunction<F::Output>> SafeFunction<I> for Then<F, G> {}

// SAFETY: Only moves and swaps the two tuple fields without copying or dropping.
unsafe impl<A, B> SafeFunction<(A, B)> for Swap {}

// SAFETY: Only reassociates tuple nesting without copying or dropping fields.
unsafe impl<A, B, C> SafeFunction<(A, (B, C))> for AssocLeft {}

// SAFETY: Only reassociates tuple nesting without copying or dropping fields.
unsafe impl<A, B, C> SafeFunction<((A, B), C)> for AssocRight {}

// SAFETY: Consumes each field once via its respective safe function; safety of both implies overall safety.
unsafe impl<A, B, F: SafeFunction<A>, G: SafeFunction<B>> SafeFunction<(A, B)> for MapPair<F, G> {}

// SAFETY: Only constructs (A, ()) without copying or dropping fields.
unsafe impl<A> SafeFunction<A> for AttachUnit {}

// SAFETY: Extracts the left element and drops the right; B: SafeDrop guarantees benign, terminating, side-effect-free destruction.
unsafe impl<A, B: SafeDrop> SafeFunction<(A, B)> for Fst {}

// SAFETY: Extracts the right element and drops the left; A: SafeDrop guarantees benign, terminating, side-effect-free destruction.
unsafe impl<A: SafeDrop, B> SafeFunction<(A, B)> for Snd {}

// SAFETY: T implements Copy, so bitwise duplication is pure, panic-free, and terminating.
unsafe impl<T: Copy> SafeFunction<T> for Dup {}

// SAFETY: T: SafeDrop guarantees benign, terminating, side-effect-free destruction.
unsafe impl<T: SafeDrop> SafeFunction<T> for Discard {}

// SAFETY: C is a nullary safe function and dropping Input is governed by SafeDrop, ensuring termination.
unsafe impl<Input: SafeDrop, C: SafeFunction<()>> SafeFunction<Input> for Always<C> {}

// SAFETY: Bitwise-copies the referent; pure, terminating, and panic-free.
unsafe impl<T: Copy> SafeFunction<&T> for Copied {}

// SAFETY: Calls Clone::clone guarded by SafeClone, ensuring logical equivalence, termination, and no destructive side effects.
unsafe impl<T: SafeClone> SafeFunction<&T> for Cloned {}

// SAFETY: F is a safe function for &T of any lifetime and output O does not borrow from the temporary input;
// T: SafeDrop ensures dropping the input after scoped evaluation is benign, panic-free, and terminating.
unsafe impl<T: SafeDrop, O, F> SafeFunction<T> for WithRef<F> where
    F: for<'a> SafeFunction<&'a T, Output = O>
{
}

// SAFETY: Zero-cost field projection; side-effect-free, panic-free, and terminating.
unsafe impl<P, T: ?Sized> SafeFunction<&Subset<P, T>> for AsInner
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue,
{
}

// SAFETY: Zero-cost removal of phantom wrapper moving out the inner value without dropping; pure, panic-free, and terminating.
unsafe impl<P, T> SafeFunction<Subset<P, T>> for IntoInner
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue,
{
}

// SAFETY: Recursively forwards a shared borrow of the base carrier via SafeValue::as_base; pure, panic-free, and terminating.
unsafe impl<P, T: ?Sized> SafeFunction<&Subset<P, T>> for AsBase
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue,
{
}

// SAFETY: Zero-cost bitwise penetration moving out the base carrier without dropping phantom wrappers; pure, panic-free, and terminating.
unsafe impl<P, T> SafeFunction<Subset<P, T>> for IntoBase
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue,
    T::Base: Sized,
{
}

/// Generates a safe struct-field projection function verified by compile-time pure pattern destructuring, avoiding manual `unsafe impl SafeFunction`.
///
/// Internally uses `let Struct { field: ref field_ref, .. } = *input;` and statically checks the exact field type,
/// never triggering `Deref` or any custom method calls; pure, deterministic, terminating, and panic-free.
///
/// - Project by shared reference: `field_projector!(pub GetLo: &Window => lo: &PosI32);`
/// - Project by `Copy` value: `field_projector!(pub GetFlags: &FlagPair => flags: u32);`
/// - Project a `Copy` reference with an explicit lifetime: `field_projector!(pub GetTag: &Item => tag: &'static str);`
/// - Tuple structs support positional indices (e.g., `0`): `field_projector!(pub GetFirst: &Pair => 0: &u32);`
/// - Under `nightly` + `#![feature(const_trait_impl)]`, prefix `const` to emit a `const impl`:
///   `field_projector!(pub const GetLo: &Window => lo: &PosI32);`
#[macro_export]
macro_rules! field_projector {
    ($(#[$meta:meta])* $vis:vis const $name:ident : &$struct_path:path => $field:tt : &$lt:lifetime $field_ty:ty $(;)?) => {
        $(#[$meta])*
        $vis struct $name;

        const impl<'a> $crate::function::Function<&'a $struct_path> for $name {
            type Output = &$lt $field_ty;

            #[inline(always)]
            fn call(input: &'a $struct_path) -> &$lt $field_ty {
                trait _ExactType<T: ?Sized> {}
                impl<T: ?Sized> _ExactType<T> for T {}
                const fn _check<'r, U: ?Sized + _ExactType<V>, V: ?Sized>(_: &'r U) {}

                let $struct_path { $field: ref field_ref, .. } = *input;
                _check::<_, &$lt $field_ty>(field_ref);
                *field_ref
            }
        }

        // SAFETY: Copies a Copy field by offset via pattern destructuring without triggering Deref or any method calls; pure, terminating, and panic-free.
        unsafe impl<'a> $crate::function::SafeFunction<&'a $struct_path> for $name {}
    };
    ($(#[$meta:meta])* $vis:vis $name:ident : &$struct_path:path => $field:tt : &$lt:lifetime $field_ty:ty $(;)?) => {
        $(#[$meta])*
        $vis struct $name;

        impl<'a> $crate::function::Function<&'a $struct_path> for $name {
            type Output = &$lt $field_ty;

            #[inline(always)]
            fn call(input: &'a $struct_path) -> &$lt $field_ty {
                trait _ExactType<T: ?Sized> {}
                impl<T: ?Sized> _ExactType<T> for T {}
                fn _check<'r, U: ?Sized + _ExactType<V>, V: ?Sized>(_: &'r U) {}

                let $struct_path { $field: ref field_ref, .. } = *input;
                _check::<_, &$lt $field_ty>(field_ref);
                *field_ref
            }
        }

        // SAFETY: Copies a Copy field by offset via pattern destructuring without triggering Deref or any method calls; pure, terminating, and panic-free.
        unsafe impl<'a> $crate::function::SafeFunction<&'a $struct_path> for $name {}
    };
    ($(#[$meta:meta])* $vis:vis const $name:ident : &$struct_path:path => $field:tt : &$field_ty:ty $(;)?) => {
        $(#[$meta])*
        $vis struct $name;

        const impl<'a> $crate::function::Function<&'a $struct_path> for $name {
            type Output = &'a $field_ty;

            #[inline(always)]
            fn call(input: &'a $struct_path) -> &'a $field_ty {
                trait _ExactType<T: ?Sized> {}
                impl<T: ?Sized> _ExactType<T> for T {}
                const fn _check<'r, U: ?Sized + _ExactType<V>, V: ?Sized>(_: &'r U) {}

                let $struct_path { $field: ref field_ref, .. } = *input;
                _check::<_, $field_ty>(field_ref);
                field_ref
            }
        }

        // SAFETY: Directly borrows a field by offset via pattern destructuring without triggering Deref or any method calls; pure, terminating, and panic-free.
        unsafe impl<'a> $crate::function::SafeFunction<&'a $struct_path> for $name {}
    };
    ($(#[$meta:meta])* $vis:vis $name:ident : &$struct_path:path => $field:tt : &$field_ty:ty $(;)?) => {
        $(#[$meta])*
        $vis struct $name;

        impl<'a> $crate::function::Function<&'a $struct_path> for $name {
            type Output = &'a $field_ty;

            #[inline(always)]
            fn call(input: &'a $struct_path) -> &'a $field_ty {
                trait _ExactType<T: ?Sized> {}
                impl<T: ?Sized> _ExactType<T> for T {}
                fn _check<'r, U: ?Sized + _ExactType<V>, V: ?Sized>(_: &'r U) {}

                let $struct_path { $field: ref field_ref, .. } = *input;
                _check::<_, $field_ty>(field_ref);
                field_ref
            }
        }

        // SAFETY: Directly borrows a field by offset via pattern destructuring without triggering Deref or any method calls; pure, terminating, and panic-free.
        unsafe impl<'a> $crate::function::SafeFunction<&'a $struct_path> for $name {}
    };
    ($(#[$meta:meta])* $vis:vis const $name:ident : &$struct_path:path => $field:tt : $field_ty:ty $(;)?) => {
        $(#[$meta])*
        $vis struct $name;

        const impl<'a> $crate::function::Function<&'a $struct_path> for $name {
            type Output = $field_ty;

            #[inline(always)]
            fn call(input: &'a $struct_path) -> $field_ty {
                trait _ExactType<T: ?Sized> {}
                impl<T: ?Sized> _ExactType<T> for T {}
                const fn _check<'r, U: ?Sized + _ExactType<V>, V: ?Sized>(_: &'r U) {}

                let $struct_path { $field: ref field_ref, .. } = *input;
                _check::<_, $field_ty>(field_ref);
                *field_ref
            }
        }

        // SAFETY: Copies a Copy field by offset via pattern destructuring without triggering Deref or any method calls; pure, terminating, and panic-free.
        unsafe impl<'a> $crate::function::SafeFunction<&'a $struct_path> for $name {}
    };
    ($(#[$meta:meta])* $vis:vis $name:ident : &$struct_path:path => $field:tt : $field_ty:ty $(;)?) => {
        $(#[$meta])*
        $vis struct $name;

        impl<'a> $crate::function::Function<&'a $struct_path> for $name {
            type Output = $field_ty;

            #[inline(always)]
            fn call(input: &'a $struct_path) -> $field_ty {
                trait _ExactType<T: ?Sized> {}
                impl<T: ?Sized> _ExactType<T> for T {}
                fn _check<'r, U: ?Sized + _ExactType<V>, V: ?Sized>(_: &'r U) {}

                let $struct_path { $field: ref field_ref, .. } = *input;
                _check::<_, $field_ty>(field_ref);
                *field_ref
            }
        }

        // SAFETY: Copies a Copy field by offset via pattern destructuring without triggering Deref or any method calls; pure, terminating, and panic-free.
        unsafe impl<'a> $crate::function::SafeFunction<&'a $struct_path> for $name {}
    };
}
