// The JavaScript API over the raw wasm-bindgen exports, shared by the Node and the browser builds.

export function wrap(raw, isReady) {
  const check = () => {
    if (!isReady()) throw new Error("tikra: call `await init()` once before converting (only needed outside Node)");
  };
  const penCall = (code, format, options) => {
    check();
    const o = options ?? {};
    return raw.pen(code, format, o.keepOrder ?? null, o.seconds ?? null, o.paper?.[0] ?? null, o.paper?.[1] ?? null);
  };
  return {
    tikzToSvg(code, options) {
      check();
      return raw.tikzToSvg(code, options?.locale ?? null, options?.animate ?? null, options?.steps ?? null, options?.source ?? null, options?.parts ?? null, options?.play ?? null, options?.points ?? null);
    },
    render(code, options) {
      check();
      return JSON.parse(raw.render(code, options?.locale ?? null, options?.animate ?? null, options?.steps ?? null, options?.source ?? null, options?.parts ?? null, options?.play ?? null, options?.points ?? null));
    },
    slides(code, options) {
      check();
      return JSON.parse(raw.slides(code, options?.locale ?? null));
    },
    clickThrough(code, options) {
      check();
      return raw.clickThrough(code, options?.locale ?? null);
    },
    gcode(code, options) {
      return penCall(code, "gcode", options);
    },
    hpgl(code, options) {
      return penCall(code, "hpgl", options);
    },
    penSvg(code, options) {
      return penCall(code, "svg", options);
    },
    penStills(code, count, options) {
      check();
      const o = options ?? {};
      return JSON.parse(raw.penStills(code, count, o.keepOrder ?? null, o.seconds ?? null, o.paper?.[0] ?? null, o.paper?.[1] ?? null));
    },
    png(svg, options) {
      check();
      const o = options ?? {};
      return raw.png(svg, o.dpi ?? null, o.background ?? null);
    },
    version() {
      check();
      return raw.version();
    },
  };
}
