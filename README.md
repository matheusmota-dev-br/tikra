<p align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/banner.svg" alt="tikra: TikZ to SVG and PNG, no TeX install" width="100%"></p>

# tikra

tikra converts TikZ and pgfplots figures to SVG and PNG, with no TeX installation, no fonts on the system and no external processes. Math in labels is typeset by [mathra](https://github.com/matheusmota-dev-br/mathra). It runs in JavaScript, Python, Rust and the terminal, and every one gives the same SVG, byte for byte. **[Try it in the browser](https://matheusmota-dev-br.github.io/tikra/)**.

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

Each image is an SVG drawn by tikra from the figure in [`corpus/`](https://github.com/matheusmota-dev-br/tikra/tree/main/corpus) it links to, shown on a white card. `tests/examples.rs` keeps them in sync with the code.

<table>
<tr><td align="center"><a href="https://github.com/matheusmota-dev-br/tikra/blob/main/corpus/01-triangle-angles.tex"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/01-triangle-angles.svg" alt="Triangle with an angle arc and a height" width="380"></a><br><sub>Triangle with an angle arc and a height</sub></td><td align="center"><a href="https://github.com/matheusmota-dev-br/tikra/blob/main/corpus/03-unit-circle.tex"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/03-unit-circle.svg" alt="Unit circle with polar points" width="380"></a><br><sub>Unit circle with polar points</sub></td></tr>
<tr><td align="center"><a href="https://github.com/matheusmota-dev-br/tikra/blob/main/corpus/05-pgfplots-legend.tex"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/05-pgfplots-legend.svg" alt="pgfplots: expressions, marks and a legend" width="380"></a><br><sub>pgfplots: expressions, marks and a legend</sub></td><td align="center"><a href="https://github.com/matheusmota-dev-br/tikra/blob/main/corpus/07-flowchart.tex"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/07-flowchart.svg" alt="Flowchart with positioning" width="380"></a><br><sub>Flowchart with positioning</sub></td></tr>
<tr><td align="center"><a href="https://github.com/matheusmota-dev-br/tikra/blob/main/corpus/08-graph-nodes.tex"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/08-graph-nodes.svg" alt="Graph with a bent edge" width="380"></a><br><sub>Graph with a bent edge</sub></td><td align="center"><a href="https://github.com/matheusmota-dev-br/tikra/blob/main/corpus/11-spring-decorations.tex"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/11-spring-decorations.svg" alt="zigzag, snake and random steps" width="380"></a><br><sub>zigzag, snake and random steps</sub></td></tr>
<tr><td align="center"><a href="https://github.com/matheusmota-dev-br/tikra/blob/main/corpus/16-bar-chart.tex"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/16-bar-chart.svg" alt="Bar chart from a foreach" width="380"></a><br><sub>Bar chart from a foreach</sub></td><td align="center"><a href="https://github.com/matheusmota-dev-br/tikra/blob/main/corpus/25-standalone-document.tex"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/25-standalone-document.svg" alt="standalone document, bend and labels" width="380"></a><br><sub>standalone document, bend and labels</sub></td></tr>
</table>

## Input

A full `standalone` document, a bare `tikzpicture`, an inline `\tikz`, a lone `axis` environment, or loose TikZ statements. The preamble is read for `\definecolor`, `\colorlet`, `\tikzset` and `\usetikzlibrary`; the rest is ignored.

## What it draws

See [`spec/subset.md`](https://github.com/matheusmota-dev-br/tikra/blob/main/spec/subset.md). In short: `\draw`, `\fill`, `\filldraw`, `\path`, `\node`, `\coordinate`, `\clip`, `\foreach` (several variables, `...`, `count`), `scope` with `rotate`/`shift`/`scale`/`x=`/`y=`, `\def`, `\pgfmathsetmacro`, `declare function`, `\tikzset` and styles, `\definecolor`, xcolor mixes, positioning (`below=of A`, `right=2cm of B`, `below left=1cm and 2cm of C`), `calc` (`$(A)!0.5!(B)$`), `--`, `|-`, `-|`, `to[out=,in=]`, `bend left/right`, `.. controls ..`, `rectangle`, `circle`, `ellipse`, `arc`, `grid`, `plot` (functions and coordinates, `smooth`), polar coordinates, nodes (`rectangle`, `circle`, `ellipse`, `diamond`) with `text width`, `align`, `minimum size`, `rounded corners`, anchors, `label`, nodes in the middle of a segment, arrow tips (`->`, `->>`, `-Latex`, `-Stealth`, `|`, `o`), dashes, line widths, opacity, hatching (`pattern=…`), the `zigzag`, `snake` and `random steps` decorations, and pgfplots basics (`axis`, `\addplot` of an expression or of coordinates, `only marks`, legend, axis labels, title).

Labels: node text is typeset by mathra, with `$…$` and `\(…\)` for math, `\textbf`, `\\` for line breaks and `\small`…`\Huge`. Labels on a path inherit the path's color. Math follows mathra's default locale (English).

## Warnings, not errors

What tikra does not understand becomes a **warning**, and the figure is drawn with the rest: `\matrix`, `\pic`, unknown commands and environments, TikZ escaped twice or for Markdown (undone), labels with broken LaTeX (recovered by mathra's tolerant mode). `--warnings` lists them; `Output::warnings` has them in the library. Options are not checked yet: an option tikra does not know (`ybar`, `xtick`, `fit`, `drop shadow`) is ignored without a warning. How often that changes the drawing is measured in [docs/conformance.md](https://github.com/matheusmota-dev-br/tikra/blob/main/docs/conformance.md).

Limits protect against hostile input: 100,000 statements per figure, 64 levels of nesting, 2,000 samples per plot, coordinates within ±100,000 cm, PNG up to 100 megapixels. Past a limit you get a clear error or a warning, never a hang.

## Benchmark

[`corpus/`](https://github.com/matheusmota-dev-br/tikra/tree/main/corpus) holds 25 hand-made figures that cover the features above. Reproduce with:

```console
$ cargo run --release -p tikra --example bench
```

One run on an Apple M5 (arm64), 15 runs per figure, median per figure:

| Figures | Without warnings | With warnings | Errors |
|---:|---:|---:|---:|
| 25 | 25 | 0 | 0 |

| Per figure | Mean | p95 |
|---|---:|---:|
| SVG | 0.12 ms | 0.26 ms |
| SVG + PNG at 300 dpi | 2.13 ms | 5.88 ms |

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
