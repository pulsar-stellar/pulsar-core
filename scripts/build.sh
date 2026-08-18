#!/usr/bin/env bash
#
# Builds the showcase contract to wasm.
#
# Wraps `stellar contract build`, which is the only supported way to produce a
# deployable artifact. Plain `cargo build` compiles the crate but does not emit
# the contract spec the runtime needs, so it is never a substitute.
#
# Usage: scripts/build.sh

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

WASM="target/wasm32v1-none/release/pulsar_showcase.wasm"

if ! command -v stellar >/dev/null 2>&1; then
    echo "stellar-cli not found on PATH."
    echo
    echo "Install it on the host stable channel, not the pinned project toolchain:"
    echo "  cd ~ && rustup run stable cargo install --locked --force stellar-cli"
    echo
    echo "See ADR-007 for why the install runs from outside this directory."
    exit 1
fi

echo "stellar-cli: $(stellar --version | head -1)"
echo "rustc:       $(rustc --version)"
echo

echo "Building contract..."
stellar contract build

if [ ! -f "$WASM" ]; then
    echo
    echo "Build reported success but $WASM is missing."
    echo "Check that the target in rust-toolchain.toml is wasm32v1-none."
    exit 1
fi

size=$(wc -c < "$WASM")
echo
echo "Artifact: $WASM"
echo "Size:     $size bytes"
echo
# Size is a reachability signal, not just a cost figure. Helpers and events that
# nothing calls are stripped, so a build that does not grow after wiring up a new
# caller usually means the wiring did not land. See .agent/testing.md.
echo "Size is a reachability signal: code nothing calls is stripped from the"
echo "artifact, so this number should grow when a new caller lands."
