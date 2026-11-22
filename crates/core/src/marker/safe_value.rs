//! Value safety contract, recursive base unwrapping, and implementations for fundamental types.
//!
//! Guarantees that a type's read-only logical value remains strictly constant under any shared borrow
//! so that predicate evaluation never drifts, while providing a safe protocol for recursively unwrapping
//! nested refinement wrappers down to the innermost underlying carrier type.

#[cfg(feature = "nightly")]
mod safe_value_nightly;
#[cfg(not(feature = "nightly"))]
mod safe_value_stable;

#[cfg(feature = "nightly")]
mod core_impls_nightly;
#[cfg(not(feature = "nightly"))]
mod core_impls_stable;

#[cfg(all(feature = "alloc", feature = "nightly"))]
mod alloc_impls_nightly;
#[cfg(all(feature = "alloc", not(feature = "nightly")))]
mod alloc_impls_stable;

#[cfg(all(feature = "std", feature = "nightly"))]
mod std_impls_nightly;
#[cfg(all(feature = "std", not(feature = "nightly")))]
mod std_impls_stable;

#[cfg(feature = "nightly")]
pub use safe_value_nightly::SafeValue;
#[cfg(not(feature = "nightly"))]
pub use safe_value_stable::SafeValue;

/// Generates a compile-time-verified [`SafeValue`] implementation via exhaustive destructuring for a struct or enum, avoiding manual `unsafe`.
///
/// Performs exhaustive pattern matching on the target type without `..` or `_` wildcards and requires
/// every field to implement [`SafeValue`], letting the compiler statically guarantee that no field or
/// variant is omitted and no uncontrolled interior mutability is present; fixes `type Base = Self`.
///
/// - Named-field struct: `impl_safe_value!(Window { lo, hi });`
/// - Tuple struct: `impl_safe_value!(Pair(left, right));`
/// - Unit struct: `impl_safe_value!(Empty);`
/// - Enum: `impl_safe_value!(enum Event { Click { x, y }, Text(s), Quit });`
/// - Under `nightly` + `#![feature(const_trait_impl)]`, prefix with `const` to generate `const unsafe impl`:
///   `impl_safe_value!(const Window { lo, hi });` or `impl_safe_value!(const enum Event { ... });`
#[macro_export]
macro_rules! impl_safe_value {
    (const enum $enum_path:path {
        $(
            $variant:ident
            $( { $($nf:ident),* $(,)? } )?
            $( ( $($tf:ident),* $(,)? ) )?
        ),+ $(,)?
    } $(;)?) => {
        // SAFETY: Verified at compile time via wildcard-free exhaustive enum matching checking `SafeValue` on every field of every variant; `Base` is `Self`.
        const unsafe impl $crate::marker::SafeValue for $enum_path {
            type Base = Self;

            #[inline(always)]
            fn as_base(&self) -> &Self {
                const fn _require_safe_value<T: ?Sized + $crate::marker::SafeValue>(_: &T) {}
                match self {
                    $(
                        Self::$variant
                        $( { $($nf),* } )?
                        $( ( $($tf),* ) )?
                        => {
                            $( $( _require_safe_value($nf); )* )?
                            $( $( _require_safe_value($tf); )* )?
                        }
                    ),+
                }
                self
            }
        }
    };
    (enum $enum_path:path {
        $(
            $variant:ident
            $( { $($nf:ident),* $(,)? } )?
            $( ( $($tf:ident),* $(,)? ) )?
        ),+ $(,)?
    } $(;)?) => {
        // SAFETY: Verified at compile time via wildcard-free exhaustive enum matching checking `SafeValue` on every field of every variant; `Base` is `Self`.
        unsafe impl $crate::marker::SafeValue for $enum_path {
            type Base = Self;

            #[inline(always)]
            fn as_base(&self) -> &Self {
                fn _require_safe_value<T: ?Sized + $crate::marker::SafeValue>(_: &T) {}
                match self {
                    $(
                        Self::$variant
                        $( { $($nf),* } )?
                        $( ( $($tf),* ) )?
                        => {
                            $( $( _require_safe_value($nf); )* )?
                            $( $( _require_safe_value($tf); )* )?
                        }
                    ),+
                }
                self
            }
        }
    };
    (const $struct_path:path { $($field:ident),+ $(,)? } $(;)?) => {
        // SAFETY: Verified at compile time via exhaustive destructuring checking `SafeValue` on every field with none omitted; `Base` is `Self`.
        const unsafe impl $crate::marker::SafeValue for $struct_path {
            type Base = Self;

            #[inline(always)]
            fn as_base(&self) -> &Self {
                const fn _require_safe_value<T: ?Sized + $crate::marker::SafeValue>(_: &T) {}
                let $struct_path { $($field),+ } = self;
                $(_require_safe_value($field);)+
                self
            }
        }
    };
    ($struct_path:path { $($field:ident),+ $(,)? } $(;)?) => {
        // SAFETY: Verified at compile time via exhaustive destructuring checking `SafeValue` on every field with none omitted; `Base` is `Self`.
        unsafe impl $crate::marker::SafeValue for $struct_path {
            type Base = Self;

            #[inline(always)]
            fn as_base(&self) -> &Self {
                fn _require_safe_value<T: ?Sized + $crate::marker::SafeValue>(_: &T) {}
                let $struct_path { $($field),+ } = self;
                $(_require_safe_value($field);)+
                self
            }
        }
    };
    (const $head:ident $(:: $tail:ident)* ( $($field:ident),+ $(,)? ) $(;)?) => {
        // SAFETY: Verified at compile time via exhaustive destructuring checking `SafeValue` on every field with none omitted; `Base` is `Self`.
        const unsafe impl $crate::marker::SafeValue for $head $(:: $tail)* {
            type Base = Self;

            #[inline(always)]
            fn as_base(&self) -> &Self {
                const fn _require_safe_value<T: ?Sized + $crate::marker::SafeValue>(_: &T) {}
                let $head $(:: $tail)* ($($field),+) = self;
                $(_require_safe_value($field);)+
                self
            }
        }
    };
    ($head:ident $(:: $tail:ident)* ( $($field:ident),+ $(,)? ) $(;)?) => {
        // SAFETY: Verified at compile time via exhaustive destructuring checking `SafeValue` on every field with none omitted; `Base` is `Self`.
        unsafe impl $crate::marker::SafeValue for $head $(:: $tail)* {
            type Base = Self;

            #[inline(always)]
            fn as_base(&self) -> &Self {
                fn _require_safe_value<T: ?Sized + $crate::marker::SafeValue>(_: &T) {}
                let $head $(:: $tail)* ($($field),+) = self;
                $(_require_safe_value($field);)+
                self
            }
        }
    };
    (const $struct_path:path $(;)?) => {
        // SAFETY: Verified at compile time via zero-field pattern matching to be a payload-free struct with no interior mutability; `Base` is `Self`.
        const unsafe impl $crate::marker::SafeValue for $struct_path {
            type Base = Self;

            #[inline(always)]
            fn as_base(&self) -> &Self {
                let $struct_path {} = *self;
                self
            }
        }
    };
    ($struct_path:path $(;)?) => {
        // SAFETY: Verified at compile time via zero-field pattern matching to be a payload-free struct with no interior mutability; `Base` is `Self`.
        unsafe impl $crate::marker::SafeValue for $struct_path {
            type Base = Self;

            #[inline(always)]
            fn as_base(&self) -> &Self {
                let $struct_path {} = *self;
                self
            }
        }
    };
}
