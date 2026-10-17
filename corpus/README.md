# Corpus

Hand-made figures used by the benchmark (`cargo run --release -p tikra --example bench`) and by `crates/core/tests/corpus.rs`. Each file is a complete input for `tikra`, written to exercise a group of features: geometry, function plots, pgfplots with a legend, flowcharts and graphs with positioning, `\foreach`, rotated scopes, decorations, hatching, arcs, arrow tips, clipping, opacity and math labels.

Add a figure by dropping a `.tex` file here; the test and the benchmark pick it up.
