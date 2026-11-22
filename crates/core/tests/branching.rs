//! Tests for branch elimination, minimal orthogonal topological wiring, non-Copy ownership transfer, and unpolluted borrowed destructuring.

#![cfg_attr(feature = "nightly", feature(const_trait_impl))]

use core::marker::PhantomData;
use fy_refine_core::function::branch::{
    AndThen, BranchOpt, BranchOptPure, BranchRes, BranchResPure, Classify, ErrAnd, ErrOr, IfElse,
    IfLetElse, IfLetElseFalse, IfLetElseTrue, IsErr, IsNone, IsOk, IsSome, IsVariant, MatchOpt,
    MatchOptPure, MatchResPure, NoneOr, OkAnd, OkElseFalse, OkOr, OptAsRef, OrElse, RefineIf,
    ResAsRef, ResErr, ResOk, SomeAnd, SomeElseFalse, SomeElseTrue, UnwrapErr, UnwrapOk, UnwrapSome,
    UnwrapVariant, ZipOpt, ZipRes,
};
use fy_refine_core::function::collection::{All, IsNonEmpty, TryFirst};
#[cfg(feature = "alloc")]
use fy_refine_core::function::predicate::IsAscii;
use fy_refine_core::function::predicate::{
    False, IsEven, IsFinite, IsInteger, IsNaN, IsNotNaN, IsOdd, IsPositive, PairEq, True,
};
#[cfg(feature = "alloc")]
use fy_refine_core::function::slice::AsBytes;
use fy_refine_core::function::wiring::{
    Always, AssocLeft, AssocRight, AttachUnit, Cloned, Copied, Discard, Dup, Fork, Fst, Id, MapFst,
    MapPair, MapSnd, PipeWith, Snd, Swap, Then, WithRef,
};
use fy_refine_core::function::{Function, SafeFunction};
use fy_refine_core::marker::{SafeClone, SafeDrop, SafeValue};
use fy_refine_core::subset::Subset;
use fy_refine_core::{impl_safe_value, variant_projector};

// ==========================================
// 1. Basic topological wiring primitives
// ==========================================

#[test]
fn basic_wiring_primitives() {
    assert_eq!(Id::call(42), 42);

    struct Add1;
    impl Function<i32> for Add1 {
        type Output = i32;
        fn call(x: i32) -> i32 {
            x + 1
        }
    }
    unsafe impl SafeFunction<i32> for Add1 {}

    struct Mul2;
    impl Function<i32> for Mul2 {
        type Output = i32;
        fn call(x: i32) -> i32 {
            x * 2
        }
    }
    unsafe impl SafeFunction<i32> for Mul2 {}

    // (x + 1) * 2
    assert_eq!(Then::<Add1, Mul2>::call(3), 8);

    // Swap / AssocLeft / AssocRight / AttachUnit
    assert_eq!(Swap::call((1, 2)), (2, 1));
    assert_eq!(AssocLeft::call((1, (2, 3))), ((1, 2), 3));
    assert_eq!(AssocRight::call(((1, 2), 3)), (1, (2, 3)));
    assert_eq!(AttachUnit::call(9), (9, ()));

    // MapPair / MapFst / MapSnd / Fork
    assert_eq!(MapPair::<Add1, Mul2>::call((10, 20)), (11, 40));
    assert_eq!(MapFst::<Add1>::call((10, 20)), (11, 20));
    assert_eq!(MapSnd::<Mul2>::call((10, 20)), (10, 40));
    assert_eq!(Fork::<Add1, Mul2>::call(5), (6, 10));

    // Fst / Snd / Dup / Discard / Copied / Cloned
    assert_eq!(Fst::call((100, 200)), 100);
    assert_eq!(Snd::call((100, 200)), 200);
    assert_eq!(Dup::call(7), (7, 7));
    Discard::call(7);
    assert_eq!(Copied::call(&42u32), 42u32);
    assert_eq!(Cloned::call(&42u32), 42u32);
    #[cfg(feature = "alloc")]
    assert_eq!(Cloned::call(&String::from("hi")), "hi");

    // PipeWith: x -> AddPair((Add1(x), x))
    struct AddPair;
    impl Function<(i32, i32)> for AddPair {
        type Output = i32;
        fn call((a, b): (i32, i32)) -> i32 {
            a + b
        }
    }
    unsafe impl SafeFunction<(i32, i32)> for AddPair {}
    assert_eq!(PipeWith::<Add1, AddPair>::call(5), 11);

    // Route
    type Route = Then<AssocLeft, MapFst<Swap>>;
    assert_eq!(Route::call((1u8, (2u16, 3u32))), ((2u16, 1u8), 3u32));
}

#[test]
fn direct_branch_and_ifelse_primitives() {
    struct HandleSome;
    impl Function<(i32, &'static str)> for HandleSome {
        type Output = String;
        fn call((val, ctx): (i32, &'static str)) -> String {
            format!("{}:{}", ctx, val)
        }
    }
    unsafe impl SafeFunction<(i32, &'static str)> for HandleSome {}

    struct HandleNone;
    impl Function<&'static str> for HandleNone {
        type Output = String;
        fn call(ctx: &'static str) -> String {
            format!("{}:none", ctx)
        }
    }
    unsafe impl SafeFunction<&'static str> for HandleNone {}

    type TestOpt = BranchOpt<HandleSome, HandleNone>;
    assert_eq!(TestOpt::call((Some(42), "ctx")), "ctx:42");
    assert_eq!(TestOpt::call((None, "ctx")), "ctx:none");

    struct HandleOk;
    impl Function<(i32, &'static str)> for HandleOk {
        type Output = String;
        fn call((val, ctx): (i32, &'static str)) -> String {
            format!("ok:{}:{}", ctx, val)
        }
    }
    unsafe impl SafeFunction<(i32, &'static str)> for HandleOk {}

    struct HandleErr;
    impl Function<(bool, &'static str)> for HandleErr {
        type Output = String;
        fn call((err, ctx): (bool, &'static str)) -> String {
            format!("err:{}:{}", ctx, err)
        }
    }
    unsafe impl SafeFunction<(bool, &'static str)> for HandleErr {}

    type TestRes = BranchRes<HandleOk, HandleErr>;
    assert_eq!(TestRes::call((Ok(10), "tag")), "ok:tag:10");
    assert_eq!(TestRes::call((Err(true), "tag")), "err:tag:true");

    // BranchResPure / IsSome / IsNone / IsOk / IsErr
    assert!(IsSome::call(&Some(1)));
    assert!(!IsSome::call(&None::<i32>));
    assert!(IsNone::call(None::<i32>));
    assert!(IsNone::call(&None::<i32>));
    assert!(!IsNone::call(&Some(1)));
    assert!(IsOk::call(Ok::<i32, &str>(1)));
    assert!(!IsOk::call(Err::<i32, &str>("e")));
    assert!(IsOk::call(&Ok::<i32, &str>(1)));
    assert!(IsErr::call(Err::<i32, &str>("e")));
    assert!(!IsErr::call(Ok::<i32, &str>(1)));
    assert!(IsErr::call(&Err::<i32, &str>("e")));

    type CheckOkPositive = Then<ResAsRef, BranchResPure<IsPositive, Always<False>>>;
    assert!(CheckOkPositive::call(&Ok::<i32, &str>(5)));
    assert!(!CheckOkPositive::call(&Ok::<i32, &str>(-1)));
    assert!(!CheckOkPositive::call(&Err::<i32, &str>("err")));

    // IfElse short-circuit branching
    struct Negate;
    impl Function<i32> for Negate {
        type Output = i32;
        fn call(x: i32) -> i32 {
            -x
        }
    }
    unsafe impl SafeFunction<i32> for Negate {}
    type Abs = IfElse<IsPositive, Id, Negate>;
    assert_eq!(Abs::call(7), 7);
    assert_eq!(Abs::call(-7), 7);
    assert_eq!(Abs::call(0), 0);
}

// ==========================================
// 2. Non-Copy exclusive ownership transfer (linear SMC routing)
// ==========================================

struct First(Box<u32>);
struct Second(Box<u16>);

struct TaskOwned {
    first: Option<First>,
    second: Option<Second>,
    expected: i64,
}
unsafe impl SafeValue for TaskOwned {
    type Base = Self;
    fn as_base(&self) -> &Self {
        self
    }
}

struct ReadFirst;
impl Function<First> for ReadFirst {
    type Output = u32;
    fn call(value: First) -> u32 {
        *value.0
    }
}
unsafe impl SafeFunction<First> for ReadFirst {}

struct ReadSecond;
impl Function<Second> for ReadSecond {
    type Output = u16;
    fn call(value: Second) -> u16 {
        *value.0
    }
}
unsafe impl SafeFunction<Second> for ReadSecond {}

struct Diff<L, R>(PhantomData<fn(L, R)>);
impl<A, B, L, R> Function<(A, B)> for Diff<L, R>
where
    L: SafeFunction<A, Output = u32>,
    R: SafeFunction<B, Output = u16>,
{
    type Output = i64;
    fn call((a, b): (A, B)) -> i64 {
        i64::from(L::call(a)) - i64::from(R::call(b))
    }
}
unsafe impl<A, B, L, R> SafeFunction<(A, B)> for Diff<L, R>
where
    L: SafeFunction<A, Output = u32>,
    R: SafeFunction<B, Output = u16>,
{
}

struct TakeFirst;
impl Function<TaskOwned> for TakeFirst {
    type Output = (Option<First>, (Option<Second>, i64));
    fn call(task: TaskOwned) -> Self::Output {
        (task.first, (task.second, task.expected))
    }
}
unsafe impl SafeFunction<TaskOwned> for TakeFirst {}

struct TakeSecond;
impl Function<(First, (Option<Second>, i64))> for TakeSecond {
    type Output = (Option<Second>, (First, i64));
    fn call((first, (second, expected)): (First, (Option<Second>, i64))) -> Self::Output {
        (second, (first, expected))
    }
}
unsafe impl SafeFunction<(First, (Option<Second>, i64))> for TakeSecond {}

struct OwnedFalse;
impl Function<(Option<Second>, i64)> for OwnedFalse {
    type Output = bool;
    fn call(_: (Option<Second>, i64)) -> bool {
        false
    }
}
unsafe impl SafeFunction<(Option<Second>, i64)> for OwnedFalse {}

impl Function<(First, i64)> for OwnedFalse {
    type Output = bool;
    fn call(_: (First, i64)) -> bool {
        false
    }
}
unsafe impl SafeFunction<(First, i64)> for OwnedFalse {}

type Route = Then<AssocLeft, MapFst<Swap>>;
type OwnedCondition = PairEq<Diff<ReadFirst, ReadSecond>, Id>;
type OwnedPredicate =
    MatchOpt<TakeFirst, MatchOpt<TakeSecond, Then<Route, OwnedCondition>, OwnedFalse>, OwnedFalse>;

#[test]
fn owned_nested_non_copy_payloads() {
    let make_task = |a: Option<u32>, b: Option<u16>, expected: i64| TaskOwned {
        first: a.map(|v| First(Box::new(v))),
        second: b.map(|v| Second(Box::new(v))),
        expected,
    };

    assert!(OwnedPredicate::call(make_task(Some(10), Some(20), -10)));
    assert!(!OwnedPredicate::call(make_task(Some(10), Some(20), 10)));
    assert!(!OwnedPredicate::call(make_task(None, Some(20), -10)));
    assert!(!OwnedPredicate::call(make_task(Some(10), None, -10)));
}

// ==========================================
// 3. Context-pollution-free borrowed destructuring (Fork + ZipOpt + PipeWith)
// ==========================================

#[derive(Debug, Clone, Copy)]
struct TaskBorrow {
    first: Option<u32>,
    second: Option<u32>,
    expected: u32,
}
unsafe impl SafeValue for TaskBorrow {
    type Base = Self;
    fn as_base(&self) -> &Self {
        self
    }
}
unsafe impl SafeClone for TaskBorrow {}

// Three completely pure, reusable single-field projections with zero context pollution.
struct GetFirst;
impl<'a> Function<&'a TaskBorrow> for GetFirst {
    type Output = Option<u32>;
    fn call(task: &'a TaskBorrow) -> Option<u32> {
        task.first
    }
}
unsafe impl SafeFunction<&TaskBorrow> for GetFirst {}

struct GetSecond;
impl<'a> Function<&'a TaskBorrow> for GetSecond {
    type Output = Option<u32>;
    fn call(task: &'a TaskBorrow) -> Option<u32> {
        task.second
    }
}
unsafe impl SafeFunction<&TaskBorrow> for GetSecond {}

struct GetExpected;
impl<'a> Function<&'a TaskBorrow> for GetExpected {
    type Output = u32;
    fn call(task: &'a TaskBorrow) -> u32 {
        task.expected
    }
}
unsafe impl SafeFunction<&TaskBorrow> for GetExpected {}

struct AddPair;
impl Function<(u32, u32)> for AddPair {
    type Output = u32;
    fn call((a, b): (u32, u32)) -> u32 {
        a + b
    }
}
unsafe impl SafeFunction<(u32, u32)> for AddPair {}

// Expressed directly via Fork + ZipOpt + PipeWith, eliminating manual AssocLeft / Swap and context wrapping.
type ZipBoth = Then<Fork<GetFirst, GetSecond>, ZipOpt>;
type BorrowedPredicate = SomeElseFalse<PipeWith<ZipBoth, Id>, PairEq<AddPair, GetExpected>>;

#[test]
fn clean_borrowed_subset_predicate_with_fork_and_zip() {
    type ValidTask = Subset<BorrowedPredicate, TaskBorrow>;

    let valid = TaskBorrow {
        first: Some(10),
        second: Some(20),
        expected: 30,
    };
    assert!(ValidTask::try_new(valid).is_ok());

    let invalid = TaskBorrow {
        first: Some(10),
        second: Some(20),
        expected: 35,
    };
    assert!(ValidTask::try_new(invalid).is_err());

    let missing = TaskBorrow {
        first: None,
        second: Some(20),
        expected: 30,
    };
    assert!(ValidTask::try_new(missing).is_err());
}

// ==========================================
// 4. Shortcut predicates and zero-copy destructuring of Option<NonCopy> via OptAsRef
// ==========================================

#[test]
fn shortcut_predicates_with_attach_unit_and_opt_as_ref() {
    struct CheckPositive;
    impl Function<(i32, ())> for CheckPositive {
        type Output = bool;
        fn call((x, ()): (i32, ())) -> bool {
            x > 0
        }
    }
    unsafe impl SafeFunction<(i32, ())> for CheckPositive {}

    // AttachUnit completely replaces hand-written TargetOpt / TargetRes
    type RequirePositive = SomeElseFalse<AttachUnit, CheckPositive>;
    assert!(RequirePositive::call(Some(5)));
    assert!(!RequirePositive::call(Some(-5)));
    assert!(!RequirePositive::call(None));

    type AllowNoneIfPositive = SomeElseTrue<AttachUnit, CheckPositive>;
    assert!(AllowNoneIfPositive::call(Some(5)));
    assert!(!AllowNoneIfPositive::call(Some(-5)));
    assert!(AllowNoneIfPositive::call(None));

    type RequireOk = OkElseFalse<AttachUnit, CheckPositive>;
    assert!(RequireOk::call(Ok::<i32, ()>(5)));
    assert!(!RequireOk::call(Ok::<i32, ()>(-5)));
    assert!(!RequireOk::call(Err::<i32, ()>(())));

    // OptAsRef and UnwrapSome in pure core
    type NonEmptyOptArr = Subset<Then<OptAsRef, BranchOptPure<IsNonEmpty, False>>, Option<[u8; 2]>>;
    assert!(NonEmptyOptArr::try_new(Some([1, 2])).is_ok());
    assert!(NonEmptyOptArr::try_new(None).is_err());

    let some_num = Subset::<IsSome, Option<i32>>::try_new(Some(7)).unwrap();
    assert_eq!(*UnwrapSome::call(&some_num), 7);
    assert_eq!(UnwrapSome::call(some_num), 7);

    // OptAsRef directly supports zero-copy validation of Subset<..., Option<String>> (alloc)
    #[cfg(feature = "alloc")]
    {
        type NonEmptyOptString =
            Subset<Then<OptAsRef, BranchOptPure<IsNonEmpty, False>>, Option<String>>;
        assert!(NonEmptyOptString::try_new(Some(String::from("ok"))).is_ok());
        assert!(NonEmptyOptString::try_new(Some(String::new())).is_err());
        assert!(NonEmptyOptString::try_new(None).is_err());

        // Branch-free refined extraction via UnwrapSome / UnwrapOk / UnwrapErr
        let some_str =
            Subset::<IsSome, Option<String>>::try_new(Some(String::from("hello"))).unwrap();
        assert_eq!(UnwrapSome::call(&some_str), "hello");
        assert_eq!(UnwrapSome::call(some_str), "hello");
        assert!(Subset::<IsSome, Option<String>>::try_new(None).is_err());
    }

    let ok_val = Subset::<IsOk, Result<i32, &str>>::try_new(Ok(42)).unwrap();
    assert_eq!(*UnwrapOk::call(&ok_val), 42);
    assert_eq!(UnwrapOk::call(ok_val), 42);

    let err_val = Subset::<IsErr, Result<i32, &str>>::try_new(Err("boom")).unwrap();
    assert_eq!(*UnwrapErr::call(&err_val), "boom");
    assert_eq!(UnwrapErr::call(err_val), "boom");

    // Nested refinement: Subset<Then<UnwrapSome, IsPositive>, Subset<IsSome, Option<i32>>>
    type PositiveSome = Subset<Then<UnwrapSome, IsPositive>, Subset<IsSome, Option<i32>>>;
    let s_pos = Subset::<IsSome, Option<i32>>::try_new(Some(7)).unwrap();
    let s_neg = Subset::<IsSome, Option<i32>>::try_new(Some(-3)).unwrap();
    assert!(PositiveSome::try_new(s_pos).is_ok());
    assert!(PositiveSome::try_new(s_neg).is_err());
}

// ==========================================
// 5. Fst, Snd, Discard, and Always for non-Copy types
// ==========================================

#[test]
fn non_copy_wiring_and_always() {
    struct LogOnDrop(String);
    impl Drop for LogOnDrop {
        fn drop(&mut self) {}
    }
    unsafe impl SafeDrop for LogOnDrop {}

    let a = LogOnDrop("left".into());
    let b = LogOnDrop("right".into());
    assert_eq!(Fst::call((a, b)).0, "left");

    let c = LogOnDrop("left2".into());
    let d = LogOnDrop("right2".into());
    assert_eq!(Snd::call((c, d)).0, "right2");

    Discard::call(LogOnDrop("dropped".into()));

    fn assert_pure_function<I, F: SafeFunction<I>>() {}
    assert_pure_function::<(u32, LogOnDrop), Fst>();
    assert_pure_function::<(LogOnDrop, u32), Snd>();
    assert_pure_function::<LogOnDrop, Discard>();

    struct TenBox;
    impl Function<()> for TenBox {
        type Output = Box<u32>;
        fn call((): ()) -> Box<u32> {
            Box::new(10)
        }
    }
    unsafe impl SafeFunction<()> for TenBox {}

    assert_eq!(*Always::<TenBox>::call(123), 10);
    assert_eq!(*Always::<TenBox>::call("ignored"), 10);
    assert!(!Always::<False>::call(999));
    assert!(Always::<True>::call(&"anything"));
}

// ==========================================
// 6. ZipRes / ResOk / ResErr and custom enum variant combinators
// ==========================================

#[test]
fn zip_res_and_result_option_bridges() {
    // Left-biased error semantics of ZipRes
    assert_eq!(
        ZipRes::call((Ok::<i32, &str>(1), Ok::<i32, &str>(2))),
        Ok((1, 2))
    );
    assert_eq!(
        ZipRes::call((Err::<i32, &str>("left"), Ok::<i32, &str>(2))),
        Err("left")
    );
    assert_eq!(
        ZipRes::call((Ok::<i32, &str>(1), Err::<i32, &str>("right"))),
        Err("right")
    );
    assert_eq!(
        ZipRes::call((Err::<i32, &str>("left"), Err::<i32, &str>("right"))),
        Err("left")
    );

    // Owned and borrowed forms of ResOk / ResErr
    let ok: Result<String, i32> = Ok(String::from("data"));
    let err: Result<String, i32> = Err(404);
    assert_eq!(ResOk::call(&ok).map(String::as_str), Some("data"));
    assert_eq!(ResOk::call(&err), None);
    assert_eq!(ResErr::call(&ok), None);
    assert_eq!(ResErr::call(&err), Some(&404));
    assert_eq!(ResOk::call(ok), Some(String::from("data")));
    assert_eq!(ResErr::call(err), Some(404));
}

#[test]
fn custom_enum_is_variant_if_let_else_and_unwrap_variant() {
    #[derive(Debug, PartialEq, Eq)]
    enum Event {
        Click { x: i32, y: i32 },
        Code(u32),
        Pair(i32, i32),
        Tag(&'static str),
        Quit,
    }
    impl_safe_value!(enum Event {
        Click { x, y },
        Code(c),
        Pair(a, b),
        Tag(t),
        Quit,
    });

    variant_projector!(AsClick: &Event => Click { x: i32, y: i32 });
    variant_projector!(AsClickRef: &Event => Click { x: &i32, y: &i32 });
    variant_projector!(AsCode: &Event => Code(u32));
    variant_projector!(AsPair: &Event => Pair(a: i32, b: i32));
    variant_projector!(AsPairRef: &Event => Pair(a: &i32, b: &i32));
    variant_projector!(AsTag: &Event => Tag(&'static str));
    variant_projector!(AsQuit: &Event => Quit);

    // 1. Branch-free extraction via IsVariant<V> and UnwrapVariant<V>
    type ClickEvent = Subset<IsVariant<AsClick>, Event>;
    type CodeEvent = Subset<IsVariant<AsCode>, Event>;
    type PairEvent = Subset<IsVariant<AsPair>, Event>;
    type TagEvent = Subset<IsVariant<AsTag>, Event>;
    type QuitEvent = Subset<IsVariant<AsQuit>, Event>;

    let click = ClickEvent::try_new(Event::Click { x: 10, y: 20 }).unwrap();
    assert_eq!(UnwrapVariant::<AsClick>::call(&click), (10, 20));
    assert_eq!(AsClickRef::call(&click), Some((&10, &20)));
    assert!(ClickEvent::try_new(Event::Quit).is_err());

    let code = CodeEvent::try_new(Event::Code(200)).unwrap();
    assert_eq!(UnwrapVariant::<AsCode>::call(&code), 200);

    let pair = PairEvent::try_new(Event::Pair(3, 4)).unwrap();
    assert_eq!(UnwrapVariant::<AsPair>::call(&pair), (3, 4));
    assert_eq!(AsPairRef::call(&pair), Some((&3, &4)));

    let tag = TagEvent::try_new(Event::Tag("v1")).unwrap();
    assert_eq!(UnwrapVariant::<AsTag>::call(&tag), "v1");

    let quit = QuitEvent::try_new(Event::Quit).unwrap();
    assert_eq!(UnwrapVariant::<AsQuit>::call(&quit), ());

    // All variant projectors must safely return None when given non-matching variants
    assert!(AsClick::call(&Event::Quit).is_none());
    assert!(AsClickRef::call(&Event::Quit).is_none());
    assert!(AsCode::call(&Event::Quit).is_none());
    assert!(AsPair::call(&Event::Quit).is_none());
    assert!(AsPairRef::call(&Event::Quit).is_none());
    assert!(AsTag::call(&Event::Quit).is_none());
    assert!(AsQuit::call(&Event::Code(1)).is_none());

    // Nested refinement: refine TagEvent directly via Then<UnwrapVariant<AsTag>, IsNonEmpty>
    type NonEmptyTagEvent = Subset<Then<UnwrapVariant<AsTag>, IsNonEmpty>, TagEvent>;
    let empty_tag = TagEvent::try_new(Event::Tag("")).unwrap();
    assert!(NonEmptyTagEvent::try_new(tag).is_ok());
    assert!(NonEmptyTagEvent::try_new(empty_tag).is_err());

    // 2. IfLetElseFalse / IfLetElseTrue and chained IfLetElse
    type ClickInFirstQuadrant = IfLetElseFalse<
        AsClick,
        fy_refine_core::function::predicate::And<Then<Fst, IsPositive>, Then<Snd, IsPositive>>,
    >;
    assert!(ClickInFirstQuadrant::call(&Event::Click { x: 3, y: 4 }));
    assert!(!ClickInFirstQuadrant::call(&Event::Click { x: -1, y: 4 }));
    assert!(!ClickInFirstQuadrant::call(&Event::Quit));

    type TagMustBeNonEmptyIfPresent = IfLetElseTrue<AsTag, IsNonEmpty>;
    assert!(TagMustBeNonEmptyIfPresent::call(&Event::Tag("hi")));
    assert!(!TagMustBeNonEmptyIfPresent::call(&Event::Tag("")));
    assert!(TagMustBeNonEmptyIfPresent::call(&Event::Quit));

    // Chained: if let Click => x > 0 && y > 0; else if let Tag => !t.is_empty(); else => true (Quit)
    type ValidEventPred = IfLetElse<
        AsClick,
        fy_refine_core::function::predicate::And<Then<Fst, IsPositive>, Then<Snd, IsPositive>>,
        IfLetElse<AsTag, IsNonEmpty, Always<True>>,
    >;
    type ValidEvent = Subset<ValidEventPred, Event>;
    assert!(ValidEvent::try_new(Event::Click { x: 1, y: 2 }).is_ok());
    assert!(ValidEvent::try_new(Event::Click { x: 0, y: 2 }).is_err());
    assert!(ValidEvent::try_new(Event::Tag("ok")).is_ok());
    assert!(ValidEvent::try_new(Event::Tag("")).is_err());
    assert!(ValidEvent::try_new(Event::Quit).is_ok());

    #[cfg(feature = "alloc")]
    {
        #[derive(Debug, PartialEq, Eq)]
        enum AllocEvent {
            Text(String),
            Quit,
        }
        impl_safe_value!(
            enum AllocEvent {
                Text(s),
                Quit,
            }
        );
        variant_projector!(AsText: &AllocEvent => Text(&String));

        type TextEvent = Subset<IsVariant<AsText>, AllocEvent>;
        type NonEmptyTextEvent = Subset<Then<UnwrapVariant<AsText>, IsNonEmpty>, TextEvent>;

        let text = TextEvent::try_new(AllocEvent::Text(String::from("hello"))).unwrap();
        assert_eq!(UnwrapVariant::<AsText>::call(&text), "hello");
        let empty_text = TextEvent::try_new(AllocEvent::Text(String::new())).unwrap();
        assert!(NonEmptyTextEvent::try_new(text).is_ok());
        assert!(NonEmptyTextEvent::try_new(empty_text).is_err());

        type TextMustBeNonEmpty = IfLetElseTrue<AsText, IsNonEmpty>;
        assert!(TextMustBeNonEmpty::call(&AllocEvent::Text(String::from(
            "hi"
        ))));
        assert!(!TextMustBeNonEmpty::call(&AllocEvent::Text(String::new())));
        assert!(TextMustBeNonEmpty::call(&AllocEvent::Quit));
    }
}

#[cfg(feature = "nightly")]
#[test]
fn nightly_const_unwrap_variant_and_zip_res() {
    #[derive(Copy, Clone)]
    enum ConstMsg {
        Num(i32),
        Ping,
    }
    impl_safe_value!(const enum ConstMsg { Num(v), Ping });
    variant_projector!(const AsConstNum: &ConstMsg => Num(i32));
    variant_projector!(const AsConstPing: &ConstMsg => Ping);

    const VAL: Result<i32, i32> = Ok(99);
    const EXTRACTED: i32 = match Subset::<IsVariant<ResOk>, Result<i32, i32>>::try_from_ref(&VAL) {
        Some(v) => *UnwrapVariant::<ResOk>::call(v),
        None => 0,
    };
    const ZIPPED: Result<(i32, i32), i32> = ZipRes::call((Ok(1), Ok(2)));

    assert_eq!(EXTRACTED, 99);
    assert_eq!(ZIPPED, Ok((1, 2)));

    const WITH_REF_CHECK: bool = WithRef::<All<IsPositive>>::call([1i32, 2, 3]);
    const SOME_AND: bool = SomeAnd::<IsPositive>::call(Some(10i32));
    const NONE_OR: bool = NoneOr::<IsPositive>::call(None::<i32>);
    const MSG_NUM: bool = matches!(AsConstNum::call(&ConstMsg::Num(7)), Some(7));
    const MSG_PING: bool = matches!(AsConstPing::call(&ConstMsg::Ping), Some(()));
    const {
        assert!(WITH_REF_CHECK);
        assert!(SOME_AND);
        assert!(NONE_OR);
        assert!(MSG_NUM);
        assert!(MSG_PING);
    }
}

#[test]
fn with_ref_and_pure_branch_shortcut_predicates() {
    // 1. WithRef bridges an owned array output to a borrowed downstream predicate (pure core)
    struct MakeNums;
    impl Function<bool> for MakeNums {
        type Output = [i32; 3];
        fn call(all_pos: bool) -> [i32; 3] {
            if all_pos { [1, 2, 3] } else { [1, -2, 3] }
        }
    }
    unsafe impl SafeFunction<bool> for MakeNums {}

    type ProducedAllPositive = Then<MakeNums, WithRef<All<IsPositive>>>;
    assert!(ProducedAllPositive::call(true));
    assert!(!ProducedAllPositive::call(false));

    // 2. Context-free Option / Result shortcut predicate aliases (pure core)
    type NonEmptyOptArr = Subset<Then<OptAsRef, SomeAnd<IsNonEmpty>>, Option<[u8; 2]>>;
    assert!(NonEmptyOptArr::try_new(Some([1, 2])).is_ok());
    assert!(NonEmptyOptArr::try_new(None).is_err());
    assert!(!Then::<OptAsRef, SomeAnd<IsNonEmpty>>::call(&Some(
        [0u8; 0]
    )));

    type EmptyOrFirstPos = Subset<Then<TryFirst, NoneOr<IsPositive>>, [i32]>;
    assert!(EmptyOrFirstPos::try_from_ref(&[]).is_some());
    assert!(EmptyOrFirstPos::try_from_ref(&[1, -5]).is_some());
    assert!(EmptyOrFirstPos::try_from_ref(&[-1, 5]).is_none());

    type OkPos = Then<ResAsRef, OkAnd<IsPositive>>;
    assert!(OkPos::call(&Ok::<i32, &str>(5)));
    assert!(!OkPos::call(&Ok::<i32, &str>(-1)));
    assert!(!OkPos::call(&Err::<i32, &str>("err")));

    type ErrOrPos = Then<ResAsRef, ErrOr<IsPositive>>;
    assert!(ErrOrPos::call(&Ok::<i32, &str>(5)));
    assert!(!ErrOrPos::call(&Ok::<i32, &str>(-1)));
    assert!(ErrOrPos::call(&Err::<i32, &str>("err")));

    type ErrNonEmpty = Then<ResAsRef, ErrAnd<IsNonEmpty>>;
    assert!(ErrNonEmpty::call(&Err::<i32, [u8; 2]>([1, 2])));
    assert!(!ErrNonEmpty::call(&Err::<i32, [u8; 0]>([])));
    assert!(!ErrNonEmpty::call(&Ok::<i32, [u8; 2]>(1)));

    type OkOrErrNonEmpty = Then<ResAsRef, OkOr<IsNonEmpty>>;
    assert!(OkOrErrNonEmpty::call(&Ok::<i32, [u8; 0]>(1)));
    assert!(OkOrErrNonEmpty::call(&Err::<i32, [u8; 2]>([1, 2])));
    assert!(!OkOrErrNonEmpty::call(&Err::<i32, [u8; 0]>([])));

    assert!(MatchOptPure::<OptAsRef, IsNonEmpty, True>::call(
        &None::<[u8; 2]>
    ));
    assert!(MatchResPure::<ResAsRef, IsPositive, Always<False>>::call(
        &Ok::<i32, &str>(3)
    ));

    // 3. Owned String WithRef and shortcut predicates under alloc
    #[cfg(feature = "alloc")]
    {
        struct MakeStr;
        impl Function<bool> for MakeStr {
            type Output = String;
            fn call(ascii: bool) -> String {
                if ascii {
                    String::from("hello")
                } else {
                    String::from("你好")
                }
            }
        }
        unsafe impl SafeFunction<bool> for MakeStr {}

        type ProducedIsAscii = Then<MakeStr, WithRef<Then<AsBytes, All<IsAscii>>>>;
        assert!(ProducedIsAscii::call(true));
        assert!(!ProducedIsAscii::call(false));
        assert!(WithRef::<Then<AsBytes, All<IsAscii>>>::call(String::from(
            "fy"
        )));
        assert!(!WithRef::<Then<AsBytes, All<IsAscii>>>::call(String::from(
            "风"
        )));

        type NonEmptyOptStr = Subset<Then<OptAsRef, SomeAnd<IsNonEmpty>>, Option<String>>;
        assert!(NonEmptyOptStr::try_new(Some(String::from("ok"))).is_ok());
        assert!(NonEmptyOptStr::try_new(Some(String::new())).is_err());
        assert!(NonEmptyOptStr::try_new(None).is_err());

        assert!(ErrNonEmpty::call(&Err::<i32, String>(String::from("boom"))));
        assert!(!ErrNonEmpty::call(&Err::<i32, String>(String::new())));
        assert!(!ErrNonEmpty::call(&Ok::<i32, String>(1)));

        assert!(OkOrErrNonEmpty::call(&Ok::<i32, String>(1)));
        assert!(OkOrErrNonEmpty::call(&Err::<i32, String>(String::from(
            "boom"
        ))));
        assert!(!OkOrErrNonEmpty::call(&Err::<i32, String>(String::new())));

        assert!(MatchOptPure::<OptAsRef, IsNonEmpty, True>::call(
            &None::<String>
        ));
    }
}

#[test]
fn classify_refine_if_and_dependent_logic() {
    // 1. `Classify<P>` zero-copy partitions `&T` into `Ok(&Subset<P, T>)` / `Err(&Subset<Not<P>, T>)`
    let nan = f32::NAN;
    let neg_val = -2.5f32;
    assert!(Classify::<IsNaN>::call(&nan).is_ok());
    let not_nan_ref = Classify::<IsNaN>::call(&neg_val).unwrap_err();
    assert_eq!(**not_nan_ref, -2.5f32);
    assert!(!IsPositive::call(not_nan_ref));

    // 2. `OrElse<IsNaN, IsPositive>`: either NaN, or positive once narrowed to non-NaN
    type NaNOrPositive = Subset<OrElse<IsNaN, IsPositive>, f32>;
    assert!(NaNOrPositive::try_new(f32::NAN).is_ok());
    assert!(NaNOrPositive::try_new(3.5).is_ok());
    assert!(NaNOrPositive::try_new(f32::INFINITY).is_ok());
    assert!(NaNOrPositive::try_new(0.0).is_err());
    assert!(NaNOrPositive::try_new(-0.0).is_err());
    assert!(NaNOrPositive::try_new(-1.0).is_err());

    // 3. `AndThen<IsNotNaN, IsPositive>`: single-layer `Subset` requiring non-NaN positive `f32`
    type PosF32 = Subset<AndThen<IsNotNaN, IsPositive>, f32>;
    assert!(PosF32::try_new(1.25).is_ok());
    assert!(PosF32::try_new(0.0).is_err());
    assert!(PosF32::try_new(-0.5).is_err());
    assert!(PosF32::try_new(f32::NAN).is_err());

    // 4. Four-tier proof-carrying chain on raw `f32`: `IsNotNaN` -> `IsFinite` -> `IsInteger` -> `IsOdd`
    type OddFloat = Subset<AndThen<IsNotNaN, AndThen<IsFinite, AndThen<IsInteger, IsOdd>>>, f32>;
    assert!(OddFloat::try_new(3.0).is_ok());
    assert!(OddFloat::try_new(-5.0).is_ok());
    assert!(OddFloat::try_new(2.0).is_err());
    assert!(OddFloat::try_new(1.5).is_err());
    assert!(OddFloat::try_new(f32::NAN).is_err());
    assert!(OddFloat::try_new(f32::INFINITY).is_err());

    // 5. Multi-stage dependent refinement: finite float that is odd if integer, or positive if fractional
    type OddIntOrPosFrac =
        Subset<AndThen<IsNotNaN, AndThen<IsFinite, RefineIf<IsInteger, IsOdd, IsPositive>>>, f32>;
    assert!(OddIntOrPosFrac::try_new(-3.0).is_ok());
    assert!(OddIntOrPosFrac::try_new(7.0).is_ok());
    assert!(OddIntOrPosFrac::try_new(2.5).is_ok());
    assert!(OddIntOrPosFrac::try_new(0.25).is_ok());
    assert!(OddIntOrPosFrac::try_new(2.0).is_err());
    assert!(OddIntOrPosFrac::try_new(0.0).is_err());
    assert!(OddIntOrPosFrac::try_new(-1.5).is_err());
    assert!(OddIntOrPosFrac::try_new(f32::NAN).is_err());
    assert!(OddIntOrPosFrac::try_new(f32::INFINITY).is_err());

    type NotNaNF32 = fy_refine_core::subset::NotNaNF32;
    type FiniteF32 = fy_refine_core::subset::FiniteF32;
    assert!(RefineIf::<IsFinite, IsPositive, IsPositive>::call(
        &NotNaNF32::try_new(f32::INFINITY).unwrap()
    ));
    assert!(RefineIf::<IsInteger, IsOdd, IsPositive>::call(
        &FiniteF32::try_new(NotNaNF32::try_new(2.5).unwrap()).unwrap()
    ));

    // 6. Combined with primitive integer parity
    assert!(RefineIf::<IsPositive, IsEven, IsOdd>::call(&4i32));
    assert!(!RefineIf::<IsPositive, IsEven, IsOdd>::call(&3i32));
    assert!(RefineIf::<IsPositive, IsEven, IsOdd>::call(&-3i32));
}

#[cfg(feature = "nightly")]
#[test]
fn nightly_const_refine_if_and_dependent_logic() {
    type NaNOrPositive = Subset<OrElse<IsNaN, IsPositive>, f32>;
    type OddFloat = Subset<AndThen<IsNotNaN, AndThen<IsFinite, AndThen<IsInteger, IsOdd>>>, f32>;
    type OddIntOrPosFrac =
        Subset<AndThen<IsNotNaN, AndThen<IsFinite, RefineIf<IsInteger, IsOdd, IsPositive>>>, f32>;

    const NAN_OK: bool = NaNOrPositive::try_new(f32::NAN).is_ok();
    const POS_OK: bool = NaNOrPositive::try_new(3.5).is_ok();
    const NEG_ERR: bool = NaNOrPositive::try_new(-1.0).is_err();
    const ODD_OK: bool = OddFloat::try_new(-5.0).is_ok();
    const EVEN_ERR: bool = OddFloat::try_new(2.0).is_err();
    const COMPOSITE_OK: bool =
        OddIntOrPosFrac::try_new(-3.0).is_ok() && OddIntOrPosFrac::try_new(2.5).is_ok();
    const COMPOSITE_ERR: bool =
        OddIntOrPosFrac::try_new(2.0).is_err() && OddIntOrPosFrac::try_new(-1.5).is_err();

    const {
        assert!(NAN_OK);
        assert!(POS_OK);
        assert!(NEG_ERR);
        assert!(ODD_OK);
        assert!(EVEN_ERR);
        assert!(COMPOSITE_OK);
        assert!(COMPOSITE_ERR);
    }
}
