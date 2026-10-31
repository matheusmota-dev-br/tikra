// Two versions of a figure on the figure itself: what stayed in grey, how it was in red, how it
// became in green. Elements are paired through the source map (`data-src`) and the lines the two
// versions share, so an added line does not mark everything after it as changed.
var tikraDiff = (() => {
  // the lines of two texts: which line of `b` each line of `a` stayed as (longest common
  // subsequence), from 1, and the script of the changes
  function lines(a, b) {
    const x = a.split("\n"), y = b.split("\n");
    const best = Array.from({ length: x.length + 1 }, () => new Uint32Array(y.length + 1));
    for (let i = x.length - 1; i >= 0; i--) {
      for (let j = y.length - 1; j >= 0; j--) {
        best[i][j] = x[i] === y[j] ? best[i + 1][j + 1] + 1 : Math.max(best[i + 1][j], best[i][j + 1]);
      }
    }
    const kept = new Map(), script = [];
    let i = 0, j = 0;
    while (i < x.length || j < y.length) {
      if (i < x.length && j < y.length && x[i] === y[j]) {
        kept.set(i + 1, j + 1);
        script.push([" ", x[i]]);
        i++; j++;
      } else if (j < y.length && (i === x.length || best[i][j + 1] >= best[i + 1][j])) {
        script.push(["+", y[j++]]);
      } else {
        script.push(["-", x[i++]]);
      }
    }
    return { kept, script };
  }

  // the place a span of the old code has in the new one, if all its lines stayed, in a row
  function moved(src, kept) {
    const [a, b, c, d] = src.match(/\d+/g).map(Number);
    if (!kept.has(a) || !kept.has(c)) return null;
    for (let l = a; l <= c; l++) if (kept.get(l) !== kept.get(a) + l - a) return null;
    return `${kept.get(a)}:${b}-${kept.get(c)}:${d}`;
  }

  // an element as a text to compare: without its number and place, ids that shift when something
  // is added before them, and coordinates seen from the picture's origin, since a figure that
  // grows to the left moves everything in the SVG
  function shape(html, [ox, oy]) {
    const at = (x, y) => `${(Number(x) - ox).toFixed(3)} ${(Number(y) - oy).toFixed(3)}`;
    return html
      .replace(/ data-(step|src)="[^"]*"/g, "")
      .replace(/url\(#[^)]*\)/g, "url()")
      .replace(/ d="([^"]*)"/, (_, d) => ` d="${d.replace(/(-?[\d.]+) (-?[\d.]+)/g, (__, x, y) => at(x, y))}"`)
      .replace(/transform="translate\((-?[\d.]+) (-?[\d.]+)\)/, (_, x, y) => `transform="translate(${at(x, y)})`)
      .replace(/transform="matrix\((\S+ \S+ \S+ \S+) (-?[\d.]+) (-?[\d.]+)\)/, (_, m, x, y) => `transform="matrix(${m} ${at(x, y)})`);
  }

  // the same shape: numbers within what the SVG's three decimals round away
  function alike(a, b) {
    const x = a.split(/(-?\d+\.?\d*)/), y = b.split(/(-?\d+\.?\d*)/);
    return x.length === y.length && x.every((t, i) => (i % 2 ? Math.abs(t - y[i]) <= 0.002 : t === y[i]));
  }

  // what happened to each element: `before` and `after` are [{src, html}], `origin` each one's
  // picture origin. "same" elements are drawn once; "old" and "new" are what changed.
  function compare(before, after) {
    const { kept, script } = lines(before.code, after.code);
    const bySrc = new Map();
    after.elements.forEach((el, i) => {
      if (!bySrc.has(el.src)) bySrc.set(el.src, []);
      bySrc.get(el.src).push(i);
    });
    const newState = after.elements.map(() => "new");
    const seen = new Map();
    const oldState = before.elements.map((el) => {
      const k = seen.get(el.src) ?? 0;
      seen.set(el.src, k + 1);
      const place = moved(el.src, kept);
      const twin = place === null ? undefined : bySrc.get(place)?.[k];
      if (twin === undefined || !alike(shape(el.html, before.origin), shape(after.elements[twin].html, after.origin))) return "old";
      newState[twin] = "same";
      return "same";
    });
    return { oldState, newState, script };
  }

  const read = (code, r, svg) => ({
    code, origin: r.origin,
    elements: [...svg.querySelectorAll("[data-src]")].map((el) => ({ src: el.dataset.src, html: el.outerHTML })),
  });

  function tint(el, color) {
    for (const x of [el, ...el.querySelectorAll("*")]) {
      if (x.hasAttribute("stroke") && x.getAttribute("stroke") !== "none") x.setAttribute("stroke", color);
      if (x.getAttribute("fill") !== "none") x.setAttribute("fill", color);
      if (x.hasAttribute("color")) x.setAttribute("color", color);
    }
  }

  // ids of one SVG apart from the other's, but the glyphs', which are the same in both
  const apart = (svg) => svg.replace(/ id="(tr-[^g"][^"]*|[a-z]\d+)"/g, ' id="old-$1"').replace(/url\(#/g, "url(#old-");

  const parser = () => document.createElement("div");

  // the two versions on one stage, over each other, in the box both fit in
  function stage(before, after) {
    const holder = parser();
    holder.innerHTML = `<div class="diff-stage">${apart(before.svg)}${after.svg}</div>`;
    const [oldSvg, newSvg] = holder.querySelectorAll("svg");
    const { oldState, newState, script } = compare(read(before.code, before, oldSvg), read(after.code, after, newSvg));
    [...oldSvg.querySelectorAll("[data-src]")].forEach((el, i) => {
      if (oldState[i] === "same") el.setAttribute("visibility", "hidden");
      else tint(el, "#C8102E");
    });
    [...newSvg.querySelectorAll("[data-src]")].forEach((el, i) => tint(el, newState[i] === "same" ? "#B9B9B9" : "#1F7A4D"));
    const a = oldSvg.viewBox.baseVal, b = newSvg.viewBox.baseVal;
    const dx = after.origin[0] - before.origin[0], dy = after.origin[1] - before.origin[1];
    const left = Math.min(a.x + dx, b.x), top = Math.min(a.y + dy, b.y);
    const right = Math.max(a.x + a.width + dx, b.x + b.width), bottom = Math.max(a.y + a.height + dy, b.y + b.height);
    const size = [right - left, bottom - top];
    oldSvg.setAttribute("viewBox", [left - dx, top - dy, ...size].join(" "));
    newSvg.setAttribute("viewBox", [left, top, ...size].join(" "));
    for (const s of [oldSvg, newSvg]) { s.removeAttribute("width"); s.removeAttribute("height"); s.setAttribute("aria-hidden", "true"); }
    holder.firstChild.style.aspectRatio = `${size[0]} / ${size[1]}`;
    return { node: holder.firstChild, script };
  }

  return { lines, moved, shape, alike, compare, stage };
})();
if (typeof module !== "undefined") module.exports = tikraDiff;
