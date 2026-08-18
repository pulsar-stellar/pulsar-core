#!/usr/bin/env bash
#
# Fails if production contract code uses an API that panics instead of returning
# a typed error. A panic in a deployed contract is an untyped failure a caller
# cannot handle, which is why requirements.md section 5.1 forbids the placeholder
# forms and CONTRIBUTING forbids the rest.
#
# Scope: everything under a crate's src/ directory, up to the first #[cfg(test)]
# marker in each file. Tests below that marker are exempt, and integration tests
# under tests/ are not examined at all. Test code may use .expect("message") with
# a descriptive message, per ADR-015: there the panic text is the diagnostic.
#
# Usage: scripts/verify-no-panic-apis.sh [src-dir ...]
# Defaults to every contracts/*/src and crates/*/src directory that exists.

set -euo pipefail

BANNED_DESC=(
    '.unwrap()'
    '.expect('
    'panic!'
    'todo!'
    'unimplemented!'
)
# Matched against code with comments and string literals already stripped.
BANNED_RE='\.unwrap\(\)|\.expect\(|\bpanic!|\btodo!|\bunimplemented!'

if [ "$#" -gt 0 ]; then
    search_dirs=("$@")
else
    search_dirs=()
    for d in contracts/*/src crates/*/src; do
        [ -d "$d" ] && search_dirs+=("$d")
    done
fi

if [ "${#search_dirs[@]}" -eq 0 ]; then
    echo "No source directories found. Nothing to check."
    exit 0
fi

violations=0
files_checked=0

while IFS= read -r file; do
    files_checked=$((files_checked + 1))

    # Production code is everything above the first #[cfg(test)]. A file without
    # one is production code throughout.
    cutoff=$(grep -n '#\[cfg(test)\]' "$file" | head -1 | cut -d: -f1 || true)
    if [ -n "$cutoff" ]; then
        end=$((cutoff - 1))
    else
        end=$(wc -l < "$file")
    fi
    [ "$end" -lt 1 ] && continue

    # Strip line comments and string literals before matching, so prose in a doc
    # comment and a message inside .expect("...") do not register as calls.
    hits=$(
        head -n "$end" "$file" \
            | sed -e 's://.*::' -e 's:"[^"]*":"":g' \
            | grep -nE "$BANNED_RE" \
            || true
    )

    if [ -n "$hits" ]; then
        while IFS= read -r hit; do
            lineno=${hit%%:*}
            printf '%s:%s: %s\n' "$file" "$lineno" \
                "$(sed -n "${lineno}p" "$file" | sed 's/^[[:space:]]*//')"
            violations=$((violations + 1))
        done <<< "$hits"
    fi
done < <(find "${search_dirs[@]}" -name '*.rs' -type f | sort)

if [ "$violations" -gt 0 ]; then
    echo
    echo "Found $violations panicking API call(s) in production code."
    echo
    echo "Banned in production: ${BANNED_DESC[*]}"
    echo
    echo "Convert Option to Result with .ok_or(Error::Variant) and propagate with ?."
    echo "See requirements.md section 5.1, CONTRIBUTING.md code rules, and ADR-015"
    echo "for the test-code exception."
    exit 1
fi

echo "No panicking APIs in production code. Checked $files_checked file(s)."
