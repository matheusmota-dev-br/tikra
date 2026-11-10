#!/bin/sh
# Every line of the Rust crates runs in the tests: runs them under cargo llvm-cov and fails if any
# line is left out. The per-line list is what counts, not the summary's percentage: the library is
# also built for its own unit tests, and the summary counts that build's closures again.
#
#   scripts/coverage.sh               the check
#   scripts/coverage.sh --html <dir>  also the HTML report in <dir>/html, and <dir>/badge.json for the
#                                     README's badge (shields.io's endpoint format), from the same run
#
# Needs cargo-llvm-cov (cargo install cargo-llvm-cov) and the llvm-tools-preview component.
set -e
cd "$(dirname "$0")/.."

out=
if [ "$1" = --html ]; then
  out=${2:?"--html needs a folder"}
fi

report=$(mktemp)
trap 'rm -f "$report"' EXIT
# instrumented binaries of an earlier run would mix old line numbers in
cargo llvm-cov clean --workspace
cargo llvm-cov --workspace --exclude tikra-python --all-features --show-missing-lines >"$report"
grep '^TOTAL' "$report"

# the missed lines: "Uncovered Lines:" and then one "file: 12, 30, 31" per file
missed=$(sed -n '/^Uncovered Lines:/,$p' "$report" | grep ': [0-9]' | tr ',' '\n' | grep -c '[0-9]' || true)

if [ -n "$out" ]; then
  rm -rf "$out"
  mkdir -p "$out"
  cargo llvm-cov report --html --output-dir "$out"
  # the totals in the report's index count the unit-test build again: say so above them
  note='<p style="font-family:sans-serif;background:#fff8c5;padding:8px 12px;border:1px solid #d4a72c">'
  if [ "$missed" -eq 0 ]; then
    note="$note<b>Every line runs in the tests</b>: no line of the files below is left out."
  else
    note="$note<b>$missed lines run in no test</b>: they are marked red in the files below."
  fi
  note="$note The totals read a little under 100% even so: the library is also built for its own unit tests, and they count that build's functions again (docs/tests.md, \"Coverage\").</p>"
  # after the report's own heading
  # (the index is one long line: the note goes in it, not on a line of its own)
  awk -v note="$note" '!done && sub(/<\/h2>/, "</h2>" note) { done = 1 } { print }' "$out/html/index.html" >"$report.html"
  mv "$report.html" "$out/html/index.html"
  if [ "$missed" -eq 0 ]; then
    message="100% lines" color=brightgreen
  else
    message="$missed lines missed" color=red
  fi
  printf '{"schemaVersion": 1, "label": "coverage", "message": "%s", "color": "%s"}\n' "$message" "$color" >"$out/badge.json"
fi

if [ "$missed" -ne 0 ]; then
  sed -n '/^Uncovered Lines:/,$p' "$report" | sed "s#$(pwd)/##" >&2
  echo "coverage: the lines above run in no test" >&2
  exit 1
fi
echo "coverage: every line runs in the tests"
