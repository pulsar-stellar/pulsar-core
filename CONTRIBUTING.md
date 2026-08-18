# Contributing to Pulsar Core

## How this project is built

This project's initial scaffolding and much of its ongoing implementation is written with Claude Code assistance under human review. Every commit is authored, reviewed, and merged by a human maintainer. Design decisions, architecture choices, and merge judgments are human. If you contribute a PR, we don't require you to disclose whether AI tools helped you write it; we do require that your code passes review, tests, and the discipline rules in the requirements document.

The discipline rules referred to above are stated in this file, under Commit rules, Test discipline, and Code rules. Those sections are the standard a PR is measured against, and they are what you need before your first PR. Testing methodology, meaning how to verify a test covers what it claims, lives in `.agent/testing.md`, and the reasoning behind any rule that looks arbitrary is in the decision log at `.agent/decisions.md`.

## Setup

This project uses two Rust toolchains. The **project toolchain** is pinned at 1.92.0 and builds the contract. The **host toolchain** is your system stable channel and exists only to install binary tools. Do not merge them.

| Tool | Version | Install |
|---|---|---|
| Rust, project | 1.92.0 | `rustup` reads `rust-toolchain.toml` and installs it on the first cargo command |
| `wasm32v1-none` target | matches toolchain | installed by the same toolchain file |
| Rust, host | stable, 1.93 or newer | `rustup toolchain install stable` |
| stellar-cli | 27.1.0 or newer | `cd ~ && rustup run stable cargo install --locked --force stellar-cli` |
| Git | 2.40 or newer | platform specific |

Installing stellar-cli from inside the project directory fails. It needs rustc 1.93 or newer to build, and the directory pins cargo to 1.92.0. The `cd ~` escapes the `rust-toolchain.toml` override and `rustup run stable` picks the host channel explicitly. Reasoning is in ADR-007 in `.agent/decisions.md`.

A prebuilt stellar-cli binary from the project's GitHub releases is a supported alternative if you would rather not compile it.

Verify before you start:

```sh
rustc --version      # expect 1.92.0, the project pin
stellar --version    # expect 27.1.0 or newer
rustup target list --installed | grep wasm32v1-none
```

Build and test:

```sh
cargo test                    # test suite
cargo fmt --check             # formatting, must be clean
cargo clippy -- -D warnings   # lints, warnings are errors
stellar contract build        # the contract wasm
```

Build the contract with `stellar contract build`. Never with plain `cargo build`. Plain cargo does not produce a deployable Soroban artifact.

Workspace members land in sequence as the build progresses, so a freshly cloned scaffold may contain fewer crates than the finished layout. The README records which artifacts exist at the current commit.

### Cargo.lock

`Cargo.lock` is committed and is not generated output you may freely overwrite. The contract wasm is uploaded to a ledger, so the build has to be reproducible: a transitive dependency shifting under us changes the artifact without changing a single line of our code. The lock file is what stops that.

It is also load-bearing right now. soroban-env-host declares `ed25519-dalek = ">=2.0.0"` with no upper bound, and 3.0.0 shipped breaking `rand_core` trait changes that its own code cannot compile against. Without the lock, a fresh clone resolves to 3.0.0 and the build fails. The lock holds it at 2.2.0.

Do not commit an incidental lock file update. If your change genuinely requires new or updated dependencies, land the lock change in its own commit whose message says which dependency moved and why, for example `build(deps): bump soroban-sdk to 26.2.0 for event topic fix`. A lock diff that appears alongside unrelated work will be sent back. Run `cargo build --locked` if you want to confirm your branch does not move it.

### Test snapshots

`contracts/showcase/test_snapshots/` is committed. The SDK writes one JSON
snapshot per test, capturing ledger state, the authorizations required, and the
calls made, and the output is deterministic across runs.

Snapshot diffs are reviewed like code. A change to an auth entry, a stored value,
or a call sequence appears there even when every assertion still passes, which is
the class of change that otherwise goes unnoticed. If your PR moves a snapshot,
be ready to explain why.

## Working from issues

Substantive work is tracked with a GitHub issue opened before the work starts. The
issue carries the scope, the acceptance criteria, and references to any ADR or
specification section that governs it. The PR closes it with `Closes #NN` in the
body.

This applies to maintainer work as much as to outside contributions. An issue
written after the fact documents what was done; one written before it is a chance
to disagree about scope while disagreeing is still cheap.

## Commit rules

These are enforced, not stylistic preferences.

**One commit per logical unit.** One function, one type, one test block, one bug fix. A change that must compile together, such as a struct field plus its caller updates, is one commit.

**Never `git add .`** Stage exact paths. This prevents build output, environment files, and key material from entering the history by accident.

**Push after every commit.** Do not batch local commits. If a push fails, stop and resolve it rather than accumulating work locally.

**Never rewrite pushed history.** Fix forward with a follow-up commit.

**Conventional commit format:** `type(scope): description`

- Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `build`, `ci`
- Scopes: `showcase`, `decoder`, `workspace`, `ci`, `docs`, `scripts`, `agent`
- Description: imperative mood, lowercase first letter, no trailing period, under 72 characters

Examples:

```
feat(showcase): add initialize function with admin storage
test(showcase): assert deposit event topics and data
chore(agent): seed decisions.md with soroban-sdk version rationale
```

## Test discipline

Contributors must:

- Include tests for behavior-carrying code, meaning any helper or public function
  that encodes a decision the SDK does not already make
- Land tests and implementation in one commit for behavior-carrying additions.
  A test calling a function that does not exist yet is a compile error in Rust,
  not a failing assertion, so a test-only commit ahead of its implementation
  leaves the branch unbuildable
- Give every public contract function a happy-path test and at least one
  failure-path test, and every error variant a test that triggers it
- Assert events with exact topic and data shapes, never partial matches
- Name tests for the single claim they make. A name that needs "and" usually
  means the test is hiding a coverage gap
- Not submit test theater: tests asserting only "did not throw", tests mocking
  the code under test, and trivially true assertions are rejected in review
- Not disable a clippy warning without ADR-level justification
- Keep line coverage above 85 percent, and run
  `cargo llvm-cov --workspace --locked --summary-only` before submitting

Some code is structural rather than behavior-carrying: types, module scaffolding,
and pass-through wrappers around a single SDK call with no decision in them.
Structural work lands without paired tests and is covered through the public
functions that call it. Do not expect every commit to add tests; whether pairing
is required follows from which kind of work it is.

For the methodology behind these rules, including how to verify a test actually
covers what it claims, see `.agent/testing.md`. The reasoning is in ADR-014.

## Code rules

- No `unwrap`, `expect`, `ok().unwrap()`, or `panic!` in contract code. Convert `Option` to `Result` with `.ok_or(Error::Variant)`. Test files may use `.expect("descriptive message")` with a non-empty message, since the panic text is the diagnostic when a test fails. A naked `unwrap` or an empty-string expect is not permitted anywhere. Production code, meaning everything under `src/` outside a `#[cfg(test)]` block, stays fully bound by the no-panic rule. See ADR-015.
- No floats anywhere. Amounts are `i128`. If proportional math is ever needed, use basis points and integer arithmetic.
- No integer casts that can truncate. Use `TryFrom` with explicit error mapping.
- Every state-changing function calls `require_auth` before any caller-dependent read and before any write.
- Event definitions live in `events.rs` as structs annotated with `#[contractevent]`. Emission happens by constructing the event struct at the call site and calling `.publish(&env)`. Never construct topic tuples or call `env.events().publish` directly; always route through an event struct.
- Every public item carries a doc comment. Every module carries a `//!` header.
- `rustfmt` defaults. `clippy::pedantic` at workspace level, warnings fail CI.
- No stubs, no `TODO` comments, no `unimplemented!()` in shipped commits.

## Writing rules for documentation and commit messages

No em dashes anywhere. Avoid "seamlessly", "robust", "powerful", "leverage", "unlock", "cutting-edge", "revolutionize", "delve into", "elevate", and "empower" used figuratively. Prefer concrete verbs and specific nouns.

Numbers in documentation come from measured data. A number that is a target is labeled as a target.

## Pull requests

Branch from `main`, one logical change per PR where practical, and open the PR with a description of what changed and how you verified it. CI must be green before review. A PR that changes behavior without changing tests will be sent back.

Changes to `crates/pulsar-decoder` carry a higher review bar than changes elsewhere. The decoder is the correctness boundary for every downstream consumer of this toolkit, so expect more review rounds and a request for fixtures derived from real testnet events rather than synthesized input.

## Security

Do not open a public issue for a security problem. Follow `SECURITY.md`.
