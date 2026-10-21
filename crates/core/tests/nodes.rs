//! Nodes: shapes, anchors, positioning, extra labels, text and size.

mod common;
use common::*;
use tikra::scene::{Node, Point, Shape};

fn node_with(body: &str, text: &str) -> Node {
    let c = scene(body);
    nodes(&c).into_iter().find(|n| n.label.is_some() && body.contains(text)).cloned().unwrap()
}

#[test]
fn shapes_by_option_and_by_shape_key() {
    for (op, shape) in [
        ("circle", Shape::Circle),
        ("ellipse", Shape::Ellipse),
        ("rectangle", Shape::Rectangle),
        ("diamond", Shape::Diamond),
        ("shape=circle", Shape::Circle),
        ("shape=ellipse", Shape::Ellipse),
        ("shape=diamond", Shape::Diamond),
        ("shape=star", Shape::Rectangle),
    ] {
        assert_eq!(first_node(&format!(r"\node[{op}, draw] {{A}};")).shape, shape, "{op}");
    }
    // a named shape=coordinate draws nothing
    let c = scene(r"\node[shape=coordinate] (p) at (1,1) {};  \node[coordinate] (q) at (2,2) {};");
    assert!(nodes(&c).is_empty());
}

#[test]
fn minimum_sizes_separation_and_aspect() {
    let n = first_node(r"\node[draw, minimum width=3cm, minimum height=2cm] {};");
    assert!(close(n.half_width, 1.5) && close(n.half_height, 1.0));
    let n = first_node(r"\node[draw, circle, minimum size=2cm] {};");
    assert!(close(n.half_width, 1.0));
    let n = first_node(r"\node[draw, inner sep=0pt] {};");
    assert!(close(n.half_width, 0.0));
    let n = first_node(r"\node[draw, inner xsep=1cm, inner ysep=2cm] {};");
    assert!(close(n.half_width, 1.0) && close(n.half_height, 2.0));
    let narrow = first_node(r"\node[diamond, draw, aspect=3] {AB};");
    let wide = first_node(r"\node[diamond, draw] {AB};");
    assert!(narrow.half_width > wide.half_width);
    let n = first_node(r"\node[ellipse, draw, minimum width=4cm, minimum height=1cm] {x};");
    assert!(close(n.half_width, 2.0));
    // bad values: defaults
    first_node(r"\node[draw, inner sep=zz, inner xsep=zz, inner ysep=zz, minimum width=zz, minimum height=zz, minimum size=zz, aspect=zz] {x};");
}

#[test]
fn named_and_angle_anchors() {
    let n = first_node(r"\node[draw, minimum width=2cm, minimum height=2cm] at (0,0) {};");
    for (a, p) in [
        ("north", (0.0, 1.0)),
        ("south", (0.0, -1.0)),
        ("east", (1.0, 0.0)),
        ("west", (-1.0, 0.0)),
        ("north east", (1.0, 1.0)),
        ("south west", (-1.0, -1.0)),
        ("center", (0.0, 0.0)),
        ("", (0.0, 0.0)),
        ("base", (0.0, 0.0)),
        ("mid", (0.0, 0.0)),
        ("text", (0.0, 0.0)),
        ("0", (1.0, 0.0)),
    ] {
        let q = n.anchor(a).unwrap();
        assert!(close_to((q.x, q.y), p), "{a}: {q:?}");
    }
    assert!(n.anchor("nothing").is_none());
    let c = first_node(r"\node[circle, draw, minimum size=2cm] at (0,0) {};");
    let q = c.anchor("north east").unwrap();
    assert!(close(q.x.hypot(q.y), 1.0), "the circle's corner is on its border: {q:?}");
    // a coordinate's anchor and degenerate directions
    let mut p = c.clone();
    p.shape = Shape::Coordinate;
    assert_eq!(p.border_toward(Point::new(1.0, 0.0)), p.center);
    assert_eq!(c.border_toward(Point::new(0.0, 0.0)), c.center);
    let l = first_node(r"\node[diamond, draw, minimum size=2cm] at (0,0) {};");
    assert!(l.border_at_angle(45.0).x > 0.0);
    // a rectangle in a purely vertical direction
    assert!(close(n.border_toward(Point::new(0.0, 1.0)).y, 1.0));
    assert!(close(n.border_toward(Point::new(1.0, 0.0)).x, 1.0));
}

#[test]
fn anchor_and_relative_placement() {
    let base = r"\node[draw, minimum size=1cm, inner sep=0pt] at (0,0) {};";
    let n = first_node(&base.replace("draw,", "draw, anchor=west,"));
    assert!(close(n.center.x, 0.5), "{:?}", n.center);
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
        let n = first_node(&base.replace("draw,", &format!("draw, {pos},")));
        assert!(n.center.x * sign.0 >= 0.0 && n.center.y * sign.1 >= 0.0, "{pos}: {:?}", n.center);
        let n = first_node(&base.replace("draw,", &format!("draw, {pos}=1cm,")));
        assert!(n.center.x.abs() + n.center.y.abs() > 0.9, "{pos}=1cm: {:?}", n.center);
    }
    let n = first_node(&base.replace("draw,", "draw, xshift=1cm, yshift=-1cm, above=zz,"));
    assert!(close(n.center.x, 1.0));
}

#[test]
fn positioning_of_with_distances() {
    let body = |op: &str| {
        format!(r"\node[draw, minimum size=1cm, inner sep=0pt] (a) at (0,0) {{}}; \coordinate (p) at (5,5); \node[draw, minimum size=1cm, inner sep=0pt, {op}] (b) {{}};")
    };
    let b = |op: &str| {
        let c = scene(&body(op));
        nodes(&c)[1].center
    };
    assert!(close(b("right=of a").x, 2.0), "1cm default distance between borders");
    assert!(close(b("below=2cm of a").y, -3.0));
    let q = b("below left=1cm and 2cm of a");
    assert!(close(q.x, -3.0) && close(q.y, -2.0), "{q:?}");
    assert!(close(b("above=of a.north").y, 2.0));
    assert!(b("above right=of p").x > 5.0, "a coordinate as the target");
    for op in ["left=of a", "above left=of a", "above right=of a", "below right=of a", "above=1cm and 2cm of a"] {
        b(op);
    }
    let c = scene(r"\begin{tikzpicture}[node distance=2cm and 3cm] \end{tikzpicture}");
    assert!(c.elements.is_empty());
    let (_, w) = scene_and_warnings(r"\node[right=of nobody] {x};");
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
    // an invalid node distance: 1cm
    no_warnings(r"\begin{tikzpicture}[node distance=zz] \node (a) {A}; \node[below=of a] {B}; \end{tikzpicture}");
}

#[test]
fn position_along_the_path() {
    for (op, x) in
        [("midway", 1.0), ("near start", 0.5), ("near end", 1.5), ("at start", 0.0), ("at end", 2.0), ("very near start", 0.25), ("very near end", 1.75), ("pos=0.1", 0.2)]
    {
        let c = scene(&format!(r"\draw (0,0) -- (2,0) node[{op}, inner sep=0pt] {{}};"));
        let n = nodes(&c);
        assert!(close(n[0].center.x, x), "{op}: {:?}", n[0].center);
    }
    // with no previous segment, it stays at the point
    let c = scene(r"\path (3,3) node[midway] {x};");
    assert!(close(nodes(&c)[0].center.x, 3.0));
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
        let c = scene(&format!(r"\node[draw, {l}] at (0,0) {{x}};"));
        assert_eq!(nodes(&c).len(), 2, "{l}");
    }
    // an anchor that does not exist: the center
    let c = scene(r"\node[draw, label=side:R] at (0,0) {x};");
    assert_eq!(nodes(&c).len(), 2);
}

#[test]
fn text_color_font_width_alignment() {
    let n = node_with(r"\node[text=blue, font=\bfseries\large] {Title};", "Title");
    let r = n.label.as_ref().unwrap();
    assert_eq!(r.color.hex(), "#0000ff");
    let wrapped = node_with(r"\node[text width=1cm, align=left] {a sentence with several words};", "sentence");
    assert!(wrapped.label.as_ref().unwrap().lines.len() > 1);
    assert!(close(wrapped.half_width * 2.0, 1.0 + 2.0 * 0.3333 * 0.3515));
    for a in ["right", "flush right", "flush left", "center", "justify"] {
        node_with(&format!(r"\node[text width=2cm, align={a}] {{word another word}};"), "word");
    }
    node_with(r"\node[text centered] {center};", "center");
    // a bare color and color=
    let n = node_with(r"\node[red] {reddish};", "reddish");
    assert_eq!(n.label.as_ref().unwrap().color.hex(), "#ff0000");
    let n = node_with(r"\node[color=green] {greenish};", "greenish");
    assert_eq!(n.label.as_ref().unwrap().color.hex(), "#00ff00");
    // a bad text width: no wrapping
    node_with(r"\node[text width=zz] {no wrapping};", "wrapping");
}

#[test]
fn node_rotation_and_scale() {
    let n = node_with(r"\node[rotate=30, scale=2] {rotated};", "rotated");
    assert!(close(n.rotation, 30.0));
    let m = node_with(r"\node {rotated};", "rotated");
    assert!(n.half_width > m.half_width);
    // the text grows with the node, not only the box around it
    let (a, b) = (n.label.as_ref().unwrap(), m.label.as_ref().unwrap());
    assert!(close(a.width(), 2.0 * b.width()) && close(a.line_height, 2.0 * b.line_height), "{} {}", a.width(), b.width());
    node_with(r"\node[rotate=zz, scale=zz, pos=zz] {bad};", "bad");
    let r = no_warnings(r"\begin{tikzpicture}\node[draw, rotate=45, dashed] {x};\end{tikzpicture}");
    assert!(r.svg.contains("rotate(-45"));
}

#[test]
fn node_syntax_errors() {
    for bad in [r"\node[draw {x};", r"\node (a {x};", r"\node at {x};", r"\node {x;"] {
        let (_, w) = scene_and_warnings(bad);
        assert!(!w.is_empty(), "{bad}");
    }
    // a node with no text and no braces
    let c = scene(r"\node[draw] (a) at (1,1);");
    assert_eq!(nodes(&c).len(), 1);
}

#[test]
fn draw_none_and_unnamed_coordinate() {
    let n = first_node(r"\node[draw=none] {x};");
    assert!(n.style.stroke.is_none());
    let n = first_node(r"\node[draw=red] {x};");
    assert_eq!(n.style.stroke.unwrap().hex(), "#ff0000");
    // an unnamed coordinate becomes an empty element
    let c = scene(r"\path (1,1) coordinate;");
    assert_eq!(nodes(&c).len(), 1);
}

#[test]
fn text_with_sizes_circled_and_grouped_words() {
    let n = node_with(r"\node {\small small \Large large};", "small");
    assert!(n.label.is_some());
    let r = no_warnings(r"\begin{tikzpicture} \node {$\textcircled{a}$}; \end{tikzpicture}");
    assert!(r.svg.contains("<ellipse"));
    // with text width, $…$ and {…} are not split
    let n = node_with(r"\node[text width=1cm] {$a + b$ {c d} e};", "e");
    assert!(n.label.as_ref().unwrap().lines.len() >= 2);
}

#[test]
fn baseline_anchors() {
    let c = scene(
        r"\node[draw, anchor=base west] (a) at (1,2) {xg}; \node[anchor=base] (b) at (5,2) {x}; \node[anchor=base east] (c) at (9,2) {x}; \node[anchor=text] (d) at (12,2) {x}; \coordinate (e) at (0,0);",
    );
    let ns = nodes(&c);
    let base = |n: &Node| n.anchor("base").unwrap();
    for n in &ns {
        assert!((base(n).y - 2.0).abs() < 1e-6, "the baseline sits on the given point: {:?}", base(n));
    }
    assert!((ns[0].anchor("base west").unwrap().x - 1.0).abs() < 1e-6);
    assert!((ns[1].anchor("base").unwrap().x - 5.0).abs() < 1e-6);
    assert!((ns[2].anchor("base east").unwrap().x - 9.0).abs() < 1e-6);
    assert!((ns[3].anchor("text").unwrap().x - 12.0).abs() < 1e-6);
    // the "g" goes below the baseline: the center sits above it by less than the half height
    assert!(ns[0].center.y > 2.0);
}

#[test]
fn tex_quotes_in_node_text_are_typographic() {
    // mathra turns `` '' into “ ” inside text; the label must not warn or fall back to boxes
    let r = no_warnings(r"\begin{tikzpicture} \node {say ``hi'' and `bye'}; \end{tikzpicture}");
    assert!(r.svg.contains("<path"));
}
