# Pulsar Core — Repository Roadmap

**Repository home**: eventually at `pulsar-stellar/pulsar-core/ROADMAP.md`
**Owner**: Emedit
**Related documents**: `pulsar-stellar-roadmap.md` (product-level), `pulsar-core-system-prompt.md` (Sprint 1 execution guide)
**Last updated**: at the start of the project, before any code

---

## Scope of this document

This roadmap covers the `pulsar-core` repository only: the Rust workspace containing the `pulsar-showcase` reference contract and the `pulsar-decoder` crate. For the app layer (SDK, indexer, web explorer, docs) see `pulsar-app`'s roadmap. For the overall product path see `pulsar-stellar-roadmap.md`.

`pulsar-core` ships two artifacts that grow independently:

- **`pulsar-showcase`** (and future showcase contracts): reference implementations used as decoder fixtures, docs demos, and educational material for Soroban contract authors
- **`pulsar-decoder`** (published as a crate on crates.io, later as a wasm package on npm): the canonical typed event decoder for Rust and browser consumers

These artifacts share a repo because they co-evolve: any new event pattern the decoder supports needs a fixture to prove it against, and any new showcase contract exists to exercise a decoder capability.

## Versioning

Track: `pulsar-core@v{MAJOR}.{MINOR}.{PATCH}-contracts`

Aligned to product-level milestones (`v0.1`, `v0.2`, `v1.0`) but released independently. Bumped when a public artifact changes: new deployed contract, new published crate version, breaking API change.

Crate versions follow strict semver from v0.2 onwards. v0.1 is scaffold-only and carries no compatibility guarantee.

---

## Now

Empty GitHub repo. Phase 6 system prompt (`pulsar-core-system-prompt.md`) drafted and ready. No code, no CI, no deployments. Sprint 1 begins with handing that prompt to Claude Code in the local clone.

---

## Release track

### v0.1.0-contracts — reference contract on testnet

**Sprint**: 1
**System prompt**: `pulsar-core-system-prompt.md` (drafted)
**Build sequence**: Phase 6 steps 1 through 57
**Estimate**: 15 to 25 focused hours

**What ships**:

- Cargo workspace scaffolded with `rust-toolchain.toml` pinning 1.84 and `wasm32v1-none`
- Agent context files in place: `CLAUDE.md`, `.agent/context.md`, `.agent/decisions.md` (seeded with ADR-001 through ADR-005), `.agent/glossary.md`
- `pulsar-showcase` contract with six state-changing functions and two read views, all tested happy-path and failure-path
- Storage discipline: instance storage for `Admin` and `Initialized`, persistent for `Balance` with TTL extension on every touching call
- Auth discipline: `require_auth` on every state-changing function, current-admin auth for `set_admin` and `emit_custom`
- Event emission through central helpers in `events.rs`, one helper per event
- Deploy script that deploys the contract to testnet, initializes it, emits one sample event per function
- `pulsar-decoder` crate placeholder (empty lib, module comment referring to v0.2)
- CI workflow green: rustfmt, clippy pedantic, cargo test, `stellar contract build`
- `v0.1.0-contracts` release tag with deployed testnet contract ID in the tag body

**Exit gates**:

- `stellar events --network testnet --contract-id <id>` returns decoded event samples for every emitted event type
- README documents the deployed contract ID and quick-start commands
- CI has been green for at least 3 consecutive commits
- `.agent/context.md` updated to reflect Sprint 2 readiness with a pointer to what generates the next system prompt

**Handoff to the app layer**: the deployed contract ID gets captured into `pulsar-app`'s `.env.example` as `NEXT_PUBLIC_SHOWCASE_CONTRACT_ID` and `PULSAR_INDEXER_BOOTSTRAP_CONTRACTS`. This is the sync point where `pulsar-app` Sprint 2 becomes unblocked.

### v0.2.0-contracts — decoder crate first release

**Sprint**: parallel to `pulsar-app` Sprints 2 through 4 (does not block them; app indexer uses its own Go decoder)
**System prompt**: to be generated at Sprint 2 start, following the Phase 6 pattern applied to the decoder crate specifically. Come back to chat before starting this sprint to produce it.
**Estimate**: 25 to 35 focused hours

**What ships**:

- `pulsar-decoder` crate implements the `ScVal` to `DecodedValue` mapping, matching the shape defined in `pulsar-app`'s SDK types (Section 6.4 of the app system prompt)
- Public API surface:
  - `Decoder::from_wasm_bytes(wasm: &[u8]) -> Result<Self, DecoderError>`
  - `Decoder::from_contract_id(contract_id: &str, rpc_url: &str) -> Future<Result<Self, DecoderError>>` (native only, gated behind `net` feature)
  - `Decoder::decode_event(&self, raw: &RawEvent) -> Result<DecodedEvent, DecoderError>`
  - `Decoder::events(&self) -> impl Iterator<Item = &EventSchema>`
- Full `DecodedValue` enum covering: Address, Symbol, I128, U128, Bytes, String, Bool, Vec, Map, Tuple, Void, plus explicit `Unsupported(scval_type)` for future ScVal types
- Zero-panic guarantee: every decode failure returns a `DecoderError` variant, never panics
- Serde support behind a `serde` feature flag for consumers that want JSON serialization
- Integration test suite: for every event `pulsar-showcase` emits, assert exact decoded shape. Fixtures pulled from testnet, not synthesized.
- Fuzz target (cargo-fuzz) for the `decode_event` function
- Published to crates.io as `pulsar-decoder` version 0.1.0
- Docs.rs page renders cleanly with all public items documented

**Exit gates**:

- `cargo publish --dry-run` passes
- crates.io badge in README shows a real version
- Fuzz target runs for at least 30 minutes without a crash
- Round-trip test: emit event on testnet from `pulsar-showcase`, fetch via RPC, decode via `pulsar-decoder`, assert equality with hand-computed expected shape
- Public API committed to backward compatibility within the 0.1.x line

### v0.3.0-contracts — SEP-41 token showcase

**Sprint**: after `pulsar-app` v0.1 ships and the toolkit has real users
**Estimate**: 20 to 30 focused hours

**What ships**:

- New `contracts/token-showcase/` implementing the full SEP-41 fungible token specification
- All SEP-41 events emitted: `transfer`, `mint`, `burn`, `approve`, `clawback`, `set_admin`, `set_authorized`
- Decoder tested against `token-showcase` events with no special-case code (proves SEP-41 conformance)
- Contract deployed on testnet alongside `pulsar-showcase`
- Documentation for SEP-41 event handling patterns in `.agent/glossary.md` and in the `pulsar-app` docs
- New crate release: `pulsar-decoder` 0.2.0 (only if SEP-41 exercises decoder paths not already covered; otherwise no bump)

**Rationale**: `pulsar-showcase` covers primitive event shapes for testing purposes. SEP-41 is the standard token contract every wallet, DEX, and DeFi protocol on Stellar interacts with. Having a canonical, well-documented SEP-41 example alongside the toolkit strengthens the decoder's real-world credibility and gives new Soroban developers a starting point that's not tied to a specific protocol.

**Exit gates**:

- Token showcase deployed on testnet and interacts correctly with the Stellar Asset Contract standard
- SEP-41 conformance test suite passes (every event shape, every required function)
- README lists both showcase contracts with their contract IDs

**Trigger to start**: at least one external user has asked "how do I decode SEP-41 events with this?", OR three months have passed since v0.2 landed and this is the next planned enhancement.

### v0.4.0-contracts — wasm bindings for the browser

**Sprint**: after v0.3 or when triggered by app SDK needs
**Estimate**: 15 to 25 focused hours

**What ships**:

- `wasm-pack` build target added to `pulsar-decoder`
- Published to npm as `@pulsar-stellar/decoder-wasm` (source-of-truth stays in `pulsar-core`; publish is cross-repo via CI)
- Bundle size budget: under 200 kB gzipped
- Browser smoke test: an HTML page that decodes a real testnet event using the wasm package with no server involvement
- `pulsar-app` SDK updated to consume `@pulsar-stellar/decoder-wasm` for optional local decoding path

**Rationale**: browsers can decode events without hitting the indexer's HTTP API. Reduces latency, enables offline-first tools, and provides a fallback when the indexer is unavailable. Deferred from v0.1 because it multiplies complexity and doesn't ship any new capability the indexer path doesn't already cover.

**Exit gates**:

- `npm publish` successful
- App SDK integration test passes with local decoding path enabled
- Bundle size in CI under the budget

**Trigger to start**: an app SDK consumer explicitly requests browser-side decoding, OR v0.3 lands and adoption metrics justify the effort.

### v0.5.0-contracts — event helper macros for contract authors

**Sprint**: only when adoption feedback justifies it
**Estimate**: 20 to 30 focused hours

**What ships**:

- `pulsar-events` proc-macro crate:
  - `#[derive(PulsarEvent)]` on a struct auto-generates the emit helper
  - Enforces topic and data shape at compile time
  - Backwards compatible with hand-rolled `env.events().publish` patterns
- Reference `pulsar-showcase` migrated to use the derive
- Documentation for contract authors with a full before-and-after example
- Compatibility statement with soroban-sdk 26.x (and 27.x if it's landed by then)

**Rationale**: contract authors today write `emit_deposit`, `emit_withdraw`, `emit_transfer` helpers by hand. Boilerplate. A derive macro reduces boilerplate and enforces schema consistency at the source. If contracts adopt it, the decoder side gets more predictable event shapes to work with.

**Exit gates**:

- Reference showcase migrated to macros with identical event emission behavior (asserted by fixture comparison)
- Docs include a full worked example
- Independently versioned as `pulsar-events` on crates.io

**Trigger to start**: at least three external contract authors ask for cleaner event emission ergonomics, OR upstream soroban-sdk shows no signs of shipping this itself in the next six months.

### v1.0.0-contracts — audit-ready and mainnet-committed

**Sprint**: only when sustained adoption exists
**Estimate**: 40 to 60 focused hours (including audit remediation cycles)

**What ships**:

- Security review of `pulsar-decoder` (either community review with documented threat model, or professional audit if funding allows)
- Mainnet compatibility test suite: decode a curated set of real mainnet contract events (Blend, Aquarius, SoroSwap) and assert correctness
- Extended fuzz corpus: 100+ hours of cumulative fuzz time without crashes
- Performance benchmarks published in the docs (target: 10000 events per second sustained decode on commodity hardware)
- Semver-stable API committed: no breaking changes within v1.x without a v2.0 major bump
- Migration guide for any breaking API changes from v0.x to v1.0
- crates.io yank policy documented for the v0.x line

**Rationale**: v0.x has "may break" implicit in the version. v1.0 is a public commitment. Only worth making when there are real consumers depending on stability.

**Exit gates**:

- Security review sign-off
- Performance benchmark documented and reproducible
- At least 5 production consumers on record (npm/crates.io download data + explicit user testimonials)
- Zero known critical bugs open for more than 30 days

**Trigger to start**: crates.io downloads sustain above 500 per month, AND at least 3 external projects use the decoder in production, AND at least one production consumer requests API stability guarantees.

---

## Future direction

Deferred by choice, not by omission. Every entry says what would move it forward.

### Multi-showcase library

Add showcases for common Soroban patterns beyond token behavior: escrow, multi-sig wallet, oracle price feed, streaming payment, DAO governance vote. Each becomes a decoder fixture AND a public reference implementation for Soroban developers learning the pattern.

**Trigger**: v0.3 ships and community requests specific pattern examples.

### Upstream contribution to soroban-sdk

Work with the Stellar SDK maintainers to upstream first-class `SCSpecEntry` event support (`rs-soroban-sdk#1097`). If merged, `pulsar-decoder`'s value narrows to runtime decoding while the schema half becomes native. This is a win for the ecosystem, not a threat to Pulsar.

**Trigger**: `pulsar-decoder` reaches v0.3 stability with a clear internal design; opens a design discussion upstream and offers to implement.

### Contract macros ecosystem

Extend `pulsar-events` into a broader set of proc-macros: `#[pulsar_storage]` for typed storage helpers with automatic TTL extension, `#[pulsar_error]` for enriched error types with backtrace support. Only if the initial macro sees real adoption.

**Trigger**: `pulsar-events` reaches 100+ dependent crates on crates.io.

### FFI bridge for the app indexer

Replace the Go indexer's hand-rolled decoder with FFI calls into `pulsar-decoder`. Eliminates the risk of Go and Rust decoders drifting apart. Costs: CGO complexity, deployment tooling, build pipeline.

**Trigger**: Go and Rust decoders drift in observed behavior on a real contract, OR a maintainer explicitly asks to consolidate.

### Formal verification

Not planned. Formal verification of the decoder crate would be interesting academically but expensive in practice. Pursued only if a grant or partnership funds it explicitly.

---

## Success signals per release

Labeled as targets, not commitments.

**v0.1.0-contracts**:
- Testnet contract responds to invocations from at least one external developer
- CI green sustained across the full 57-commit sequence
- The tagged contract ID becomes referenceable in `pulsar-app` docs and demos

**v0.2.0-contracts**:
- crates.io downloads: 50 to 100 in month one
- Decoder used by the `pulsar-app` indexer via at least one path (either FFI or as a schema authority)
- At least one external Rust project imports the crate

**v0.3.0-contracts**:
- SEP-41 token showcase referenced by at least one Stellar tutorial, blog post, or community doc
- Zero SEP-41 event shape unhandled by the decoder

**v0.4.0-contracts**:
- npm downloads for `@pulsar-stellar/decoder-wasm`: 30 to 50 in month one
- At least one browser-only consumer confirms usage (a wallet, an analytics dashboard, a contract inspector tool)

**v0.5.0-contracts**:
- Macros adopted by at least 3 external contracts (public repos on GitHub)
- Zero major API changes required in the first month post-release

**v1.0.0-contracts**:
- Security review sign-off documented
- Sustained multi-project production adoption
- Semver-stable API commitment kept for at least 90 days without breaking changes

---

## Risks specific to pulsar-core

Every risk has a mitigation. If a risk fires without a mitigation, it becomes an emergency triage session, not a project-ending event.

**soroban-sdk 27.x breaking changes before we upgrade**
- Mitigation: pinned to 26.1.0 in `Cargo.toml`. Integration test suite must pass cleanly against 27.x before any upgrade. Upgrade lands as a fresh ADR entry explaining what changed and why the upgrade is worth doing.

**`wasm32v1-none` target changes on Rust nightly**
- Mitigation: `rust-toolchain.toml` pinned to stable 1.84 or later. Nightly builds explicitly unsupported. Any target change in stable triggers a coordinated version bump.

**`SCSpecEntry` format evolution upstream**
- Mitigation: decoder tolerant of unknown spec entry variants (log with details, skip, do not crash). ADR captures the tolerance policy. Version bump when a new spec entry variant needs first-class handling.

**Decoder correctness bugs**
- Mitigation: fuzz testing from v0.2 onwards. Every ScVal type the decoder claims to handle has a fixture derived from a real testnet event. Regression tests grow monotonically (never delete a test that catches a bug).

**Small crate name collision or brand confusion**
- Mitigation: the `pulsar-decoder` crate name is distinctive; `pulsar-events` is the future crate; both live in the `pulsar-stellar` GitHub org. Documentation reinforces "Pulsar for Stellar" positioning to keep separation from Apache Pulsar in developers' mental models.

**Loss of testnet contract state affecting fixtures**
- Mitigation: fixtures include the raw XDR (base64), the ledger number, and the tx hash. Fixtures are checked into the repo, not fetched at test time. Testnet resets do not break tests.

**Solo maintainer bandwidth for both the crate and the app repo**
- Mitigation: Drips Wave contributors focus on `pulsar-app` (bigger surface, more scoped issues). `pulsar-core` stays tight and reviewer-only until v1.0. Explicit note in CONTRIBUTING.md that decoder changes require higher-bar review.

---

## Changelog

Every meaningful update to this document lands here.

- **YYYY-MM-DD**: Initial pulsar-core roadmap drafted, before any code

Update whenever a sprint completes, a release ships, or a directional decision changes.

---

**End of pulsar-core roadmap.**