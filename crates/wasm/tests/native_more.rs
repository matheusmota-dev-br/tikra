//! The JavaScript exports themselves, outside the browser, where they succeed: a `JsError` is only
//! made on failure, which takes wasm (tests/web.rs).

use tikra_wasm::{click_through_js, pen, pen_stills, png, render, slides_js};

const FIGURE: &str = r"\begin{tikzpicture}\draw (0,0) -- (1,1); \only<2>{\draw (0,1) -- (1,0);}\end{tikzpicture}";

/// `pen`: G-code, HPGL or a pen drawing the figure, as text.
#[test]
fn pen_formats() {
    let g = pen(FIGURE, "gcode", Some(true), None, Some(100.0), Some(50.0)).unwrap();
    assert!(g.starts_with("G21\nG90\n"), "{g}");
    let h = pen(FIGURE, "hpgl", None, None, None, None).unwrap();
    assert!(h.starts_with("IN;SP1;"), "{h}");
    let s = pen(FIGURE, "svg", None, Some(2.0), None, None).unwrap();
    assert!(s.starts_with("<svg"), "{s}");
}

/// `penStills`: the frames of the pen's drawing, `count` + 1 of them.
#[test]
fn pen_stills_frames() {
    let json: serde_json::Value = serde_json::from_str(&pen_stills(FIGURE, 3, None, Some(1.0), None, None).unwrap()).unwrap();
    assert_eq!(json.as_array().unwrap().len(), 4);
}

/// `slides` and `clickThrough`: each slide's SVG, and all of them in one.
#[test]
fn slides_and_click_through() {
    let json: serde_json::Value = serde_json::from_str(&slides_js(FIGURE, None).unwrap()).unwrap();
    assert_eq!(json.as_array().unwrap().len(), 2);
    let all = click_through_js(FIGURE, Some("fr".into())).unwrap();
    assert!(all.contains(r#"data-slides="2""#));
}

/// `render`: the JSON of the figure, with its points when asked for.
#[test]
fn render_json() {
    let json: serde_json::Value = serde_json::from_str(&render(FIGURE, None, Some(true), None, None, None, None, None).unwrap()).unwrap();
    assert!(json["svg"].as_str().unwrap().contains("data-steps"));
}

/// `png`: PNG bytes of an SVG.
#[test]
fn png_export() {
    let svg = tikra::convert(FIGURE).unwrap().svg;
    let bytes = png(&svg, Some(72.0), Some(false)).unwrap();
    assert_eq!(&bytes[..4], b"\x89PNG");
}
