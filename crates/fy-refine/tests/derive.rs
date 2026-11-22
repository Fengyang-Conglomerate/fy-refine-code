//! Integration tests for `#[derive(SafeValue, SafeDrop, SafeClone, Projectors)]` on the facade crate.
#![cfg_attr(feature = "nightly", feature(const_trait_impl))]

use fy_refine::Subset;
use fy_refine::function::Function;
use fy_refine::function::bitwise::{HasAllBits, HasNoBits};
use fy_refine::function::branch::{IfLetElse, IsVariant, UnwrapVariant};
use fy_refine::function::collection::{All, IsNonEmpty, LenGt};
use fy_refine::function::constant::ConstU32;
use fy_refine::function::predicate::{
    AllOf, Closed, ForkLt, IsAscii, IsPositive, Open, Range, True,
};
use fy_refine::function::slice::AsBytes;
use fy_refine::function::wiring::{Always, Fst, Snd, Then};
use fy_refine::{Projectors, SafeClone, SafeDrop, SafeValue};

#[test]
fn derive_named_struct_and_submodule_projectors() {
    type PosU32 = Subset<IsPositive, u32>;

    #[derive(Debug, PartialEq, Eq, SafeValue, SafeDrop, SafeClone, Projectors)]
    struct PacketWindow {
        #[projector(copy)]
        lo: PosU32,
        #[projector(copy)]
        hi: PosU32,
        #[projector(copy)]
        flags: u32,
        #[projector(copy)]
        label: &'static str,
        #[projector(skip)]
        _meta: u8,
    }

    type FlagMask = AllOf<(HasAllBits<ConstU32<0x01>>, HasNoBits<ConstU32<0xF000>>)>;
    type ValidWindowPred = AllOf<(
        ForkLt<packet_window::Lo, packet_window::Hi>,
        Then<packet_window::Flags, FlagMask>,
        Then<packet_window::Label, LenGt<1>>,
    )>;

    struct ValidWindow(Subset<ValidWindowPred, PacketWindow>);

    let ok = ValidWindow(
        Subset::try_new(PacketWindow {
            lo: PosU32::try_new(10).unwrap(),
            hi: PosU32::try_new(20).unwrap(),
            flags: 0x0003,
            label: "hdr",
            _meta: 0,
        })
        .unwrap(),
    );
    let cloned = ok.0.clone();
    assert_eq!(*packet_window::Lo::call(&cloned), 10);
    assert_eq!(packet_window::Flags::call(&cloned), 0x0003);
    assert_eq!(packet_window::Label::call(&cloned), "hdr");

    // Reject frame when reserved bits 0xF000 are set
    assert!(
        Subset::<ValidWindowPred, PacketWindow>::try_new(PacketWindow {
            lo: PosU32::try_new(10).unwrap(),
            hi: PosU32::try_new(20).unwrap(),
            flags: 0x1001,
            label: "hdr",
            _meta: 0,
        })
        .is_err()
    );
}

#[test]
fn derive_tuple_unit_and_enum_projectors() {
    #[derive(Copy, Debug, PartialEq, Eq, SafeValue, SafeDrop, SafeClone, Projectors)]
    struct Bounds(#[projector(copy)] u32, u32);

    #[derive(Copy, Debug, PartialEq, Eq, SafeValue, SafeDrop, SafeClone, Projectors)]
    struct PingUnit;

    #[derive(Debug, PartialEq, Eq, SafeValue, SafeDrop, SafeClone, Projectors)]
    #[refine(mod = cmd)]
    enum Command {
        Resize {
            #[projector(copy)]
            width: u32,
            #[projector(copy)]
            height: u32,
        },
        #[projector(copy)]
        Rename(&'static str),
        Raw {
            data: &'static [u8],
            #[projector(skip)]
            _unused: u8,
        },
        Quit,
    }

    let b = Bounds(3, 9);
    assert_eq!(bounds::F0::call(&b), 3);
    assert_eq!(*bounds::F1::call(&b), 9);

    let p = PingUnit;
    assert_eq!(p.clone(), PingUnit);

    type DimOk = Range<Open<ConstU32<0>>, Closed<ConstU32<4096>>>;
    type ValidResize = AllOf<(Then<Fst, DimOk>, Then<Snd, DimOk>)>;
    type ValidName = Then<AsBytes, AllOf<(IsNonEmpty, All<IsAscii>)>>;
    type ValidCmd = Subset<
        IfLetElse<cmd::Resize, ValidResize, IfLetElse<cmd::Rename, ValidName, Always<True>>>,
        Command,
    >;

    assert!(
        ValidCmd::try_new(Command::Resize {
            width: 1920,
            height: 1080
        })
        .is_ok()
    );
    assert!(
        ValidCmd::try_new(Command::Resize {
            width: 0,
            height: 1080
        })
        .is_err()
    );
    assert!(ValidCmd::try_new(Command::Rename("main")).is_ok());
    assert!(ValidCmd::try_new(Command::Rename("")).is_err());
    assert!(ValidCmd::try_new(Command::Quit).is_ok());

    let resized = Subset::<IsVariant<cmd::Resize>, Command>::try_new(Command::Resize {
        width: 800,
        height: 600,
    })
    .unwrap();
    assert_eq!(UnwrapVariant::<cmd::Resize>::call(&resized), (800, 600));

    let raw = Command::Raw {
        data: b"abc",
        _unused: 9,
    };
    assert_eq!(cmd::Raw::call(&raw), Some(&(&b"abc"[..])));
    assert_eq!(cmd::Quit::call(&Command::Quit), Some(()));
}

#[cfg(feature = "nightly")]
#[test]
fn derive_nightly_const_evaluation() {
    #[derive(Copy, Clone, Debug, PartialEq, Eq, SafeValue, SafeDrop, Projectors)]
    #[refine(const)]
    struct ConstRange {
        #[projector(copy)]
        start: i32,
        #[projector(copy)]
        end: i32,
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq, SafeValue, Projectors)]
    #[refine(const)]
    enum ConstEvent {
        #[projector(copy)]
        Value(i32),
        Done,
    }

    type Ordered = Subset<ForkLt<const_range::Start, const_range::End>, ConstRange>;
    const OK: Result<Ordered, ConstRange> = Ordered::try_new(ConstRange { start: 1, end: 8 });
    const BAD: Result<Ordered, ConstRange> = Ordered::try_new(ConstRange { start: 8, end: 1 });
    assert!(OK.is_ok());
    assert!(BAD.is_err());

    const VAL: Option<i32> = const_event::Value::call(&ConstEvent::Value(42));
    const DONE: Option<()> = const_event::Done::call(&ConstEvent::Done);
    const NOT_DONE: Option<()> = const_event::Done::call(&ConstEvent::Value(42));
    assert_eq!(VAL, Some(42));
    assert_eq!(DONE, Some(()));
    assert_eq!(NOT_DONE, None);
}

#[test]
fn derive_additional_syntax_and_attributes() {
    #[derive(Debug, PartialEq, Eq, SafeValue, SafeDrop, SafeClone, Projectors)]
    #[refine(crate = ::fy_refine)]
    pub(crate) struct Match {
        #[projector(copy)]
        v: u8,
    }

    #[derive(Copy, Debug, PartialEq, Eq, SafeValue, SafeDrop, SafeClone, Projectors)]
    struct EmptyBrace {}

    #[derive(Projectors)]
    struct ArrowAndGeneric(
        #[projector(skip)] u8,
        Result<u8, u16>,
        #[projector(copy)] fn(u8) -> u8,
    );

    #[derive(Debug, PartialEq, Eq, SafeValue, SafeDrop, SafeClone, Projectors)]
    #[repr(u8)]
    enum MixedEnum {
        Disc = 1,
        #[projector(skip)]
        Hidden(u8),
        TupleMix(
            #[projector(copy)] u8,
            #[projector(ref)] u16,
            #[projector(skip)] u32,
        ),
        NamedMix {
            #[projector(ref)]
            r: u8,
            #[projector(copy)]
            c: u16,
            #[projector(skip)]
            _s: u32,
        },
    }

    let m = Match { v: 7 }.clone();
    assert_eq!(r#match::V::call(&m), 7);

    let e = EmptyBrace {};
    assert_eq!(e.clone(), EmptyBrace {});

    fn inc(x: u8) -> u8 {
        x + 1
    }
    let ag = ArrowAndGeneric(0, Ok(5), inc);
    assert_eq!(ag.0, 0);
    assert_eq!(arrow_and_generic::F1::call(&ag), &Ok::<u8, u16>(5));
    assert_eq!(arrow_and_generic::F2::call(&ag)(9), 10);

    let d = MixedEnum::Disc.clone();
    let h = MixedEnum::Hidden(1).clone();
    let tm = MixedEnum::TupleMix(3, 4, 5).clone();
    let nm = MixedEnum::NamedMix { r: 6, c: 7, _s: 8 }.clone();

    assert_eq!(mixed_enum::Disc::call(&d), Some(()));
    assert_eq!(mixed_enum::Disc::call(&h), None);
    assert_eq!(mixed_enum::TupleMix::call(&tm), Some((3, &4)));
    assert_eq!(mixed_enum::TupleMix::call(&d), None);
    assert_eq!(mixed_enum::NamedMix::call(&nm), Some((&6, 7)));
    assert_eq!(mixed_enum::NamedMix::call(&d), None);
}
