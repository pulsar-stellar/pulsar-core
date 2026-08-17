#![cfg(test)]

use pulsar_showcase::{PulsarShowcase, PulsarShowcaseClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};

/// Registers the contract and returns the env, its id, and a client.
fn setup() -> (Env, Address, PulsarShowcaseClient<'static>) {
    let env = Env::default();
    let id = env.register(PulsarShowcase, ());
    let client = PulsarShowcaseClient::new(&env, &id);
    (env, id, client)
}

#[test]
fn balance_reports_a_deposited_amount() {
    let (env, _id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let holder = Address::generate(&env);

    client.initialize(&admin);
    client.deposit(&holder, &500);

    assert_eq!(client.balance(&holder), 500);
}

#[test]
fn balance_reports_zero_for_an_account_that_never_deposited() {
    let (env, _id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let stranger = Address::generate(&env);

    client.initialize(&admin);

    assert_eq!(client.balance(&stranger), 0);
}

#[test]
fn balance_reflects_both_sides_of_a_transfer() {
    let (env, _id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.initialize(&admin);
    client.deposit(&from, &500);
    client.transfer(&from, &to, &200);

    assert_eq!(client.balance(&from), 300);
    assert_eq!(client.balance(&to), 200);
}

#[test]
fn balance_reflects_a_withdrawal() {
    let (env, _id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let holder = Address::generate(&env);

    client.initialize(&admin);
    client.deposit(&holder, &500);
    client.withdraw(&holder, &175);

    assert_eq!(client.balance(&holder), 325);
}
