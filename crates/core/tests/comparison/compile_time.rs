use super::*;

const RESULTS: [bool; 10] = [
    Eq::call((1, 1)),
    Ne::call((-1, 0)),
    Lt::call((-1, u128::MAX)),
    Le::call((1, 1)),
    Gt::call((1, 0)),
    Ge::call((1, 1)),
    LtConst::<ConstU8<10>>::call(&5u8),
    EqConst::<ConstChar<'a'>>::call(&'a'),
    SafeEq::eq(&[1i8, 2], &[1u32, 2]),
    SafeOrd::lt(&&[-1i8, 5], &[0u64, 1]),
];

#[test]
fn const_predicates() {
    assert_eq!(RESULTS, [true; 10]);
}
