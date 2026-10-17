# Tests

Every figure in the tests and in `corpus/` was written for this repository to exercise the interpreter.

## How to run

```console
$ cargo test                                    # 152 tests, under 3 s
$ cargo llvm-cov --workspace --summary-only     # coverage (install with cargo install cargo-llvm-cov)
$ cargo llvm-cov --workspace --html             # browsable report in target/llvm-cov/html
$ cargo clippy --all-targets -- -D warnings
$ TIKRA_UPDATE_BANNER=1 cargo test -p tikra --test banner   # redraws docs/banner.svg and docs/banner.png
$ cargo run --release -p tikra --example bench              # benchmark over corpus/ (README)
```

tikra depends on mathra by path (`../mathra/crates/core`). Labels go through it, so a change in mathra can change a test that checks label text or size. Math follows mathra's default locale, which is English (`\sin` reads "sin", decimal point).

## What each file covers

Integration, in `crates/core/tests/`:

| File | Tests | What it exercises |
|---|---:|---|
| `input.rs` | 15 | The forms `convert` accepts: `standalone` with and without `border`, `tikzpicture` with options, inline `\tikz`, a bare `axis`, loose statements, input with no figure. Repairs (doubled backslashes, Markdown), a preamble with colors and styles, comments, SVG escaping. |
| `paths.rs` | 32 | `--`, `-\|`, `\|-`, `+` and `++`, polar coordinates, lengths with units, names and anchors, `calc` (`!t!`, sums, factors, rotation), `.. controls ..`, `to`/`edge` with `out`/`in`/`bend`/`looseness`, `rectangle`, `circle`, `ellipse`, `arc` (parentheses and options), `grid` (with the path's own `step` or its own), `plot` (expression, coordinates, `smooth`), decorations, rounded corners, `shorten`, `\clip`, polar coordinates with a braced angle, nodes in the middle of a segment, `bend left` given to the whole statement. Each operation also with an invalid coordinate. |
| `nodes.rs` | 14 | Shapes (`circle`, `ellipse`, `diamond`, `shape=`), minimum sizes, `inner sep`, `aspect`, named and angle anchors, baseline anchors (`base`, `base west`, `base east`, `text`), `anchor`, `above`/`below left=…`, `positioning` (`below=2cm of a`, `and`), `node distance`, `midway`/`pos=`, `label=`, text color, `font`, `text width`, `align`, `rotate`, `scale` (text included). |
| `styles_and_colors.rs` | 14 | Dashes, widths, opacities, hatching, arrow tips (every family and `>=`), xcolor colors (mixes, complement, names, `\definecolor` in rgb/RGB/gray/cmyk/HTML, `\colorlet`), `\tikzset` with `.style` and `.append style`, `\tikzstyle`, `every node`/`every path`, option list parsing, hatching as the fill, the `->>` tip. |
| `control.rs` | 13 | `\foreach` (ellipsis, `count`, several variables, nested, no leaking variables), `scope` with `shift`/`scale`/`rotate`/`x=`/`y=`, `\def` and relatives, `\pgfmathsetmacro`, `declare function`, unknown environments, `\end` without `\begin`, ignored commands. |
| `expressions.rs` | 3 | The pgfmath evaluator through the public API: every function (`sin` … `veclen`, `atan2`, `log10`), constants, scientific notation, units, `^` associativity, syntax errors. |
| `plots.rs` | 12 | pgfplots: `axis lines` (left, middle, box, none), the window from the data, `domain`/`samples` on the axis and on the series, `\addplot+`, marks, a legend with `at=` and a frame, log variants read as linear, `axis` without `\end`, `only marks`, `mark=`, `no marks`. |
| `svg_and_png.rs` | 7 | The generated SVG (opacities, dashes, drawn nodes, multiline labels and alignments, mathra's rules, lines and circles, nested clips, an empty scene) and the PNG (background, dpi, invalid SVG, zero/negative/infinite dpi, pixel cap). |
| `figures.rs` | 12 | Whole figures that combine several features: a number line, a folding fan in a rotated scope, a flowchart, stacked waves from a declared function, a signpost with hatching and inherited label color, a plot with a legend. |
| `corpus.rs` | 1 | Every figure in `corpus/` converts without warnings, with no `NaN`/`inf`, to SVG and PNG. |
| `problematic.rs` | 17 | Input built to break the interpreter (section below). |
| `banner.rs` | 1 | The README banner: `docs/banner.svg` must be byte for byte what tikra draws from `docs/banner.tex`, and `docs/banner.png` must be 1280 px wide. |

Command line, in `crates/cli/tests/cli.rs` (6 tests): help and version, stdin → stdout, file → SVG and PNG, `--dpi`, `--transparent`, `--warnings`, a missing file, a file and a stdin that are not UTF-8, input with no figure, output into a missing folder, a PNG that is too large. The test runs the real binary (`CARGO_BIN_EXE_tikra`), and `cargo llvm-cov` counts that child process's coverage.

Unit tests, inside the modules: `color.rs` (mixes), `expr.rs` (basic expressions), `options.rs` (styles), `label.rs` (wrapping and font size). Plus the documentation example in `lib.rs`.

## Coverage

`cargo llvm-cov --workspace --summary-only`:

| File | Regions | Functions | Lines |
|---|---:|---:|---:|
| `cli/src/main.rs` | 100.00% | 100.00% | 100.00% |
| `core/src/axes.rs` | 98.55% | 100.00% | 100.00% |
| `core/src/color.rs` | 100.00% | 100.00% | 100.00% |
| `core/src/expr.rs` | 99.80% | 100.00% | 100.00% |
| `core/src/label.rs` | 98.23% | 100.00% | 100.00% |
| `core/src/lib.rs` | 99.44% | 100.00% | 100.00% |
| `core/src/options.rs` | 100.00% | 100.00% | 100.00% |
| `core/src/path.rs` | 99.61% | 100.00% | 100.00% |
| `core/src/reader.rs` | 99.82% | 100.00% | 100.00% |
| `core/src/render.rs` | 99.63% | 100.00% | 100.00% |
| `core/src/scene.rs` | 100.00% | 100.00% | 100.00% |
| **Total** | **99.54%** | **100.00%** | **100.00%** |

The missing regions are almost all the error side of a `?` whose call cannot fail at that point (a point already validated earlier, for example) and the implicit `else` of an `if let` that would only fail with impossible state. Covering them would mean fabricating state from outside the API.

## Problematic cases

`tests/problematic.rs` runs each input on a thread with a 5-second deadline. An infinite loop fails the test instead of hanging the suite, and a panic shows up as a failure. Each case must end with a clear error or a warning, with no `NaN` or `inf` in the SVG.

| Input | Behavior |
|---|---|
| `\draw[step=0] (0,0) grid (1,1);` or `step=-1` | warning; grid ignored |
| `grid [step=0.000001] (100,100)` | warning; cap of 10,000 lines per direction |
| `decoration={zigzag, segment length=0pt}` | one step; cap of 10,000 |
| `\draw (1,0) arc (0:1000000000:1);` | one turn plus the rest, same end point |
| `plot[samples=1000000000]` | cap of 2,000 samples |
| `\def\a{\a\a}` and using `\a` | expansion stops at 100,000 characters; statement ignored with a warning |
| `a/.style={b}, b/.style={a}`, styles that multiply by 4 over 20 levels | cycle detected; cap of 10,000 options |
| `declare function={f(\x)=f(\x)+1;}` | error "nesting went past 200 levels", reported as a warning |
| 10,000 nested `(`, 10,000 `-` in a row | the same nesting error |
| `1^1^…^1` with 10,000 terms, `-` × 100,000 before a color | read in a loop, no deep recursion |
| `\foreach` 10,000 × 10,000 | error "figure too large: more than 100000 statements" |
| 5,000 nested `scope`, 200 nested `\foreach` | nesting error (cap of 64) |
| `\def` at the end of the text | ignored |
| `{1/0}`, `{0/0}`, `ln(0)`, `sqrt(-1)` in a coordinate, radius or angle | warning "coordinate is not finite"; path or node ignored |
| `\pgfmathsetmacro{\z}{1/0}` | warning "non-finite result" |
| `(1e300,1e300)`, `(200000,0)`, `minimum width=1e9cm` | scientific notation is read; warning beyond ±100,000 cm |
| an axis with `xmin=1e17, xmax=1e17+16` | ticks by index, cap of 50 |
| `--dpi 1000000`, a 1 km figure at 300 dpi | error "image too large" (cap of 100 megapixels) |
| `--dpi 0`, negative, `NaN` | error "invalid dpi: …" |
| stdin that is not UTF-8 | error "could not read standard input" |
| `\foreach \k in {1} {}` and using `\k` afterwards | the loop variable does not leak |
| `\end{ção}` (a name with accents) | read correctly |

Also tested, with no change needed: empty input or only comments (clear error), unbalanced braces, brackets and parentheses (warning), a node with no text, missing node names, Unicode and emoji in labels and node names, `samples=0`, `samples=1` and negative, an inverted or empty domain, `xmin=xmax` in pgfplots, a one-point path, `cycle` with no path, `\draw;`, `;;;;`, a zero or negative radius, a 0° arc, a missing file in the CLI, a label mathra refuses even in tolerant mode (nesting over 64 levels: the label goes away, with a warning).
