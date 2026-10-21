use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

fn error(e: tikra::Error) -> PyErr {
    PyValueError::new_err(e.to_string())
}

/// Converts a TikZ figure to SVG.
#[pyfunction]
fn tikz_to_svg(code: &str) -> PyResult<String> {
    tikra::convert(code).map(|r| r.svg).map_err(error)
}

/// The SVG and the warnings (what was ignored or approximated); `tikra.render` turns them into a dict.
#[pyfunction]
fn render(code: &str) -> PyResult<(String, Vec<String>)> {
    tikra::convert(code).map(|r| (r.svg, r.warnings)).map_err(error)
}

/// Rasterizes an SVG written by tikra to PNG bytes.
#[pyfunction]
#[pyo3(signature = (svg, *, dpi = 300.0, background = true))]
fn png(svg: &str, dpi: f32, background: bool) -> PyResult<Vec<u8>> {
    tikra::png(svg, dpi, background).map_err(error)
}

#[pymodule]
#[pyo3(name = "_tikra")]
fn tikra_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(tikz_to_svg, m)?)?;
    m.add_function(wrap_pyfunction!(render, m)?)?;
    m.add_function(wrap_pyfunction!(png, m)?)?;
    // the Python package has its own version (crates/python/Cargo.toml); the core's is `core_version`
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("core_version", tikra::VERSION)?;
    Ok(())
}
