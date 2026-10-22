<p align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/banner.svg" alt="tikra: TikZ to SVG and PNG, no TeX install" width="100%"></p>

# tikra

TikZ and pgfplots figures to SVG and PNG, without a TeX install: no LaTeX, no fonts on the system, no external processes. Math in labels is typeset by [mathra](https://github.com/matheusmota-dev-br/mathra). **[Try it in the browser](https://matheusmota-dev-br.github.io/tikra/)**.

```sh
pip install tikra
```

Wheels for Linux, macOS and Windows, for Python 3.9 and later; nothing else to install.

## Usage

```python
import tikra

svg = tikra.tikz_to_svg(r"""
\begin{tikzpicture}
  \draw[->] (0,0) -- (2,1) node[right] {$\vec v$};
\end{tikzpicture}
""")
open("v.svg", "w", encoding="utf-8").write(svg)
open("v.png", "wb").write(tikra.png(svg, dpi=300))

r = tikra.render(r"\tikz \draw (0,0) circle (1);")
r["warnings"]      # [] — what tikra skips or approximates is listed here, and the rest is drawn
```

| | |
|---|---|
| `tikz_to_svg(code)` | The SVG as a string. |
| `render(code)` | A dict: `svg` and `warnings` (a list of strings). |
| `png(svg, *, dpi=300, background=True)` | PNG bytes; `background=False` keeps it transparent instead of white. |

`code` is a full `standalone` document, a bare `tikzpicture`, an inline `\tikz`, an `axis` environment or loose TikZ statements. Errors (no figure in the input, an invalid dpi, a PNG past 100 megapixels) are `ValueError`. Types are included (`py.typed`).

## What it draws

Every image below is an SVG written by tikra from a figure in the repository's corpus, on a white card.

<table>
<tr><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/01-triangle-angles.svg" alt="Triangle with an angle arc and a height" width="380"><br><sub>Triangle with an angle arc and a height</sub></td><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/03-unit-circle.svg" alt="Unit circle with polar points" width="380"><br><sub>Unit circle with polar points</sub></td></tr>
<tr><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/05-pgfplots-legend.svg" alt="pgfplots: expressions, marks and a legend" width="380"><br><sub>pgfplots: expressions, marks and a legend</sub></td><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/07-flowchart.svg" alt="Flowchart with positioning" width="380"><br><sub>Flowchart with positioning</sub></td></tr>
<tr><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/11-spring-decorations.svg" alt="zigzag, snake and random steps" width="380"><br><sub>zigzag, snake and random steps</sub></td><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/16-bar-chart.svg" alt="Bar chart from a foreach" width="380"><br><sub>Bar chart from a foreach</sub></td></tr>
</table>

Paths, nodes and shapes, positioning and `calc`, `\foreach`, scopes and styles, xcolor, arrow tips, patterns, decorations and pgfplots basics. What tikra does not understand becomes a warning, never a failure: the full subset is in [spec/subset.md](https://github.com/matheusmota-dev-br/tikra/blob/main/spec/subset.md).

## Also for JavaScript, Rust and the terminal

The same core is an npm package (`@mathlabs/tikra`), a Rust crate and a command-line tool: [github.com/matheusmota-dev-br/tikra](https://github.com/matheusmota-dev-br/tikra#install-and-use).

## License

The code is MIT or Apache-2.0; the fonts embedded through mathra are Latin Modern Math, under the GUST Font License, and STIX Two Math, under the SIL Open Font License 1.1.
