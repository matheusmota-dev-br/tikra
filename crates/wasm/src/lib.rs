use wasm_bindgen::prelude::*;

/// The options from a language code (`pt-BR`, `fr`…); none is English.
fn options(locale: Option<String>) -> Result<tikra::Options, String> {
    match locale {
        Some(code) => Ok(tikra::Options::locale(code.parse()?)),
        None => Ok(tikra::Options::default()),
    }
}

/// The figure as JSON, `{"svg": …, "alt": …, "description": …, "warnings": […]}`, its words in
/// `locale`: testable outside the browser.
pub fn convert_json(code: &str, locale: Option<String>) -> Result<String, String> {
    let r = tikra::convert_with(code, &options(locale)?).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "svg": r.svg, "alt": r.alt, "description": r.description, "warnings": r.warnings }).to_string())
}

/// The PNG bytes of an SVG written by tikra. `dpi` defaults to 300, `background` (white) to true.
pub fn png_bytes(svg: &str, dpi: Option<f32>, background: Option<bool>) -> Result<Vec<u8>, String> {
    tikra::png(svg, dpi.unwrap_or(300.0), background.unwrap_or(true)).map_err(|e| e.to_string())
}

/// Converts a TikZ figure (a `standalone` document, a `tikzpicture`, `\tikz` or an `axis`) to SVG.
#[wasm_bindgen(js_name = tikzToSvg)]
pub fn tikz_to_svg(code: &str, locale: Option<String>) -> Result<String, JsError> {
    svg(code, locale).map_err(|e| JsError::new(&e))
}

/// The SVG, its words in `locale`.
pub fn svg(code: &str, locale: Option<String>) -> Result<String, String> {
    tikra::convert_with(code, &options(locale)?).map(|r| r.svg).map_err(|e| e.to_string())
}

/// The SVG, the figure in words and what was ignored or approximated, as JSON (see `convert_json`).
#[wasm_bindgen]
pub fn render(code: &str, locale: Option<String>) -> Result<String, JsError> {
    convert_json(code, locale).map_err(|e| JsError::new(&e))
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
