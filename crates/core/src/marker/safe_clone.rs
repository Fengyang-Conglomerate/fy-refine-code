//! Faithful cloning contract and implementations for fundamental types.
//!
//! Constrains the standard library [`Clone`] trait via an `unsafe` marker contract (symmetric with
//! [`SafeDrop`](super::SafeDrop)), promising that cloned copies are logically equivalent to the
//! original value under all safe functions, thereby guarding soundness when refined subsets
//! duplicate instances without re-checking predicates.

mod core_impls;

#[cfg(feature = "alloc")]
mod alloc_impls;
#[cfg(feature = "std")]
mod std_impls;

/// Promises that the type's standard library [`Clone`] implementation is faithful, producing a copy logically equivalent to the original value.
///
/// This protocol constrains [`Clone`] via an `unsafe` marker contract (symmetric with [`SafeDrop`](super::SafeDrop))
/// to preserve strong invariants when [`Subset`](crate::subset::Subset) clones constrained values without re-validating predicates.
///
/// # Safety
/// Implementors must guarantee that [`Clone::clone`] (and [`Clone::clone_from`]) satisfies all of the following requirements:
///
/// 1. **Faithful Equivalence**:
///    For any valid instance `x`, the copy `y = x.clone()` must yield identical results to `x` under every safe function
///    (`SafeFunction<&Self>`). It must never alter, drop, or recompute data invariants in a way that could break refinement
///    predicates (including private internal state such as the exhausted flag of `RangeInclusive` or the internal topology/contents of trees and hash tables).
/// 2. **Consistency with Bitwise Copy (`Copy` Compatibility)**:
///    If the type also implements `Copy`, `Clone::clone` must be semantically identical to a bitwise copy `*self`.
/// 3. **Guaranteed Termination and No Domain-Mutating Side Effects**:
///    `clone` must terminate in finite steps, must not trigger domain-logic panics, and must not mutate external application state
///    (excluding controlled runtime metadata updates such as heap allocation or incrementing `Rc`/`Arc` reference counts).
pub unsafe trait SafeClone: Clone {}

/// Generates a structural [`Clone`] implementation and [`SafeClone`] safety contract for a struct or enum, avoiding manual `unsafe` and mutating clones.
///
/// The macro destructures the target type exhaustively without `..` or `_` wildcards, requires every field
/// to implement [`SafeClone`], and delegates directly to `<Field as Clone>::clone` to construct the new instance,
/// ensuring faithful logical equivalence by construction (compatible with `#[derive(Copy)]`).
///
/// ```
/// use fy_refine_core::function::predicate::IsPositive;
/// use fy_refine_core::{impl_safe_clone, impl_safe_value, Subset};
///
/// struct Window { lo: Subset<IsPositive, i32>, hi: Subset<IsPositive, i32>, tag: &'static str }
/// impl_safe_value!(Window { lo, hi, tag });
/// impl_safe_clone!(Window { lo, hi, tag });
///
/// #[derive(Copy, Debug, PartialEq, Eq)]
/// struct Pair(u32, u8);
/// impl_safe_value!(Pair(a, b));
/// impl_safe_clone!(Pair(a, b));
///
/// #[derive(Copy)]
/// struct Unit;
/// impl_safe_value!(Unit);
/// impl_safe_clone!(Unit);
///
/// enum Event { Click { x: i32, y: i32 }, Text(&'static str), Quit }
/// impl_safe_value!(enum Event { Click { x, y }, Text(s), Quit });
/// impl_safe_clone!(enum Event { Click { x, y }, Text(s), Quit });
///
/// let p = Subset::<IsPositive, i32>::try_new(1).unwrap();
/// let _copy: Pair = Pair(1, 2);
/// let _dup = (_copy, _copy);
/// let _w = Window { lo: p, hi: p, tag: "ok" }.clone();
/// ```
#[macro_export]
macro_rules! impl_safe_clone {
    (enum $enum_path:path {
        $(
            $variant:ident
            $( { $($nf:ident),* $(,)? } )?
            $( ( $($tf:ident),* $(,)? ) )?
        ),+ $(,)?
    } $(;)?) => {
        #[allow(clippy::expl_impl_clone_on_copy, clippy::non_canonical_clone_impl)]
        impl ::core::clone::Clone for $enum_path {
            #[inline]
            fn clone(&self) -> Self {
                #[allow(dead_code)]
                #[inline(always)]
                fn _clone_safe_field<T: $crate::marker::SafeClone>(field: &T) -> T {
                    <T as ::core::clone::Clone>::clone(field)
                }

                match self {
                    $(
                        Self::$variant
                        $( { $($nf),* } )?
                        $( ( $($tf),* ) )?
                        => Self::$variant
                        $( { $($nf: _clone_safe_field($nf)),* } )?
                        $( ( $(_clone_safe_field($tf)),* ) )?
                    ),+
                }
            }
        }

        // SAFETY: `Clone` is generated from exhaustive wildcard-free destructuring and delegates each field to `SafeClone`, preserving full logical equivalence with the original value.
        unsafe impl $crate::marker::SafeClone for $enum_path {}
    };
    ($struct_path:path { $($field:ident),+ $(,)? } $(;)?) => {
        #[allow(clippy::expl_impl_clone_on_copy, clippy::non_canonical_clone_impl)]
        impl ::core::clone::Clone for $struct_path {
            #[inline]
            fn clone(&self) -> Self {
                #[inline(always)]
                fn _clone_safe_field<T: $crate::marker::SafeClone>(field: &T) -> T {
                    <T as ::core::clone::Clone>::clone(field)
                }

                let $struct_path { $($field),+ } = self;
                $struct_path {
                    $($field: _clone_safe_field($field)),+
                }
            }
        }

        // SAFETY: `Clone` is generated via exhaustive destructuring and field-wise `SafeClone` delegation, producing a copy logically equivalent to the original value.
        unsafe impl $crate::marker::SafeClone for $struct_path {}
    };
    ($head:ident $(:: $tail:ident)* ( $($field:ident),+ $(,)? ) $(;)?) => {
        #[allow(clippy::expl_impl_clone_on_copy, clippy::non_canonical_clone_impl)]
        impl ::core::clone::Clone for $head $(:: $tail)* {
            #[inline]
            fn clone(&self) -> Self {
                #[inline(always)]
                fn _clone_safe_field<T: $crate::marker::SafeClone>(field: &T) -> T {
                    <T as ::core::clone::Clone>::clone(field)
                }

                let $head $(:: $tail)* ($($field),+) = self;
                $head $(:: $tail)* ($(_clone_safe_field($field)),+)
            }
        }

        // SAFETY: `Clone` is generated via exhaustive destructuring and field-wise `SafeClone` delegation, producing a copy logically equivalent to the original value.
        unsafe impl $crate::marker::SafeClone for $head $(:: $tail)* {}
    };
    ($struct_path:path $(;)?) => {
        #[allow(clippy::expl_impl_clone_on_copy, clippy::non_canonical_clone_impl)]
        impl ::core::clone::Clone for $struct_path {
            #[inline(always)]
            fn clone(&self) -> Self {
                let $struct_path {} = self;
                $struct_path {}
            }
        }

        // SAFETY: Cloning a zero-payload struct reconstructs the empty instance directly, identical to a bitwise copy and the original value.
        unsafe impl $crate::marker::SafeClone for $struct_path {}
    };
}
