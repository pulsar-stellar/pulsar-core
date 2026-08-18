#!/usr/bin/env bash
#
# Deploys the showcase contract to Stellar testnet and prints the contract ID.
#
# Assumes scripts/build.sh has produced the artifact and that a funded deployer
# identity exists. Both are checked before anything is submitted, because a
# failure halfway through a deploy is harder to reason about than a refusal.
#
# Testnet only, and the network is hardcoded below rather than taken as an
# argument. Deploying to the public network is a separate deliberate act with its
# own key handling, not a flag on this script. `grep -c` for the public network's
# name in this file should return zero, which is a check worth keeping honest.
#
# Usage: scripts/deploy-testnet.sh [identity-name]
# Identity defaults to "deployer".

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

WASM="target/wasm32v1-none/release/pulsar_showcase.wasm"
NETWORK="testnet"
IDENTITY="${1:-deployer}"

if ! command -v stellar >/dev/null 2>&1; then
    echo "stellar-cli not found on PATH."
    echo "  cd ~ && rustup run stable cargo install --locked --force stellar-cli"
    exit 1
fi

if [ ! -f "$WASM" ]; then
    echo "No artifact at $WASM."
    echo
    echo "Build it first:"
    echo "  scripts/build.sh"
    exit 1
fi

if ! stellar keys ls 2>/dev/null | grep -qx "$IDENTITY"; then
    echo "Identity '$IDENTITY' does not exist."
    echo
    echo "Create and fund one:"
    echo "  stellar keys generate $IDENTITY --network $NETWORK"
    echo "  stellar keys fund $IDENTITY --network $NETWORK"
    echo
    echo "Existing identities:"
    stellar keys ls 2>/dev/null | sed 's/^/  /' || echo "  none"
    exit 1
fi

deployer_address="$(stellar keys address "$IDENTITY")"

echo "Network:  $NETWORK"
echo "Identity: $IDENTITY ($deployer_address)"
echo "Artifact: $WASM ($(wc -c < "$WASM") bytes)"
echo
echo "Deploying..."
echo

contract_id="$(
    stellar contract deploy \
        --wasm "$WASM" \
        --source "$IDENTITY" \
        --network "$NETWORK"
)"

# Strkey for a contract is 56 characters: a C followed by 55 base32 characters.
# Validating here means a malformed or empty capture fails loudly rather than
# being recorded in a tag body and propagated to the app repo.
if ! [[ "$contract_id" =~ ^C[A-Z0-9]{55}$ ]]; then
    echo
    echo "Deploy returned something that is not a contract ID:"
    echo "  $contract_id"
    echo
    echo "Nothing has been recorded. Check the output above before retrying."
    exit 1
fi

echo
echo "================================================================"
echo "  Deployed to $NETWORK"
echo "================================================================"
echo
echo "  Contract ID: $contract_id"
echo "  Deployer:    $deployer_address"
echo "  Explorer:    https://stellar.expert/explorer/testnet/contract/$contract_id"
echo
echo "  This ID is a release artifact. Record it in:"
echo
echo "    1. The v0.1.0-contracts tag body"
echo "    2. The README deployment section"
echo "    3. pulsar-app, as NEXT_PUBLIC_SHOWCASE_CONTRACT_ID and"
echo "       PULSAR_INDEXER_BOOTSTRAP_CONTRACTS"
echo
echo "  The contract is deployed but not initialized. Call initialize with the"
echo "  admin address before invoking anything else."
echo
echo "================================================================"
