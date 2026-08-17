#![cfg(test)]

use pulsar_showcase::{Error, PulsarShowcase, PulsarShowcaseClient};
use soroban_sdk::testutils::{Address as _, Events as _, MockAuth, MockAuthInvoke};
use soroban_sdk::{vec, Address, Bytes, Env, IntoVal, Symbol};

/// Registers the contract and returns the env, its id, and a client.
fn setup() -> (Env, Address, PulsarShowcaseClient<'static>) {
    let env = Env::default();
    let id = env.register(PulsarShowcase, ());
    let client = PulsarShowcaseClient::new(&env, &id);
    (env, id, client)
}

#[test]
fn emit_custom_publishes_the_tag_and_payload() {
    let (env, id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let tag = Symbol::new(&env, "settled");
    let payload = Bytes::from_array(&env, &[0xDE, 0xAD, 0xBE, 0xEF]);

    client.initialize(&admin);
    client.emit_custom(&tag, &payload);

    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                id.clone(),
                (Symbol::new(&env, "custom"), tag.clone()).into_val(&env),
                payload.clone().into_val(&env),
            ),
        ]
    );
}

#[test]
fn emit_custom_rejects_a_caller_who_is_not_the_admin() {
    let (env, id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);
    let tag = Symbol::new(&env, "spoofed");
    let payload = Bytes::from_array(&env, &[0x01]);

    client.initialize(&admin);

    // The mock authorizes the attacker, which is the address an implementation
    // that trusted the caller would check. Only reading the admin from storage
    // rejects this call.
    let result = client
        .mock_auths(&[MockAuth {
            address: &attacker,
            invoke: &MockAuthInvoke {
                contract: &id,
                fn_name: "emit_custom",
                args: (tag.clone(), payload.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_emit_custom(&tag, &payload);

    // Host-raised auth failure, so is_err() rather than a variant match. See
    // ADR-018.
    assert!(
        result.is_err(),
        "only the admin may write into the contract's event history"
    );
}

#[test]
fn emit_custom_fails_on_an_uninitialized_contract() {
    let (env, _id, client) = setup();
    env.mock_all_auths();
    let tag = Symbol::new(&env, "early");
    let payload = Bytes::from_array(&env, &[0x02]);

    assert_eq!(
        client.try_emit_custom(&tag, &payload),
        Err(Ok(Error::NotInitialized))
    );
}
