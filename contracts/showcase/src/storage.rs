//! Storage keys and access helpers for the showcase contract.
//!
//! Storage class per key follows ADR-004. `Initialized` and `Admin` are small,
//! bounded, and read on nearly every call, so they live in instance storage and
//! share the instance TTL. `Balance` grows with the number of addresses that
//! ever hold funds, so each entry lives in persistent storage under its own TTL.
//! Temporary storage is deliberately unused: nothing this contract stores is
//! disposable.

// #![allow(dead_code)] necessary through Phase B because storage helpers land
// before their callers in contract.rs (step 27). Remove this line as part of
// the step 27 commit when contract.rs adds the calls.
#![allow(dead_code)]

use soroban_sdk::{contracttype, Address, Env};

use crate::error::Error;

/// Ledgers in a day at Stellar's roughly five second close time.
///
/// TTL is counted in ledgers, not wall-clock time, so every duration below is
/// expressed as a multiple of this rather than as a bare number.
pub(crate) const DAY_IN_LEDGERS: u32 = 17_280;

/// Extend the instance TTL to seven days when it is bumped.
///
/// Seven days follows the convention in `stellar/soroban-examples/token`, which
/// is what real protocol contracts use. Instance state is small and gets bumped
/// on nearly every call, so a shorter window costs little and keeps archived
/// instances from lingering. This contract is a reference developers copy
/// patterns from, which is why it matches the ecosystem convention rather than
/// picking its own number.
///
/// ADR-004 specified thirty days uniformly for instance and persistent storage.
/// ADR-012 corrects that at the end of Phase B.
pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;

/// Bump the instance only once its remaining life drops below this.
///
/// Sitting one day under the bump amount means a contract called at least daily
/// pays for one extension per day rather than one per call.
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

/// Extend a persistent entry's TTL to thirty days when it is bumped.
///
/// Balances stay at thirty days per ADR-004. A balance can go untouched far
/// longer than the contract instance does, and an archived balance costs a
/// restore before the holder can use it again.
pub(crate) const PERSISTENT_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;

/// Bump a persistent entry only once its remaining life drops below this.
pub(crate) const PERSISTENT_LIFETIME_THRESHOLD: u32 = PERSISTENT_BUMP_AMOUNT - DAY_IN_LEDGERS;

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

/// Extend the contract instance TTL.
///
/// Called at the top of every state-changing public function, per ADR-004. The
/// extension covers both the instance entry and the contract code, and the SDK
/// skips it when remaining life is still above the threshold, so calling it
/// unconditionally costs nothing on a contract already in good standing.
pub(crate) fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

/// Read the admin address.
///
/// Returns `NotInitialized` when no admin is stored, which is the case for a
/// deployed but uninitialized contract. Converting the absent case into a typed
/// error here means callers never see an `Option` and never have a reason to
/// unwrap one.
///
/// Does not extend the instance TTL. Callers that change state already extend it
/// at their entry point, and read views deliberately do not, per ADR-004.
pub(crate) fn get_admin(env: &Env) -> Result<Address, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(Error::NotInitialized)
}
