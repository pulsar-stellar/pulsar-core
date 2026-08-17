//! Event emission helpers.
//!
//! Every event this contract emits is published through a helper here. Nothing
//! calls `env.events().publish` inline, so the wire shape of each event has one
//! definition and one place to change it. Those shapes are the contract's public
//! interface: the pulsar-decoder crate and every downstream consumer decode
//! against them, so a change to a topic tuple or data payload is a breaking
//! change even when the contract's own behavior is unaffected.

// #![allow(dead_code)] necessary while the emission helpers land ahead of their
// callers in contract.rs. Remove this line as part of the step 27 commit when
// contract.rs adds the calls.
#![allow(dead_code)]

use soroban_sdk::{contractevent, Address, Env};

/// The initialize event, marking successful contract initialization.
///
/// Emitted with a single Symbol topic, `initialize`, derived from the type name,
/// and the admin's address as the data payload. `data_format = "single-value"`
/// keeps the payload a bare Address rather than a map keyed by field name, which
/// is the shape the pulsar-decoder crate decodes as the canonical initialization
/// marker for this contract.
#[contractevent(data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Initialize {
    pub admin: Address,
}

/// Emits the initialize event marking successful contract initialization.
pub(crate) fn emit_initialize(env: &Env, admin: &Address) {
    Initialize {
        admin: admin.clone(),
    }
    .publish(env);
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Events as _};
    use soroban_sdk::{contract, contractimpl, vec, IntoVal, Symbol};

    #[contract]
    struct Harness;

    #[contractimpl]
    impl Harness {}

    #[test]
    fn emit_initialize_publishes_expected_topic_and_data() {
        let env = Env::default();
        let id = env.register(Harness, ());
        let admin = Address::generate(&env);

        env.as_contract(&id, || {
            emit_initialize(&env, &admin);
        });

        // Whole-collection equality: pins the event count, the emitting
        // contract, the exact topic tuple, and the exact data payload in one
        // assertion. ContractEvents compares against Vec<(Address, Vec<Val>, Val)>.
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
}
