//! The path operations of TikZ one by one, as `\draw` and `\path` read them: their less common
//! forms (curves to and from nodes, `to` paths with their own controls, SVG data, parabolas, `let`,
//! plots from files and the basic layer's plot handlers), and what is read when a path is written
//! wrong.

mod common;
use common::*;
use tikra::canvas::Item;
use tikra::scene::Segment;

/// The error a statement of `body` stopped with: it is ignored, with the error as a warning.
fn error(body: &str) -> String {
    let w = warnings(body);
    w.into_iter().find(|w| w.contains("statement ignored")).unwrap_or_else(|| panic!("no error for {body}"))
}

/// The warnings of a body.
fn warnings(body: &str) -> Vec<String> {
    canvas_and_warnings(body).1
}

const PT: f64 = 2.54 / 72.27;

fn near(a: (f64, f64), b: (f64, f64)) -> bool {
    (a.0 - b.0).abs() < 1e-4 && (a.1 - b.1).abs() < 1e-4
}

fn moves(segs: &[Segment]) -> usize {
    segs.iter().filter(|s| matches!(s, Segment::MoveTo(_))).count()
}

fn curves(segs: &[Segment]) -> usize {
    segs.iter().filter(|s| matches!(s, Segment::CurveTo(..))).count()
}

// ---------------------------------------------------------------- what is read wrong

#[test]
fn fragments_that_are_not_operations() {
    for (body, fragment) in [
        (r"\draw (0,0) -x (1,0);", "unrecognized path fragment: '-'"),
        (r"\draw (0,0) |x (1,0);", "unrecognized path fragment: '|'"),
        (r"\draw (0,0) .x (1,0);", "unrecognized path fragment: '.'"),
        (r"\draw (0,0) l (1,0);", "unrecognized path fragment: 'l'"),
        (r"\draw (0,0) l-1 (1,0);", "unrecognized path fragment: 'l'"),
        (r"\draw (0,0) lindenmayer (1,0);", "unrecognized path fragment: 'lindenmayer'"),
        (r"\draw (0,0) lindenmayer 1 (1,0);", "unrecognized path fragment: 'lindenmayer'"),
    ] {
        let (c, w) = canvas_and_warnings(body);
        assert!(w.iter().any(|w| w == fragment), "{body}: {w:?}");
        // the rest of the path is skipped: no line is drawn
        assert!(paths(&c).iter().all(|d| points(&d.segments).len() < 2), "{body}");
    }
    // a word longer than any operation is cut at 65 letters
    let long = "x".repeat(80);
    let w = warnings(&format!(r"\draw (0,0) {long} -- (1,0);"));
    assert_eq!(w, [format!("unrecognized path fragment: '{}'", "x".repeat(65))]);
}

#[test]
fn points_that_are_missing() {
    assert!(error(r"\draw (0,0) -- +x;").contains("'+' without a point"));
    assert!(error(r"\draw (0,0) .. controls (1,1) .. cat;").contains("controls without a point"));
    assert!(error(r"\draw (0,0) foreach \i in {1} {grid};").contains("grid without a point"));
    assert!(error(r"\draw (0,0) foreach \i in {1} {plot};").contains("plot without an expression"));
    assert!(error(r"\draw (0,0) foreach \i in {1} {svg};").contains("svg without a {path}"));
    assert!(error(r"\draw plot cosine;").contains("plot cosine not supported"));
    assert!(error(r"\draw plot fun {x};").contains("plot fun not supported"));
}

#[test]
fn a_path_cut_short_by_the_end_of_the_input() {
    let (c, w) = tikra::engine::run_document(r"\begin{tikzpicture}\draw (0,0) -- (1,0)").unwrap();
    assert!(w.iter().any(|w| w == "path without ';' at the end"), "{w:?}");
    assert_eq!(points(&paths(&c)[0].segments).len(), 2);
    let (_, w) = tikra::engine::run_document(r"\begin{tikzpicture}\draw").unwrap();
    assert!(w.iter().any(|w| w == "path without ';' at the end"), "{w:?}");
    // a relative point with nothing after its `+`
    let (_, w) = tikra::engine::run_document(r"\begin{tikzpicture}\draw (0,0) -- +").unwrap();
    assert!(w.iter().any(|w| w.contains("'+' without a point")), "{w:?}");
}

// ---------------------------------------------------------------- curves and cycles

#[test]
fn a_curve_back_to_the_start() {
    // `.. cycle` ends at the subpath's start and closes it
    let t = first_path(r"\draw (0,0) -- (2,0) .. controls (2,2) and (0,2) .. cycle;");
    assert!(matches!(t.segments.last(), Some(Segment::Close)), "{:?}", t.segments);
    let Some(Segment::CurveTo(_, _, end)) = t.segments.get(t.segments.len() - 2).copied() else { panic!("{:?}", t.segments) };
    assert!(close_to((end.x, end.y), (0.0, 0.0)));
    // a path that goes on after the cycle starts there again
    let t = first_path(r"\draw (0,0) -- (2,0) .. controls (2,2) and (0,2) .. cycle -- (1,1);");
    assert!(close_to(*points(&t.segments).last().unwrap(), (1.0, 1.0)));
}

#[test]
fn nodes_on_a_curve_before_its_controls() {
    let c = canvas(r"\draw (0,0) .. node[above] (m) {m} coordinate (k) controls (1,1) and (3,1) .. (4,0);");
    // both are halfway along the curve, at (2, 0.75)
    for name in ["m.south", "k"] {
        let p = point(r"\path (0,0) .. node[above] (m) {m} coordinate (k) controls (1,1) and (3,1) .. (4,0);", name);
        assert!(near((p.x, p.y), (2.0, 0.75)), "{name}: {p:?}");
    }
    assert_eq!(texts(&c).len(), 1);
}

#[test]
fn a_curve_without_its_closing_dots() {
    // `.. controls (c) (b)`: the dots before the end may be left out
    let t = first_path(r"\draw (0,0) .. controls (1,1) (2,0);");
    assert_eq!(curves(&t.segments), 1);
    assert!(close_to(*points(&t.segments).last().unwrap(), (2.0, 0.0)));
}

#[test]
fn a_curve_from_a_node_starts_on_its_border() {
    let body = r"\node[draw, minimum size=1cm, outer sep=0pt] (a) at (2,0) {}; \draw (0,0) -- (a) .. controls (2,2) .. (4,0);";
    let t = paths(&canvas(body)).into_iter().find(|d| curves(&d.segments) == 1).unwrap();
    // the line stops at the node; the curve starts again on its border toward its control
    assert_eq!(moves(&t.segments), 2, "{:?}", t.segments);
    let Some(Segment::MoveTo(m)) = t.segments.iter().filter(|s| matches!(s, Segment::MoveTo(_))).nth(1).copied() else { panic!() };
    assert!(near((m.x, m.y), (2.0, 0.5)), "{m:?}");
}

#[test]
fn a_curve_after_a_cycle() {
    let t = first_path(r"\draw (0,0) -- (1,0) -- (1,1) -- cycle .. controls (2,2) .. (3,0);");
    let n = t.segments.len();
    assert!(matches!(t.segments[n - 2], Segment::Close), "{:?}", t.segments);
    assert!(matches!(t.segments[n - 1], Segment::CurveTo(..)), "{:?}", t.segments);
}

#[test]
fn turn_after_a_cycle_goes_on_from_the_start() {
    // after `cycle` the path is at its start: the line to (0,2) goes up, and `turn` keeps going up
    let t = first_path(r"\draw (0,0) -- (1,0) -- cycle -- (0,2) -- ([turn]0:1);");
    assert!(close_to(*points(&t.segments).last().unwrap(), (0.0, 3.0)), "{:?}", points(&t.segments));
}

// ---------------------------------------------------------------- elbows, to and edges

#[test]
fn an_elbow_without_a_start() {
    // with no point before it, the elbow starts at its corner
    let t = first_path(r"\draw -| (1,1);");
    assert_eq!(points(&t.segments), [(1.0, 0.0), (1.0, 1.0)]);
}

#[test]
fn to_with_bounded_distances_and_explicit_controls() {
    // `max distance` shortens the controls: 0.3915 × 4cm would be more than 0.5cm
    let t = first_path(r"\draw (0,0) to[out=90, in=90, max distance=0.5cm] (4,0);");
    let Segment::CurveTo(c1, c2, _) = t.segments[1] else { panic!("{:?}", t.segments) };
    assert!(near((c1.x, c1.y), (0.0, 0.5)) && near((c2.x, c2.y), (4.0, 0.5)), "{c1:?} {c2:?}");
    // controls given as points, absolute or from their end
    let t = first_path(r"\draw (0,0) to[out control={+(0,1)}, in control={(3,2)}] (2,0);");
    let Segment::CurveTo(c1, c2, _) = t.segments[1] else { panic!("{:?}", t.segments) };
    assert!(near((c1.x, c1.y), (0.0, 1.0)) && near((c2.x, c2.y), (3.0, 2.0)), "{c1:?} {c2:?}");
}

#[test]
fn nodes_in_a_users_to_path() {
    // `\tikztonodes` puts the nodes given to the `to`, halfway along by default
    let body = r"\tikzset{straight/.style={to path={-- (\tikztotarget) \tikztonodes}}} \draw (0,0) to[straight] node[above] (n) {n} coordinate (k) (2,0);";
    let c = canvas(body);
    assert_eq!(texts(&c).len(), 1);
    let k = point(body, "k");
    assert!(near((k.x, k.y), (1.0, 0.0)), "{k:?}");
}

#[test]
fn a_bent_to_from_a_node_the_path_came_to() {
    let body = r"\node[draw, minimum size=1cm] (a) at (2,0) {}; \draw (0,0) -- (a) to[bend left] (4,0);";
    let t = paths(&canvas(body)).into_iter().find(|d| curves(&d.segments) == 1).unwrap();
    // a new piece from a's border
    assert_eq!(moves(&t.segments), 2, "{:?}", t.segments);
    // and a bent `to` with nothing before it starts at the origin
    let t = first_path(r"\draw to[bend left] (2,0);");
    assert!(close_to(points(&t.segments)[0], (0.0, 0.0)), "{:?}", t.segments);
    assert_eq!(curves(&t.segments), 1);
}

#[test]
fn an_edge_to_path_with_options_in_braces() {
    // the options a to path starts with are the edge's own path's
    let c = canvas(r"\draw (0,0) edge[to path={[dash pattern={on 1pt off 1pt}] -- (\tikztotarget)}] (2,0);");
    assert!(paths(&c).iter().any(|d| d.stroke.as_ref().is_some_and(|s| s.dash.is_some())));
}

#[test]
fn an_edge_from_parent_with_nodes_on_it() {
    let body = r"\node (r) {r} child { node (c) {c} edge from parent node[left] {e} coordinate (m) };";
    let c = canvas(body);
    let t: Vec<_> = texts(&c);
    assert_eq!(t.len(), 3);
    let m = point(body, "m");
    let (r, ch) = (point(body, "r"), point(body, "c"));
    assert!(near((m.x, m.y), ((r.x + ch.x) / 2.0, (r.y + ch.y) / 2.0)), "{m:?}");
    // a child after `edge from parent` is the next child of the same node
    let body = r"\node (r) {r} child { node (c) {c} edge from parent child { node (d) {d} } };";
    assert_eq!(texts(&canvas(body)).len(), 3);
}

// ---------------------------------------------------------------- nodes on paths

#[test]
fn node_also_adds_options_to_a_node() {
    let c = canvas(r"\node (a) {A}; \path node also [label=above:L] (a);");
    assert_eq!(texts(&c).len(), 2);
}

#[test]
fn options_after_a_coordinates_name_are_the_paths() {
    let t = first_path(r"\draw coordinate (a) [red] (0,0) -- (1,0);");
    assert_eq!(t.stroke.unwrap().color.hex(), tikra::color::Color::rgb(1.0, 0.0, 0.0).hex());
}

#[test]
fn node_contents_and_nodes_without_text() {
    let c = canvas(r"\path (0,0) node[node contents=hello];");
    assert_eq!(texts(&c).len(), 1);
    let w = warnings(r"\draw (0,0) node[draw];");
    assert_eq!(w, ["node without text: '{…}' missing"]);
}

#[test]
fn a_node_text_in_bgroup_and_egroup() {
    let c = canvas(r"\path (0,0) node\bgroup a{b}\egroup;");
    assert_eq!(texts(&c).len(), 1);
}

#[test]
fn a_coordinate_followed_by_an_arc() {
    // `a` after a coordinate's name is not `at`: here it begins `arc`
    let body = r"\draw (1,0) coordinate (c) arc (0:90:1);";
    let t = first_path(body);
    assert!(close_to(*points(&t.segments).last().unwrap(), (0.0, 1.0)), "{:?}", points(&t.segments));
    let c = point(body, "c");
    assert!(near((c.x, c.y), (1.0, 0.0)));
}

#[test]
fn node_on_a_path_written_as_a_command() {
    let c = canvas(r"\draw (0,0) -- (1,0) \node[right] {a};");
    assert_eq!(texts(&c).len(), 1);
}

#[test]
fn a_pic_between_an_operation_and_its_target() {
    let body = r"\tikzset{dot/.pic={\fill (0,0) circle (2pt);}} \draw (0,0) -- pic {dot} (2,0);";
    let c = canvas(body);
    // the pic halfway along the line
    let dot = paths(&c).into_iter().find(|d| d.fill.is_some()).unwrap();
    let (lo, hi) = bounds(&dot.segments);
    assert!(near(((lo.x + hi.x) / 2.0, (lo.y + hi.y) / 2.0), (1.0, 0.0)), "{lo:?} {hi:?}");
}

#[test]
fn an_animated_node_on_a_path() {
    // at the snapshot's time, halfway: half opaque
    let c = canvas(r#"\begin{scope}[make snapshot of=1s] \path (0,0) node :fill opacity = { 0s = "0", 2s = "1" } [fill=red] {a}; \end{scope}"#);
    assert!((paths(&c)[0].fill.as_ref().unwrap().opacity - 0.5).abs() < 1e-9);
}

// ---------------------------------------------------------------- options in the middle of a path

#[test]
fn options_in_the_middle_are_the_whole_paths() {
    // the node's border is drawn with the path's final color and dashes
    let c = canvas(r"\draw (0,0) node[draw] {a} -- (1,0) [red, dashed];");
    let red = tikra::color::Color::rgb(1.0, 0.0, 0.0).hex();
    for d in paths(&c) {
        let s = d.stroke.unwrap();
        assert_eq!(s.color.hex(), red);
        assert!(s.dash.is_some());
    }
}

#[test]
fn a_color_in_a_group_of_the_path_is_the_texts() {
    let c = canvas(r"\draw (0,0) {[red] -- (1,0) node[right] {a}};");
    let red = tikra::color::Color::rgb(1.0, 0.0, 0.0).hex();
    assert_ne!(paths(&c)[0].stroke.as_ref().unwrap().color.hex(), red);
    assert_eq!(texts(&c)[0].0.color.hex(), red);
}

#[test]
fn insert_path_goes_on_where_the_options_are() {
    let t = first_path(r"\draw (0,0) [insert path={-- (1,0)}] -- (1,1) [insert path={}] -- (0,1);");
    assert_eq!(points(&t.segments), [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
}

#[test]
fn a_canvas_transformation_of_a_path_in_a_scope_ends_with_it() {
    let c = canvas(r"\begin{scope} \draw[transform canvas={shift={(1,0)}}] (0,0) -- (1,0); \draw (0,1) -- (1,1); \end{scope}");
    let i = c.items.iter().position(|i| matches!(i, Item::Transform { .. })).unwrap();
    // the transformed path, its end, then the next path untransformed
    assert!(matches!(c.items[i + 1], Item::Path { .. }));
    assert!(matches!(c.items[i + 2], Item::End));
    assert!(matches!(c.items[i + 3], Item::Path { .. }));
}

// ---------------------------------------------------------------- \pgfextra

#[test]
fn pgfextra_runs_code_in_the_middle_of_a_path() {
    // \pgfextra code \endpgfextra
    let t = first_path(r"\draw (0,0) \pgfextra \def\x{2} \endpgfextra -- (\x,0);");
    assert_eq!(points(&t.segments), [(0.0, 0.0), (2.0, 0.0)]);
    // a path built on the basic layer becomes the path
    let t = first_path(r"\draw \pgfextra{\pgfpathmoveto{\pgfpointorigin}\pgfpathlineto{\pgfpoint{1cm}{0cm}}};");
    assert_eq!(points(&t.segments), [(0.0, 0.0), (1.0, 0.0)]);
}

// ---------------------------------------------------------------- decorate

#[test]
fn decorate_needs_a_brace_and_something_to_decorate() {
    let w = warnings(r"\draw decorate (0,0) -- (1,0);");
    assert!(w.iter().any(|w| w == "a decoration must begin with a brace: decorate ignored"), "{w:?}");
    // nothing to decorate: nothing drawn, nothing said
    let (c, w) = canvas_and_warnings(r"\draw[decoration=zigzag] decorate {};");
    assert!(w.is_empty(), "{w:?}");
    assert!(paths(&c).is_empty());
    // a wrong fragment inside the decoration skips the rest of it
    let w = warnings(r"\draw[decoration=zigzag] decorate { (0,0) -x (1,0) } -- (2,0);");
    assert!(w.iter().any(|w| w == "unrecognized path fragment: '-'"), "{w:?}");
}

#[test]
fn a_node_at_the_end_of_a_decorations_path() {
    let (c, w) = canvas_and_warnings(r"\draw[decoration=zigzag] decorate { (0,0) -- (1,0) node };");
    assert!(w.iter().any(|w| w == "node without text: '{…}' missing"), "{w:?}");
    assert!(!paths(&c).is_empty());
}

// ---------------------------------------------------------------- Lindenmayer systems

#[test]
fn lindenmayer_systems_on_a_path() {
    let decl = r"\pgfdeclarelindenmayersystem{hook}{\rule{F -> F+F}}";
    // `l-system` with its options after it
    let t = first_path(&format!(r"{decl} \draw (0,0) l-system[l-system={{hook, axiom=F, order=1, step=1cm, angle=90}}];"));
    let p = points(&t.segments);
    assert!(near(p[p.len() - 2], (1.0, 0.0)) && near(p[p.len() - 1], (1.0, 1.0)), "{p:?}");
    let t = first_path(&format!(r"{decl} \draw[l-system={{hook, axiom=F, order=1, step=1cm, angle=90}}] (0,0) lindenmayer system -- (0,3);"));
    assert!(near(*points(&t.segments).last().unwrap(), (0.0, 3.0)));
}

// ---------------------------------------------------------------- SVG path data

#[test]
fn svg_path_data_commands() {
    let pt = |x: f64, y: f64| (x * PT, y * PT);
    // lines: absolute and relative, horizontal and vertical, a number with its own minus sign
    let t = first_path(r"\draw svg {M 0 0 L 10 0 v 10 h-10 V 20 H 5 l5-5 z};");
    let p = points(&t.segments);
    for (got, want) in p.iter().zip([pt(0.0, 0.0), pt(10.0, 0.0), pt(10.0, 10.0), pt(0.0, 10.0), pt(0.0, 20.0), pt(5.0, 20.0), pt(10.0, 15.0)]) {
        assert!(near(*got, want), "{got:?} {want:?}");
    }
    assert!(matches!(t.segments.last(), Some(Segment::Close)));
    // curves: cubic, smooth cubic, quadratic, smooth quadratic
    let t = first_path(r#"\draw svg "M 0 0 C 0 10 10 10 10 0 S 20 -10 20 0 Q 25 10 30 0 T 40 0 c 0 5 5 5 5 0 s 5 -5 5 0 q 5 5 10 0 t 10 0";"#);
    assert_eq!(curves(&t.segments), 8);
    // T after Q mirrors the control: (35, -10)
    let Segment::CurveTo(c1, _, end) = t.segments[4] else { panic!("{:?}", t.segments) };
    assert!(near((end.x, end.y), pt(40.0, 0.0)));
    assert!(near((c1.x, c1.y), pt(30.0 + 5.0 * 2.0 / 3.0, -10.0 * 2.0 / 3.0)), "{c1:?}");
    // an elliptical arc, as pgf turns it with y up
    let t = first_path(r"\draw svg {M 0 0 A 10 10 0 0 1 20 0};");
    let (lo, hi) = bounds(&t.segments[1..]);
    assert!(near((hi.x, lo.y), pt(20.0, -10.0)) || near((hi.x, hi.y), pt(20.0, 10.0)), "{lo:?} {hi:?}");
    // an unknown command ends the data there
    let t = first_path(r"\draw svg {M 0 0 L 10 0 X 5 5 L 0 10};");
    assert_eq!(points(&t.segments).len(), 2);
}

#[test]
fn svg_arcs_on_the_basic_layer() {
    // \pgfpatharcto: radii, rotation, large arc, counterclockwise, target
    let half = |large: u8, ccw: u8, rx: &str| {
        let c = canvas(&format!(r"\pgfpathmoveto{{\pgfpointorigin}}\pgfpatharcto{{{rx}}}{{{rx}}}{{0}}{{{large}}}{{{ccw}}}{{\pgfpoint{{20pt}}{{0pt}}}}\pgfusepath{{stroke}}"));
        bounds(&paths(&c)[0].segments)
    };
    // counterclockwise from (0,0) to (20pt,0) about (10pt,0): below the line
    let (lo, hi) = half(0, 1, "10pt");
    assert!((lo.y + 10.0 * PT).abs() < 1e-3 && hi.y.abs() < 1e-3, "{lo:?} {hi:?}");
    // clockwise: above it
    let (lo, hi) = half(0, 0, "10pt");
    assert!((hi.y - 10.0 * PT).abs() < 1e-3 && lo.y.abs() < 1e-3, "{lo:?} {hi:?}");
    // radii too small for the distance grow to reach it
    let (lo, _) = half(1, 1, "5pt");
    assert!((lo.y + 10.0 * PT).abs() < 1e-3, "{lo:?}");
    // a radius of zero: a line
    let c = canvas(r"\pgfpathmoveto{\pgfpointorigin}\pgfpatharcto{0pt}{0pt}{0}{0}{1}{\pgfpoint{20pt}{0pt}}\pgfusepath{stroke}");
    assert!(matches!(paths(&c)[0].segments[1], Segment::LineTo(_)));
    // \pgfpathsvg: SVG path data on the basic layer
    let c = canvas(r"\pgfpathmoveto{\pgfpoint{10pt}{0pt}}\pgfpathsvg{l 10 0 a 5 5 0 1 0 10 0}\pgfusepath{stroke}");
    let s = &paths(&c)[0].segments;
    assert!(near(points(s)[1], (20.0 * PT, 0.0)), "{s:?}");
    assert!(curves(s) >= 2);
}

// ---------------------------------------------------------------- arcs, circles, grids

#[test]
fn arcs_by_end_and_delta_angles() {
    // the end angle and the angle it spans: from 0 to 90
    let t = first_path(r"\draw (1,0) arc[end angle=90, delta angle=90, radius=1];");
    assert!(close_to(*points(&t.segments).last().unwrap(), (0.0, 1.0)), "{:?}", points(&t.segments));
    // a start angle alone: to the default end at 90
    let t = first_path(r"\draw (1,0) arc[start angle=0, radius=1];");
    assert!(close_to(*points(&t.segments).last().unwrap(), (0.0, 1.0)), "{:?}", points(&t.segments));
}

#[test]
fn circles_in_a_foreach_with_the_radius_of_the_path() {
    let t = first_path(r"\draw[radius=1] foreach \p in {(0,0),(3,0)} {\p circle};");
    assert_eq!(curves(&t.segments), 8);
    let (lo, hi) = bounds(&t.segments);
    assert!(close(lo.x, -1.0) && close(hi.x, 4.0), "{lo:?} {hi:?}");
}

// ---------------------------------------------------------------- parabolas

#[test]
fn parabolas() {
    // without a bend, the vertex at the start
    let t = first_path(r"\draw (0,0) parabola (2,4);");
    assert_eq!(curves(&t.segments), 1);
    let Segment::CurveTo(c1, c2, _) = t.segments[1] else { panic!() };
    assert!(near((c1.x, c1.y), (1.0, 0.0)) && near((c2.x, c2.y), (1.775, 3.1)), "{c1:?} {c2:?}");
    // the vertex at the end
    let t = first_path(r"\draw (0,0) parabola[bend at end] (2,4);");
    assert_eq!(curves(&t.segments), 1);
    // a bend given absolutely and from the start
    for body in [r"\draw (0,0) parabola bend (1,1) (2,0);", r"\draw (0,0) parabola bend +(1,1) (2,0);"] {
        let t = first_path(body);
        assert_eq!(curves(&t.segments), 2, "{body}");
        let Segment::CurveTo(_, _, b) = t.segments[1] else { panic!() };
        assert!(near((b.x, b.y), (1.0, 1.0)), "{body}: {b:?}");
    }
    // other options are not the parabola's
    let t = first_path(r"\draw (0,0) parabola[red] (2,4);");
    assert_eq!(t.stroke.unwrap().color.hex(), tikra::color::Color::BLACK.hex());
    // with no point before it, it starts at the origin
    let t = first_path(r"\draw parabola (1,1);");
    assert!(close_to(points(&t.segments)[0], (0.0, 0.0)));
}

// ---------------------------------------------------------------- let

#[test]
fn let_with_numbers() {
    let t = first_path(r"\draw let \p1 = (3,4), \n1 = {veclen(\x1,\y1)}, \n2 = 2 in (0,0) -- (\n1,\n2);");
    assert!(near(*points(&t.segments).last().unwrap(), (5.0, 2.0)), "{:?}", points(&t.segments));
    // in a foreach body inside the path
    let t = first_path(r"\draw (0,0) foreach \i in {1} { let \n1 = {\i*2} in -- (\n1,0) };");
    assert!(near(*points(&t.segments).last().unwrap(), (2.0, 0.0)), "{:?}", points(&t.segments));
}

#[test]
fn let_written_wrong() {
    for (body, message) in [
        (r"\draw let \p1 = (0,0) inx (0,0);", "let: 'inx' unexpected"),
        (r"\draw let 5 in (0,0);", "let: '5' unexpected"),
        (r"\draw let \p1 (0,0) in (0,0);", "let: '=' missing"),
        (r"\draw let \p1 = 3 in (0,0);", "let \\p: a point is missing"),
        (r"\draw let \q1 = 3 in (0,0);", "let: \\q is not \\p or \\n"),
    ] {
        assert!(error(body).contains(message), "{body}: {}", error(body));
    }
}

// ---------------------------------------------------------------- foreach on a path

#[test]
fn foreach_on_a_path() {
    // a body of one token, a macro of path operations
    let t = first_path(r"\def\seg{-- (\i,1)} \draw (0,0) foreach \i in {1,2} \seg;");
    assert_eq!(points(&t.segments), [(0.0, 0.0), (1.0, 1.0), (2.0, 1.0)]);
    // with options: count
    let t = first_path(r"\draw (0,0) foreach [count=\k] \i in {5,6} { -- (\k,\i) };");
    assert_eq!(points(&t.segments), [(0.0, 0.0), (1.0, 5.0), (2.0, 6.0)]);
}

#[test]
fn nodes_for_each_item_and_each_pair() {
    let c = canvas(r"\path node foreach \x in {1,2} foreach \y in {1,2} [draw] at (\x,\y) {.};");
    assert_eq!(texts(&c).len(), 4);
    let body = r"\path coordinate foreach \x/\n in {1/a,2/b} (c\n) at (\x,0);";
    let b = point(body, "cb");
    assert!(near((b.x, b.y), (2.0, 0.0)));
}

// ---------------------------------------------------------------- plots

/// A folder tikra reads files from, with `name` in it.
fn data_file(name: &str, text: &str) {
    let dir = std::env::temp_dir().join(format!("tikra-path-operations-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(name), text).unwrap();
    std::env::set_var("TIKRA_INPUT_DIR", format!("{}:.", dir.display()));
}

#[test]
fn plots_from_files() {
    data_file("line.table", "# x y\n0 0\n\n1 1\n% note\n2 0\nnot numbers\n");
    let t = first_path(r"\draw plot file {line.table};");
    assert_eq!(points(&t.segments), [(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)]);
    // gnuplot's table of a function, by `prefix` and `id`; `u` breaks the line
    data_file("fig.sine.table", "0 0 i\n1 1 i\n2 2 u\n3 1 i\n4 0 i\n");
    let t = first_path(r"\draw plot[prefix=fig., id=sine] function {sin(x)};");
    assert_eq!(moves(&t.segments), 2, "{:?}", t.segments);
    data_file("plot.table", "0 0 u\n");
    let (c, _) = canvas_and_warnings(r"\draw plot function {x} -- (1,1);");
    assert!(paths(&c).iter().all(|d| !d.segments.is_empty()));
    assert!(error(r"\draw plot file {no-such.table};").contains("plot file: file 'no-such.table' not found"));
}

#[test]
fn only_marks_leave_the_path_at_the_last_point() {
    let c = canvas(r"\draw plot[only marks, mark=*] coordinates {(0,0) (1,1)} -- (2,0);");
    let line = paths(&c).into_iter().find(|d| d.fill.is_none()).unwrap();
    assert_eq!(points(&line.segments), [(1.0, 1.0), (2.0, 0.0)]);
}

/// The path of the basic layer's plot `handler` through `points` (in cm).
fn handled(handler: &str, setup: &str, pts: &[(f64, f64)]) -> Vec<Segment> {
    let stream: String = pts.iter().map(|(x, y)| format!(r"\pgfplotstreampoint{{\pgfpoint{{{x}cm}}{{{y}cm}}}}")).collect();
    let c = canvas(&format!(r"{setup}\pgfplothandler{handler}\pgfplotstreamstart{stream}\pgfplotstreamend\pgfusepath{{stroke}}"));
    paths(&c).into_iter().next().unwrap().segments
}

#[test]
fn plot_handlers_of_the_basic_layer() {
    let pts = [(1.0, 2.0), (2.0, 1.0), (3.0, 3.0)];
    // combs from the axes, or from the origin
    assert_eq!(points(&handled("xcomb", "", &pts))[..2], [(0.0, 2.0), (1.0, 2.0)]);
    assert_eq!(points(&handled("ycomb", "", &pts))[..2], [(1.0, 0.0), (1.0, 2.0)]);
    assert_eq!(points(&handled("polarcomb", "", &pts))[..2], [(0.0, 0.0), (1.0, 2.0)]);
    // bars from the axes, `bar width` wide
    let s = handled("ybar", r"\pgfsetlinewidth{0pt}\pgfkeys{/pgf/bar width=1cm}", &pts);
    assert_eq!(points(&s)[..4], [(0.5, 0.0), (1.5, 0.0), (1.5, 2.0), (0.5, 2.0)]);
    let s = handled("xbar", r"\pgfkeys{/pgf/bar width=1cm, /pgf/bar shift=0cm}", &pts);
    assert_eq!(points(&s)[..4], [(0.0, 1.5), (1.0, 1.5), (1.0, 2.5), (0.0, 2.5)]);
    // a bar for each interval, as high as its first point
    let s = handled("ybarinterval", "", &pts);
    assert_eq!(points(&s)[..4], [(1.0, 0.0), (2.0, 0.0), (2.0, 2.0), (1.0, 2.0)]);
    let s = handled("xbarinterval", r"\pgfkeys{/pgf/bar interval width=0.5, /pgf/bar interval shift=0.5}", &pts);
    assert!(near(points(&s)[0], (0.0, 1.75)), "{:?}", points(&s));
    // lines that stop short of each point, by 1.5pt
    let s = handled("gaplineto", "", &[(0.0, 0.0), (1.0, 0.0)]);
    assert!(near(points(&s)[0], (1.5 * PT, 0.0)) && near(points(&s)[1], (1.0 - 1.5 * PT, 0.0)), "{:?}", points(&s));
    let s = handled("gapcycle", "", &[(0.0, 0.0), (1.0, 0.0), (1.0, 0.0)]);
    assert_eq!(moves(&s), 3);
    // steps
    let two = [(0.0, 0.0), (2.0, 1.0)];
    assert_eq!(points(&handled("constantlineto", "", &two)), [(0.0, 0.0), (2.0, 0.0), (2.0, 1.0)]);
    assert_eq!(points(&handled("constantlinetomarkright", "", &two)), [(0.0, 0.0), (0.0, 1.0), (2.0, 1.0)]);
    assert_eq!(points(&handled("constantlinetomarkmid", "", &two)), [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (2.0, 1.0)]);
    assert_eq!(points(&handled("jumpmarkleft", "", &two)), [(0.0, 0.0), (2.0, 0.0), (2.0, 1.0)]);
    assert_eq!(moves(&handled("jumpmarkleft", "", &two)), 2);
    assert_eq!(points(&handled("jumpmarkright", "", &two)), [(0.0, 0.0), (0.0, 1.0), (2.0, 1.0)]);
    assert_eq!(points(&handled("jumpmarkmid", "", &two)), [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (2.0, 1.0)]);
    // closed: a polygon, a closed curve
    assert!(matches!(handled("polygon", "", &pts).last(), Some(Segment::Close)));
    let s = handled("closedcurve", "", &pts);
    assert_eq!(curves(&s), 3);
    assert!(matches!(s.last(), Some(Segment::Close)));
}

#[test]
fn dots_without_controls() {
    let w = warnings(r"\draw (0,0) .. node {a} (1,1);");
    assert!(w.iter().any(|w| w == "'..' without 'controls'"), "{w:?}");
}

#[test]
fn paths_cut_short_inside_an_operation() {
    for doc in [r"\draw (0,0) .. controls (1,1) and (2,1)", r"\draw (0,0) \pgfextra", r"\path (0,0) node\bgroup a"] {
        let (_, w) = tikra::engine::run_document(&format!(r"\begin{{tikzpicture}}{doc}")).unwrap();
        assert!(!w.is_empty(), "{doc}");
    }
    let (c, w) = tikra::engine::run_document(r"\begin{tikzpicture}\path (0,0) node\bgroup a").unwrap();
    assert_eq!(texts(&c).len(), 1, "{w:?}");
    let (_, w) = tikra::engine::run_document(r"\begin{tikzpicture}\draw (0,0) .. controls (1,1) and (2,1)").unwrap();
    assert!(w.iter().any(|w| w.contains("controls without a point")), "{w:?}");
}

#[test]
fn animations_after_other_options() {
    let c = canvas(r#"\begin{scope}[make snapshot of=1s] \path (0,0) node[fill=red] :fill opacity = { 0s = "0", 2s = "1" } {a}; \end{scope}"#);
    assert!((paths(&c)[0].fill.as_ref().unwrap().opacity - 0.5).abs() < 1e-9);
}

#[test]
fn svg_numbers_after_a_move_are_lines() {
    let t = first_path(r"\draw svg {M 0 0 10 0 10 10};");
    assert_eq!(t.segments.iter().filter(|s| matches!(s, Segment::LineTo(_))).count(), 2);
    assert!(near(*points(&t.segments).last().unwrap(), (10.0 * PT, 10.0 * PT)));
}

#[test]
fn svg_arcs_of_every_flag() {
    // a quarter circle from (0,0) to (10pt,10pt) about (10pt,0) or about (0,10pt), the short or the
    // long way, about one center or the other as the flags choose
    for (large, ccw, center) in [(0, 1, (10.0, 0.0)), (0, 0, (0.0, 10.0)), (1, 1, (10.0, 0.0)), (1, 0, (0.0, 10.0))] {
        let c = canvas(&format!(r"\pgfpathmoveto{{\pgfpointorigin}}\pgfpatharcto{{10pt}}{{10pt}}{{0}}{{{large}}}{{{ccw}}}{{\pgfpoint{{10pt}}{{10pt}}}}\pgfusepath{{stroke}}"));
        let s = &paths(&c)[0].segments;
        // every end is 10pt from the center
        for p in points(&s[1..]) {
            let d = ((p.0 / PT - center.0).powi(2) + (p.1 / PT - center.1).powi(2)).sqrt();
            assert!((d - 10.0).abs() < 1e-6, "{large} {ccw}: {p:?}");
        }
        assert_eq!(curves(s), if large == 1 { 3 } else { 1 }, "{large} {ccw}");
    }
}

#[test]
fn nothing_is_drawn_outside_a_picture() {
    let (c, w) = tikra::engine::run_document(r"\draw (0,0) -- (1,0);").unwrap();
    assert!(paths(&c).is_empty() && w.is_empty(), "{w:?}");
}

// ---------------------------------------------------------------- shapes and keys from outside

#[test]
fn declared_shapes_inherit_what_they_name() {
    // an anchor and a background path inherited, the border not declarable, a base shape tikra does not have
    let decl = r"\pgfdeclareshape{plain}{
      \inheritsavedanchors[from=circle]
      \inheritanchor{north}
      \inheritbackgroundpath[from=circle]
      \anchor{tip} {\pgfpoint{0pt}{20pt}}
      \anchorborder{\pgfpointorigin}
      \nodeparts{text}}
      \pgfdeclareshape{odd}{\inheritsavedanchors[from=no such shape]}";
    let (c, w) = canvas_and_warnings(&format!(r"{decl} \node[plain, draw] (p) {{x}}; \node[odd, draw] {{y}};"));
    assert!(w.iter().any(|w| w == r"\pgfdeclareshape{plain}: \anchorborder not supported: the inherited ones used"), "{w:?}");
    assert!(w.iter().any(|w| w == r"\pgfdeclareshape{plain}: \nodeparts not supported: the inherited ones used"), "{w:?}");
    assert!(w.iter().any(|w| w == r"\pgfdeclareshape{odd}: shape 'no such shape' to inherit from not supported: a rectangle"), "{w:?}");
    // the circle's background path, curves
    assert!(paths(&c).iter().any(|d| d.segments.iter().any(|s| matches!(s, Segment::CurveTo(..)))));
    // a space between an anchor's name and its code changes nothing
    let tip = point(&format!(r"{decl} \node[plain] (p) {{x}};"), "p.tip");
    let joined = point(&format!(r"{} \node[plain] (p) {{x}};", decl.replace("{tip} {", "{tip}{")), "p.tip");
    assert!(near((tip.x, tip.y), (joined.x, joined.y)), "{tip:?} {joined:?}");
}

use tikra::extension::{Context, Error as ExtensionError, Library, Tokens};

/// `shifted={options}`: the options in a group of their own, and the picture moved 1cm right for
/// what follows; a warning if the line is dashed, or if no options are given.
fn shifted(cx: &mut Context, value: Option<Tokens>) -> Result<(), ExtensionError> {
    let v = value.unwrap_or_default();
    if v.trim().is_empty() {
        cx.warn("shifted: no options");
    }
    cx.begin_group();
    cx.keys(v)?;
    let dashed = cx.is_dashed();
    cx.end_group();
    if dashed {
        cx.warn("shifted: dashed");
    }
    let t = cx.transform();
    cx.set_transform(t.compose(&tikra::scene::Transform::translation(1.0, 0.0)));
    Ok(())
}

/// `relabel={label options}`: those labels on the last node drawn, 2pt away.
fn relabel(cx: &mut Context, value: Option<Tokens>) -> Result<(), ExtensionError> {
    for l in cx.labels_of(value.unwrap_or_default())? {
        cx.draw_label_on_last_node(&l, Tokens::text("label distance=2pt"))?;
    }
    Ok(())
}

/// `zigzag at={options}`: decoration keys at their own path.
fn zigzag_at(cx: &mut Context, value: Option<Tokens>) -> Result<(), ExtensionError> {
    cx.keys_at("/pgf/decoration", value.unwrap_or_default())?;
    cx.keys(Tokens::text("decorate, decoration=zigzag"))
}

/// `opts={[options] rest}`: the options at its start, as keys.
fn opts(cx: &mut Context, value: Option<Tokens>) -> Result<(), ExtensionError> {
    let (o, rest) = value.unwrap_or_default().leading_options();
    if !rest.is_empty() {
        cx.warn(format!("opts: rest '{}'", rest.to_text()));
    }
    cx.keys(o)
}

fn no_definitions() -> String {
    String::new()
}

static TOOLS: Library =
    Library { name: "tools", definitions: no_definitions, keys: &[("/tikz/shifted", shifted), ("/tikz/relabel", relabel), ("/tikz/zigzag at", zigzag_at), ("/tikz/opts", opts)] };

fn with_tools(body: &str) -> (tikra::canvas::Canvas, Vec<String>) {
    let doc = format!("\\begin{{tikzpicture}}{body}\\end{{tikzpicture}}");
    let (c, w, _) = tikra::engine::run_document_with(&doc, &[&TOOLS]).unwrap();
    (c, w)
}

#[test]
fn a_librarys_keys_move_the_picture_and_read_its_state() {
    let (c, w) = with_tools(r"\draw[shifted={dashed}] (0,0) -- (1,0); \draw[shifted={ }] (0,0) -- (1,0);");
    assert!(w.iter().any(|w| w == "shifted: dashed") && w.iter().any(|w| w == "shifted: no options"), "{w:?}");
    // the group kept the dashes to the key; the first line is moved by 1cm, the second by 1cm of its own
    let p: Vec<_> = paths(&c).iter().map(|d| (points(&d.segments), d.stroke.as_ref().unwrap().dash.is_some())).collect();
    assert!(!p[0].1 && !p[1].1, "{p:?}");
    let (a, b) = (p[0].0[0], p[1].0[0]);
    assert!(close(b.0 - a.0, 0.0) && close(p[0].0[1].0 - a.0, 1.0), "{p:?}");
}

#[test]
fn a_librarys_key_labels_the_last_node() {
    let (c, w) = with_tools(r"\node (a) {A}; \path[relabel={label=above:L}];");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(texts(&c).len(), 2);
}

#[test]
fn a_librarys_key_sets_decoration_keys() {
    let (c, w) = with_tools(r"\draw[zigzag at={amplitude=2mm, segment length=5mm}] (0,0) -- (3,0);");
    assert!(w.is_empty(), "{w:?}");
    let (lo, hi) = bounds(&paths(&c)[0].segments);
    assert!((hi.y - lo.y - 0.4).abs() < 0.05, "{lo:?} {hi:?}");
}

#[test]
fn leading_options_without_their_bracket() {
    let (_, w) = with_tools(r"\draw[opts={[red] x}] (0,0) -- (1,0); \draw[opts={[red x}] (0,0) -- (1,0);");
    assert!(w.iter().any(|w| w == "opts: rest ' x'") && w.iter().any(|w| w == "opts: rest '[red x'"), "{w:?}");
}

#[test]
fn operations_written_wrong_are_read_past() {
    // TikZ stops with an error on these; tikra reads on and draws what it can
    // a word after the first control that is not `and`: one control
    let t = first_path(r"\draw (0,0) .. controls (1,1) ax .. (2,0);");
    assert!(close_to(*points(&t.segments).last().unwrap(), (2.0, 0.0)));
    // a word after `parabola` that is not `bend`: no bend
    let t = first_path(r"\draw (0,0) parabola box (2,4);");
    assert_eq!(curves(&t.segments), 1);
    // a letter among the variables of a foreach
    let t = first_path(r"\draw (0,0) foreach \x ix in {1,2} { -- (\x,0) };");
    assert_eq!(points(&t.segments), [(0.0, 0.0), (1.0, 0.0), (2.0, 0.0)]);
    // a foreach with nothing after it
    let (_, w) = tikra::engine::run_document(r"\begin{tikzpicture}\draw (0,0) foreach \i in {1}").unwrap();
    assert!(w.iter().any(|w| w == "path without ';' at the end"), "{w:?}");
    // an anchor without its code, a command a shape's declaration does not know
    let (c, w) = canvas_and_warnings(r"\pgfdeclareshape{bare}{\inheritsavedanchors[from=rectangle]\inheritbackgroundpath[from=rectangle]\relax\anchor{a}} \node[bare, draw] {x};");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(texts(&c).len(), 1);
}

#[test]
fn bar_intervals_of_the_default_width() {
    // an empty `bar interval width` is the default, 1
    let a = handled("ybarinterval", r"\pgfkeys{/pgf/bar interval width=, /pgf/bar interval shift=}", &[(1.0, 2.0), (2.0, 1.0)]);
    let b = handled("ybarinterval", "", &[(1.0, 2.0), (2.0, 1.0)]);
    assert_eq!(points(&a), points(&b));
}
