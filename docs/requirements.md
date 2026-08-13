# Pulsar Stellar — Project Requirements Document

**Purpose**: single authoritative reference for external tools, test discipline, context folders, dependency management, and authenticity principles across both `pulsar-core` and `pulsar-app`.

**Relationship to other documents**: this supplements the system prompts (`pulsar-core-system-prompt.md`, `pulsar-app-system-prompt.md`) and roadmaps (`pulsar-stellar-roadmap.md`, `pulsar-core-roadmap.md`). Where any rule here is stricter than the system prompts, this document wins. Where the system prompts are stricter, the system prompts win. Never the more lenient one.

**Audience**: every AI agent (Claude in chat, Claude Code in repo, any future coding agent) that touches this project. Feed this document alongside the two system prompts and two roadmaps at the start of every working session.

**Owner**: Emedit

---

## 1. External tools and services

The project depends on the following. Each entry includes: what it does, why we use it, setup responsibility, and where credentials live.

### 1.1 Version control and collaboration

**GitHub organization: `pulsar-stellar`**
- Contains both repos: `pulsar-core` and `pulsar-app`
- GitHub Issues for tracking work
- GitHub Projects (optional) for sprint kanban
- GitHub Actions for CI on both repos
- GitHub Releases for versioned artifacts with checksums
- Branch protection on `main`: PRs required, one approval minimum, CI status checks required, no force-push
- Setup: one-time. Emedit creates the org, invites collaborators as they join.
- Credentials: SSH keys per contributor, personal access tokens for local `gh` CLI.

### 1.2 Package publishing

**crates.io** (Rust)
- Publishes: `pulsar-decoder` (v0.2+), `pulsar-events` (v0.5+)
- Credentials: `CARGO_REGISTRY_TOKEN` in GitHub Actions secrets for `pulsar-core` repo
- Setup: create crates.io account, generate publish token, add to repo secrets before Sprint 2

**npm** (TypeScript + wasm)
- Publishes: `@pulsar-stellar/sdk` (v0.1+), `@pulsar-stellar/decoder-wasm` (v0.4+)
- Credentials: `NPM_TOKEN` (automation token, not classic) in GitHub Actions secrets for `pulsar-app` repo (and `pulsar-core` for the wasm package)
- Setup: create npm account with 2FA, create `@pulsar-stellar` scope, generate automation token before Sprint 6

**Docker Hub** (optional, only if we push indexer images publicly)
- Publishes: `pulsar-stellar/indexer` image
- Alternative: GitHub Container Registry (`ghcr.io/pulsar-stellar/indexer`) if we want to keep everything in one place
- Recommendation: use `ghcr.io`. No separate account needed.

### 1.3 Deployment

**Vercel** (Next.js web explorer)
- Hosts: `apps/web` from `pulsar-app`
- Free tier is sufficient for the traffic level this project will see through v1.0
- Setup: connect GitHub, set project root to `apps/web`, add environment variables via UI
- Credentials: none in repo. All env vars in Vercel dashboard.

**Render** (Go indexer + Postgres)
- Hosts: indexer as a web service, Postgres as a managed database
- Uses internal connection string when both are in the same region
- Setup: connect GitHub, create service via `render.yaml`, provision Postgres in same region
- Credentials: none in repo. All env vars in Render dashboard. `DATABASE_URL` auto-provisioned by Render.

Both are configured in `pulsar-app/deploy/`. Do not migrate everything to one platform to save on cognitive overhead; the split platforms are purpose-built and cheaper this way.

### 1.4 Documentation

**GitBook**
- Renders docs from `pulsar-app/apps/docs/` via GitHub Sync
- Free tier acceptable for open-source projects (verify current terms)
- Setup: create space, connect GitHub integration, point at `main` branch's `apps/docs/` folder
- Credentials: none. GitBook has read access to the repo folder via GitHub App.

### 1.5 Domain and DNS

**Registrar**: any (Namecheap, Cloudflare Registrar, Porkbun)
- Primary domain: `pulsar-stellar.dev` (falls back to `.xyz` if `.dev` is unavailable)
- DNS: managed at the registrar or fronted by Cloudflare
- Records needed: apex + `www` for the web explorer (Vercel provides values), `indexer.pulsar-stellar.dev` (Render provides value), `docs.pulsar-stellar.dev` (GitBook provides value)

### 1.6 Blockchain infrastructure

**Stellar Testnet**
- Used for: contract deployment during v0.1 through v0.4, all CI and development
- RPC endpoint: `https://soroban-testnet.stellar.org`
- Fallback RPC providers: NowNodes, GetBlock, RPC providers with Stellar support
- Credentials: none for public RPC. Test accounts funded via `stellar keys fund`.

**Stellar Mainnet**
- Used for: production deployment at v1.0
- RPC endpoint: `https://mainnet.sorobanrpc.com` (SDF-run) or third-party
- Credentials: mainnet keys managed via hardware wallet or Freighter. Never in repo. Never in CI.

**Stellar Expert**
- Used for: contract verification links, transaction history, human-readable block explorer
- URL: `https://stellar.expert/explorer/testnet/contract/<id>` and mainnet equivalent
- No account needed for read access.

### 1.7 Community channels

**Telegram** (primary; ecosystem standard for Stellar)
- Setup: create a group named `Pulsar Stellar`; add to README maintainer table
- Credentials: bot token if we add automation later

**Discord** (secondary, optional)
- Only if there's demand. Do not create prematurely.

**Twitter or X** (optional)
- Only for announcements. Do not create prematurely.

### 1.8 Local development toolchain

Required on the developer machine before any sprint starts:

| Tool | Version | Install command | Purpose |
|---|---|---|---|
| Node.js | 20 LTS | `nvm install 20` | TS runtime |
| pnpm | 9+ | `corepack enable && corepack prepare pnpm@latest --activate` | JS package manager |
| Rust (project toolchain) | 1.84.0 exactly | pinned in `rust-toolchain.toml`, installed by rustup on the first cargo command | Contract + decoder builds, all in-project cargo commands, CI |
| Rust (host toolchain) | stable, 1.93+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` | Installing host binary tools only, never project builds |
| Go | 1.23+ | Platform-specific (see go.dev/dl) | Indexer |
| Stellar CLI | 27.1.0+ | `cd ~ && rustup run stable cargo install --locked --force stellar-cli` | Contract build + deploy |
| Docker | 24+ | Platform-specific | Local Postgres, local indexer |
| Docker Compose | v2+ | Bundled with modern Docker | Local dev stack |
| Git | 2.40+ | Platform-specific | Version control |
| `gh` CLI | Latest | Platform-specific | Issue creation script |

Two Rust rows, not one. stellar-cli 27.1.0 requires rustc 1.93.0 or newer to build, while the contract is pinned to 1.84.0 because that is the earliest stable Rust providing the `wasm32v1-none` target that soroban-sdk 26.1.0 needs. Running `cargo install --locked stellar-cli` from inside the project directory picks up the pinned 1.84 toolchain and fails. The `cd ~` moves out of the `rust-toolchain.toml` override and `rustup run stable` selects the host channel explicitly. See ADR-007 in `pulsar-core/.agent/decisions.md`.

Verification script at `pulsar-app/scripts/verify-toolchain.sh` runs `<tool> --version` for each and asserts minimum version. Runs at scaffold time and before every major sprint.

### 1.9 CI/CD

**GitHub Actions**
- Workflows in `.github/workflows/` in both repos
- `pulsar-core`: single workflow (`ci.yml`) covering fmt + clippy + build + test + coverage
- `pulsar-app`: three workflows (`ci-ts.yml`, `ci-go.yml`, `ci-web.yml`) matching the three sub-stacks
- Secrets in repo settings, referenced as `${{ secrets.<NAME> }}`
- No third-party CI required (no CircleCI, no Travis, no Jenkins)

### 1.10 Optional tooling worth considering

Not adopted by default; each requires an ADR to bring in.

- **Codecov or Coveralls**: coverage badge + PR comments. Free for open source. ADR triggered if we want visible coverage tracking.
- **Sentry**: error tracking for the web explorer. ADR triggered if we hit production error volume.
- **Plausible or Fathom**: privacy-respecting analytics. ADR triggered if we want usage metrics.
- **Snyk or Dependabot**: automated dependency PRs. Dependabot is free and GitHub-native, likely to enable in Sprint 7.
- **Renovate**: alternative to Dependabot with finer control. Consider if Dependabot proves too noisy.

---

## 2. Test discipline (enforced across both repos)

Non-negotiable. Enforced in CI. Blocking on PR merge.

### 2.1 Test-case-per-implementation rule

Every code commit satisfies one of these three:

1. It is a test commit (adds or modifies test files only)
2. It adds or modifies implementation code AND a preceding commit contains the failing test that this implementation makes pass
3. It is a scaffolding commit (Cargo.toml edits, tsconfig.json, folder structure, no logic)

No merged PR contains code changes without corresponding test changes. Enforced by PR template checklist and CI diff-check script (`scripts/verify-test-parity.sh`).

Rare exceptions require an ADR entry justifying why the code cannot be tested (example: build script that only runs during publish).

### 2.2 Coverage minimums

Enforced in CI. PR blocking if coverage drops below floor.

| Sub-stack | Floor | Measurement tool |
|---|---|---|
| Rust (contracts + decoder) | 85% line coverage | `cargo-llvm-cov` |
| Go (indexer) | 80% statement coverage | `go test -coverprofile` |
| TypeScript SDK | 80% line coverage | `vitest --coverage` (c8) |
| Next.js critical components | 60% line coverage | `vitest --coverage` |

Critical components in the frontend are the ones users depend on for the core flow: `event-table`, `event-row`, `event-detail`, `filter-bar`, `contract-input`, `export-json`. Presentational components (Skeleton wrappers, layout shells) are exempt.

Coverage measured on `main` merges and reported as a PR comment. Coverage badges in READMEs.

### 2.3 Test types per layer

**Rust contracts (`pulsar-showcase`, future showcase contracts)**:
- Unit tests: every public function has happy-path AND at least one failure-path test
- Integration tests: full contract deployment + invocation using `soroban-sdk::testutils::Env`
- Event assertion: every event emission asserted with exact topic and data shapes using `env.events().all()`
- Auth assertion: every state-changing function has a test that fails when `require_auth` is not mocked

**Rust decoder (`pulsar-decoder`)**:
- Unit tests: every public function
- Property tests via `proptest`: XDR round-tripping (encode + decode + assert equality)
- Fuzz tests via `cargo-fuzz`: `decode_event` runs for at least 30 minutes cumulatively before v0.2 release, extended to 100 hours before v1.0
- Regression corpus: every bug fixed adds a fixture that reproduces it

**Go indexer**:
- Unit tests: every internal package has `_test.go` files
- Table-driven tests where inputs vary
- Integration tests: full HTTP stack against in-memory SQLite + mocked RPC using `testify` + `httptest`
- No live RPC calls in unit or integration tests

**TypeScript SDK**:
- Unit tests: every public method
- Contract tests: Zod schemas accept every valid shape and reject every invalid one (parametric)
- Integration tests: full SDK client against `msw`-mocked indexer
- No live network calls in unit or integration tests

**Next.js frontend**:
- Component tests via `vitest` + `@testing-library/react` for critical components
- Route-level tests for filter state, URL sync, cursor pagination
- E2E test via Playwright: paste showcase contract ID → see events → filter → expand row → export JSON. Runs locally against docker-compose stack; skipped in CI without env flag.

### 2.4 Test naming conventions

- **Rust**: `fn <function>_<scenario>()` inside `#[cfg(test)] mod tests`. Example: `deposit_rejects_negative_amount`. Integration test file names describe the surface: `initialize.rs`, `transfer.rs`.
- **Go**: `Test<Function>_<Scenario>(t *testing.T)`. Example: `TestPollLoop_BackoffOnRPCError`. Table-driven tests use `t.Run(name, ...)` for scenarios.
- **TypeScript**: `it('should <expected behavior> when <condition>', () => {})`. Grouped with `describe('<subject>', () => {})`.

### 2.5 Test data management (fixtures)

Fixtures represent real, decoded events from `pulsar-showcase` (and other showcase contracts as they're added).

- Location: `tests/fixtures/` (Rust), `testdata/fixtures/` (Go), `__fixtures__/` (TS)
- Contents: raw XDR (base64), expected decoded output (JSON), source metadata (ledger number, tx hash, source contract ID)
- Fixtures checked into the repo. Never fetched from testnet at test time.
- Regenerating a fixture requires an ADR entry explaining what changed and why
- Cross-repo fixtures shared via `pulsar-core`'s `tests/fixtures/` folder, mirrored into `pulsar-app/indexer/testdata/fixtures/` at scaffold time and updated only when `pulsar-core` releases a new set

### 2.6 Test file colocation

- **Rust**: `#[cfg(test)] mod tests` at the bottom of each source file for unit tests. `tests/` folder at the crate root for integration tests.
- **Go**: `_test.go` in the same package as the source.
- **TypeScript**: `<name>.test.ts` colocated with `<name>.ts` in the same folder. Exception: E2E tests in `tests/e2e/`.

### 2.7 CI enforcement

Every PR runs the full test suite. No skipped tests without justification in the PR body. Coverage report as PR comment. Test failures block merge (protected by branch rules).

CI matrix:

- Rust: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo-llvm-cov` coverage
- Go: `gofmt -l -d`, `go vet`, `staticcheck`, `go test -race -coverprofile=coverage.out`
- TypeScript: `pnpm lint`, `pnpm typecheck`, `pnpm test --coverage`, `pnpm build`

---

## 3. Context folder full specification

Every repo has a `.agent/` folder. These files are load-bearing for continuity across AI sessions and human onboarding. Kept current at every phase transition.

### 3.1 `.agent/context.md`

Full onboarding for a fresh session. Written once at scaffold, updated at phase transitions only.

Sections:

- What Pulsar Stellar is (one-paragraph description locked earlier)
- Why this repo exists
- Relationship to the sibling repo
- Current phase, current release, definition of done for the current phase
- Where the authoritative spec lives
- How to run tests, build, deploy locally (with exact commands)
- Drips Wave context

### 3.2 `.agent/decisions.md`

Append-only ADR log. Format:

```
## ADR-NNN: <title>
Date: YYYY-MM-DD
Status: accepted | superseded by ADR-MMM

### Context
### Decision
### Alternatives considered
### Consequences
```

Never rewrite an entry. Supersede by appending.

### 3.3 `.agent/glossary.md`

One-paragraph definitions of every domain term the repo uses. Updated when new terms enter the codebase.

### 3.4 `.agent/test-strategy.md` (new)

Repo-specific restatement of Section 2 of this document. Includes:

- Coverage floors specific to this repo
- Test types required per module in this repo
- Fixture management policy
- How to add new fixtures (with the ADR requirement restated)
- How to reproduce a CI test locally
- Which test failures are known-flaky and their tracking issue

### 3.5 `.agent/dependency-audit.md` (new)

Table of every direct dependency:

| Package | Version | Purpose | License | Last audited | Known CVEs |
|---|---|---|---|---|---|

- Updated at every dependency change
- Monthly review cycle documented
- Format machine-readable enough that a script can lint it (`scripts/verify-dependency-audit.sh` in each repo)

Any dependency addition or version bump requires an ADR entry cross-referenced from this table.

### 3.6 `.agent/skills.md` (new)

Documents which locally installed skills apply to this repo and when:

- `humanizer`: every text-output task (docs, commit messages, README updates)
- `frontend-patterns`: every UI-related commit (pulsar-app only)
- `coding-standards`: every code commit
- `tdd-workflow`: every function or handler implementation
- `blueprint`: architecture-level tasks (scaffolding, refactoring)
- `security-review`: every commit touching auth, storage, HTTP boundaries, database queries, or dependency changes

Plus a note that the built-in `frontend-design` skill applies to `apps/web` styling decisions.

### 3.7 `.agent/external-tools.md` (new)

Repo-specific restatement of Section 1 of this document. Only lists the tools actually used by this repo. For `pulsar-core`: GitHub, crates.io, Stellar Testnet, Stellar Expert, Telegram. For `pulsar-app`: everything else.

Includes credential handling rules restated:

- All secrets in Vercel or Render UIs, never in the repo
- `.env.local` for local dev, never committed
- Pre-commit hook (gitleaks or similar) blocks accidental credential commits
- CI has GitHub Actions secrets scoped to the least-privilege token

---

## 4. Dependency management

### 4.1 Direct dependency policy

Every direct dependency:

- Appears in `.agent/dependency-audit.md` with rationale
- Has a version pin appropriate to its sensitivity:
  - Rust security-sensitive (`stellar/*`, `soroban-*`): exact `=X.Y.Z`
  - Rust general: semver `X.Y` or `X.Y.Z`
  - TypeScript: caret `^X.Y.Z` for most; exact for `@stellar/stellar-sdk`
  - Go: semver via `go.mod`, integrity via `go.sum`
- Has a documented purpose in the audit file
- Has a license compatible with our project's license (Apache-2.0)

No unpinned wildcards. Ever.

### 4.2 Transitive dependency audit

CI runs vulnerability scanners on every PR:

- **Rust**: `cargo-audit --deny warnings`, blocks on high or critical CVEs
- **TypeScript**: `pnpm audit --audit-level high`, blocks on high or critical
- **Go**: `govulncheck ./...`, blocks on any known vulnerability affecting our imports

Scanner failures block merge. Bypass requires an ADR entry explaining why the vulnerability does not affect us (usage pattern, version constraint, etc.) with an issue tracked for cleanup.

### 4.3 License compliance

Direct dependencies must be under one of:

- Apache-2.0
- MIT
- ISC
- BSD (2-clause or 3-clause)
- MPL-2.0

Anything else (GPL, AGPL, SSPL, commercial, custom) requires ADR review and explicit approval before adoption.

License scan tools in CI:

- Rust: `cargo-license` runs and asserts against an allowlist
- TypeScript: `license-checker` (or `pnpm license-checker`) runs and asserts against allowlist
- Go: `go-licenses check ./...` with allowlist

### 4.4 Update cadence

- **Weekly**: Dependabot (or Renovate) opens automatic PRs for patch and minor bumps
- **Applied immediately if CI passes**: patch bumps
- **Applied within two weeks with ADR-lite entry**: minor bumps
- **Applied only after ADR review**: major bumps, any dep with breaking changes, any transitive change flagged by audit

Monthly manual review:
- `cargo outdated` in `pulsar-core`
- `pnpm outdated -r` in `pulsar-app`
- `go list -u -m all` in `pulsar-app/indexer`

Log findings in `.agent/dependency-audit.md` update entry with the review date.

---

## 5. Authenticity principles

Reinforcing the playbook's operating principles. These rules apply to every artifact produced for this project.

### 5.1 No placeholder code

- No stubs
- No `TODO` comments in shipped commits (`TODO` acceptable in draft branches before rebase)
- No `unimplemented!()` (Rust)
- No `panic("not implemented")` (Go)
- No `throw new Error("TODO")` (TS)
- Every function commit is complete and testable at the moment it lands

### 5.2 No fabricated numbers

- Every metric in docs comes from measured data (test runs, benchmarks, telemetry)
- Real ecosystem statistics only from primary sources (SDF blog, crates.io, npm registry stats)
- Vague quantifiers ("many", "several", "most") replaced with real numbers or removed
- If a number is a target, label it explicitly as a target

### 5.3 No AI-sounding filler

Banned words and phrases in shipped content (README, docs, blog posts, commit messages):

- "seamlessly"
- "robust" (as adjective)
- "powerful" (as adjective)
- "leverage" (as verb)
- "unlock" (metaphorical)
- "cutting-edge"
- "in the ever-evolving landscape of"
- "revolutionize"
- "delve into"
- "elevate"
- "empower" (except when literal, e.g. permissions)
- Long em dashes anywhere

If a sentence uses any of these, rewrite. Prefer concrete verbs and specific nouns.

### 5.4 No credential leakage

- `.env.example`: placeholder values only. Real keys never committed.
- Secrets in Vercel or Render UI, never in repo
- Pre-commit hook (`gitleaks` recommended) blocks accidental commits of high-entropy strings that look like keys
- If a credential is committed by mistake, rotate immediately, do not just delete the commit

### 5.5 No test theater

Tests must genuinely verify behavior. Banned patterns:

- Tests that call a function and only assert "did not throw"
- Tests that mock the code under test itself
- Tests with `expect(true).toBe(true)` or equivalents
- Tests that pass only because the assertion is trivially true (e.g. asserting a value equals itself)
- Snapshot tests that just capture whatever the code produced without human review

Every test has: a specific behavior it verifies, an assertion that would fail if that behavior broke, and a name that describes what's being verified.

### 5.6 Verifiable claims in docs

- Every "we do X" claim in README or docs corresponds to code that does X
- SDK reference examples pulled from real test files (kept in sync via a script or documented manual step)
- Indexer API examples come from real request-response pairs, not hand-composed JSON
- Contract event tables generated from the real event helpers, not typed by hand
- If a claim can't be verified against code, it's aspirational and labeled as such

### 5.7 Real deployment, not mockups

- Every URL in submission materials points at a real deployed instance
- Demo video shows the real product flow end to end, not a Figma prototype or a talking head
- Contract IDs in docs are real testnet or mainnet deployments, not placeholders
- Screenshots come from the real running app

---

## 6. Guidance for AI agents going forward

When Claude in chat, Claude Code in a repo, or any other coding agent generates future prompts, code, documentation, ADRs, or deliverables for this project, the following are non-negotiable:

1. **Reference this document** at task start alongside the two system prompts and two roadmaps
2. **Cite the six locally installed skills** applicable to the task: `humanizer`, `frontend-patterns`, `coding-standards`, `tdd-workflow`, `blueprint`, `security-review`
3. **Apply the test discipline from Section 2** to every code change
4. **Update the context folder from Section 3** at every phase transition or significant decision
5. **Follow dependency management rules from Section 4** for any package addition or bump
6. **Follow authenticity principles from Section 5** for every artifact
7. **No em dashes anywhere** in any output (reinforces user preference)
8. **Halt and ask** on ambiguity; never guess
9. **Live-verify** version pins, deployment steps, and ecosystem facts before locking them; do not rely on memory

Any deliverable that violates these is not accepted. The stricter rule always wins.

---

## 7. Document changelog

- **YYYY-MM-DD**: Initial requirements document drafted, before any code

Update whenever a rule is added, tightened, or removed.

---

**End of requirements document.**