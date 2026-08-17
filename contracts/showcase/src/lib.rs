//! Reference Soroban contract for the Pulsar toolkit.
//!
//! The events this contract emits are the fixtures the decoder, indexer, and
//! explorer are tested against, so its public surface is deliberately narrow:
//! the contract type and the error type callers can receive.
#![no_std]

mod contract;
mod error;
mod events;
mod storage;

/// The contract type and the client `#[contractimpl]` generates for it. Tests
/// and other crates invoke the contract through the client.
pub use contract::{PulsarShowcase, PulsarShowcaseClient};

/// Every failure a caller can receive. Exported so callers can match on the
/// variant rather than on a bare error code.
pub use error::Error;
