//! Reference contract crate. Modules land across build steps 14 through 27; re-exports finalize at step 28.
#![no_std]

mod contract;
mod error;
mod events;
mod storage;

/// The contract type, exported so tests and the generated client can reach it.
pub use contract::PulsarShowcase;
