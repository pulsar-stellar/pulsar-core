//! Event definitions.
//!
//! Events are declared here as structs annotated with `#[contractevent]`, not as
//! emission helper functions. The macro derives the leading topic Symbol from the
//! type name, adds a topic for each `#[topic]` field, and encodes the remaining
//! fields as data, so the wire shape follows from the declaration rather than
//! from a hand-built topic tuple. Contract code emits an event by constructing
//! the struct and calling `.publish(&env)`.
//!
//! These shapes are the contract's public interface. The pulsar-decoder crate
//! and every downstream consumer decode against them, so a change to a field, its
//! topic marking, or the data format is a breaking change even when the
//! contract's own behavior is unaffected.
//!
//! Schema version discrimination deliberately does not appear in topics. It
//! belongs to the decoder, which inspects the contract spec at deserialize time.
//! A version topic alongside that would be a second, independently driftable
//! signal for the same fact.

// #![allow(dead_code)] necessary while the event definitions land ahead of their
// callers in contract.rs. Remove this line as part of the step 27 commit when
// contract.rs adds the calls.
#![allow(dead_code)]

use soroban_sdk::{contractevent, Address};

/// Marks successful contract initialization.
///
/// Topic is the single Symbol `initialize`. Data is the admin's address, kept a
/// bare `Address` by `data_format = "single-value"` rather than the default map
/// keyed by field name. The decoder treats this as the canonical initialization
/// marker for the contract.
#[contractevent(data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Initialize {
    pub admin: Address,
}

/// Records a deposit crediting an address's balance.
///
/// Topics are the Symbol `deposit` followed by the depositing address, so a
/// consumer can follow one account's deposits without decoding every event on
/// the contract. Data is the deposited amount as a bare `i128`.
#[contractevent(data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Deposit {
    #[topic]
    pub from: Address,
    pub amount: i128,
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Events as _};
    use soroban_sdk::{contract, contractimpl, vec, Env, IntoVal, Symbol};

    #[contract]
    struct Harness;

    #[contractimpl]
    impl Harness {}

    #[test]
    fn initialize_publishes_expected_topic_and_data() {
        let env = Env::default();
        let id = env.register(Harness, ());
        let admin = Address::generate(&env);

        env.as_contract(&id, || {
            Initialize {
                admin: admin.clone(),
            }
            .publish(&env);
        });

        // Whole-collection equality pins the event count, the emitting contract,
        // the exact topic tuple, and the exact data payload in one assertion.
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
    fn deposit_publishes_expected_topics_and_data() {
        let env = Env::default();
        let id = env.register(Harness, ());
        let from = Address::generate(&env);

        env.as_contract(&id, || {
            Deposit {
                from: from.clone(),
                amount: 250,
            }
            .publish(&env);
        });

        assert_eq!(
            env.events().all(),
            vec![
                &env,
                (
                    id.clone(),
                    (Symbol::new(&env, "deposit"), from.clone()).into_val(&env),
                    250_i128.into_val(&env),
                ),
            ]
        );
    }
}
