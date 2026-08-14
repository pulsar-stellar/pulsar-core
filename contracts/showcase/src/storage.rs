//! Storage keys and access helpers for the showcase contract.
//!
//! Storage class per key follows ADR-004. `Initialized` and `Admin` are small,
//! bounded, and read on nearly every call, so they live in instance storage and
//! share the instance TTL. `Balance` grows with the number of addresses that
//! ever hold funds, so each entry lives in persistent storage under its own TTL.
//! Temporary storage is deliberately unused: nothing this contract stores is
//! disposable.

use soroban_sdk::{contracttype, Address};

/// Every key the showcase contract stores under.
///
/// The variant set is closed. A new kind of state needs a new variant here and
/// an ADR recording which storage class it belongs in and why.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Instance storage. Set once by `initialize` and never cleared, so its
    /// presence is what distinguishes an initialized contract from a fresh one.
    Initialized,
    /// Instance storage. The address authorized for admin-only operations.
    Admin,
    /// Persistent storage, one entry per address. Absent means a zero balance;
    /// the contract never writes an explicit zero.
    Balance(Address),
}
