# fy-refine

[![crates.io](https://img.shields.io/crates/v/fy-refine)](https://crates.io/crates/fy-refine)
[![docs.rs](https://docs.rs/fy-refine/badge.svg)](https://docs.rs/fy-refine)
[![CI](https://github.com/Fengyang-Conglomerate/fy-refine-code/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/Fengyang-Conglomerate/fy-refine-code/actions/workflows/ci.yml)
[![Coverage](https://coveralls.io/repos/github/Fengyang-Conglomerate/fy-refine-code/badge.svg?branch=main)](https://coveralls.io/github/Fengyang-Conglomerate/fy-refine-code?branch=main)
[![Rust 1.86+](https://img.shields.io/badge/rust-1.86%2B-blue)](https://www.rust-lang.org/)
[![no_std](https://img.shields.io/badge/no__std-compatible-green)](#cargo-features)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)

English | [简体中文](README.zh-CN.md)

**Validate once at the boundary, encode proofs into the type system, and replace repeated checks, defensive `.unwrap()` calls, and hand-written `unsafe` with static type guarantees.**

`fy-refine` is a `no_std`-compatible refinement type and pure function composition library for Rust. Its core wrapper `Subset<P, T>` uses `#[repr(transparent)]`, stores zero predicate state at runtime, and natively supports dynamically sized type (DST) fat-pointer views such as `&Subset<P, str>` and `&Subset<P, [T]>`.

---

## Core Design: Why Type-Level Function Composition?

Closures in Rust have unnameable types and cannot appear inside struct definitions or generic bounds. Even when values share the same valid numeric range—such as `[0, 100]` for a percentage, an age, or an audio volume—structural equality alone does not prevent domain mix-ups.

`fy-refine` encodes checks as zero-sized **type-level pure functions** `P` and separates two distinct layers:

1. **Structural subset `Subset<P, T>` (value domain)**: certifies that `T` mathematically satisfies predicate `P` while preserving `T`'s exact memory layout;
2. **Nominal newtype `struct Percentage(...)` (domain identity)**: wraps a `Subset` so structurally identical types with different domain meanings cannot be mixed.

---

## 1. Stratified Numeric Domains & Proof-Carrying Branching

In IEEE 754 floating-point arithmetic, `NaN` ("Not a Number") falls outside the ordered reals, breaking reflexivity and trichotomy. Allowing raw `f32`/`f64` to directly participate in total ordering or sign predicates would cause negation (`Not<IsNonPositive>`) to classify `NaN` as positive. Likewise, evaluating parity directly on general floats would cause `Not<IsEven>` (`IsOdd`) to admit fractions and `NaN`.

To guarantee that Boolean negation `Not<P>` always forms a clean semantic complement within each domain, `fy-refine` stratifies floats and integers into a four-tier binary-partition ladder:

1. **Representation layer (raw `f32` / `f64`)**: exposes only `IsNaN` and its transparent alias `IsNotNaN = Not<IsNaN>`, intentionally omitting `SafeEq`, `SafeOrd`, sign, finiteness, integrality, and parity checks (`Subset<IsNonPositive, f32>` and `Subset<IsInteger, f32>` fail to compile);
2. **Extended real domain (`NotNaNF32` / `NotNaNF64`)**: once `NaN` is excluded, carriers unlock exact mathematical total order `SafeOrd` (with `-0.0 == +0.0 == 0` and bit-exact cross-integer comparisons across all 12 integer primitives) and all sign/ordering predicates, together with `IsFinite` and its transparent alias `IsInfinite = Not<IsFinite>` ("not finite" means "infinite" with no `NaN` leakage);
3. **Finite real domain (`FiniteF32 = Subset<IsFinite, NotNaNF32>` / `FiniteF64`)**: once both `NaN` and `±infinity` are excluded, `IsInteger` is unlocked, so its complement `Not<IsInteger>` strictly represents finite non-integers;
4. **Integer domain ($\mathbb{Z}$, primitive integers plus `Subset<IsInteger, FiniteF32/64>`)**: within $\mathbb{Z}$, $\neg\text{Even}(n) \iff \text{Odd}(n)$ holds without exceptions, unlocking `IsEven` and the transparent alias `IsOdd = Not<IsEven>`.

To express cross-domain conditions directly on a raw type (such as "either `NaN` or positive"), use the **proof-partitioning primitive `Classify<P>`** and its transparent combinators **`RefineIf` / `AndThen` / `OrElse`**: `RefineIf<P, Then, Else>` automatically narrows the input to `&Subset<P, T>` on the `true` branch and `&Subset<Not<P>, T>` on the `false` branch.

```rust
use fy_refine::function::Function;
use fy_refine::function::branch::{AndThen, OrElse, RefineIf};
use fy_refine::function::constant::ConstU8;
use fy_refine::function::predicate::{
    Closed, IsFinite, IsInteger, IsNaN, IsNonPositive, IsNotNaN, IsOdd, IsPositive, Range,
};
use fy_refine::{FiniteF64, NotNaNF32, NotNaNF64, SafeEq, SafeOrd, Subset};

// 1. Separate structural domain ([0, 100]) from nominal identity (Percentage)
type U8In0To100 = Subset<Range<Closed<ConstU8<0>>, Closed<ConstU8<100>>>, u8>;
#[derive(Debug, PartialEq, Eq)]
struct Percentage(U8In0To100);

let pct = Percentage(U8In0To100::try_new(85).unwrap());
assert_eq!(*pct.0, 85);

// 2. Stratified domains: narrow `NotNaN -> Finite` before ordering or cross-integer comparisons
type NonPosF32 = Subset<IsNonPositive, NotNaNF32>;
type Probability = Subset<Range<Closed<ConstU8<0>>, Closed<ConstU8<1>>>, FiniteF64>;

let neg_zero = NotNaNF32::try_new(-0.0).unwrap();
assert!(NonPosF32::try_new(neg_zero).is_ok());
assert!(NotNaNF32::try_new(f32::NAN).is_err());

let fin_half = FiniteF64::try_new(NotNaNF64::try_new(0.5).unwrap()).unwrap();
let half = Probability::try_new(fin_half).unwrap();
assert!(SafeEq::eq(&neg_zero, &0u8));
assert!(SafeOrd::gt(&half, &0u8) && SafeOrd::lt(&half, &1u8));

// Bit-exact precision: 2^53 is representable, and 2^53 < 2^53 + 1 avoids float-cast rounding errors
let f_pow53 = FiniteF64::try_new(NotNaNF64::try_new((1u64 << 53) as f64).unwrap()).unwrap();
assert!(SafeEq::eq(&f_pow53, &(1u64 << 53)));
assert!(SafeOrd::lt(&f_pow53, &((1u64 << 53) + 1)));

// 3. Proof-carrying branching (`OrElse` / `AndThen` / `RefineIf`) directly on raw `f32`:
//    when `IsNaN` is false, `Classify` passes `&Subset<Not<IsNaN>, f32>` (= `&NotNaNF32`) to `IsPositive`
type NaNOrPositive = Subset<OrElse<IsNaN, IsPositive>, f32>;
assert!(NaNOrPositive::try_new(f32::NAN).is_ok());
assert!(NaNOrPositive::try_new(3.5).is_ok());
assert!(NaNOrPositive::try_new(-1.0).is_err());

//    Follow the four-tier ladder in one type: non-NaN -> finite -> odd if integer, else positive
type OddIntOrPosFrac =
    Subset<AndThen<IsNotNaN, AndThen<IsFinite, RefineIf<IsInteger, IsOdd, IsPositive>>>, f32>;
assert!(OddIntOrPosFrac::try_new(-3.0).is_ok());
assert!(OddIntOrPosFrac::try_new(2.5).is_ok());
assert!(OddIntOrPosFrac::try_new(2.0).is_err());
assert!(OddIntOrPosFrac::try_new(-1.5).is_err());
assert!(OddIntOrPosFrac::try_new(f32::INFINITY).is_err());
```

---

## 2. Sequences & Containers: Pre-Refinement Probes (`Try*`) & Post-Refinement Indexing (`Get`/`AsStr`)

Containers naturally require different access primitives before and after refinement:

- **Before refinement (inside a predicate)**: the container length is not yet proven, so use fallible probes `TryFirst`, `TryLast`, and `TryGet<I>` (returning `Option<&T>`), paired with `SomeAnd<P>` or `NoneOr<P>` to validate positional constraints;
- **After refinement (downstream consumption)**: once holding `Subset<IsNonEmpty, C>` or `Subset<LenGt<I>, C>`—whether `C` is a borrowed slice `[u8]`, a fixed-size array `[u8; N]`, or an owned `Vec<u8>` / `VecDeque<u8>` (`std` / `alloc`)—`First`, `Last`, and `Get<I>` return `&T` directly with no `Option` and no hand-written `unsafe` (out-of-bounds indices such as `Get::<3>` on `Subset<LenGt<2>, _>` fail to compile);
- **Encoding & Unicode proofs**: likewise, `[u8]`, `[u8; N]`, and `Vec<u8>` certified by `All<IsAscii>` or `IsUtf8` convert to `&str` in $O(1)$ via `AsStr`; a `u32` certified by `IsChar` (excluding surrogates `0xD800..=0xDFFF`) round-trips losslessly with `char` via `U32ToChar` and `CharToU32`.

```rust
use fy_refine::Subset;
use fy_refine::function::Function;
use fy_refine::function::branch::SomeAnd;
use fy_refine::function::collection::{All, First, Get, IsNonEmpty, Last, LenGt, TryGet};
use fy_refine::function::predicate::{
    CharToU32, IsAscii, IsAsciiUppercase, IsChar, IsUtf8, U32ToChar,
};
use fy_refine::function::slice::AsStr;
use fy_refine::function::wiring::Then;

// 1. Pre-refinement check: safely inspect byte 0 inside a predicate using `TryGet<0>` + `SomeAnd`
type StartsWithUpper = Then<TryGet<0>, SomeAnd<IsAsciiUppercase>>;
let header = Subset::<StartsWithUpper, Vec<u8>>::try_new(b"GET /index".to_vec()).unwrap();
assert_eq!(Subset::as_inner(&header).len(), 10);

// 2. Post-refinement branchless access: works uniformly on owned `Vec<u8>` and borrowed `[u8]`
type AtLeast3Vec = Subset<LenGt<2>, Vec<u8>>;
let frame_vec = AtLeast3Vec::try_new(vec![b'A', b'B', b'C']).unwrap();
assert_eq!(*Get::<2>::call(&frame_vec), b'C'); // calling `Get::<3>` on `frame_vec` fails to compile

type NonEmptyBytes = Subset<IsNonEmpty, [u8]>;
let non_empty = NonEmptyBytes::try_from_ref(b"refine").unwrap();
assert_eq!((*First::call(non_empty), *Last::call(non_empty)), (b'r', b'e'));

let utf8_vec = Subset::<IsUtf8, Vec<u8>>::try_new("café🦀".as_bytes().to_vec()).unwrap();
assert_eq!(AsStr::call(&utf8_vec), "café🦀");

let ascii_slice = Subset::<All<IsAscii>, [u8]>::try_from_ref(b"fy-refine").unwrap();
assert_eq!(AsStr::call(ascii_slice), "fy-refine");

// 3. Unicode scalar value conversions (rejecting surrogate code points such as 0xD800)
let code = Subset::<IsChar, u32>::try_new(0x1F980).unwrap();
assert_eq!(U32ToChar::call(code), '🦀');
assert!(Subset::<IsChar, u32>::try_new(0xD800).is_err());
assert_eq!(*CharToU32::call('🦀'), 0x1F980);
```

---

## 3. Custom Structs & Enums: `#[derive(Projectors)]` Compound Invariants

### 3.1 Cross-Field Coupling on Structs

Deriving `#[derive(SafeValue, SafeDrop, SafeClone, Projectors)]` generates pure field-projection types inside a `snake_case` submodule. Combine `ForkLt` (routing `&PacketWindow` to two field projectors before comparing `<`) and `Then` (chaining field projection into bitmask checks) to form reusable type-level invariants.

```rust
use fy_refine::function::Function;
use fy_refine::function::bitwise::{HasAllBits, HasNoBits};
use fy_refine::function::constant::ConstU32;
use fy_refine::function::predicate::{AllOf, ForkLt};
use fy_refine::function::wiring::Then;
use fy_refine::{Projectors, SafeClone, SafeDrop, SafeValue, Subset};

#[derive(Debug, PartialEq, Eq, SafeValue, SafeDrop, SafeClone, Projectors)]
struct PacketWindow {
    #[projector(copy)]
    lo: u32,
    #[projector(copy)]
    hi: u32,
    #[projector(copy)]
    flags: u32,
}

// Require `lo < hi`, bit 0x01 set, and reserved bits 0xF000 cleared
type FlagMask = AllOf<(HasAllBits<ConstU32<0x01>>, HasNoBits<ConstU32<0xF000>>)>;
type ValidWindowPred = AllOf<(
    ForkLt<packet_window::Lo, packet_window::Hi>,
    Then<packet_window::Flags, FlagMask>,
)>;
#[derive(Debug, PartialEq, Eq)]
struct ValidWindow(Subset<ValidWindowPred, PacketWindow>);

let frame = ValidWindow(
    Subset::try_new(PacketWindow {
        lo: 128,
        hi: 512,
        flags: 0x0003,
    })
    .unwrap(),
);
let cloned = frame.0.clone();
assert_eq!(packet_window::Lo::call(&cloned), 128);
assert!(
    Subset::<ValidWindowPred, PacketWindow>::try_new(PacketWindow {
        lo: 128,
        hi: 512,
        flags: 0x1001, // triggers reserved bit mask 0xF000
    })
    .is_err()
);
```

### 3.2 Variant Routing & Branchless Unwrapping on Enums

After deriving `Projectors` on an enum, use chained `IfLetElse` combinators to express per-variant invariants. Once a value is narrowed via `IsVariant`, `UnwrapVariant` extracts its payload directly without `match` or `Option`.

```rust
use fy_refine::function::Function;
use fy_refine::function::branch::{IfLetElse, IsVariant, UnwrapVariant};
use fy_refine::function::collection::{All, IsNonEmpty};
use fy_refine::function::constant::ConstU32;
use fy_refine::function::predicate::{AllOf, Closed, IsAscii, Open, Range, True};
use fy_refine::function::slice::AsBytes;
use fy_refine::function::wiring::{Always, Fst, Snd, Then};
use fy_refine::{Projectors, SafeValue, Subset};

#[derive(Debug, PartialEq, Eq, SafeValue, Projectors)]
enum Command {
    Resize {
        #[projector(copy)]
        width: u32,
        #[projector(copy)]
        height: u32,
    },
    #[projector(copy)]
    Rename(&'static str),
    Quit,
}

type DimOk = Range<Open<ConstU32<0>>, Closed<ConstU32<4096>>>;
type ValidResize = AllOf<(Then<Fst, DimOk>, Then<Snd, DimOk>)>;
type ValidName = Then<AsBytes, AllOf<(IsNonEmpty, All<IsAscii>)>>;
type ValidCmd = Subset<
    IfLetElse<
        command::Resize,
        ValidResize,
        IfLetElse<command::Rename, ValidName, Always<True>>,
    >,
    Command,
>;

assert!(
    ValidCmd::try_new(Command::Resize {
        width: 1920,
        height: 1080,
    })
    .is_ok()
);
assert!(ValidCmd::try_new(Command::Resize { width: 0, height: 1080 }).is_err());
assert!(ValidCmd::try_new(Command::Rename("viewport")).is_ok());
assert!(ValidCmd::try_new(Command::Rename("")).is_err());

// Proven `Resize` variant: extract `(800, 600)` directly without `match` or `Option`
let resized = Subset::<IsVariant<command::Resize>, Command>::try_new(Command::Resize {
    width: 800,
    height: 600,
})
.unwrap();
assert_eq!(UnwrapVariant::<command::Resize>::call(&resized), (800, 600));
```

---

## 4. Zero-Copy Stacked DST Views & Static Binary Lookup

`Subset<P, T>` composes across arbitrary nesting depths while preserving `?Sized` DSTs. Below, an `AsciiStr` layer and a sorted `IanaScripts` binary-search layer stack directly on `&str` with zero heap allocations: `&RegisteredScript` has the exact same two-word fat-pointer size as `&str`, and `Subset::as_base` penetrates all wrappers back to `&str`.

```rust
use core::mem::size_of;
use fy_refine::function::Function;
use fy_refine::function::collection::All;
use fy_refine::function::lookup::{InSortedSlice, is_strictly_sorted};
use fy_refine::function::predicate::IsAscii;
use fy_refine::function::slice::AsBytes;
use fy_refine::function::wiring::{AsBase, Then};
use fy_refine::{Subset, static_table};

static_table!(
    IanaScripts: &'static [&'static str] = &["Arab", "Cyrl", "Hans", "Hant", "Latn"]
);
assert!(is_strictly_sorted(IanaScripts::call(())));

type AsciiStr = Subset<Then<AsBytes, All<IsAscii>>, str>;
type RegisteredScript = Subset<Then<AsBase, InSortedSlice<IanaScripts>>, AsciiStr>;

// Two-layer refined DST reference has identical size to `&str`
assert_eq!(size_of::<&RegisteredScript>(), size_of::<&str>());

let ascii = AsciiStr::try_from_ref("Hans").unwrap();
let script = RegisteredScript::try_from_ref(ascii).unwrap();
assert_eq!(Subset::as_base(script), "Hans");

let unknown = AsciiStr::try_from_ref("Xxxx").unwrap();
assert!(RegisteredScript::try_from_ref(unknown).is_none());
```

---

## 5. Soundness Model: Why Omitting Runtime Checks Remains Safe

If a value could be mutated behind a shared reference after passing `try_new` (e.g., via `Cell` or `RefCell`), or cloned into a forged copy with different observable properties, branchless operators like `First`, `Get<I>`, `AsStr`, `U32ToChar`, and `UnwrapVariant` would become unsound.

`fy-refine` closes every soundness gap through five explicit contract traits—verified automatically in everyday code via `#[derive(...)]` and declarative macros so downstream code does not need manual `unsafe`:

1. **`SafeValue` (Value Immutability)**: forbids interior mutability observable through `&T` (standard interior-mutable primitives are excluded), guaranteeing predicate invariance between validation and use, while recursively unwrapping `Base` types to their raw carrier;
2. **`SafeClone` (Clone Fidelity)**: guards `Subset::clone` independently so cloned values remain indistinguishable under all pure observers without re-evaluating predicates;
3. **`SafeDrop` (Benign Destruction)**: guarantees that dropping discarded values in product projections (`Fst`, `Snd`, `Discard`) or short-circuit branches terminates cleanly without panicking;
4. **`SafeEq` / `SafeOrd` (Equivalence & Total Order)**: enforces strict equivalence and mathematical total order, separates raw IEEE 754 floats from non-`NaN` refined floats, and performs bit-exact comparisons between refined floats and all 12 primitive integer types;
5. **`SafeFunction` / `Predicate` (Total Pure Functions & Transparent Extensional Algebra)**: requires predicates and projections to be total, deterministic, terminating, and free of destructive side effects; derived combinators expand as **transparent type aliases** (`IsInfinite = Not<IsFinite>`, `IsOdd = Not<IsEven>`, `IsNonEmpty = Not<IsEmpty>`, `IsZero`..`IsNonPositive` over `ConstU8<0>`, `Range` as endpoint conjunctions, `RefineIf` via `Classify` + `BranchResPure`, and `OrElse`/`AndThen` via `RefineIf`) rather than opaque black boxes.

---

## 6. Module Reference & Workspace Layout

| Module | Role | Key Combinators & Types |
| --- | --- | --- |
| `wiring` | Data-flow routing, products & subset penetration | `Id`, `Then`, `Dup`, `Fork`, `PipeWith`, `MapPair`, `Fst`, `Snd`, `WithRef`, `AsInner`, `AsBase`, `IntoBase` |
| `predicate` | Extensional Boolean algebra, comparisons, intervals & domain checks | `And`, `Or`, `Not`, `Xor`, `AllOf`, `AnyOf`, `EqConst`..`GeConst`, `ForkLt`..`ForkGe`, `Range`, `Closed`/`Open`, `IsZero`..`IsNonPositive`, `IsNaN`/`IsNotNaN`/`IsFinite`/`IsInfinite`/`IsInteger`, `IsEven`/`IsOdd`, `IsAscii*`, `IsChar`/`IsUtf8` |
| `branch` | Proof-carrying partitioning, sum-type & enum routing | `Classify`, `RefineIf`, `AndThen`, `OrElse`, `IfElse`, `IfLetElse`, `IsVariant`, `UnwrapVariant`, `IsSome`/`IsOk`, `UnwrapSome`/`UnwrapOk` |
| `collection` | Container quantifiers, fallible probes & branchless access | `Len`, `IsEmpty`, `IsNonEmpty`, `LenGt<I>`, `All`, `Any`, `TryFirst`/`TryLast`/`TryGet<I>`, `First`/`Last`/`Get<I>` |
| `slice` | Zero-copy contiguous memory views & proof-carrying `str` conversion | `AsBytes`, `AsSlice`, `AsStr` (for `All<IsAscii>` and `IsUtf8`) |
| `bitwise` | Bitmask & flag checks on unsigned integers | `HasAllBits`, `HasAnyBits`, `HasNoBits` |
| `lookup` | Static tables, disjoint sorted ranges & 256-bit bitsets | `static_table!`, `InSliceEq`, `InSortedSlice`, `InSortedRanges`, `InBitset` |
| `constant` | Compile-time primitive constant morphisms | `ConstU8`..`ConstU128`, `ConstI8`..`ConstI128`, `ConstUsize`/`ConstIsize`, `ConstBool`/`ConstChar` |

### Workspace Crates

- `fy-refine` (`crates/fy-refine`): Unified facade re-exporting `fy-refine-core` (`fy_refine::core`) and derive macros (`SafeValue`, `SafeDrop`, `SafeClone`, `Projectors`).
- `fy-refine-core` (`crates/core`): Minimal-dependency core crate providing safety contracts, combinators, declarative macros, refined float aliases, and `Subset<P, T>`.
- `fy-refine-macro` (`crates/macro`): Zero-dependency procedural derive macro crate built directly on `proc_macro`.

### Installation

Add to `Cargo.toml`:

```toml
[dependencies]
fy-refine = "0.1"
```

Pure `no_std` (no allocator) with checked Serde support:

```toml
[dependencies]
fy-refine = { version = "0.1", default-features = false, features = ["serde"] }
```

### Cargo Features

| Feature | Default | Description |
| --- | --- | --- |
| `std` | Yes | Enables standard library container support alongside `alloc`. |
| `alloc` | Via `std` | Enables allocator-backed containers (`Box`, `Vec`, `String`, `Rc`, `Arc`, `BTreeMap`, etc.). |
| `serde` | No | Enables checked `Serialize` and `Deserialize` for `Subset<P, T>` and nested DST references (`&'de Subset<P, str>`, `&'de Subset<P, [u8]>`, etc.). |
| `nightly` | No | Enables `#![feature(const_trait_impl, const_destruct)]` for conditional `const` evaluation, `#[refine(const)]`, and compile-time checked construction. |

### MSRV

The Minimum Supported Rust Version (MSRV) is **Rust 1.86.0** (`edition = "2024"`).

### Star History

[![Star History Chart](https://api.star-history.com/svg?repos=Fengyang-Conglomerate/fy-refine-code&type=Date)](https://star-history.com/#Fengyang-Conglomerate/fy-refine-code&Date)

### License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
