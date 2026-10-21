export interface Result {
  /** Standalone SVG, with math labels typeset by mathra. */
  svg: string;
  /** What was ignored or approximated: unsupported commands, labels with broken LaTeX. */
  warnings: string[];
}

export interface PngOptions {
  /** Resolution. Default 300, for print. */
  dpi?: number;
  /** Paint a white background (default) or keep it transparent (false). */
  background?: boolean;
}

/** Loads the WebAssembly module: await it once before converting in browsers and bundlers. In Node it does nothing. */
export function init(input?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module): Promise<void>;

/** Converts a TikZ figure (a `standalone` document, a `tikzpicture`, `\tikz`, an `axis` or loose statements) to SVG. Throws when there is no figure to draw. */
export function tikzToSvg(code: string): string;

/** Converts a TikZ figure and returns the SVG with the warnings. */
export function render(code: string): Result;

/** Rasterizes an SVG written by tikra to PNG bytes. Throws on an invalid SVG or dpi, or past 100 megapixels. */
export function png(svg: string, options?: PngOptions): Uint8Array;

/** The version of tikra's core inside this package (the npm package has its own version). */
export function version(): string;
