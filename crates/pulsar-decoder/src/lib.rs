//! Typed decoder for Soroban contract events.
//!
//! This crate turns the raw `ScVal` values a Soroban contract emits into the
//! [`DecodedValue`] taxonomy that the rest of the Pulsar toolkit reads: the Go
//! indexer, the TypeScript SDK, and the web explorer all depend on it agreeing
//! with what contracts actually emit. That shared wire contract is fixed by
//! pulsar-app ADR-023, so a change to the taxonomy here is a coordinated change
//! across every consumer.
//!
//! # What this crate decodes
//!
//! [`DecodedValue`] is a discriminated union keyed by a `type` string. It
//! serializes to, and deserializes from, exactly the JSON the other
//! implementations already produce, so a value stored by the indexer round trips
//! through this crate unchanged. Integers wider than 53 bits are carried as
//! decimal strings rather than JSON numbers, which would round silently.
//!
//! A value this crate cannot name does not become an error: it becomes
//! [`DecodedValue::Unknown`], carrying its base64 XDR intact, so a protocol
//! upgrade that adds an `ScVal` variant does not stop an old consumer from
//! reading new data.
//!
//! # Scope
//!
//! This crate decodes values and topic lists, and assembles an event from parts
//! a caller already holds. It does not parse RPC envelopes and makes no network
//! calls; fetching events belongs to the indexer. See the workspace ADR log for
//! the recorded scope boundary.
//!
//! # Panics
//!
//! Nothing in this crate's public API panics on malformed input. Decoding is
//! fallible and total: bad bytes return an error, and an unnameable value
//! degrades to [`DecodedValue::Unknown`].

mod hex;
mod value;

pub use value::{DecodedValue, MapEntry};
