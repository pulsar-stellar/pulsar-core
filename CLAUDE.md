# Pulsar Core

## Project

**Pulsar Stellar**: a developer toolkit for Soroban contract events. It provides a Rust library that turns raw contract events into typed data, a Go daemon that stores historical events past the seven-day RPC retention window, and a web explorer for browsing any contract's decoded event history.

## Repo role

`pulsar-core` is the Rust contract layer. It holds `pulsar-showcase`, the reference contract whose emitted events are the toolkit's test fixture and documentation demo, and `pulsar-decoder`, the typed event decoder crate.

Out of scope here: TypeScript SDK, Go indexer, Next.js explorer, GitBook docs. If you reach for a `.ts`, `.tsx`, `.go`, or `next.config` file, stop. Those belong to the sibling repo.

## Sibling repo

`pulsar-stellar/pulsar-app` at https://github.com/pulsar-stellar/pulsar-app

## Current phase

Phase 6, Sprint 1. Build sequence Phase A (scaffolding).

Steps 1 through 8 have landed. **Next: step 9**, `chore(agent): add .agent/context.md with full onboarding content`.

Do not skip forward in the build sequence. If a step reveals a design gap, halt and ask.

## Skills to load and cite

Restate which of these are active at the top of every task:

- `humanizer`
- `frontend-patterns`
- `coding-standards`
- `tdd-workflow`
- `blueprint`
- `security-review`

`security-review` is mandatory for every commit touching auth, storage, or events. `tdd-workflow` is mandatory for every function commit, and the test is written before the implementation.

## Non-negotiables

Short list. Full detail in `docs/requirements.md`.

- No em dashes anywhere in any output
- No `unwrap`, `expect`, or `panic` in shipped contract code. Tests may use them.
- No floats. Amounts are `i128`.
- Target `wasm32v1-none`, build with `stellar contract build`, never plain `cargo build` for the contract
- soroban-sdk pinned exactly at 26.1.0
- One commit per logical unit, push after every commit, never `git add .`
- Every code commit paired with a test commit, or the same commit for trivial cases
- Every state-changing function calls `require_auth` before any caller-dependent read or write
- Every event emission routed through a helper in `events.rs`
- Secrets never enter the repo
- Halt and ask on ambiguity, never guess

## Where the authoritative spec lives

- **Contract behavior**: `contracts/showcase/src/contract.rs` and its tests. The contract source is the specification.
- **Project standards**: `docs/requirements.md`
- **Release tracks**: `docs/roadmap-core.md` for this repo, `docs/roadmap-product.md` for cross-repo context
- **Session onboarding**: `.agent/context.md`
- **Build sequence**: `docs/planning/system-prompt.md`, which carries the numbered 57-step sequence. This path is maintainer-local and excluded from version control, so it is absent from a fresh clone.

## Where decisions live

`.agent/decisions.md`, an append-only ADR log. Never rewrite an entry. Supersede by appending a new one.

Domain terminology lives in `.agent/glossary.md`.

The `.agent/` folder is created by build steps 9 through 11. Steps 1 through 8 predate it.
