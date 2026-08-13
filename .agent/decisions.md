# Architecture Decision Records: pulsar-core

Append-only. Never rewrite an entry. If a decision is reversed, append a new ADR that supersedes the old one and set the old entry's status to `superseded by ADR-NNN`.

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
