const { test } = require("node:test");
const assert = require("node:assert/strict");
const { lines, moved, shape, alike, compare } = require("../diff.js");

const el = (src, d) => ({ src, html: `<path d="${d}" data-step="0" data-src="${src}"/>` });
const version = (code, elements, origin = [0, 0]) => ({ code, elements, origin });

test("the lines that stayed, and the script of the changes", () => {
  const { kept, script } = lines("a\nb\nc", "a\nx\nb\nc");
  assert.deepEqual([...kept], [[1, 1], [2, 3], [3, 4]]);
  assert.deepEqual(script.filter(([k]) => k !== " "), [["+", "x"]]);
});

test("a span moves with its lines, unless they were split or changed", () => {
  const { kept } = lines("a\nb\nc", "z\na\nb\nc");
  assert.equal(moved("2:3-3:9", kept), "3:3-4:9");
  assert.equal(moved("1:3-2:9", lines("a\nb\nc", "a\nz\nb\nc").kept), null);
  assert.equal(moved("1:1-1:2", lines("a", "b").kept), null);
});

test("a line added at the top marks nothing else as changed", () => {
  const before = version("one\ntwo", [el("1:1-1:3", "M1 1 L2 2"), el("2:1-2:3", "M3 3 L4 4")]);
  const after = version("new\none\ntwo", [el("1:1-1:3", "M9 9 L8 8"), el("2:1-2:3", "M1 1 L2 2"), el("3:1-3:3", "M3 3 L4 4")]);
  const { oldState, newState } = compare(before, after);
  assert.deepEqual(oldState, ["same", "same"]);
  assert.deepEqual(newState, ["new", "same", "same"]);
});

test("a changed line is the old element and the new one", () => {
  const { oldState, newState } = compare(version("a", [el("1:1-1:1", "M1 1")]), version("b", [el("1:1-1:1", "M2 2")]));
  assert.deepEqual(oldState, ["old"]);
  assert.deepEqual(newState, ["new"]);
});

test("a line that stayed but draws something else, as a macro it uses changed", () => {
  const before = version("r=1\ndraw", [el("2:1-2:4", "M1 1")]);
  const after = version("r=2\ndraw", [el("2:1-2:4", "M2 2")]);
  assert.deepEqual(compare(before, after).oldState, ["old"]);
});

test("a figure that grows to the left moves every coordinate, and nothing changed", () => {
  const before = version("one", [el("1:1-1:3", "M0.2 -0.2 L28.5 -0.2")], [0.2, -0.2]);
  const after = version("left\none", [el("1:1-1:4", "M0.2 0 L5 5"), el("2:1-2:3", "M85.2 -0.2 L113.5 -0.2")], [85.2, -0.2]);
  const { oldState, newState } = compare(before, after);
  assert.deepEqual(oldState, ["same"]);
  assert.deepEqual(newState, ["new", "same"]);
});

test("what rounding leaves does not count, a real change does", () => {
  assert.ok(alike('d="M-0.000 0.000 L85.040 0.000"', 'd="M0.000 0.000 L85.041 0.000"'));
  assert.ok(!alike('d="M0.000 0.000"', 'd="M0.000 0.100"'));
  assert.ok(!alike('stroke="#000000"', 'stroke="#ff0000"'));
});

test("the shape of a label is seen from the origin too, and ids do not count", () => {
  const a = '<g transform="translate(10 20)" fill="url(#c1)" data-step="1"><use href="#tr-g1"/></g>';
  const b = '<g transform="translate(15 20)" fill="url(#c9)" data-step="4"><use href="#tr-g1"/></g>';
  assert.ok(alike(shape(a, [0, 0]), shape(b, [5, 0])));
});
