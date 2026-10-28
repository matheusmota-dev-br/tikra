# Changelog

What changed in each release of tikra: the crates `tikra` and `tikra-cli`, `@mathlabs/tikra` on npm and `tikra` on PyPI, which are released together. The site shows the same, with the figures drawn live: [the project's evolution](https://matheusmota-dev-br.github.io/tikra/changelog.html).

Each release has a headline (the line under its title), what changed, and the figures of [`corpus/`](corpus) that show it (`Figures:`), drawn by the site with today's tikra. `scripts/release.sh` will not release a version without its section here, and makes the GitHub release from it.

## 1.4.1 - 2026-10-27

The engine opens to libraries from outside tikra.

- `tikra::extension`: a package drawn on TikZ can live in a crate of its own, as TeX definitions run once and keys written in Rust, without a copy of the engine. `tikra::convert_with_libraries` converts with them defined next to TikZ's own.
- TikZ's circuit libraries go through the same opening, which shows it is enough for them; they draw as before.
- The engine's internal modules are hidden from the documentation: they are not an API and change with any release. What is kept stable is `convert`, `png` and `extension`.
- Nothing changes for the CLI, npm and PyPI.

**Matches TeX:** 2365 of 2502 figures of the manuals (94.5%).

## 1.4.0 - 2026-10-27

The figure draws itself, in the order TikZ drew it.

- The construction animation: `--animate`, `{ animate: true }`, `animate=True`. Each stroke is drawn along its length and its fill follows, all in about six seconds; off by default, and the SVG without it is unchanged byte for byte.
- The steps apart from the animation: `--steps`, `{ steps: true }`, `steps=True` number each element by the order TikZ drew it, for a page that walks through the figure.
- The playground walks through the figure step by step, gives a slider to each number the code defines (`\def`, `\pgfmathsetmacro`, `\newcommand`), and downloads the animation as a GIF, a video or an animated SVG with its own controls.
- The gallery animates every figure, and figures with numbers to change have their sliders.
- Five figures in the corpus with numbers to change, a topic of their own; the corpus has 241 figures.

**Matches TeX:** 2365 of 2502 figures of the manuals (94.5%).

Figures: `sine-wave-controls` `polar-rose-petals` `lissajous-figure` `regular-polygon-sides` `projectile-launch`

## 1.3.2 - 2026-10-27

How tikra grew, on the site and in this changelog.

- The site's [evolution page](https://matheusmota-dev-br.github.io/tikra/changelog.html): every release with its figures drawn live, and the conformance over time as a chart tikra draws.
- This changelog, from 1.0.0 on; each release on GitHub is made from it.
- The playground's editor has a maximum height and scrolls inside: a figure with a lot of data, such as the Lorenz attractor, no longer stretches the page ([#17](https://github.com/matheusmota-dev-br/tikra/issues/17)).
- pgf's and pgfplots' key definitions written anew, clean-room, with credits to the original authors.

**Matches TeX:** 2365 of 2502 figures of the manuals (94.5%).

## 1.3.1 - 2026-10-26

Molecules and commutative diagrams inside the drawing.

- The bonds of a molecule in a label take the label's color, as its letters do.
- Lines of a label too tall for the distance between baselines are a `\lineskip` apart, as in TeX.
- A function declared with `\pgfmathdeclarefunction` is plotted, and a plot that fails says why.
- Three figures in the gallery that only tikra draws: molecules and commutative diagrams in nodes.

**Matches TeX:** 2365 of 2502 figures of the manuals (94.5%).

Figures: `methylxanthines` `adjunctions` `substitution-energy-profile`

## 1.3.0 - 2026-10-26

Our 100% at 99.6%: of the figures a converter can reproduce, almost every one is drawn as TeX draws it.

- Snapshots of the system layer's animations, and of `whom:` with several attributes.
- pgfplots computes as pgf's fpu does: `exp`, square roots, log ticks and log samples.
- `shader=interp` shades smoothly, without seams, in surfaces and matrix plots; `\addplot graphics` draws a PNG image.
- Rounded corners as pgf makes them, curves included; Smith charts, callouts and patterns as pgf draws them.
- A path filled and stroked at one opacity is painted as one group, as in PDF.
- Labels set by mathra 1.5.0.

**Matches TeX:** 2365 of 2502 figures of the manuals (94.5%), up from 93.1%.

Figures: `clock-snapshots` `smooth-shaded-surface` `interpolated-heat-map` `matrix-with-moustaches` `rounded-corners-on-curves` `smith-chart-scaled`

## 1.2.1 - 2026-10-25

The 60-second demo.

- The [demo video](https://youtu.be/vTX69CwtjB4) in the README and on the site.
- The GitHub Sponsors button on the repository.

**Matches TeX:** 2330 of 2502 figures of the manuals (93.1%).

## 1.2.0 - 2026-10-24

The figure in words, for readers who cannot see it.

- Every figure comes with its alt text and a description of what it shows (axes, data, nodes and their connections), written into the SVG's `<title>` and `<desc>`.
- In 16 languages: `render()` in JavaScript and Python, `--describe` and `--locale` in the terminal.
- The playground tells the figure in words.

**Matches TeX:** 2330 of 2502 figures of the manuals (93.1%).

Figures: `petersen-graph` `normal-distribution`

## 1.1.1 - 2026-10-24

Every registry shows the new README.

- The READMEs of crates.io, npm and PyPI up to date, with famous figures as examples.

**Matches TeX:** 2330 of 2502 figures of the manuals (93.1%).

Figures: `golden-spiral`

## 1.1.0 - 2026-10-24

An engine that reads TikZ the way TikZ does, and almost every library of TikZ and pgfplots.

- A new engine: a small TeX that reads the figure as TeX reads it, pgfkeys, and pgf's basic layer.
- Almost every TikZ library: shapes, arrows, decorations, matrices, trees, graphs, mind maps, circuits, automata, calendars, fadings, shadings, patterns, spy, 3d and more.
- pgfplots: 2D and 3D axes, surfaces, colormaps and colorbars, polar, ternary and Smith charts, statistics, groupplots.
- Labels in Latin Modern Math, TeX's Computer Modern, at the widths of TeX's own fonts.
- The conformance report: every example of the TikZ and pgfplots manuals, drawn by TeX and by tikra and compared. 3.5% matched on its first day, October 20.
- The corpus grew from 25 figures to 227.

**Matches TeX:** 2330 of 2502 figures of the manuals (93.1%).

Figures: `lorenz-attractor` `nightingale-rose` `feynman-diagram` `periodic-table-corner` `surface-3d`

## 1.0.0 - 2026-10-21

The first release: TikZ and pgfplots figures to SVG and PNG, with no TeX installation.

- A subset of TikZ and pgfplots drawn in Rust, labels typeset by [mathra](https://github.com/matheusmota-dev-br/mathra).
- Everywhere at once: the `tikra` crate and command on crates.io, `@mathlabs/tikra` on npm (WebAssembly, for Node, bundlers and browsers) and `tikra` on PyPI, with the same API.
- The site: a playground, a gallery and the docs.
- A corpus of 25 hand-made figures.

Figures: `triangle-angles` `unit-circle` `flowchart` `bar-chart`
