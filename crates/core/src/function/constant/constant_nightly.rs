//! Compile-time primitive literal constant morphism implementations supporting `const Function` under `nightly`.

use crate::function::{Function, SafeFunction};

macro_rules! define_const {
    (#[$doc:meta] $name:ident : $ty:ty) => {
        #[$doc]
        pub struct $name<const V: $ty>;

        const impl<const V: $ty> Function<()> for $name<V> {
            type Output = $ty;

            #[inline(always)]
            fn call((): ()) -> $ty {
                V
            }
        }

        // SAFETY: Returns a compile-time literal constant; pure, deterministic, guaranteed to terminate, and panic-free.
        unsafe impl<const V: $ty> SafeFunction<()> for $name<V> {}
    };
}

define_const!(/// Compile-time constant morphism of type `u8`.
    ConstU8: u8);
define_const!(/// Compile-time constant morphism of type `u16`.
    ConstU16: u16);
define_const!(/// Compile-time constant morphism of type `u32`.
    ConstU32: u32);
define_const!(/// Compile-time constant morphism of type `u64`.
    ConstU64: u64);
define_const!(/// Compile-time constant morphism of type `u128`.
    ConstU128: u128);
define_const!(/// Compile-time constant morphism of type `usize`.
    ConstUsize: usize);
define_const!(/// Compile-time constant morphism of type `i8`.
    ConstI8: i8);
define_const!(/// Compile-time constant morphism of type `i16`.
    ConstI16: i16);
define_const!(/// Compile-time constant morphism of type `i32`.
    ConstI32: i32);
define_const!(/// Compile-time constant morphism of type `i64`.
    ConstI64: i64);
define_const!(/// Compile-time constant morphism of type `i128`.
    ConstI128: i128);
define_const!(/// Compile-time constant morphism of type `isize`.
    ConstIsize: isize);
define_const!(/// Compile-time constant morphism of type `bool`.
    ConstBool: bool);
define_const!(/// Compile-time constant morphism of type `char`.
    ConstChar: char);
