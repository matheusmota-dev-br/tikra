import { test } from "node:test";
import assert from "node:assert/strict";
import { init, tikzToSvg, render, slides, clickThrough, png, version } from "@mathlabs/tikra";

const FIGURE = String.raw`\begin{tikzpicture}\draw[->] (0,0) -- (1,1) node[right] {$x$};\end{tikzpicture}`;

test("works right away in Node, init is optional", async () => {
  assert.match(tikzToSvg(FIGURE), /^<svg/);
  await init();
});

test("every input form", () => {
  for (const code of [
    String.raw`\documentclass[tikz]{standalone}\begin{document}\begin{tikzpicture}\draw (0,0)--(1,0);\end{tikzpicture}\end{document}`,
    String.raw`\tikz \draw (0,0) circle (1);`,
    String.raw`\begin{axis}\addplot {x^2};\end{axis}`,
    String.raw`\draw (0,0) -- (1,1);`,
  ]) assert.match(tikzToSvg(code), /^<svg/);
});

test("render returns the svg and the warnings", () => {
  assert.deepEqual(render(FIGURE).warnings, []);
  const r = render(String.raw`\begin{tikzpicture}\pic {x}; \draw (0,0) -- (1,0);\end{tikzpicture}`);
  assert.match(r.svg, /^<svg/);
  assert.ok(r.warnings.length > 0);
  assert.equal(typeof r.warnings[0], "string");
});

test("render describes the figure in words", () => {
  const r = render(FIGURE);
  assert.equal(r.alt, "Drawing of one shape with the text “x”");
  assert.equal(r.description, "Text in the figure: x.");
  assert.equal(render(FIGURE, { locale: "pt-BR" }).description, "Texto na figura: x.");
  assert.match(tikzToSvg(FIGURE, { locale: "de" }), /Text in der Abbildung: x\./);
  assert.throws(() => render(FIGURE, { locale: "xx" }), /unknown locale/);
});

test("errors are Error objects", () => {
  assert.throws(() => tikzToSvg("just text"), /no tikzpicture/);
  assert.throws(() => png("not svg"), Error);
  assert.throws(() => png(tikzToSvg(FIGURE), { dpi: 0 }), /invalid dpi/);
});

test("png", () => {
  const svg = tikzToSvg(FIGURE);
  const opaque = png(svg);
  assert.ok(opaque instanceof Uint8Array);
  assert.deepEqual([...opaque.slice(0, 4)], [0x89, 0x50, 0x4e, 0x47]);
  assert.notDeepEqual(png(svg, { background: false }), opaque);
  assert.ok(png(svg, { dpi: 72 }).length < opaque.length);
});

test("version", () => {
  assert.match(version(), /^\d+\.\d+\.\d+/);
});

test("the source map is asked for apart", () => {
  assert.doesNotMatch(tikzToSvg(FIGURE), /data-src/);
  assert.match(tikzToSvg(FIGURE, { source: true }), / data-src="1:/);
});

test("the parts come with their words", () => {
  const r = render(FIGURE, { parts: true });
  assert.match(r.svg, /role="group" tabindex="0"/);
  assert.ok(r.parts.length > 0 && r.parts[0].text);
  assert.equal(render(FIGURE).parts.length, 0);
});

test("the slides of a figure", () => {
  const figure = String.raw`\begin{tikzpicture}\draw (0,0) -- (1,0); \only<2>{\draw (0,1) -- (1,1);}\end{tikzpicture}`;
  assert.equal(slides(figure).length, 2);
  assert.match(clickThrough(figure), /data-slides="2"/);
});

test("the animations library played", () => {
  const figure = String.raw`\usetikzlibrary{animations}\tikz \fill[red] :fill = {0s="red", 1s="blue"} (0,0) circle (1pt);`;
  assert.match(tikzToSvg(figure, { play: true }), /<animate /);
  assert.doesNotMatch(tikzToSvg(figure), /<animate/);
});

test("each point of a plot an element of its own", () => {
  const figure = String.raw`\begin{tikzpicture}\begin{axis}[ybar]\addplot coordinates {(1,2) (2,5)};\end{axis}\end{tikzpicture}`;
  assert.match(tikzToSvg(figure, { points: true }), /data-x="2" data-y="5"/);
  assert.doesNotMatch(tikzToSvg(figure), /tr-point/);
});
