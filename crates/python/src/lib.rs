// one option per argument: this is the raw boundary, and the packages wrap it in named options
#![allow(clippy::too_many_arguments)]

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

fn error(e: tikra::Error) -> PyErr {
    PyValueError::new_err(e.to_string())
}

/// The options from a language code (`pt-BR`, `fr`…), with the construction animation and the
/// numbered steps and their place in the code or not.
fn options(locale: &str, animate: bool, steps: bool, source: bool, parts: bool, play: bool, points: bool) -> PyResult<tikra::Options> {
    let o: tikra::Options = locale.parse().map(tikra::Options::locale).map_err(PyValueError::new_err)?;
    let o = if animate { o.animated() } else { o };
    let o = if steps { o.steps() } else { o };
    let o = if source { o.source() } else { o };
    let o = if parts { o.parts() } else { o };
    let o = if play { o.play() } else { o };
    Ok(if points { o.points() } else { o })
}

/// Converts a TikZ figure to SVG, with the figure in words in `locale`.
#[pyfunction]
#[pyo3(signature = (code, *, locale = "en", animate = false, steps = false, source = false, parts = false, play = false, points = false))]
fn tikz_to_svg(code: &str, locale: &str, animate: bool, steps: bool, source: bool, parts: bool, play: bool, points: bool) -> PyResult<String> {
    tikra::convert_with(code, &options(locale, animate, steps, source, parts, play, points)?).map(|r| r.svg).map_err(error)
}

/// The SVG, the figure in words (alt text and description) and the warnings (what was ignored or
/// approximated); `tikra.render` turns them into a dict.
#[pyfunction]
#[pyo3(signature = (code, *, locale = "en", animate = false, steps = false, source = false, parts = false, play = false, points = false))]
fn render(code: &str, locale: &str, animate: bool, steps: bool, source: bool, parts: bool, play: bool, points: bool) -> PyResult<(String, String, String, Vec<String>)> {
    tikra::convert_with(code, &options(locale, animate, steps, source, parts, play, points)?).map(|r| (r.svg, r.alt, r.description, r.warnings)).map_err(error)
}

/// Rasterizes an SVG written by tikra to PNG bytes.
#[pyfunction]
#[pyo3(signature = (svg, *, dpi = 300.0, background = true))]
fn png(svg: &str, dpi: f32, background: bool) -> PyResult<Vec<u8>> {
    tikra::png(svg, dpi, background).map_err(error)
}

/// The construction animation as GIF bytes.
#[pyfunction]
#[pyo3(signature = (code, *, width = 560, fps = 15))]
fn tikz_to_gif(code: &str, width: u32, fps: u32) -> PyResult<Vec<u8>> {
    let svg = tikra::convert_with(code, &tikra::Options::default().animated()).map_err(error)?.svg;
    tikra::motion::gif(&svg, tikra::motion::Motion { width, fps }).map_err(error)
}

/// The construction animation as APNG bytes.
#[pyfunction]
#[pyo3(signature = (code, *, width = 560, fps = 15))]
fn tikz_to_apng(code: &str, width: u32, fps: u32) -> PyResult<Vec<u8>> {
    let svg = tikra::convert_with(code, &tikra::Options::default().animated()).map_err(error)?.svg;
    tikra::motion::apng(&svg, tikra::motion::Motion { width, fps }).map_err(error)
}

/// The SVG of each slide of a figure with Beamer's overlays.
#[pyfunction]
#[pyo3(signature = (code, *, locale = "en"))]
fn slides(code: &str, locale: &str) -> PyResult<Vec<String>> {
    let slides = tikra::slides(code, &options(locale, false, false, false, false, false, false)?).map_err(error)?;
    Ok(slides.into_iter().map(|s| s.svg).collect())
}

/// The slides as one SVG that advances on a click.
#[pyfunction]
#[pyo3(signature = (code, *, locale = "en"))]
fn click_through(code: &str, locale: &str) -> PyResult<String> {
    let slides = tikra::slides(code, &options(locale, false, false, false, false, false, false)?).map_err(error)?;
    tikra::click_through(&slides).ok_or_else(|| PyValueError::new_err("could not join the slides"))
}

/// What the pen does with a figure, and the figure's SVG.
fn pen_of(code: &str, keep_order: bool, seconds: f64, paper: Option<(f64, f64)>) -> PyResult<(String, tikra::pen::Pen)> {
    let svg = tikra::convert(code).map_err(error)?.svg;
    Ok((svg, tikra::pen::Pen { keep_order, seconds, paper, ..Default::default() }))
}

/// The figure's strokes as G-code.
#[pyfunction]
#[pyo3(signature = (code, *, keep_order = false, paper = None))]
fn tikz_to_gcode(code: &str, keep_order: bool, paper: Option<(f64, f64)>) -> PyResult<String> {
    let (svg, pen) = pen_of(code, keep_order, 10.0, paper)?;
    tikra::pen::gcode(&svg, &pen).map_err(error)
}

/// The figure's strokes as HPGL.
#[pyfunction]
#[pyo3(signature = (code, *, keep_order = false, paper = None))]
fn tikz_to_hpgl(code: &str, keep_order: bool, paper: Option<(f64, f64)>) -> PyResult<String> {
    let (svg, pen) = pen_of(code, keep_order, 10.0, paper)?;
    tikra::pen::hpgl(&svg, &pen).map_err(error)
}

/// An SVG where a pen draws the figure.
#[pyfunction]
#[pyo3(signature = (code, *, seconds = 10.0, keep_order = false, paper = None))]
fn tikz_to_pen_svg(code: &str, seconds: f64, keep_order: bool, paper: Option<(f64, f64)>) -> PyResult<String> {
    let (svg, pen) = pen_of(code, keep_order, seconds, paper)?;
    tikra::pen::animation(&svg, &pen).map_err(error)
}

/// A pen drawing the figure, as GIF bytes.
#[pyfunction]
#[pyo3(signature = (code, *, seconds = 10.0, width = 560, fps = 15, keep_order = false, paper = None))]
fn tikz_to_pen_gif(code: &str, seconds: f64, width: u32, fps: u32, keep_order: bool, paper: Option<(f64, f64)>) -> PyResult<Vec<u8>> {
    let (svg, pen) = pen_of(code, keep_order, seconds, paper)?;
    tikra::motion::pen_gif(&svg, &pen, tikra::motion::Motion { width, fps }).map_err(error)
}

/// A pen drawing the figure, as APNG bytes.
#[pyfunction]
#[pyo3(signature = (code, *, seconds = 10.0, width = 560, fps = 15, keep_order = false, paper = None))]
fn tikz_to_pen_apng(code: &str, seconds: f64, width: u32, fps: u32, keep_order: bool, paper: Option<(f64, f64)>) -> PyResult<Vec<u8>> {
    let (svg, pen) = pen_of(code, keep_order, seconds, paper)?;
    tikra::motion::pen_apng(&svg, &pen, tikra::motion::Motion { width, fps }).map_err(error)
}

#[pymodule]
#[pyo3(name = "_tikra")]
fn tikra_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(tikz_to_svg, m)?)?;
    m.add_function(wrap_pyfunction!(render, m)?)?;
    m.add_function(wrap_pyfunction!(png, m)?)?;
    m.add_function(wrap_pyfunction!(tikz_to_gcode, m)?)?;
    m.add_function(wrap_pyfunction!(tikz_to_hpgl, m)?)?;
    m.add_function(wrap_pyfunction!(tikz_to_pen_svg, m)?)?;
    m.add_function(wrap_pyfunction!(tikz_to_pen_gif, m)?)?;
    m.add_function(wrap_pyfunction!(tikz_to_pen_apng, m)?)?;
    m.add_function(wrap_pyfunction!(slides, m)?)?;
    m.add_function(wrap_pyfunction!(click_through, m)?)?;
    m.add_function(wrap_pyfunction!(tikz_to_gif, m)?)?;
    m.add_function(wrap_pyfunction!(tikz_to_apng, m)?)?;
    // the Python package has its own version (crates/python/Cargo.toml); the core's is `core_version`
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("core_version", tikra::VERSION)?;
    Ok(())
}
