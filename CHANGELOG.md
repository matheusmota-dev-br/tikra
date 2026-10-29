# Changelog

What changed in each release of tikra: the crates `tikra` and `tikra-cli`, `@mathlabs/tikra` on npm and `tikra` on PyPI, which are released together. The site shows the same, with the figures drawn live: [the project's evolution](https://matheusmota-dev-br.github.io/tikra/changelog.html).

Each release has a headline (the line under its title), what changed, and the figures of [`corpus/`](corpus) that show it (`Figures:`), drawn by the site with today's tikra. `scripts/release.sh` will not release a version without its section here, and makes the GitHub release from it.

## Unreleased

- `\the<counter>` of a `\newcounter` is the counter's value, as LaTeX prints it: in an expression (`(\thea/100*360:1)`), a color (`red!\thedensity`) and a node's text (#66).
- `\pgfmathsetcounter`, `\pgfmathaddtocounter` and `\pgfmathaddtolength`, as pgfmath does them: the value truncated toward zero for a counter, set locally (a `\foreach` body keeps it to itself, unlike `\setcounter`). A `\whiledo` on a counter they move ends (#67).
- `\pgfmathsetlength` reads a plain number as points, as pgfmath does: `2cm+3` is 59.9pt, not 5cm (#68).

## 1.7.0 - 2026-10-29

Figures you drag by their points, labels kerned as TeX kerns them, and cmyk colors as TeX's PDF shows them.

- In the playground, each point the code names gets a handle on the figure: dragging it rewrites the numbers that move it, in the code. A demo drags the figure by itself; "Interactive HTML" downloads the figure as one page that works offline, as does `scripts/interactive-html.mjs`. A range comment can give a step: `% -4..4:0.1` (#40).
- `Output::points`: every node and coordinate the author named, where it is in the SVG, and `Output::origin`, where the picture's (0,0) landed; both are in the wasm `render` JSON (#40).
- A color ending in `!` is read as xcolor reads it: `red!50!` is `red!50!white`, and `red!` is red (#41).
- The preview package's setup (`\PreviewEnvironment{tikzpicture}`, `\setlength\PreviewBorder{5pt}`) is no longer drawn as text or warned about (#42).
- pgfmath's `Mod` (never negative: `Mod(-7,3)` is 2) and `\pgfmathMod`, and a quoted string as the value of `?:` (`Mod(\k,2) ? "pink" : "purple!50"`), also in `\pgfmathsetmacro` (#43).
- A `rectangle callout` narrower than its pointer is drawn instead of stopping tikra (#44).
- `shift={(0,\R)}` reads a length register as a point on a path does (#45).
- In a `\matrix` cell, a named point is read in the cell's own coordinates, as TeX sets the cell before moving it into place: `(A)` from outside the matrix moves with the cell, and a node from another cell is where it was in that cell (#47).
- `\pagecolor{…}` (and `\pagecolor[model]{…}`) paints the page behind the figure, margin included, as TeX does, instead of being drawn as text; `\nopagecolor` takes it away (#50).
- `tikra::extension`: a shape's text can be turned around its point (`ShapeText::turn`, a label along a side) and kept upright when its node turns (`ShapeText::upright`, a turned chip's pin numbers) (#39).
- `left=30`, `above=.2`…: a bare number is in points, as TikZ reads it, and in the x and y vectors' units only with the `positioning` library (#52).
- Code that builds a `ShapeText` listing every field needs `..Default::default()` for the new ones (#39).
- Labels in Latin Modern are kerned as TeX kerns them: by the OT1 fonts (`cmr10`, `cmbx10`) by default and the T1 ones (`ec-lmr10`, `ec-lmbx10`) after `\usepackage[T1]{fontenc}` (`va`, `AV`, `To`) (#48; mathra#81). 3 more figures match TeX.
- LaTeX's `picture` environment in a text: `\begin{picture}(w,h)(x,y)` with `\put`, `\multiput`, `\line`, `\circle`, `\circle*`, `\thinlines`, `\thicklines` and `\linethickness`, in `\unitlength`, drawn as LaTeX draws it (#53).
- Images in a text: `\pgfdeclareimage[width=…, height=…]{name}{file}` and `\pgfuseimage{name}`, `\pgfimage` and `\includegraphics` in a node or `\pgftext`, a PNG file in a box of its size (#53).
- `\begingroup`, `\endgroup` and `\relax` in a text no longer leave the space after them.
- `\NewDocumentCommand` and its kin (`Renew`, `Provide`, `Declare`, `Expandable`): commands with arguments `m`, `o`, `O{…}`, `s`, `t`, `d`, `D`, `r` and `R`, and `\IfNoValueTF`, `\IfValueTF`, `\IfBooleanTF` and their `T` and `F` forms (#53).
- The `Jack Tap` arrow tip of CircuiTikZ: a step, then half a triangle, 0.3cm by 0.15cm whatever the line width; `swap`, `reversed`, `width`, `length`, `fill`, `open` and the line cap and join (#53).
- `\scalebox{0.7}{…}` (and `\scalebox{2}[1]`) around pictures in a document scales them, their lines and texts with them, as graphicx does.
- Colors in xcolor's cmyk (`cyan`, `magenta`, `yellow`, `olive`, the dvipsnames, `\definecolor{x}{cmyk}{…}`) are drawn as Ghostscript draws them from TeX's PDF, through its ICC profiles: `cyan` is #00aeef, `cyan!30!white` #abe1fa (`tools/cmyk/table.py`). Mixes are in their first color's model, as xcolor mixes them (`white!70!cyan` is cmyk, `red!50!cyan` rgb), and so are complements and the extended syntax's `cmyk,1:…` and `gray,1:…` (#53).
- `tikra::color::Color` has a `model`: code that builds one with `Color { r, g, b }` uses `Color::rgb(r, g, b)` (#53).
- `tikra::extension`: `Context::measure` runs code without drawing it and gives the box of what it drew, with what a closure read before its nodes are forgotten (a subcircuit placed by one of its anchors) (#53).
- Labels are set by mathra 1.8.0: inside text, `\hspace{…}` adds its space and `\phantom`, `\hphantom` and `\vphantom` leave the room of their argument, and text is kerned by TeX's OT1 or T1 fonts.
- Our 100% (the figures a converter can reproduce) is 2480 of 2486, down from 2481.

**Matches TeX:** 2370 of 2502 figures of the manuals (94.7%).

Figures: `inscribed-angle-theorem` `bezier-control-points` `parallelogram-rule`

## 1.6.0 - 2026-10-29

Labels in Times, fills without seams, and library shapes with texts and styles of their own.

- Labels in `\fontfamily{ptm}` are drawn in Times (TeX Gyre Termes, in its four styles), as wide as TeX sets them and kerned as TeX kerns them (#35).
- In bold text, `\emph` and `\textit` are bold italic, as in TeX, and the text after a formula stays bold (#35; mathra 1.7.1).
- Fills side by side no longer show a seam of the background along the edge they share: grids of cells, bars and faces of solids are solid (#35).
- A path filled and stroked at different opacities is drawn as PDF draws it: under a translucent stroke you see what is under the path, not its fill (#35).
- `tikra::extension`: a library shape's paths may have a color, a dash pattern and line caps of their own (#32). Its shapes may draw texts of their own, turned and mirrored with the node, in its size and its words (#33). A shape knows its node's name, its fill and line colors and its transformation, and a key knows whether `transform shape` is on and can define TeX as tokens (`Context::define_tex`) (#34).
- Code that builds a `ShapePath` or a `NodeShape` listing every field needs `..Default::default()` for the new ones (#36).
- Colors read xcolor's extended syntax (`rgb,1:red,1;green,0.5;blue,0`).
- Labels are set by mathra 1.7.1: the text after a formula keeps its bold, small capitals or monospace; `\phi` (and `\epsilon`, `\vartheta`, `\varrho`, `\varpi`) in math italic as TeX sets them; `\textsubscript`, `\textsuperscript`, `\ensuremath` and `\boldmath` inside text; siunitx ranges; graphicx's `\scalebox`, `\rotatebox`, `\resizebox` and `\raisebox`, drawn turned and scaled in the label.
- Our 100% (the figures a converter can reproduce) is 2481 of 2486, up from 2476.

**Matches TeX:** 2368 of 2502 figures of the manuals (94.6%).

Figures: `spotlight-clipping` `fading-directions`

## 1.5.0 - 2026-10-28

Node shapes drawn by a library.

- `tikra::extension`: a library declares node shapes with a Rust function (`Context::declare_shape`), computed for each node from its text box: its anchors, its border, where its text goes, its paths (stroked as wide as it says, filled with the node's fill, with its fill or white, or solid in its line color) and what else counts in the picture's size. One function may draw several shapes, told apart by name. They are nodes as any other: placed by their anchors (`anchor=`), turned with the node, with lines stopping at their border (#29).
- Drawing is unchanged: every figure is the same as 1.4.2's.

**Matches TeX:** 2365 of 2502 figures of the manuals (94.5%).

## 1.4.2 - 2026-10-28

How to contribute, and how to report a vulnerability.

- A code of conduct (Contributor Covenant 2.1), a contributing guide and a security policy, in `.github/`. Vulnerabilities are reported privately through the repository's Security tab.
- The checks run on the contributor's machine: `scripts/check.sh` runs fmt, clippy and the Rust, wasm, npm and Python tests, and the git hooks in `.githooks/` run it before each commit and push. GitHub runs them again on `main`, not on pull requests.
- Drawing is unchanged: every figure is the same as 1.4.1's.

**Matches TeX:** 2365 of 2502 figures of the manuals (94.5%).

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
