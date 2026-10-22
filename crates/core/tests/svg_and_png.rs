//! The generated SVG (bounds, labels, clips, opacities) and PNG rasterization.

mod common;
use common::*;
use tikra::canvas::{Canvas, Item};
use tikra::color::Color;
use tikra::scene::{Label, Transform};

fn png_width(png: &[u8]) -> u32 {
    u32::from_be_bytes([png[16], png[17], png[18], png[19]])
}

#[test]
fn svg_with_opacities_dashes_and_drawn_nodes() {
    let r = no_warnings(
        r"\begin{tikzpicture}
            \filldraw[fill opacity=0.4, draw opacity=0.6, dashed] (0,0) rectangle (1,1);
            \node[draw, fill=yellow, dashed, rounded corners] at (3,0) {box};
            \node[draw, circle, fill=red] at (5,0) {c};
            \node[draw, diamond] at (7,0) {d};
            \node[draw, rounded corners=1mm, minimum size=1cm] at (9,0) {};
          \end{tikzpicture}",
    );
    for fragment in ["fill-opacity=\"0.4\"", "stroke-opacity=\"0.6\"", "stroke-dasharray", "#ffff00", "<use href=\"#tr-g"] {
        assert!(r.svg.contains(fragment), "{fragment}");
    }
}

#[test]
fn multiline_label_and_alignments() {
    for a in ["left", "right", "center"] {
        let r = no_warnings(&format!(r"\begin{{tikzpicture}} \node[align={a}] {{line one\\ two}}; \end{{tikzpicture}}"));
        assert!(r.svg.contains("<g transform="), "{a}");
    }
    // math with a fraction and a root: mathra's rules and lines
    let r = no_warnings(r"\begin{tikzpicture} \node {$\frac{a}{b} + \sqrt{x} + \overline{y}$}; \end{tikzpicture}");
    assert!(r.svg.contains("<rect"));
}

#[test]
fn mathra_items_with_color_and_ellipse() {
    let r = figure(
        r"\begin{tikzpicture} \node {$\textcolor{red}{x} + \cancel{y} + \boxed{z} + \fcolorbox{red}{yellow}{w}$}; \node at (0,1) {$\bigcirc$}; \node at (0,2) {$\textcolor{red}{\frac{a}{b}}$}; \end{tikzpicture}",
    );
    assert!(r.svg.contains("<svg"));
}

#[test]
fn empty_canvas_has_minimum_size() {
    let svg = tikra::driver::svg(&Canvas::default(), "", "empty");
    assert!(svg.contains("viewBox"));
    // a text whose label has no lines
    let mut c = Canvas::default();
    let label = Label { lines: Vec::new(), scale: 1.0, color: Color::BLACK, align: 'c', line_height: 0.0 };
    assert_eq!(label.depth(), 0.0);
    assert_eq!(label.height(), 0.0);
    c.push(Item::Text { label, place: Transform::default(), opacity: 1.0 });
    assert!(tikra::driver::svg(&c, "", "x").contains("viewBox"));
}

#[test]
fn nested_clips() {
    let r = no_warnings(r"\begin{tikzpicture} \begin{scope} \clip (0,0) circle (1); \clip (0,0) rectangle (2,2); \fill (0,0) rectangle (3,3); \end{scope} \end{tikzpicture}");
    assert!(r.svg.contains("tr-c2"));
    assert_eq!(r.svg.matches("<g clip-path").count(), 2);
}

#[test]
fn png_with_and_without_background_and_dpi() {
    let r = no_warnings(r"\begin{tikzpicture} \draw (0,0) -- (2.54,0); \end{tikzpicture}");
    let a = tikra::png(&r.svg, 96.0, true).unwrap();
    let b = tikra::png(&r.svg, 192.0, false).unwrap();
    assert!(png_width(&b) > png_width(&a) * 19 / 10);
    assert_ne!(a, b);
}

#[test]
fn png_rejects_invalid_svg_and_bad_dpi() {
    assert!(matches!(tikra::png("this is not svg", 300.0, true), Err(tikra::Error::Png(_))));
    let r = no_warnings(r"\begin{tikzpicture} \draw (0,0) -- (1,0); \end{tikzpicture}");
    for dpi in [0.0, -10.0, f32::NAN, f32::INFINITY] {
        let e = tikra::png(&r.svg, dpi, true).unwrap_err();
        assert!(e.to_string().contains("invalid dpi"), "{dpi}: {e}");
    }
    // a tiny dpi: the image vanishes
    assert!(tikra::png(&r.svg, 0.001, true).is_err());
    // a huge dpi: goes past the pixel cap instead of allocating gigabytes
    let e = tikra::png(&r.svg, 1e6, true).unwrap_err();
    assert!(e.to_string().contains("too large"), "{e}");
}
