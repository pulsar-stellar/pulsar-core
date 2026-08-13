# Pulsar Core

Pulsar Stellar is a developer toolkit for Soroban contract events. Every Stellar project that needs to consume its contract's events today writes the same plumbing from scratch: XDR decoders, indexer glue, custom APIs. Pulsar Stellar provides three shared building blocks so they don't have to. A Rust library that turns raw contract events into typed data, a Go daemon that stores historical events past the seven-day RPC retention window, and a web explorer where anyone can paste a contract ID and browse every event that contract has ever emitted, decoded and searchable. It serves Soroban dapp builders, backend engineers integrating with existing protocols, and auditors reviewing contract behavior post-deployment.

## What this repository holds

`pulsar-core` is the Rust contract layer. It carries two artifacts that co-evolve:

- **`pulsar-showcase`**: the reference Soroban contract. Every function on it exists to exercise a specific decoder capability, which makes its emitted events the test fixture and documentation demo for the rest of the toolkit.
- **`pulsar-decoder`**: the crate that turns raw contract events into typed data, published to crates.io from v0.2.0-contracts onward.

The sibling repository `pulsar-stellar/pulsar-app` carries the TypeScript SDK, the Go indexer, the web explorer, and the documentation site.

## Status

Phase 6, Sprint 1, build step 4 of 57.

The Cargo workspace is scaffolded and the toolchain is pinned. No contract code has landed yet.

| Artifact | State |
|---|---|
| Cargo workspace and toolchain pin | in place |
| `pulsar-showcase` contract | not started, lands across Sprints 1 and 2 |
| `pulsar-decoder` crate | not started, placeholder lands in Sprint 3, real content at v0.2.0-contracts |
| Testnet deployment | not deployed |
| Published crate | not published |

The first release is `v0.1.0-contracts`, which ships the showcase contract deployed to Stellar testnet with its contract ID recorded in the tag body. This README gains a quickstart section at that point.

## Toolchain

| Tool | Version |
|---|---|
| Rust | 1.84.0, pinned in `rust-toolchain.toml` |
| Build target | `wasm32v1-none` |
| soroban-sdk | 26.1.0, pinned exactly |
| stellar-cli | 27.0.0 or newer |

The contract is built with `stellar contract build`, never with plain `cargo build`.

## Contributing

This project's initial scaffolding and much of its ongoing implementation is written with Claude Code assistance under human review. Every commit is authored, reviewed, and merged by a human maintainer. Design decisions, architecture choices, and merge judgments are human. If you contribute a PR, we don't require you to disclose whether AI tools helped you write it; we do require that your code passes review, tests, and the discipline rules in the requirements document.

## License

Apache-2.0.
