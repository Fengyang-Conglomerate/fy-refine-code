//! Safe destruction contract and implementations for fundamental types.
//!
//! In refinement types and pure function wiring combinators (such as projections and discard operators),
//! dropping values must be guaranteed to terminate cleanly without destructive side effects.

mod core_impls;

#[cfg(feature = "alloc")]
mod alloc_impls;
#[cfg(feature = "std")]
mod std_impls;

/// Promises that dropping values of this type is benign, panic-free, and guaranteed to terminate.
///
/// In a refinement type system and pure function combinator topology, dropping data need not be
/// devoid of physical state changes (for instance, freeing heap allocations, closing OS resource
/// handles, or emitting benign debug/monitoring logs is permitted), provided it remains transparent
/// to logical verification state and never compromises the determinism or termination of pure functions.
///
/// # Safety
/// Implementors must guarantee:
///
/// 1. **Guaranteed Termination**: Invoking the destructor never deadlocks or loops infinitely and always completes in finite steps.
/// 2. **Panic-free**: The destructor never panics.
/// 3. **Logical Transparency (No Domain Side Effects)**: Dropping must not mutate external application state,
///    nor alter invariants or global state that affect refinement predicate evaluation or domain logic.
pub unsafe trait SafeDrop {}

/// Generates a compile-time-verified [`SafeDrop`] implementation via exhaustive destructuring for a struct or enum, avoiding manual `unsafe`.
///
/// Performs two compile-time checks inside an anonymous `const` block without `..` or `_` wildcards:
/// 1. **Exhaustive Move-out (`_verify_move`)**: Requires every field to implement [`SafeDrop`]. If the target type implements `Drop` itself and any field is `!Copy` (such as `String` or `Vec`), compilation fails directly with `E0509`.
/// 2. **Compile-time Drop Check (`_verify_no_custom_drop`)**: If the target type consists solely of `Copy`/no-drop fields (`any_field_drop == false`) or is an empty struct, asserts via `core::mem::needs_drop::<Self>()` that the target type itself has no custom `Drop` implementation.
///
/// ```
/// use fy_refine_core::marker::SafeDrop;
/// use fy_refine_core::{impl_safe_drop, Subset};
/// use fy_refine_core::function::predicate::IsPositive;
/// use fy_refine_core::function::wiring::Discard;
/// use fy_refine_core::function::Function;
///
/// struct Window { lo: Subset<IsPositive, i32>, hi: Subset<IsPositive, i32>, label: &'static str }
/// impl_safe_drop!(Window { lo, hi, label });
///
/// struct Pair(u32, &'static str);
/// impl_safe_drop!(Pair(a, b));
///
/// struct Unit;
/// impl_safe_drop!(Unit);
///
/// enum Event { Click { x: i32, y: i32 }, Text(&'static str), Quit }
/// impl_safe_drop!(enum Event { Click { x, y }, Text(s), Quit });
///
/// let p = Subset::<IsPositive, i32>::try_new(1).unwrap();
/// Discard::call(Window { lo: p, hi: p, label: "ok" });
/// ```
#[macro_export]
macro_rules! impl_safe_drop {
    (enum $enum_path:path {
        $(
            $variant:ident
            $( { $($nf:ident),* $(,)? } )?
            $( ( $($tf:ident),* $(,)? ) )?
        ),+ $(,)?
    } $(;)?) => {
        // SAFETY: Verified at compile time by both exhaustive move-out destructuring (`_verify_move`) and drop-presence checking (`_verify_no_custom_drop`):
        // 1. Every field of every variant satisfies `SafeDrop`;
        // 2. The target enum defines no custom `Drop` (if any field is `!Copy`, `E0509` rejects it; if all fields are `Copy`/no-drop, the `needs_drop` assertion rejects it).
        unsafe impl $crate::marker::SafeDrop for $enum_path {}

        const _: () = {
            #[allow(dead_code)]
            #[inline(always)]
            fn _consume_safe_drop<T: $crate::marker::SafeDrop>(_: T) {}

            #[allow(dead_code)]
            #[inline(always)]
            const fn _opt_needs_drop<T: $crate::marker::SafeDrop>(_: ::core::option::Option<&T>) -> bool {
                ::core::mem::needs_drop::<T>()
            }

            #[allow(dead_code)]
            #[inline(always)]
            const fn _none<T>(_: &str) -> ::core::option::Option<T> {
                ::core::option::Option::None
            }

            type _Target = $enum_path;

            fn _verify_move(value: _Target) {
                match value {
                    $(
                        _Target::$variant
                        $( { $($nf),* } )?
                        $( ( $($tf),* ) )?
                        => {
                            $( $( _consume_safe_drop($nf); )* )?
                            $( $( _consume_safe_drop($tf); )* )?
                        }
                    ),+
                }
            }

            #[allow(unused_mut)]
            const fn _verify_no_custom_drop(v: ::core::option::Option<&_Target>) {
                let mut any_field_drop = false;
                $(
                    {
                        let ( $( $($nf,)* )? $( $($tf,)* )? ) = match v {
                            ::core::option::Option::Some(
                                _Target::$variant
                                $( { $($nf),* } )?
                                $( ( $($tf),* ) )?
                            ) => (
                                $( $( ::core::option::Option::Some($nf), )* )?
                                $( $( ::core::option::Option::Some($tf), )* )?
                            ),
                            _ => (
                                $( $( _none(::core::stringify!($nf)), )* )?
                                $( $( _none(::core::stringify!($tf)), )* )?
                            ),
                        };
                        $( $( if _opt_needs_drop($nf) { any_field_drop = true; } )* )?
                        $( $( if _opt_needs_drop($tf) { any_field_drop = true; } )* )?
                    }
                )+
                assert!(
                    !::core::mem::needs_drop::<_Target>() || any_field_drop,
                    "target enum has a custom Drop implementation and cannot be verified by impl_safe_drop!"
                );
            }

            let _: fn(_Target) = _verify_move;
            _verify_no_custom_drop(::core::option::Option::None);
        };
    };
    ($struct_path:path { $($field:ident),+ $(,)? } $(;)?) => {
        // SAFETY: Verified at compile time by both exhaustive move-out destructuring and the `needs_drop` assertion: the target struct has no custom `Drop` and all fields satisfy `SafeDrop`.
        unsafe impl $crate::marker::SafeDrop for $struct_path {}

        const _: () = {
            #[inline(always)]
            fn _consume_safe_drop<T: $crate::marker::SafeDrop>(_: T) {}

            #[inline(always)]
            const fn _opt_needs_drop<T: $crate::marker::SafeDrop>(_: ::core::option::Option<&T>) -> bool {
                ::core::mem::needs_drop::<T>()
            }

            #[inline(always)]
            const fn _none<T>(_: &str) -> ::core::option::Option<T> {
                ::core::option::Option::None
            }

            fn _verify_move(value: $struct_path) {
                let $struct_path { $($field),+ } = value;
                $(_consume_safe_drop($field);)+
            }

            const fn _verify_no_custom_drop(v: ::core::option::Option<&$struct_path>) {
                let ($($field,)+) = match v {
                    ::core::option::Option::Some($struct_path { $($field),+ }) => (
                        $(::core::option::Option::Some($field),)+
                    ),
                    ::core::option::Option::None => (
                        $(_none(::core::stringify!($field)),)+
                    ),
                };
                let any_field_drop = false $( || _opt_needs_drop($field) )+;
                assert!(
                    !::core::mem::needs_drop::<$struct_path>() || any_field_drop,
                    "target struct has a custom Drop implementation and cannot be verified by impl_safe_drop!"
                );
            }

            let _: fn($struct_path) = _verify_move;
            _verify_no_custom_drop(::core::option::Option::None);
        };
    };
    ($head:ident $(:: $tail:ident)* ( $($field:ident),+ $(,)? ) $(;)?) => {
        // SAFETY: Verified at compile time by both exhaustive move-out destructuring and the `needs_drop` assertion: the target tuple struct has no custom `Drop` and all fields satisfy `SafeDrop`.
        unsafe impl $crate::marker::SafeDrop for $head $(:: $tail)* {}

        const _: () = {
            #[inline(always)]
            fn _consume_safe_drop<T: $crate::marker::SafeDrop>(_: T) {}

            #[inline(always)]
            const fn _opt_needs_drop<T: $crate::marker::SafeDrop>(_: ::core::option::Option<&T>) -> bool {
                ::core::mem::needs_drop::<T>()
            }

            #[inline(always)]
            const fn _none<T>(_: &str) -> ::core::option::Option<T> {
                ::core::option::Option::None
            }

            fn _verify_move(value: $head $(:: $tail)*) {
                let $head $(:: $tail)* ($($field),+) = value;
                $(_consume_safe_drop($field);)+
            }

            const fn _verify_no_custom_drop(v: ::core::option::Option<&$head $(:: $tail)*>) {
                let ($($field,)+) = match v {
                    ::core::option::Option::Some($head $(:: $tail)* ($($field),+)) => (
                        $(::core::option::Option::Some($field),)+
                    ),
                    ::core::option::Option::None => (
                        $(_none(::core::stringify!($field)),)+
                    ),
                };
                let any_field_drop = false $( || _opt_needs_drop($field) )+;
                assert!(
                    !::core::mem::needs_drop::<$head $(:: $tail)*>() || any_field_drop,
                    "target tuple struct has a custom Drop implementation and cannot be verified by impl_safe_drop!"
                );
            }

            let _: fn($head $(:: $tail)*) = _verify_move;
            _verify_no_custom_drop(::core::option::Option::None);
        };
    };
    ($struct_path:path $(;)?) => {
        // SAFETY: Verified at compile time by zero-field move-out destructuring and the `needs_drop` assertion to be a payload-free struct with no custom `Drop`.
        unsafe impl $crate::marker::SafeDrop for $struct_path {}

        const _: () = {
            fn _verify_move(value: $struct_path) {
                let $struct_path {} = value;
            }

            const fn _verify_no_custom_drop() {
                assert!(
                    !::core::mem::needs_drop::<$struct_path>(),
                    "target unit struct has a custom Drop implementation and cannot be verified by impl_safe_drop!"
                );
            }

            let _: fn($struct_path) = _verify_move;
            _verify_no_custom_drop();
        };
    };
}
