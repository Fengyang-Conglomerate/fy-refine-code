//! Stable implementations of integer bitwise primitives.

use super::{BitAnd, BitNot, BitOr, BitXor, CheckedShl, CheckedShr, Shl, Shr};
use crate::function::Function;

macro_rules! impl_binary_bitwise {
    ($op:ident, $ty:ty, $fn:tt) => {
        impl Function<($ty, $ty)> for $op {
            type Output = $ty;
            #[inline(always)]
            fn call((a, b): ($ty, $ty)) -> $ty {
                a $fn b
            }
        }
        impl<'a> Function<(&'a $ty, $ty)> for $op {
            type Output = $ty;
            #[inline(always)]
            fn call((a, b): (&'a $ty, $ty)) -> $ty {
                *a $fn b
            }
        }
        impl<'a> Function<($ty, &'a $ty)> for $op {
            type Output = $ty;
            #[inline(always)]
            fn call((a, b): ($ty, &'a $ty)) -> $ty {
                a $fn *b
            }
        }
        impl<'a, 'b> Function<(&'a $ty, &'b $ty)> for $op {
            type Output = $ty;
            #[inline(always)]
            fn call((a, b): (&'a $ty, &'b $ty)) -> $ty {
                *a $fn *b
            }
        }
    };
}

macro_rules! impl_checked_shift {
    ($op:ident, $ty:ty, $method:ident) => {
        impl Function<($ty, u32)> for $op {
            type Output = Option<$ty>;
            #[inline(always)]
            fn call((a, shift): ($ty, u32)) -> Option<$ty> {
                a.$method(shift)
            }
        }
        impl<'a> Function<(&'a $ty, u32)> for $op {
            type Output = Option<$ty>;
            #[inline(always)]
            fn call((a, shift): (&'a $ty, u32)) -> Option<$ty> {
                (*a).$method(shift)
            }
        }
        impl<'a> Function<($ty, &'a u32)> for $op {
            type Output = Option<$ty>;
            #[inline(always)]
            fn call((a, shift): ($ty, &'a u32)) -> Option<$ty> {
                a.$method(*shift)
            }
        }
        impl<'a, 'b> Function<(&'a $ty, &'b u32)> for $op {
            type Output = Option<$ty>;
            #[inline(always)]
            fn call((a, shift): (&'a $ty, &'b u32)) -> Option<$ty> {
                (*a).$method(*shift)
            }
        }
    };
}

macro_rules! impl_const_shift {
    ($op:ident, $ty:ty, $shift:tt, $msg:literal) => {
        impl<const N: u32> Function<$ty> for $op<N> {
            type Output = $ty;
            #[inline(always)]
            fn call(input: $ty) -> $ty {
                const { assert!(N < <$ty>::BITS, $msg) };
                input $shift N
            }
        }
        impl<'a, const N: u32> Function<&'a $ty> for $op<N> {
            type Output = $ty;
            #[inline(always)]
            fn call(input: &'a $ty) -> $ty {
                const { assert!(N < <$ty>::BITS, $msg) };
                *input $shift N
            }
        }
    };
}

macro_rules! impl_bitwise_stable {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Function<$ty> for BitNot {
                type Output = $ty;
                #[inline(always)]
                fn call(input: $ty) -> $ty {
                    !input
                }
            }
            impl<'a> Function<&'a $ty> for BitNot {
                type Output = $ty;
                #[inline(always)]
                fn call(input: &'a $ty) -> $ty {
                    !*input
                }
            }

            impl_binary_bitwise!(BitAnd, $ty, &);
            impl_binary_bitwise!(BitOr, $ty, |);
            impl_binary_bitwise!(BitXor, $ty, ^);

            impl_checked_shift!(CheckedShl, $ty, checked_shl);
            impl_checked_shift!(CheckedShr, $ty, checked_shr);

            impl_const_shift!(Shl, $ty, <<, "left shift amount must be strictly less than integer bit width");
            impl_const_shift!(Shr, $ty, >>, "right shift amount must be strictly less than integer bit width");
        )*
    };
}

impl_bitwise_stable!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);
