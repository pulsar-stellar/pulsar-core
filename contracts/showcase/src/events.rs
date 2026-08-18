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
//! Both annotations on each event are deliberate. `topics` names the leading
//! wire Symbol explicitly rather than deriving it from the Rust type name, so
//! renaming a struct cannot silently change the wire contract. `data_format`
//! is explicit because the macro's default is `Map` regardless of field count,
//! which would encode a single payload as a map keyed by field name instead of
//! the bare value the decoder expects.
//!
//! Schema version discrimination deliberately does not appear in topics. It
//! belongs to the decoder, which inspects the contract spec at deserialize time.
//! A version topic alongside that would be a second, independently driftable
//! signal for the same fact.

use soroban_sdk::{contractevent, Address, Bytes, Symbol};

/// Marks successful contract initialization.
///
/// Topic is the single Symbol `initialize`. Data is the admin's address, kept a
/// bare `Address` by `data_format = "single-value"` rather than the default map
/// keyed by field name. The decoder treats this as the canonical initialization
/// marker for the contract.
#[contractevent(topics = ["initialize"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Initialize {
    pub admin: Address,
}

/// Records a deposit crediting an address's balance.
///
/// Topics are the Symbol `deposit` followed by the depositing address, so a
/// consumer can follow one account's deposits without decoding every event on
/// the contract. Data is the deposited amount as a bare `i128`.
#[contractevent(topics = ["deposit"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Deposit {
    #[topic]
    pub from: Address,
    pub amount: i128,
}

/// Records a withdrawal debiting an address's balance.
///
/// Mirrors `Deposit`: topics are the Symbol `withdraw` followed by the
/// withdrawing address, and data is the amount as a bare `i128`. The two share a
/// shape so a consumer can treat balance movement uniformly.
#[contractevent(topics = ["withdraw"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Withdraw {
    #[topic]
    pub to: Address,
    pub amount: i128,
}

/// Records a transfer moving value between two addresses.
///
/// Topics are the Symbol `transfer` followed by the sending and receiving
/// addresses, in that order, and data is the amount as a bare `i128`. Topic
/// order follows field declaration order and is fixed: this is the SEP-41
/// conformance shape, so wallets and indexers already match on it and a
/// reordering here would break them silently.
#[contractevent(topics = ["transfer"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transfer {
    #[topic]
    pub from: Address,
    #[topic]
    pub to: Address,
    pub amount: i128,
}

/// Records rotation of the admin address.
///
/// Topics are the Symbol `admin_change` followed by the incoming admin, and data
/// is the outgoing admin as a bare `Address`. The asymmetry is deliberate: a
/// consumer can filter for "who holds authority now" without decoding payloads,
/// which is the question auditors ask, while reconstructing the full chain of
/// past holders requires reading the data field of each event in sequence.
#[contractevent(topics = ["admin_change"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminChange {
    #[topic]
    pub new_admin: Address,
    pub old_admin: Address,
}

/// Records a caller-supplied event with a runtime-chosen tag.
///
/// Topics are the Symbol `custom` followed by the caller's tag, and data is an
/// opaque `Bytes` payload. Unlike every other event here, the second topic is not
/// fixed at compile time: the tag is a function argument, so the emitted topic
/// varies per call.
///
/// That is the point of the event. A decoder tested only against events whose
/// topics are known statically can pass its whole suite and still fail on the
/// first contract that computes a topic, and an opaque byte payload is the case
/// where the decoder has no type information to lean on. This event exists to
/// give both cases a fixture drawn from a real emission rather than hand-built
/// XDR.
///
/// The wire Symbol is `custom` rather than the type name, pinned by the topics
/// annotation, so the Rust type can be named for local clarity without moving
/// the wire contract.
#[contractevent(topics = ["custom"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmitCustom {
    #[topic]
    pub tag: Symbol,
    pub payload: Bytes,
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

    #[test]
    fn withdraw_publishes_expected_topics_and_data() {
        let env = Env::default();
        let id = env.register(Harness, ());
        let to = Address::generate(&env);

        env.as_contract(&id, || {
            Withdraw {
                to: to.clone(),
                amount: 175,
            }
            .publish(&env);
        });

        assert_eq!(
            env.events().all(),
            vec![
                &env,
                (
                    id.clone(),
                    (Symbol::new(&env, "withdraw"), to.clone()).into_val(&env),
                    175_i128.into_val(&env),
                ),
            ]
        );
    }

    #[test]
    fn transfer_publishes_sep41_topic_order_and_data() {
        let env = Env::default();
        let id = env.register(Harness, ());
        let from = Address::generate(&env);
        let to = Address::generate(&env);

        env.as_contract(&id, || {
            Transfer {
                from: from.clone(),
                to: to.clone(),
                amount: 900,
            }
            .publish(&env);
        });

        // from precedes to. SEP-41 consumers match on that order.
        assert_eq!(
            env.events().all(),
            vec![
                &env,
                (
                    id.clone(),
                    (Symbol::new(&env, "transfer"), from.clone(), to.clone()).into_val(&env),
                    900_i128.into_val(&env),
                ),
            ]
        );
    }

    #[test]
    fn admin_change_publishes_new_admin_as_topic_and_old_as_data() {
        let env = Env::default();
        let id = env.register(Harness, ());
        let old_admin = Address::generate(&env);
        let new_admin = Address::generate(&env);

        env.as_contract(&id, || {
            AdminChange {
                new_admin: new_admin.clone(),
                old_admin: old_admin.clone(),
            }
            .publish(&env);
        });

        // The incoming admin is the queryable topic; the outgoing one is payload.
        assert_eq!(
            env.events().all(),
            vec![
                &env,
                (
                    id.clone(),
                    (Symbol::new(&env, "admin_change"), new_admin.clone()).into_val(&env),
                    old_admin.into_val(&env),
                ),
            ]
        );
    }

    #[test]
    fn emit_custom_publishes_runtime_tag_as_topic_with_opaque_payload() {
        let env = Env::default();
        let id = env.register(Harness, ());
        let payload = Bytes::from_array(&env, &[0xDE, 0xAD, 0xBE, 0xEF]);

        env.as_contract(&id, || {
            EmitCustom {
                tag: Symbol::new(&env, "settled"),
                payload: payload.clone(),
            }
            .publish(&env);
        });

        assert_eq!(
            env.events().all(),
            vec![
                &env,
                (
                    id.clone(),
                    (Symbol::new(&env, "custom"), Symbol::new(&env, "settled")).into_val(&env),
                    payload.clone().into_val(&env),
                ),
            ]
        );
    }

    #[test]
    fn emit_custom_topic_varies_with_the_tag_argument() {
        let env = Env::default();
        let id = env.register(Harness, ());
        let payload = Bytes::from_array(&env, &[0x01]);

        // Two calls differing only in the tag must produce two different topic
        // tuples. A statically fixed topic would make these identical, which is
        // the case the decoder has to handle and cannot be proven by a single
        // emission.
        env.as_contract(&id, || {
            for tag in ["opened", "closed"] {
                EmitCustom {
                    tag: Symbol::new(&env, tag),
                    payload: payload.clone(),
                }
                .publish(&env);
            }
        });

        assert_eq!(
            env.events().all(),
            vec![
                &env,
                (
                    id.clone(),
                    (Symbol::new(&env, "custom"), Symbol::new(&env, "opened")).into_val(&env),
                    payload.clone().into_val(&env),
                ),
                (
                    id.clone(),
                    (Symbol::new(&env, "custom"), Symbol::new(&env, "closed")).into_val(&env),
                    payload.clone().into_val(&env),
                ),
            ]
        );
    }
}
