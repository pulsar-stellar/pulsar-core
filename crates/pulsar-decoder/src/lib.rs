//! Typed decoder for Soroban contract events.
//!
//! Placeholder. The crate exists so the workspace layout, the published name, and
//! the release plan are settled before any API is written, not because there is
//! anything to use yet.
//!
//! The decoder turns raw contract events into typed data, and it is the
//! correctness boundary for every downstream consumer of this toolkit: the Go
//! indexer, the TypeScript SDK, and the web explorer all depend on it agreeing
//! with what contracts actually emit. That is why it carries a higher review bar
//! than anything else here, and why its fixtures come from real emitted events
//! rather than hand-built XDR.
//!
//! The wire shapes it will decode are specified in `docs/requirements.md` section
//! 8.4, and `pulsar-showcase` emits every one of them. Those events are the
//! fixtures this crate is tested against.
//!
//! Real content lands at v0.2.0-contracts, deliberately after the app repo
//! demonstrates what shape consumers need. Designing the API in isolation and
//! reworking it later costs more than waiting.
#![no_std]
