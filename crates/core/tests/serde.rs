//! Tests for optional `serde` serialization and checked deserialization (including zero-copy DST slice deserialization).
#![cfg(feature = "serde")]

use core::fmt;
use serde::de::{self, Deserializer, Visitor};
use serde::ser::{self, Serializer};
use serde::{Deserialize, Serialize};

use fy_refine_core::function::collection::{IsNonEmpty, Len};
use fy_refine_core::function::constant::{ConstU8, ConstUsize};
use fy_refine_core::function::predicate::{
    All, And, Closed, EqConst, IsAscii, IsAsciiAlphabetic, Range,
};
use fy_refine_core::function::slice::AsBytes;
use fy_refine_core::function::wiring::{AsBase, Then};
use fy_refine_core::subset::Subset;

#[derive(Debug, PartialEq, Eq)]
struct MockError(String);

impl fmt::Display for MockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl core::error::Error for MockError {}
impl ser::Error for MockError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self(msg.to_string())
    }
}
impl de::Error for MockError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self(msg.to_string())
    }
}

/// Minimal offline test serializer that only records pass-through u8 / str / bytes tokens.
#[derive(Debug, PartialEq, Eq)]
enum Token {
    U8(u8),
    Str(String),
    Bytes(Vec<u8>),
}

struct MockSerializer;
impl Serializer for MockSerializer {
    type Ok = Token;
    type Error = MockError;
    type SerializeSeq = ser::Impossible<Token, MockError>;
    type SerializeTuple = ser::Impossible<Token, MockError>;
    type SerializeTupleStruct = ser::Impossible<Token, MockError>;
    type SerializeTupleVariant = ser::Impossible<Token, MockError>;
    type SerializeMap = ser::Impossible<Token, MockError>;
    type SerializeStruct = ser::Impossible<Token, MockError>;
    type SerializeStructVariant = ser::Impossible<Token, MockError>;

    fn serialize_u8(self, v: u8) -> Result<Token, MockError> {
        Ok(Token::U8(v))
    }
    fn serialize_str(self, v: &str) -> Result<Token, MockError> {
        Ok(Token::Str(v.to_owned()))
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Token, MockError> {
        Ok(Token::Bytes(v.to_vec()))
    }
    fn serialize_bool(self, _: bool) -> Result<Token, MockError> {
        unimplemented!()
    }
    fn serialize_i8(self, _: i8) -> Result<Token, MockError> {
        unimplemented!()
    }
    fn serialize_i16(self, _: i16) -> Result<Token, MockError> {
        unimplemented!()
    }
    fn serialize_i32(self, _: i32) -> Result<Token, MockError> {
        unimplemented!()
    }
    fn serialize_i64(self, _: i64) -> Result<Token, MockError> {
        unimplemented!()
    }
    fn serialize_u16(self, _: u16) -> Result<Token, MockError> {
        unimplemented!()
    }
    fn serialize_u32(self, _: u32) -> Result<Token, MockError> {
        unimplemented!()
    }
    fn serialize_u64(self, _: u64) -> Result<Token, MockError> {
        unimplemented!()
    }
    fn serialize_f32(self, _: f32) -> Result<Token, MockError> {
        unimplemented!()
    }
    fn serialize_f64(self, _: f64) -> Result<Token, MockError> {
        unimplemented!()
    }
    fn serialize_char(self, _: char) -> Result<Token, MockError> {
        unimplemented!()
    }
    fn serialize_none(self) -> Result<Token, MockError> {
        unimplemented!()
    }
    fn serialize_some<T: ?Sized + Serialize>(self, _: &T) -> Result<Token, MockError> {
        unimplemented!()
    }
    fn serialize_unit(self) -> Result<Token, MockError> {
        unimplemented!()
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<Token, MockError> {
        unimplemented!()
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
    ) -> Result<Token, MockError> {
        unimplemented!()
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: &T,
    ) -> Result<Token, MockError> {
        unimplemented!()
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<Token, MockError> {
        unimplemented!()
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, MockError> {
        unimplemented!()
    }
    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, MockError> {
        unimplemented!()
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, MockError> {
        unimplemented!()
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, MockError> {
        unimplemented!()
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, MockError> {
        unimplemented!()
    }
    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, MockError> {
        unimplemented!()
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, MockError> {
        unimplemented!()
    }
    fn collect_str<T: ?Sized + fmt::Display>(self, _: &T) -> Result<Token, MockError> {
        unimplemented!()
    }
}

enum BorrowedInput<'de> {
    U8(u8),
    Str(&'de str),
    Bytes(&'de [u8]),
}

impl<'de> Deserializer<'de> for BorrowedInput<'de> {
    type Error = MockError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, MockError> {
        match self {
            BorrowedInput::U8(v) => visitor.visit_u8(v),
            BorrowedInput::Str(s) => visitor.visit_borrowed_str(s),
            BorrowedInput::Bytes(b) => visitor.visit_borrowed_bytes(b),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

#[test]
fn serde_owned_and_zero_copy_dst() {
    type Percent = Subset<Range<Closed<ConstU8<0>>, Closed<ConstU8<100>>>, u8>;
    type ScriptStr =
        Subset<Then<AsBytes, And<Then<Len, EqConst<ConstUsize<4>>>, All<IsAsciiAlphabetic>>>, str>;
    type AsciiBytes = Subset<All<IsAscii>, [u8]>;

    // 1. Owned serialization and checked deserialization
    let ok = Percent::try_new(88).unwrap();
    assert_eq!(ok.serialize(MockSerializer).unwrap(), Token::U8(88));
    let de_ok = Percent::deserialize(BorrowedInput::U8(88)).unwrap();
    assert_eq!(de_ok, ok);
    assert!(Percent::deserialize(BorrowedInput::U8(101)).is_err());

    // 2. Zero-copy deserialization of &'de Subset<P, str> and transparent serialization
    let raw_script = "Latn";
    let de_str: &ScriptStr = Deserialize::deserialize(BorrowedInput::Str(raw_script)).unwrap();
    assert_eq!(&**de_str, "Latn");
    assert_eq!(
        de_str.serialize(MockSerializer).unwrap(),
        Token::Str(String::from("Latn"))
    );
    assert!(<&ScriptStr>::deserialize(BorrowedInput::Str("1234")).is_err());

    // 3. Zero-copy deserialization of &'de Subset<P, [u8]>
    let raw_bytes: &[u8] = b"ascii";
    let de_bytes: &AsciiBytes = Deserialize::deserialize(BorrowedInput::Bytes(raw_bytes)).unwrap();
    assert_eq!(&**de_bytes, b"ascii");
    assert!(<&AsciiBytes>::deserialize(BorrowedInput::Bytes(&[0x80])).is_err());

    // 4. Arbitrary multi-layer nested DST zero-copy deserialization &'de Subset<P2, Subset<P1, str>>
    type AsciiStr = Subset<Then<AsBytes, All<IsAscii>>, str>;
    type NonEmptyAsciiStr = Subset<Then<AsBase, IsNonEmpty>, AsciiStr>;
    let nested: &NonEmptyAsciiStr = Deserialize::deserialize(BorrowedInput::Str("fy")).unwrap();
    assert_eq!(Subset::as_base(nested), "fy");
    assert!(<&NonEmptyAsciiStr>::deserialize(BorrowedInput::Str("")).is_err());
    assert!(<&NonEmptyAsciiStr>::deserialize(BorrowedInput::Str("风")).is_err());
}
