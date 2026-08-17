#![cfg(test)]

use pulsar_showcase::{Error, PulsarShowcase, PulsarShowcaseClient};
use soroban_sdk::testutils::{Address as _, Events as _, MockAuth, MockAuthInvoke};
use soroban_sdk::{vec, Address, Env, IntoVal, Symbol};

/// Registers the contract and returns the env, its id, and a client.
fn setup() -> (Env, Address, PulsarShowcaseClient<'static>) {
    let env = Env::default();
    let id = env.register(PulsarShowcase, ());
    let client = PulsarShowcaseClient::new(&env, &id);
    (env, id, client)
}

#[test]
fn set_admin_emits_admin_change_with_old_and_new() {
    let (env, id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let next = Address::generate(&env);

    client.initialize(&admin);
    client.set_admin(&next);

    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                id.clone(),
                (Symbol::new(&env, "admin_change"), next.clone()).into_val(&env),
                admin.into_val(&env),
            ),
        ]
    );
}

#[test]
fn set_admin_rejects_a_caller_who_is_not_the_admin() {
    let (env, id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);
    client.initialize(&admin);

    // The attack this guards against: an attacker names themselves as the new
    // admin and authorizes their own call. Authorizing the argument rather than
    // the stored admin would hand them the contract, so the mock deliberately
    // satisfies exactly the address an incorrect implementation would check.
    let result = client
        .mock_auths(&[MockAuth {
            address: &attacker,
            invoke: &MockAuthInvoke {
                contract: &id,
                fn_name: "set_admin",
                args: (attacker.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_set_admin(&attacker);

    // A require_auth rejection is raised by the host, so it arrives as an
    // invocation error rather than as a typed Error variant. There is no
    // Unauthorized to match against, by design. See ADR-018.
    assert!(
        result.is_err(),
        "rotation must not succeed on a non-admin's authorization"
    );
}

#[test]
fn set_admin_fails_on_an_uninitialized_contract() {
    let (env, _id, client) = setup();
    env.mock_all_auths();
    let next = Address::generate(&env);

    assert_eq!(client.try_set_admin(&next), Err(Ok(Error::NotInitialized)));
}

#[test]
fn set_admin_transfers_authority_to_the_new_admin() {
    let (env, id, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let next = Address::generate(&env);
    let third = Address::generate(&env);

    client.initialize(&admin);
    client.set_admin(&next);

    // The new admin can rotate again.
    client
        .mock_auths(&[MockAuth {
            address: &next,
            invoke: &MockAuthInvoke {
                contract: &id,
                fn_name: "set_admin",
                args: (third.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .set_admin(&third);

    // The original admin no longer can.
    let result = client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &id,
                fn_name: "set_admin",
                args: (admin.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_set_admin(&admin);

    // Host-raised auth failure again, hence is_err() rather than a variant match.
    assert!(
        result.is_err(),
        "a replaced admin must not retain authority"
    );
}
