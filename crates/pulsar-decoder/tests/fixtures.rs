//! Fixture-driven decode tests against the corpus shared with the Go decoder.
//!
//! Each `fixtures/<name>.xdr` holds one `ScVal` as raw XDR bytes, and each
//! `fixtures/<name>.json` holds the [`DecodedValue`] decoding it must produce.
//! The corpus is copied verbatim from pulsar-app's `internal/decoder` testdata:
//! the `deposit_*`, `transfer_*` and `withdraw_*` cases are real testnet events
//! whose expected JSON was checked against the amounts the CLI reported, not
//! against any decoder's own output, and the rest are constructed boundaries.
//! See `fixtures/README.md`.
//!
//! Reusing the same bytes and the same expected JSON as the Go decoder is what
//! makes "the two implementations agree" a checked fact rather than an intention:
//! a divergence in either decoder shows up here as a failing fixture.

use std::fs;
use std::path::{Path, PathBuf};

use pulsar_decoder::Decoder;
use stellar_xdr::curr::{Limits, ReadXdr, ScVal, WriteXdr};

/// The number of `.xdr`/`.json` pairs in the corpus. Asserted so a glob that
/// silently matches nothing, or a half-copied corpus, fails loudly rather than
/// passing by vacuously iterating an empty set.
const EXPECTED_FIXTURE_COUNT: usize = 31;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Turns raw fixture XDR into the canonical base64 a caller would receive from
/// RPC, so the test drives the real [`Decoder::decode_base64`] entry point
/// rather than an internal shortcut.
fn base64_of(raw: &[u8]) -> String {
    let value = ScVal::from_xdr(raw, Limits::none()).expect("fixture holds valid ScVal XDR");
    value
        .to_xdr_base64(Limits::none())
        .expect("a parsed ScVal re-encodes to base64")
}

#[test]
fn every_fixture_decodes_to_its_expected_json() {
    let dir = fixtures_dir();
    let mut xdr_paths: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("fixtures directory is readable")
        .map(|entry| entry.expect("directory entry is readable").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "xdr"))
        .collect();
    xdr_paths.sort();

    assert_eq!(
        xdr_paths.len(),
        EXPECTED_FIXTURE_COUNT,
        "expected {EXPECTED_FIXTURE_COUNT} .xdr fixtures in {}",
        dir.display()
    );

    let decoder = Decoder::new();
    let mut unknowns = 0i64;

    for xdr_path in xdr_paths {
        let name = xdr_path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("fixture name is valid UTF-8");
        let json_path = xdr_path.with_extension("json");

        let raw = fs::read(&xdr_path).expect("fixture xdr is readable");
        let expected: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&json_path).unwrap_or_else(|_| panic!("{name}.json is readable")),
        )
        .unwrap_or_else(|_| panic!("{name}.json is valid JSON"));

        let got = decoder
            .decode_base64(&base64_of(&raw))
            .unwrap_or_else(|e| panic!("{name} must decode: {e}"));
        let actual = serde_json::to_value(&got)
            .unwrap_or_else(|_| panic!("{name} decoded value serializes"));

        assert_eq!(actual, expected, "{name} decoded to the wrong wire shape");

        // A stored value must also read back into the same in-memory value.
        let round_tripped: pulsar_decoder::DecodedValue =
            serde_json::from_value(expected).unwrap_or_else(|_| panic!("{name} round trips"));
        assert_eq!(round_tripped, got, "{name} did not survive a round trip");

        if actual.get("type").and_then(serde_json::Value::as_str) == Some("unknown") {
            unknowns += 1;
        }
    }

    // The corpus carries the four variants ADR-023 routes to the fallback
    // (a wasm error, a contract error, a contract instance, and a ledger-key
    // nonce), so the decoder's own unknown counter must have moved by exactly
    // that many. This ties the counter to observed behaviour, not just its own
    // unit test.
    assert_eq!(
        decoder.unknown_count(),
        unknowns,
        "the decoder's unknown counter must match the unknown fixtures seen"
    );
    assert_eq!(
        unknowns, 4,
        "the corpus is expected to hold four unknown cases"
    );
}
