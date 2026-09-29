//! The `DecodedValue` taxonomy: one decoded Soroban value, as a typed Rust enum.
//!
//! This is the wire contract fixed by pulsar-app ADR-023 and shared with the Go
//! indexer and the TypeScript SDK. A `DecodedValue` serializes to a discriminated
//! union keyed by a `type` string, and deserializes back from one, so a value
//! stored by the indexer round trips through this crate unchanged.
//!
//! The enum holds idiomatic Rust types (`u128`, `Vec<u8>`, and so on), while the
//! hand-written `Serialize` and `Deserialize` bridge to the exact JSON shape the
//! other two implementations already produce and read. Two representations meet
//! here: the in-memory one a Rust caller works with, and the wire one every
//! consumer agrees on.

use serde::de::Error as _;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::hex;

/// One decoded Soroban value.
///
/// The variants mirror the `ScVal` cases this decoder handles, plus an
/// [`Unknown`](DecodedValue::Unknown) fallback for anything it cannot yet name.
/// Integers narrower than 53 bits (`u32`, `i32`) are carried as JSON numbers;
/// everything wider is carried as a decimal string on the wire, because a JSON
/// number rounds silently past 2^53. See pulsar-app ADR-023.
///
/// There is deliberately no tuple variant. Soroban encodes a tuple as a vector,
/// and only a decoder holding the contract's spec can tell the two apart, so a
/// decoder working from XDR alone always emits [`Vec`](DecodedValue::Vec).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodedValue {
    /// A Soroban address, strkey-encoded (`G...` account or `C...` contract).
    Address(String),
    /// A Soroban symbol.
    Symbol(String),
    /// A UTF-8 string.
    String(String),
    /// A boolean.
    Bool(bool),
    /// A byte string, carried on the wire as lowercase hex.
    Bytes(Vec<u8>),
    /// An unsigned 32-bit integer.
    U32(u32),
    /// A signed 32-bit integer.
    I32(i32),
    /// An unsigned 64-bit integer.
    U64(u64),
    /// A signed 64-bit integer.
    I64(i64),
    /// An unsigned 128-bit integer.
    U128(u128),
    /// A signed 128-bit integer.
    I128(i128),
    /// An unsigned 256-bit integer, as a decimal string. Rust has no native
    /// 256-bit type, so the decimal rendering is carried directly.
    U256(String),
    /// A signed 256-bit integer, as a decimal string.
    I256(String),
    /// A point in time, as an unsigned count of seconds since the Unix epoch.
    Timepoint(u64),
    /// A span of time, as an unsigned count of seconds.
    Duration(u64),
    /// An ordered list of values.
    Vec(Vec<DecodedValue>),
    /// An ordered list of key/value entries. See [`MapEntry`].
    Map(Vec<MapEntry>),
    /// The absence of a value.
    Void,
    /// A value this decoder cannot name, carrying its base64 XDR intact.
    ///
    /// A protocol upgrade that adds an `ScVal` variant produces this rather than
    /// an error, so an old consumer keeps reading new data. The base64 is the
    /// original value, so nothing is lost.
    Unknown {
        /// The base64-encoded XDR of the value.
        xdr: String,
    },
}

/// One key and value of a Soroban map.
///
/// A map is an ordered list of these rather than a keyed collection, per
/// pulsar-app ADR-023: keys are themselves values and need not be strings, wire
/// ordering is meaningful, and duplicate keys must survive rather than collapse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapEntry {
    /// The entry's key.
    pub key: DecodedValue,
    /// The entry's value.
    pub value: DecodedValue,
}

impl DecodedValue {
    /// Returns the `type` discriminant this value serializes under.
    ///
    /// Total: every variant has a name, so this never panics and never returns
    /// an empty string.
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            DecodedValue::Address(_) => "address",
            DecodedValue::Symbol(_) => "symbol",
            DecodedValue::String(_) => "string",
            DecodedValue::Bool(_) => "bool",
            DecodedValue::Bytes(_) => "bytes",
            DecodedValue::U32(_) => "u32",
            DecodedValue::I32(_) => "i32",
            DecodedValue::U64(_) => "u64",
            DecodedValue::I64(_) => "i64",
            DecodedValue::U128(_) => "u128",
            DecodedValue::I128(_) => "i128",
            DecodedValue::U256(_) => "u256",
            DecodedValue::I256(_) => "i256",
            DecodedValue::Timepoint(_) => "timepoint",
            DecodedValue::Duration(_) => "duration",
            DecodedValue::Vec(_) => "vec",
            DecodedValue::Map(_) => "map",
            DecodedValue::Void => "void",
            DecodedValue::Unknown { .. } => "unknown",
        }
    }
}

/// Serializes a two-entry object `{"type": tag, "value": value}`.
fn serialize_tagged<S, T>(serializer: S, tag: &str, value: &T) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize + ?Sized,
{
    let mut map = serializer.serialize_map(Some(2))?;
    map.serialize_entry("type", tag)?;
    map.serialize_entry("value", value)?;
    map.end()
}

impl Serialize for DecodedValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            // Strings on both sides.
            DecodedValue::Address(s) => serialize_tagged(serializer, "address", s),
            DecodedValue::Symbol(s) => serialize_tagged(serializer, "symbol", s),
            DecodedValue::String(s) => serialize_tagged(serializer, "string", s),
            DecodedValue::U256(s) => serialize_tagged(serializer, "u256", s),
            DecodedValue::I256(s) => serialize_tagged(serializer, "i256", s),

            // Boolean and the two narrow integers, carried as JSON scalars.
            DecodedValue::Bool(b) => serialize_tagged(serializer, "bool", b),
            DecodedValue::U32(n) => serialize_tagged(serializer, "u32", n),
            DecodedValue::I32(n) => serialize_tagged(serializer, "i32", n),

            // Wide integers and time values, rendered as decimal strings so a
            // JSON number never rounds them.
            DecodedValue::U64(n) => serialize_tagged(serializer, "u64", &n.to_string()),
            DecodedValue::I64(n) => serialize_tagged(serializer, "i64", &n.to_string()),
            DecodedValue::U128(n) => serialize_tagged(serializer, "u128", &n.to_string()),
            DecodedValue::I128(n) => serialize_tagged(serializer, "i128", &n.to_string()),
            DecodedValue::Timepoint(n) => serialize_tagged(serializer, "timepoint", &n.to_string()),
            DecodedValue::Duration(n) => serialize_tagged(serializer, "duration", &n.to_string()),

            // Bytes as lowercase hex.
            DecodedValue::Bytes(b) => serialize_tagged(serializer, "bytes", &hex::encode(b)),

            // Nested collections carry their contents under `value`.
            DecodedValue::Vec(items) => serialize_tagged(serializer, "vec", items),
            DecodedValue::Map(entries) => serialize_tagged(serializer, "map", entries),

            // Void carries nothing beyond its tag.
            DecodedValue::Void => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("type", "void")?;
                map.end()
            }

            // Unknown carries its XDR under `xdr`, not `value`.
            DecodedValue::Unknown { xdr } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "unknown")?;
                map.serialize_entry("xdr", xdr)?;
                map.end()
            }
        }
    }
}

/// The wire shape, read back before conversion to a [`DecodedValue`].
///
/// This mirrors the JSON exactly: narrow integers as numbers, wide integers as
/// strings, `value` for most variants and `xdr` for the fallback. The conversion
/// to `DecodedValue` in [`TryFrom`] is where a stored string becomes a native
/// integer, so a corrupt string is rejected there rather than carried forward.
#[derive(Deserialize)]
#[serde(tag = "type")]
enum Repr {
    #[serde(rename = "address")]
    Address { value: String },
    #[serde(rename = "symbol")]
    Symbol { value: String },
    #[serde(rename = "string")]
    String { value: String },
    #[serde(rename = "bool")]
    Bool { value: bool },
    #[serde(rename = "bytes")]
    Bytes { value: String },
    #[serde(rename = "u32")]
    U32 { value: u32 },
    #[serde(rename = "i32")]
    I32 { value: i32 },
    #[serde(rename = "u64")]
    U64 { value: String },
    #[serde(rename = "i64")]
    I64 { value: String },
    #[serde(rename = "u128")]
    U128 { value: String },
    #[serde(rename = "i128")]
    I128 { value: String },
    #[serde(rename = "u256")]
    U256 { value: String },
    #[serde(rename = "i256")]
    I256 { value: String },
    #[serde(rename = "timepoint")]
    Timepoint { value: String },
    #[serde(rename = "duration")]
    Duration { value: String },
    #[serde(rename = "vec")]
    Vec { value: Vec<DecodedValue> },
    #[serde(rename = "map")]
    Map { value: Vec<MapEntry> },
    #[serde(rename = "void")]
    Void,
    #[serde(rename = "unknown")]
    Unknown { xdr: String },
}

impl TryFrom<Repr> for DecodedValue {
    type Error = String;

    fn try_from(repr: Repr) -> Result<Self, Self::Error> {
        // Parses a decimal string into a native integer, naming the field so a
        // corrupt stored value points at itself.
        fn parse<T>(field: &str, value: &str) -> Result<T, String>
        where
            T: std::str::FromStr,
            T::Err: std::fmt::Display,
        {
            value
                .parse::<T>()
                .map_err(|e| format!("{field} is not a valid integer: {e}"))
        }

        Ok(match repr {
            Repr::Address { value } => DecodedValue::Address(value),
            Repr::Symbol { value } => DecodedValue::Symbol(value),
            Repr::String { value } => DecodedValue::String(value),
            Repr::Bool { value } => DecodedValue::Bool(value),
            Repr::Bytes { value } => DecodedValue::Bytes(hex::decode(&value)?),
            Repr::U32 { value } => DecodedValue::U32(value),
            Repr::I32 { value } => DecodedValue::I32(value),
            Repr::U64 { value } => DecodedValue::U64(parse("u64", &value)?),
            Repr::I64 { value } => DecodedValue::I64(parse("i64", &value)?),
            Repr::U128 { value } => DecodedValue::U128(parse("u128", &value)?),
            Repr::I128 { value } => DecodedValue::I128(parse("i128", &value)?),
            Repr::U256 { value } => DecodedValue::U256(value),
            Repr::I256 { value } => DecodedValue::I256(value),
            Repr::Timepoint { value } => DecodedValue::Timepoint(parse("timepoint", &value)?),
            Repr::Duration { value } => DecodedValue::Duration(parse("duration", &value)?),
            Repr::Vec { value } => DecodedValue::Vec(value),
            Repr::Map { value } => DecodedValue::Map(value),
            Repr::Void => DecodedValue::Void,
            Repr::Unknown { xdr } => DecodedValue::Unknown { xdr },
        })
    }
}

impl<'de> Deserialize<'de> for DecodedValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let repr = Repr::deserialize(deserializer)?;
        DecodedValue::try_from(repr).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::{DecodedValue, MapEntry};
    use serde_json::json;

    /// Asserts that a value serializes to the given JSON and reads back equal.
    fn round_trips(value: &DecodedValue, expected: serde_json::Value) {
        let encoded = serde_json::to_value(value).expect("serializes");
        assert_eq!(encoded, expected, "wire shape");
        let decoded: DecodedValue =
            serde_json::from_value(expected).expect("deserializes from its wire shape");
        assert_eq!(&decoded, value, "round trip");
    }

    #[test]
    fn address_carries_a_string() {
        round_trips(
            &DecodedValue::Address("GABC".to_string()),
            json!({"type": "address", "value": "GABC"}),
        );
    }

    #[test]
    fn symbol_and_string_are_distinct_types() {
        round_trips(
            &DecodedValue::Symbol("transfer".to_string()),
            json!({"type": "symbol", "value": "transfer"}),
        );
        round_trips(
            &DecodedValue::String("transfer".to_string()),
            json!({"type": "string", "value": "transfer"}),
        );
    }

    #[test]
    fn bool_carries_a_json_boolean() {
        round_trips(
            &DecodedValue::Bool(true),
            json!({"type": "bool", "value": true}),
        );
    }

    #[test]
    fn narrow_integers_are_json_numbers() {
        round_trips(
            &DecodedValue::U32(4_294_967_295),
            json!({"type": "u32", "value": 4_294_967_295u32}),
        );
        round_trips(
            &DecodedValue::I32(-2_147_483_648),
            json!({"type": "i32", "value": -2_147_483_648i32}),
        );
    }

    #[test]
    fn wide_integers_are_decimal_strings() {
        round_trips(
            &DecodedValue::U64(u64::MAX),
            json!({"type": "u64", "value": "18446744073709551615"}),
        );
        round_trips(
            &DecodedValue::I64(i64::MIN),
            json!({"type": "i64", "value": "-9223372036854775808"}),
        );
        round_trips(
            &DecodedValue::U128(u128::MAX),
            json!({"type": "u128", "value": "340282366920938463463374607431768211455"}),
        );
        round_trips(
            &DecodedValue::I128(i128::MIN),
            json!({"type": "i128", "value": "-170141183460469231731687303715884105728"}),
        );
    }

    #[test]
    fn wide_integer_strings_survive_a_json_number_boundary() {
        // 2^53 + 1 is the first integer a JSON number cannot represent exactly.
        // Carried as a string, it must come back bit for bit.
        round_trips(
            &DecodedValue::U64(9_007_199_254_740_993),
            json!({"type": "u64", "value": "9007199254740993"}),
        );
    }

    #[test]
    fn two_fifty_six_bit_integers_are_opaque_decimal_strings() {
        let big = "115792089237316195423570985008687907853269984665640564039457584007913129639935";
        round_trips(
            &DecodedValue::U256(big.to_string()),
            json!({"type": "u256", "value": big}),
        );
        round_trips(
            &DecodedValue::I256("-1".to_string()),
            json!({"type": "i256", "value": "-1"}),
        );
    }

    #[test]
    fn timepoint_and_duration_are_second_counts_as_strings() {
        round_trips(
            &DecodedValue::Timepoint(1_700_000_000),
            json!({"type": "timepoint", "value": "1700000000"}),
        );
        round_trips(
            &DecodedValue::Duration(3600),
            json!({"type": "duration", "value": "3600"}),
        );
    }

    #[test]
    fn bytes_are_lowercase_hex() {
        round_trips(
            &DecodedValue::Bytes(vec![0xde, 0xad, 0xbe, 0xef]),
            json!({"type": "bytes", "value": "deadbeef"}),
        );
    }

    #[test]
    fn empty_bytes_are_an_empty_hex_string() {
        round_trips(
            &DecodedValue::Bytes(vec![]),
            json!({"type": "bytes", "value": ""}),
        );
    }

    #[test]
    fn void_carries_only_its_type() {
        round_trips(&DecodedValue::Void, json!({"type": "void"}));
    }

    #[test]
    fn unknown_carries_xdr_not_value() {
        round_trips(
            &DecodedValue::Unknown {
                xdr: "AAAAAQ==".to_string(),
            },
            json!({"type": "unknown", "xdr": "AAAAAQ=="}),
        );
    }

    #[test]
    fn empty_vec_is_an_empty_array() {
        round_trips(
            &DecodedValue::Vec(vec![]),
            json!({"type": "vec", "value": []}),
        );
    }

    #[test]
    fn vec_preserves_order() {
        round_trips(
            &DecodedValue::Vec(vec![
                DecodedValue::U32(1),
                DecodedValue::U32(2),
                DecodedValue::U32(3),
            ]),
            json!({"type": "vec", "value": [
                {"type": "u32", "value": 1},
                {"type": "u32", "value": 2},
                {"type": "u32", "value": 3},
            ]}),
        );
    }

    #[test]
    fn empty_map_is_an_empty_array() {
        round_trips(
            &DecodedValue::Map(vec![]),
            json!({"type": "map", "value": []}),
        );
    }

    #[test]
    fn map_keeps_non_string_keys_order_and_duplicates() {
        // A symbol key and an address key coexist, one key appears twice, and
        // wire order is meaningful. All three are the reason a map is an ordered
        // list rather than a keyed collection.
        let value = DecodedValue::Map(vec![
            MapEntry {
                key: DecodedValue::Symbol("admin".to_string()),
                value: DecodedValue::Bool(true),
            },
            MapEntry {
                key: DecodedValue::U32(1),
                value: DecodedValue::Address("GABC".to_string()),
            },
            MapEntry {
                key: DecodedValue::Symbol("admin".to_string()),
                value: DecodedValue::Bool(false),
            },
        ]);
        round_trips(
            &value,
            json!({"type": "map", "value": [
                {"key": {"type": "symbol", "value": "admin"}, "value": {"type": "bool", "value": true}},
                {"key": {"type": "u32", "value": 1}, "value": {"type": "address", "value": "GABC"}},
                {"key": {"type": "symbol", "value": "admin"}, "value": {"type": "bool", "value": false}},
            ]}),
        );
    }

    #[test]
    fn nested_collections_round_trip() {
        let value = DecodedValue::Vec(vec![
            DecodedValue::Map(vec![MapEntry {
                key: DecodedValue::Symbol("k".to_string()),
                value: DecodedValue::Vec(vec![DecodedValue::I128(-5), DecodedValue::Void]),
            }]),
            DecodedValue::Bytes(vec![0x01]),
        ]);
        let encoded = serde_json::to_value(&value).expect("serializes");
        let decoded: DecodedValue = serde_json::from_value(encoded).expect("deserializes");
        assert_eq!(decoded, value);
    }

    #[test]
    fn type_name_matches_the_wire_discriminant() {
        assert_eq!(DecodedValue::Void.type_name(), "void");
        assert_eq!(DecodedValue::U128(0).type_name(), "u128");
        assert_eq!(
            DecodedValue::Unknown { xdr: String::new() }.type_name(),
            "unknown"
        );
    }

    #[test]
    fn deserialize_rejects_a_non_numeric_wide_integer() {
        let err =
            serde_json::from_value::<DecodedValue>(json!({"type": "u64", "value": "not a number"}));
        assert!(err.is_err(), "a corrupt u64 string must not decode");
    }

    #[test]
    fn deserialize_rejects_a_negative_unsigned_integer() {
        let err = serde_json::from_value::<DecodedValue>(json!({"type": "u64", "value": "-1"}));
        assert!(err.is_err(), "a negative value must not decode as u64");
    }

    #[test]
    fn deserialize_rejects_malformed_hex_bytes() {
        let err = serde_json::from_value::<DecodedValue>(json!({"type": "bytes", "value": "zz"}));
        assert!(err.is_err(), "non-hex bytes must not decode");
    }

    #[test]
    fn deserialize_rejects_an_unknown_type_discriminant() {
        let err = serde_json::from_value::<DecodedValue>(json!({"type": "quantum", "value": 1}));
        assert!(err.is_err(), "an unrecognized type tag must not decode");
    }
}
