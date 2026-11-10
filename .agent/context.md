# Context: pulsar-core

Onboarding for a fresh session, human or agent. Written at scaffold, updated at phase transitions only. If this file disagrees with the repository, the repository is right and this file needs an update.

## 1. What Pulsar Stellar is

Pulsar Stellar is a developer toolkit for Soroban contract events. Every Stellar project that needs to consume its contract's events today writes the same plumbing from scratch: XDR decoders, indexer glue, custom APIs. Pulsar Stellar provides three shared building blocks so they don't have to. A Rust library that turns raw contract events into typed data, a Go daemon that stores historical events past the seven-day RPC retention window, and a web explorer where anyone can paste a contract ID and browse every event that contract has ever emitted, decoded and searchable. It serves Soroban dapp builders, backend engineers integrating with existing protocols, and auditors reviewing contract behavior post-deployment.

That paragraph is locked wording. Reuse it verbatim wherever the project is described. Do not paraphrase it.

## 2. Why this repo exists

`pulsar-core` is the Rust contract layer. It carries two artifacts:

**`pulsar-showcase`** is a reference contract, not a product. Every function on it exists to exercise a specific decoder capability. `transfer` produces a three-topic event that doubles as the SEP-41 conformance case. `emit_custom` produces a dynamic Symbol topic with a Bytes payload, which is the case that breaks naive decoders. Its events are the fixtures the rest of the toolkit tests against, which is why the contract stays small and why its scope is closed at six state-changing functions and two read views.

**`pulsar-decoder`** is the typed event decoder, the correctness boundary for every downstream consumer. It shipped as a placeholder in v0.1.0-contracts and got its real content once the app repo had proved what shape consumers need: it decodes `ScVal` values and topic lists into the `DecodedValue` taxonomy fixed by pulsar-app ADR-023. It is not published yet; that is the v0.2.0-contracts release.

The two share a repo because they co-evolve. A new decoder capability needs a fixture to prove it, and a new showcase function exists to produce one.

## 3. Relationship to pulsar-app

`pulsar-stellar/pulsar-app` holds the TypeScript SDK, the Go indexer, the Next.js explorer, and the documentation site. It consumes what this repo produces.

The sync point is the deployed testnet contract ID. When v0.1.0-contracts is tagged, that ID goes into `pulsar-app` as `NEXT_PUBLIC_SHOWCASE_CONTRACT_ID` and `PULSAR_INDEXER_BOOTSTRAP_CONTRACTS`, which unblocks its Sprint 4.

Nothing in this repo may create `.ts`, `.tsx`, `.go`, or `next.config` files. Reaching for one means the work belongs in the sibling repo.

Note that the Go indexer reimplements decoding rather than calling into the Rust decoder. The two are kept honest against shared fixtures. Consolidating them through FFI is on the roadmap as a future direction, triggered only if the two decoders are observed to drift on a real contract.

## 4. Current phase and definition of done

**`v0.1.0-contracts` is released.** Current release target: `v0.2.0-contracts`, the decoder crate's first release. Its scope and exit gates are in `docs/roadmap-core.md`.

`v0.1.0-contracts` was done when all of the following held, and they all do:

- Every public function has a happy-path test and at least one failure-path test
- Every variant of the error enum is triggered by at least one test
- Every event emission is asserted with exact topic and data shapes
- The contract is deployed to testnet and invocable, with `stellar events` returning decoded samples for every event type
- CI has been green for at least three consecutive commits
- Rust line coverage is at or above 85 percent
- The tag body records the deployed contract ID

## 5. Drips Wave context

This project is being built toward a Drips Wave submission. Two consequences shape day-to-day work.

The repository has to look like something another person could contribute to today. That means real CI, a real license, a security policy, contribution rules, and issues scoped tightly enough that a stranger can pick one up without a conversation. Sprint 9 creates between 40 and 65 labeled issues across the two repos, with complexity labels of 100, 150, or 200 points.

Contributor issues concentrate in `pulsar-app`, which has the larger surface. This repo stays reviewer-only until v1.0, and decoder changes carry a higher review bar than anything else in either repo.

Every claim in submission materials points at something real: a live URL, a deployed contract, a published package. No mockups, no placeholder contract IDs.

## 6. Where the authoritative spec lives

- **Contract behavior**: `contracts/showcase/src/contract.rs` and its tests. The contract source is the specification.
- **Project standards**: `docs/requirements.md`, the constitution. Where another document is more lenient, this one wins. Where another is stricter, the stricter rule wins.
- **Release tracks**: `docs/roadmap-core.md` for this repo, `docs/roadmap-product.md` for cross-repo context.
- **Decisions**: `.agent/decisions.md`, append-only.
- **Terminology**: `.agent/glossary.md`.
- **Build sequence**: `docs/planning/system-prompt.md`, maintainer-local and excluded from version control, so absent from a fresh clone.

## 7. How to run things locally

Toolchain. `rustup` reads `rust-toolchain.toml` and installs Rust 1.92.0 with the `wasm32v1-none` target on the first cargo command. That pin is the lowest version soroban-sdk 26.1.0 and stellar-cli 27.1.0 both accept, per ADR-009. stellar-cli is a host binary tool, installed with the host stable channel rather than the project pin, per ADR-007. Running the install from inside this directory fails because stellar-cli needs rustc 1.93 or newer and the directory pins cargo to 1.92.0.

```sh
cd ~ && rustup run stable cargo install --locked --force stellar-cli

rustc --version      # expect 1.92.0, the project pin
stellar --version    # expect 27.1.0 or newer
rustup target list --installed | grep wasm32v1-none
```

Test and lint.

```sh
cargo test
cargo fmt --check
cargo clippy -- -D warnings
```

Build the contract. Never plain `cargo build`, which does not produce a deployable Soroban artifact.

```sh
stellar contract build
```

Deploy to testnet. Requires a funded identity.

```sh
stellar keys generate deployer --network testnet
stellar keys fund deployer --network testnet
stellar contract deploy \
  --wasm target/wasm32v1-none/release/pulsar_showcase.wasm \
  --source deployer \
  --network testnet
```

`scripts/build.sh` and `scripts/deploy-testnet.sh` wrap the last two blocks, with preflight checks on the artifact and the identity.

Coverage.

```sh
cargo llvm-cov --workspace   # floor is 85 percent line coverage
```

## 8. State at the time of writing

The Phase 6 build sequence is complete. All 57 steps have landed and
`v0.1.0-contracts` is tagged and released.

`contracts/showcase` is complete: 8 public functions (six state-changing, two
read views) and six event types, covered by 54 tests at 98.7 percent line
coverage. It is deployed to testnet and initialized; the contract ID, admin, and
wasm hash are in the README's Deployment section. The optimized artifact is 11834
bytes for `wasm32v1-none`. That number is a reachability signal: code nothing
calls is stripped, so a commit adding a caller should grow it.

Where the sequence paired a test step with the implementation step after it, the
two landed as one commit, because in Rust a test calling a missing function is a
compile error, not a failing assertion. Tests were still written first and the
RED state verified locally. See ADR-014.

`crates/pulsar-decoder` holds the real decoder (PRs #5 and #6, ADR-020 and
ADR-021): `Decoder::decode_base64`, `decode_topics`, and `decode` map every
`ScVal` variant to the `DecodedValue` taxonomy, degrading unnameable variants to
`unknown` with a counter rather than erroring. It has 50 unit tests and a fixture
suite over a 31-case corpus copied from the Go indexer's testdata, at 94.8
percent line coverage. It is `publish = false`. Event assembly, a fuzz target,
and the crates.io publish remain for `v0.2.0-contracts`.

`Cargo.lock` is committed. It holds `ed25519-dalek` at 2.2.0 because
soroban-env-host declares an unbounded `>=2.0.0` requirement and 3.0.0 broke the
trait bounds its own code relies on.

Deployed to testnet. No published crate.
