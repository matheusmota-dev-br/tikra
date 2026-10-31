// The figure at the top of the site: the Lorenz attractor of 1963, drawn by tikra again on every
// move. The page integrates the equations and writes the TikZ; tikra does the drawing. Two
// trajectories start a hair apart, stay together and then part: the butterfly effect.
var tikraLorenz = (() => {
  const SIGMA = 10, BETA = 8 / 3;
  const DT = 0.01, END = 40, STEPS = END / DT, EVERY = 1; // seconds of the system, and one point in EVERY kept
  const SCALE = 0.135, MIDDLE = 24; // the attractor on the figure: z around 24 goes to the center
  const VIEW = [-0.55, 0.6]; // azimuth and elevation, in radians: both wings in sight

  const flow = ([x, y, z], rho) => [SIGMA * (y - x), x * (rho - z) - y, x * y - BETA * z];

  // the path from `start`, by fourth-order Runge-Kutta
  function integrate(start, rho) {
    const points = [start];
    let p = start;
    const along = (a, k, h) => a.map((v, i) => v + k[i] * h);
    for (let n = 1; n <= STEPS; n++) {
      const k1 = flow(p, rho), k2 = flow(along(p, k1, DT / 2), rho), k3 = flow(along(p, k2, DT / 2), rho), k4 = flow(along(p, k3, DT), rho);
      p = p.map((v, i) => v + (DT / 6) * (k1[i] + 2 * k2[i] + 2 * k3[i] + k4[i]));
      if (n % EVERY === 0) points.push(p);
    }
    return points;
  }

  // a point of the system on the figure, seen from azimuth `az` and elevation `el`, in radians
  function project([x, y, z], az, el) {
    const across = x * Math.cos(az) - y * Math.sin(az), deep = x * Math.sin(az) + y * Math.cos(az);
    return [across * SCALE, (deep * Math.sin(el) + (z - MIDDLE) * Math.cos(el)) * SCALE];
  }

  const place = (p, az, el) => `(${project(p, az, el).map((v) => v.toFixed(2)).join(",")})`;
  const curve = (points, count, az, el, color) => (count < 2 ? ""
    : `\\draw[${color}, line width=0.45pt, line join=round] plot coordinates {${points.slice(0, count).map((p) => place(p, az, el)).join(" ")}};\n`);

  // the TikZ of the two paths, up to time `t`
  function tikz(red, blue, t, az, el) {
    const count = Math.max(1, Math.min(red.length, Math.round(t / (DT * EVERY)) + 1));
    return String.raw`\documentclass[tikz,border=0pt]{standalone}
\begin{document}
\begin{tikzpicture}[scale=1.15]
  \definecolor{byrnered}{HTML}{C8102E}
  \definecolor{byrneblue}{HTML}{1B4F9C}
  \definecolor{byrneyellow}{HTML}{F2B705}
  \definecolor{byrneink}{HTML}{15120E}
  \path[use as bounding box] (-4.2,-3.6) rectangle (4.2,3.6);
  \clip (-4.2,-3.6) rectangle (4.2,3.6);
` + curve(blue, count, az, el, "byrneblue") + curve(red, count, az, el, "byrnered") + String.raw`
  \fill[byrneyellow, draw=byrneink, line width=0.6pt] ${place(red[0], az, el)} circle (2.2pt);
  \fill[byrneblue, draw=byrneink, line width=0.6pt] ${place(blue[count - 1], az, el)} circle (2.8pt);
  \fill[byrnered, draw=byrneink, line width=0.6pt] ${place(red[count - 1], az, el)} circle (2.8pt);
\end{tikzpicture}
\end{document}
`;
  }

  // in `box`, with `render(code)` → {svg}: drag turns the figure, the sliders change the system
  function mount(box, render) {
    box.innerHTML = `<div class="lz-figure" tabindex="0" role="img" aria-label="The Lorenz attractor: two paths that start almost together and part. Drag, or use the arrow keys, to turn it."></div>
      <div class="lz-controls">
        <label class="param"><span>time</span><input id="lz-time" type="range" min="0" max="${END}" step="0.02" value="${END}"><output></output></label>
        <label class="param"><span>ρ</span><input id="lz-rho" type="range" min="10" max="40" step="0.1" value="28"><output></output></label>
        <label class="param"><span>gap</span><input id="lz-gap" type="range" min="-4" max="0" step="0.1" value="-3"><output></output></label>
        <button class="button" type="button" id="lz-play">Run it again</button>
      </div>`;
    const figure = box.querySelector(".lz-figure");
    const input = (id) => box.querySelector("#lz-" + id);
    const [time, rho, gap] = ["time", "rho", "gap"].map(input);
    let [az, el] = VIEW, red, blue, frame = 0, wanted = false;

    function solve() {
      const start = [1, 1, 25], gapNow = 10 ** Number(gap.value);
      red = integrate(start, Number(rho.value));
      blue = integrate([start[0] + gapNow, start[1], start[2]], Number(rho.value));
    }
    function draw() {
      figure.innerHTML = render(tikz(red, blue, Number(time.value), az, el)).svg;
      time.nextElementSibling.value = Number(time.value).toFixed(1);
      rho.nextElementSibling.value = Number(rho.value).toFixed(1);
      gap.nextElementSibling.value = (10 ** Number(gap.value)).toPrecision(1);
    }
    // one drawing per screen refresh, however many moves come in
    const ask = () => {
      if (wanted) return;
      wanted = true;
      requestAnimationFrame(() => { wanted = false; draw(); });
    };

    const stop = () => cancelAnimationFrame(frame);
    function play() {
      stop();
      if (matchMedia("(prefers-reduced-motion: reduce)").matches) { time.value = END; draw(); return; }
      const began = performance.now();
      const tick = (now) => {
        time.value = Math.min(END, ((now - began) / 1000) * 3.5);
        draw();
        if (Number(time.value) < END) frame = requestAnimationFrame(tick);
      };
      frame = requestAnimationFrame(tick);
    }

    time.oninput = () => { stop(); ask(); };
    rho.oninput = gap.oninput = () => { solve(); ask(); };
    input("play").onclick = play;

    let held = null;
    figure.addEventListener("pointerdown", (e) => { held = [e.clientX, e.clientY]; figure.setPointerCapture(e.pointerId); });
    figure.addEventListener("pointermove", (e) => {
      if (!held) return;
      az -= (e.clientX - held[0]) * 0.01;
      el = Math.max(-1.57, Math.min(1.57, el + (e.clientY - held[1]) * 0.01));
      held = [e.clientX, e.clientY];
      ask();
    });
    for (const end of ["pointerup", "pointercancel"]) figure.addEventListener(end, () => { held = null; });
    figure.addEventListener("keydown", (e) => {
      const turn = { ArrowLeft: 0.1, ArrowRight: -0.1 }[e.key], tilt = { ArrowUp: -0.1, ArrowDown: 0.1 }[e.key];
      if (turn === undefined && tilt === undefined) return;
      e.preventDefault();
      az += turn ?? 0;
      el = Math.max(-1.57, Math.min(1.57, el + (tilt ?? 0)));
      ask();
    });

    solve();
    draw();
    play();
  }

  return { mount, integrate, tikz, VIEW };
})();
if (typeof module !== "undefined") module.exports = tikraLorenz;
