# fy-refine

[![crates.io](https://img.shields.io/crates/v/fy-refine)](https://crates.io/crates/fy-refine)
[![docs.rs](https://docs.rs/fy-refine/badge.svg)](https://docs.rs/fy-refine)
[![CI](https://github.com/Fengyang-Conglomerate/fy-refine-code/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/Fengyang-Conglomerate/fy-refine-code/actions/workflows/ci.yml)
[![Coverage](https://coveralls.io/repos/github/Fengyang-Conglomerate/fy-refine-code/badge.svg?branch=main)](https://coveralls.io/github/Fengyang-Conglomerate/fy-refine-code?branch=main)
[![Rust 1.86+](https://img.shields.io/badge/rust-1.86%2B-blue)](https://www.rust-lang.org/)
[![no_std](https://img.shields.io/badge/no__std-compatible-green)](#特性标志)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#许可)

[English](README.md) | 简体中文

**在系统边界做一次校验，把条件证明编码进类型系统；进入业务逻辑后，以静态类型约束替代重复检查、防御性 `unwrap()` 与手写 `unsafe`。**

`fy-refine` 是一个原生支持 `no_std` 的 Rust 精化类型（Refinement Types）与纯函数组合库。核心类型 `Subset<P, T>` 采用 `#[repr(transparent)]` 透明布局，不在运行期存储谓词状态，且原生支持 `str`、`[T]` 等动态大小类型（DST）胖指针视图。

---

## 核心理念：为什么使用类型级函数组合？

Rust 中的闭包没有可命名的独立类型，无法直接写进结构体签名或泛型边界；此外，如果只在外部包裹简单的作用域守卫，同样满足 `[0, 100]` 范围的“百分比”“年龄”“音量”也很容易互相混用。

`fy-refine` 把基础逻辑判断封装为零大小的**类型级纯函数** `P`，并严格区分两层职责：

1. **结构子集 `Subset<P, T>`（值域约束）**：表达底层载体 `T` 在数学上满足谓词 `P`，与 `T` 共享完全相同的内存布局；
2. **名义新类型 `struct Percentage(...)`（领域身份）**：基于 `Subset` 定义业务概念，防止结构相同但语义不同的数值彼此混用。

---

## 一、数值域分层与携带证明的条件分支

在 IEEE 754 浮点数设计中，`NaN`（Not a Number）的本义就是“不是数”，它会破坏自反性与三分律。若允许直接对原始 `f32`/`f64` 做全序比较或正负符号判断，逻辑取反（`Not<IsNonPositive>`）时就会把 `NaN` 误归入“正数”分支；若允许直接对未证为整数的浮点数做奇偶判断，取反 `Not<IsEven>`（即 `IsOdd`）同样会混入小数与 `NaN`。

因此，`fy-refine` 按照**“每一层的逻辑取反 `Not<P>` 均构成无杂质互补概念”**的原则，将浮点数到整数划分为严格递进的四级二分阶梯：

1. **表示层（裸 `f32` / `f64`）**：仅开放 `IsNaN` 与透明别名 `IsNotNaN = Not<IsNaN>`，故意不实现 `SafeEq`、`SafeOrd` 及正负/有限/整数/奇偶判定（直接写 `Subset<IsNonPositive, f32>` 或 `Subset<IsInteger, f32>` 在编译期拒绝）；
2. **扩展实数域（`NotNaNF32` / `NotNaNF64`）**：排除 `NaN` 后激活精确数学全序 `SafeOrd`（含 `-0.0 == +0.0 == 0`，以及按 IEEE 754 位模式执行的跨 12 种整数无损比较）与全部序/符号谓词，同时开放 `IsFinite` 与透明别名 `IsInfinite = Not<IsFinite>`（非有限即无穷，无 `NaN` 混入）；
3. **有限实数域（`FiniteF32 = Subset<IsFinite, NotNaNF32>` / `FiniteF64`）**：排除 `NaN` 与 `±∞` 后开放 `IsInteger`，使其补集 `Not<IsInteger>` 严格对应“带非零小数部分的有限实数”；
4. **整数域（12 种原生整数及 `Subset<IsInteger, FiniteF32/64>`）**：在此域内“非偶即奇”严格成立，开放 `IsEven` 与透明别名 `IsOdd = Not<IsEven>`。

若希望在原始 `f32` 上一步表达跨域条件（例如“要么是 `NaN`，要么是正数”），可使用**携带证明的条件分支原语 `Classify<P>`** 及其组合别名 **`RefineIf` / `AndThen` / `OrElse`**：`RefineIf<P, Then, Else>` 在判真分支自动把输入类型精化为 `&Subset<P, T>`，在判假分支精化为 `&Subset<Not<P>, T>`。

```rust
use fy_refine::function::Function;
use fy_refine::function::branch::{AndThen, OrElse, RefineIf};
use fy_refine::function::constant::ConstU8;
use fy_refine::function::predicate::{
    Closed, IsFinite, IsInteger, IsNaN, IsNonPositive, IsNotNaN, IsOdd, IsPositive, Range,
};
use fy_refine::{FiniteF64, NotNaNF32, NotNaNF64, SafeEq, SafeOrd, Subset};

// 1. 结构子集与名义领域类型分离
type U8In0To100 = Subset<Range<Closed<ConstU8<0>>, Closed<ConstU8<100>>>, u8>;
#[derive(Debug, PartialEq, Eq)]
struct Percentage(U8In0To100);

let pct = Percentage(U8In0To100::try_new(85).unwrap());
assert_eq!(*pct.0, 85);

// 2. 四级数域分层：先经 NotNaN -> Finite 收窄进入有限全序实数域，再使用序谓词与跨整数精确比较
type NonPosF32 = Subset<IsNonPositive, NotNaNF32>;
type Probability = Subset<Range<Closed<ConstU8<0>>, Closed<ConstU8<1>>>, FiniteF64>;

let neg_zero = NotNaNF32::try_new(-0.0).unwrap();
assert!(NonPosF32::try_new(neg_zero).is_ok());
assert!(NotNaNF32::try_new(f32::NAN).is_err());

let fin_half = FiniteF64::try_new(NotNaNF64::try_new(0.5).unwrap()).unwrap();
let half = Probability::try_new(fin_half).unwrap();
assert!(SafeEq::eq(&neg_zero, &0u8));
assert!(SafeOrd::gt(&half, &0u8) && SafeOrd::lt(&half, &1u8));

// 精确到逐比特：2^53 可以精确表示，2^53 < 2^53 + 1 不会被浮点舍入误判相等
let f_pow53 = FiniteF64::try_new(NotNaNF64::try_new((1u64 << 53) as f64).unwrap()).unwrap();
assert!(SafeEq::eq(&f_pow53, &(1u64 << 53)));
assert!(SafeOrd::lt(&f_pow53, &((1u64 << 53) + 1)));

// 3. 证据传递分支（OrElse / AndThen / RefineIf）：直接在裸 f32 上构造跨域精化约束
//    当 IsNaN 为 false 时，Classify 自动将 &Subset<Not<IsNaN>, f32>（即 &NotNaNF32）递给 IsPositive
type NaNOrPositive = Subset<OrElse<IsNaN, IsPositive>, f32>;
assert!(NaNOrPositive::try_new(f32::NAN).is_ok());
assert!(NaNOrPositive::try_new(3.5).is_ok());
assert!(NaNOrPositive::try_new(-1.0).is_err());

//    沿着四级阶梯精化：非 NaN -> 有限 -> 若为整数则必须为奇数（IsOdd），否则必须为正数
type OddIntOrPosFrac =
    Subset<AndThen<IsNotNaN, AndThen<IsFinite, RefineIf<IsInteger, IsOdd, IsPositive>>>, f32>;
assert!(OddIntOrPosFrac::try_new(-3.0).is_ok());
assert!(OddIntOrPosFrac::try_new(2.5).is_ok());
assert!(OddIntOrPosFrac::try_new(2.0).is_err());
assert!(OddIntOrPosFrac::try_new(-1.5).is_err());
assert!(OddIntOrPosFrac::try_new(f32::INFINITY).is_err());
```

---

## 二、序列与容器：精化前可失败探针与精化后无分支直取

在实际编码中，容器在“精化之前（校验谓词内部）”与“精化之后（下游消费时）”需要两种不同语义的访问方式：

- **精化之前（构建谓词）**：容器长度未知，使用可失败探针 `TryFirst` / `TryLast` / `TryGet<I>` 安全借出 `Option<&T>`，常与 `SomeAnd<P>` / `NoneOr<P>` 组合约束指定位置元素；
- **精化之后（下游直取）**：一旦持有 `Subset<IsNonEmpty, C>` 或 `Subset<LenGt<I>, C>`（同时支持借用切片 `[u8]`、定长数组 `[u8; N]` 以及在 `std`/`alloc` 下的拥有型 `Vec<u8>` / `VecDeque<u8>` 等），即可调用 `First` / `Last` / `Get<I>` **无 `Option`、无手写 `unsafe` 直接返回 `&T`**（越界下标在编译期即被类型检查拒绝）；
- **编码与字符证明**：同理，携带 `All<IsAscii>` 或 `IsUtf8` 的 `[u8]` / `[u8; N]` / `Vec<u8>` 可由 `AsStr` 以 $O(1)$ 零拷贝直接转为 `&str`；携带 `IsChar`（排除代理区 `0xD800..=0xDFFF`）的 `u32` 可通过 `U32ToChar` 与 `CharToU32` 双向免检转换 `char`。

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

// 1. 精化之前：在谓词内部用 TryGet<0> + SomeAnd 安全检查第 0 字节是否为大写 ASCII
type StartsWithUpper = Then<TryGet<0>, SomeAnd<IsAsciiUppercase>>;
let header = Subset::<StartsWithUpper, Vec<u8>>::try_new(b"GET /index".to_vec()).unwrap();
assert_eq!(Subset::as_inner(&header).len(), 10);

// 2. 精化之后：Vec<u8> 与切片 &[u8] 均可免 Option/Result 直取与转 &str
type AtLeast3Vec = Subset<LenGt<2>, Vec<u8>>;
let frame_vec = AtLeast3Vec::try_new(vec![b'A', b'B', b'C']).unwrap();
assert_eq!(*Get::<2>::call(&frame_vec), b'C'); // 对 frame_vec 调用 Get::<3> 将在编译期拒绝

type NonEmptyBytes = Subset<IsNonEmpty, [u8]>;
let non_empty = NonEmptyBytes::try_from_ref(b"refine").unwrap();
assert_eq!((*First::call(non_empty), *Last::call(non_empty)), (b'r', b'e'));

let utf8_vec = Subset::<IsUtf8, Vec<u8>>::try_new("café🦀".as_bytes().to_vec()).unwrap();
assert_eq!(AsStr::call(&utf8_vec), "café🦀");

let ascii_slice = Subset::<All<IsAscii>, [u8]>::try_from_ref(b"fy-refine").unwrap();
assert_eq!(AsStr::call(ascii_slice), "fy-refine");

// 3. Unicode 标量值（u32 与 char 双向免检互转，拒绝 0xD800 等代理区码点）
let code = Subset::<IsChar, u32>::try_new(0x1F980).unwrap();
assert_eq!(U32ToChar::call(code), '🦀');
assert!(Subset::<IsChar, u32>::try_new(0xD800).is_err());
assert_eq!(*CharToU32::call('🦀'), 0x1F980);
```

---

## 三、自定义结构体与枚举：`#[derive(Projectors)]` 复合不变量

### 3.1 多字段结构体耦合约束

通过 `#[derive(SafeValue, SafeDrop, SafeClone, Projectors)]`，编译器会在 `snake_case` 同名子模块下自动为各字段生成纯函数投影。借助 `ForkLt`（将同一个结构体引用分流给两个字段投影并比较大小）与 `Then`（串联字段投影与位掩码谓词），即可把跨字段耦合约束表达为纯类型。

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

// 约束条件：lo < hi，且 flags 必须置位 0x01 并清零保留位 0xF000
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
        flags: 0x1001, // 触犯保留位 0xF000
    })
    .is_err()
);
```

### 3.2 代数数据类型（`enum`）按变体分流与免 `match` 解包

为自定义 `enum` 派生 `Projectors` 后，可用链式 `IfLetElse` 为不同变体指定独立的精化条件；对已证明处于目标变体的 `Subset<IsVariant<...>, Command>`，调用 `UnwrapVariant` 即可直接消除 `match` 分支提取内部载荷。

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

// 已证明为 Resize 变体的子集，免 match 分支与 Option 直接提取 (800, 600)
let resized = Subset::<IsVariant<command::Resize>, Command>::try_new(Command::Resize {
    width: 800,
    height: 600,
})
.unwrap();
assert_eq!(UnwrapVariant::<command::Resize>::call(&resized), (800, 600));
```

---

## 四、零拷贝多层 DST 视图与静态有序表检索

`Subset<P, T>` 原生支持连续多层嵌套且保留 `?Sized` 动态大小类型。下面在无堆分配下对借用 `&str` 依次叠加“全 ASCII”与“IanaScripts 静态有序表二分命中”两层约束，`&RegisteredScript` 与 `&str` 尺寸完全一致，可通过 `Subset::as_base` 一步穿透还原底层 `&str`。

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

// 双层精化 DST 引用与原生 &str 布局尺寸完全一致
assert_eq!(size_of::<&RegisteredScript>(), size_of::<&str>());

let ascii = AsciiStr::try_from_ref("Hans").unwrap();
let script = RegisteredScript::try_from_ref(ascii).unwrap();
assert_eq!(Subset::as_base(script), "Hans");

let unknown = AsciiStr::try_from_ref("Xxxx").unwrap();
assert!(RegisteredScript::try_from_ref(unknown).is_none());
```

---

## 五、健全性边界：为什么省略运行期检查依然安全？

如果下游类型在通过 `try_new` 校验后仍能在共享引用下篡改内部数据（例如借助 `Cell` / `RefCell`），或在 `Clone` 中构造伪造副本，那么后续的 `First`、`Get<I>`、`AsStr`、`U32ToChar` 与 `UnwrapVariant` 将失去健全性前提。

`fy-refine` 通过五项契约约束形成类型系统闭环，并在日常开发中由 `#[derive(...)]` 与声明宏自动核验，使业务代码无需手写 `unsafe`：

1. **`SafeValue`（值不变性公理）**：禁止在共享借用下更改影响纯函数观察结果的状态（标准库内部可变容器均被排除），确保判定不在构造与使用之间发生漂移，同时负责沿着 `Base` 类型递归穿透至最底层原始载体；
2. **`SafeClone`（克隆保真公理）**：独立约束 `Subset::clone`，确保副本与原值在所有纯函数观察下等价，从而无需在克隆时重复校验谓词；
3. **`SafeDrop`（良性析构公理）**：保证在元组投影丢弃一侧值（`Fst` / `Snd` / `Discard`）或短路分支丢弃值时，析构必在有限步骤内完成且绝不 panic；
4. **`SafeEq` / `SafeOrd`（严格等价与全序契约）**：严格满足等价关系与数学全序，显式区分原始 IEEE 754 浮点与非 `NaN` 精化浮点，并在精化浮点与 12 种整数之间实现逐比特精确数值比较；
5. **`SafeFunction` / `Predicate`（全域纯函数与透明外延代数）**：要求参与精化的纯函数确定、必终止且无破坏性副作用；核心派生谓词优先实现为**透明组合别名**（如 `IsInfinite = Not<IsFinite>`、`IsOdd = Not<IsEven>`、`IsNonEmpty = Not<IsEmpty>`、`IsZero`..`IsNonPositive` 基于 `ConstU8<0>` 比较、`Range` 展开为双端比较合取、`RefineIf` 展开为 `Then<Classify, BranchResPure>`、`OrElse`/`AndThen` 基于 `RefineIf` 展开），避免不透明黑盒阻断外延关系推导。

---

## 六、模块索引与工作区包结构

| 模块 | 职责说明 | 代表算子与类型 |
| --- | --- | --- |
| `wiring` | 数据流路由、积配对分流与子集穿透 | `Id`、`Then`、`Dup`、`Fork`、`PipeWith`、`MapPair`、`Fst`、`Snd`、`WithRef`、`AsInner`、`AsBase`、`IntoBase` |
| `predicate` | 逻辑代数、比较、开闭区间与域分层判定 | `And`、`Or`、`Not`、`Xor`、`AllOf`、`AnyOf`、`EqConst`..`GeConst`、`ForkLt`..`ForkGe`、`Range`、`Closed`/`Open`、`IsZero`..`IsNonPositive`、`IsNaN`/`IsNotNaN`/`IsFinite`/`IsInfinite`/`IsInteger`、`IsEven`/`IsOdd`、`IsAscii*`、`IsChar`/`IsUtf8` |
| `branch` | 证明二分传递、和类型与枚举变体消除 | `Classify`、`RefineIf`、`AndThen`、`OrElse`、`IfElse`、`IfLetElse`、`IsVariant`、`UnwrapVariant`、`IsSome`/`IsOk`、`UnwrapSome`/`UnwrapOk` |
| `collection` | 通用容器量词、可失败探针与免分支直取 | `Len`、`IsEmpty`、`IsNonEmpty`、`LenGt<I>`、`All`、`Any`、`TryFirst`/`TryLast`/`TryGet<I>`、`First`/`Last`/`Get<I>` |
| `slice` | 连续内存切片零拷贝投影与编码证明直转 | `AsBytes`、`AsSlice`、`AsStr`（支持 `All<IsAscii>` 与 `IsUtf8`） |
| `bitwise` | 无符号整数字段位掩码检查 | `HasAllBits`、`HasAnyBits`、`HasNoBits` |
| `lookup` | 静态查找表、有序区间二分与 256-bit 位图 | `static_table!`、`InSliceEq`、`InSortedSlice`、`InSortedRanges`、`InBitset` |
| `constant` | 编译期字面量常量态射 | `ConstU8`..`ConstU128`、`ConstI8`..`ConstI128`、`ConstUsize`/`ConstIsize`、`ConstBool`/`ConstChar` |

### 工作区包结构

- `fy-refine`（`crates/fy-refine`）：面向下游应用的统一门面包，重导出 `fy_refine_core`（`fy_refine::core`）与派生宏 `SafeValue`、`SafeDrop`、`SafeClone`、`Projectors`；
- `fy-refine-core`（`crates/core`）：最小依赖核心库，提供安全契约、组合原语、声明宏、精化浮点别名与 `Subset<P, T>`；
- `fy-refine-macro`（`crates/macro`）：零第三方依赖（纯 `proc_macro`）派生宏实现包。

### 安装配置

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
fy-refine = "0.1"
```

纯 `no_std`（无 `alloc`）环境按需开启受检 Serde 支持：

```toml
[dependencies]
fy-refine = { version = "0.1", default-features = false, features = ["serde"] }
```

### 特性标志

| 特性 | 默认开启 | 说明 |
| --- | --- | --- |
| `std` | 是 | 启用标准库容器与 `alloc` 能力。 |
| `alloc` | 由 `std` 启用 | 启用堆分配容器支持（`Box`、`Vec`、`String`、`Rc`、`Arc`、`BTreeMap` 等）。 |
| `serde` | 否 | 为 `Subset<P, T>` 及嵌套 DST 引用（`&'de Subset<P, str>`、`&'de Subset<P, [u8]>` 等）提供受检序列化与反序列化。 |
| `nightly` | 否 | 启用 `#![feature(const_trait_impl, const_destruct)]`，支持条件 `const` 求值、`#[refine(const)]` 与编译期受检构造。 |

### MSRV

最低支持的 Rust 版本（MSRV）为 **Rust 1.86.0**（`edition = "2024"`）。

### Star 趋势

[![Star History Chart](https://api.star-history.com/svg?repos=Fengyang-Conglomerate/fy-refine-code&type=Date)](https://star-history.com/#Fengyang-Conglomerate/fy-refine-code&Date)

### 许可

可根据您的意愿选择以下任一许可证授权：

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))
