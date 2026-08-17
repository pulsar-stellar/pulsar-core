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

use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, Symbol};

use crate::error::Error;
use crate::events::{AdminChange, Deposit, EmitCustom, Initialize, Transfer, Withdraw};
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

    /// Debits an address's balance by `amount`.
    ///
    /// Requires authorization from the address being debited, so no one can move
    /// value out of an account they do not control.
    ///
    /// # Errors
    ///
    /// Returns `Error::NotInitialized` if the contract has no admin yet,
    /// `Error::AmountOutOfRange` if `amount` is not positive, and
    /// `Error::InsufficientBalance` if the stored balance is below `amount`.
    pub fn withdraw(env: Env, to: Address, amount: i128) -> Result<(), Error> {
        to.require_auth();

        if !storage::is_initialized(&env) {
            return Err(Error::NotInitialized);
        }
        if amount <= 0 {
            return Err(Error::AmountOutOfRange);
        }

        storage::extend_instance_ttl(&env);

        let balance = storage::get_balance(&env, &to);
        if amount > balance {
            return Err(Error::InsufficientBalance);
        }
        storage::set_balance(&env, &to, balance - amount);

        Withdraw { to, amount }.publish(&env);

        Ok(())
    }

    /// Moves `amount` from one address's balance to another's.
    ///
    /// Requires authorization from the sending address. The recipient does not
    /// authorize: receiving value is not a burden they need to consent to, which
    /// is also what SEP-41 specifies.
    ///
    /// A transfer to oneself is permitted and leaves the balance unchanged.
    /// Rejecting it would add an error path for a case with no economic effect
    /// and no fraud potential, and consumers that care can compare the topics.
    ///
    /// # Errors
    ///
    /// Returns `Error::NotInitialized` if the contract has no admin yet,
    /// `Error::AmountOutOfRange` if `amount` is not positive, and
    /// `Error::InsufficientBalance` if the sender's balance is below `amount`.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), Error> {
        from.require_auth();

        if !storage::is_initialized(&env) {
            return Err(Error::NotInitialized);
        }
        if amount <= 0 {
            return Err(Error::AmountOutOfRange);
        }

        storage::extend_instance_ttl(&env);

        let from_balance = storage::get_balance(&env, &from);
        if amount > from_balance {
            return Err(Error::InsufficientBalance);
        }
        storage::set_balance(&env, &from, from_balance - amount);

        // Read the recipient's balance after writing the sender's. When from and
        // to are the same address, reading both up front would credit a stale
        // figure and inflate the balance.
        let to_balance = storage::get_balance(&env, &to);
        let credited = to_balance
            .checked_add(amount)
            .ok_or(Error::AmountOutOfRange)?;
        storage::set_balance(&env, &to, credited);

        Transfer { from, to, amount }.publish(&env);

        Ok(())
    }

    /// Replaces the admin address.
    ///
    /// Authorization comes from the admin currently stored, not from any address
    /// the caller supplies. Trusting an argument here would let anyone name
    /// themselves and seize the contract.
    ///
    /// # Errors
    ///
    /// Returns `Error::NotInitialized` if no admin is stored yet.
    pub fn set_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        let old_admin = storage::get_admin(&env)?;
        old_admin.require_auth();

        storage::extend_instance_ttl(&env);
        storage::set_admin(&env, &new_admin);

        AdminChange {
            new_admin,
            old_admin,
        }
        .publish(&env);

        Ok(())
    }

    /// Emits an event carrying a caller-chosen tag and an opaque payload.
    ///
    /// Changes no state. It exists so the decoder has a fixture whose topic is
    /// computed at run time rather than fixed at compile time, and whose payload
    /// carries no type information.
    ///
    /// Authorization comes from the stored admin. An unauthenticated event-only
    /// endpoint would let anyone write arbitrary entries into the event history
    /// of the contract the whole toolkit tests against.
    ///
    /// # Errors
    ///
    /// Returns `Error::NotInitialized` if no admin is stored yet.
    pub fn emit_custom(env: Env, tag: Symbol, payload: Bytes) -> Result<(), Error> {
        let admin = storage::get_admin(&env)?;
        admin.require_auth();

        storage::extend_instance_ttl(&env);

        EmitCustom { tag, payload }.publish(&env);

        Ok(())
    }

    /// Reports an address's balance.
    ///
    /// A read view. It requires no authorization, since balances are public
    /// ledger state, and it does not extend the entry's TTL: observing an entry
    /// is not a reason to keep it alive.
    #[must_use]
    pub fn balance(env: Env, of: Address) -> i128 {
        storage::read_balance(&env, &of)
    }

    /// Reports the current admin address.
    ///
    /// A read view. No authorization is required, since who holds authority is
    /// public ledger state, and the instance TTL is not extended: observing the
    /// contract is not a reason to keep it alive.
    ///
    /// # Errors
    ///
    /// Returns `Error::NotInitialized` if no admin is stored yet.
    pub fn admin(env: Env) -> Result<Address, Error> {
        storage::get_admin(&env)
    }
}
