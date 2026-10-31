#!/bin/sh
# The examples that use the command line, into examples/out. Open the SVGs in a browser.
set -e
cd "$(dirname "$0")/.."
cargo build --quiet --release -p tikra-cli
tikra=target/release/tikra
out=examples/out
mkdir -p "$out"
$tikra examples/source-map.tex --source -o "$out/source-map.svg"
