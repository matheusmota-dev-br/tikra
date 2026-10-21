//! Nodes: shapes, anchors, positioning, extra labels, text and size.

mod common;
use common::*;
use tikra::scene::{Label, Point, Segment, Transform};

/// The border a node draws (the first path of the body).
fn border(body: &str) -> Vec<Segment> {
    first_path(body).segments
}

/// Width and height of the border a node draws.
fn border_size(body: &str) -> (f64, f64) {
    let (a, b) = bounds(&border(body));
    (b.x - a.x, b.y - a.y)
}

/// The first label of a body and where it is placed.
fn label(body: &str) -> (Label, Transform) {
    texts(&canvas(body)).into_iter().next().unwrap_or_else(|| panic!("no text in {body}"))
}

fn curves(segs: &[Segment]) -> usize {
    segs.iter().filter(|s| matches!(s, Segment::CurveTo(..))).count()
}

fn lines(segs: &[Segment]) -> usize {
    segs.iter().filter(|s| matches!(s, Segment::LineTo(..))).count()
}

/// The shape a border path has: "rectangle" (straight, its corners off the axes), "diamond"
/// (straight, its corners on the axes through the center), "circle" or "ellipse" (curves).
fn shape_of(segs: &[Segment]) -> &'static str {
    let (a, b) = bounds(segs);
    let c = a.lerp(b, 0.5);
    if curves(segs) > 0 {
        return if close(b.x - a.x, b.y - a.y) { "circle" } else { "ellipse" };
    }
    let on_axis = points(segs).iter().all(|&(x, y)| (x - c.x).abs() < 1e-6 || (y - c.y).abs() < 1e-6);
    if on_axis {
        "diamond"
    } else {
        "rectangle"
    }
}

#[test]
fn shapes_by_option_and_by_shape_key() {
    for (op, shape) in [
        ("circle", "circle"),
        ("ellipse", "ellipse"),
        ("rectangle", "rectangle"),
        ("diamond", "diamond"),
        ("shape=circle", "circle"),
        ("shape=ellipse", "ellipse"),
        ("shape=diamond", "diamond"),
    ] {
        // a wide text, so that an ellipse is not a circle
        assert_eq!(shape_of(&border(&format!(r"\node[{op}, draw] {{ABCD}};"))), shape, "{op}");
    }
    // a shape with no definition still draws the node
    assert!(!paths(&canvas(r"\node[shape=star, draw] {A};")).is_empty());
    // a coordinate draws nothing
    let c = canvas(r"\node[shape=coordinate] (p) at (1,1) {};  \node[coordinate] (q) at (2,2) {};");
    assert!(paths(&c).is_empty() && texts(&c).is_empty());
}

#[test]
fn minimum_sizes_separation_and_aspect() {
    let (w, h) = border_size(r"\node[draw, minimum width=3cm, minimum height=2cm] {};");
    assert!(close(w, 3.0) && close(h, 2.0), "{w} {h}");
    let (w, _) = border_size(r"\node[draw, circle, minimum size=2cm] {};");
    assert!(close(w, 2.0), "{w}");
    let (w, _) = border_size(r"\node[draw, inner sep=0pt] {};");
    // an empty node with no inner sep: TikZ's minimum width of 1pt
    assert!(close(w, tikra::PT), "{w}");
    let (w, _) = border_size(r"\node[draw, inner sep=0pt, minimum size=0pt] {};");
    assert!(close(w, 0.0), "{w}");
    let (w, h) = border_size(r"\node[draw, inner xsep=1cm, inner ysep=2cm] {};");
    assert!(close(w, 2.0) && close(h, 4.0), "{w} {h}");
    let (narrow, _) = border_size(r"\node[diamond, draw, aspect=3] {AB};");
    let (wide, _) = border_size(r"\node[diamond, draw] {AB};");
    assert!(narrow > wide, "{narrow} {wide}");
    let (w, _) = border_size(r"\node[ellipse, draw, minimum width=4cm, minimum height=1cm] {x};");
    assert!(close(w, 4.0), "{w}");
    // bad values: the node is still drawn
    let (c, _) = canvas_and_warnings(r"\node[draw, inner sep=zz, inner xsep=zz, inner ysep=zz, minimum width=zz, minimum height=zz, minimum size=zz, aspect=zz] {x};");
    assert!(!texts(&c).is_empty());
}

#[test]
fn named_and_angle_anchors() {
    // outer sep=0pt: the anchors on the border itself (TikZ puts them half a line width outside)
    let n = r"\node[draw, outer sep=0pt, minimum width=2cm, minimum height=2cm] (n) at (0,0) {};";
    for (a, p) in [
        ("north", (0.0, 1.0)),
        ("south", (0.0, -1.0)),
        ("east", (1.0, 0.0)),
        ("west", (-1.0, 0.0)),
        ("north east", (1.0, 1.0)),
        ("south west", (-1.0, -1.0)),
        ("center", (0.0, 0.0)),
        ("base", (0.0, 0.0)),
        // mid: half an ex (2.15pt) above the baseline, as in TikZ
        ("mid", (0.0, 2.15 * tikra::PT)),
        ("text", (0.0, 0.0)),
        ("0", (1.0, 0.0)),
    ] {
        let q = point(n, &format!("n.{a}"));
        assert!(close_to((q.x, q.y), p), "{a}: {q:?}");
    }
    let q = point(n, "n");
    assert!(close_to((q.x, q.y), (0.0, 0.0)), "the node's name alone is its center: {q:?}");
    // the default outer sep: half the line width outside the border
    let q = point(r"\node[draw, minimum width=2cm, minimum height=2cm] (n) at (0,0) {};", "n.north");
    assert!(close(q.y, 1.0 + 0.2 * tikra::PT), "{q:?}");
    // an anchor that does not exist
    let (_, w) = canvas_and_warnings(&format!(r"{n} \draw (n.nothing) -- (1,1);"));
    assert!(!w.is_empty());
    let c = r"\node[circle, draw, outer sep=0pt, minimum size=2cm] (c) at (0,0) {};";
    let q = point(c, "c.north east");
    assert!(close(q.x.hypot(q.y), 1.0), "the circle's corner is on its border: {q:?}");
    // a coordinate's anchors are its point
    let q = point(r"\coordinate (p) at (1,2);", "p.east");
    assert!(close_to((q.x, q.y), (1.0, 2.0)), "{q:?}");
    let l = r"\node[diamond, draw, outer sep=0pt, minimum size=2cm] (l) at (0,0) {};";
    assert!(point(l, "l.45").x > 0.0);
    // a line toward a node stops at its border, also in a purely vertical or horizontal direction
    let p = paths(&canvas(&format!(r"{n} \draw (0,5) -- (n);")));
    let q = points(&p.last().unwrap().segments);
    assert!(close(q[1].1, 1.0), "{q:?}");
    let p = paths(&canvas(&format!(r"{n} \draw (5,0) -- (n);")));
    let q = points(&p.last().unwrap().segments);
    assert!(close(q[1].0, 1.0), "{q:?}");
}

#[test]
fn anchor_and_relative_placement() {
    let base = r"\node[draw, minimum size=1cm, inner sep=0pt, outer sep=0pt] (n) at (0,0) {};";
    let center = |options: &str| point(&base.replace("draw,", &format!("draw, {options},")), "n.center");
    assert!(close(center("anchor=west").x, 0.5), "{:?}", center("anchor=west"));
    for (pos, sign) in [
        ("above", (0.0, 1.0)),
        ("below", (0.0, -1.0)),
        ("left", (-1.0, 0.0)),
        ("right", (1.0, 0.0)),
        ("above left", (-1.0, 1.0)),
        ("above right", (1.0, 1.0)),
        ("below left", (-1.0, -1.0)),
        ("below right", (1.0, -1.0)),
    ] {
        let c = center(pos);
        assert!(c.x * sign.0 >= 0.0 && c.y * sign.1 >= 0.0, "{pos}: {c:?}");
        let c = center(&format!("{pos}=1cm"));
        assert!(c.x.abs() + c.y.abs() > 0.9, "{pos}=1cm: {c:?}");
    }
    assert!(close(center("xshift=1cm, yshift=-1cm").x, 1.0));
}

#[test]
fn positioning_of_with_distances() {
    let body = |op: &str| {
        format!(
            r"\node[draw, minimum size=1cm, inner sep=0pt, outer sep=0pt] (a) at (0,0) {{}}; \coordinate (p) at (5,5); \node[draw, minimum size=1cm, inner sep=0pt, outer sep=0pt, {op}] (b) {{}};"
        )
    };
    let b = |op: &str| point(&body(op), "b.center");
    assert!(close(b("right=of a").x, 2.0), "1cm default distance between borders");
    assert!(close(b("below=2cm of a").y, -3.0));
    let q = b("below left=1cm and 2cm of a");
    assert!(close(q.x, -3.0) && close(q.y, -2.0), "{q:?}");
    assert!(close(b("above=of a.north").y, 2.0));
    assert!(b("above right=of p").x > 5.0, "a coordinate as the target");
    for op in ["left=of a", "above left=of a", "above right=of a", "below right=of a", "above=1cm and 2cm of a"] {
        b(op);
    }
    let c = canvas(r"\begin{tikzpicture}[node distance=2cm and 3cm] \end{tikzpicture}");
    assert!(paths(&c).is_empty() && texts(&c).is_empty());
    let (_, w) = canvas_and_warnings(r"\node[right=of nobody] {x};");
    assert!(w.iter().any(|a| a.contains("nobody")), "{w:?}");
}

#[test]
fn figure_node_distance() {
    let r = no_warnings(
        r"\begin{tikzpicture}[node distance=2cm and 3cm]
            \node[draw] (a) {A}; \node[draw, below=of a] (b) {B}; \node[draw, right=of a] (c) {C};
          \end{tikzpicture}",
    );
    assert!(r.svg.contains("<path"));
    // an invalid node distance: a warning
    let r = figure(r"\begin{tikzpicture}[node distance=zz] \node (a) {A}; \node[below=of a] {B}; \end{tikzpicture}");
    assert!(!r.warnings.is_empty());
}

#[test]
fn position_along_the_path() {
    for (op, x) in
        [("midway", 1.0), ("near start", 0.5), ("near end", 1.5), ("at start", 0.0), ("at end", 2.0), ("very near start", 0.25), ("very near end", 1.75), ("pos=0.1", 0.2)]
    {
        let q = point(&format!(r"\draw (0,0) -- (2,0) node[{op}, inner sep=0pt] (n) {{}};"), "n.center");
        assert!(close(q.x, x), "{op}: {q:?}");
    }
    // with no previous segment, it stays at the point
    let q = point(r"\path (3,3) node[midway] (n) {x};", "n.center");
    assert!(close(q.x, 3.0), "{q:?}");
}

#[test]
fn extra_labels_by_side_and_by_angle() {
    for l in [
        "label=above:A",
        "label=below:B",
        "label=left:C",
        "label=right:D",
        "label=north:E",
        "label=south:F",
        "label=west:G",
        "label=east:H",
        "label=45:I",
        "label=-90:J",
        "label=90:K",
        "label=180:L",
        "label=0:M",
        "label=270:N",
        "label=north east:O",
        "label={[red]above:P}",
        "label=Q",
    ] {
        let c = canvas(&format!(r"\node[draw, {l}] at (0,0) {{x}};"));
        assert_eq!(texts(&c).len(), 2, "{l}");
    }
    // the label goes on its side: above is higher than the node's text, left further left
    let t = texts(&canvas(r"\node[label=above:A] at (0,0) {x};"));
    assert!(t[1].1.f > t[0].1.f, "{:?} {:?}", t[0].1, t[1].1);
    let t = texts(&canvas(r"\node[label=left:A] at (0,0) {x};"));
    assert!(t[1].1.e < t[0].1.e, "{:?} {:?}", t[0].1, t[1].1);
    let t = texts(&canvas(r"\node[label={[red]above:P}] at (0,0) {x};"));
    assert_eq!(t[1].0.color.hex(), "#ff0000");
}

#[test]
fn text_color_font_width_alignment() {
    let (r, _) = label(r"\node[text=blue, font=\bfseries\large] {Title};");
    assert_eq!(r.color.hex(), "#0000ff");
    let wrapped = r"\node[draw, text width=1cm, align=left] {a sentence with several words};";
    assert!(label(wrapped).0.lines.len() > 1);
    // the text width and the inner sep (.3333em) on both sides
    let (w, _) = border_size(wrapped);
    assert!((w - (1.0 + 2.0 * 0.3333 * 10.0 * tikra::PT)).abs() < 1e-4, "{w}");
    for a in ["right", "flush right", "flush left", "center", "justify"] {
        label(&format!(r"\node[text width=2cm, align={a}] {{word another word}};"));
    }
    label(r"\node[text centered] {center};");
    // a bare color and color=
    assert_eq!(label(r"\node[red] {reddish};").0.color.hex(), "#ff0000");
    assert_eq!(label(r"\node[color=green] {greenish};").0.color.hex(), "#00ff00");
    // a bad text width: an error in TikZ; the statement is left out, with a warning
    let (c, w) = canvas_and_warnings(r"\node[text width=zz] {no wrapping};");
    assert!(texts(&c).is_empty() && w.iter().any(|a| a.contains("zz")), "{w:?}");
}

#[test]
fn node_rotation_and_scale() {
    let (rotated, place) = label(r"\node[rotate=30, scale=2] {rotated};");
    let (plain, plain_place) = label(r"\node {rotated};");
    assert!(close(place.b.atan2(place.a).to_degrees(), 30.0), "{place:?}");
    // the text grows with the node, not only the box around it
    let grown = rotated.width() * place.factor() / (plain.width() * plain_place.factor());
    assert!(close(grown, 2.0), "{grown}");
    let (w, _) = border_size(r"\node[draw, scale=2] {rotated};");
    let (v, _) = border_size(r"\node[draw] {rotated};");
    assert!(close(w, 2.0 * v), "{w} {v}");
    canvas_and_warnings(r"\node[rotate=zz, scale=zz, pos=zz] {bad};");
    let r = no_warnings(r"\begin{tikzpicture}\node[draw, rotate=45, dashed] {x};\end{tikzpicture}");
    // the text turns with the node
    assert!(r.svg.contains("matrix(0.70711 -0.70711 0.70711 0.70711"), "{}", r.svg);
}

#[test]
fn node_syntax_errors() {
    for bad in [r"\node[draw {x};", r"\node (a {x};", r"\node at {x};", r"\node {x;"] {
        // an error or a warning, never a silent drawing
        if let Ok((_, w)) = tikra::engine::run_document(&format!("\\begin{{tikzpicture}}{bad}\\end{{tikzpicture}}")) {
            assert!(!w.is_empty(), "{bad}");
        }
    }
}

#[test]
fn draw_none_and_unnamed_coordinate() {
    let c = canvas(r"\node[draw=none] {x};");
    assert!(paths(&c).iter().all(|p| p.stroke.is_none()));
    let p = first_path(r"\node[draw=red] {x};");
    assert_eq!(p.stroke.unwrap().color.hex(), "#ff0000");
    // an unnamed coordinate draws nothing, and is no error
    let (c, w) = canvas_and_warnings(r"\path (1,1) coordinate;");
    assert!(w.is_empty(), "{w:?}");
    assert!(paths(&c).is_empty() && texts(&c).is_empty());
}

#[test]
fn text_with_sizes_circled_and_grouped_words() {
    let c = canvas(r"\node {\small small \Large large};");
    assert!(!texts(&c).is_empty());
    let r = no_warnings(r"\begin{tikzpicture} \node {$\textcircled{a}$}; \end{tikzpicture}");
    assert!(r.svg.contains("<ellipse"));
    // with text width, $…$ and {…} are not split
    let (l, _) = label(r"\node[text width=1cm] {$a + b$ {c d} e};");
    assert!(l.lines.len() >= 2);
}

#[test]
fn baseline_anchors() {
    let body =
        r"\node[draw, anchor=base west] (a) at (1,2) {xg}; \node[anchor=base] (b) at (5,2) {x}; \node[anchor=base east] (c) at (9,2) {x}; \node[anchor=text] (d) at (12,2) {x};";
    for n in ["a", "b", "c", "d"] {
        let q = point(body, &format!("{n}.base"));
        assert!(close(q.y, 2.0), "the baseline sits on the given point: {n} {q:?}");
    }
    assert!(close(point(body, "a.base west").x, 1.0));
    assert!(close(point(body, "b.base").x, 5.0));
    assert!(close(point(body, "c.base east").x, 9.0));
    assert!(close(point(body, "d.text").x, 12.0));
    // the text's baseline starts at the node's text anchor
    let t = texts(&canvas(body));
    let d: Point = point(body, "d.text");
    assert!(close(t[3].1.e, d.x) && close(t[3].1.f, d.y), "{:?}", t[3].1);
    // the "g" goes below the baseline: the center sits above it by less than the half height
    assert!(point(body, "a.center").y > 2.0);
}

#[test]
fn tex_quotes_in_node_text_are_typographic() {
    // mathra turns `` '' into “ ” inside text; the label must not warn or fall back to boxes
    let r = no_warnings(r"\begin{tikzpicture} \node {say ``hi'' and `bye'}; \end{tikzpicture}");
    assert!(r.svg.contains("<path"));
}

#[test]
fn shapes_have_their_border_paths() {
    // a rectangle: four sides; a diamond: four sides too; a circle: four quarter curves
    let r = border(r"\node[draw] {x};");
    assert_eq!((lines(&r), curves(&r)), (3, 0), "{r:?}");
    let d = border(r"\node[draw, diamond] {x};");
    assert_eq!(curves(&d), 0, "{d:?}");
    let c = border(r"\node[draw, circle] {x};");
    assert_eq!(curves(&c), 4, "{c:?}");
}
