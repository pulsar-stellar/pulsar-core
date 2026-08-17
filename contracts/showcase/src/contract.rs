//! The showcase contract's public interface.
//!
//! Every function on this type exists to exercise a specific decoder capability
//! rather than to model a product. A function that does not produce an event
//! shape worth decoding does not belong here, which is what keeps the surface at
//! six state-changing functions and two read views.
//!
//! Public functions are added one at a time, each with its tests, so the
//! interface grows in reviewable units rather than arriving whole.

use soroban_sdk::{contract, contractimpl};

/// The showcase contract.
///
/// Deployed to testnet as the toolkit's reference contract: the events it emits
/// are the fixtures the decoder, indexer, and explorer are tested against, and
/// its contract ID is what a developer pastes into the explorer to see decoded
/// history end to end.
#[contract]
pub struct PulsarShowcase;

#[contractimpl]
impl PulsarShowcase {}
