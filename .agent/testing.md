# Testing: pulsar-core

How to write and verify tests in this repository. The reasoning behind these
rules lives in the decision log, principally ADR-014, ADR-015, and ADR-019. This
document is the procedure; those entries are the argument. Where the two appear
to disagree, the ADR is authoritative and this file needs an update.

Every rule here came out of a specific commit. Those are cited so a reader can see
the case that produced the rule rather than taking it on trust.

## Where tests live

**Unit tests** go in a `#[cfg(test)] mod tests` block at the bottom of the module
they cover. Crate-private items are reachable there and nowhere else, which is
what storage helpers and event structs need: `DataKey` and the helpers are
`pub(crate)`, so an external test cannot name them.

**Integration tests** go in `contracts/showcase/tests/`, one file per area, and
reach the contract through the generated client:

```rust
use pulsar_showcase::{Error, PulsarShowcase, PulsarShowcaseClient};

let env = Env::default();
let id = env.register(PulsarShowcase, ());
let client = PulsarShowcaseClient::new(&env, &id);
```

This works because the crate declares `crate-type = ["cdylib", "rlib"]`. The
`rlib` half exists solely so test binaries can link the crate; `stellar contract
build` passes `--crate-type=cdylib` explicitly, so the deployed wasm is
unaffected. Verified byte-identical when `rlib` was added in `305dfa4`.

Choosing between them is not a style question. If the test needs a crate-private
item, it goes in the module. If it exercises the public contract interface, it
goes in `tests/`.

## Test-first for behavior-carrying code

A helper is behavior-carrying if it encodes a decision a downstream reader
depends on. Those get tests, and the tests are written first. Pass-through
wrappers around a single SDK call are covered through their public callers. The
distinction and its rationale are ADR-014.

**Tests and implementation land in one commit.** In Rust a test calling a
function that does not exist is a compile error, not a failing assertion, so a
test-only commit ahead of its implementation leaves the branch unbuildable. RED
is still real and still comes first, it just does not get its own commit.

The local cycle:

1. Write the tests. Run them. Confirm the failure is the one you expect, normally
   `E0425: cannot find function`.
2. Write the implementation. Run the tests. Confirm they pass.
3. Run the mutation checks below.
4. Run the full gate.
5. Commit both together, quoting the RED error in the message.

RED is worth observing rather than assuming. In `d9e6c04` it surfaced two missing
`testutils` trait imports alongside the expected missing function; had the
implementation been written first, those would have arrived mixed in with real
failures.

**Test-only commits are valid** when they close a coverage gap on behavior
already shipped. `c556a22` and `ec42489` are both of that kind: the guard existed,
nothing exercised it.

## expect in test code

Test code may use `.expect("message")` with a non-empty descriptive message. The
panic text is the diagnostic, and a test is read most often at the moment it has
failed. A bare `.expect("")` or an `unwrap` standing in for an assertion is not
acceptable.

Production code, meaning everything under `src/` outside a `#[cfg(test)]` block,
remains bound by the no-panic rule. See ADR-015. The mechanical check enforcing
it is scoped the same way: it scans each file under `src/` up to the first
`#[cfg(test)]` marker and does not look at `tests/` at all.

## One behavior per test

A test whose name needs "and" is doing too much, and the cost is not readability.
It is that merged tests hide which behaviors are actually covered: one mutation
failing one merged test is indistinguishable from one mutation failing the
assertion that matters.

This is not hypothetical. In `48f968f` the withdraw suite began as a single test
named `withdraw_debits_balance_and_emits_the_withdraw_event`. Merged, all three
mutations failed it and coverage looked thorough. Split into three, the truth
appeared: removing the balance check entirely failed nothing, because no test yet
withdrew more than the balance. The merged test's incidental rejection assertion
had been standing in for a test that did not exist. That gap was closed
separately in `c556a22`.

Name a test for the single claim it makes.

## Mock construction for security guards

When testing that a guard rejects an unauthorized caller, the mock must authorize
precisely the address a wrong implementation would check.

A test that authorizes an unrelated address and asserts `is_err()` passes in two
situations: the guard correctly rejected the call, or the call failed for an
unrelated reason such as no authorization being present at all. Only the first
proves anything.

For guards that authorize against stored state, such as `set_admin` and
`emit_custom`, construct the mock so the attacker names themselves as the
argument and authorizes themselves:

```rust
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
```

An implementation that authorized the argument instead of the stored admin would
find this mock satisfied and the call would succeed. Only reading the admin from
storage rejects it. That is what makes the test load-bearing against the specific
bug, rather than against auth being absent.

## Mutation checks

For each behavior-carrying test, name the mutation that should fail it, introduce
the mutation, run the tests, and confirm exactly the expected tests fail. Then
restore. One extra run per behavior.

This is a specification, not a formality. It states which behavior each test
guards, and a test that survives the removal of the behavior it claims to cover
is not testing that behavior.

Useful mutations, by what they check:

| Mutation | Verifies |
|---|---|
| Delete a guard clause | the guard's test fails and no other |
| Flip a comparison, `>` to `>=` | a boundary test exists |
| Delete an event publish | the event assertion is real |
| Delete a storage write | the read-back assertion is real |
| Remove `require_auth` | the auth test is not passing for another reason |
| Swap two same-typed fields | field order is pinned |
| Point one helper at another | a side-effect difference is caught |

Two results are worth reading carefully rather than treating as failures of
method. A mutation that fails **no** test means the behavior is uncovered, as in
`48f968f`. A mutation that fails **several** tests can be correct when one line
carries several jobs: removing `let admin = storage::get_admin(&env)?` from
`emit_custom` fails both its uninitialized test and its auth test, because that
line carries the `NotInitialized` path and supplies the address the auth check
reads.

A mutation that will not compile is a stronger result than a failing test. Demoting
a `#[topic]` field to data breaks the build under `data_format = "single-value"`,
so the topic and data split is enforced structurally rather than by assertion. See
ADR-016.

## Value equivalence hides behavior differences

Two implementations returning identical values can differ in side effects, and no
value assertion will tell them apart.

`read_balance` and `get_balance` return the same number for the same input. They
differ only in that `get_balance` extends the entry's TTL when the balance is
positive. Pointing the public view at the wrong one returns correct numbers
forever while quietly extending entry lifetimes on every read, contradicting
ADR-004. Only a TTL assertion catches it, which is why the covering test in
`4fe6f64` asserts `get_ttl` rather than a balance.

Where a helper's contract includes both a return value and a side effect, assert
both. Side effects worth asserting: TTL extension, storage writes, event
emissions, authorization calls.

## Wasm size as a reachability signal

Dead-code elimination strips anything nothing calls, so the wasm artifact only
contains reachable code. A commit that adds a caller for previously unreferenced
helpers should grow it.

The history, which is why the signal is trustworthy here:

| Commit | Size | What changed |
|---|---|---|
| `2c77db3` | 313 B | stub `lib.rs`, no modules |
| step 14 | 975 B | `mod error;` declared, error type reachable |
| `d9e6c04` | 2,003 B | storage helpers present but uncalled |
| step 26 | 5,067 B | all six event structs present, none emitted |
| `305dfa4` | 6,260 B | `initialize` gives helpers their first caller |
| `eea773f` | 7,456 B | `deposit` reaches the balance helpers |
| `48f968f` | 8,434 B | `withdraw` |
| final | 11,834 B | all eight public functions |

Between step 14 and `305dfa4` the size barely moved despite eleven commits of
storage and event code, because none of it was reachable. That is expected, and
it is also why a flat size after adding a caller is a signal to investigate: it
means the wiring did not land.

Check the size on any commit that should activate previously unreferenced code.

## env.events().all() is scoped to one invocation

`env.events().all()` returns the events of the most recent contract invocation,
not the cumulative history of the test.

A test that makes several client calls and asserts against `all()` at the end sees
only the last call's events. The first draft of the deposit test in `eea773f`
asserted both the initialize and deposit events and failed with only the deposit
event present.

Assert events immediately after the call that emits them. Unit tests inside a
single `env.as_contract(...)` block are one invocation, so multiple events do
appear there, which is why the event unit tests in `events.rs` can assert two at
once.

Assert the whole collection rather than indexing into it:

```rust
assert_eq!(
    env.events().all(),
    vec![&env, (id.clone(), (Symbol::new(&env, "deposit"), from).into_val(&env), 500_i128.into_val(&env))]
);
```

`ContractEvents` has no `len()` or `first()`, and comparing the whole collection
pins the event count alongside the shape, so a stray second emission fails.

## Test snapshots

The SDK writes a JSON snapshot per test under `contracts/showcase/test_snapshots/`,
capturing ledger state, authorizations required, and calls made. They are
deterministic: running the suite twice produces byte-identical files.

They are committed. A diff that changes an auth entry, a stored value, or a call
sequence shows up in review even when assertions still pass, which is the class
of change that otherwise goes unnoticed. Regeneration is then deliberate and
visible.

They are review artifacts, not assertions. Tests assert through explicit
expectations; snapshots record what the run did. A snapshot diff is a prompt to
look, not a failure.

## Lints apply to tests

Clippy runs across all targets:

```sh
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Unused imports, unused bindings, and every other lint fail the gate in test code
exactly as in contract code. This is deliberate. Test code that accumulates lint
debt becomes code nobody wants to read, and it is read at the worst possible
moment. Two failures during Phase C were in test files rather than the contract.

## Coverage

```sh
cargo llvm-cov --workspace --locked --summary-only
cargo llvm-cov --workspace --locked --html --output-dir target/coverage-report
```

The floor is 85 percent line coverage.

Two prerequisites, both listed in `docs/requirements.md` section 1.8. The
`llvm-tools` component is not installed by default because `rust-toolchain.toml`
sets `profile = "minimal"`, and `cargo-llvm-cov` is a host tool installed on the
stable channel per ADR-007:

```sh
rustup component add llvm-tools --toolchain 1.92.0
cd ~ && rustup run stable cargo install --locked cargo-llvm-cov
```

Read the uncovered lines rather than only the percentage. Some are artifacts:
`#[contracttype]` attribute lines attract macro-expanded code, and `assert!`
message arguments are evaluated only when an assertion fails, so they stay
uncovered on a green run. A genuinely uncovered line is one with real logic and
an execution count of zero, which is how the missing `transfer` amount-guard test
was found and closed in `ec42489`.
