# Context: pulsar-core

Onboarding for a fresh session, human or agent. Written at scaffold, updated at phase transitions only. If this file disagrees with the repository, the repository is right and this file needs an update.

## 1. What Pulsar Stellar is

Pulsar Stellar is a developer toolkit for Soroban contract events. Every Stellar project that needs to consume its contract's events today writes the same plumbing from scratch: XDR decoders, indexer glue, custom APIs. Pulsar Stellar provides three shared building blocks so they don't have to. A Rust library that turns raw contract events into typed data, a Go daemon that stores historical events past the seven-day RPC retention window, and a web explorer where anyone can paste a contract ID and browse every event that contract has ever emitted, decoded and searchable. It serves Soroban dapp builders, backend engineers integrating with existing protocols, and auditors reviewing contract behavior post-deployment.

That paragraph is locked wording. Reuse it verbatim wherever the project is described. Do not paraphrase it.

## 2. Why this repo exists

`pulsar-core` is the Rust contract layer. It carries two artifacts:

**`pulsar-showcase`** is a reference contract, not a product. Every function on it exists to exercise a specific decoder capability. `transfer` produces a three-topic event that doubles as the SEP-41 conformance case. `emit_custom` produces a dynamic Symbol topic with a Bytes payload, which is the case that breaks naive decoders. Its events are the fixtures the rest of the toolkit tests against, which is why the contract stays small and why its scope is closed at six state-changing functions and two read views.

**`pulsar-decoder`** is the typed event decoder, the correctness boundary for every downstream consumer. It ships as a placeholder in v0.1.0-contracts and gets its real content at v0.2.0-contracts, deliberately after the app repo proves what shape consumers actually need. Designing that API in isolation and reworking it later would cost more than waiting.

The two share a repo because they co-evolve. A new decoder capability needs a fixture to prove it, and a new showcase function exists to produce one.

## 3. Relationship to pulsar-app

`pulsar-stellar/pulsar-app` holds the TypeScript SDK, the Go indexer, the Next.js explorer, and the documentation site. It consumes what this repo produces.

The sync point is the deployed testnet contract ID. When v0.1.0-contracts is tagged, that ID goes into `pulsar-app` as `NEXT_PUBLIC_SHOWCASE_CONTRACT_ID` and `PULSAR_INDEXER_BOOTSTRAP_CONTRACTS`, which unblocks its Sprint 4.

Nothing in this repo may create `.ts`, `.tsx`, `.go`, or `next.config` files. Reaching for one means the work belongs in the sibling repo.

Note that the Go indexer reimplements decoding rather than calling into the Rust decoder. The two are kept honest against shared fixtures. Consolidating them through FFI is on the roadmap as a future direction, triggered only if the two decoders are observed to drift on a real contract.

## 4. Current phase and definition of done

**Phase 6, Sprint 1. Build sequence Phase A, scaffolding.** Current release target: `v0.1.0-contracts`.

Sprint 1 covers build steps 1 through 28 and is done when the contract skeleton compiles: workspace and toolchain pinned, agent context files in place, CI green, `error.rs` and `storage.rs` and `events.rs` complete, and `contract.rs` holding a contract struct with an empty impl block. No public functions yet. Those are Sprint 2.

`v0.1.0-contracts` as a whole is done when all of the following hold:

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

Toolchain. `rustup` reads `rust-toolchain.toml` and installs Rust 1.84.0 with the `wasm32v1-none` target on the first cargo command. stellar-cli is a host binary tool, installed with the host stable channel rather than the project pin, per ADR-007. Running the install from inside this directory fails because stellar-cli needs rustc 1.93 or newer and the directory pins cargo to 1.84.0.

```sh
cd ~ && rustup run stable cargo install --locked --force stellar-cli

rustc --version      # expect 1.84.0, the project pin
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

`scripts/build.sh` and `scripts/deploy-testnet.sh` wrap the last two blocks and land at build steps 53 and 54. Until then, run the commands directly.

Coverage.

```sh
cargo llvm-cov --workspace   # floor is 85 percent line coverage
```

## 8. State at the time of writing

Build steps 1 through 8 have landed. Next is step 10, seeding the ADR log.

The workspace compiles nothing yet: `Cargo.toml` declares members through globs that stay unresolvable until `contracts/showcase` lands at step 13. Cargo commands against the workspace fail until then, which is expected and not a defect.

No contract code, no deployment, no published crate.
