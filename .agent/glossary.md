# Glossary: Soroban and Pulsar terminology

One paragraph per term. Updated when a new term enters the codebase.

## Soroban and Stellar

**Contract event**: a record a contract emits during execution through `env.events().publish(topics, data)`. It carries up to four topics and one data value, and it is written into the transaction's metadata, which makes it part of the permanent ledger history rather than a side channel. Events are how a contract tells the outside world what it did, since the ledger stores state but not the reasoning behind a state change. Everything Pulsar Stellar decodes, indexes, and displays is a contract event.

**Diagnostic event**: an event emitted for debugging rather than for consumers. Diagnostic events carry host-level detail such as contract log output, error context, and call boundaries. They are not persisted in transaction metadata by default, and a node operator has to opt in to producing them, so they are reliably available only during simulation. Pulsar Stellar does not index them. Treating them as durable history would be a mistake, because most nodes discard them.

**ScVal**: the XDR union that represents any value the Soroban host can hold. Every variant carries a tag naming its type, `ScvBool`, `ScvU32`, `ScvI128`, `ScvSymbol`, `ScvBytes`, `ScvAddress`, `ScvVec`, `ScvMap`, `ScvVoid`, and others. Event topics and event data are both `ScVal`s on the wire. Decoding an event means turning these tagged values into something typed and readable in the consumer's language, which is the decoder's entire job.

**SCSpecEntry**: an entry in the contract specification embedded in a compiled contract's wasm as a custom section. Spec entries describe the contract's functions, their argument and return types, and its user-defined structs, unions, and enums. Tooling reads them to generate typed bindings without a hand-written interface file. Events are not yet first-class in the spec format, which is the gap that makes a dedicated event decoder worth building.

**XDR**: External Data Representation, the binary serialization format defined in RFC 4506 that Stellar uses for everything on the wire, including transactions, ledger entries, contract values, and events. It is compact and unambiguous but not human-readable, and APIs commonly hand it over base64-encoded. Turning XDR into readable typed data is the plumbing that every project consuming contract events currently rewrites.

**TTL**: time to live, measured in ledgers rather than wall-clock time. Every contract data entry carries a TTL, and when it lapses the entry is archived and is no longer readable by a contract. Instance and persistent entries can be restored after archival; temporary entries cannot. Contracts push the expiry outward by calling `extend_ttl` with a threshold and a bump amount, where the threshold is the remaining-lifetime level below which the bump applies. At roughly five seconds per ledger, the 518,400-ledger bump this contract uses is about thirty days.

**Instance storage**: storage attached to the contract instance itself, sharing a single TTL with the instance and loaded whenever the contract is invoked. It suits small, bounded state read on nearly every call. This contract keeps `Initialized` and `Admin` there. Because the whole instance entry is read on every invocation, putting unbounded state in it would tax every call regardless of what that call touches.

**Persistent storage**: storage where each entry carries its own TTL and is archived independently. It suits per-user or otherwise unbounded state, which is why `Balance(Address)` lives there. An archived persistent entry is restorable, so value is recoverable after expiry, at the cost of a restore operation before the entry can be used again.

**Temporary storage**: the cheapest storage class, with per-entry TTL and no restoration after expiry. It fits genuinely disposable state such as short-lived nonces or rate-limit counters. This contract does not use it and must not add it, because everything it stores needs to survive.

**SEP-41**: the Stellar Ecosystem Proposal defining the standard fungible token interface for Soroban, covering functions such as `transfer`, `mint`, `burn`, `approve`, `allowance`, and `balance`, along with the event shapes those operations emit. Wallets, exchanges, and DeFi protocols code against it, so matching its event shapes means every SEP-41 token decodes without special-case handling. The showcase contract's `transfer` follows the SEP-41 event shape for exactly that reason.

**`getEvents` RPC method**: the Soroban RPC method that returns contract events, filtered by ledger range, contract ID, and topic patterns, and paginated with a cursor. It is the only practical way to read historical events without running your own full node. Its retention window is about seven days, which is the specific limitation the Pulsar indexer exists to overcome by storing what it fetches.

**Ledger**: one closed block of the Stellar network, containing the transactions that were applied and the resulting state changes, identified by a monotonically increasing sequence number. Ledgers close roughly every five seconds. The ledger sequence is the unit of time inside contracts, since a contract cannot read a clock and TTLs, event ordering, and pagination cursors are all expressed in ledger numbers.

**Contract ID**: the identifier of a deployed contract instance, a 32-byte value presented in strkey form beginning with `C`. It is derived deterministically from the deployer and a salt, so it is known before deployment completes. It is what a user pastes into the Pulsar explorer and what the indexer keys its stored events against.

**Wasm hash**: the SHA-256 hash of an uploaded contract's wasm bytes. Uploading code and instantiating a contract are separate operations in Soroban, so one wasm hash can back many contract IDs, each with its own storage. The hash identifies the code, the contract ID identifies the instance.

**`require_auth`**: the `Address` method that asserts the address authorized this invocation with these exact arguments. It checks a signed authorization entry for an account address, or delegates to `__check_auth` for a contract address. Failure halts execution with a host error. Calling it is what separates a caller passing you an address from that address actually consenting, so every state-changing function in this contract calls it before any caller-dependent read or any write.

**Invoker**: the immediate caller of the current contract call, either an account that submitted the transaction or another contract that called into this one. It is distinct from an address passed as an argument, and the distinction is a common source of authorization bugs: an argument is a claim, while authorization is proof.

## Pulsar

**`pulsar-showcase`**: the reference contract in this repository. Six state-changing functions and two read views, each existing to exercise a specific decoder capability rather than to model a product. Its emitted events are the fixtures the rest of the toolkit tests against.

**`pulsar-decoder`**: the Rust crate that turns raw contract events into typed data. It is the correctness boundary for every downstream consumer, which is why it carries a higher review bar, a zero-panic guarantee on malformed input, and fuzz coverage. It ships as a placeholder in v0.1.0-contracts and gains real content at v0.2.0-contracts.

**Fixture**: a stored event captured from a real deployment, checked into the repository with its raw base64 XDR, its expected decoded output, and its source metadata of ledger number, transaction hash, and contract ID. Fixtures are never fetched at test time, so a testnet reset cannot break the test suite. Regenerating one requires an ADR entry explaining what changed.

**Decoded event**: the output of the decoder, an event whose topics and data have been converted from `ScVal` into typed values with their names attached. This is the shape the indexer stores, the SDK returns, and the explorer renders.

**Indexer**: the Go daemon in the sibling repository that polls `getEvents`, decodes what it receives, and stores it in Postgres or SQLite. It exists because RPC retention is about seven days and consumers need history older than that. It lives in `pulsar-app`, not here.
