//! The [`Decoder`]: parsed `ScVal` in, [`DecodedValue`] out.
//!
//! This mirrors the Go decoder in pulsar-app (`internal/decoder`), which is the
//! reference implementation for the shared wire contract. The rules it enforces
//! come from pulsar-app ADR-023 and ADR-033:
//!
//! - A timepoint or duration is read from its raw second count, never through
//!   `ScVal`'s `Display`, which would render a timepoint as a formatted local
//!   date and carry the host's timezone into stored data.
//! - Wide integers become native `u128`/`i128` (reconstructed from their hi/lo
//!   halves) or, for 256-bit, the decimal string their XDR parts render to.
//! - A variant this decoder cannot represent is not an error: it degrades to
//!   [`DecodedValue::Unknown`], re-encoding the parsed value to base64 so nothing
//!   is lost, and a counter records that it happened.

use std::sync::atomic::{AtomicI64, Ordering};

use stellar_xdr::curr::{Limits, ReadXdr, ScVal, WriteXdr};

use crate::error::DecodeError;
use crate::value::{DecodedValue, MapEntry};

/// Maximum XDR nesting depth accepted when parsing untrusted input.
///
/// The depth limit is a stack-overflow guard: without it, a deeply nested value
/// crafted by a malicious contract could exhaust the stack and abort the process
/// with an unrecoverable `SIGABRT`. 500 matches the Soroban host's own recursion
/// limit, so any value the network accepts this decoder also accepts.
const MAX_DEPTH: u32 = 500;

/// Stands in for the XDR of a parsed value that will not re-encode.
///
/// A parsed `ScVal` should always re-encode, so this is unreachable in practice;
/// it exists because [`DecodedValue::Unknown`] must carry a non-empty base64
/// string and re-encoding is nonetheless fallible. `"="` is valid base64 that
/// decodes to the empty slice, which no real `ScVal` produces, so it cannot be
/// mistaken for a genuine value. This matches the Go decoder's marker exactly.
const UNENCODABLE_MARKER: &str = "=";

/// Converts parsed `ScVal`s into the [`DecodedValue`] taxonomy and counts how
/// often it falls back to [`DecodedValue::Unknown`].
///
/// The counter exists because degrading is silent by design: a protocol upgrade
/// that adds an `ScVal` variant produces correct-looking events full of unknowns,
/// and nothing else would say so. A caller can read [`unknown_count`] and surface
/// a sudden rise in operation rather than have a downstream consumer discover it.
///
/// [`unknown_count`]: Decoder::unknown_count
///
/// The counter is atomic, so a shared `&Decoder` may be used to decode
/// concurrently from several threads.
#[derive(Debug, Default)]
pub struct Decoder {
    unknown_count: AtomicI64,
}

impl Decoder {
    /// Returns a new decoder with a zeroed unknown counter.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reports how many values have degraded to [`DecodedValue::Unknown`] over
    /// this decoder's lifetime.
    #[must_use]
    pub fn unknown_count(&self) -> i64 {
        self.unknown_count.load(Ordering::Relaxed)
    }

    /// Decodes one base64 XDR value, as it arrives from RPC.
    ///
    /// # Errors
    ///
    /// Returns [`DecodeError::InvalidXdr`] if `encoded` is not parseable as an
    /// `ScVal`. A value that parses but names a variant this decoder cannot
    /// represent is not an error: it becomes [`DecodedValue::Unknown`].
    pub fn decode_base64(&self, encoded: &str) -> Result<DecodedValue, DecodeError> {
        // Bound the parse against untrusted input: cap nesting depth, and cap
        // the byte length at the input's own length, since the decoded XDR can
        // never be larger than the base64 that carries it.
        let limits = Limits {
            depth: MAX_DEPTH,
            len: encoded.len(),
        };
        let value = ScVal::from_xdr_base64(encoded, limits).map_err(DecodeError::InvalidXdr)?;
        Ok(self.decode(&value))
    }

    /// Decodes an event's topic list, preserving order.
    ///
    /// # Errors
    ///
    /// Returns [`DecodeError::InvalidTopic`] carrying the zero-based index of
    /// the first topic that is not parseable `ScVal` XDR.
    pub fn decode_topics(&self, encoded: &[String]) -> Result<Vec<DecodedValue>, DecodeError> {
        let mut topics = Vec::with_capacity(encoded.len());
        for (index, raw) in encoded.iter().enumerate() {
            let value = self.decode_base64(raw).map_err(|e| e.in_topic(index))?;
            topics.push(value);
        }
        Ok(topics)
    }

    /// Converts one parsed `ScVal`. Never fails and never panics: a variant this
    /// decoder cannot name becomes [`DecodedValue::Unknown`] carrying its XDR, so
    /// one unrepresentable value does not discard the rest of a page.
    #[must_use]
    pub fn decode(&self, value: &ScVal) -> DecodedValue {
        match self.try_named(value) {
            Some(named) => named,
            None => self.unknown(value),
        }
    }

    /// Decodes the variants this decoder represents, returning `None` for any it
    /// does not so [`decode`](Self::decode) can fall back to the unknown branch.
    fn try_named(&self, value: &ScVal) -> Option<DecodedValue> {
        Some(match value {
            ScVal::Bool(b) => DecodedValue::Bool(*b),
            ScVal::Void => DecodedValue::Void,
            ScVal::U32(n) => DecodedValue::U32(*n),
            ScVal::I32(n) => DecodedValue::I32(*n),
            ScVal::U64(n) => DecodedValue::U64(*n),
            ScVal::I64(n) => DecodedValue::I64(*n),
            // Raw second counts, not Display. See ADR-033.
            ScVal::Timepoint(t) => DecodedValue::Timepoint(t.0),
            ScVal::Duration(d) => DecodedValue::Duration(d.0),
            // Reassemble the native integer from its two 64-bit halves.
            ScVal::U128(parts) => {
                DecodedValue::U128((u128::from(parts.hi) << 64) | u128::from(parts.lo))
            }
            ScVal::I128(parts) => {
                DecodedValue::I128((i128::from(parts.hi) << 64) | i128::from(parts.lo))
            }
            // Rust has no native 256-bit type; the XDR parts render to decimal.
            ScVal::U256(parts) => DecodedValue::U256(parts.to_string()),
            ScVal::I256(parts) => DecodedValue::I256(parts.to_string()),
            ScVal::Bytes(bytes) => DecodedValue::Bytes(bytes.to_vec()),
            // A string may hold arbitrary bytes; a Rust String cannot, so invalid
            // UTF-8 is replaced lossily rather than made an error. A symbol is
            // always valid UTF-8 by Soroban's own rule, so it is unaffected.
            ScVal::String(s) => DecodedValue::String(s.to_utf8_string_lossy()),
            ScVal::Symbol(s) => DecodedValue::Symbol(s.to_utf8_string_lossy()),
            ScVal::Address(address) => {
                // ScAddress renders to strkey infallibly; guard the empty string
                // only so a degenerate address degrades rather than decodes.
                let rendered = address.to_string();
                if rendered.is_empty() {
                    return None;
                }
                DecodedValue::Address(rendered)
            }
            ScVal::Vec(Some(items)) => {
                DecodedValue::Vec(items.iter().map(|item| self.decode(item)).collect())
            }
            ScVal::Map(Some(entries)) => DecodedValue::Map(
                entries
                    .iter()
                    .map(|entry| MapEntry {
                        key: self.decode(&entry.key),
                        value: self.decode(&entry.val),
                    })
                    .collect(),
            ),
            // Everything else degrades: an error value, an absent (None) vec or
            // map, a contract instance, and the ledger-key variants. Mirrors the
            // Go decoder, which names none of these.
            _ => return None,
        })
    }

    /// Produces the [`DecodedValue::Unknown`] fallback, bumping the counter and
    /// re-encoding the parsed value to base64.
    ///
    /// Re-encoding rather than threading the original base64 through every call
    /// reproduces the input byte for byte (verified in pulsar-app ADR-033), and
    /// keeps [`decode`](Self::decode) infallible for a value it already holds.
    fn unknown(&self, value: &ScVal) -> DecodedValue {
        self.unknown_count.fetch_add(1, Ordering::Relaxed);
        // A parsed value re-encodes without bound, so Limits::none() is safe here
        // and avoids re-imposing a cap on data that already passed one on parse.
        let xdr = value
            .to_xdr_base64(Limits::none())
            .unwrap_or_else(|_| UNENCODABLE_MARKER.to_string());
        DecodedValue::Unknown { xdr }
    }
}

#[cfg(test)]
mod tests {
    use super::{Decoder, MAX_DEPTH};
    use crate::DecodedValue;
    use stellar_xdr::curr::{
        Int128Parts, Limits, ScAddress, ScBytes, ScMap, ScMapEntry, ScString, ScSymbol, ScVal,
        ScVec, UInt128Parts, WriteXdr,
    };

    /// Base64-encodes an `ScVal` the way it would arrive from RPC.
    fn to_base64(value: &ScVal) -> String {
        value
            .to_xdr_base64(Limits::none())
            .expect("a constructed ScVal re-encodes")
    }

    #[test]
    fn decodes_a_bool() {
        let decoder = Decoder::new();
        let got = decoder
            .decode_base64(&to_base64(&ScVal::Bool(true)))
            .expect("valid xdr decodes");
        assert_eq!(got, DecodedValue::Bool(true));
    }

    #[test]
    fn reconstructs_a_u128_from_its_halves() {
        // hi = 1, lo = 0 is exactly 2^64, a value neither half holds alone.
        let value = ScVal::U128(UInt128Parts { hi: 1, lo: 0 });
        let decoder = Decoder::new();
        let got = decoder.decode(&value);
        assert_eq!(got, DecodedValue::U128(1u128 << 64));
    }

    #[test]
    fn reconstructs_a_negative_i128_from_its_halves() {
        // -2 as a 128-bit two's-complement value: every bit set except the low
        // one. Its halves are hi = -1 and lo = u64::MAX - 1.
        let value = ScVal::I128(Int128Parts {
            hi: -1,
            lo: u64::MAX - 1,
        });
        let decoder = Decoder::new();
        assert_eq!(decoder.decode(&value), DecodedValue::I128(-2));
    }

    #[test]
    fn timepoint_is_a_raw_second_count() {
        use stellar_xdr::curr::TimePoint;
        let value = ScVal::Timepoint(TimePoint(1_700_000_000));
        let decoder = Decoder::new();
        assert_eq!(
            decoder.decode(&value),
            DecodedValue::Timepoint(1_700_000_000)
        );
    }

    #[test]
    fn bytes_carry_their_raw_octets() {
        let bytes: ScBytes = vec![0xde, 0xad, 0xbe, 0xef].try_into().expect("fits");
        let value = ScVal::Bytes(bytes);
        let decoder = Decoder::new();
        assert_eq!(
            decoder.decode(&value),
            DecodedValue::Bytes(vec![0xde, 0xad, 0xbe, 0xef])
        );
    }

    #[test]
    fn symbol_and_string_stay_distinct() {
        let symbol = ScVal::Symbol(ScSymbol("transfer".try_into().expect("fits")));
        let string = ScVal::String(ScString("transfer".try_into().expect("fits")));
        let decoder = Decoder::new();
        assert_eq!(
            decoder.decode(&symbol),
            DecodedValue::Symbol("transfer".to_string())
        );
        assert_eq!(
            decoder.decode(&string),
            DecodedValue::String("transfer".to_string())
        );
    }

    #[test]
    fn an_empty_vec_decodes_to_an_empty_vec() {
        let value = ScVal::Vec(Some(ScVec(Vec::new().try_into().expect("fits"))));
        let decoder = Decoder::new();
        assert_eq!(decoder.decode(&value), DecodedValue::Vec(Vec::new()));
    }

    #[test]
    fn a_nested_vec_recurses() {
        let inner = ScVal::Vec(Some(ScVec(vec![ScVal::U32(7)].try_into().expect("fits"))));
        let outer = ScVal::Vec(Some(ScVec(vec![inner].try_into().expect("fits"))));
        let decoder = Decoder::new();
        assert_eq!(
            decoder.decode(&outer),
            DecodedValue::Vec(vec![DecodedValue::Vec(vec![DecodedValue::U32(7)])])
        );
    }

    #[test]
    fn a_map_preserves_entry_order_and_non_string_keys() {
        let entries = vec![
            ScMapEntry {
                key: ScVal::U32(1),
                val: ScVal::Bool(true),
            },
            ScMapEntry {
                key: ScVal::Symbol(ScSymbol("k".try_into().expect("fits"))),
                val: ScVal::Void,
            },
        ];
        let value = ScVal::Map(Some(ScMap(entries.try_into().expect("fits"))));
        let decoder = Decoder::new();
        match decoder.decode(&value) {
            DecodedValue::Map(entries) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].key, DecodedValue::U32(1));
                assert_eq!(entries[0].value, DecodedValue::Bool(true));
                assert_eq!(entries[1].key, DecodedValue::Symbol("k".to_string()));
            }
            other => panic!("expected a map, got {other:?}"),
        }
    }

    #[test]
    fn an_error_value_degrades_to_unknown_and_counts() {
        use stellar_xdr::curr::{ScError, ScErrorCode};
        let value = ScVal::Error(ScError::Context(ScErrorCode::InternalError));
        let decoder = Decoder::new();
        assert_eq!(decoder.unknown_count(), 0);
        match decoder.decode(&value) {
            DecodedValue::Unknown { xdr } => assert!(!xdr.is_empty(), "unknown carries its xdr"),
            other => panic!("an error value must be unknown, got {other:?}"),
        }
        assert_eq!(decoder.unknown_count(), 1);
        // A second fallback advances the counter again.
        let _ = decoder.decode(&ScVal::Error(ScError::Context(ScErrorCode::InvalidInput)));
        assert_eq!(decoder.unknown_count(), 2);
    }

    #[test]
    fn an_absent_vec_degrades_to_unknown() {
        let decoder = Decoder::new();
        match decoder.decode(&ScVal::Vec(None)) {
            DecodedValue::Unknown { .. } => {}
            other => panic!("a None vec must be unknown, got {other:?}"),
        }
    }

    #[test]
    fn a_named_value_does_not_touch_the_counter() {
        let decoder = Decoder::new();
        let _ = decoder.decode(&ScVal::U32(1));
        assert_eq!(decoder.unknown_count(), 0);
    }

    #[test]
    fn decode_base64_rejects_bytes_that_are_not_xdr() {
        let decoder = Decoder::new();
        assert!(decoder.decode_base64("not valid base64 xdr !!!").is_err());
    }

    #[test]
    fn decode_topics_preserves_order() {
        let decoder = Decoder::new();
        let topics = vec![
            to_base64(&ScVal::Symbol(ScSymbol(
                "transfer".try_into().expect("fits"),
            ))),
            to_base64(&ScVal::U32(42)),
        ];
        let result = decoder.decode_topics(&topics).expect("valid topics decode");
        assert_eq!(
            result,
            vec![
                DecodedValue::Symbol("transfer".to_string()),
                DecodedValue::U32(42),
            ]
        );
    }

    #[test]
    fn decode_topics_reports_the_failing_index() {
        use crate::error::DecodeError;
        let decoder = Decoder::new();
        let topics = vec![to_base64(&ScVal::U32(1)), "garbage !!!".to_string()];
        match decoder.decode_topics(&topics) {
            Err(DecodeError::InvalidTopic { index, .. }) => assert_eq!(index, 1),
            other => panic!("expected topic 1 to fail, got {other:?}"),
        }
    }

    #[test]
    fn an_address_renders_to_strkey() {
        use stellar_xdr::curr::{AccountId, PublicKey, Uint256};
        let account = ScAddress::Account(AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(
            [0u8; 32],
        ))));
        let decoder = Decoder::new();
        match decoder.decode(&ScVal::Address(account)) {
            DecodedValue::Address(s) => assert!(
                s.starts_with('G'),
                "an account strkey starts with G, got {s:?}"
            ),
            other => panic!("expected an address, got {other:?}"),
        }
    }

    #[test]
    fn max_depth_matches_the_soroban_host_limit() {
        assert_eq!(MAX_DEPTH, 500);
    }
}
