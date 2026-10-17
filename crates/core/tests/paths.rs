//! Path operations: lines, elbows, curves, relatives, shapes, arcs, grids, plots,
//! decorations and post-processing (corners, shortening, clipping).

mod common;
use common::*;
use tikra::scene::{Element, Point, Segment, Tip};

#[test]
fn line_elbows_and_closing() {
    let t = first_path(r"\draw (0,0) -- (2,0) -| (3,2) |- (0,3) -- cycle;");
    let p = points(&t.segments);
    assert!(close_to(p[2], (3.0, 0.0)), "the -| corner in {p:?}");
    assert!(close_to(p[4], (3.0, 3.0)), "the |- corner in {p:?}");
    assert!(matches!(t.segments.last(), Some(Segment::Close)));
    // cycle with no operation before it does not close
    let t = first_path(r"\draw (0,0) -- (1,0) cycle;");
    assert!(!t.segments.contains(&Segment::Close));
}

#[test]
fn relative_coordinates() {
    // ++ accumulates, + does not move the base
    let t = first_path(r"\draw (1,1) -- ++(1,0) -- ++(0,1) -- +(1,1) -- +(2,0);");
    let p = points(&t.segments);
    assert!(close_to(p[2], (2.0, 2.0)), "{p:?}");
    assert!(close_to(p[3], (3.0, 3.0)), "{p:?}");
    assert!(close_to(p[4], (4.0, 2.0)), "{p:?}");
}

#[test]
fn polar_coordinates_with_and_without_unit() {
    let t = first_path(r"\draw (0,0) -- (90:2) -- (0:1cm);");
    let p = points(&t.segments);
    assert!(close_to(p[1], (0.0, 2.0)), "{p:?}");
    assert!(close_to(p[2], (1.0, 0.0)), "{p:?}");
    // without a unit it follows the scope's unit vector; with a unit, it does not
    let c = scene(r"\begin{scope}[x=2cm] \draw (0,0) -- (0:1) -- (0:1cm); \end{scope}");
    let p = points(&paths(&c)[0].segments);
    assert!(close_to(p[1], (2.0, 0.0)) && close_to(p[2], (1.0, 0.0)), "{p:?}");
    // the angle may be a braced expression, as in a \foreach over the vertices of a polygon
    let t = first_path(r"\def\k{1} \draw ({90*\k}:2) -- ({45+45}:{1+1});");
    let p = points(&t.segments);
    assert!(close_to(p[0], (0.0, 2.0)) && close_to(p[1], (0.0, 2.0)), "{p:?}");
}

#[test]
fn explicit_length_in_coordinates() {
    let c = scene(r"\begin{scope}[x=2cm, y=3cm] \draw (1cm, 1cm) -- (1,1); \end{scope}");
    let p = points(&paths(&c)[0].segments);
    assert!(close_to(p[0], (1.0, 1.0)) && close_to(p[1], (2.0, 3.0)), "{p:?}");
}

#[test]
fn names_anchors_and_angles() {
    let c = scene(
        r"\node[draw, minimum size=2cm] (q) at (0,0) {};
          \coordinate (o) at (5,5);
          \draw (q.north) -- (q.30) -- (o);",
    );
    let p = points(&paths(&c)[0].segments);
    assert!(close_to(p[0], (0.0, 1.0)), "{p:?}");
    assert!(close(p[1].0, 1.0), "the square's border at 30°: {p:?}");
    assert!(close_to(p[2], (5.0, 5.0)));
}

#[test]
fn unknown_coordinate_and_unknown_anchor_become_warnings() {
    let (_, w) = scene_and_warnings(r"\draw (0,0) -- (ghost);");
    assert!(w.iter().any(|a| a.contains("unknown coordinate")), "{w:?}");
    let (_, w) = scene_and_warnings(r"\node (a) {x}; \draw (a.side) -- (1,1);");
    assert!(w.iter().any(|a| a.contains("unknown anchor")), "{w:?}");
}

#[test]
fn calc_midpoint_sum_factors_and_rotation() {
    let c = scene(
        r"\coordinate (A) at (0,0); \coordinate (B) at (4,0);
          \draw ($(A)!0.5!(B)$) -- ($(A)+(1,2)$) -- ($0.5*(B) - (1,0)$) -- ($(A)!1cm!(B)$) -- ($(A)!0.5!90:(B)$) -- ($(A)!0.5! (B)$);",
    );
    let p = points(&paths(&c)[0].segments);
    assert!(close_to(p[0], (2.0, 0.0)), "{p:?}");
    assert!(close_to(p[1], (1.0, 2.0)), "{p:?}");
    assert!(close_to(p[2], (1.0, 0.0)), "{p:?}");
    assert!(close_to(p[3], (1.0, 0.0)), "{p:?}");
    assert!(close_to(p[4], (0.0, 2.0)), "{p:?}");
}

#[test]
fn calc_errors_become_warnings() {
    for bad in [r"($(A)!0.5$)", r"($(A)!0.5!$)", r"($(A)!0.5!(B$)", r"($(A)!xx!(B)$)", r"($(A$)", r"($(Z)!0.5!(A)$)"] {
        let (_, w) = scene_and_warnings(&format!(r"\coordinate (A) at (0,0); \coordinate (B) at (1,0); \draw {bad} -- (1,1);"));
        assert!(!w.is_empty(), "{bad}");
    }
}

#[test]
fn bezier_curves_with_one_and_two_controls() {
    let t = first_path(r"\draw (0,0) .. controls (1,1) and (2,1) .. (3,0) .. controls (4,-1) .. (5,0);");
    assert_eq!(t.segments.iter().filter(|s| matches!(s, Segment::CurveTo(..))).count(), 2);
    // .. without controls: ignored without hanging
    first_path(r"\draw (0,0) -- (1,0) .. (2,2);");
    // a curve that starts the path
    let t = first_path(r"\draw .. controls (1,1) and (2,1) .. (3,0);");
    assert!(matches!(t.segments[0], Segment::MoveTo(_)));
}

#[test]
fn curve_errors() {
    for bad in [r"(0,0) .. controls", r"(0,0) .. controls (1,1) and", r"(0,0) .. controls (1,1) and (2,2) .."] {
        let (_, w) = scene_and_warnings(&format!(r"\draw {bad};"));
        assert!(!w.is_empty(), "{bad}");
    }
}

#[test]
fn to_with_angles_bends_and_looseness() {
    for op in ["to[out=90, in=180]", "to[bend left]", "to[bend right=45]", "to[bend left=20, looseness=2]", "edge[bend right]"] {
        let t = first_path(&format!(r"\draw (0,0) {op} (2,0);"));
        assert!(t.segments.iter().any(|s| matches!(s, Segment::CurveTo(..))), "{op}");
    }
    // to without options is a straight line
    let t = first_path(r"\draw (0,0) to (2,0);");
    assert!(matches!(t.segments[1], Segment::LineTo(_)));
    // to a node stops at its border (curve)
    let c = scene(r"\node[draw, circle, minimum size=1cm] (b) at (3,0) {}; \draw (0,0) to[bend left] (b);");
    assert!(points(&paths(&c)[0].segments)[1].0 < 3.0);
}

#[test]
fn rectangle_circle_ellipse() {
    let t = first_path(r"\draw (0,0) rectangle (2,1);");
    assert_eq!(points(&t.segments).len(), 5, "the start point and the four corners");
    for shape in ["circle (1)", "circle [radius=1]", "circle [x radius=2, y radius=1]", "ellipse (2 and 1)", "circle[radius=5mm] ", "ellipse [x radius=1cm, y radius=5mm]"] {
        let t = first_path(&format!(r"\draw (1,1) {shape};"));
        assert_eq!(t.segments.iter().filter(|s| matches!(s, Segment::CurveTo(..))).count(), 4, "{shape}");
    }
}

#[test]
fn shape_errors() {
    for bad in [r"(0,0) rectangle", r"(0,0) circle (1", r"(0,0) circle [radius=zz]", r"(0,0) circle [x radius=zz]", r"(0,0) circle [y radius=zz]", r"(0,0) ellipse (zz and 1)", r"(0,0) ellipse (1 and zz)", r"(0,0) circle (zz)"] {
        let (_, w) = scene_and_warnings(&format!(r"\draw {bad};"));
        assert!(!w.is_empty(), "{bad}");
    }
}

#[test]
fn arcs_by_parentheses_and_by_options() {
    let t = first_path(r"\draw (1,0) arc (0:90:1);");
    let p = points(&t.segments);
    assert!(close_to(*p.last().unwrap(), (0.0, 1.0)), "{p:?}");
    let t = first_path(r"\draw (2,0) arc (0:180:2 and 1);");
    assert!(close_to(*points(&t.segments).last().unwrap(), (-2.0, 0.0)));
    let t = first_path(r"\draw (1,0) arc [start angle=0, end angle=270, radius=1];");
    assert_eq!(t.segments.iter().filter(|s| matches!(s, Segment::CurveTo(..))).count(), 3);
    let t = first_path(r"\draw (1,0) arc [start angle=0, delta angle=-90, x radius=1, y radius=2];");
    assert!(close_to(*points(&t.segments).last().unwrap(), (0.0, -2.0)));
    // an arc that starts the path, an arc with no parameters and with incomplete parameters
    first_path(r"\draw arc (0:90:1);");
    first_path(r"\draw (0,0) arc;");
    first_path(r"\draw (0,0) arc (0:90);");
    // unknown options on the arc
    first_path(r"\draw (1,0) arc [radius=1, nothing=2];");
}

#[test]
fn arc_errors() {
    for bad in [r"(0,0) arc (0:90:1", r"(0,0) arc (zz:90:1)", r"(0,0) arc (0:zz:1)", r"(0,0) arc (0:90:zz)", r"(0,0) arc (0:90:zz and 1)", r"(0,0) arc (0:90:1 and zz)", r"(0,0) arc [start angle=zz]", r"(0,0) arc [end angle=zz]", r"(0,0) arc [delta angle=zz]", r"(0,0) arc [radius=zz]", r"(0,0) arc [x radius=zz]", r"(0,0) arc [y radius=zz]"] {
        let (_, w) = scene_and_warnings(&format!(r"\draw {bad};"));
        assert!(!w.is_empty(), "{bad}");
    }
}

#[test]
fn grid_with_steps() {
    let t = first_path(r"\draw (0,0) grid (2,2);");
    assert_eq!(t.segments.len(), 13, "the start point, 3 vertical and 3 horizontal lines");
    let t = first_path(r"\draw[step=0.5] (0,0) grid (1,1);");
    assert_eq!(t.segments.len(), 13, "the path's step applies to the grid");
    let t = first_path(r"\draw[xstep=2, ystep=zz] (0,0) grid (2,2);");
    assert_eq!(t.segments.len(), 1 + 2 * (2 + 3));
    let t = first_path(r"\draw (0,0) grid [step=0.5] (1,1);");
    assert_eq!(t.segments.len(), 13);
    let t = first_path(r"\draw (0,0) grid [xstep=1, ystep=0.5, other=1] (2,1);");
    assert_eq!(t.segments.len(), 13, "the start point, 3 vertical and 3 horizontal lines");
    for bad in [r"(0,0) grid", r"(0,0) grid [step=zz] (1,1)", r"(0,0) grid [xstep=zz] (1,1)", r"(0,0) grid [ystep=zz] (1,1)"] {
        let (_, w) = scene_and_warnings(&format!(r"\draw {bad};"));
        assert!(!w.is_empty(), "{bad}");
    }
}

#[test]
fn plot_of_expression_coordinates_and_smooth() {
    let t = first_path(r"\draw[domain=0:1, samples=11] plot (\x, {\x*\x});");
    assert_eq!(points(&t.segments).len(), 11);
    let t = first_path(r"\draw plot[smooth] coordinates {(0,0) (1,1) (2,0) (3,1)};");
    assert_eq!(t.segments.iter().filter(|s| matches!(s, Segment::CurveTo(..))).count(), 3);
    let t = first_path(r"\draw plot[domain=0:2, samples=3, variable=\t] (\t, 1);");
    assert_eq!(points(&t.segments).len(), 3);
    // a plot that continues a path
    let t = first_path(r"\draw (0,5) -- plot coordinates {(1,1) (2,2)};");
    assert!(matches!(t.segments[1], Segment::LineTo(_)));
    // the variable does not leak and a macro with the same name comes back afterwards
    let c = scene(r"\def\x{7} \draw[domain=0:1, samples=2, variable=\x] plot (\x, 0); \draw (\x, 0) -- (8,0);");
    assert!(close(points(&paths(&c)[1].segments)[0].0, 7.0));
    // a saved numeric variable value comes back
    let c = scene(r"\foreach \x in {3} { \draw[domain=0:1, samples=2] plot (\x, 0); \draw (\x,0) -- (9,9); }");
    assert!(close(points(&paths(&c)[1].segments)[0].0, 3.0));
    // non-finite points are skipped
    let t = first_path(r"\draw[domain=-1:1, samples=3] plot (\x, {1/\x});");
    assert_eq!(points(&t.segments).len(), 2);
}

#[test]
fn plot_errors() {
    let (_, w) = scene_and_warnings(r"\draw plot (\x, {\doesnotexist});");
    assert!(w.iter().any(|a| a.contains("plot:")), "{w:?}");
    for bad in [r"plot coordinates", r"plot", r"plot[domain=zz:1] (\x,0)", r"plot[domain=0:zz] (\x,0)", r"plot[samples=zz] (\x,0)", r"plot coordinates {(zz,0)}"] {
        let (_, w) = scene_and_warnings(&format!(r"\draw {bad};"));
        assert!(!w.is_empty(), "{bad}");
    }
    // a domain without ':' and unknown options: ignored
    first_path(r"\draw[domain=1] plot[domain=2, mark=*] (\x, 0);");
}

#[test]
fn decorations() {
    for d in ["zigzag", "snake", "random steps", "coil"] {
        let t = first_path(&format!(r"\draw[decorate, decoration={{{d}, amplitude=1mm, segment length=2mm}}] (0,0) -- (2,0) arc (0:90:1) -- cycle;"));
        assert!(points(&t.segments).len() > 5, "{d}");
    }
    // decorate without decoration uses zigzag; decoration without decorate does not decorate
    assert!(points(&first_path(r"\draw[decorate] (0,0) -- (2,0);").segments).len() > 2);
    assert_eq!(points(&first_path(r"\draw[decoration=zigzag] (0,0) -- (2,0);").segments).len(), 2);
    // a zero-length segment stays as it is
    first_path(r"\draw[decorate] (0,0) -- (0,0) -- (1,0);");
    // bad values keep the defaults
    first_path(r"\draw[decorate, decoration={zigzag, amplitude=zz, segment length=zz, x=1}] (0,0) -- (1,0);");
}

#[test]
fn rounded_corners() {
    let t = first_path(r"\draw[rounded corners] (0,0) -- (2,0) -- (2,2);");
    assert!(t.segments.iter().any(|s| matches!(s, Segment::CurveTo(..))));
    let t = first_path(r"\draw[rounded corners=2mm] (0,0) -- (2,0) -- (2,2) -- cycle;");
    assert!(matches!(t.segments.last(), Some(Segment::Close)));
    // closed, repeating the first point
    first_path(r"\draw[rounded corners] (0,0) -- (2,0) -- (2,2) -- (0,0) -- cycle;");
    // few points: no corner to round
    let t = first_path(r"\draw[rounded corners] (0,0) -- (2,0);");
    assert_eq!(t.segments.len(), 2);
    let t = first_path(r"\draw[rounded corners] (0,0) -- (2,0) -- cycle;");
    assert!(matches!(t.segments.last(), Some(Segment::Close)));
    // with a curve on the path, nothing changes
    let t = first_path(r"\draw[rounded corners] (0,0) -- (1,0) arc (0:90:1);");
    assert!(t.segments.iter().any(|s| matches!(s, Segment::CurveTo(..))));
    // sharp corners turns it off
    let t = first_path(r"\draw[rounded corners, sharp corners] (0,0) -- (2,0) -- (2,2);");
    assert!(!t.segments.iter().any(|s| matches!(s, Segment::CurveTo(..))));
    // a closed rectangle with corners
    first_path(r"\draw[rounded corners=zz] (0,0) rectangle (2,1);");
}

#[test]
fn shorten_ends() {
    let t = first_path(r"\draw[shorten >=1cm, shorten <=5mm] (0,0) -- (3,0);");
    let p = points(&t.segments);
    assert!(close_to(p[0], (0.5, 0.0)) && close_to(p[1], (2.0, 0.0)), "{p:?}");
    // a closed end and a curve at the end: not shortened
    first_path(r"\draw[shorten >=1mm] (0,0) -- (1,0) -- cycle;");
    first_path(r"\draw[shorten >=1mm, shorten <=1mm] (0,0) arc (0:90:1);");
    // shortening by more than the segment: not shortened
    let t = first_path(r"\draw[shorten >=5cm, shorten <=5cm] (0,0) -- (1,0);");
    assert!(close_to(points(&t.segments)[1], (1.0, 0.0)));
    // the last segment after a curve
    first_path(r"\draw[shorten >=1mm] (0,0) arc (0:90:1) -- (0,3);");
    // a one-segment path: nothing to shorten
    let c = scene(r"\draw[shorten >=1mm] (0,0) circle (1);");
    assert_eq!(paths(&c).len(), 1);
}

#[test]
fn clip_and_end_clip_in_the_scope() {
    let c = scene(r"\begin{scope} \clip (0,0) rectangle (1,1); \draw (0,0) -- (2,2); \end{scope}");
    assert!(matches!(c.elements[0], Element::Clip(_)));
    assert!(matches!(c.elements.last(), Some(Element::EndClip)));
}

#[test]
fn options_and_braces_mid_path_and_junk() {
    first_path(r"\draw (0,0) -- (1,0) [red] -- (2,0) {text} -- (3,0);");
    let (_, w) = scene_and_warnings(r"\draw (0,0) -- (1,0) ??? -- (2,0);");
    assert!(w.iter().any(|a| a.contains("unrecognized")), "{w:?}");
    // an unmatched bracket in the statement's options
    let (_, w) = scene_and_warnings(r"\draw[red (0,0) -- (1,0);");
    assert!(!w.is_empty());
    // an unmatched parenthesis
    let (_, w) = scene_and_warnings(r"\draw (0,0 -- (1,0);");
    assert!(!w.is_empty());
    // options in several brackets
    first_path(r"\draw[red][thick] (0,0) -- (1,0);");
}

#[test]
fn path_without_drawing_does_not_become_a_path() {
    let c = scene(r"\path (0,0) -- (1,1); \draw (0,0); \fill[draw=none, fill=none] (0,0) -- (1,0);");
    assert!(paths(&c).is_empty());
}

#[test]
fn arrows_without_stroke_vanish() {
    let t = first_path(r"\fill[->] (0,0) -- (1,0) -- (1,1);");
    assert_eq!((t.style.start_tip, t.style.end_tip), (None, None));
    let t = first_path(r"\draw[<->] (0,0) -- (1,0);");
    assert_eq!((t.style.start_tip, t.style.end_tip), (Some(Tip::Default), Some(Tip::Default)));
}

#[test]
fn nodes_on_a_path_inherit_its_color() {
    let c = scene(r"\tikzset{f/.style={color=blue}} \draw[f] (0,0) -- (1,0) node[right] {a}; \draw[red] (0,0) -- (1,0) node {b}; \draw (0,0) -- (1,1) coordinate (end);");
    assert_eq!(nodes(&c).len(), 2);
}

#[test]
fn error_inside_a_path_node_becomes_a_warning() {
    let (_, w) = scene_and_warnings(r"\draw (0,0) -- (1,0) node[below=of nobody] {x};");
    assert!(w.iter().any(|a| a.contains("nobody")), "{w:?}");
}

#[test]
fn statement_through_the_api() {
    let mut env = tikra::reader::Env::new();
    tikra::path::statement(&mut env, "draw", "(0,0) -- (1,0)").unwrap();
    assert_eq!(env.scene.elements.len(), 1);
    assert!(tikra::path::absolute_coordinate(&env, "zz").is_err());
}

#[test]
fn builder_edge_cases() {
    // a line with no start point: becomes the start point
    let t = first_path(r"\draw -- (1,0) -- (2,0);");
    assert!(matches!(t.segments[0], Segment::MoveTo(_)));
    // a line after closing: restarts from the start point
    let t = first_path(r"\draw (0,0) -- (1,0) -- (1,1) -- cycle -- (2,2);");
    assert!(matches!(t.segments[4], Segment::MoveTo(_)), "{:?}", t.segments);
    // space before the semicolon
    first_path("\\draw (0,0) -- (1,0)   \n ;");
    // an unknown option on to and on circle
    first_path(r"\draw (0,0) to[out=0, in=180, red] (2,0);");
    first_path(r"\draw (0,0) circle [radius=1, red];");
    // a node with two option blocks
    let c = scene(r"\draw (0,0) -- (1,0) node[red][above] {x};");
    assert_eq!(nodes(&c).len(), 1);
    // shortening a one-point path
    let c = scene(r"\draw[shorten >=1mm] (0,0);");
    assert!(paths(&c).is_empty());
    // arrows with braces on both sides
    let t = first_path(r"\draw[{Latex[length=2mm]}-{Stealth[width=1mm]}] (0,0) -- (1,0);");
    assert_eq!((t.style.start_tip, t.style.end_tip), (Some(Tip::Latex), Some(Tip::Stealth)));
}

#[test]
fn coordinate_errors_in_each_operation() {
    for bad in [
        r"(zz:1) -- (1,1)",
        r"(0:zz) -- (1,1)",
        r"(0,0) .. controls (zz) .. (1,1)",
        r"(0,0) .. controls (1,1) and (zz) .. (2,2)",
        r"(0,0) .. controls (1,1) and (2,2) .. (zz)",
        r"(0,0) rectangle (zz)",
        r"(0,0) grid (zz)",
        r"(0,0) -- ($(zz)$)",
        r"(0,0) -- ($(0,0)!0.5!(zz)$)",
        r"(0,0) node at (zz) {x}",
        r"(0,0) -- (zz) -- (1,1)",
        r"(0,0) -- +(zz)",
        r"(0,0) plot coordinates {(zz,1)}",
    ] {
        let (_, w) = scene_and_warnings(&format!(r"\draw {bad};"));
        assert!(!w.is_empty(), "{bad}");
    }
}

#[test]
fn node_between_operation_and_target_goes_midway() {
    let c = scene(r"\draw (0,0) -- node {m} (4,0); \draw (0,0) -| node {c} (2,2); \draw (0,0) to[bend left] node[pos=0.25] {q} (4,0); \draw (1,1) -- node {z};");
    let n = nodes(&c);
    assert!(close_to((n[0].center.x, n[0].center.y), (2.0, 0.0)), "midway on a line: {:?}", n[0].center);
    assert!(close_to((n[1].center.x, n[1].center.y), (2.0, 0.0)), "at the corner of -|: {:?}", n[1].center);
    // pos is the Bézier parameter on the bent curve, as in TikZ: near x = 1, above the chord
    assert!((n[2].center.x - 1.0).abs() < 0.05 && n[2].center.y > 0.3, "an explicit pos wins, on the curve: {:?}", n[2].center);
    assert!(close_to((n[3].center.x, n[3].center.y), (1.0, 1.0)), "no target: stays where the path is: {:?}", n[3].center);
}

#[test]
fn bend_given_to_the_whole_statement() {
    let t = first_path(r"\draw[bend left=30] (0,0) to (4,0);");
    assert!(t.segments.iter().any(|s| matches!(s, tikra::scene::Segment::CurveTo(..))), "{:?}", t.segments);
}

#[test]
fn node_on_a_bent_path_sits_on_the_curve() {
    // the chord runs along y = 0; bend right pulls the curve below it, and the node follows the curve
    let c = scene(r"\coordinate (a) at (0,0); \coordinate (b) at (4,0); \draw (a) to[bend right] node {m} (b);");
    let m = nodes(&c)[0];
    assert!(close(m.center.x, 2.0), "{:?}", m.center);
    assert!(m.center.y < -0.3, "on the curve, not on the chord: {:?}", m.center);
}

#[test]
fn curve_leaves_from_the_starting_node_border() {
    let c = scene(r"\node[draw, circle, minimum size=1cm] (t) at (3,0) {}; \node[draw] (s) at (0,0) {}; \draw (t) to[bend left] (s);");
    let t = paths(&c)[0];
    let Segment::MoveTo(p) = t.segments[0] else { panic!() };
    let r = p.minus(Point::new(3.0, 0.0)).norm();
    assert!(close(r, 0.5), "starts on the circle (radius 0.5), not at its center: {r}");
}
