use wasm_bindgen::prelude::*;

/// The figure as JSON, `{"svg": …, "warnings": […]}`: testable outside the browser.
pub fn convert_json(code: &str) -> Result<String, String> {
    let r = tikra::convert(code).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "svg": r.svg, "warnings": r.warnings }).to_string())
}

/// The PNG bytes of an SVG written by tikra. `dpi` defaults to 300, `background` (white) to true.
pub fn png_bytes(svg: &str, dpi: Option<f32>, background: Option<bool>) -> Result<Vec<u8>, String> {
    tikra::png(svg, dpi.unwrap_or(300.0), background.unwrap_or(true)).map_err(|e| e.to_string())
}

/// Converts a TikZ figure (a `standalone` document, a `tikzpicture`, `\tikz` or an `axis`) to SVG.
#[wasm_bindgen(js_name = tikzToSvg)]
pub fn tikz_to_svg(code: &str) -> Result<String, JsError> {
    tikra::convert(code).map(|r| r.svg).map_err(|e| JsError::new(&e.to_string()))
}

/// The SVG and what was ignored or approximated, as JSON (see `convert_json`).
#[wasm_bindgen]
pub fn render(code: &str) -> Result<String, JsError> {
    convert_json(code).map_err(|e| JsError::new(&e))
}

/// Rasterizes an SVG to PNG (see `png_bytes`).
#[wasm_bindgen]
pub fn png(svg: &str, dpi: Option<f32>, background: Option<bool>) -> Result<Vec<u8>, JsError> {
    png_bytes(svg, dpi, background).map_err(|e| JsError::new(&e))
}

/// The version of tikra's core.
#[wasm_bindgen]
pub fn version() -> String {
    tikra::VERSION.to_string()
}
