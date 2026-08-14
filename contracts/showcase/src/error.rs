//! Error type for the showcase contract.
//!
//! Discriminants are part of the contract's public interface. A caller that
//! traps sees the number, not the name, so an existing variant's value is never
//! reused or renumbered. New variants take the next free number.

use soroban_sdk::contracterror;

/// Every failure the showcase contract can return.
///
/// Each variant is reachable from at least one public function and is covered
/// by a test that triggers it.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// `initialize` was called on a contract that already holds an admin.
    AlreadyInitialized = 1,
    /// A function needing contract state ran before `initialize` set it up.
    NotInitialized = 2,
    /// The caller is not the admin this operation requires.
    Unauthorized = 3,
    /// The withdrawal or transfer exceeds the sender's stored balance.
    InsufficientBalance = 4,
    /// An amount argument was zero or negative. Amounts must be positive.
    InvalidAmount = 5,
}
