# Corpus

Hand-made figures used by the benchmark (`cargo run --release -p tikra --example bench`) and by `crates/core/tests/corpus.rs`. Each file is a complete input for `tikra`, written to exercise a group of features: geometry, function plots, pgfplots with a legend, flowcharts and graphs with positioning, `\foreach`, rotated scopes, decorations, hatching, arcs, arrow tips, clipping, opacity and math labels; and, from 26 on, what the libraries can do: 3D surfaces and heat maps, polar, ternary and Smith charts, fill between, stacked bars, mind maps, automata, Petri nets, circuits, trees, the graphs library, matrices, text along paths, markings, shadings, fadings, patterns, shapes, calendars, spy, Lindenmayer systems, data visualization, 3D coordinates and angles.

Add a figure by dropping a `.tex` file here; the test and the benchmark pick it up.
