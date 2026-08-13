# Pulsar Stellar — Roadmap

**Repository home**: eventually at `pulsar-stellar/pulsar-app/apps/docs/roadmap.md`
**Owner**: Emedit
**Last updated**: at the start of the project, before any code

---

## How to read this

Two dimensions run through this document.

**Execution**: what gets built and shipped in what order, sprint by sprint, from empty repos to a live Drips Wave submission. This is the near-term view. Weeks and months.

**Direction**: where Pulsar Stellar goes after v0.1 ships. What's deliberately deferred, what's aspirational, what would only make sense if adoption is real. This is the long-term view. Months and quarters.

Both are updated at every phase transition. Sprint completions land as commits to this file with a note in the changelog at the bottom. Directional shifts land as new ADR entries in `.agent/decisions.md` in whichever repo they touch.

## Versioning scheme

Pulsar Stellar uses two independent semver tracks, one per repo, joined at product-level milestones.

- `pulsar-core` releases: `v0.1.0-contracts`, `v0.2.0-contracts`, ...
- `pulsar-app` releases: `v0.1.0-app`, `v0.2.0-app`, ...
- Product-level milestones: `v0.1` (both above land together), `v0.2` (submission-ready), `v1.0` (production-ready), etc.

A product-level tag is announced only when both repo tracks reach compatible states.

## Sprint numbering

Eleven sprints total, from empty repos to sustainable contributor cadence:

- Sprints 1 to 3: `pulsar-core` execution
- Sprints 4 to 8: `pulsar-app` execution
- Sprint 9: Drips Wave repo hygiene
- Sprint 10: submission ceremony
- Sprint 11 onwards: contributor onboarding cadence

---

## Now

Two system prompts are drafted (Phase 6 and Phase 7 of the Stellar Wave Builder playbook). No code has been written. Two empty repos need to be created under the `pulsar-stellar` GitHub org. Then execution starts with `pulsar-core`.

Immediate next actions, in order:

1. Register `pulsar-stellar.dev` domain (or `.xyz` if `.dev` is taken)
2. Create `pulsar-stellar` GitHub org
3. Create empty repos `pulsar-stellar/pulsar-core` and `pulsar-stellar/pulsar-app`
4. Create the GitBook space and connect GitHub Sync (paused until `apps/docs/` exists)
5. Open Claude Code in the local clone of `pulsar-core` and hand it `pulsar-core-system-prompt.md`
6. Execute Sprint 1

---

## Time estimates and pace

Every sprint below carries a scope-based estimate in focused-work hours. Translate to calendar time based on your real cadence:

- At **4 hours per day, 5 days per week** (20 hours per week): the full path from empty repos to submission takes roughly **7 to 9 weeks**
- At **6 hours per day, 5 days per week** (30 hours per week): roughly **5 to 6 weeks**
- At **weekend-only, ~10 hours per week**: roughly **14 to 18 weeks**

These are honest ranges. Toolchain friction (Phase 8 in the playbook) is not padding, it's real and hits every stack transition. Do not compress estimates below the low end without evidence.

---

## Execution roadmap: pulsar-core

Three sprints. Same system prompt (`pulsar-core-system-prompt.md`) drives all three; the split is where you pause, verify, and take breath.

### Sprint 1: Scaffold, context files, contract skeleton

**Build sequence**: Phase 6, steps 1 through 28
**Estimate**: 7 to 11 focused hours

**What ships in this sprint**:

- Steps 1-12 (Phase A): Cargo workspace, `rust-toolchain.toml` pinned to 1.84 with `wasm32v1-none` target, `.gitignore`, LICENSE, SECURITY.md, CONTRIBUTING.md, `CLAUDE.md`, `.agent/context.md`, `.agent/decisions.md` seeded with ADR-001 through ADR-005, `.agent/glossary.md`, GitHub Actions CI workflow. Repo has a public identity and a green empty-scaffold build.
- Steps 13-14 (start of Phase B): showcase Cargo.toml pinning soroban-sdk 26.1.0, `error.rs` with `contracterror` enum.
- Steps 15-20: `storage.rs` with `DataKey` enum, TTL constants, `get_admin`, `set_admin`, `get_balance`, `set_balance` helpers with persistent TTL extension.
- Steps 21-26: `events.rs` with one helper per event (`emit_initialize`, `emit_deposit`, `emit_withdraw`, `emit_transfer`, `emit_admin_change`, `emit_custom`).
- Steps 27-28: `contract.rs` stub with `#[contract]` struct and empty impl block, `lib.rs` re-exporting the module tree.

**Milestones inside the sprint**:

- End of Phase A (step 12): CI green on scaffold, no code yet, agent context files complete.
- End of storage helpers (step 20): `cargo check --package pulsar-showcase` clean.
- End of events helpers (step 26): every event shape in the coverage matrix has a dedicated helper.
- End of skeleton (step 28): `cargo build --package pulsar-showcase` produces an empty-but-valid contract binary.

**Exit criteria**:

- Skeleton compiles cleanly.
- Every event helper's signature matches Phase 5's coverage matrix.
- CI green.
- No public functions on the contract yet. That's Sprint 2.

**Common friction points to expect**:

- Rust 1.84 not installed. Fix via `rustup update` or `rustup install 1.84`.
- `wasm32v1-none` target missing. Fix via `rustup target add wasm32v1-none`.
- `stellar-cli` not installed or wrong version. Install via `cargo install --locked stellar-cli` and verify with `stellar --version`.
- `soroban-sdk` version resolution conflict if a transitive dep pulls a different major. Fix by pinning explicitly in `Cargo.toml`.

Do not iterate version by version. Halt on the actual error, fix at the root.

### Sprint 2: Public functions with test-first discipline

**Build sequence**: Phase 6, steps 29 through 50
**Estimate**: 6 to 10 focused hours

**What ships in this sprint**:

Every public function on `pulsar-showcase`, implemented test-first per the `tdd-workflow` skill. The order below reflects the build sequence exactly.

- Steps 29-31: `initialize`. Test-first, then implementation, then double-init rejection test.
- Steps 32-38: `deposit` and `withdraw`. Interleaved: happy-path tests, implementation, failure-path tests for invalid amounts, insufficient balance, not-initialized.
- Steps 39-42: `transfer`. SEP-41 conformant. Happy-path test asserting three-topic event, implementation, insufficient balance test, unauthorized from-party test.
- Steps 43-45: `set_admin`. Happy-path test, implementation with current-admin auth read pattern, unauthorized caller test.
- Steps 46-47: `emit_custom`. Happy-path test asserting dynamic Symbol topic and Bytes payload, implementation.
- Steps 48-49: `balance` and `admin` read views.
- Step 50: read-view assertions added to existing test files to prove state transitions across all tests.

**Milestones inside the sprint**:

- End of initialize (step 31): first public function landed, storage read/write path proven end to end.
- End of transfer (step 42): SEP-41 conformance verified, the highest-complexity event shape works.
- End of admin functions (step 47): governance surface complete.
- End of read views (step 50): every test asserts both event emission AND state transition.

**Exit criteria**:

- Every public function has at least one happy-path test AND one failure-path test.
- Every variant of the `Error` enum is triggered by at least one test.
- Every event emission is asserted with exact topic and data shapes using `env.events().all()`.
- `cargo test --package pulsar-showcase` green.
- No `unwrap`, `expect`, or `panic!` anywhere outside test code.
- CI green.

**Common friction points**:

- `soroban_sdk::testutils::Events` import missing when asserting events. Fix by importing per Phase 6 Section 6.7.
- `Address::generate(&env)` behaving unexpectedly if `env.mock_all_auths()` not called before the invocation. Fix by mocking auth per test.
- Persistent storage TTL extension calls counted as read operations, adding noise to event assertions. Ignore reads in event count assertions; only contract-published events count.

### Sprint 3: Decoder placeholder, deploy, release

**Build sequence**: Phase 6, steps 51 through 57
**Estimate**: 2 to 4 focused hours

**What ships in this sprint**:

- Steps 51-52 (Phase D): `crates/pulsar-decoder/Cargo.toml` and `src/lib.rs` as an empty lib with a module comment pointing at Phase 7. This is a placeholder; the decoder crate's real content lives in a future sprint after `pulsar-app` proves what the SDK actually needs.
- Steps 53-54 (Phase E): `scripts/build.sh` invoking `stellar contract build`, and `scripts/deploy-testnet.sh` running sequential `stellar contract deploy` + `initialize` + a set of sample invocations that exercise every public function.
- Step 55: README quickstart section pointing at the scripts.
- Step 56 (Phase F): final CI verification on HEAD.
- Step 57: `v0.1.0-contracts` tag with deployed testnet contract ID in the tag body.

**Milestones inside the sprint**:

- After step 52: workspace has both `pulsar-showcase` and `pulsar-decoder` crates, both compile.
- After step 54: `./scripts/deploy-testnet.sh` produces a deployed contract on testnet with a captured contract ID.
- After step 57: v0.1.0-contracts is live, tagged, and ready to be referenced from `pulsar-app`.

**Exit criteria for v0.1.0-contracts release**:

- The showcase contract is invocable on testnet via `stellar contract invoke`. Every function returns success or its documented error.
- `stellar events --network testnet --start-ledger <ledger> --id <contract-id>` returns decoded event samples from the sample invocations.
- The deployed contract ID is captured in the release notes and in `.agent/context.md`.
- The contract ID is ready to hand to `pulsar-app` for `.env.example`.
- CI green on the tag commit.

**Common friction points**:

- Deployer key not funded on testnet. Fix via `stellar keys fund <name> --network testnet`.
- Contract deploy succeeds but `initialize` fails because auth wasn't attached. Fix by using the correct signer profile with `--source <name>`.
- Sample invocations succeed but no events show up in `stellar events`. Fix by verifying you passed `--start-ledger` at or before the deploy ledger.
- Testnet RPC endpoint rate-limited. Fix by switching to a different public RPC or waiting for the window to reset.

---

## Execution roadmap: pulsar-app

Five sprints, same system prompt (`pulsar-app-system-prompt.md`).

### Sprint 4: Monorepo scaffold and TypeScript SDK

**Build sequence**: Phase 7, steps 1 through 38
**Estimate**: 20 to 30 focused hours

**Milestones inside the sprint**:

- Steps 1-15: monorepo scaffolds, workspace works, CI workflows exist. Agent context files match `pulsar-core` discipline.
- Steps 16-19: SDK skeleton with types and Zod schemas passes tests.
- Steps 20-31: `PulsarClient` implements every method against mocked HTTP.
- Steps 32-35: direct RPC path via `@stellar/stellar-sdk` works against testnet.
- Steps 36-38: SDK builds cleanly (ESM + CJS), README written, ready to publish.

**Exit criteria**:

- `pnpm --filter @pulsar-stellar/sdk test` green.
- `pnpm --filter @pulsar-stellar/sdk build` produces valid dual-format package.
- SDK not yet published to npm (publish happens in Sprint 8 with the app release).

### Sprint 5: Go indexer

**Build sequence**: Phase 7, steps 39 through 73
**Estimate**: 25 to 35 focused hours

**Milestones inside the sprint**:

- Steps 39-49: Go module, config, logging, DB drivers, and migrations working.
- Steps 50-58: models, store, and decoder implemented and tested against `pulsar-showcase` fixtures.
- Steps 59-71: RPC polling loop, HTTP handlers, and GraphQL surface work end to end.
- Steps 72-73: `pulsar-indexer` binary starts, polls testnet, serves decoded events over HTTP.

**Exit criteria**:

- `go test ./...` green in `indexer/`.
- Running `pulsar-indexer` against testnet with the showcase contract registered indexes events within 60 seconds of emission.
- Both SQLite and Postgres drivers pass the same test suite.
- HTTP handlers return decoded events in the shape the SDK expects.

### Sprint 6: Next.js explorer

**Build sequence**: Phase 7, steps 74 through 95
**Estimate**: 20 to 30 focused hours

**Milestones inside the sprint**:

- Steps 74-84: monorepo web app scaffolds, design tokens locked, landing page works.
- Steps 85-92: event stream page, filters, table, detail panel, export JSON.
- Steps 93-95: single-event deep link, health endpoint, loading states.

**Exit criteria**:

- Paste the showcase contract ID on the landing page, navigate to `/c/[contractId]`, see decoded events from the local indexer.
- Filter by event name, ledger range, and topic-value substring. URL reflects filter state.
- Export JSON downloads a valid file.
- Single-event deep links resolve correctly.
- Accessibility check passes on every route.

### Sprint 7: GitBook documentation

**Build sequence**: Phase 7, steps 96 through 111
**Estimate**: 15 to 20 focused hours

**Milestones inside the sprint**:

- Steps 96-98: `apps/docs/` structure, `SUMMARY.md`, `.gitbook.yaml` land. GitHub Sync activates.
- Steps 99-111: every section in `SUMMARY.md` has real content matching the discipline rules (no em dashes, no filler, real numbers).

**Exit criteria**:

- GitBook space is live and reachable at a public URL.
- Every page renders correctly.
- Every code block is runnable as-is (verify with a copy-paste smoke pass).
- Cross-references work.

### Sprint 8: Deploy, publish, release

**Build sequence**: Phase 7, steps 112 through 121
**Estimate**: 10 to 15 focused hours

**Milestones inside the sprint**:

- Steps 112-115: local dev scripts work smoothly (`./scripts/setup.sh`, `./scripts/dev.sh`).
- Steps 116-118: Docker image builds, `docker-compose up` runs indexer + Postgres locally.
- Step 119: Vercel and Render projects created, environment variables set.
- Step 120: all CI workflows green.
- Step 121: `v0.1.0-app` tag pushed with production URLs in the tag body.

**Exit criteria for v0.1 product-level milestone**:

- SDK published to npm as `@pulsar-stellar/sdk` version 0.1.0.
- Indexer running at `https://indexer.pulsar-stellar.dev` (or equivalent Render URL) polling testnet, showcase contract indexed.
- Web explorer live at `https://pulsar-stellar.dev` (or equivalent Vercel URL).
- Docs live on GitBook.
- Both repos tagged: `pulsar-core@v0.1.0-contracts` and `pulsar-app@v0.1.0-app`.
- The path from "paste the showcase contract ID on the web explorer" to "see decoded events" works with zero manual setup for a first-time visitor.

---

## Post-execution roadmap: submission and beyond

### Sprint 9: Repo hygiene for Drips Wave approval

**Playbook phase**: Phase 10
**Estimate**: 5 to 8 focused hours per repo, so 10 to 16 total

Everything below matches the pattern of already-approved Drips Wave Stellar repos (verify by fetching a few current top-approved repos before assuming any pattern).

Per repo:

- Branch protection on `main`: PRs required, at least one approval, CI status checks required, matching the actual job names from the workflows
- `CONTRIBUTING.md` with commit rules, local setup, submission process, code of conduct link
- `SECURITY.md` with responsible disclosure contact, in-scope surfaces, unaudited disclaimer
- README rewrite matching the approved-repo pattern (banner or logo, badges for CI + license + npm + crates, maintainer table with contact info, community link, concise architecture, quick-start commands, contributing section, contributors credits with `contrib.rocks`)
- GitHub topics added for discoverability (`stellar`, `soroban`, `smart-contracts`, `blockchain`, `dev-tools`, `event-indexer`, `rust`, `typescript`, `golang`)
- Issue templates for bug reports and feature requests
- PR template referring to the commit discipline

For `pulsar-app` specifically, add:

- A `gh` CLI script (`scripts/create-wave-issues.sh`) that batch-creates every planned issue in one run. Each issue has: commit-style title, complexity label (`100`, `150`, or `200`), type label (`feat`, `fix`, `docs`, `test`, `refactor`), body with Summary, Acceptance Criteria as checkboxes, and Tech Stack section.

**Planned issue areas for the Wave submission** (grouped so contributors find scoped work):

- SDK (10-15 issues): add contract client generation wrapper, add cursor iterator helper, add SSE subscription helper for live events, add typed event codegen from indexer schema, add examples app
- Indexer (10-15 issues): add rate limiting middleware, add Prometheus metrics endpoint, add contract spec fetch and cache, add pagination cursor documentation, add graceful RPC reconnect, add multi-contract batch polling optimization
- Web (10-15 issues): add contract search history, add topic-value autocomplete, add copy-XDR button on detail panel, add mobile-responsive filter drawer, add dark mode toggle, add per-event permalink share preview
- Docs (5-10 issues): add "your first indexer" tutorial, add "consuming Blend events" case study, add architecture decision record explainer, add self-hosting guide

For `pulsar-core` specifically, add:

- SDK-side issues for the decoder crate that was placeholdered in Sprint 3 (10-15 issues): add SCSpecEntry parser, add typed event struct codegen macro, add native decoder for each ScVal variant, add wasm-pack build target, add crates.io publish workflow
- Contract-side issues (5-10 issues): add fuzz tests for the showcase contract, add cross-version compatibility test for soroban-sdk upgrades, add benchmark suite

**Exit criteria for Sprint 9**:

- Every repo passes the hygiene checklist above.
- Between 40 and 65 well-scoped issues exist across the two repos, labeled with point values.
- Both repos look like they belong to a real project someone else could contribute to today.

### Sprint 10: Drips Wave submission ceremony

**Playbook phase**: Phase 12
**Estimate**: 5 to 8 focused hours

Assemble everything before opening the submission form:

- Live app URL (Vercel)
- Live docs URL (GitBook)
- Both repo URLs
- Contract verification link on Stellar Expert for the deployed showcase contract
- Demo video (60 to 120 seconds) showing the full flow: paste contract ID, see events, expand one, filter, export JSON. Recorded with real narration, not voiceover-generated.
- Repo relationship description (one paragraph explaining how `pulsar-core` and `pulsar-app` connect)
- Planned issues description (organized by area, referring to real issues created in Sprint 9)
- Project description for the submission form (plain English, one paragraph, states problem + mechanism + who benefits + real-world scale if a credible figure exists)

Submit. Wait for review. If accepted, move to Sprint 11. If revisions requested, address them per Phase 13 rules (scope honestly, cross-reference issues across repos when they span both, never build fixes without confirming which architecture depends on what).

### Sprint 11 onwards: Contributor onboarding and iteration

**Playbook phase**: Phase 13
**Cadence**: weekly

Once approved:

- Weekly issue triage cadence (30 to 60 minutes): pick from the queue, unblock stuck contributors, close stale
- New contributor PR review: same-day acknowledge, 48-hour review turnaround
- Every merged PR earns a `contributors.md` credit and a Discord shoutout (or wherever the community lives)
- Any new feature request goes through the ADR process: scope it, options if design decision, acceptance criteria, tech stack. Never build without confirming which repo and which existing architecture depends on it first.

---

## Future direction

This section is deliberate about what's deferred and why. Every entry has a trigger condition explaining what would move it into an active sprint.

### Deferred from v0.1 (nearest horizon)

**Rust decoder crate (real content)**
Sprint 3 shipped an empty placeholder for `pulsar-decoder` in `pulsar-core`. The real content (SCSpecEntry parser, typed event struct codegen macro, XDR-to-native decoder) comes after `pulsar-app` proves what shape the SDK actually needs. This avoids designing the Rust API in isolation and then having to rework it once TypeScript consumers hit its edges.
Trigger: `v0.1.0-app` shipped and stable; user feedback points at specific consumption patterns to lock into the Rust API.

**Wasm decoder in browser**
Ship the Rust decoder crate as `@pulsar-stellar/decoder-wasm` compiled via `wasm-pack`, so the SDK can decode events locally without an indexer call. Useful for consumers who only need last-24-hour events and want to skip the indexer dependency.
Trigger: decoder crate content lands (per entry above); at least one user request.

**Diagnostic events**
Extend the indexer and SDK to handle diagnostic events (simulation-only, not persisted). This overlaps with the transaction-simulator lane where `erst` already lives. Only worth building if there's a clear per-contract-inspection use case that the transaction simulator doesn't serve.
Trigger: at least three distinct users request it, or an integration with `erst` is proposed.

**Typed contract event codegen**
Generate typed TypeScript event definitions from a contract's `SCSpecEntry` metadata (once `rs-soroban-sdk#1097` and `js-stellar-sdk#1257` land). Consumers get compile-time safety on event shapes.
Trigger: those two upstream issues merge. Until then, hand-rolled schemas are the workaround.

### Medium horizon

**Cross-contract search**
Search events across a set of contracts in one query. Useful for wallet integrations tracking user activity across a protocol suite (Blend + Stellar Aquarius + SoroSwap, for example).
Trigger: v0.2 submitted successfully, first non-Emedit consumer requests it.

**Webhooks and SSE**
Push decoded events to consumers instead of polling. SSE for browser SDK subscribers, webhook POSTs for backend consumers.
Trigger: at least one production consumer builds a workaround using polling that the primitives should handle natively.

**Historical replay from archive nodes**
Index events older than the 7-day RPC retention window by parsing `stellar-etl` output or replaying from archive nodes. Extends usefulness to auditors reviewing pre-existing contract history.
Trigger: someone with archive-node infrastructure offers a partnership, or the effort scope becomes clearer after v0.2 is in production.

### Far horizon

**Hosted service tier**
Optional hosted indexer for people who don't want to self-host. User accounts, saved queries, API keys, rate limiting, per-plan retention, dashboards. Free tier for small usage, paid tier for teams. Business model that could sustain a small team.
Trigger: at least six months of adoption data, meaningful npm/crates downloads, real requests for a hosted version. Do not chase this without proof.

**gRPC surface**
High-throughput consumers (analytics platforms, chain-explorers building on top) get a gRPC interface with streaming semantics.
Trigger: analytics-scale consumer approaches the project.

**Kafka/NATS sink**
Emit decoded events to a message bus for downstream systems.
Trigger: enterprise consumer request.

**Integration with Mercury or SubQuery**
If someone wants to combine our per-contract typed decoding with their broad indexing infrastructure, ship an integration.
Trigger: partnership conversation.

### Research spikes

Not on the roadmap, but worth tracking:

- FFI between Go indexer and Rust decoder crate (avoid duplicating decode logic). Currently the Go decoder reimplements what the Rust decoder does; keeping them in sync via fixtures is fine but fragile.
- Wasm-in-Deno-and-Bun runtime tests for the SDK's browser decoder path.
- Formal verification of the reference contract (probably not; it's a fixture, not a protocol).

---

## Success signals per milestone

These are targets, not commitments. Adjust based on real feedback after v0.1.

**v0.1 (both repos shipped)**
- Both repos publicly reachable, docs live, submission draft ready
- At least one non-Emedit developer clones the repo and gets it running end to end
- Zero critical bugs in the reference contract on testnet

**v0.2 (Drips Wave approved)**
- Submission accepted
- At least five external issues picked up by contributors within four weeks
- npm downloads for `@pulsar-stellar/sdk` show organic growth (target: 50 to 100 in month one)
- GitHub stars: target 30 to 50 in month one

**v1.0 (production-ready)**
- At least three non-Emedit projects use the SDK in production
- Indexer tested against mainnet events at production scale (measured: at least 1000 events per hour sustained without decode failures)
- Decoder crate published to crates.io with basic download velocity
- Security review completed on the reference contract and the decoder crate (either community audit or self-review with a documented threat model)
- SDF grant application eligible and drafted (whether or not submitted)

**Post-v1.0**
- Adoption drives priorities, not roadmap ambition
- Any hosted-service or paid-tier decision is data-driven, not speculative

---

## Risk register

Every risk here has a mitigation. If a risk fires without a mitigation in place, it becomes an emergency triage session, not a project-ending event.

**soroban-sdk breaking changes between v26 and v27**
- Mitigation: pinned version in `Cargo.toml` and `rust-toolchain.toml`. Integration test suite run against each new soroban-sdk release before upgrading. ADR entry for any upgrade explaining what changed.

**Stellar RPC public endpoint rate limits or downtime**
- Mitigation: indexer backoff + retry logic. Documentation for self-hosted RPC alternatives. `PULSAR_INDEXER_RPC_URL` env var makes it trivial to point at a different endpoint.

**Mercury, SubQuery, or Goldsky expanding into per-contract dev tooling**
- Mitigation: our differentiation is Rust-first + typed + self-hostable + open source. If an incumbent copies the concept, we still win on developer trust and code control. Do not compete on hosted-infra reliability, we won't win there.

**Solo maintainer burnout**
- Mitigation: Drips Wave contributors reduce the load. Weekly triage boundary (30-60 minutes max). Every ADR captures reasoning so future-you (or a co-maintainer) can pick up cleanly. Say no to scope creep.

**Testnet contract or deployment fails audit before Wave approval**
- Mitigation: reference contract is small, well-tested, and clearly marked "not for production." SECURITY.md is explicit about unaudited status.

**GitBook pricing or policy changes**
- Mitigation: markdown source lives in the repo. If GitBook becomes unusable, migrate to Docusaurus or Mintlify in a week. Docs content is portable.

**Vercel or Render pricing or policy changes**
- Mitigation: deploy config is straightforward. Both platforms have direct competitors (Netlify for web, Fly.io for backend + managed Postgres). Migration cost is a weekend.

**Drips Wave submission rejected**
- Mitigation: submission is a resubmittable process, not a one-shot. Read the rejection reasons, address them under Phase 13 rules, resubmit in the next wave.

---

## Changelog

Every meaningful update to this document lands here as a one-liner with the date and author.

- **YYYY-MM-DD**: Initial roadmap drafted, before any code.
- **YYYY-MM-DD**: Split pulsar-core execution into three sprints (Sprints 1-3) matching the natural break points in Phase 6's build sequence. Renumbered pulsar-app sprints from 2-6 to 4-8, and downstream sprints accordingly.

Update this changelog whenever a sprint completes, a milestone shifts, or a directional decision changes.

---

**End of roadmap document.**