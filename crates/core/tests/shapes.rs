//! Node shapes of the shape libraries: drawn without warnings, with their named anchors usable in
//! coordinates.

use tikra::canvas::Item;
use tikra::scene::{Point, Segment};
use tikra::PT;

fn no_warnings(body: &str) -> String {
    let r = tikra::convert(&format!("\\begin{{tikzpicture}}{body}\\end{{tikzpicture}}")).unwrap();
    assert!(r.warnings.is_empty(), "{body}: {:?}", r.warnings);
    r.svg
}

#[test]
fn geometric_shapes_and_their_anchors() {
    for (shape, anchors) in [
        ("regular polygon, regular polygon sides=6", "corner 1,side 6"),
        ("star, star points=7, star point ratio=2", "outer point 7,inner point 1"),
        ("trapezium, trapezium left angle=70", "bottom left corner,top side"),
        ("semicircle", "apex,arc start,chord center"),
        ("isosceles triangle, shape border rotate=90", "apex,left corner,lower side"),
        ("kite, kite vertex angles=100 and 50", "upper vertex,lower left side"),
        ("dart", "tip,tail center,left tail"),
        ("circular sector, circular sector angle=90", "arc start,sector center,arc center"),
        ("cylinder, aspect=0.5", "top,before top,after bottom,shape center"),
    ] {
        let mut body = format!("\\node[draw, {shape}] (s) {{text}};");
        for a in anchors.split(',') {
            body.push_str(&format!("\\draw (0,2) -- (s.{a}) -- (s.30) -- (s.mid);"));
        }
        let svg = no_warnings(&body);
        assert!(svg.matches("<path").count() >= 2, "{shape}");
    }
}

/// The paths the engine draws for a picture body, in cm.
fn paths(body: &str) -> Vec<Vec<Segment>> {
    let (canvas, warnings) = tikra::engine::run_document(&format!("\\begin{{tikzpicture}}{body}\\end{{tikzpicture}}")).unwrap();
    assert!(warnings.is_empty(), "{body}: {warnings:?}");
    canvas.items.into_iter().filter_map(|i| if let Item::Path { segments, .. } = i { Some(segments) } else { None }).collect()
}

fn near(p: Point, x: f64, y: f64) -> bool {
    (p.x - x).abs() < 1e-3 && (p.y - y).abs() < 1e-3
}

#[test]
fn a_relative_second_control_is_relative_to_the_end() {
    let p = paths(r"\draw (0,0) .. controls +(1,1) and +(-1,1) .. (2,0);");
    match (p[0][0], p[0][1]) {
        (Segment::MoveTo(o), Segment::CurveTo(a, b, _)) => assert!(near(a.minus(o), 1.0, 1.0) && near(b.minus(o), 1.0, 1.0), "{a:?} {b:?}"),
        ref s => panic!("{s:?}"),
    }
}

#[test]
fn lines_stop_at_the_border_with_outer_sep() {
    // an empty circle of radius √2 · 3.333pt, plus .2pt of outer sep
    let r = (2.0f64.sqrt() * 3.333 + 0.2) * PT;
    let p = paths(r"\node[circle,draw] (a) {}; \draw (a) -- (1,0);");
    match (p[1][0], p[1][1]) {
        (Segment::MoveTo(q), Segment::LineTo(e)) => assert!(near(e.minus(q), 1.0 - r, 0.0), "{q:?} {e:?} {r}"),
        ref s => panic!("{s:?}"),
    }
    // a diamond: from its center toward its east corner
    let p = paths(r"\node[diamond,draw] (a) {}; \draw (a) -- (1,0);");
    match (p[1][0], p[1][1]) {
        (Segment::MoveTo(q), Segment::LineTo(e)) => assert!(near(e.minus(q), 1.0 - (2.0 * 3.333 + 0.2) * PT, 0.0), "{q:?} {e:?}"),
        ref s => panic!("{s:?}"),
    }
}

#[test]
fn split_parts_are_placed_and_named() {
    let p = paths(r"\node[circle split,draw] (s) {a \nodepart{lower} b}; \draw (s.lower) -- (s.center);");
    // the circle, the split line and the line between the anchors
    assert!(p.len() >= 3, "{p:?}");
    let p = paths(r"\node[rectangle split,rectangle split parts=3,draw] (s) {a \nodepart{two} b \nodepart{three} c}; \draw (s.two split) -- (s.text east);");
    assert!(p.len() >= 2, "{p:?}");
}

#[test]
fn shape_names_alone_are_shapes() {
    no_warnings("\\node[trapezium, draw] {a}; \\node[regular polygon, draw] at (2,0) {b};");
}

#[test]
fn a_part_named_again_starts_its_box_anew() {
    // `\nodepart{text}` at the start restarts the text box: the text part holds `text`, not nothing
    let ends = |text: &str| {
        let p = paths(&format!(r"\node[rectangle split,rectangle split parts=2,draw] (s) {{{text}}}; \draw (0,0) -- (s.text split) -- (s.text);"));
        match p.last().map(|s| s.as_slice()) {
            Some([Segment::MoveTo(_), Segment::LineTo(a), Segment::LineTo(b)]) => (*a, *b),
            s => panic!("{s:?}"),
        }
    };
    let (a, b) = ends(r"\nodepart{text}text\nodepart{two}two");
    let (c, d) = ends(r"text\nodepart{two}two");
    assert!(near(a, c.x, c.y) && near(b, d.x, d.y), "{a:?} {b:?} / {c:?} {d:?}");
}
