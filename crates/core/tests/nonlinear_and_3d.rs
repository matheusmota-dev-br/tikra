//! Nonlinear transformations (`\pgftransformnonlinear`, their linear approximations), the
//! curvilinear library's points along a curve, the 3d and perspective libraries' corners, and view
//! boxes (`meet`, `slice`, `pgfviewboxscope`).

mod common;
use common::*;
use tikra::canvas::{Canvas, Item};
use tikra::scene::{Point, Segment, Transform};

/// The canvas and warnings of a picture body in which `@` is a letter, in the picture's own
/// coordinates (cm), as `canvas_and_warnings` gives them.
fn at_letter(body: &str) -> (Canvas, Vec<String>) {
    let frame = 500.0;
    let document = format!("\\makeatletter\\begin{{tikzpicture}}\\useasboundingbox (-{frame},-{frame}) rectangle ({frame},{frame}); {body}\\end{{tikzpicture}}");
    let (mut c, w) = tikra::engine::run_document(&document).unwrap_or_else(|e| panic!("{e}\n{body}"));
    let back = Point::new(-frame, -frame);
    for i in &mut c.items {
        i.translate(back);
    }
    (c, w)
}

/// The segments of the first path a body (with `@` a letter) draws.
fn first_segments(body: &str) -> Vec<Segment> {
    let (c, w) = at_letter(body);
    assert!(w.is_empty(), "{w:?}\n{body}");
    paths(&c).into_iter().next().unwrap_or_else(|| panic!("no path in {body}")).segments
}

/// The point a body (with `@` a letter) leaves in `\pgf@x`, `\pgf@y`, in cm: a line is drawn from it.
fn pgf_point(body: &str) -> (f64, f64) {
    points(&first_segments(&format!(r"{body} \pgfpathmoveto{{\pgfqpoint{{\pgf@x}}{{\pgf@y}}}}\pgfpathlineto{{\pgfpointorigin}}\pgfusepath{{stroke}}")))[0]
}

fn near(a: (f64, f64), b: (f64, f64)) -> bool {
    (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3
}

// --- nonlinear transformations ---

/// A shift of 1cm in x, as a nonlinear transformation.
const SHIFT: &str = r"\pgftransformnonlinear{\advance\pgf@x by 1cm\relax}";

/// A line becomes curves (halved until their controls are close), each point moved by the code.
#[test]
fn a_line_becomes_moved_curves() {
    let s = first_segments(&format!(r"{SHIFT} \draw (0,0) -- (2,0);"));
    assert!(matches!(s[0], Segment::MoveTo(_)) && s[1..].iter().all(|s| matches!(s, Segment::CurveTo(..))), "{s:?}");
    assert!(s.len() > 3, "the line is cut in pieces: {s:?}");
    let p = points(&s);
    assert!(near(p[0], (1.0, 0.0)) && near(*p.last().unwrap(), (3.0, 0.0)), "{p:?}");
}

/// A line shorter than a tenth of a point stays a line; a curve is halved and moved too.
#[test]
fn short_lines_and_curves() {
    let s = first_segments(&format!(r"{SHIFT} \draw (0,0) -- (0.001,0);"));
    assert!(matches!(s[1], Segment::LineTo(_)), "{s:?}");
    assert!(near(points(&s)[1], (1.001, 0.0)));
    let s = first_segments(&format!(r"{SHIFT} \draw (0,0) .. controls (1,1) and (2,1) .. (3,0);"));
    assert!(s.len() > 2 && s[1..].iter().all(|s| matches!(s, Segment::CurveTo(..))), "{s:?}");
    assert!(near(*points(&s).last().unwrap(), (4.0, 0.0)));
}

/// Closing a path draws the line back to its start (moved too) unless it is already there.
#[test]
fn closing_a_path() {
    let s = first_segments(&format!(r"{SHIFT} \draw (0,0) -- (1,0) -- (1,1) -- cycle;"));
    assert!(matches!(s.last(), Some(Segment::Close)));
    let p = points(&s);
    assert!(near(*p.last().unwrap(), (1.0, 0.0)), "the way back ends at the moved start: {p:?}");
    let s = first_segments(&format!(r"{SHIFT} \draw (0,0) -- (0.001,0) -- (0,0) -- cycle;"));
    assert_eq!(s.iter().filter(|s| matches!(s, Segment::LineTo(_))).count(), 2, "nothing more to close: {s:?}");
}

/// `\pgfapproximatenonlineartransformation` turns the nonlinear transformation into the linear one
/// it makes at the origin; `…translation` keeps only its shift. Without one, both do nothing.
#[test]
fn linear_approximations() {
    let double = r"\pgftransformnonlinear{\pgf@x=2\pgf@x \advance\pgf@x by 1cm\relax}";
    let s = first_segments(&format!(r"{double} \pgfapproximatenonlineartransformation \draw (0,0) -- (1,1);"));
    assert!(matches!(s[1], Segment::LineTo(_)), "a straight line again: {s:?}");
    assert!(near(points(&s)[1], (3.0, 1.0)), "{s:?}");
    let s = first_segments(&format!(r"{double} \pgfapproximatenonlineartranslation \draw (0,0) -- (1,1);"));
    assert!(near(points(&s)[0], (1.0, 0.0)) && near(points(&s)[1], (2.0, 1.0)), "{s:?}");
    let s = first_segments(r"\pgfapproximatenonlineartransformation \draw (0,0) -- (1,1);");
    assert!(near(points(&s)[1], (1.0, 1.0)), "{s:?}");
}

/// `\pgf@sys@tonumber` gives a register's value in pt without the unit; a count is 0.
#[test]
fn registers_as_numbers() {
    let s = first_segments(r"\pgf@x=2cm \draw (\pgf@sys@tonumber\pgf@x pt,0) -- (\pgf@sys@tonumber\c@pgf@counta,1);");
    let p = points(&s);
    assert!(near(p[0], (2.0, 0.0)) && near(p[1], (0.0, 1.0)), "{p:?}");
    // at the very end, with nothing to read
    no_warnings(r"\makeatletter\begin{tikzpicture}\draw (0,0) -- (1,0);\end{tikzpicture}\pgf@sys@tonumber");
}

/// `\pgfmathsincos@{angle}`: its cosine and sine.
#[test]
fn sine_and_cosine_at_once() {
    let s = first_segments(r"\pgfmathsincos@{30} \draw (0,0) -- (\pgfmathresultx,\pgfmathresulty);");
    assert!(near(points(&s)[1], (3f64.sqrt() / 2.0, 0.5)), "{s:?}");
}

// --- curvilinear coordinates ---

/// A straight curve along x, 3cm long.
const ALONG_X: &str = r"\pgfsetcurvilinearbeziercurve{\pgfpointorigin}{\pgfpoint{1cm}{0cm}}{\pgfpoint{2cm}{0cm}}{\pgfpoint{3cm}{0cm}}";

/// A point a distance along the curve and an offset to its left.
#[test]
fn orthogonal_points_on_a_curve() {
    assert!(near(pgf_point(&format!(r"{ALONG_X}\pgfpointcurvilinearbezierorthogonal{{1cm}}{{0.5cm}}")), (1.0, 0.5)));
    assert!(near(pgf_point(&format!(r"{ALONG_X}\pgfpointcurvilinearbezierorthogonal{{2cm}}{{-1cm}}")), (2.0, -1.0)));
    // a curve that starts without a direction: the direction toward the second control
    let still = r"\pgfsetcurvilinearbeziercurve{\pgfpointorigin}{\pgfpointorigin}{\pgfpoint{1cm}{0cm}}{\pgfpoint{2cm}{0cm}}";
    assert!(near(pgf_point(&format!(r"{still}\pgfpointcurvilinearbezierorthogonal{{0cm}}{{1cm}}")), (0.0, 1.0)));
}

/// A polar point: as far along the curve as it is from the origin, turned by its angle.
#[test]
fn polar_points_on_a_curve() {
    for (x, y, expected) in [("1cm", "0cm", (1.0, 0.0)), ("0cm", "2cm", (0.0, 2.0)), ("0cm", "0cm", (0.0, 0.0)), ("1cm", "1cm", (1.0, 1.0))] {
        let p = pgf_point(&format!(r"{ALONG_X}\pgfpointcurvilinearbezierpolar{{{x}}}{{{y}}}"));
        assert!(near(p, expected), "({x},{y}): {p:?}");
    }
    // x < 0: the distance counts backward, the angle that of (−x,−y): before the start, mirrored
    let p = pgf_point(&format!(r"{ALONG_X}\pgfpointcurvilinearbezierpolar{{-1cm}}{{0cm}}"));
    assert!(near(p, (-1.0, 0.0)), "{p:?}");
}

/// Without a curve set, the points leave `\pgf@x`, `\pgf@y` as they were.
#[test]
fn no_curve_no_point() {
    for cmd in [r"\pgfpointcurvilinearbezierorthogonal{1cm}{1cm}", r"\pgfpointcurvilinearbezierpolar{1cm}{1cm}"] {
        let p = pgf_point(&format!(r"\pgf@x=1cm \pgf@y=2cm {cmd}"));
        assert!(near(p, (1.0, 2.0)), "{cmd}: {p:?}");
    }
}

// --- 3d and perspective ---

/// `canvas is xy plane at z` without a value is the plane at 0.
#[test]
fn a_plane_without_its_height() {
    let c = canvas(r"\begin{scope}[z={(1,1)}, canvas is xy plane at z] \draw (0,0) -- (1,1); \end{scope}");
    assert!(close_to(points(&paths(&c)[0].segments)[1], (1.0, 1.0)));
    let c = canvas(r"\begin{scope}[z={(1,1)}, canvas is xy plane at z=2] \draw (0,0) -- (1,1); \end{scope}");
    assert!(close_to(points(&paths(&c)[0].segments)[0], (2.0, 2.0)));
}

/// Each of the six canvas planes takes the picture's x and y to two of the three axes.
#[test]
fn the_six_planes() {
    for (plane, expected) in [
        ("xy plane at z=0", (1.0, 2.0)),
        ("yx plane at z=0", (2.0, 1.0)),
        ("xz plane at y=0", (3.0, 2.0)),
        ("zx plane at y=0", (3.0, 1.0)),
        ("yz plane at x=0", (2.0, 3.0)),
        ("zy plane at x=0", (1.0, 3.0)),
    ] {
        let c = canvas(&format!(r"\begin{{scope}}[z={{(1,1)}}, canvas is {plane}] \draw (0,0) -- (1,2); \end{{scope}}"));
        let p = points(&paths(&c)[0].segments)[1];
        assert!(near(p, expected), "{plane}: {p:?}");
    }
}

/// The isometric view: x and y at 45° down to each side of the vertical z.
#[test]
fn isometric_view() {
    let h = 0.5f64.sqrt();
    let (s, c) = (1.0 / 3f64.sqrt(), (2.0 / 3.0f64).sqrt());
    for (point, expected) in [("1,0,0", (h, h * s)), ("0,1,0", (-h, h * s)), ("0,0,1", (0.0, c))] {
        let p = point_of_view(point);
        assert!(near(p, expected), "{point}: {p:?} {expected:?}");
    }
}

fn point_of_view(xyz: &str) -> (f64, f64) {
    let c = canvas(&format!(r"\begin{{scope}}[isometric view] \draw (0,0,0) -- ({xyz}); \end{{scope}}"));
    points(&paths(&c)[0].segments)[1]
}

/// `canvas is plane` through three points given as coordinates.
#[test]
fn a_plane_through_three_points() {
    let c = canvas(r"\begin{scope}[plane origin={(1,1)}, plane x={(3,1)}, plane y={(1,2)}, canvas is plane] \draw (0,0) -- (1,1); \end{scope}");
    let p = points(&paths(&c)[0].segments);
    assert!(close_to(p[0], (1.0, 1.0)) && close_to(p[1], (3.0, 2.0)), "{p:?}");
}

/// `xyz spherical` takes `angle` for the longitude when there is no `longitude`.
#[test]
fn spherical_angle_as_longitude() {
    let a = point("", "xyz spherical cs: angle=90, latitude=0, radius=2");
    let b = point("", "xyz spherical cs: longitude=90, latitude=0, radius=2");
    assert!(close(a.x, b.x) && close(a.y, b.y) && close(a.x, 2.0), "{a:?} {b:?}");
}

/// An unknown coordinate system is an error that names it.
#[test]
fn unknown_coordinate_system() {
    let e = tikra::convert(r"\begin{tikzpicture}\draw (nowhere cs: x=1) -- (1,0);\end{tikzpicture}");
    let text = match e {
        Ok(r) => r.warnings.join(" "),
        Err(e) => e.to_string(),
    };
    assert!(text.contains("nowhere"), "{text}");
}

/// A vanishing point needs three coordinates; one at 0 on its own axis gives no vanishing; the
/// key without a value sets pgf's default vanishing points.
#[test]
fn vanishing_points() {
    let r = tikra::convert(r"\begin{tikzpicture}[perspective={p={(1,2)}}]\draw (0,0) -- (1,0);\end{tikzpicture}");
    let text = match r {
        Ok(r) => r.warnings.join(" "),
        Err(e) => e.to_string(),
    };
    assert!(text.contains("three coordinates expected"), "{text}");
    // p at 0 along x: x has no vanishing point, a point stays where the unit vectors put it
    let p = point(r"\tikzset{perspective={p={(0,1,0)}}}", "tpp cs: x=2, y=0, z=0");
    assert!(close_to((p.x, p.y), (2.0, 0.0)), "{p:?}");
    // the default: x vanishes at (10,0,0), so x=5 is drawn at 10·5/(10+5)
    let p = point(r"\tikzset{perspective}", "tpp cs: x=5, y=0, z=0");
    assert!((p.x - 10.0 / 3.0).abs() < 1e-6, "{p:?}");
}

// --- view boxes ---

/// The canvas transformations of a body.
fn canvas_transforms(c: &Canvas) -> Vec<Transform> {
    c.items
        .iter()
        .filter_map(|i| match i {
            Item::Transform { transform } => Some(*transform),
            _ => None,
        })
        .collect()
}

/// `pgfviewboxscope`: the 1cm square onto a 2cm × 4cm box, fitted in (`meet`) or filling it (`slice`).
#[test]
fn view_box_scopes() {
    for (kind, s, e, f) in [("meet", 2.0, 0.0, 1.0), ("slice", 4.0, -1.0, 0.0)] {
        let c = canvas(&format!(
            r"\begin{{pgfviewboxscope}}{{\pgfpointorigin}}{{\pgfpoint{{1cm}}{{1cm}}}}{{\pgfpointorigin}}{{\pgfpoint{{2cm}}{{4cm}}}}{{{kind}}}
                \draw (0,0) -- (1,1);
              \end{{pgfviewboxscope}}"
        ));
        let t = canvas_transforms(&c);
        assert_eq!(t.len(), 1, "{kind}");
        let t = t[0];
        assert!(close(t.a, s) && close(t.d, s) && close(t.e, e) && close(t.f, f), "{kind}: {t:?}");
    }
}

/// A `meet` or `slice` that is not `(a) (b) at (c) (d)` is ignored with a warning.
#[test]
fn malformed_view_keys() {
    for (key, warning) in [("meet={(0,0) (1,1)}", "is not '(a) (b) at (c) (d)'"), ("slice={(0,0) (1,1)}", "slice:"), ("meet={(0,0) at (2,2) (3,3)}", "four points expected")] {
        let (c, w) = canvas_and_warnings(&format!(r"\begin{{scope}}[{key}] \draw (0,0) -- (1,1); \end{{scope}}"));
        assert!(w.iter().any(|w| w.contains(warning)), "{key}: {w:?}");
        assert!(canvas_transforms(&c).is_empty(), "{key}");
    }
}
