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
fn transfer_emits_the_three_topic_event() {
    let (env, id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.initialize(&admin);
    client.deposit(&from, &500);
    client.transfer(&from, &to, &200);

    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                id.clone(),
                (Symbol::new(&env, "transfer"), from.clone(), to.clone()).into_val(&env),
                200_i128.into_val(&env),
            ),
        ]
    );
}

#[test]
fn transfer_credits_the_recipient() {
    let (env, _id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.initialize(&admin);
    client.deposit(&from, &500);
    client.transfer(&from, &to, &200);

    // The recipient can spend exactly what arrived and no more.
    assert_eq!(
        client.try_withdraw(&to, &201),
        Err(Ok(Error::InsufficientBalance))
    );
    client.withdraw(&to, &200);
}

#[test]
fn transfer_debits_the_sender() {
    let (env, _id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.initialize(&admin);
    client.deposit(&from, &500);
    client.transfer(&from, &to, &200);

    // The sender keeps exactly the remainder.
    assert_eq!(
        client.try_withdraw(&from, &301),
        Err(Ok(Error::InsufficientBalance))
    );
    client.withdraw(&from, &300);
}

#[test]
fn transfer_permits_draining_the_entire_source_balance() {
    let (env, _id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.initialize(&admin);
    client.deposit(&from, &500);

    // Exactly the balance must succeed, which separates `>` from an off-by-one.
    client.transfer(&from, &to, &500);
}

#[test]
fn transfer_rejects_an_amount_above_the_source_balance() {
    let (env, _id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.initialize(&admin);
    client.deposit(&from, &500);

    assert_eq!(
        client.try_transfer(&from, &to, &501),
        Err(Ok(Error::InsufficientBalance))
    );
}

#[test]
fn transfer_to_self_leaves_the_balance_unchanged() {
    let (env, _id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let holder = Address::generate(&env);

    client.initialize(&admin);
    client.deposit(&holder, &500);
    client.transfer(&holder, &holder, &200);

    // A self transfer is a no-op on the balance. Reading the recipient's balance
    // before writing the sender's would double count and leave 700 here.
    assert_eq!(
        client.try_withdraw(&holder, &501),
        Err(Ok(Error::InsufficientBalance))
    );
    client.withdraw(&holder, &500);
}

#[test]
fn transfer_requires_authorization_from_the_sender() {
    let (env, _id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.initialize(&admin);
    client.deposit(&from, &500);

    env.set_auths(&[]);
    assert!(
        client.try_transfer(&from, &to, &100).is_err(),
        "transfer must not succeed without the sender's authorization"
    );
}
