use wasm_bindgen::prelude::*;

/// The options from a language code (`pt-BR`, `fr`…), none is English, with the construction
/// animation when `animate`, the numbered steps when `steps`, their place in the code when `source`, and the figure split in
/// parts for a reader when `parts`.
fn options(locale: Option<String>, animate: Option<bool>, steps: Option<bool>, source: Option<bool>, parts: Option<bool>) -> Result<tikra::Options, String> {
    let o = match locale {
        Some(code) => tikra::Options::locale(code.parse()?),
        None => tikra::Options::default(),
    };
    let o = if animate.unwrap_or(false) { o.animated() } else { o };
    let o = if steps.unwrap_or(false) { o.steps() } else { o };
    let o = if source.unwrap_or(false) { o.source() } else { o };
    Ok(if parts.unwrap_or(false) { o.parts() } else { o })
}

/// The figure as JSON, `{"svg": …, "alt": …, "description": …, "warnings": […], "points": [{"name",
/// "x", "y"}], "origin": [x, y]}`, its words in `locale`: testable outside the browser.
pub fn convert_json(code: &str, locale: Option<String>, animate: Option<bool>, steps: Option<bool>, source: Option<bool>, parts: Option<bool>) -> Result<String, String> {
    let r = tikra::convert_with(code, &options(locale, animate, steps, source, parts)?).map_err(|e| e.to_string())?;
    let points: Vec<_> = r.points.iter().map(|p| serde_json::json!({ "name": p.name, "x": p.x, "y": p.y })).collect();
    let parts: Vec<_> = r.parts.iter().map(|p| serde_json::json!({ "source": p.source, "text": p.text, "sentence": p.sentence })).collect();
    Ok(serde_json::json!({ "svg": r.svg, "alt": r.alt, "description": r.description, "warnings": r.warnings, "points": points, "origin": [r.origin.0, r.origin.1], "parts": parts }).to_string())
}

/// The SVG of each slide of a figure with Beamer's overlays, as a JSON array.
pub fn slides_json(code: &str, locale: Option<String>) -> Result<String, String> {
    let slides = tikra::slides(code, &options(locale, None, None, None, None)?).map_err(|e| e.to_string())?;
    Ok(serde_json::json!(slides.iter().map(|s| &s.svg).collect::<Vec<_>>()).to_string())
}

/// The slides of a figure as one SVG that advances on a click.
pub fn click_svg(code: &str, locale: Option<String>) -> Result<String, String> {
    let slides = tikra::slides(code, &options(locale, None, None, None, None)?).map_err(|e| e.to_string())?;
    tikra::click_through(&slides).ok_or_else(|| "could not join the slides".to_string())
}

/// `slidesJson` for JavaScript.
#[wasm_bindgen(js_name = slides)]
pub fn slides_js(code: &str, locale: Option<String>) -> Result<String, JsError> {
    slides_json(code, locale).map_err(|e| JsError::new(&e))
}

/// `clickSvg` for JavaScript.
#[wasm_bindgen(js_name = clickThrough)]
pub fn click_through_js(code: &str, locale: Option<String>) -> Result<String, JsError> {
    click_svg(code, locale).map_err(|e| JsError::new(&e))
}

/// The PNG bytes of an SVG written by tikra. `dpi` defaults to 300, `background` (white) to true.
pub fn png_bytes(svg: &str, dpi: Option<f32>, background: Option<bool>) -> Result<Vec<u8>, String> {
    tikra::png(svg, dpi.unwrap_or(300.0), background.unwrap_or(true)).map_err(|e| e.to_string())
}

/// Converts a TikZ figure (a `standalone` document, a `tikzpicture`, `\tikz` or an `axis`) to SVG.
#[wasm_bindgen(js_name = tikzToSvg)]
pub fn tikz_to_svg(code: &str, locale: Option<String>, animate: Option<bool>, steps: Option<bool>, source: Option<bool>, parts: Option<bool>) -> Result<String, JsError> {
    svg(code, locale, animate, steps, source, parts).map_err(|e| JsError::new(&e))
}

/// The SVG, its words in `locale`.
pub fn svg(code: &str, locale: Option<String>, animate: Option<bool>, steps: Option<bool>, source: Option<bool>, parts: Option<bool>) -> Result<String, String> {
    tikra::convert_with(code, &options(locale, animate, steps, source, parts)?).map(|r| r.svg).map_err(|e| e.to_string())
}

/// The SVG, the figure in words and what was ignored or approximated, as JSON (see `convert_json`).
#[wasm_bindgen]
pub fn render(code: &str, locale: Option<String>, animate: Option<bool>, steps: Option<bool>, source: Option<bool>, parts: Option<bool>) -> Result<String, JsError> {
    convert_json(code, locale, animate, steps, source, parts).map_err(|e| JsError::new(&e))
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
