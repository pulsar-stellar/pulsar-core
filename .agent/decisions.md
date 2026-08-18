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

---

## ADR-010: Install stellar-cli in CI from a pinned release binary
Date: 2026-08-18
Status: accepted

### Context

ADR-007 established that stellar-cli is a host tool, installed on the host stable
channel rather than the pinned project toolchain, and named `cargo install` as
the install path for both local development and CI.

Building it from crates.io on a GitHub runner turned out to be a poor fit. The
compile pulls in `libdbus-sys`, which needs a system `libdbus-1-dev` the runner
image does not carry, and even where the build succeeds it costs over four
minutes on every cache miss. The `contract-build` job needs the binary and
nothing else from it, so the entire compile is overhead.

### Decision

CI installs stellar-cli from the project's published release archive rather than
compiling it. The download is pinned by SHA-256, checked with `sha256sum
--check --strict` before the archive is unpacked, and the resulting binary is
cached under a key carrying the version.

This supersedes ADR-007's CI clause only. Local development still installs with
`cd ~ && rustup run stable cargo install --locked --force stellar-cli`, and the
two-toolchain model ADR-007 established is unchanged.

### Alternatives considered

**Install `libdbus-1-dev` in the workflow.** Rejected. It fixes the build failure
but keeps the four-minute compile, and it adds a system package dependency to a
job that only wants one binary.

**Drop stellar-cli from CI.** Rejected. The `contract-build` job exists to prove
the wasm artifact builds, which is the one check plain cargo cannot perform, and
ADR-002 warns specifically about target regressions reaching deploy time.

**Pin by version tag without a digest.** Rejected. A release asset can be
replaced at its existing tag. The digest makes a swapped or truncated download
fail at the check rather than silently producing a different binary.

### Consequences

- The workflow carries a `STELLAR_CLI_SHA256` value that must be updated
  alongside `STELLAR_CLI_VERSION`. A mismatch fails the job loudly, which is the
  intended behavior.
- CI and local development install stellar-cli by different mechanisms. Both are
  documented, and both produce the same pinned version.
- Bumping stellar-cli is a two-line change plus a fresh digest.

---

## ADR-011: Declare each module in lib.rs as it lands
Date: 2026-08-18
Status: accepted

### Context

ADR-008 expanded step 13 to create a stub `lib.rs` so the crate would parse.
The stub declared no modules, and the sequence placed the `mod` declarations at
step 28, after every module file had landed.

Step 14 showed what that costs. A module file that nothing declares is not
compiled: cargo never reads it, rustfmt never formats it, and clippy never lints
it. CI reported green on `error.rs` while nothing had checked whether it even
parsed. Carrying that through step 27 would have stacked fourteen unverified
files, with step 28 the first moment any of them met the compiler.

### Decision

Each step that creates a module file also declares it in `lib.rs` in the same
commit. `mod error;` landed with `error.rs`, `mod storage;` with `storage.rs`,
and so on.

Step 28's role narrows accordingly, from introducing the module declarations to
adding the `pub use` re-exports that expose the public surface. ADR-008
anticipated that narrowing; this entry makes it explicit.

### Alternatives considered

**Keep the orphan files and verify each one by temporarily declaring it before
committing.** Rejected. It works, and it was how step 14 was actually checked,
but it makes verification procedural rather than mechanical. CI green stops
meaning "the committed files compile", and the next contributor has no way to
know the manual step exists.

**Defer all module work to a single later commit.** Rejected. It undoes the
incremental sequence and produces one unreviewable change.

### Consequences

- Every module-adding commit touches `lib.rs` by one line.
- CI green means the committed code compiles, for every commit rather than for
  the first one after step 28.
- The `use soroban_sdk as _;` linker directive in the stub came out earlier than
  ADR-008 predicted. That entry expected it to survive until step 28, because it
  existed only to pull soroban-sdk in so its `#[panic_handler]` would be linked
  into a `no_std` wasm build. Declaring `mod error;` at step 14 made `error.rs`
  reference the SDK directly, which satisfied the linker on its own and made the
  directive redundant fourteen steps early.

---

## ADR-012: Instance TTL bump of seven days
Date: 2026-08-18
Status: accepted

### Context

ADR-004 set a thirty-day TTL bump for both instance and persistent storage. When
the constants were written, checking them against the canonical token contract in
`stellar/soroban-examples` showed a different convention: seven days for the
instance, thirty for balances, both expressed as multiples of a
`DAY_IN_LEDGERS` constant.

The showcase contract is a reference developers copy patterns from, which gives
matching ecosystem convention more weight than it would otherwise carry.

### Decision

The instance bump is seven days. Persistent entries stay at thirty.

This narrows ADR-004 rather than superseding it. That entry's split between
instance and persistent storage stands, and so does its rule that instance TTL is
extended at the top of every state-changing function. Only the specific duration
for the instance changes.

Both durations are expressed as multiples of `DAY_IN_LEDGERS`, itself derived
from Stellar's roughly five second ledger close time, so the numbers read as
durations rather than as unexplained constants. The threshold sits one day under
the bump in both cases, following the same convention.

### Alternatives considered

**Keep thirty days for the instance.** Rejected. Instance state is small and gets
bumped on nearly every state-changing call, so a shorter window costs almost
nothing and keeps archived instances from lingering longer than they need to.

**Match the token contract on both durations.** Not applicable: it already uses
thirty days for balances, which is what ADR-004 chose.

### Consequences

- A contract left idle is archived after seven days rather than thirty. For a
  testnet fixture this is acceptable and arguably desirable.
- Balances outlive the instance that manages them. Restoring an archived instance
  does not require restoring every balance, and vice versa.
- The constants read as `7 * DAY_IN_LEDGERS` and `30 * DAY_IN_LEDGERS`, so
  changing a duration is a change to one number with an obvious unit.

---

## ADR-013: TTL extension belongs with the write for persistent entries and at the entry point for instance state
Date: 2026-08-18
Status: accepted

### Context

Writing the storage helpers raised a question the spec answers only by example:
which layer extends TTL. Section 6.1 shows the instance extension at the top of a
public function and the persistent extension inside the balance write helper, but
does not say why the two differ, and the asymmetry looks like an inconsistency
until the storage semantics are considered.

### Decision

The rule follows the storage class, because the two classes have different TTL
semantics.

Instance storage shares one TTL with the contract instance. Extending it once
per invocation covers every instance read and write in that call, so the
extension lives at the public function's entry point through
`extend_instance_ttl`, which is where ADR-004 already places it. Instance-write
helpers such as `set_admin` perform only the write.

Persistent storage carries a TTL per entry. Extending it is meaningful only in
relation to a specific key, so the extension lives beside the write that creates
or refreshes that entry. `set_balance` writes and extends together.

The canonical token contract in `stellar/soroban-examples` follows exactly this
split: `write_administrator` performs only the write, while `write_balance`
writes and extends.

### Alternatives considered

**Extend in every write helper, including instance ones.** Rejected. A public
function already extends the instance TTL at its entry, so a second call inside
`set_admin` is a no-op that splits one responsibility across two layers and
invites a reader to wonder which one is authoritative.

**Extend only at public function entry, including for persistent entries.**
Rejected. The entry point does not know which persistent key a helper will touch,
so the extension would have to name the key twice or extend the wrong entry.

### Consequences

- Instance-write helpers are pure writes. Persistent-write helpers write and
  extend.
- `set_admin` and `set_balance` look inconsistent side by side, so the doc comment
  on `set_admin` explains the asymmetry at the point a reader would question it.
- Any future helper is classified by its storage class before it is written.
- ADR-019 later carves out the read-only case, where a persistent read that will
  not be followed by a write must not extend at all.

---

## ADR-014: Behavior-carrying helpers get tests; pass-throughs do not
Date: 2026-08-18
Status: accepted

### Context

Phase B lands types, storage helpers, and event definitions before any public
function calls them. Applying test-first uniformly would have meant writing
assertions for code with no observable behavior, and skipping it uniformly would
have meant shipping untested decisions.

Rust sharpens the question further. A test calling a function that does not exist
is a compile error, not a failing assertion, so a test-only commit preceding its
implementation leaves the branch unbuildable and CI red.

### Decision

A helper is behavior-carrying if it encodes a decision that a downstream reader
depends on. Those get tests. A helper is a pass-through if it adds nothing the
SDK does not already provide, and those are covered through the public functions
that call them.

The distinction is decision-carrying versus SDK-passthrough, not the narrower
"has logic versus has no logic". `get_admin` converts an absent key into a typed
error, `get_balance` extends TTL conditionally, and every event struct fixes a
wire shape the decoder matches on. All are behavior-carrying. `set_admin` is a
single unconditional SDK call and is not.

Tests and implementation land in one commit for behavior-carrying work. Tests are
still written first and the RED state verified locally, it simply does not get its
own commit. Test-only commits remain valid when they close a coverage gap on
behavior already shipped.

Three practices make the tests worth having:

**Mutation as specification.** For each behavior-carrying test, name the mutation
that should fail it, introduce the mutation, and confirm exactly the expected
tests fail. This states precisely which behavior each test guards.

**Splitting is coverage-critical, not stylistic.** A test whose name needs "and"
can hide which behaviors are covered, because one mutation failing one merged
test is indistinguishable from one mutation failing the assertion that matters.
The withdraw tests demonstrated this: merged, every mutation failed the single
test and coverage looked complete; split, removing the balance check failed
nothing, because no test yet withdrew more than the balance.

**Value equivalence hides behavior differences.** Two implementations returning
identical values can differ in side effects. Where a helper's contract includes
both a return value and a side effect, assert both. `read_balance` and
`get_balance` return the same number for the same input, and only a TTL
assertion tells them apart.

### Alternatives considered

**Test every helper.** Rejected. Tests over pass-throughs restate the
implementation or re-verify the SDK, and they cost review attention that the
decision-carrying code deserves instead.

**Test nothing until Phase C.** Rejected. It defers all feedback on the storage
and event layer to the moment public functions arrive, which is when several
failures at once are hardest to attribute.

**Land the failing test as its own commit, as the sequence describes.** Rejected
for the language reason above. Verified rather than assumed: a test calling an
absent function fails with E0425 at compile time.

### Consequences

- Phase B commits pair tests with implementation. The sequence's separate test
  and implementation steps collapse where it pairs them.
- RED is verified locally and reported in the commit message rather than being
  visible as a red commit on `main`.
- Every commit on `main` builds, which keeps the three-consecutive-green release
  gate meaningful.
- Test-first surfaces problems in the test itself, not only in the code. Writing
  tests after an implementation lets them bend to match whatever the code happens
  to do.

---

## ADR-015: Test code may use expect with a descriptive message
Date: 2026-08-18
Status: accepted

### Context

Requirements section 5.1 forbids `unwrap`, `expect`, and `panic!` in contract
code, because a panic in a deployed contract is an untyped failure a caller
cannot handle. Test code has the opposite relationship to panics: a failing
assertion is how a test reports, and the panic message is the diagnostic.

### Decision

Test code may use `.expect("message")` with a non-empty descriptive message. A
bare `.expect("")` or an unwrap standing in for an assertion is not acceptable,
because the panic then carries no more information than a line number.

Production code, meaning everything under `src/` outside a `#[cfg(test)]` block,
remains fully bound by the no-panic rule.

The mechanical check enforces the same split: it scans `src/` up to the first
`#[cfg(test)]` marker and skips integration tests under `tests/` entirely.

### Alternatives considered

**Apply the no-panic rule everywhere.** Rejected. It would force tests to convert
every fallible setup step into a `Result` and propagate it, which adds noise to
the part of a test that is not under test and makes failures report worse.

**Allow anything in test code.** Rejected. A bare `unwrap` in a long test tells a
reader nothing about which step failed, and test code is read most often at
exactly the moment it has failed.

### Consequences

- Setup steps in tests read directly, with their failure messages naming the step.
- The no-panic scanner is scoped rather than global, and its scoping is part of
  the rule rather than an implementation detail.
- A reviewer seeing `expect` in a diff checks which side of the boundary it is on.

---

## ADR-016: single-value data format enforces the topic and data split at compile time
Date: 2026-08-18
Status: accepted

### Context

Event structs annotate with `#[contractevent]`, which derives the wire shape from
the declaration: `#[topic]` fields become topics and the rest become data. The
macro's `data_format` defaults to `Map`, which encodes the data section as a map
keyed by field name.

The contract's events each carry one payload, and the decoder expects that
payload as a bare value.

### Decision

Every payload-carrying event annotates `data_format = "single-value"`
explicitly.

The annotation is load-bearing twice. It encodes the payload as a bare value
rather than a single-entry map, which is the shape the decoder reads. It also
constrains the struct: `single-value` permits at most one non-topic field, so
promoting a field to a topic or demoting one out of the topic set changes the
data field count and fails compilation.

That second property is the more valuable one. Moving a field between topics and
data is a wire-contract change that no test would catch unless it happened to
assert that exact event, and it is the kind of edit that looks like a formatting
change in review. Under this annotation it is a build failure.

### Alternatives considered

**Let the default `Map` format stand.** Rejected. It changes every payload to a
field-name-keyed map, which is a different wire shape from the one the decoder
and the fixtures expect, and the field name becomes part of the wire contract.

**Annotate only where the shape matters.** Rejected. It matters on every event,
and a partially applied convention is one a contributor has to reason about
rather than follow.

### Consequences

- Every event struct carries the annotation, including single-field ones.
- Demoting a topic to data, or the reverse, is a compile error rather than a
  silent wire change. Verified: removing `#[topic]` from `Deposit.from` fails to
  build with "single-value requires exactly 0 or 1 data fields".
- An event needing two data fields cannot use this format, and adding one is a
  deliberate wire-shape decision rather than an incidental one.
- The decoder can rely on every event this contract emits having its topics and
  its payload cleanly separated.

---

## ADR-017: Read each balance immediately before its write
Date: 2026-08-18
Status: accepted

### Context

`transfer` touches two balance entries. Its first implementation read both
balances up front, then wrote both:

    let from_balance = get_balance(&env, &from);
    let to_balance = get_balance(&env, &to);
    set_balance(&env, &from, from_balance - amount);
    set_balance(&env, &to, to_balance + amount);

That is correct when the two addresses differ and wrong when they are the same.
A self-transfer reads the same balance twice, then writes twice, and the second
write is computed from a figure captured before the first write landed. The
result is a balance inflated by the transferred amount: the contract mints value
out of an aliasing bug.

### Decision

A function touching more than one balance entry reads each balance immediately
before the write that consumes it, never both up front.

    let from_balance = get_balance(&env, &from);
    set_balance(&env, &from, from_balance - amount);
    let to_balance = get_balance(&env, &to);
    set_balance(&env, &to, to_balance + amount);

The self-transfer test is a regression guard, not a curiosity. It asserts that a
transfer to oneself leaves the balance unchanged, and it fails against the
read-both-then-write-both ordering.

### Alternatives considered

**Reject self-transfers.** Rejected. It adds an error path for a case with no
economic effect and no fraud potential, and SEP-41 does not require it, so a
conformant consumer would not expect the rejection. It also fixes the symptom
rather than the ordering that caused it: a future two-balance function would
reproduce the same bug.

**Detect aliasing explicitly and branch.** Rejected. It requires every
multi-entry function to remember the check, whereas the ordering rule makes
aliasing a non-event.

### Consequences

- Any future function touching multiple balance entries follows the ordering, and
  gets a same-address test.
- The self-transfer test must not be removed as redundant. It looks like it
  asserts nothing, and it is the only test that fails against the aliasing bug.
- The ordering is stated in a comment at the point in `transfer` where a reader
  would otherwise be tempted to hoist the reads together.

---

---

## ADR-018: Remove the Unauthorized error variant
Date: 2026-08-17
Status: accepted

### Context

The error enum shipped with an `Unauthorized` variant on discriminant 3, intended
for callers who are not the admin. Implementing `set_admin` showed it can never be
returned.

Authorization is enforced with `require_auth`, which is checked by the host. When
it fails, the host raises the failure itself and the contract function never runs
to completion, so no code path can construct `Error::Unauthorized`. The failure
surfaces to a caller as a host invocation error rather than as a contract error
value. Both of `set_admin`'s authorization tests assert `is_err()` for this
reason: there is no typed variant to match against.

A variant no code can return is a false promise. A consumer writing a match arm
for it would be writing dead code, and the release criterion requiring every
variant to be triggered by a test could not be met.

### Decision

Remove `Unauthorized`. The remaining variants are compacted to 1 through 4:
`AlreadyInitialized`, `NotInitialized`, `InsufficientBalance`, `AmountOutOfRange`.

Compacting rather than leaving a hole is safe only because the contract has not
been deployed. Discriminants are wire-visible, so from the first deployment
onward the numbering is frozen and a removed variant would leave its number
retired rather than reused. The module doc in `error.rs` states this.

### Alternatives considered

**Keep it as a reserved variant.** Rejected. It documents an error the contract
cannot produce, and the release criterion would need weakening from "every
variant" to "every reachable variant" to accommodate one unreachable entry.

**Add a code path that returns it**, comparing the caller against the stored admin
explicitly rather than relying on `require_auth`. Rejected, and it is the more
dangerous option: hand-rolled address comparison is precisely the pattern the auth
rule warns against, and it would duplicate a check the host already performs
correctly. Reachability is not worth a weaker guard.

### Consequences

- The enum contains only variants a caller can actually receive.
- Authorization failures are host errors. Tests assert `is_err()` rather than
  matching a variant, and the comment at each site says so.
- Any future function authorizing against stored state follows the same pattern
  and needs no new variant.
- Discriminants are frozen from first deployment. This is the last commit in which
  renumbering is available.

---

## ADR-019: Separate read-only and read-before-write balance helpers
Date: 2026-08-18
Status: accepted

### Context

Adding the `balance` public view exposed a conflict between two rules already in
force.

ADR-013 pairs a persistent write with its TTL extension, which is why
`get_balance` extends when it returns a live balance: it is the read half of a
read-then-write, and the entry it returns is about to be written.

ADR-004 says read views do not extend TTL, so observing a balance never keeps the
entry alive. A public view routed through `get_balance` would have broken that
rule, and removing the extension from `get_balance` would have broken ADR-013.

### Decision

Two helpers, each named for its caller.

`get_balance` keeps the extend-on-positive-read behavior and is for
state-changing functions where the read precedes a write to the same entry.
`deposit`, `withdraw`, and `transfer` use it.

`read_balance` reads persistent storage, returns zero for an absent key, and
leaves the entry's lifetime exactly where it found it. The `balance` public view
uses it, and so does any future read-only path.

Both doc comments say which to use and why, because the two are otherwise
indistinguishable at a call site.

### Alternatives considered

**Route the view through `get_balance`.** Rejected. Every read through the
explorer would extend entry lifetimes, which is the behavior ADR-004 exists to
prevent, and no test of the returned value would notice.

**Remove the extension from `get_balance` and extend explicitly in each
state-changing caller.** Rejected. It moves the pairing ADR-013 established out
of the helper and into three call sites, where it can be forgotten.

**Add a boolean parameter selecting whether to extend.** Rejected. Call sites
would read `get_balance(&env, &addr, false)`, which says nothing about why.

### Consequences

- Two helpers differing by one side effect, distinguishable only by their TTL
  behavior. The doc comments carry that weight.
- The test that pins the difference asserts TTL rather than a value, since values
  are identical by construction. Pointing `read_balance` at `get_balance` fails
  only that test.
- A future read-only accessor uses `read_balance`. A future state-changing one
  uses `get_balance`.

