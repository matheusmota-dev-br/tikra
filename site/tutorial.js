// The tutorials: lessons that run on the playground itself. Each step says what a control does,
// marks it on the page, and either waits for the reader to try it or does it for them.
var tikraTutorial = (() => {
  const SINE = String.raw`\begin{tikzpicture}
  \def\a{1}   % amplitude, 0..2
  \def\k{1}   % frequency, 0..3
  \draw[help lines, step=0.5] (-0.2,-2.2) grid (4.2,2.2);
  \draw[->] (-0.2,0) -- (4.3,0) node[right] {$x$};
  \draw[->] (0,-2.2) -- (0,2.4) node[above] {$y$};
  \draw[blue, thick, domain=0:4, samples=100] plot (\x, {\a*sin(\k*deg(\x))});
  \node[blue, right] at (4.3,1.8) {$y = a \sin kx$};
\end{tikzpicture}`;
  const DIAGRAM = String.raw`\begin{tikzpicture}[box/.style={draw, circle, minimum size=1cm}]
  \node[box] (a) at (0,0) {Sun};
  \node[box] (b) at (4,0) {Earth};
  \node[box] (c) at (4,3) {Moon};
  \draw[->] (a) -- node[below] {light} (b);
  \draw[->] (b) -- (c);
  \draw[dashed] (0,3) -- (2,3);
\end{tikzpicture}`;
  const SLIDES = String.raw`\begin{tikzpicture}
  \draw[->, thick] (0,0) -- (5,0) node[right] {$x$};
  \draw[->, thick] (0,0) -- (0,3) node[above] {$y$};
  \pause
  \draw[blue, thick] (0,0.5) .. controls (2,3) and (3,0) .. (5,2.5);
  \uncover<3->{\node[above] at (2.5,2.5) {uncover keeps its room};}
  \only<4>{\fill[red] (2.5,1.4) circle (3pt);}
\end{tikzpicture}`;
  const ANIMATED = String.raw`\usetikzlibrary{animations}
\begin{tikzpicture}
  \path (-1,-1) rectangle (6,3);
  \draw[help lines] (0,0) grid (5,2);
  \fill[red] :fill = {0s="red", 2s="blue", repeats} :xshift = {0s="0cm", 2s="4cm", repeats} (0.5,1.5) circle (3mm);
  \fill[orange] :fill = {0s="orange", 1s="green", begin on=click, freeze} (3.5,0.2) rectangle (4.5,1);
\end{tikzpicture}`;
  const BARS = String.raw`\begin{tikzpicture}
  \begin{axis}[ybar, width=7cm, height=5cm, ymin=-3, bar width=14pt]
    \addplot[fill=blue!40] coordinates {(1,2) (2,5) (3,-2) (4,6)};
  \end{axis}
\end{tikzpicture}`;

  const q = (s) => document.querySelector(s);
  const LINE = "\\draw[blue, thick";

  const LESSONS = [
    {
      id: "code", title: "Figure and code", about: "Numbers you can drag, and what drew what.",
      start: (p) => p.begin(SINE, "figure"),
      steps: [
        { say: "Every number the code defines gets a slider. Drag <b>\\a</b>: the code and the figure change together.", mark: "#params",
          done: (p) => !p.tex.value.includes("\\def\\a{1}"), show: (p, h) => h.slide("#params input[type=range]", 1.7) },
        { say: "Each stroke knows the line that drew it. <b>Click the blue curve</b>, and its line is selected in the code.", mark: "#preview",
          done: (p) => p.tex.selectionEnd > p.tex.selectionStart && p.tex.value.slice(p.tex.selectionStart, p.tex.selectionEnd).includes(LINE),
          show: (p, h) => h.clickFigure('[stroke="#0000ff"]') },
        { say: "It works the other way too. <b>Put the cursor on the line of a <code>\\draw</code></b>, and the rest of the figure fades.", mark: "#tex",
          done: (p) => p.preview.classList.contains("traced"), show: (p, h) => h.cursorOn(LINE) },
      ],
    },
    {
      id: "build", title: "Build", about: "Walk through a figure, step by step.",
      start: (p) => p.begin(DIAGRAM, "build"),
      steps: [
        { say: "tikra knows the order the figure was drawn in. <b>Drag the slider to the left</b> to take it apart.", mark: "#step",
          done: () => Number(q("#step").value) < Number(q("#step").max), show: (p, h) => h.slide("#step", 2) },
        { say: "Under the slider, a caption says what the step drew. <b>Stop on a step with a node or a connection.</b> Free lines stay silent.", mark: "#step",
          done: () => q("#step-caption").textContent !== "", show: (p, h) => h.slide("#step", 3) },
        { say: "<b>Animate</b> plays it from the start. The Export menu turns the same thing into a GIF or a video.", mark: "#animate",
          done: () => q("#animate").textContent.includes("Stop"), show: (p, h) => h.click("#animate") },
      ],
    },
    {
      id: "compare", title: "Compare", about: "See what a change did to the figure.",
      start: (p) => p.begin(SINE, "figure"),
      steps: [
        { say: "Open <b>Compare</b>. The figure as it is now becomes the version to compare against.", mark: '.mode[data-mode="compare"]',
          done: (p) => p.mode() === "compare", show: (p, h) => h.click('.mode[data-mode="compare"]') },
        { say: "Now <b>drag \\a</b>. What did not change turns grey, how it was is red, and how it became is green.", mark: "#params",
          done: () => q("#version-note").textContent.includes("added"), show: (p, h) => h.slide("#params input[type=range]", 0.5) },
        { say: "<b>Before</b> and <b>After</b> show one version at a time. Try <b>Before</b>.", mark: "#version-before",
          done: () => q("#version-before").getAttribute("aria-pressed") === "true", show: (p, h) => h.click("#version-before") },
      ],
    },
    {
      id: "pen", title: "Pen", about: "A pen plotter draws the figure.",
      start: (p) => p.begin(SINE, "figure"),
      steps: [
        { say: "A pen plotter draws with a real pen, and knows only lines. Open <b>Pen</b>.", mark: '.mode[data-mode="pen"]',
          done: (p) => p.mode() === "pen", show: (p, h) => h.click('.mode[data-mode="pen"]') },
        { say: "Press <b>Draw with a pen</b>. The dot is the tip: small while drawing, bigger while lifted. Fills and letters become outlines.", mark: "#pen-play",
          done: () => !!q("#preview animateMotion"), show: (p, h) => h.click("#pen-play") },
        { say: "<b>Export</b> gives the drawing as a GIF, and the strokes as G-code or HPGL for a machine.", mark: "#export-toggle",
          done: () => !q("#export-menu").hidden, show: (p, h) => h.click("#export-toggle") },
      ],
    },
    {
      id: "keyboard", title: "Keyboard", about: "Walk a figure without a mouse.",
      start: (p) => p.begin(DIAGRAM, "figure"),
      steps: [
        { say: "A screen reader hears each part of a figure on its own. <b>Click a node in the figure</b>, or press Tab until the figure has the focus.", mark: "#preview",
          done: () => !!document.activeElement?.closest?.("[data-part]") || !!q("#preview .on"), show: (p, h) => h.focusPart(0) },
        { say: "With the focus on the figure, <b>the arrow keys</b> go from part to part, and each part's code lights up.", mark: "#preview",
          done: (p, seen) => seen.moved, show: (p, h) => h.focusPart(1) },
      ],
    },
    {
      id: "slides", title: "Slides", about: "Beamer's overlays: one figure, several slides.",
      start: async (p) => {
        await p.begin(SLIDES, "figure");
        p.show(p.clickThrough(SLIDES));
      },
      steps: [
        { say: "<code>\\pause</code>, <code>\\uncover&lt;3-&gt;</code> and <code>\\only&lt;4&gt;</code> come from Beamer. This is one figure with four slides. <b>Click the figure</b> to go to the next one.", mark: "#preview",
          done: () => { const g = q("#tr-slide-1"); return !!g && getComputedStyle(g).display === "none"; }, show: (p, h) => h.clickSvg("#trNext1") },
        { say: "From the terminal: <code>tikra figure.tex --slides -o figure</code> writes one SVG per slide, and <code>--click</code> writes this one.", mark: "#preview", free: true },
      ],
    },
    {
      id: "motion", title: "Animations", about: "TikZ's animations library, played.",
      start: async (p) => {
        await p.begin(ANIMATED, "figure");
        p.show(p.render(ANIMATED, { play: true }).svg);
      },
      steps: [
        { say: "TikZ's <code>animations</code> library is played here: the dot crosses by itself. The square waits for you, with <code>begin on=click</code>. <b>Click it.</b>", mark: "#preview",
          done: () => { const r = q('#preview path[fill="#ff8000"]'); return !!r && getComputedStyle(r).fill !== "rgb(255, 128, 0)"; },
          show: (p, h) => h.clickSvg('#preview g:last-of-type[id^="trAnim"]') },
        { say: "From the terminal: <code>tikra figure.tex --play -o figure.svg</code>. A PNG stays still, and without <code>--play</code> so does the SVG.", mark: "#preview", free: true },
      ],
    },
    {
      id: "points", title: "Plot values", about: "Each bar with its numbers.",
      start: async (p) => {
        await p.begin(BARS, "figure");
        p.show(p.render(BARS, { points: true }).svg);
      },
      steps: [
        { say: "Each bar is an element of its own, with its values. <b>Point at a bar</b>, or reach it with Tab.", mark: "#preview",
          done: () => [...document.querySelectorAll("#preview .tr-tip")].some((t) => getComputedStyle(t).visibility === "visible"), show: (p, h) => h.focusPoint(1) },
        { say: "With the construction animation, the bars grow from the axis. Press <b>Grow them</b> below.", mark: "#preview", free: true,
          action: { label: "Grow them", run: (p) => p.show(p.render(BARS, { points: true, animate: true }).svg) } },
      ],
    },
  ];

  const STORE = "tikra-tutorials";
  const remembered = () => { try { return new Set(JSON.parse(localStorage.getItem(STORE) ?? "[]")); } catch { return new Set(); } };
  const remember = (set) => { try { localStorage.setItem(STORE, JSON.stringify([...set])); } catch { /* the page works without it */ } };

  function mount(p) {
    const panel = q("#tutor");
    const calm = matchMedia("(prefers-reduced-motion: reduce)").matches;
    const hand = Object.assign(document.createElement("div"), { className: "tutor-hand", hidden: true });
    document.body.append(hand);
    const finished = remembered();
    let lesson = null, at = 0, timer = 0, token = 0;
    const seen = { moved: false };
    const sleep = (ms) => new Promise((r) => setTimeout(r, calm ? 0 : ms));

    // where the page's own controls are clicked and dragged, with a hand the reader can follow
    async function point(el, x, y) {
      // asked to show it, the page may bring it into view; only then
      const box = el.getBoundingClientRect();
      if (box.top < 0 || box.bottom > innerHeight) {
        el.scrollIntoView({ block: "center", behavior: "instant" });
        await sleep(60);
        x = 0;
      }
      if (!x) { const r = el.getBoundingClientRect(); x = r.left + r.width / 2; y = r.top + r.height / 2; }
      hand.hidden = false;
      hand.style.transform = `translate(${x}px, ${y}px)`;
      await sleep(450);
    }
    const helper = (run) => ({
      async click(s) { const el = q(s); await point(el); if (run()) el.click(); },
      async clickSvg(s) {
        const el = q(s); await point(el);
        if (run()) el.dispatchEvent(new MouseEvent("click", { bubbles: true, view: window }));
      },
      async clickFigure(s) {
        const el = q(`#preview ${s}`); const len = el.getTotalLength?.() ?? 0;
        const m = el.getScreenCTM(), c = el.getPointAtLength(len / 3);
        const x = m.a * c.x + m.e, y = m.d * c.y + m.f;
        await point(el, x, y);
        if (run()) p.preview.dispatchEvent(new MouseEvent("click", { bubbles: true, clientX: x, clientY: y }));
      },
      async cursorOn(text) {
        await point(p.tex);
        if (!run()) return;
        p.tex.focus();
        const i = p.tex.value.indexOf(text);
        p.tex.setSelectionRange(i + 3, i + 3);
        p.tex.dispatchEvent(new MouseEvent("click", { bubbles: true }));
      },
      async slide(s, to) {
        const el = q(s); await point(el);
        const from = Number(el.value), n = calm ? 1 : 24;
        for (let k = 1; k <= n && run(); k++) {
          el.value = from + (to - from) * k / n;
          el.dispatchEvent(new Event("input", { bubbles: true }));
          await sleep(25);
        }
        if (run()) el.dispatchEvent(new Event("change", { bubbles: true }));
      },
      async focusPart(k) {
        const part = document.querySelectorAll("#preview [data-part][tabindex]")[k];
        await point(part);
        if (!run()) return;
        if (k === 0) part.focus();
        else (document.activeElement.closest?.("[data-part]") ?? part).dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true }));
      },
      async focusPoint(k) {
        const bar = document.querySelectorAll("#preview .tr-point")[k];
        await point(bar);
        if (run()) bar.focus();
      },
    });

    // a nudge on the control a step is about, once: the page does not move, and nothing stays lit
    function nudge(selector) {
      const el = selector ? q(selector) : null;
      if (!el) return;
      el.classList.remove("tutor-nudge");
      void el.getBoundingClientRect();
      el.classList.add("tutor-nudge");
      el.addEventListener("animationend", () => el.classList.remove("tutor-nudge"), { once: true });
    }
    function stop() {
      clearInterval(timer);
      token += 1;
      hand.hidden = true;
    }
    function close() {
      stop();
      lesson = null;
      panel.hidden = true;
      q("#tutor-open")?.focus();
    }
    function list() {
      stop();
      lesson = null;
      panel.hidden = false;
      panel.innerHTML = `<div class="tutor-head"><h3>Tutorials</h3><button class="tutor-x" type="button" aria-label="Close the tutorials">×</button></div>
        <ul class="tutor-list">${LESSONS.map((l) => `<li><button type="button" data-lesson="${l.id}"><b>${finished.has(l.id) ? "✓ " : ""}${l.title}</b><span>${l.about}</span></button></li>`).join("")}</ul>`;
    }
    async function open(l) {
      stop();
      lesson = l;
      at = 0;
      seen.moved = false;
      panel.innerHTML = `<div class="tutor-head"><h3>${l.title}</h3></div><p class="tutor-say">Getting the figure ready…</p>`;
      await l.start(p);
      if (lesson === l) step();
    }
    function step() {
      stop();
      const s = lesson.steps[at], last = at === lesson.steps.length - 1;
      nudge(s.mark);
      panel.innerHTML = `<div class="tutor-head"><h3>${lesson.title}</h3><span class="tutor-count">${at + 1} of ${lesson.steps.length}</span><button class="tutor-x" type="button" aria-label="Close the tutorials">×</button></div>
        <p class="tutor-say">${s.say}</p>
        <p class="tutor-state" id="tutor-state" aria-live="polite">${s.free ? "" : "Your turn."}</p>
        <div class="tutor-actions">
          <button class="button" type="button" data-go="list">All</button>
          ${at ? '<button class="button" type="button" data-go="back">Back</button>' : ""}
          ${s.show ? '<button class="button" type="button" data-go="show">Show me</button>' : ""}
          ${s.action ? `<button class="button" type="button" data-go="action">${s.action.label}</button>` : ""}
          <button class="button${s.free ? " solid" : ""}" id="tutor-next" type="button" data-go="next">${last ? "Finish" : "Next"}</button>
        </div>`;
      if (s.free) return;
      timer = setInterval(() => {
        if (!s.done(p, seen)) return;
        clearInterval(timer);
        hand.hidden = true;
        q("#tutor-state").textContent = "Done.";
        q("#tutor-next").classList.add("solid");
      }, 150);
    }
    panel.addEventListener("click", (e) => {
      if (e.target.closest(".tutor-x")) return close();
      const l = e.target.closest("[data-lesson]");
      if (l) return void open(LESSONS.find((x) => x.id === l.dataset.lesson));
      const go = e.target.closest("[data-go]")?.dataset.go;
      if (go === "list") list();
      if (go === "back") { at -= 1; step(); }
      if (go === "next") {
        if (at < lesson.steps.length - 1) { at += 1; step(); return; }
        finished.add(lesson.id);
        remember(finished);
        list();
      }
      if (go === "action") lesson.steps[at].action.run(p);
      if (go === "show") {
        const mine = ++token;
        lesson.steps[at].show(p, helper(() => mine === token)).finally(() => { if (mine === token) hand.hidden = true; });
      }
    });
    p.preview.addEventListener("keydown", (e) => { if (e.key.startsWith("Arrow")) seen.moved = true; });
    document.addEventListener("keydown", (e) => { if (e.key === "Escape" && !panel.hidden) close(); });
    q("#tutor-open").onclick = list;
    for (const link of document.querySelectorAll("[data-tutorials]")) link.addEventListener("click", list);
    return { list, close };
  }

  return { LESSONS, mount };
})();
if (typeof module !== "undefined") module.exports = tikraTutorial;
