# Architecture Decision Records: pulsar-core

Append-only. Never rewrite an entry. If a decision is reversed, append a new ADR that supersedes the old one and set the old entry's status to `superseded by ADR-NNN`.

Append-only applies to ADRs from the moment they are pushed. Before that first push, corrections to unpublished entries are edits, not violations.

Format:

```
## ADR-NNN: <title>
Date: YYYY-MM-DD
Status: accepted | superseded by ADR-MMM

### Context
### Decision
### Alternatives considered
### Consequences
```

---

## ADR-001: Adopt soroban-sdk 26.1.0 stable
Date: 2026-08-13
Status: accepted

### Context

At scaffold time soroban-sdk offers a 26.1.0 stable release and a 27.0.0 release candidate line. The contract in this repo is a fixture: other people's tests, documentation, and decoder implementations are calibrated against the event shapes it emits. A change in emitted shapes is a breaking change for every downstream consumer, whether or not the contract's own behavior is correct.

### Decision

Pin soroban-sdk at exactly `=26.1.0` in the workspace manifest. Do not adopt 27.0.0-rc.x.

The pin is exact rather than caret, following the dependency policy for `soroban-*` and `stellar/*` crates. A minor bump changing event encoding would silently invalidate committed fixtures.

### Alternatives considered

**Adopt 27.0.0-rc.** Rejected. Release candidates change before final. A fixture repo trading stability for unreleased features has the tradeoff backwards.

**Caret range `^26.1`.** Rejected. Permits automatic minor upgrades. Any change in the SDK's event encoding would then reach fixtures without a deliberate decision.

### Consequences

- Features landing in 27.x are unavailable until a deliberate upgrade.
- Upgrading requires a new ADR stating what changed and why, plus a full fixture comparison proving event shapes are unaffected.
- The integration test suite must pass against a new SDK version before any upgrade lands.

---

## ADR-002: Target wasm32v1-none
Date: 2026-08-13
Status: accepted

### Context

From Rust 1.82, the `wasm32-unknown-unknown` target emits WebAssembly features beyond the 1.0 feature set, including bulk memory operations and reference types. The Soroban runtime rejects modules using them, so contracts built with that target on a current compiler fail to deploy. Rust 1.84 added `wasm32v1-none`, which restricts output to the WebAssembly 1.0 feature set the runtime accepts.

### Decision

Build the contract for `wasm32v1-none`, declared in `rust-toolchain.toml` alongside the 1.84.0 channel pin. Build with `stellar contract build`, never with plain `cargo build`.

The 1.84.0 pin follows from the target: it is the earliest stable Rust that provides `wasm32v1-none`.

### Alternatives considered

**Stay on `wasm32-unknown-unknown` with an older compiler.** Rejected. Freezes the toolchain below 1.82 forever and forfeits every later compiler fix.

**Post-process the wasm to strip unsupported features.** Rejected. Adds a build step that can silently produce a module diverging from what the compiler verified.

### Consequences

- `rust-toolchain.toml` and the target are coupled. Changing either requires revisiting both.
- CI must install the target explicitly, which the toolchain file handles.
- Plain `cargo build` produces an artifact that is not deployable. Contributors are warned in CONTRIBUTING.md.
- TTL constants and other runtime-dependent values are verified against Soroban runtime documentation before the commit that first references them.

---

## ADR-003: One reference contract, not several
Date: 2026-08-13
Status: accepted

### Context

The toolkit needs contract events to decode in its tests and documentation. That need could be met by one contract covering many event shapes or by several contracts each modeling a realistic use case, such as escrow, multi-sig, or an oracle feed.

### Decision

Ship exactly one contract, `pulsar-showcase`, with six state-changing functions and two read views. Every function exists to exercise a specific decoder capability. A function that does not map to a decoder capability does not belong in the contract.

The scope is closed. Additional showcase contracts are a v0.3.0-contracts decision, starting with SEP-41, and each needs its own ADR.

### Alternatives considered

**Several contracts modeling realistic patterns.** Rejected for v0.1. Multiplies deployment, funding, and fixture maintenance while adding no decoder coverage that one contract cannot provide. Deferred to v0.3.0-contracts where the SEP-41 case earns its own keep.

**No contract, synthesize fixtures by hand.** Rejected. Hand-written XDR proves the decoder agrees with the author's belief about encoding, not with the runtime. Fixtures must come from real emitted events.

### Consequences

- Every proposed function faces one test: which decoder capability does it exercise. If there is no answer, it is not added.
- The contract stays small enough to review completely, which suits an unaudited fixture.
- Realistic contract patterns are not demonstrated here in v0.1. That is a documentation gap accepted for now and addressed at v0.3.0-contracts.

---

## ADR-004: Instance storage for Admin and Initialized, persistent for Balance
Date: 2026-08-13
Status: accepted

### Context

Soroban offers instance, persistent, and temporary storage with different lifetime and cost characteristics. Instance storage shares one TTL with the contract instance and is loaded whenever the contract is invoked. Persistent storage carries a per-entry TTL. Both are archived when TTL expires.

This contract holds three kinds of state: an initialization flag, an admin address, and per-address balances.

### Decision

`Initialized` and `Admin` live in instance storage. Both are small, bounded, and read on effectively every state-changing call, which is exactly what instance storage is priced for.

`Balance(Address)` lives in persistent storage. Its cardinality is unbounded because it grows with the number of addresses that ever hold a balance. Unbounded state in instance storage would grow the instance footprint on every invocation.

Temporary storage is not used and must not be added to this contract.

TTL is extended on the instance at the top of every state-changing function, and on a persistent entry on any read or write that retains it. Read views do not extend TTL.

### Alternatives considered

**Everything in persistent storage.** Rejected. Admin and Initialized are read on nearly every call, so per-entry TTL bookkeeping costs more than the shared instance TTL for no benefit.

**Balances in instance storage.** Rejected. Unbounded growth in the instance entry, which is loaded on every invocation regardless of which addresses a call touches.

**Temporary storage for balances.** Rejected. Temporary entries are unrecoverable once expired. Balances must survive.

### Consequences

- Every state-changing function starts with an instance TTL extension.
- Every persistent read that retains an entry pays for a TTL extension, which is why `get_balance` extends only when the balance is non-zero.
- A balance untouched past its TTL is archived and must be restored before use. Acceptable for a testnet fixture.
- Read views deliberately do not extend TTL, so a contract read alone never keeps an entry alive.

---

## ADR-005: Include an emit_custom endpoint
Date: 2026-08-13
Status: accepted

### Context

Five of the contract's events have topics fixed at compile time and typed data. Real Soroban contracts also emit events whose topics are computed at runtime and whose payloads are opaque bytes. A decoder that only ever sees compile-time-fixed topics can pass its entire test suite while failing on the first contract that builds a topic dynamically.

### Decision

Include `emit_custom(env: Env, tag: Symbol, payload: Bytes)`, which emits topics `(Symbol("custom"), tag)` with `payload` as data. It changes no state and exists only to emit.

Auth is the current admin, read from storage, not the caller. An unauthenticated event-only endpoint would let anyone write arbitrary entries into the contract's event history and poison the fixtures.

### Alternatives considered

**Omit it and test dynamic topics with hand-built XDR.** Rejected for the reason in ADR-003: synthesized fixtures prove agreement with the author's belief about encoding, not with the runtime.

**Allow any caller to invoke it.** Rejected. Anyone could flood the event log of the contract the whole toolkit tests against.

**Accept an arbitrary ScVal payload rather than Bytes.** Rejected for v0.1. `Bytes` is the opaque case that matters for decoder coverage. Broader ScVal coverage belongs with the decoder's own test suite at v0.2.0-contracts.

### Consequences

- The contract has a function with no state effect, which is deliberate and documented here so it is not mistaken for dead code and removed.
- The decoder gains a fixture for the dynamic-topic and opaque-payload case, the shape most likely to break naive implementations.
- Admin auth means the deploy script must invoke it with the admin identity when generating sample events.

---

## ADR-006: Positioning relative to Stellar Optics
Date: 2026-08-13
Status: accepted

### Context

Stellar Optics is a Go-only toolkit for Stellar and Soroban that ships three CLI tools plus a shared Go library:
- stellar-xdr-lens: single-value XDR decoding at the terminal
- stellar-prism: live-stream event decoding from RPC with NDJSON output
- stellar-focus: Go testing helpers for XDR assertions and golden fixtures

Nearest-neighbor project to Pulsar Stellar. Both make Soroban events more legible. Both decode XDR. Both consume RPC. Direct overlap exists between stellar-prism and the Pulsar Stellar indexer's polling loop, and between stellar-xdr-lens and the Pulsar decoder crate.

The question this ADR settles: how do we describe the difference clearly enough that a Stellar developer choosing between the two projects can make the right call, and how do we hold that positioning as both projects evolve?

### Decision

We hold Pulsar Stellar's positioning as follows:

Optics is inspection and testing tooling for Go developers at the terminal. Stateless, offline where possible, Unix-friendly, one job per tool. Its center of gravity is the developer's own workflow at the CLI.

Pulsar Stellar is a persistent event layer for developers building applications. Stateful (the indexer stores history past the 7-day RPC retention window), multi-language (Rust decoder for Soroban contract authors, TypeScript SDK for frontends and Node backends, Go indexer as the bridge), and includes a web surface (the explorer) so non-CLI users can inspect any contract's event history without installing anything.

The differences that matter, restated in one sentence for every pitch and every doc:

> Optics inspects and tests. Pulsar Stellar stores, serves, and displays.

Where a Go developer wants to decode a single blob at their terminal, Optics is the right tool. Where a Rust dapp builder wants typed events in their own code, or a wallet team wants historical events past RPC retention, or an auditor wants a paste-contract-ID web UI, Pulsar Stellar is the right tool. Both should exist. Ecosystems mature with multiple tools attacking the same underlying pain from different angles.

Cross-reference: the same decision lands in the sibling repo pulsar-app as ADR-011. That repo's system prompt seeds ADR-001 through ADR-010, so the shared decision takes the next free number there. In this repo it is the sixth ADR. The two entries carry the same context, decision, alternatives, and consequences, and are amended together.

### Alternatives considered

**Merge or absorb into Optics.** Rejected. Different language commitments (they are Go-only, we ship Rust + TS + Go), different surface commitments (they are terminal-only, we include a web app), and different persistence commitments (they are stateless, we index and store). Merging would compromise both projects.

**Compete directly and drive one out.** Rejected. Zero-sum framing is wrong for developer tooling. Vercel and Netlify coexist. pnpm and yarn and npm coexist. There is room for both projects, and both benefit from the Stellar developer ecosystem growing.

**Ignore Optics in our public positioning.** Rejected. It will come up during Drips Wave review, during Discord conversations, and during any grant application. Better to have a rehearsed, accurate answer than to be caught improvising.

### Consequences

- Every future pitch and README revision cites the "Optics inspects and tests, Pulsar stores serves and displays" framing.
- The differentiation narrows if Optics ships persistent indexing in stellar-prism. Watch their roadmap. If that shift happens, our defensible territory is the Rust decoder, the TypeScript SDK, and the web explorer. The Go indexer becomes the overlap zone.
- Interop is preferred over duplication where possible. If Optics' decoder matures and their maintainers are open, we may adopt stellar-xdr-lens internals inside our Go indexer to eliminate Rust-Go decoder drift. Cost is a dependency; benefit is one canonical Go decoder in the ecosystem. This is a separate ADR at that point, not now.
- Our docs will include a "when to use what" section pointing at Optics for CLI-first workflows and testing helpers. Mutual linking, if they reciprocate, strengthens both projects.

---

## ADR-007: Two-toolchain model (project vs host)
Date: 2026-08-13
Status: accepted

### Context

At Sprint 1 execution time, stellar-cli 27.1.0 (the version required by requirements.md Section 1.8) requires rustc 1.93.0 or newer to build. The project toolchain is pinned to rust 1.84.0 via rust-toolchain.toml because that is the minimum Rust version supporting the wasm32v1-none target required by soroban-sdk 26.1.0.

Attempting `cargo install --locked stellar-cli` from within the project directory uses the pinned 1.84 toolchain and fails.

### Decision

We operate a two-toolchain model:

1. Project toolchain (pinned in rust-toolchain.toml): Rust 1.84.0. Used for all contract builds (`stellar contract build`, `cargo build --target wasm32v1-none`) and all in-project cargo commands. This is what CI uses.

2. Host toolchain (system stable): whatever rustup's stable channel provides on the developer's machine (currently 1.95+). Used exclusively for installing host binary tools like stellar-cli that are not part of the project's dependency graph.

Host tools are installed with:

    cd ~ && rustup run stable cargo install --locked --force <tool>

The `cd ~` moves out of any directory with a rust-toolchain.toml override, and `rustup run stable` forces the stable channel explicitly.

### Alternatives considered

**Bump project toolchain to 1.93+.** Rejected. Not driven by our own code; only driven by a peripheral host tool. Coupling the contract's compiler pin to stellar-cli's minimum makes future stellar-cli upgrades force contract recompilation with unrelated Rust versions.

**Pin an older stellar-cli that supports 1.84.** Rejected. stellar-cli 22.8.1 supports 1.84 but predates several deploy features we rely on in Phase E scripts (step 53 to 55). Downgrading undoes those.

**Install stellar-cli from a prebuilt binary.** Acceptable fallback for CI or Docker images, and documented in CONTRIBUTING.md as an alternative. Not the default because compiling from crates.io keeps the install path uniform with every other cargo-based tool.

### Consequences

- requirements.md Section 1.8 will be updated to split the Rust pin into project vs host, with an explicit callout that stellar-cli requires a newer host Rust than the project toolchain.
- CONTRIBUTING.md (step 7) documents the install command with `cd ~ && rustup run stable cargo install ...` prefix.
- CI workflow (step 12) installs stellar-cli using the host stable channel, not the pinned project channel.
- Future stellar-cli bumps do not require a project toolchain bump unless the contract itself needs a newer Rust feature.

---

## ADR-008: Sequencing bugs in the Phase 6 build sequence
Date: 2026-08-14
Status: accepted

### Context

Local verification during step 13 execution uncovered three sequencing bugs in the Phase 6 build sequence where declarations preceded the artifacts they described:

1. **Missing lib root**: step 13 creates `contracts/showcase/Cargo.toml`, which declares `[lib]` with `crate-type = ["cdylib"]`, but `src/lib.rs` was sequenced at step 28. Cargo cannot parse a crate whose declared lib root is missing.

2. **Unresolvable workspace glob**: the root workspace at step 1 declares `members = ["contracts/*", "crates/*"]`, but `crates/pulsar-decoder` does not land until step 51. Cargo treats a glob matching zero directories as a literal path and fails to load the workspace.

3. **Missing lockfile**: `.gitignore` was written to keep `Cargo.lock` tracked, deliberately and per the README, for reproducible builds. No step in the sequence ever generated one. Cargo therefore had no version pins for transitive dependencies. This let soroban-env-host's unbounded `ed25519-dalek = ">=2.0.0"` requirement resolve to 3.0.0, whose breaking `rand_core` changes make the upstream code fail to compile.

All three would either cause consecutive red CI runs, violating the discipline that CI stays green, or in the third case allow silent breakage from upstream drift months after Sprint 1.

### Decision

Three targeted deviations from the sequence, applied in a single block during step 13:

1. Step 13 expanded to create both `contracts/showcase/Cargo.toml` and a minimal `contracts/showcase/src/lib.rs` stub. The stub contains a module doc comment, `#![no_std]`, which is mandatory for the `wasm32v1-none` target, and `use soroban_sdk as _;`, a linker directive forcing soroban-sdk to be pulled in so its panic handler is available. Both disappear at step 28 when `mod contract;` and its siblings reference the SDK directly.

2. Root `Cargo.toml` `members` narrowed from `["contracts/*", "crates/*"]` to `["contracts/*"]` in a follow-up commit landing immediately after step 13. The `crates/*` entry is restored at step 51 when `pulsar-decoder` lands.

3. `Cargo.lock` generated with `cargo generate-lockfile`, then `cargo update ed25519-dalek@3.0.0 --precise 2.2.0` to hold that transitive at the last compatible version, and committed. CONTRIBUTING.md documents that the lock file is committed and that regeneration requires its own commit naming the dependency that moved.

Each deviation preserves every downstream step's stated intent. Step 28 still rewrites `lib.rs` to add re-exports. Step 51 still creates the decoder crate and adds it to the members list. The lock file stays committed and moves only on deliberate dependency changes.

### Alternatives considered

**Reorder the build sequence to land `lib.rs`, the decoder placeholder, and lockfile generation earlier.** Rejected. Cascades to every step number already referenced in pushed commits, ADRs, and CI logic.

**Accept red CI between step 13 and step 51.** Rejected. Erodes CI signal discipline and trains the habit that red is expected.

**Skip `Cargo.lock` and rely on manifest version ranges.** Rejected. A wasm artifact uploaded to a ledger must be reproducible, and silent drift in a transitive dependency breaks the audit trail.

**Three separate ADRs, one per finding.** Rejected. All three share one root cause: the sequence declared things ahead of the artifacts they reference. One ADR captures the pattern, and splitting them would obscure that shared root.

### Consequences

- Step 13's commit title expands to name both files it creates.
- A `fix(workspace):` commit follows step 13 immediately to narrow the root manifest.
- A further commit adds `Cargo.lock` to the repository.
- Step 28's role tightens from "add lib.rs" to "rewrite lib.rs to add re-exports of contract, storage, events, error".
- Step 51 gains a matching workspace-manifest edit alongside creating the decoder crate. Missing that edit leaves the decoder outside the workspace.
- CONTRIBUTING.md gains a section on lock file discipline.
- The stub's `#![no_std]` and `use soroban_sdk as _;` are load-bearing for the wasm target and were not anticipated in the original expansion. `wasm32v1-none` supplies no `std` and no panic handler, and Rust does not link a dependency nothing references, so a doc-comment-only stub fails to compile with `#[panic_handler] function required, but not found`. They exist only to satisfy that target's compile requirements and are removed by step 28's rewrite of `lib.rs`.
- `docs/requirements.md` section 5 gets a note in a later fix-forward: future build sequences are checked for glob-versus-member and lockfile-versus-manifest ordering before execution starts.

---

## ADR-009: MSRV correction from 1.84 to 1.92
Date: 2026-08-14
Status: accepted

### Context

Local verification during step 13 execution surfaced that soroban-sdk 26.1.0 declares `rust-version = "1.91.0"` in its published manifest, read directly from `~/.cargo/registry/src/*/soroban-sdk-26.1.0/Cargo.toml`. The project had been pinning Rust 1.84.0 on the stated premise that this was the minimum version supporting soroban-sdk 26.1.0. That premise was factually wrong, and cargo refused to build the workspace with `soroban-sdk@26.1.0 requires rustc 1.91.0`.

Correcting that pin to 1.91.0 then hit a second constraint. stellar-cli 27.1.0 refuses to build contracts on that exact version, reporting `use a rust version other than 1.81, 1.82, 1.83 or 1.91.0 to build contracts`. The three lower versions in that list sit below the SDK's floor and were never candidates, so the blocklist reads as targeting releases with known bad wasm codegen. The pin is therefore bounded from below by the SDK at 1.91.0 and excluded at 1.91.0 by stellar-cli, leaving 1.92.0 as the lowest workable version.

The original premise entered the record by two paths:

- ADR-002 established `wasm32v1-none` as the build target and noted that 1.84 is the earliest stable release providing it. That is correct.
- ADR-007 stated in its Context that the project toolchain is pinned to 1.84.0 "because that is the minimum Rust version supporting the wasm32v1-none target required by soroban-sdk 26.1.0". The clause about the target is right in isolation, but attributing the pin to the SDK's requirements conflated the target's availability with the SDK's MSRV.

`docs/requirements.md` section 1.8 and the toolchain tables in README, CONTRIBUTING.md, and `.agent/context.md` all inherited the same premise.

### Decision

The project toolchain pin moves from 1.84.0 to 1.92.0:

- `rust-toolchain.toml` channel changes from `1.84.0` to `1.92.0`.
- Workspace `rust-version` changes from `1.84` to `1.92`.
- `docs/requirements.md` section 1.8's project row states 1.92.0 exactly.
- Toolchain tables in README, CONTRIBUTING.md, and `.agent/context.md` state 1.92.

1.92.0 is not chosen for anything it adds. It is the lowest version clearing soroban-sdk 26.1.0's declared MSRV of 1.91.0 that stellar-cli 27.1.0 is also willing to build contracts on.

ADR-002 is not amended. Its target decision stands on its own, and `wasm32v1-none` is present in 1.84 and every release after it, so the target neither constrains the pin upward nor downward.

ADR-007's Decision, Alternatives, and Consequences all stand. Its Context sentence about 1.84 is corrected here rather than by editing that entry, because the log is append-only.

The two-toolchain model holds unchanged: the project toolchain is pinned in `rust-toolchain.toml`, and the host stable channel builds host tools such as stellar-cli. Only the project pin's number changes.

### Alternatives considered

**Pin 1.91.0, the SDK's exact MSRV.** Rejected once verification showed stellar-cli 27.1.0 refuses that version for contract builds. `cargo metadata`, `cargo fmt`, and `cargo clippy` all pass on 1.91.0; only the wasm build is refused, and it is refused by policy rather than by a compile error. A pin that cannot produce a deployable artifact is not viable.

**Unify project and host at 1.93.0.** Rejected. It collapses the separation ADR-007 established and couples the contract's compiler pin to a peripheral tool's requirement, which is the coupling ADR-007 specifically rejected.

**Downgrade soroban-sdk to a release whose MSRV is 1.84 or lower.** Rejected. ADR-001 chose 26.1.0 deliberately for stability. Rolling the SDK back to preserve an incorrect toolchain premise optimizes the wrong variable.

**Amend ADR-002 and ADR-007 in place.** Rejected. The log is append-only, so corrections land as new entries that cross-reference the earlier ones.

### Consequences

- `rust-toolchain.toml`, the workspace manifest, `docs/requirements.md` section 1.8, README, CONTRIBUTING.md, and `.agent/context.md` are updated together.
- ADR-002 and ADR-007 both stand. This entry cross-references both, and a reader of ADR-007's Context is expected to read this entry alongside it.
- CI runs on 1.92.0 from here on. Contributors need it locally, which `rust-toolchain.toml` handles on the first cargo command.
- The two-toolchain model is validated rather than weakened by this correction. stellar-cli still needs 1.93 or newer to build itself while the contract builds on 1.92.0, so the two pins remain genuinely different and the separation is real.
- The pin is now traceable to a published manifest and a verified tool constraint rather than to reasoning about target availability. Future SDK bumps are checked against the SDK's declared `rust-version` and against stellar-cli's blocklist by running `stellar contract build` before the toolchain file is considered settled.
- A toolchain pin is not proven by cargo alone. `stellar contract build` is part of the verification set, because it enforces constraints cargo knows nothing about.
