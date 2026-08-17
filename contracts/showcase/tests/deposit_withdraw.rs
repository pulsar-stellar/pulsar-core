#![cfg(test)]

use pulsar_showcase::{Error, PulsarShowcase, PulsarShowcaseClient};
use soroban_sdk::testutils::{Address as _, Events as _};
use soroban_sdk::{vec, Address, Env, IntoVal, Symbol};

/// Registers the contract and returns the env, its id, and a client.
fn setup() -> (Env, Address, PulsarShowcaseClient<'static>) {
    let env = Env::default();
    let id = env.register(PulsarShowcase, ());
    let client = PulsarShowcaseClient::new(&env, &id);
    (env, id, client)
}

#[test]
fn deposit_emits_the_deposit_event_with_depositor_and_amount() {
    let (env, id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let from = Address::generate(&env);

    client.initialize(&admin);
    client.deposit(&from, &500);

    // all() reports the events of the most recent invocation, not the whole
    // history, so the initialize event from the previous call is not present.
    // Asserting the whole collection still pins the count: a stray second
    // emission from deposit would fail here.
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                id.clone(),
                (Symbol::new(&env, "deposit"), from.clone()).into_val(&env),
                500_i128.into_val(&env),
            ),
        ]
    );
}

#[test]
fn deposit_requires_authorization_from_the_depositor() {
    let (env, _id, client) = setup();
    let admin = Address::generate(&env);
    let from = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin);

    // Clear the blanket auth so the depositor's require_auth has nothing to
    // satisfy it.
    env.set_auths(&[]);
    let result = client.try_deposit(&from, &500);

    assert!(
        result.is_err(),
        "deposit must not succeed without the depositor's authorization"
    );
}

#[test]
fn deposit_rejects_non_positive_amounts() {
    let (env, _id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let from = Address::generate(&env);
    client.initialize(&admin);

    // Zero and negative are the same failure: neither can be applied.
    assert_eq!(
        client.try_deposit(&from, &0),
        Err(Ok(Error::AmountOutOfRange))
    );
    assert_eq!(
        client.try_deposit(&from, &-1),
        Err(Ok(Error::AmountOutOfRange))
    );
}

#[test]
fn deposit_fails_on_an_uninitialized_contract() {
    let (env, _id, client) = setup();
    env.mock_all_auths();
    let from = Address::generate(&env);

    // No initialize call, so the contract has no admin and no Initialized flag.
    assert_eq!(
        client.try_deposit(&from, &100),
        Err(Ok(Error::NotInitialized))
    );
}

#[test]
fn withdraw_emits_the_withdraw_event_with_holder_and_amount() {
    let (env, id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let holder = Address::generate(&env);

    client.initialize(&admin);
    client.deposit(&holder, &500);
    client.withdraw(&holder, &200);

    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                id.clone(),
                (Symbol::new(&env, "withdraw"), holder.clone()).into_val(&env),
                200_i128.into_val(&env),
            ),
        ]
    );
}

#[test]
fn withdraw_permits_spending_the_entire_balance() {
    let (env, _id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let holder = Address::generate(&env);

    client.initialize(&admin);
    client.deposit(&holder, &500);
    client.withdraw(&holder, &200);

    // Exactly the remainder must succeed. This is the boundary that separates
    // a correct `amount > balance` check from an off-by-one `>=`, and it is also
    // how the debit arithmetic is observable before a balance view exists.
    client.withdraw(&holder, &300);
}

#[test]
fn withdraw_requires_authorization_from_the_holder() {
    let (env, _id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let holder = Address::generate(&env);
    client.initialize(&admin);
    client.deposit(&holder, &500);

    env.set_auths(&[]);
    assert!(
        client.try_withdraw(&holder, &100).is_err(),
        "withdraw must not succeed without the holder's authorization"
    );
}

#[test]
fn withdraw_rejects_an_amount_above_the_balance() {
    let (env, _id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let holder = Address::generate(&env);

    client.initialize(&admin);
    client.deposit(&holder, &500);

    // One over the balance is the tightest case that must still be refused.
    assert_eq!(
        client.try_withdraw(&holder, &501),
        Err(Ok(Error::InsufficientBalance))
    );

    // An address that never deposited has no entry at all, which reads as zero
    // rather than as a missing-key failure.
    let stranger = Address::generate(&env);
    assert_eq!(
        client.try_withdraw(&stranger, &1),
        Err(Ok(Error::InsufficientBalance))
    );
}
