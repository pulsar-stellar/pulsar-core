//! Error type for the showcase contract.
//!
//! Discriminants are part of the contract's public interface. A caller that
//! traps sees the number, not the name, so once this contract is deployed an
//! existing variant's value is never reused or renumbered, and new variants take
//! the next free number. Before the first deployment the numbering is still
//! malleable, which is why the set could be compacted after `Unauthorized` was
//! removed.

use soroban_sdk::contracterror;

/// Every failure the showcase contract can return.
///
/// Each variant is reachable from at least one public function and is covered by
/// a test that triggers it. Failed authorization is deliberately absent: a
/// `require_auth` rejection is raised by the host and never reaches this type,
/// so a variant for it would be permanently unreachable. See ADR-018.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// `initialize` was called on a contract that already holds an admin.
    AlreadyInitialized = 1,
    /// A function needing contract state ran before `initialize` set it up.
    NotInitialized = 2,
    /// The withdrawal or transfer exceeds the sender's stored balance.
    InsufficientBalance = 3,
    /// An amount argument was zero or negative, or the operation's result would
    /// fall outside the range an `i128` balance can hold. Both are the same
    /// failure from a caller's view: the amount cannot be applied as given.
    AmountOutOfRange = 4,
}
