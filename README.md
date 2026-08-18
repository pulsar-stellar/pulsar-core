# Pulsar Core

Pulsar Stellar is a developer toolkit for Soroban contract events. Every Stellar project that needs to consume its contract's events today writes the same plumbing from scratch: XDR decoders, indexer glue, custom APIs. Pulsar Stellar provides three shared building blocks so they don't have to. A Rust library that turns raw contract events into typed data, a Go daemon that stores historical events past the seven-day RPC retention window, and a web explorer where anyone can paste a contract ID and browse every event that contract has ever emitted, decoded and searchable. It serves Soroban dapp builders, backend engineers integrating with existing protocols, and auditors reviewing contract behavior post-deployment.

## What this repository holds

`pulsar-core` is the Rust contract layer. It carries two artifacts that co-evolve:

- **`pulsar-showcase`**: the reference Soroban contract. Every function on it exists to exercise a specific decoder capability, which makes its emitted events the test fixture and documentation demo for the rest of the toolkit.
- **`pulsar-decoder`**: the crate that turns raw contract events into typed data, published to crates.io from v0.2.0-contracts onward.

The sibling repository `pulsar-stellar/pulsar-app` carries the TypeScript SDK, the Go indexer, the web explorer, and the documentation site.

## Status

Sprint 1. The contract is complete and its full public surface is implemented and
tested.

| Artifact | State |
|---|---|
| `pulsar-showcase` contract | complete, 8 public functions, 6 event types |
| Test suite | 54 tests, 98.7 percent line coverage |
| `pulsar-decoder` crate | placeholder, real content at v0.2.0-contracts |
| Testnet deployment | see Deployment below |
| Published crate | not published |

The first release is `v0.1.0-contracts`, which ships the showcase contract
deployed to Stellar testnet with its contract ID recorded in the tag body.

## Quickstart

### Prerequisites

Rust installs itself from `rust-toolchain.toml` on the first cargo command.
stellar-cli is a host tool and installs on the stable channel, from outside this
directory, because the project pin is older than stellar-cli needs to build:

```sh
cd ~ && rustup run stable cargo install --locked --force stellar-cli
```

Full toolchain list in `docs/requirements.md` section 1.8. Reasoning for the two
channels in ADR-007.

### Build

```sh
scripts/build.sh
```

Wraps `stellar contract build`, checks the artifact exists, and reports its size.
Plain `cargo build` compiles the crate but does not produce a deployable
artifact.

### Deploy to testnet

Needs a funded identity:

```sh
stellar keys generate deployer --network testnet
stellar keys fund deployer --network testnet
scripts/deploy-testnet.sh
```

The script checks the artifact and identity before submitting anything, and
prints the contract ID on success.

### Initialize

A freshly deployed contract has no admin, and every state-changing function
returns `NotInitialized` until one is set. This call is required before the
contract does anything:

```sh
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source deployer \
  --network testnet \
  -- initialize --admin <ADMIN_ADDRESS>
```

The admin address authorizes `set_admin` and `emit_custom`. Rotation requires the
current admin's signature, so a lost admin key cannot be recovered.

### Verify

```sh
stellar contract invoke --id <CONTRACT_ID> --source deployer --network testnet \
  -- admin
```

Returns the admin address. The contract's event history is browsable at
`https://stellar.expert/explorer/testnet/contract/<CONTRACT_ID>`.

## Deployment

Contract ID is recorded here once the first testnet deployment lands.

## Toolchain

| Tool | Version |
|---|---|
| Rust, project toolchain | 1.92.0, pinned in `rust-toolchain.toml` |
| Rust, host toolchain | stable 1.93 or newer, for installing binary tools only |
| Build target | `wasm32v1-none` |
| soroban-sdk | 26.1.0, pinned exactly |
| stellar-cli | 27.1.0 or newer |

The contract is built with `stellar contract build`, never with plain `cargo build`.

## Contributing

This project's initial scaffolding and much of its ongoing implementation is written with Claude Code assistance under human review. Every commit is authored, reviewed, and merged by a human maintainer. Design decisions, architecture choices, and merge judgments are human. If you contribute a PR, we don't require you to disclose whether AI tools helped you write it; we do require that your code passes review, tests, and the discipline rules in the requirements document.

## License

Apache-2.0.
