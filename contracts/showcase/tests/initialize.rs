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
fn initialize_stores_the_admin_and_emits_the_event() {
    let (env, id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);

    client.initialize(&admin);

    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                id.clone(),
                (Symbol::new(&env, "initialize"),).into_val(&env),
                admin.into_val(&env),
            ),
        ]
    );
}

#[test]
fn initialize_requires_authorization_from_the_admin() {
    let (env, _id, client) = setup();
    let admin = Address::generate(&env);

    // No mock_all_auths, so the require_auth call has nothing to satisfy it.
    let result = client.try_initialize(&admin);

    assert!(
        result.is_err(),
        "initialize must not succeed without the admin's authorization"
    );
}

#[test]
fn initialize_a_second_time_returns_already_initialized() {
    let (env, _id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);

    client.initialize(&admin);
    let second = client.try_initialize(&admin);

    assert_eq!(second, Err(Ok(Error::AlreadyInitialized)));
}
