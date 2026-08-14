# Contributing to Pulsar Core

## How this project is built

This project's initial scaffolding and much of its ongoing implementation is written with Claude Code assistance under human review. Every commit is authored, reviewed, and merged by a human maintainer. Design decisions, architecture choices, and merge judgments are human. If you contribute a PR, we don't require you to disclose whether AI tools helped you write it; we do require that your code passes review, tests, and the discipline rules in the requirements document.

The discipline rules referred to above are restated in full in this file, under Commit rules, Test discipline, and Code rules. Those sections are the standard a PR is measured against. Read them before your first PR.

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

Every code commit satisfies one of three conditions: it is a test commit, it is implementation preceded by a commit containing the failing test it makes pass, or it is scaffolding with no logic.

No PR merges with code changes and no corresponding test changes. Coverage floor for this repository is 85% line coverage, measured with `cargo-llvm-cov` and enforced in CI.

Every public contract function needs a happy-path test and at least one failure-path test. Every variant of the error enum needs at least one test that triggers it. Every event emission is asserted with exact topic and data shapes through `env.events().all()`, not partial matches.

Tests that only assert "did not throw", tests that mock the code under test, and assertions that are trivially true do not count as tests and will be rejected in review.

## Code rules

- No `unwrap`, `expect`, `ok().unwrap()`, or `panic!` in contract code. Convert `Option` to `Result` with `.ok_or(Error::Variant)`. Tests may use `unwrap` and `expect`.
- No floats anywhere. Amounts are `i128`. If proportional math is ever needed, use basis points and integer arithmetic.
- No integer casts that can truncate. Use `TryFrom` with explicit error mapping.
- Every state-changing function calls `require_auth` before any caller-dependent read and before any write.
- Every event emission goes through a helper in `events.rs`. No inline `env.events().publish` in `contract.rs`.
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
