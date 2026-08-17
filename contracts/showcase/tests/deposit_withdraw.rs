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
