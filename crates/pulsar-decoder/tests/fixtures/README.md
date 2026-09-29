# Decoder fixtures

Each `<name>.xdr` holds one `ScVal` as raw XDR bytes. Each `<name>.json` holds
the `DecodedValue` that decoding it must produce. `TestFixtures` iterates every
pair, so adding a case means adding two files and nothing else.

## Where they came from

**`deposit_*`, `transfer_*`, `withdraw_*`** are real events emitted by the
showcase contract `CDNWTVUDKCCGW7GOC6SBLUFXXUCD2YDHWRDUSXZ6CYBQKQWLCUYYWI5L` on
Stellar testnet on 2026-09-01, in ledgers 4482356, 4482360 and 4482362, then
read back through `getEvents`. ADR-028 recorded that this contract had emitted
nothing inside the retention window, which is why the events were created rather
than found. The amounts decode to 5000, 1500 and 500, matching what the CLI
reported when the transactions were submitted, so the expected JSON is checked
against something other than this decoder's own output.

The contract's admin-only entry points, `emit_custom`, `set_admin` and
`initialize`, are not represented. The admin key belongs to whoever deployed it
and this repository does not hold it.

**Everything else** is constructed, because the showcase contract emits only
symbols, addresses and `i128` values, so the rest of the taxonomy has no live
source. They cover the boundaries a real event is unlikely to reach: `u32` at
its maximum, `i32` and `i64` at their minimums, `u64` at its maximum, `i128` at
its minimum, 256-bit values, a nested `vec`, a `map` with a non-string key, and
the four types ADR-023 routes to the `unknown` fallback.

## Regenerating

There is no generator committed. A fixture is a recorded observation, and a
script that rewrites expectations from the current implementation would turn
every regression into a passing test. Add a case by writing the two files, and
check the JSON against something other than this decoder before committing it.
