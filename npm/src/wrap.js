// The JavaScript API over the raw wasm-bindgen exports, shared by the Node and the browser builds.

export function wrap(raw, isReady) {
  const check = () => {
    if (!isReady()) throw new Error("tikra: call `await init()` once before converting (only needed outside Node)");
  };
  return {
    tikzToSvg(code) {
      check();
      return raw.tikzToSvg(code);
    },
    render(code) {
      check();
      return JSON.parse(raw.render(code));
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
