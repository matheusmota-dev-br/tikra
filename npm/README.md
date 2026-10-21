<p align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/banner.svg" alt="tikra: TikZ to SVG and PNG, no TeX install" width="100%"></p>

# @mathlabs/tikra

TikZ and pgfplots figures to SVG and PNG, without a TeX install: no LaTeX, no fonts on the system, no external processes. Math in labels is typeset by [mathra](https://github.com/matheusmota-dev-br/mathra). **[Try it in the browser](https://matheusmota-dev-br.github.io/tikra/)**.

```sh
npm install @mathlabs/tikra
```

## What it draws

Every image below is an SVG written by tikra from a figure in the repository's corpus, on a white card.

<table>
<tr><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/01-triangle-angles.svg" alt="Triangle with an angle arc and a height" width="380"><br><sub>Triangle with an angle arc and a height</sub></td><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/03-unit-circle.svg" alt="Unit circle with polar points" width="380"><br><sub>Unit circle with polar points</sub></td></tr>
<tr><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/05-pgfplots-legend.svg" alt="pgfplots: expressions, marks and a legend" width="380"><br><sub>pgfplots: expressions, marks and a legend</sub></td><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/07-flowchart.svg" alt="Flowchart with positioning" width="380"><br><sub>Flowchart with positioning</sub></td></tr>
<tr><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/11-spring-decorations.svg" alt="zigzag, snake and random steps" width="380"><br><sub>zigzag, snake and random steps</sub></td><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/tikra/main/docs/examples/16-bar-chart.svg" alt="Bar chart from a foreach" width="380"><br><sub>Bar chart from a foreach</sub></td></tr>
</table>

## Node

Works right away:

```js
import { tikzToSvg, render, png } from "@mathlabs/tikra";      // or: const { tikzToSvg } = require("@mathlabs/tikra")
import { writeFileSync } from "node:fs";

const svg = tikzToSvg(String.raw`\begin{tikzpicture}\draw[->] (0,0) -- (2,1) node[right] {$\vec v$};\end{tikzpicture}`);
writeFileSync("v.svg", svg);
writeFileSync("v.png", png(svg, { dpi: 300 }));
const { warnings } = render(String.raw`\tikz \draw (0,0) circle (1);`);   // []
```

## Browsers and bundlers (Vite, webpack, esbuild)

Await `init()` once; it fetches `tikra.wasm` next to the module:

```js
import { init, tikzToSvg } from "@mathlabs/tikra";

await init();
document.querySelector("#figure").innerHTML = tikzToSvg(String.raw`\tikz \draw (0,0) circle (1);`);
```

Without a bundler, from a CDN:

```html
<script type="module">
  import { init, tikzToSvg } from "https://cdn.jsdelivr.net/npm/@mathlabs/tikra/web/index.js";
  await init();
  document.body.innerHTML = tikzToSvg(String.raw`\tikz \draw[->] (0,0) -- (1,1);`);
</script>
```

## API

| | |
|---|---|
| `init(input?)` | Loads the wasm. Needed once in browsers and bundlers; does nothing in Node. `input` overrides where the wasm comes from (a URL, a `Response`, bytes or a `WebAssembly.Module`). |
| `tikzToSvg(code)` | The SVG as a string. |
| `render(code)` | `{ svg, warnings }`: `warnings` lists, as strings, what was ignored or approximated. |
| `png(svg, options?)` | PNG bytes as a `Uint8Array`. Options: `dpi` (300 by default) and `background` (`true`, white, by default; `false` for transparent). |
| `version()` | The version of tikra's core inside the package (the npm package has its own version). |

`code` is a full `standalone` document, a bare `tikzpicture`, an inline `\tikz`, an `axis` environment or loose TikZ statements. Errors (no figure in the input, an invalid dpi, a PNG past 100 megapixels) are thrown as `Error`.

## Warnings, not errors

What tikra does not understand becomes a warning, and the figure is drawn with the rest: `\matrix`, `\pic`, unknown commands and options, TikZ escaped twice or for Markdown (undone), labels with broken LaTeX. The full subset is in [spec/subset.md](https://github.com/matheusmota-dev-br/tikra/blob/main/spec/subset.md).

## Also for Python, Rust and the terminal

The same core is a Python module, a Rust crate and a command-line tool, and every one gives the same SVG, byte for byte. How to install each: [github.com/matheusmota-dev-br/tikra](https://github.com/matheusmota-dev-br/tikra#install-and-use).

## License

The code is MIT or Apache-2.0; the STIX Two Math font embedded through mathra is under the SIL Open Font License 1.1.
