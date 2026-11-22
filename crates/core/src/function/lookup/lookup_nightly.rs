//! Static lookup table and bitset membership implementations supporting conditional `const` calls.

use core::cmp::Ordering;

use super::{InBitset, InSliceEq, InSortedRanges, InSortedSlice, test_bitset};
use crate::cmp::{SafeEq, SafeOrd};
use crate::function::{Function, SafeFunction};

/// Checks whether a slice is strictly ascending according to [`SafeOrd`] (no duplicates, no out-of-order elements); supports conditional compile-time calls under `nightly`.
#[inline]
pub const fn is_strictly_sorted<T: [const] SafeOrd>(slice: &[T]) -> bool {
    let mut i = 1;
    while i < slice.len() {
        if !matches!(
            <T as SafeOrd>::cmp(&slice[i - 1], &slice[i]),
            Ordering::Less
        ) {
            return false;
        }
        i += 1;
    }
    true
}

/// Checks whether a slice of closed intervals satisfies `lo <= hi` for every interval and strictly ordered disjointness (`prev.hi < next.lo`) across intervals; supports conditional compile-time calls under `nightly`.
#[inline]
pub const fn is_sorted_disjoint_ranges<T: [const] SafeOrd>(ranges: &[(T, T)]) -> bool {
    let mut i = 0;
    while i < ranges.len() {
        if matches!(
            <T as SafeOrd>::cmp(&ranges[i].0, &ranges[i].1),
            Ordering::Greater
        ) {
            return false;
        }
        if i > 0
            && !matches!(
                <T as SafeOrd>::cmp(&ranges[i - 1].1, &ranges[i].0),
                Ordering::Less
            )
        {
            return false;
        }
        i += 1;
    }
    true
}

const impl<T, Table, U: 'static> Function<T> for InSliceEq<Table>
where
    Table: SafeFunction<(), Output = &'static [U]> + [const] Function<(), Output = &'static [U]>,
    T: [const] SafeEq<U> + [const] core::marker::Destruct,
{
    type Output = bool;

    #[inline]
    fn call(input: T) -> bool {
        let table = Table::call(());
        let mut i = 0;
        while i < table.len() {
            if <T as SafeEq<U>>::eq(&input, &table[i]) {
                return true;
            }
            i += 1;
        }
        false
    }
}

const impl<T, Table, U: 'static> Function<T> for InSortedSlice<Table>
where
    Table: SafeFunction<(), Output = &'static [U]> + [const] Function<(), Output = &'static [U]>,
    T: [const] SafeOrd<U> + [const] core::marker::Destruct,
{
    type Output = bool;

    #[inline]
    fn call(input: T) -> bool {
        let table = Table::call(());
        let mut lo = 0usize;
        let mut hi = table.len();
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            match <T as SafeOrd<U>>::cmp(&input, &table[mid]) {
                Ordering::Equal => return true,
                Ordering::Greater => lo = mid + 1,
                Ordering::Less => hi = mid,
            }
        }
        false
    }
}

const impl<T, Table, U: 'static> Function<T> for InSortedRanges<Table>
where
    Table: SafeFunction<(), Output = &'static [(U, U)]>
        + [const] Function<(), Output = &'static [(U, U)]>,
    T: [const] SafeOrd<U> + [const] core::marker::Destruct,
{
    type Output = bool;

    #[inline]
    fn call(input: T) -> bool {
        let ranges = Table::call(());
        let mut lo = 0usize;
        let mut hi = ranges.len();
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let (start, end) = &ranges[mid];
            if matches!(<T as SafeOrd<U>>::cmp(&input, start), Ordering::Less) {
                hi = mid;
            } else if matches!(<T as SafeOrd<U>>::cmp(&input, end), Ordering::Greater) {
                lo = mid + 1;
            } else {
                return true;
            }
        }
        false
    }
}

macro_rules! impl_in_bitset_nightly {
    ($($ty:ty),* $(,)?) => {
        $(
            const impl<Table> Function<$ty> for InBitset<Table>
            where
                Table: SafeFunction<(), Output = &'static [u64]>
                    + [const] Function<(), Output = &'static [u64]>,
            {
                type Output = bool;

                #[inline(always)]
                fn call(input: $ty) -> bool {
                    test_bitset(Table::call(()), input as usize)
                }
            }

            const impl<'a, Table> Function<&'a $ty> for InBitset<Table>
            where
                Table: SafeFunction<(), Output = &'static [u64]>
                    + [const] Function<(), Output = &'static [u64]>,
            {
                type Output = bool;

                #[inline(always)]
                fn call(input: &'a $ty) -> bool {
                    test_bitset(Table::call(()), *input as usize)
                }
            }
        )*
    };
}

impl_in_bitset_nightly!(u8, u16, u32, usize);
