# Security Policy

## Audit status

**This code is unaudited. Do not use it to custody real value.**

`pulsar-showcase` is a reference contract. It exists to emit a known set of event shapes that the Pulsar Stellar toolkit decodes in its tests and documentation. It is not a financial product, it holds no real assets, and its balance tracking is a fixture rather than a ledger anyone should rely on.

No third-party security audit has been performed on any code in this repository. No audit is scheduled. A security review of `pulsar-decoder` is planned for `v1.0.0-contracts`, either a community review with a documented threat model or a professional audit if funding allows, and this section will be updated with the result when that happens.

Deployments before `v1.0.0-contracts` target Stellar testnet only.

## Reporting a vulnerability

Report privately through GitHub. Open the repository's **Security** tab and choose **Report a vulnerability**. The report stays private between you and the maintainers until an advisory is published.

Do not open a public issue for a security problem. Do not post details in the Telegram group.

Include what you have:

- Which surface is affected, from the table below
- What an attacker gains
- Steps to reproduce, ideally a failing test or a testnet transaction hash
- Affected version, commit SHA, or deployed contract ID

Response targets, stated as targets rather than guarantees while this is a solo-maintained project:

| Stage | Target |
|---|---|
| Acknowledge receipt | 3 working days |
| Initial assessment | 10 working days |
| Fix or documented mitigation for a confirmed high or critical issue | 30 days |

If a report goes unacknowledged past the first target, escalate by opening a public issue that says a security report is awaiting acknowledgement. Include no vulnerability details in it.

## Scope

In scope:

| Surface | Path |
|---|---|
| Reference contract source | `contracts/showcase/` |
| Decoder crate | `crates/pulsar-decoder/` |
| Build and deploy scripts | `scripts/` |
| CI workflow, including supply chain concerns such as unpinned actions | `.github/workflows/` |
| Workspace dependency pins | `Cargo.toml`, `Cargo.lock` |

Out of scope here:

- The TypeScript SDK, Go indexer, web explorer, and documentation site. Report those against `pulsar-stellar/pulsar-app`, which has its own policy.
- Stellar protocol, `soroban-sdk`, and Stellar RPC itself. Report those to the Stellar Development Foundation.
- Availability of public testnet RPC endpoints and of the testnet deployment.
- Findings that require a maintainer to run untrusted code or hand over credentials.

Reports that a testnet deployment can be drained are in scope as contract logic findings, and out of scope as loss of value. Testnet assets carry no value.

## What this project treats as a vulnerability

In the contract: a missing or incorrect `require_auth`, an arithmetic overflow or underflow reachable from a public function, a storage entry that can be made permanently unreachable through TTL mishandling, a state transition that violates the specification in `contracts/showcase/src/contract.rs` and its tests (the contract source is the specification), or any input that causes a panic instead of returning a typed error.

In the decoder: any input that causes a panic, an unbounded allocation, or a non-terminating loop. The decoder's stated guarantee is that every decode failure returns an error variant, so a panic on malformed input is a bug, not expected behavior.

In the repository: a committed credential, a dependency pinned to a version with a known advisory, or a CI configuration that lets an untrusted PR reach repository secrets.

## Disclosure

Coordinated disclosure. Once a fix ships, a GitHub Security Advisory is published naming the reporter, unless the reporter asks to stay anonymous.

There is no bug bounty. This project has no funding to pay for one, and saying otherwise would be dishonest.

## Credentials

Secrets never enter this repository. Deployer keys stay in the local Stellar CLI identity store or in GitHub Actions secrets, and mainnet keys are handled with a hardware wallet. If a credential reaches a commit, the response is to rotate it, not merely to remove the commit.
