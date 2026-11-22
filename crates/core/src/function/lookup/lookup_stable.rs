//! Stable implementations of static lookup table and bitset membership.

use core::cmp::Ordering;

use super::{InBitset, InSliceEq, InSortedRanges, InSortedSlice, test_bitset};
use crate::cmp::{SafeEq, SafeOrd};
use crate::function::{Function, SafeFunction};

/// Checks whether a slice is strictly ascending according to [`SafeOrd`] (no duplicates, no out-of-order elements).
#[inline]
pub fn is_strictly_sorted<T: SafeOrd>(slice: &[T]) -> bool {
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

/// Checks whether a slice of closed intervals satisfies `lo <= hi` for every interval and strictly ordered disjointness (`prev.hi < next.lo`) across intervals.
#[inline]
pub fn is_sorted_disjoint_ranges<T: SafeOrd>(ranges: &[(T, T)]) -> bool {
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

impl<T, Table, U: 'static> Function<T> for InSliceEq<Table>
where
    Table: SafeFunction<(), Output = &'static [U]>,
    T: SafeEq<U>,
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

impl<T, Table, U: 'static> Function<T> for InSortedSlice<Table>
where
    Table: SafeFunction<(), Output = &'static [U]>,
    T: SafeOrd<U>,
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

impl<T, Table, U: 'static> Function<T> for InSortedRanges<Table>
where
    Table: SafeFunction<(), Output = &'static [(U, U)]>,
    T: SafeOrd<U>,
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

macro_rules! impl_in_bitset_stable {
    ($($ty:ty),* $(,)?) => {
        $(
            impl<Table> Function<$ty> for InBitset<Table>
            where
                Table: SafeFunction<(), Output = &'static [u64]>,
            {
                type Output = bool;

                #[inline(always)]
                fn call(input: $ty) -> bool {
                    test_bitset(Table::call(()), input as usize)
                }
            }

            impl<'a, Table> Function<&'a $ty> for InBitset<Table>
            where
                Table: SafeFunction<(), Output = &'static [u64]>,
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

impl_in_bitset_stable!(u8, u16, u32, usize);
