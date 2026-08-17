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
    ///
    /// Declared but unread until Phase C step 29, where `initialize` writes it
    /// through `set_initialized` and reads it through `is_initialized` to tell
    /// `AlreadyInitialized` apart from `NotInitialized`. Those helpers land with
    /// their caller rather than ahead of it, per ADR-011. Note that `get_admin`
    /// does not consult this flag: an absent `Admin` already implies an
    /// uninitialized contract, since the two are written together.
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

/// Write the admin address.
///
/// Deliberately does only the write. Instance storage shares one TTL with the
/// contract instance, so the bump belongs at the public function's entry point
/// through `extend_instance_ttl`, not here. Bumping in both places would be a
/// redundant call that splits one responsibility across two layers. Contrast
/// `set_balance`, where a persistent entry carries its own TTL and the extension
/// does belong beside the write.
pub(crate) fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

/// Read an address's balance.
///
/// An absent entry means zero. The contract never writes an explicit zero, so
/// "no entry" and "zero balance" are the same state and `unwrap_or(0)` is the
/// honest reading of it rather than a swallowed error.
///
/// Extends the entry's TTL only when the balance is positive. A persistent entry
/// carries its own TTL, so a read that returns live value is exactly the moment
/// worth paying to keep it alive. Extending on a zero read would create TTL cost
/// for an entry that does not exist.
pub(crate) fn get_balance(env: &Env, addr: &Address) -> i128 {
    let key = DataKey::Balance(addr.clone());
    let balance: i128 = env.storage().persistent().get(&key).unwrap_or(0);

    if balance > 0 {
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    balance
}

/// Write an address's balance.
///
/// Extends the entry's TTL immediately after the write, per ADR-013. A
/// persistent entry carries its own TTL, so the bump belongs beside the write
/// that creates or refreshes it. This is the deliberate counterpart to
/// `set_admin`, which does not extend, because instance storage shares one TTL
/// bumped once at the public function's entry point.
///
/// The extension is unconditional here, unlike in `get_balance`. A write always
/// leaves a live entry worth keeping, including a write of zero, which is why
/// callers avoid storing explicit zeros in the first place.
pub(crate) fn set_balance(env: &Env, addr: &Address, amount: i128) {
    let key = DataKey::Balance(addr.clone());
    env.storage().persistent().set(&key, &amount);
    env.storage().persistent().extend_ttl(
        &key,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::storage::Persistent as _;
    use soroban_sdk::testutils::{Address as _, Ledger as _};
    use soroban_sdk::{contract, contractimpl};

    /// Minimal registered contract, present only to give the helpers an instance
    /// context. Instance storage reads fail against an address with no
    /// registered contract behind it, so a harness is required even for tests
    /// that never call a public function.
    #[contract]
    struct Harness;

    #[contractimpl]
    impl Harness {}

    #[test]
    fn get_admin_returns_not_initialized_on_empty_instance_store() {
        let env = Env::default();
        let id = env.register(Harness, ());

        env.as_contract(&id, || {
            assert_eq!(get_admin(&env), Err(Error::NotInitialized));
        });
    }

    #[test]
    fn get_admin_returns_the_stored_admin_once_set() {
        let env = Env::default();
        let id = env.register(Harness, ());

        env.as_contract(&id, || {
            let admin = Address::generate(&env);
            set_admin(&env, &admin);
            assert_eq!(get_admin(&env), Ok(admin));
        });
    }

    #[test]
    fn get_balance_returns_zero_for_an_absent_key() {
        let env = Env::default();
        let id = env.register(Harness, ());

        env.as_contract(&id, || {
            let addr = Address::generate(&env);
            assert_eq!(get_balance(&env, &addr), 0);
            assert!(
                !env.storage()
                    .persistent()
                    .has(&DataKey::Balance(addr.clone())),
                "reading an absent balance must not create an entry"
            );
        });
    }

    #[test]
    fn get_balance_returns_the_written_value() {
        let env = Env::default();
        let id = env.register(Harness, ());

        env.as_contract(&id, || {
            let addr = Address::generate(&env);
            env.storage()
                .persistent()
                .set(&DataKey::Balance(addr.clone()), &4_200_i128);
            assert_eq!(get_balance(&env, &addr), 4_200);
        });
    }

    #[test]
    fn get_balance_extends_ttl_only_when_the_balance_is_positive() {
        let env = Env::default();
        let id = env.register(Harness, ());

        env.as_contract(&id, || {
            let holder = Address::generate(&env);
            let key = DataKey::Balance(holder.clone());

            // Write a positive balance, then let its TTL decay by advancing the
            // ledger far enough to drop it under the extension threshold.
            env.storage().persistent().set(&key, &1_i128);
            env.ledger()
                .set_sequence_number(env.ledger().sequence() + DAY_IN_LEDGERS * 2);
            let decayed = env.storage().persistent().get_ttl(&key);
            assert!(
                decayed < PERSISTENT_BUMP_AMOUNT,
                "TTL should have decayed before the read under test"
            );

            // A positive balance read bumps the entry back to the full window.
            assert_eq!(get_balance(&env, &holder), 1);
            assert_eq!(
                env.storage().persistent().get_ttl(&key),
                PERSISTENT_BUMP_AMOUNT
            );

            // A zero balance read creates nothing and so extends nothing.
            let empty = Address::generate(&env);
            assert_eq!(get_balance(&env, &empty), 0);
            assert!(
                !env.storage().persistent().has(&DataKey::Balance(empty)),
                "a zero read must not create or extend an entry"
            );
        });
    }

    #[test]
    fn set_balance_writes_a_readable_value() {
        let env = Env::default();
        let id = env.register(Harness, ());

        env.as_contract(&id, || {
            let addr = Address::generate(&env);
            set_balance(&env, &addr, 7_500);
            assert_eq!(get_balance(&env, &addr), 7_500);
        });
    }

    #[test]
    fn set_balance_extends_ttl_to_the_full_window() {
        let env = Env::default();
        let id = env.register(Harness, ());

        env.as_contract(&id, || {
            let addr = Address::generate(&env);
            let key = DataKey::Balance(addr.clone());

            set_balance(&env, &addr, 1);
            assert_eq!(
                env.storage().persistent().get_ttl(&key),
                PERSISTENT_BUMP_AMOUNT,
                "the write itself must extend the entry it creates"
            );
        });
    }

    #[test]
    fn set_balance_resets_ttl_when_overwriting_a_decayed_entry() {
        let env = Env::default();
        let id = env.register(Harness, ());

        env.as_contract(&id, || {
            let addr = Address::generate(&env);
            let key = DataKey::Balance(addr.clone());

            // Seed an entry through the SDK so it carries only the default TTL,
            // then let it decay so a reset is observable.
            env.storage().persistent().set(&key, &10_i128);
            env.ledger()
                .set_sequence_number(env.ledger().sequence() + DAY_IN_LEDGERS);
            let decayed = env.storage().persistent().get_ttl(&key);
            assert!(decayed < PERSISTENT_BUMP_AMOUNT);

            // Assert the TTL before any read. get_balance extends on a positive
            // balance too, so reading first would let its extension stand in for
            // the one under test here.
            set_balance(&env, &addr, 25);
            assert_eq!(
                env.storage().persistent().get_ttl(&key),
                PERSISTENT_BUMP_AMOUNT
            );
            assert_eq!(get_balance(&env, &addr), 25);
        });
    }

    #[test]
    fn set_balance_writes_to_persistent_datakey_balance() {
        let env = Env::default();
        let id = env.register(Harness, ());

        env.as_contract(&id, || {
            let addr = Address::generate(&env);
            set_balance(&env, &addr, 99);

            // Read through the SDK rather than get_balance. A round-trip test
            // cannot catch the two helpers agreeing on the wrong key, so this
            // one names the storage class and key variant explicitly.
            let stored: Option<i128> = env
                .storage()
                .persistent()
                .get(&DataKey::Balance(addr.clone()));
            assert_eq!(stored, Some(99));

            assert!(
                !env.storage().instance().has(&DataKey::Balance(addr)),
                "balances belong in persistent storage, not instance"
            );
        });
    }
}
