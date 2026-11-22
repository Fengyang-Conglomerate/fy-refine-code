//! Optional Serde serialization and checked deserialization support.
//!
//! - `Serialize` transparently forwards the underlying value for both owned values and DST views;
//! - `Deserialize` for owned values deserializes the underlying `T` first and validates it via [`Subset::try_new`];
//! - Provides generic zero-copy checked deserialization for any borrowed deserializable type
//!   (such as `&'de str`, `&'de [u8]`, and arbitrarily nested `&'de Subset<..., T>`).

use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::Subset;
use crate::function::Predicate;
use crate::marker::SafeValue;

impl<P, T: ?Sized + Serialize> Serialize for Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.value.serialize(serializer)
    }
}

impl<'de, P, T> Deserialize<'de> for Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue + Deserialize<'de>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = T::deserialize(deserializer)?;
        Subset::try_new(raw).map_err(|_| {
            D::Error::custom("deserialized value does not satisfy refinement predicate")
        })
    }
}

impl<'de, P, T: ?Sized> Deserialize<'de> for &'de Subset<P, T>
where
    P: for<'a> Predicate<&'a T>,
    T: SafeValue,
    &'de T: Deserialize<'de>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = <&'de T>::deserialize(deserializer)?;
        Subset::try_from_ref(raw).ok_or_else(|| {
            D::Error::custom("deserialized borrowed value does not satisfy refinement predicate")
        })
    }
}
