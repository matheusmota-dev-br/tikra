#!/bin/sh
# Every line of the Rust crates runs in the tests: runs them under cargo llvm-cov and fails if any
# line is left out. The per-line list is what counts, not the summary's percentage: the library is
# also built for its own unit tests, and the summary counts that build's closures again.
#
# Needs cargo-llvm-cov (cargo install cargo-llvm-cov) and the llvm-tools-preview component.
set -e
cd "$(dirname "$0")/.."

report=$(mktemp)
trap 'rm -f "$report"' EXIT
# instrumented binaries of an earlier run would mix old line numbers in
cargo llvm-cov clean --workspace
cargo llvm-cov --workspace --exclude tikra-python --all-features --show-missing-lines >"$report"
grep '^TOTAL' "$report"
if grep -q '^Uncovered Lines:' "$report"; then
  sed -n '/^Uncovered Lines:/,$p' "$report" | sed "s#$(pwd)/##" >&2
  echo "coverage: the lines above run in no test" >&2
  exit 1
fi
echo "coverage: every line runs in the tests"
