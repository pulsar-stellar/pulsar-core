<!--
  Pulsar Core pull request template. Fill every section that applies.

  This is a standard your PR is measured against, alongside CONTRIBUTING.md.
  A PR that leaves sections as unedited placeholder text, or that changes
  behavior without changing tests, is sent back before review.

  If a section does not apply, keep its heading and write "N/A" with a
  one-line reason. Do not delete headings.

  Writing rules (see CONTRIBUTING.md > Writing rules):
  - No em dashes anywhere.
  - Avoid "seamlessly", "robust", "powerful", "leverage", "unlock",
    "cutting-edge", "revolutionize", "delve into", "elevate", and "empower"
    used figuratively. Prefer concrete verbs and specific nouns.
  - Numbers come from measured data. A number that is a target is labeled
    a target.
-->

# <type(scope): one-line summary>

<!--
  Match your squash-merge commit subject. Conventional format:
  type(scope): imperative, lowercase first letter, no trailing period, under 72 chars.
  Types:  feat  fix  refactor  test  docs  chore  build  ci
  Scopes: showcase  decoder  workspace  ci  docs  scripts  agent
-->

> **Status:** Draft | Ready for review
> **Closes:** #<issue>
> **ADRs:** <ADR-0NN, ADR-0NN | none>
> **Areas touched:** <contract | decoder | scripts | ci | docs | workspace>

---

## 1. Summary and core invariant

<!--
  Two or three sentences: what this PR changes and why. Then state the one
  invariant it establishes or preserves. The invariant is the single property
  a reviewer should hold the whole diff against.
-->

**What changed:**

**Core invariant this PR upholds:**

---

## 2. Scope and workstream audit

<!--
  One logical change per PR where practical (CONTRIBUTING.md > Commit rules).
  Say what is in scope and what is deliberately left out, so a reviewer is not
  hunting for work that was never intended here.
-->

**In scope:**
-

**Deliberately out of scope, and where it is tracked:**
-

**Requirement to evidence map:**

| Requirement (issue / ADR / spec section) | Where it is satisfied in this diff |
|---|---|
|  |  |

---

## 3. Background and correctness-boundary context

<!--
  What was the behavior before this change, and why did it need to change?
  Name the boundary this PR touches, if any:
  - The contract state machine, where every state-changing function calls
    require_auth before any caller-dependent read or write, amounts are i128,
    and no arithmetic can overflow or truncate unnoticed.
  - The decoder, the correctness boundary for every downstream consumer, whose
    stated guarantee is that every decode failure returns an error variant
    rather than a panic, an unbounded allocation, or a non-terminating loop.
  The contract source in contracts/showcase/src/contract.rs and its tests are
  the specification. A PR that touches no boundary can say so in one line.
-->

**Before:**

**After:**

| Case | Before | After |
|---|---|---|
|  |  |  |

---

## 4. Design and approach

<!--
  How the change works, and the decisions behind it. If it changes public
  behavior, link the ADR that authorizes it (.agent/decisions.md), or note that
  a new ADR is added in this PR. Call out any change to an event shape, an
  error variant, or an authorization check.
-->

**Approach:**

**Key decisions:**

**Event / error-variant / auth impact:** <!-- new or changed events, Error variants, or require_auth points, or "none" -->

---

## 5. Detailed changes, file by file

<!--
  Group by area. One short paragraph or bullet per file that carries a decision.
  Pure scaffolding can be summarized in a line.
-->

### Contract (`contracts/showcase`)
-

### Decoder (`crates/pulsar-decoder`)
-

### Scripts and CI
-

### Other (workspace, docs, agent)
-

---

## 6. Failure-mode analysis

<!--
  For each failure path the change adds or moves: what triggers it and which
  typed error it returns. Contract and decoder code return a typed error rather
  than panicking, so a panic on any input is a bug, not an outcome to document.
-->

| Failure scenario | Trigger | Typed error returned |
|---|---|---|
|  |  |  |

---

## 7. Verification and test evidence

<!--
  Paste the commands you ran and their results. CI must be green before review;
  this section is the local evidence that it will be. Behavior-carrying
  additions land with their tests in one commit (CONTRIBUTING.md > Test
  discipline). Coverage stays above 85 percent.
-->

```
cargo test
cargo fmt --check
cargo clippy -- -D warnings
cargo llvm-cov --workspace --locked --summary-only
scripts/verify-no-panic-apis.sh
```

<paste result>

**Contract artifact, if `contracts/showcase` changed:**

<!-- Build the contract with stellar contract build, never plain cargo build. -->

```
stellar contract build
```

| Metric | Before | After |
|---|---|---|
| `stellar contract build` succeeds |  |  |
| Wasm size in bytes (measured) |  |  |
| CPU / memory / rent change, if measured |  |  |

**Coverage (measured):**

**New or changed tests, and the single claim each one makes:**
-

**Test snapshots:** <!-- unchanged, or a diff under contracts/showcase/test_snapshots/ with a one-line reason for each moved snapshot -->

---

## 8. Configuration and toolchain reference

<!--
  This repo pins two Rust toolchains and a Soroban CLI. Note any change to a pin
  or a build or deploy parameter here, or write "no toolchain or config change".
  Secrets never enter the repo (CONTRIBUTING.md, SECURITY.md).
-->

| Pin / parameter | Value before | Value after | Where it lives |
|---|---|---|---|
|  |  |  |  |

---

## 9. Security, invariants, and correctness checklist

<!--
  Tick what applies. An unticked box that should be ticked is a reason the PR is
  not ready. Changes to crates/pulsar-decoder carry a higher review bar
  (CONTRIBUTING.md > Pull requests): expect fixtures derived from real testnet
  events rather than synthesized input.
-->

- [ ] No secret, key, or credential enters the repo or the diff. Deployer keys stay in the Stellar CLI identity store or GitHub Actions secrets; mainnet keys are handled with a hardware wallet.
- [ ] No `unwrap`, `unwrap_err`, `expect`, or `panic!` in production code under `src/`. `Option` converts to `Result` with `.ok_or(Error::Variant)` and propagates with `?`. Test files use `.expect("message")` with a non-empty message only. `scripts/verify-no-panic-apis.sh` passes.
- [ ] Every state-changing contract function calls `require_auth` before any caller-dependent read and before any write.
- [ ] No floats anywhere. Amounts are `i128`. No integer cast that can truncate; `TryFrom` with explicit error mapping is used instead.
- [ ] Arithmetic reachable from a public function cannot overflow or underflow unchecked. Storage TTL is handled so no entry becomes permanently unreachable.
- [ ] Events are defined in `events.rs` as `#[contractevent]` structs and emitted with `.publish(&env)`. No raw topic tuples, no direct `env.events().publish`.
- [ ] The decoder returns an error variant on every malformed input. No panic, no unbounded allocation, no non-terminating loop.
- [ ] `Cargo.lock` is unchanged, or the change lands in its own `build(deps)` commit that names the dependency that moved and why. `cargo build --locked` passes.
- [ ] Test snapshots under `contracts/showcase/test_snapshots/` are unchanged, or each moved snapshot is explained in this PR.
- [ ] The change stays within the SECURITY.md scope, and anything security-sensitive is handled through the repository Security tab, not a public issue.

---

## 10. Files changed

<!--
  A table of the files this PR touches. Keep it short: path, area, a one-line
  change, and the sign of the diff. The totals line is a sanity check against
  the diff stat.
-->

| Path | Area | Change | +/- |
|---|---|---|---|
|  |  |  |  |

**Totals:** <N files, +A / -B>

---

## 11. Rollout, deployment, and operations

<!--
  What has to happen for this to ship, and what an operator should watch.
  Deployments before v1.0.0-contracts target Stellar testnet only. Note whether
  the change alters the deployed contract and needs a redeploy, and whether the
  deployed contract ID moves. The decoder crate is publish = false until
  v0.2.0-contracts.
-->

**Deploy impact:** <none | contract changed, redeploy needed?, does the deployed ID move?>

**Reproducibility:** <Cargo.lock unchanged, or the build(deps) commit that moved it>

**Operator note:** <what to watch after deploy, or "none">

---

<!--
  Maintainer merge gate (the reviewer confirms these, not the author):
  - Branch from main, one logical change, CI green.
  - Commits follow the conventional format; pushed history is not rewritten.
  - A behavior change is paired with tests in the same commit.
  - Decoder changes are reviewed at the higher bar, with testnet-derived fixtures.
-->
