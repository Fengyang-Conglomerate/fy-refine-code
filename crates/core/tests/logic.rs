//! Tests for logical predicate combinators (And, Or, Not, Xor, True, False, Always) and struct boolean field composition.

use core::any::TypeId;
use fy_refine_core::function::constant::ConstBool;
use fy_refine_core::function::predicate::{AllOf, And, AnyOf, False, Not, Or, True, Xor};
use fy_refine_core::function::wiring::{Always, Fst, Id, Snd};
use fy_refine_core::function::{Function, SafeFunction};
use fy_refine_core::marker::SafeValue;
use fy_refine_core::subset::Subset;

struct CheckEven;
impl Function<i32> for CheckEven {
    type Output = bool;
    fn call(x: i32) -> bool {
        x % 2 == 0
    }
}
unsafe impl SafeFunction<i32> for CheckEven {}

#[test]
fn boolean_logic_combinators() {
    assert!(True::call(()));
    assert!(!False::call(()));

    assert!(Always::<True>::call(12345));
    assert!(!Always::<False>::call("ignored"));

    type EvenAndTrue = And<CheckEven, Always<True>>;
    assert!(EvenAndTrue::call(4));
    assert!(!EvenAndTrue::call(5));

    type EvenOrFalse = Or<CheckEven, Always<False>>;
    assert!(EvenOrFalse::call(4));
    assert!(!EvenOrFalse::call(5));

    type NotEven = Not<CheckEven>;
    assert!(!NotEven::call(4));
    assert!(NotEven::call(5));

    type EvenXorTrue = Xor<CheckEven, Always<True>>;
    assert!(!EvenXorTrue::call(4));
    assert!(EvenXorTrue::call(5));

    // (bool, bool) pairs composed directly via And<Fst, Snd> / Or<Fst, Snd> / Xor<Fst, Snd>
    assert!(And::<Fst, Snd>::call((true, true)));
    assert!(!And::<Fst, Snd>::call((true, false)));
    assert!(!And::<Fst, Snd>::call((false, true)));
    assert!(!And::<Fst, Snd>::call((false, false)));

    assert!(Or::<Fst, Snd>::call((true, false)));
    assert!(Or::<Fst, Snd>::call((false, true)));
    assert!(Or::<Fst, Snd>::call((true, true)));
    assert!(!Or::<Fst, Snd>::call((false, false)));

    assert!(Xor::<Fst, Snd>::call((true, false)));
    assert!(Xor::<Fst, Snd>::call((false, true)));
    assert!(!Xor::<Fst, Snd>::call((true, true)));
    assert!(!Xor::<Fst, Snd>::call((false, false)));

    assert!(!Not::<Id>::call(true));
    assert!(Not::<Id>::call(false));

    // True / False are strictly type-identical to ConstBool<true> / ConstBool<false>
    assert_eq!(TypeId::of::<True>(), TypeId::of::<ConstBool<true>>());
    assert_eq!(TypeId::of::<False>(), TypeId::of::<ConstBool<false>>());

    // AnyOf / AllOf flat tuple normalization (empty, single-element, and multi-element tuples)
    assert!(!AnyOf::<()>::call(0i32));
    assert!(AllOf::<()>::call(0i32));
    assert!(AnyOf::<(CheckEven,)>::call(4));
    assert!(!AllOf::<(CheckEven,)>::call(5));
    assert!(AnyOf::<(Always<False>, CheckEven, Always<False>)>::call(6));
    assert!(!AnyOf::<(Always<False>, CheckEven, Always<False>)>::call(7));
    assert!(AllOf::<(Always<True>, CheckEven, Always<True>)>::call(6));
    assert!(!AllOf::<(Always<True>, CheckEven, Always<True>)>::call(7));
}

#[test]
fn struct_boolean_fields_direct_predicate_composition() {
    #[derive(Debug, PartialEq, Eq)]
    struct Ab {
        a: bool,
        b: bool,
    }
    unsafe impl SafeValue for Ab {
        type Base = Self;
        fn as_base(&self) -> &Self {
            self
        }
    }

    struct GetA;
    impl<'a> Function<&'a Ab> for GetA {
        type Output = bool;
        fn call(input: &'a Ab) -> bool {
            input.a
        }
    }
    unsafe impl SafeFunction<&Ab> for GetA {}

    struct GetB;
    impl<'a> Function<&'a Ab> for GetB {
        type Output = bool;
        fn call(input: &'a Ab) -> bool {
            input.b
        }
    }
    unsafe impl SafeFunction<&Ab> for GetB {}

    // Directly compose And<GetA, GetB> without EqConst<ConstBool<true>> or BoolAnd
    type BothTrue = Subset<And<GetA, GetB>, Ab>;
    type EitherTrue = Subset<Or<GetA, GetB>, Ab>;
    type ExactlyOne = Subset<Xor<GetA, GetB>, Ab>;

    assert!(BothTrue::try_new(Ab { a: true, b: true }).is_ok());
    assert!(BothTrue::try_new(Ab { a: true, b: false }).is_err());

    assert!(EitherTrue::try_new(Ab { a: false, b: true }).is_ok());
    assert!(EitherTrue::try_new(Ab { a: false, b: false }).is_err());

    assert!(ExactlyOne::try_new(Ab { a: true, b: false }).is_ok());
    assert!(ExactlyOne::try_new(Ab { a: true, b: true }).is_err());
}
