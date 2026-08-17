//! The showcase contract's public interface.
//!
//! Every function on this type exists to exercise a specific decoder capability
//! rather than to model a product. A function that does not produce an event
//! shape worth decoding does not belong here, which is what keeps the surface at
//! six state-changing functions and two read views.
//!
//! Public functions are added one at a time, each with its tests, so the
//! interface grows in reviewable units rather than arriving whole.

// Soroban's ABI requires public contract functions to take Env by value, even
// though the body only ever borrows it. The lint is correct in general and wrong
// here, so it is disabled at module scope rather than per function.
#![allow(clippy::needless_pass_by_value)]

use soroban_sdk::{contract, contractimpl, Address, Env};

use crate::error::Error;
use crate::events::{Deposit, Initialize};
use crate::storage;

/// The showcase contract.
///
/// Deployed to testnet as the toolkit's reference contract: the events it emits
/// are the fixtures the decoder, indexer, and explorer are tested against, and
/// its contract ID is what a developer pastes into the explorer to see decoded
/// history end to end.
#[contract]
pub struct PulsarShowcase;

#[contractimpl]
impl PulsarShowcase {
    /// Sets the contract's admin, once.
    ///
    /// Returns `AlreadyInitialized` if an admin is already stored, so a second
    /// call cannot quietly hand authority to a different address.
    ///
    /// Requires authorization from the address being installed as admin, which
    /// prevents naming an address that has not consented to the role.
    ///
    /// # Errors
    ///
    /// Returns `Error::AlreadyInitialized` if an admin is already stored.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();

        if storage::is_initialized(&env) {
            return Err(Error::AlreadyInitialized);
        }

        storage::extend_instance_ttl(&env);
        storage::set_initialized(&env);
        storage::set_admin(&env, &admin);

        Initialize { admin }.publish(&env);

        Ok(())
    }

    /// Credits an address's balance by `amount`.
    ///
    /// Requires authorization from the depositing address, so a caller cannot
    /// credit an account that has not consented to the deposit.
    ///
    /// # Errors
    ///
    /// Returns `Error::NotInitialized` if the contract has no admin yet, and
    /// `Error::AmountOutOfRange` if `amount` is not positive, or if crediting it
    /// would overflow the stored balance.
    pub fn deposit(env: Env, from: Address, amount: i128) -> Result<(), Error> {
        from.require_auth();

        if !storage::is_initialized(&env) {
            return Err(Error::NotInitialized);
        }
        if amount <= 0 {
            return Err(Error::AmountOutOfRange);
        }

        storage::extend_instance_ttl(&env);

        let updated = storage::get_balance(&env, &from)
            .checked_add(amount)
            .ok_or(Error::AmountOutOfRange)?;
        storage::set_balance(&env, &from, updated);

        Deposit { from, amount }.publish(&env);

        Ok(())
    }
}
