// The figure and its code, tied by tikra's source map (`data-src`): a click on the figure selects
// the statement that drew it, and the cursor on a line of code picks out what that line drew.
var tikraSource = (() => {
  // "line:column-line:column", from 1, both ends included
  const parse = (src) => src.match(/\d+/g).map(Number);

  const lineStarts = (code) => {
    const starts = [0];
    for (let i = code.indexOf("\n"); i >= 0; i = code.indexOf("\n", i + 1)) starts.push(i + 1);
    return starts;
  };

  // the [start, end) of a span in the code
  function range(code, src) {
    const [a, b, c, d] = parse(src), starts = lineStarts(code);
    return [starts[a - 1] + b - 1, starts[c - 1] + d];
  }

  // the spans at an offset of the code; between statements, those on its line
  function at(code, offset, spans) {
    const inside = spans.filter((s) => { const [a, b] = range(code, s); return a <= offset && offset <= b; });
    if (inside.length) return inside;
    const line = lineStarts(code).filter((s) => s <= offset).length;
    return spans.filter((s) => { const [a, , c] = parse(s); return a <= line && line <= c; });
  }

  // the element under a point, or a stroke a few pixels away: lines are thin to click
  function near(x, y, root) {
    for (let r = 0; r <= 6; r++) {
      for (let k = 0; k < (r ? 8 : 1); k++) {
        const hit = document.elementFromPoint(x + r * Math.cos(k * Math.PI / 4), y + r * Math.sin(k * Math.PI / 4));
        const el = hit && root.contains(hit) ? hit.closest("[data-src]") : null;
        if (el) return el;
      }
    }
    return null;
  }

  // the figure's parts as one stop of the Tab key: the arrows walk between them
  function walk(root) {
    root.querySelectorAll("[data-part][tabindex]").forEach((el, i) => { el.tabIndex = i ? -1 : 0; });
  }

  function attach(preview, tex) {
    const elements = () => [...preview.querySelectorAll("[data-src]")];
    function trace(spans) {
      const on = new Set(spans);
      preview.classList.toggle("traced", on.size > 0);
      for (const el of elements()) el.classList.toggle("on", on.has(el.dataset.src));
    }
    const clear = () => trace([]);
    function follow() {
      const spans = [...new Set(elements().map((el) => el.dataset.src))];
      trace(spans.length ? at(tex.value, tex.selectionStart, spans) : []);
    }
    preview.addEventListener("keydown", (e) => {
      const by = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 }[e.key];
      const parts = [...preview.querySelectorAll("[data-part][tabindex]")];
      const at = parts.indexOf(e.target);
      if (!by || at < 0) return;
      e.preventDefault();
      const next = parts[(at + by + parts.length) % parts.length];
      e.target.tabIndex = -1;
      next.tabIndex = 0;
      next.focus();
    });
    preview.addEventListener("focusin", (e) => {
      const el = e.target.matches?.("[data-part]") ? e.target.querySelector("[data-src]") : null;
      if (el) trace([el.dataset.src]);
    });
    preview.addEventListener("focusout", clear);
    preview.addEventListener("click", (e) => {
      const el = near(e.clientX, e.clientY, preview);
      if (!el) return;
      tex.focus();
      tex.setSelectionRange(...range(tex.value, el.dataset.src));
      trace([el.dataset.src]);
    });
    tex.addEventListener("click", follow);
    tex.addEventListener("keyup", (e) => { if (e.key.startsWith("Arrow") || e.key === "Home" || e.key === "End") follow(); });
    tex.addEventListener("keydown", (e) => { if (e.key === "Escape") clear(); });
    tex.addEventListener("input", clear);
    tex.addEventListener("blur", clear);
    return { clear };
  }

  return { attach, at, range, walk };
})();
