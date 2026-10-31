#!/bin/sh
# The examples that use the command line, into examples/out. Open the SVGs in a browser.
set -e
cd "$(dirname "$0")/.."
cargo build --quiet --release -p tikra-cli
tikra=target/release/tikra
out=examples/out
mkdir -p "$out"
$tikra examples/source-map.tex --source -o "$out/source-map.svg"
$tikra examples/construction.tex -o "$out/construction.gif"
$tikra examples/construction.tex -o "$out/construction.apng"
$tikra examples/walk.tex --parts -o "$out/walk.svg"
$tikra examples/slides.tex --slides -o "$out/slides"
$tikra examples/slides.tex --click -o "$out/slides-click.svg"
$tikra examples/plotter.tex -o "$out/plotter.gcode"
$tikra examples/plotter.tex -o "$out/plotter.hpgl"
$tikra examples/play.tex --play -o "$out/play.svg"
