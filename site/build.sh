#!/bin/sh
# Builds what the playground loads into site/pkg. Open site/index.html straight from disk or serve site/.
# Needs the wasm32-unknown-unknown target and wasm-bindgen-cli at the version in Cargo.lock.
set -e
cd "$(dirname "$0")/.."
cargo build -p tikra-wasm --release --target wasm32-unknown-unknown
rm -rf site/pkg
wasm-bindgen --target no-modules --no-typescript --out-dir site/pkg target/wasm32-unknown-unknown/release/tikra_wasm.wasm
# file:// pages cannot fetch the .wasm, but they can run a classic script: the same bytes as base64
printf 'var TIKRA_WASM_BASE64 = "%s";\n' "$(base64 < site/pkg/tikra_wasm_bg.wasm | tr -d '\n')" > site/pkg/tikra_wasm_bytes.js
# the gallery, every figure of corpus/ (checked by crates/core/tests/corpus.rs), and the banner, as a script
node -e '
  const fs = require("fs");
  const figures = fs.readdirSync("corpus").filter((f) => f.endsWith(".tex")).sort()
    .map((f) => ({ name: f.slice(0, -4), tex: fs.readFileSync("corpus/" + f, "utf8") }));
  fs.writeFileSync("site/pkg/corpus.js", "var TIKRA_CORPUS = " + JSON.stringify(figures) + ";\n" +
    "var TIKRA_BANNER = " + JSON.stringify(fs.readFileSync("docs/banner.tex", "utf8")) + ";\n");
'
# the icons, drawn by tikra itself from site/favicon.tex (1 cm wide: dpi = pixels × 2.54)
cargo build -q -p tikra-cli --release
tikra=target/release/tikra
$tikra site/favicon.tex -o site/pkg/favicon.svg
$tikra site/favicon.tex -o site/pkg/favicon-32.png --dpi 81.28 --transparent
$tikra site/favicon.tex -o site/pkg/apple-touch-icon.png --dpi 457.2 --transparent
