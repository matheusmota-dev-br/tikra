<p align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/banner.svg" alt="tikra: TikZ to SVG and PNG, no TeX install" width="100%"></p>

# tikra

tikra converts TikZ and pgfplots figures to SVG and PNG, with no TeX installation, no fonts on the system and no external processes. It reimplements TikZ, pgf and pgfplots in Rust, and labels are set by [mathra](https://github.com/matheusmota-dev-br/mathra) in Latin Modern, TeX's own Computer Modern. It runs in JavaScript, Python, Rust and the terminal, and every one gives the same SVG, byte for byte. **[Try it in the browser](https://matheusmota-dev-br.github.io/tikra/)** · **[See the gallery](https://matheusmota-dev-br.github.io/tikra/gallery.html)**.

## Install and use

| Where | Install | Registry |
|---|---|---|
| JavaScript and TypeScript: Node, bundlers, browsers | `npm install @mathlabs/tikra` | [npm](https://www.npmjs.com/package/@mathlabs/tikra) |
| Python 3.9+: wheels for Linux, macOS and Windows | `pip install tikra` | [PyPI](https://pypi.org/project/tikra/) |
| Terminal | `cargo install tikra-cli` | [crates.io](https://crates.io/crates/tikra-cli) |
| Rust | `cargo add tikra` | [crates.io](https://crates.io/crates/tikra) · [docs.rs](https://docs.rs/tikra) |

### JavaScript and TypeScript

Types included. In Node it works right away; in browsers and bundlers, await `init()` once. More in [`npm/README.md`](https://github.com/matheusmota-dev-br/tikra/blob/main/npm/README.md).

```js
import { init, tikzToSvg, render, png } from "@mathlabs/tikra";

await init();                                                    // browsers and bundlers; Node needs nothing
const svg = tikzToSvg(String.raw`\tikz \draw[->] (0,0) -- (2,1) node[right] {$v$};`);
const { warnings } = render(code);                               // what was ignored or approximated
const bytes = png(svg, { dpi: 300 });                            // Uint8Array; { background: false } for transparent
```

### Python

```python
import tikra

svg = tikra.tikz_to_svg(r"\tikz \draw[->] (0,0) -- (2,1) node[right] {$v$};")
open("v.png", "wb").write(tikra.png(svg, dpi=300))     # background=False for transparent
tikra.render(code)["warnings"]                          # dict: svg, warnings
```

Types included; more in [`crates/python/README.md`](https://github.com/matheusmota-dev-br/tikra/blob/main/crates/python/README.md).

### Terminal

```console
$ tikra figure.tex -o figure.svg
$ tikra figure.tex -o figure.png --dpi 300       # white background
$ tikra figure.tex -o figure.png --transparent
$ cat figure.tex | tikra --warnings              # SVG on stdout, warnings listed
```

### Rust

```rust
let out = tikra::convert(r"\begin{tikzpicture}\draw[->] (0,0) -- (2,1) node[right] {$v$};\end{tikzpicture}")?;
std::fs::write("v.svg", &out.svg)?;
std::fs::write("v.png", tikra::png(&out.svg, 300.0, true)?)?;
for w in &out.warnings { eprintln!("{w}"); }
```

The banner above is drawn by tikra itself from [`docs/banner.tex`](https://github.com/matheusmota-dev-br/tikra/blob/main/docs/banner.tex).

## Examples

Each image is an SVG drawn by tikra from the figure in [`corpus/`](https://github.com/matheusmota-dev-br/tikra/tree/main/corpus) it links to, shown on a white card; `tests/examples.rs` keeps them in sync with the code. The [gallery](https://matheusmota-dev-br.github.io/tikra/gallery.html) draws all 227 figures of the corpus, by topic, live in the browser.

<table>
<tr><td align="center"><a href="https://github.com/matheusmota-dev-br/tikra/blob/main/corpus/plots-3d/lorenz-attractor.tex"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/lorenz-attractor.svg" alt="The Lorenz attractor, colored by height" width="380"></a><br><sub>The Lorenz attractor, colored by height</sub></td><td align="center"><a href="https://github.com/matheusmota-dev-br/tikra/blob/main/corpus/charts/nightingale-rose.tex"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/nightingale-rose.svg" alt="Nightingale's rose of the causes of mortality" width="380"></a><br><sub>Nightingale's rose of the causes of mortality</sub></td></tr>
<tr><td align="center"><a href="https://github.com/matheusmota-dev-br/tikra/blob/main/corpus/plots-2d/normal-distribution.tex"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/normal-distribution.svg" alt="The bell curve and the 68–95–99.7 rule" width="380"></a><br><sub>The bell curve and the 68–95–99.7 rule</sub></td><td align="center"><a href="https://github.com/matheusmota-dev-br/tikra/blob/main/corpus/diagrams/petersen-graph.tex"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/petersen-graph.svg" alt="The Petersen graph" width="380"></a><br><sub>The Petersen graph</sub></td></tr>
<tr><td align="center"><a href="https://github.com/matheusmota-dev-br/tikra/blob/main/corpus/science/feynman-diagram.tex"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/feynman-diagram.svg" alt="A Feynman diagram, with decorations" width="380"></a><br><sub>A Feynman diagram, with decorations</sub></td><td align="center"><a href="https://github.com/matheusmota-dev-br/tikra/blob/main/corpus/geometry/golden-spiral.tex"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/golden-spiral.svg" alt="The golden spiral" width="380"></a><br><sub>The golden spiral</sub></td></tr>
<tr><td align="center"><a href="https://github.com/matheusmota-dev-br/tikra/blob/main/corpus/nodes-and-text/periodic-table-corner.tex"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/periodic-table-corner.svg" alt="The periodic table, by family" width="380"></a><br><sub>The periodic table, by family</sub></td><td align="center"><a href="https://github.com/matheusmota-dev-br/tikra/blob/main/corpus/plots-3d/surface-3d.tex"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/surface-3d.svg" alt="A pgfplots surface and its colorbar" width="380"></a><br><sub>A pgfplots surface and its colorbar</sub></td></tr>
</table>

## How close to TikZ

Every example of the TikZ manual and of the pgfplots manual is drawn by pdflatex and by tikra, and the two are compared figure by figure ([docs/conformance.md](https://github.com/matheusmota-dev-br/tikra/blob/main/docs/conformance.md), [how](https://github.com/matheusmota-dev-br/tikra/blob/main/tools/conformance/README.md)):

| | Figures | Drawn as TeX draws them |
|---|---:|---:|
| Every figure of the manuals | 2502 | 2330 (93.1%) |
| Our 100%: the ones a converter can reproduce | 2486 | 2371 (95.4%) |

"Our 100%" leaves out what no converter can reproduce (random numbers without a seed, image files, gnuplot, the day's date) and counts the figures that differ only by antialiasing as drawn; the list is checked by eye ([unreproducible.json](https://github.com/matheusmota-dev-br/tikra/blob/main/tools/conformance/unreproducible.json)). tikra converts a figure in about half a millisecond, against about 0.7 s for pdflatex.

## Input

A full `standalone` document, a bare `tikzpicture` or `pgfpicture`, an inline `\tikz`, a lone `axis` environment, or loose TikZ statements. The preamble is read for `\definecolor`, `\colorlet`, `\tikzset`, `\pgfplotsset`, `\newcommand` and `\def`; `\usetikzlibrary` lines are accepted, and every library is available.

## What it draws

TikZ and pgf as the manuals describe them, through a small TeX interpreter (macros, `\def`, `\newcommand`, conditionals, registers, `\foreach`, pgfmath and `\tikzmath`):

- **Paths and coordinates**: every path operation (`--`, `|-`, `to`, `.. controls ..`, `arc`, `circle`, `rectangle`, `grid`, `plot`, `svg`, `let`), `calc`, positioning, polar and 3D coordinates, intersections, `through`, transformations and `transform canvas`, nonlinear transformations, `spy`.
- **Nodes and shapes**: every shape library (geometric, symbols, arrows, multipart, callouts, misc), labels and pins, `quotes`, matrices and `matrix of nodes`, `fit`, pics and `\pgfdeclareshape`; text with `text width`, alignment, line breaking with hyphenation, `itemize`, simple `tabular`, font sizes and families.
- **Diagrams**: chains, trees, the `graphs` library, mind maps, automata, Petri nets, ER diagrams, circuits (`circuits.ee.IEC`, logic gates), calendars, folding, turtle and Lindenmayer systems.
- **Painting**: arrow tips (`arrows.meta`, `bending`), dashes, opacity, transparency groups and blend modes, shadings (axial, radial, ball, declared and functional), fadings, patterns (classic, `patterns.meta`, declared), shadows, backgrounds and layers, every decoration (`pathmorphing`, `pathreplacing`, `markings`, `shapes`, `text`, `fractals`, `footprints`).
- **pgfplots**: 2D and 3D axes, `\addplot` of expressions, coordinates and tables, marks, legends, colorbars and colormaps, bar charts and histograms, error bars, `fill between`, surfaces, meshes, patches and contours, polar, ternary, Smith charts, `dateplot`, group plots and statistics; and the `datavisualization` library.

The details, and what is left out, are in [`spec/subset.md`](https://github.com/matheusmota-dev-br/tikra/blob/main/spec/subset.md) and [docs/conformance.md](https://github.com/matheusmota-dev-br/tikra/blob/main/docs/conformance.md).

Labels: node text is typeset by mathra in Latin Modern Math, with `$…$` and `\(…\)` for math, `\textbf`, `\textit`, `\texttt`, `\textcolor`, `\\` for line breaks and `\tiny`…`\Huge`. Math follows mathra's default locale (English).

## Warnings, not errors

What tikra does not understand becomes a **warning**, and the figure is drawn with the rest: unknown commands, environments and options, TikZ escaped twice or for Markdown (undone), labels with broken LaTeX (recovered by mathra's tolerant mode). `--warnings` lists them; `Output::warnings` has them in the library.

Limits protect against hostile input: 100,000 statements per figure, 64 levels of nesting, 2,000 samples per plot, coordinates within ±100,000 cm, PNG up to 100 megapixels. Past a limit you get a clear error or a warning, never a hang.

## Benchmark

[`corpus/`](https://github.com/matheusmota-dev-br/tikra/tree/main/corpus) holds 227 hand-made figures, one folder per topic, that cover the features above. Reproduce with:

```console
$ cargo run --release -p tikra --example bench
```

One run on an Apple M5 (arm64), 15 runs per figure, median per figure:

| Figures | Without warnings | With warnings | Errors |
|---:|---:|---:|---:|
| 227 | 227 | 0 | 0 |

| Per figure | Mean | p95 |
|---|---:|---:|
| SVG | 1.68 ms | 5.76 ms |
| SVG + PNG at 300 dpi | 6.58 ms | 17.10 ms |

The corpus now holds large figures too (3D surfaces of a thousand cells, fractals, the Lorenz attractor): most figures take well under a millisecond.

`tests/corpus.rs` checks on every `cargo test` that all corpus figures convert without warnings.

## Development

```console
$ cargo test                                  # every test
$ cargo llvm-cov --workspace --summary-only   # coverage (100% of lines)
$ cargo clippy --all-targets -- -D warnings
$ npm/build.sh && npm/test/run.sh             # the npm package, packed and installed as users get it
$ site/build.sh                               # the playground: then open site/index.html
$ scripts/release.sh --dry-run                # what a release would publish
$ tools/conformance/run.sh                    # how far from TikZ: every manual example, TeX next to tikra
```

What each test file covers and the inputs built to break the interpreter are in [docs/tests.md](https://github.com/matheusmota-dev-br/tikra/blob/main/docs/tests.md).

tikra depends on the published `mathra` crate. To work against a local checkout of mathra next to this one, add to `.cargo/config.toml` (ignored by git), and leave the `Cargo.lock` changes it causes out of commits:

```toml
[patch.crates-io]
mathra = { path = "../mathra/crates/core" }
```

**Releasing.** Each package has its own version: `Cargo.toml` (crates `tikra` and `tikra-cli`), `npm/package.json` and `crates/python/Cargo.toml`. Bump the ones to release in a pull request, merge it, then run `scripts/release.sh` on `main`: it tags every version not yet on its registry (`v*`, `npm-v*`, `py-v*`) and pushes the tags at once; each tag runs its workflow (`crates.yml`, `npm.yml`, `pypi.yml`), which publishes with trusted publishing, without stored tokens. A rerun of a half-done release skips what is already out.

The first release needs a few steps by hand, because crates.io and npm only let a package that already exists trust a workflow:

1. crates.io: `cargo login`, then `cargo publish -p tikra` and `cargo publish -p tikra-cli`. Then, for each crate, Settings → Trusted Publishing: repository `matheusmota-dev-br/tikra`, workflow `crates.yml`.
2. npm: `npm/build.sh && cd npm/pkg && npm publish --access public` (the `@mathlabs` scope). Then in the package settings, Trusted Publisher → GitHub Actions: repository `matheusmota-dev-br/tikra`, workflow `npm.yml`.
3. PyPI accepts a trusted publisher before the project exists: Publishing → add a pending publisher, project `tikra`, repository `matheusmota-dev-br/tikra`, workflow `pypi.yml`, environment `pypi`. The `py-v*` tag publishes it.
4. GitHub: Settings → Pages → Source: "GitHub Actions", for the playground (`pages.yml`), and an environment named `pypi`.

Every pull request runs the same checks (`.github/workflows/ci.yml`): fmt, clippy, the Rust, wasm, npm and Python tests.

## Licenses

Code: MIT or Apache-2.0. Labels are set in Latin Modern Math, the OpenType version of TeX's Computer Modern (GUST Font License), which comes from mathra with its STIX Two Math (SIL OFL 1.1); the Python and npm packages carry both license texts.
