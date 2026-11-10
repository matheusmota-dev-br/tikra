//! pgf's basic layer used directly: its points (`\pgfpoint…`), path commands (`\pgfpath…`), how a
//! path is used (`\pgfusepath`), the graphic state (`\pgfset…`), transformations (`\pgftransform…`),
//! texts and nodes, plot streams and handlers, and the small utilities around them.

mod common;
use common::*;
use tikra::canvas::{Cap, Item, Join, Paint};
use tikra::scene::{Point, Segment};
use tikra::PT;

/// The canvas and warnings of a picture body in which `@` is a letter, in the picture's own
/// coordinates (cm).
fn at_letter(body: &str) -> (tikra::canvas::Canvas, Vec<String>) {
    let document = format!("\\makeatletter\\begin{{tikzpicture}}\\useasboundingbox (-500,-500) rectangle (500,500); {body}\\end{{tikzpicture}}");
    let (mut c, w) = tikra::engine::run_document(&document).unwrap_or_else(|e| panic!("{e}\n{body}"));
    for i in &mut c.items {
        i.translate(Point::new(-500.0, -500.0));
    }
    (c, w)
}

/// The paths of a body, which must draw without warnings.
fn drawn(body: &str) -> Vec<Drawn> {
    let (c, w) = at_letter(body);
    assert!(w.is_empty(), "{w:?}\n{body}");
    paths(&c)
}

/// The end points of the first path of a body.
fn first_points(body: &str) -> Vec<(f64, f64)> {
    points(&drawn(body).first().unwrap_or_else(|| panic!("nothing drawn: {body}")).segments)
}

/// Where a point lands: a line is drawn from it to the origin, after `before`.
fn landing(before: &str, point: &str) -> (f64, f64) {
    first_points(&format!(r"{before}\pgfpathmoveto{{{point}}}\pgfpathlineto{{\pgfpointorigin}}\pgfusepath{{stroke}}"))[0]
}

/// The warnings of a body.
fn warned(body: &str) -> Vec<String> {
    at_letter(body).1
}

fn near(a: (f64, f64), b: (f64, f64)) -> bool {
    (a.0 - b.0).abs() < 1e-4 && (a.1 - b.1).abs() < 1e-4
}

/// `\pgfpoint{x cm}{y cm}`.
fn p(x: f64, y: f64) -> String {
    format!(r"\pgfpoint{{{x}cm}}{{{y}cm}}")
}

// --- points ---

/// Polar points, with one radius or two (an ellipse's).
#[test]
fn polar_points() {
    for (point, expected) in [
        (r"\pgfpointpolar{90}{2cm}", (0.0, 2.0)),
        (r"\pgfqpointpolar{180}{1cm}", (-1.0, 0.0)),
        (r"\pgfpointpolar{90}{1cm and 2cm}", (0.0, 2.0)),
        (r"\pgfpointpolar{0}{1cm and 2cm}", (1.0, 0.0)),
        (r"\pgfpointpolarxy{90}{2}", (0.0, 2.0)),
    ] {
        assert!(near(landing("", point), expected), "{point}: {:?}", landing("", point));
    }
}

/// Points in the xy and xyz coordinate systems follow the unit vectors.
#[test]
fn points_in_the_unit_vectors() {
    let z = r"\pgfsetzvec{\pgfpoint{1cm}{1cm}}";
    for (before, point, expected) in [
        ("", r"\pgfpointxy{1}{2}", (1.0, 2.0)),
        (r"\pgfsetxvec{\pgfpoint{2cm}{0cm}}", r"\pgfqpointxy{1}{1}", (2.0, 1.0)),
        (r"\pgfsetyvec{\pgfpoint{0cm}{3cm}}", r"\pgfpointxy{1}{1}", (1.0, 3.0)),
        (z, r"\pgfpointxyz{1}{2}{3}", (4.0, 5.0)),
        (z, r"\pgfqpointxyz{0}{0}{1}", (1.0, 1.0)),
        // cylindrical: angle, radius, height; spherical: longitude, latitude, radius
        (z, r"\pgfpointcylindrical{90}{2}{1}", (1.0, 3.0)),
        (z, r"\pgfpointspherical{90}{0}{2}", (2.0, 0.0)),
        (z, r"\pgfpointspherical{0}{0}{2}", (0.0, 2.0)),
        (z, r"\pgfpointspherical{0}{90}{2}", (2.0, 2.0)),
    ] {
        let at = landing(before, point);
        assert!(near(at, expected), "{point}: {at:?}");
    }
}

/// Points made from other points.
#[test]
fn computed_points() {
    for (point, expected) in [
        (format!(r"\pgfpointadd{{{}}}{{{}}}", p(1.0, 0.0), p(0.0, 2.0)), (1.0, 2.0)),
        (format!(r"\pgfpointdiff{{{}}}{{{}}}", p(1.0, 0.0), p(3.0, 2.0)), (2.0, 2.0)),
        (format!(r"\pgfpointscale{{2}}{{{}}}", p(1.0, 1.5)), (2.0, 3.0)),
        (format!(r"\pgfqpointscale{{0.5}}{{{}}}", p(2.0, 2.0)), (1.0, 1.0)),
        (format!(r"\pgfpointlineattime{{0.25}}{{{}}}{{{}}}", p(0.0, 0.0), p(4.0, 0.0)), (1.0, 0.0)),
        (format!(r"\pgfpointlineatdistance{{1cm}}{{{}}}{{{}}}", p(0.0, 1.0), p(0.0, 5.0)), (0.0, 2.0)),
        (format!(r"\pgfpointcurveattime{{0.5}}{{{}}}{{{}}}{{{}}}{{{}}}", p(0.0, 0.0), p(0.0, 2.0), p(2.0, 2.0), p(2.0, 0.0)), (1.0, 1.5)),
        (format!(r"\pgfpointnormalised{{{}}}", p(3.0, 4.0)), (0.6 * PT, 0.8 * PT)),
        (format!(r"\pgfpointintersectionoflines{{{}}}{{{}}}{{{}}}{{{}}}", p(0.0, 0.0), p(2.0, 2.0), p(0.0, 2.0), p(2.0, 0.0)), (1.0, 1.0)),
        (format!(r"\pgfpointarcaxesattime{{0.5}}{{{}}}{{{}}}{{{}}}{{0}}{{180}}", p(1.0, 1.0), p(2.0, 0.0), p(0.0, 1.0)), (1.0, 2.0)),
        // leading spaces are skipped
        (format!(r"  {}", p(1.0, 1.0)), (1.0, 1.0)),
    ] {
        let at = landing("", &point);
        assert!(near(at, expected), "{point}: {at:?}");
    }
}

/// A transformed point has the transformation applied once more when it goes into the path.
#[test]
fn transformed_points() {
    let at = landing(r"\pgftransformshift{\pgfpoint{1cm}{0cm}}", &format!(r"\pgfpointtransformed{{{}}}", p(1.0, 1.0)));
    assert!(near(at, (3.0, 1.0)), "{at:?}");
}

/// A node's anchors and border points, in the coordinate system in force.
#[test]
fn anchors_and_borders() {
    let node = r"\node[minimum size=2cm, inner sep=0pt, outer sep=0pt] (n) at (1,1) {};";
    assert!(near(landing(node, r"\pgfpointanchor{n}{north}"), (1.0, 2.0)));
    assert!(near(landing(node, r"\pgfpointshapeborder{n}{\pgfpoint{5cm}{1cm}}"), (2.0, 1.0)));
    // an empty name names no node (even one given `name=`): the origin
    assert!(near(landing(r"\node[name=] at (1,1) {};", r"\pgfpointanchor{}{center}"), (0.0, 0.0)));
    // in a shifted system, the anchor is where it is on the canvas
    assert!(near(landing(&format!(r"{node}\pgftransformshift{{\pgfpoint{{1cm}}{{0cm}}}}"), r"\pgfpointanchor{n}{east}"), (2.0, 1.0)));
}

/// What is not a point, or names nothing, is an error, reported and the command ignored.
#[test]
fn points_that_fail() {
    for (point, message) in [
        ("hello", "not a point"),
        ("", "a point is missing"),
        (r"\pgfpointanchor{nobody}{north}", "unknown node: nobody"),
        (r"\pgfpointanchor{n}{nowhere}", "unknown anchor: n.nowhere"),
        (&format!(r"\pgfpointintersectionoflines{{{}}}{{{}}}{{{}}}{{{}}}", p(0.0, 0.0), p(1.0, 0.0), p(0.0, 1.0), p(1.0, 1.0)), "intersection of parallel lines"),
        (r"\pgfpointintersectionsolution{1}", "no intersection 1"),
        (r"\pgfpointintersectionsolution{0}", "no intersection 0"),
        (r"\pgfplotspointaxisxy{1}{1}", "outside an axis"),
    ] {
        let w = warned(&format!(r"\node (n) {{}}; \pgfpathmoveto{{{point}}}\pgfusepath{{stroke}}"));
        assert!(w.iter().any(|w| w.contains(message)), "{point}: {w:?}");
    }
}

/// Where two paths cross: `\pgfintersectionofpaths` counts the solutions and gives each one.
#[test]
fn intersections_of_paths() {
    let cross = format!(
        r"\pgfintersectionofpaths{{\pgfpathmoveto{{{}}}\pgfpathlineto{{{}}}}}{{\pgfpathmoveto{{{}}}\pgfpathlineto{{{}}}}}",
        p(0.0, 0.0),
        p(2.0, 2.0),
        p(0.0, 2.0),
        p(2.0, 0.0)
    );
    assert!(near(landing(&cross, r"\pgfpointintersectionsolution{1}"), (1.0, 1.0)));
    // the number of solutions, as a point's x
    assert!(near(landing(&cross, r"\pgfpointxy{\pgfintersectionsolutions}{0}"), (1.0, 0.0)));
    // the solution in a shifted system
    assert!(near(landing(&format!(r"{cross}\pgftransformshift{{\pgfpoint{{1cm}}{{0cm}}}}"), r"\pgfpointintersectionsolution{1}"), (1.0, 1.0)));
}

/// pgfplots' points: in the axis's coordinates, where `axis cs` puts them, and directions in it.
#[test]
fn points_of_an_axis() {
    let axis = |body: &str| {
        let doc = format!(r"\begin{{tikzpicture}}\begin{{axis}}[xmin=0,xmax=10,ymin=0,ymax=10]\pgfplotsextra{{{body}}}\end{{axis}}\end{{tikzpicture}}");
        let (c, w) = tikra::engine::run_document(&doc).unwrap();
        assert!(w.is_empty(), "{w:?}");
        paths(&c).last().map(|d| points(&d.segments)).expect("a path")
    };
    let reference = axis(r"\draw (axis cs:2,3) -- (axis cs:7,1);");
    for body in [
        r"\pgfpathmoveto{\pgfplotspointaxisxy{2}{3}}\pgfpathlineto{\pgfplotspointaxisxy{7}{1}}\pgfusepath{stroke}",
        r"\pgfpathmoveto{\pgfplotspointaxisxyz{2}{3}{0}}\pgfpathlineto{\pgfplotspointaxisxyz{7}{1}{0}}\pgfusepath{stroke}",
        r"\pgfpathmoveto{\pgfplotspointaxisxy{2}{3}}\pgfpathlineto{\pgfpointadd{\pgfplotspointaxisxy{2}{3}}{\pgfplotspointaxisdirectionxy{5}{-2}}}\pgfusepath{stroke}",
        r"\pgfpathmoveto{\pgfplotspointaxisxy{2}{3}}\pgfpathlineto{\pgfpointadd{\pgfplotspointaxisxy{2}{3}}{\pgfplotspointaxisdirectionxyz{5}{-2}{0}}}\pgfusepath{stroke}",
    ] {
        let got = axis(body);
        assert!(got.len() == 2 && near(got[0], reference[0]) && near(got[1], reference[1]), "{body}: {got:?} {reference:?}");
    }
}

// --- path construction ---

/// A quadratic curve is the cubic with its controls two thirds toward the quadratic's.
#[test]
fn quadratic_curves() {
    let d = drawn(&format!(r"\pgfpathmoveto{{\pgfpointorigin}}\pgfpathquadraticcurveto{{{}}}{{{}}}\pgfusepath{{stroke}}", p(1.0, 1.0), p(2.0, 0.0)));
    match d[0].segments[1] {
        Segment::CurveTo(a, b, c) => {
            assert!(near((a.x, a.y), (2.0 / 3.0, 2.0 / 3.0)) && near((b.x, b.y), (4.0 / 3.0, 2.0 / 3.0)) && near((c.x, c.y), (2.0, 0.0)), "{:?}", d[0].segments);
        }
        s => panic!("{s:?}"),
    }
}

/// Arcs from the current point: of a circle, of an ellipse, and of an ellipse given by its axes.
#[test]
fn arcs() {
    for (arc, end) in [
        (r"\pgfpatharc{0}{90}{1cm}", (0.0, 1.0)),
        (r"\pgfpatharc{0}{90}{1cm and 2cm}", (0.0, 2.0)),
        (r"\pgfpatharc{0}{180}{1cm}", (-1.0, 0.0)),
        (r"\pgfpatharcaxes{0}{90}{\pgfpoint{1cm}{0cm}}{\pgfpoint{0cm}{3cm}}", (0.0, 3.0)),
        (r"\pgfpatharcaxes{0}{90}{\pgfpoint{1cm}{1cm}}{\pgfpoint{-1cm}{1cm}}", (-1.0, 0.0)),
    ] {
        // the arc goes on from where it ends: a line from there
        let pts = first_points(&format!(r"\pgfpathmoveto{{\pgfpoint{{1cm}}{{0cm}}}}{arc}\pgfpathlineto{{\pgfpoint{{5cm}}{{5cm}}}}\pgfusepath{{stroke}}"));
        let before_line = pts[pts.len() - 2];
        assert!(near(before_line, end), "{arc}: {pts:?}");
    }
}

/// An ellipse from its center and two axes.
#[test]
fn ellipses() {
    let d = drawn(&format!(r"\pgfpathellipse{{{}}}{{{}}}{{{}}}\pgfusepath{{stroke}}", p(1.0, 1.0), p(2.0, 0.0), p(0.0, 1.0)));
    let (lo, hi) = bounds(&d[0].segments);
    assert!(near((lo.x, lo.y), (-1.0, 0.0)) && near((hi.x, hi.y), (3.0, 2.0)), "{lo:?} {hi:?}");
    assert!(matches!(d[0].segments.last(), Some(Segment::Close)));
}

/// A grid: lines at the multiples of its steps inside the rectangle.
#[test]
fn grids() {
    let lines = |options: &str| {
        let d = drawn(&format!(r"\pgfpathgrid{options}{{{}}}{{{}}}\pgfusepath{{stroke}}", p(2.0, 1.0), p(0.0, 0.0)));
        let s = &d[0].segments;
        let moves: Vec<_> = s.iter().filter_map(|s| if let Segment::MoveTo(p) = s { Some((p.x, p.y)) } else { None }).collect();
        moves
    };
    // 1cm steps: two horizontal lines and three vertical ones
    assert_eq!(lines("").len(), 5);
    assert_eq!(lines(r"[step={\pgfpoint{1cm}{1cm}}]").len(), 5);
    assert_eq!(lines(r"[step={\pgfpoint{-1cm}{0.5cm}}]").len(), 6);
    assert_eq!(lines("[stepx=0.5cm]").len(), 2 + 5);
    assert_eq!(lines("[stepy=0.25cm]").len(), 5 + 3);
    // a key it does not know, or without a value, changes nothing
    assert_eq!(lines("[color=red, stepx]").len(), 5);
    // a step of zero: no lines that way
    assert_eq!(lines("[stepx=0cm]").len(), 2);
}

/// A parabola through its bend to its end, and the sine and cosine quarter waves.
#[test]
fn parabolas_and_waves() {
    let pts = first_points(&format!(r"\pgfpathmoveto{{\pgfpointorigin}}\pgfpathparabola{{{}}}{{{}}}\pgfusepath{{stroke}}", p(1.0, 1.0), p(1.0, -1.0)));
    assert_eq!(pts.len(), 3);
    assert!(near(pts[1], (1.0, 1.0)) && near(pts[2], (2.0, 0.0)), "{pts:?}");
    // no bend: one curve from the start; no end: one curve to the bend
    for (bend, end, last) in [(p(0.0, 0.0), p(2.0, 1.0), (2.0, 1.0)), (p(2.0, 1.0), p(0.0, 0.0), (2.0, 1.0))] {
        let pts = first_points(&format!(r"\pgfpathmoveto{{\pgfpointorigin}}\pgfpathparabola{{{bend}}}{{{end}}}\pgfusepath{{stroke}}"));
        assert!(pts.len() == 2 && near(pts[1], last), "{pts:?}");
    }
    for (wave, c1, c2) in [(r"\pgfpathsine", (0.326, 0.512), (0.638, 1.0)), (r"\pgfpathcosine", (0.362, 0.0), (0.674, 0.488))] {
        let d = drawn(&format!(r"\pgfpathmoveto{{\pgfpointorigin}}{wave}{{{}}}\pgfusepath{{stroke}}", p(1.0, 1.0)));
        match d[0].segments[1] {
            Segment::CurveTo(a, b, c) => assert!(near((a.x, a.y), c1) && near((b.x, b.y), c2) && near((c.x, c.y), (1.0, 1.0)), "{wave}: {:?}", d[0].segments),
            s => panic!("{s:?}"),
        }
    }
}

/// An SVG arc to a point: the ellipse of the radii through both ends, on the side the flags say.
#[test]
fn arcs_to_a_point() {
    for (ccw, y) in [(0, 1.0), (1, -1.0)] {
        let pts = first_points(&format!(
            r"\pgfpathmoveto{{\pgfpointorigin}}\pgfpatharcto{{1cm}}{{1cm}}{{0}}{{0}}{{{ccw}}}{{{}}}\pgfpathlineto{{\pgfpointorigin}}\pgfusepath{{stroke}}",
            p(2.0, 0.0)
        ));
        assert!(pts.iter().any(|q| near(*q, (1.0, y))), "{ccw}: {pts:?}");
        assert!(near(pts[pts.len() - 2], (2.0, 0.0)), "{pts:?}");
    }
    // precomputed: the arc of the ellipse around a center, to the end point given
    let pts = first_points(&format!(
        r"\pgfpathmoveto{{\pgfpoint{{1cm}}{{0cm}}}}\pgfpatharctoprecomputed{{{}}}{{0}}{{90}}{{{}}}{{1cm}}{{2cm}}{{0.5}}{{2}}\pgfpathlineto{{\pgfpointorigin}}\pgfusepath{{stroke}}",
        p(0.0, 0.0),
        p(0.0, 2.0)
    ));
    assert!(pts.iter().any(|q| near(*q, (0.0, 2.0))), "{pts:?}");
}

/// pgf's SVG path syntax, in pt.
#[test]
fn svg_paths() {
    let pts = first_points(r"\pgfpathsvg{M 0 0 L 10 0 L 10 10}\pgfusepath{stroke}");
    assert!(near(pts[1], (10.0 * PT, 0.0)) && near(pts[2], (10.0 * PT, 10.0 * PT)), "{pts:?}");
}

/// The quick commands take canvas lengths; a path made only of them does not make the picture larger.
#[test]
fn quick_paths() {
    let d = drawn(r"\pgfpathqmoveto{0pt}{0pt}\pgfpathqlineto{10pt}{0pt}\pgfpathqcurveto{10pt}{5pt}{5pt}{10pt}{0pt}{10pt}\pgfusepath{stroke}");
    let pts = points(&d[0].segments);
    assert!(near(pts[1], (10.0 * PT, 0.0)) && near(pts[2], (0.0, 10.0 * PT)), "{pts:?}");
    let d = drawn(r"\pgfpathqcircle{10pt}\pgfusepath{fill}");
    let (lo, hi) = bounds(&d[0].segments);
    assert!(near((lo.x, hi.y), (-10.0 * PT, 10.0 * PT)), "{lo:?} {hi:?}");
    // the size of a picture with a line and a far quick path is the line's
    let quick = r"\draw[line width=0pt] (0,0) -- (1,1); \pgfpathqmoveto{0pt}{0pt}\pgfpathqlineto{300pt}{0pt}\pgfusepath{stroke}";
    let (w, h) = size(quick);
    assert!(close(w, 1.0) && close(h, 1.0), "{w} {h}");
    // a quick command after an ordinary one: the path counts
    let mixed = r"\draw[line width=0pt] (0,0) -- (1,1); \pgfpathmoveto{\pgfpointorigin}\pgfpathqlineto{2cm}{0pt}\pgfusepath{stroke}";
    let (w, _) = size(mixed);
    assert!((w - 2.0 - 0.4 * PT).abs() < 1e-3, "{w}");
}

/// `\pgfgetpath` keeps the path being built and `\pgfsetpath` brings it back.
#[test]
fn saved_paths() {
    let body = format!(
        r"\pgfpathmoveto{{{}}}\pgfpathlineto{{{}}}\pgfgetpath\kept\pgfusepath{{discard}}\pgfsetpath\kept\pgfusepath{{stroke}}\pgfsetpath\nothing\pgfusepath{{stroke}}",
        p(1.0, 0.0),
        p(2.0, 3.0)
    );
    let d = drawn(&body);
    assert_eq!(d.len(), 1);
    assert_eq!(points(&d[0].segments), vec![(1.0, 0.0), (2.0, 3.0)]);
}

/// `\pgfinterruptpath`: a path of its own inside the one being built, which goes on after it.
#[test]
fn interrupted_paths() {
    let d = drawn(&format!(
        r"\pgfpathmoveto{{\pgfpointorigin}}\pgfpathlineto{{{}}}\pgfinterruptpath\pgfpathmoveto{{{}}}\pgfpathlineto{{{}}}\pgfusepath{{stroke}}\endpgfinterruptpath\pgfpathlineto{{{}}}\pgfusepath{{stroke}}",
        p(1.0, 0.0),
        p(5.0, 5.0),
        p(6.0, 5.0),
        p(1.0, 1.0)
    ));
    assert_eq!(d.len(), 2);
    assert_eq!(points(&d[0].segments), vec![(5.0, 5.0), (6.0, 5.0)]);
    assert_eq!(points(&d[1].segments), vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)]);
}

// --- using a path ---

/// The quick uses: stroke, fill, both, clip.
#[test]
fn quick_uses() {
    let square = format!(r"\pgfpathrectangle{{\pgfpointorigin}}{{{}}}", p(1.0, 1.0));
    for (usage, fill, stroke) in [(r"\pgfusepathqstroke", false, true), (r"\pgfusepathqfill", true, false), (r"\pgfusepathqfillstroke", true, true)] {
        let d = drawn(&format!("{square}{usage}"));
        assert_eq!((d[0].fill.is_some(), d[0].stroke.is_some()), (fill, stroke), "{usage}");
    }
    let (c, _) = at_letter(&format!(r"{square}\pgfusepathqclip \draw (0,0) -- (2,2);"));
    assert!(c.items.iter().any(|i| matches!(i, Item::Clip { .. })), "{:?}", c.items);
}

/// An action `\pgfusepath` does not know is ignored, with a warning.
#[test]
fn unknown_path_use() {
    let w = warned(r"\pgfpathmoveto{\pgfpointorigin}\pgfpathlineto{\pgfpoint{1cm}{0cm}}\pgfusepath{stroke, sparkle}");
    assert!(w.iter().any(|w| w.contains(r"\pgfusepath{sparkle} not supported")), "{w:?}");
}

// --- graphic state ---

fn line_with(state: &str) -> Drawn {
    drawn(&format!(r"{state}\pgfpathmoveto{{\pgfpointorigin}}\pgfpathlineto{{{}}}\pgfpathlineto{{{}}}\pgfusepath{{fill,stroke}}", p(2.0, 0.0), p(2.0, 2.0))).remove(0)
}

/// Dash patterns, line caps and joins, the miter limit.
#[test]
fn dashes_caps_and_joins() {
    let s = line_with(r"\pgfsetdash{{3pt}{2pt}}{1pt}").stroke.unwrap();
    let (on_off, phase) = s.dash.unwrap();
    assert!(close(on_off[0], 3.0 * PT) && close(on_off[1], 2.0 * PT) && close(phase, PT), "{on_off:?} {phase}");
    assert!(line_with(r"\pgfsetdash{{3pt}{2pt}}{0pt}\pgfsetdash{}{0pt}").stroke.unwrap().dash.is_none());
    for (state, cap) in [(r"\pgfsetroundcap", Cap::Round), (r"\pgfsetrectcap", Cap::Square), (r"\pgfsetroundcap\pgfsetbuttcap", Cap::Butt)] {
        assert_eq!(line_with(state).stroke.unwrap().cap, cap, "{state}");
    }
    for (state, join) in [(r"\pgfsetroundjoin", Join::Round), (r"\pgfsetbeveljoin", Join::Bevel), (r"\pgfsetbeveljoin\pgfsetmiterjoin", Join::Miter)] {
        assert_eq!(line_with(state).stroke.unwrap().join, join, "{state}");
    }
    assert!(close(line_with(r"\pgfsetmiterlimit{4}").stroke.unwrap().miter_limit, 4.0));
}

/// Stroke and fill colors and opacities, and the fill rule.
#[test]
fn colors_opacities_and_rules() {
    let d = line_with(r"\pgfsetstrokecolor{red}\pgfsetfillcolor{blue}\pgfsetstrokeopacity{0.5}\pgfsetfillopacity{0.25}");
    let (f, s) = (d.fill.unwrap(), d.stroke.unwrap());
    assert!(s.color.r > 0.99 && s.color.b < 0.01 && close(s.opacity, 0.5), "{s:?}");
    assert!(matches!(f.paint, Paint::Solid(c) if c.b > 0.99 && c.r < 0.01) && close(f.opacity, 0.25), "{f:?}");
    // \pgfsetcolor sets both, after any of them
    let d = line_with(r"\pgfsetstrokecolor{red}\pgfsetfillcolor{blue}\pgfsetcolor{green}");
    assert!(d.stroke.unwrap().color.g > 0.4 && matches!(d.fill.unwrap().paint, Paint::Solid(c) if c.g > 0.4 && c.b < 0.01));
    assert!(line_with(r"\pgfseteorule").fill.unwrap().even_odd);
    assert!(!line_with(r"\pgfseteorule\pgfsetnonzerorule").fill.unwrap().even_odd);
    // a color it does not know: a warning, the color unchanged
    for cmd in [r"\pgfsetstrokecolor", r"\pgfsetfillcolor", r"\pgfsetcolor", r"\pgfsetinnerstrokecolor"] {
        let w = warned(&format!(r"{cmd}{{nocolor}}"));
        assert!(w.iter().any(|w| w.contains("unknown color 'nocolor'")), "{cmd}: {w:?}");
    }
}

/// Arrow tips at the ends of a basic-layer path, and the path shortened.
#[test]
fn arrows_and_shortening() {
    let plain = line_with("");
    let count = |state: &str| drawn(&format!(r"{state}\pgfpathmoveto{{\pgfpointorigin}}\pgfpathlineto{{{}}}\pgfusepath{{stroke}}", p(2.0, 0.0))).len();
    assert_eq!(count(""), 1);
    assert_eq!(count(r"\pgfsetarrows{->}"), 2);
    assert_eq!(count(r"\pgfsetarrows{<->}"), 3);
    assert_eq!(count(r"\pgfsetarrowsstart{stealth}"), 2);
    assert_eq!(count(r"\pgfsetarrowsend{latex}"), 2);
    assert_eq!(count(r"\pgfsetarrows{->}\pgfsetarrowsend{}"), 1);
    assert_eq!(count(r"\pgfsetarrows{<-}\pgfsetarrowsstart{}"), 1);
    assert!(plain.stroke.is_some());
    let pts =
        first_points(&format!(r"\pgfsetshortenstart{{0.5cm}}\pgfsetshortenend{{0.25cm}}\pgfpathmoveto{{\pgfpointorigin}}\pgfpathlineto{{{}}}\pgfusepath{{stroke}}", p(2.0, 0.0)));
    assert!(near(pts[0], (0.5, 0.0)) && near(pts[1], (1.75, 0.0)), "{pts:?}");
}

/// Arced corners: a polygon's corners become curves.
#[test]
fn arced_corners() {
    let d = drawn(&format!(r"\pgfsetcornersarced{{\pgfpoint{{5pt}}{{5pt}}}}\pgfpathrectangle{{\pgfpointorigin}}{{{}}}\pgfusepath{{stroke}}", p(2.0, 1.0)));
    assert!(d[0].segments.iter().any(|s| matches!(s, Segment::CurveTo(..))), "{:?}", d[0].segments);
}

/// Inner lines of double lines: their width and color.
#[test]
fn inner_lines() {
    let d = drawn(&format!(
        r"\pgfsetlinewidth{{3pt}}\pgfsetinnerlinewidth{{1pt}}\pgfsetinnerstrokecolor{{red}}\pgfpathmoveto{{\pgfpointorigin}}\pgfpathlineto{{{}}}\pgfusepath{{stroke}}",
        p(2.0, 0.0)
    ));
    assert_eq!(d.len(), 2, "{d:?}");
    let inner = d[1].stroke.clone().unwrap();
    assert!(close(inner.width, PT) && inner.color.r > 0.99 && inner.color.g < 0.01, "{inner:?}");
}

// --- transformations ---

/// Each transformation, seen on where the point (1cm,1cm) lands.
#[test]
fn transformations() {
    let shift = r"\pgftransformshift{\pgfpoint{1cm}{0cm}}";
    for (t, expected) in [
        (r"\pgftransformxshift{1cm}".to_string(), (2.0, 1.0)),
        (r"\pgftransformyshift{1cm}".to_string(), (1.0, 2.0)),
        (r"\pgftransformscale{2}".to_string(), (2.0, 2.0)),
        (r"\pgftransformxscale{2}".to_string(), (2.0, 1.0)),
        (r"\pgftransformyscale{3}".to_string(), (1.0, 3.0)),
        (r"\pgftransformxslant{1}".to_string(), (2.0, 1.0)),
        (r"\pgftransformyslant{1}".to_string(), (1.0, 2.0)),
        (r"\pgftransformrotate{90}".to_string(), (-1.0, 1.0)),
        (r"\pgftransformcm{1}{0}{0}{2}{\pgfpoint{1cm}{0cm}}".to_string(), (2.0, 2.0)),
        (format!(r"{shift}\pgftransformreset"), (1.0, 1.0)),
        (format!(r"{shift}\pgftransformscale{{2}}\pgftransformresetnontranslations"), (2.0, 1.0)),
        (format!(r"{shift}\pgftransforminvert"), (0.0, 1.0)),
        // the origin to the first point, (1pt,0pt) to the second, (0pt,1pt) to the third
        (r"\pgftransformtriangle{\pgfpoint{1cm}{0cm}}{\pgfpoint{1cm+2pt}{0cm}}{\pgfpoint{1cm}{1pt}}".to_string(), (3.0, 1.0)),
        // at the end, x pointing away from the start
        (r"\pgftransformarrow{\pgfpointorigin}{\pgfpoint{0cm}{2cm}}".to_string(), (-1.0, 3.0)),
        (r"\pgftransformarrow{\pgfpointorigin}{\pgfpointorigin}".to_string(), (1.0, 1.0)),
    ] {
        let at = landing(&t, &p(1.0, 1.0));
        assert!(near(at, expected), "{t}: {at:?}");
    }
}

/// The transformations to a point of a line, curve or arc: shifted there; turned along it when
/// sloped (upright unless allowed upside down); without the transformation before when reset.
#[test]
fn transformations_at_a_time() {
    let line = |flags: &str, from: &str, to: &str| landing(&format!(r"{flags}\pgftransformlineattime{{0.5}}{{{from}}}{{{to}}}"), &p(1.0, 0.0));
    let (a, b) = (p(0.0, 0.0), p(2.0, 2.0));
    let h = 0.5f64.sqrt();
    assert!(near(line("", &a, &b), (2.0, 1.0)));
    assert!(near(line(r"\pgfslopedattimetrue", &a, &b), (1.0 + h, 1.0 + h)));
    // backwards: kept upright, or turned around when allowed
    assert!(near(line(r"\pgfslopedattimetrue", &b, &a), (1.0 + h, 1.0 + h)));
    assert!(near(line(r"\pgfslopedattimetrue\pgfallowupsidedownattimetrue", &b, &a), (1.0 - h, 1.0 - h)));
    // a line of no length has no direction: x stays x
    assert!(near(line(r"\pgfslopedattimetrue", &a, &a), (1.0, 0.0)));
    // the scaling before is dropped when reset
    assert!(near(line(r"\pgftransformscale{2}", &a, &b), (4.0, 2.0)));
    assert!(near(line(r"\pgftransformscale{2}\pgfresetnontranslationattimetrue", &a, &b), (3.0, 2.0)));
    // a curve at half time, along its tangent there
    let curve = format!(r"\pgfslopedattimetrue\pgftransformcurveattime{{0.5}}{{{}}}{{{}}}{{{}}}{{{}}}", p(0.0, 0.0), p(0.0, 2.0), p(2.0, 2.0), p(2.0, 0.0));
    assert!(near(landing(&curve, &p(1.0, 0.0)), (2.0, 1.5)));
    // an arc: counterclockwise its tangent at the top points left, clockwise right
    for (from, to) in [(0, 180), (180, 0)] {
        let arc = format!(
            r"\pgfslopedattimetrue\pgfallowupsidedownattimetrue\pgftransformarcaxesattime{{0.5}}{{{}}}{{{}}}{{{}}}{{{from}}}{{{to}}}",
            p(0.0, 0.0),
            p(1.0, 0.0),
            p(0.0, 1.0)
        );
        let x = if from < to { -1.0 } else { 1.0 };
        assert!(near(landing(&arc, &p(1.0, 0.0)), (x, 1.0)), "{from} {to}: {:?}", landing(&arc, &p(1.0, 0.0)));
    }
}

/// The adjustments: how much a length must shrink to come out at its size under the transformation.
#[test]
fn transformation_adjustments() {
    let at = landing(
        r"\pgftransformscale{2}\pgftransformyscale{4}\pgftransformationadjustments",
        r"\pgfpoint{\pgfhorizontaltransformationadjustment cm}{\pgfverticaltransformationadjustment cm}",
    );
    assert!(near(at, (1.0, 1.0)), "{at:?}");
}

/// The low-level transformations go to the canvas: line widths and texts are transformed too.
#[test]
fn low_level_transformations() {
    let transforms = |body: &str| {
        let (c, w) = at_letter(body);
        assert!(w.is_empty(), "{w:?}");
        c.items.iter().filter_map(|i| if let Item::Transform { transform } = i { Some(*transform) } else { None }).collect::<Vec<_>>()
    };
    let line = format!(r"\pgfpathmoveto{{\pgfpointorigin}}\pgfpathlineto{{{}}}\pgfusepath{{stroke}}", p(1.0, 0.0));
    let t = transforms(&format!(r"\pgflowlevel{{\pgftransformscale{{2}}}}{line}"));
    assert!(t.len() == 1 && close(t[0].a, 2.0) && close(t[0].d, 2.0), "{t:?}");
    let t = transforms(&format!(r"\pgflowlevelobj{{\pgftransformshift{{\pgfpoint{{1cm}}{{0cm}}}}}}{{{line}}}"));
    assert!(t.len() == 1 && close(t[0].e, 1.0), "{t:?}");
    let t = transforms(&format!(r"\begin{{pgflowlevelscope}}{{\pgftransformrotate{{90}}}}{line}\end{{pgflowlevelscope}}"));
    assert!(t.len() == 1 && close(t[0].b, 1.0), "{t:?}");
    // the coordinate transformation moves to the canvas, and is reset
    let (c, _) = at_letter(&format!(r"\pgftransformxshift{{1cm}}\pgflowlevelsynccm{line}"));
    let t: Vec<_> = c.items.iter().filter_map(|i| if let Item::Transform { transform } = i { Some(*transform) } else { None }).collect();
    assert!(t.len() == 1 && close(t[0].e, 1.0), "{t:?}");
    assert_eq!(points(&paths(&c)[0].segments)[0], (0.0, 0.0), "the path in the canvas's new system");
}

/// The nonlinear flatness is taken and has no effect of its own.
#[test]
fn nonlinear_flatness() {
    let d = drawn(&format!(r"\pgfsettransformnonlinearflatness{{2pt}}\pgfpathmoveto{{\pgfpointorigin}}\pgfpathlineto{{{}}}\pgfusepath{{stroke}}", p(1.0, 0.0)));
    assert_eq!(points(&d[0].segments), vec![(0.0, 0.0), (1.0, 0.0)]);
}

/// `\pgfscope` keeps the graphic state and transformations inside it.
#[test]
fn scopes() {
    let d = drawn(&format!(
        r"\pgfscope\pgfsetlinewidth{{2pt}}\pgftransformxshift{{1cm}}\pgfscope\pgfsetlinewidth{{5pt}}\endpgfscope\pgfpathmoveto{{\pgfpointorigin}}\pgfpathlineto{{{a}}}\pgfusepath{{stroke}}\endpgfscope\pgfpathmoveto{{\pgfpointorigin}}\pgfpathlineto{{{a}}}\pgfusepath{{stroke}}",
        a = p(1.0, 0.0)
    ));
    assert!(close(d[0].stroke.as_ref().unwrap().width, 2.0 * PT) && near(points(&d[0].segments)[0], (1.0, 0.0)));
    assert!(close(d[1].stroke.as_ref().unwrap().width, 0.4 * PT) && near(points(&d[1].segments)[0], (0.0, 0.0)));
}

/// A `pgfpicture` (the command pair or the environment) is a picture like `tikzpicture`.
#[test]
fn pgf_pictures() {
    for document in [
        r"\pgfpicture\pgfpathmoveto{\pgfpointorigin}\pgfpathlineto{\pgfpoint{2cm}{1cm}}\pgfusepath{stroke}\endpgfpicture",
        r"\begin{pgfpicture}\pgfpathmoveto{\pgfpointorigin}\pgfpathlineto{\pgfpoint{2cm}{1cm}}\pgfusepath{stroke}\end{pgfpicture}",
    ] {
        let (c, w) = tikra::engine::run_document(document).unwrap();
        assert!(w.is_empty(), "{w:?}");
        let (lo, hi) = (c.min.unwrap(), c.max.unwrap());
        assert!((hi.x - lo.x - 2.0 - 0.4 * PT).abs() < 1e-3, "{lo:?} {hi:?}");
    }
}

/// `\pgfsetbaseline` and the baseline at a point (now, or once the picture is drawn) put the
/// picture's baseline where `baseline=` does.
#[test]
fn baselines() {
    let first = |options: &str, inside: &str| {
        let document = format!(r"\begin{{tikzpicture}}[{options}]\useasboundingbox (0,-5) rectangle (20,5);{inside}\draw (1,-2) -- (2,3);\end{{tikzpicture}}");
        let (c, _) = tikra::engine::run_document(&document).unwrap();
        points(&paths(&c)[0].segments)[0]
    };
    let reference = first("baseline=1cm", "");
    assert!(!near(reference, first("baseline=0cm", "")));
    for inside in [r"\pgfsetbaseline{1cm}", r"\pgfsetbaselinepointnow{\pgfpoint{3cm}{1cm}}", r"\pgfsetbaselinepointlater{\pgfpoint{3cm}{1cm}}"] {
        assert!(near(first("", inside), reference), "{inside}: {:?} {reference:?}", first("", inside));
    }
}

// --- texts, nodes and coordinates ---

/// A `\pgftext` is a rectangle node without separation, placed by its anchor options.
#[test]
fn texts_and_their_anchors() {
    let place = |options: &str| {
        let (c, w) = at_letter(&format!(r"\pgftext[{options}]{{M}}"));
        assert!(w.is_empty(), "{w:?}");
        let (_, t) = texts(&c).into_iter().next().expect("a text");
        (t.e, t.f)
    };
    let center = place("");
    let left = place("left");
    let right = place("right");
    let (top, bottom, base) = (place("top"), place("bottom"), place("base"));
    assert!(near(left, (0.0, center.1)) && right.0 < center.0 && center.0 < left.0, "{left:?} {center:?} {right:?}");
    assert!(top.1 < center.1 && center.1 < bottom.1 && close(base.1, 0.0), "{top:?} {center:?} {bottom:?} {base:?}");
    assert!(near(place("base,left"), (0.0, 0.0)));
    assert!(near(place("top,right"), (right.0, top.1)));
    // an option it does not know changes nothing
    assert!(near(place("left,frobnicate=2"), left));
    // at, x and y move it; rotate turns it
    assert!(near(place(r"base,left,at={\pgfpoint{1cm}{2cm}},x=1cm,y=-1cm"), (2.0, 1.0)));
    let (c, _) = at_letter(r"\pgftext[rotate=90]{M}");
    let (_, t) = texts(&c).into_iter().next().unwrap();
    assert!(close(t.b, 1.0) && close(t.c, -1.0), "{t:?}");
    // under a scaling, the text is scaled too
    let (c, _) = at_letter(r"\pgftransformscale{2}\pgftext{M}");
    let (_, t) = texts(&c).into_iter().next().unwrap();
    assert!(close(t.a, 2.0), "{t:?}");
}

/// A shading as the text: its box placed by the anchor.
#[test]
fn shadings_as_texts() {
    let shading = r"\pgfdeclarehorizontalshading{h}{1cm}{rgb(0cm)=(1,0,0); rgb(2cm)=(0,0,1)}";
    let corners = |options: &str| {
        let (c, w) = at_letter(&format!(r"{shading}\pgftext[{options}]{{\pgfuseshading{{h}}}}"));
        assert!(w.is_empty(), "{w:?}");
        bounds(&paths(&c)[0].segments)
    };
    for (options, lo, hi) in [
        ("", (-1.0, -0.5), (1.0, 0.5)),
        ("left", (0.0, -0.5), (2.0, 0.5)),
        ("right", (-2.0, -0.5), (0.0, 0.5)),
        ("top", (-1.0, -1.0), (1.0, 0.0)),
        ("bottom", (-1.0, 0.0), (1.0, 1.0)),
        ("base", (-1.0, 0.0), (1.0, 1.0)),
    ] {
        let (a, b) = corners(options);
        assert!(near((a.x, a.y), lo) && near((b.x, b.y), hi), "{options}: {a:?} {b:?}");
    }
    // a shading never declared, or a text that starts with another command, is typeset as text
    for text in [r"\pgfuseshading{nothing}", r"\bfseries h"] {
        let (c, _) = at_letter(&format!(r"{shading}\pgftext{{{text}}}"));
        assert!(paths(&c).is_empty() && texts(&c).len() == 1, "{text}");
    }
}

/// A text in a declared shape's code: the shape draws it where `\pgftext` would.
#[test]
fn texts_of_a_declared_shape() {
    for anchor in ["", "left", "right", "top", "bottom", "base", "top,left", "top,right", "bottom,left", "bottom,right", "base,left", "base,right"] {
        let shape = format!(r"\pgfdeclareshape{{labelled}}{{\anchor{{center}}{{\pgfpointorigin}}\beforebackgroundpath{{\pgftext[{anchor},at={{\pgfpoint{{1cm}}{{0cm}}}}]{{M}}}}}}");
        let (c, w) = at_letter(&format!(r"{shape}\node[labelled] at (0,0) {{}};"));
        assert!(w.is_empty(), "{anchor}: {w:?}");
        let (_, from_shape) = texts(&c).into_iter().next().expect("the shape's text");
        let (c, _) = at_letter(&format!(r"\pgftext[{anchor},at={{\pgfpoint{{1cm}}{{0cm}}}}]{{M}}"));
        let (_, alone) = texts(&c).into_iter().next().unwrap();
        assert!(near((from_shape.e, from_shape.f), (alone.e, alone.f)), "{anchor}: {from_shape:?} {alone:?}");
    }
}

/// A declared shape's code that fills and strokes its own paths.
#[test]
fn a_declared_shape_fills_and_strokes() {
    let shape = r"\pgfdeclareshape{dot}{\anchor{center}{\pgfpointorigin}\backgroundpath{\pgfpathcircle{\pgfpointorigin}{3pt}\pgfsetfillcolor{red}\pgfusepath{fill,stroke}}}";
    let d = drawn(&format!(r"{shape}\node[dot] at (1,1) {{}};"));
    assert!(d.iter().any(|d| d.fill.as_ref().is_some_and(|f| matches!(f.paint, Paint::Solid(c) if c.r > 0.99 && c.g < 0.01))), "{d:?}");
    assert!(d.iter().any(|d| d.stroke.is_some()), "{d:?}");
}

/// `\pgfnode`: a node of a shape at the origin of the transformation, drawn as its usage says.
#[test]
fn basic_nodes() {
    for (usage, fill, stroke) in
        [(r"\pgfusepath{stroke}", false, true), (r"\pgfusepath{fill}", true, false), (r"\pgfusepath{draw,fill}", true, true), (r"\pgfusepath{discard}", false, false)]
    {
        let (c, w) = at_letter(&format!(r"\pgftransformshift{{\pgfpoint{{1cm}}{{1cm}}}}\pgfnode{{circle}}{{center}}{{x}}{{n}}{{{usage}}} \draw (n.center) -- (3,3);"));
        assert!(w.is_empty(), "{w:?}");
        let ps = paths(&c);
        let line = ps.last().unwrap();
        assert!(near(points(&line.segments)[0], (1.0, 1.0)), "{usage}: {:?}", line.segments);
        let border = ps.iter().find(|d| d.segments.len() > 2);
        assert_eq!(border.map(|b| (b.fill.is_some(), b.stroke.is_some())).unwrap_or((false, false)), (fill, stroke), "{usage}");
    }
    // a node without a name
    let (c, _) = at_letter(r"\pgfnode{rectangle}{center}{x}{}{\pgfusepath{stroke}}");
    assert_eq!(texts(&c).len(), 1);
}

/// `\pgfcoordinate`: a named point.
#[test]
fn basic_coordinates() {
    assert!(near(landing(r"\pgftransformxshift{1cm}\pgfcoordinate{c}{\pgfpoint{1cm}{2cm}}\pgftransformreset", r"\pgfpointanchor{c}{center}"), (2.0, 2.0)));
}

// --- plots ---

/// The stream of three points with a handler, used with `usage`.
fn stream(handler: &str, usage: &str) -> Vec<Drawn> {
    drawn(&format!(
        r"{handler}\pgfplotstreamstart\pgfplotstreampoint{{{}}}\pgfplotstreampoint{{{}}}\pgfplotstreampoint{{{}}}\pgfplotstreamend\pgfusepath{{{usage}}}",
        p(0.0, 0.0),
        p(1.0, 2.0),
        p(2.0, 1.0)
    ))
}

/// Each plot handler draws the stream as TikZ's `plot` of the same name draws its points.
#[test]
fn plot_handlers() {
    for (handler, name) in [
        ("lineto", "sharp plot"),
        ("polygon", "sharp cycle"),
        ("curveto", "smooth"),
        ("closedcurve", "smooth cycle"),
        ("xcomb", "xcomb"),
        ("ycomb", "ycomb"),
        ("polarcomb", "polar comb"),
        ("constantlineto", "const plot"),
        ("constantlinetomarkright", "const plot mark right"),
        ("constantlinetomarkmid", "const plot mark mid"),
        ("jumpmarkleft", "jump mark left"),
        ("jumpmarkright", "jump mark right"),
        ("jumpmarkmid", "jump mark mid"),
        ("xbar", "xbar"),
        ("ybar", "ybar"),
        ("xbarinterval", "xbar interval"),
        ("ybarinterval", "ybar interval"),
    ] {
        let basic = stream(&format!(r"\pgfplothandler{handler}"), "stroke");
        let (c, w) = at_letter(&format!(r"\draw[{name}] plot coordinates {{(0,0) (1,2) (2,1)}};"));
        let tikz = paths(&c);
        assert!(w.is_empty(), "{name}: {w:?}");
        assert_eq!(basic.len(), tikz.len(), "{handler}");
        for (a, b) in basic.iter().zip(&tikz) {
            assert_eq!(a.segments, b.segments, "{handler}");
        }
    }
    assert!(stream(r"\pgfplothandlerdiscard", "stroke").is_empty());
}

/// The gap handlers: a line between each two points (and back to the first, for the cycle), with
/// a gap at each point.
#[test]
fn gap_plot_handlers() {
    for (handler, lines) in [("gaplineto", 2), ("gapcycle", 3)] {
        let d = stream(&format!(r"\pgfplothandler{handler}"), "stroke");
        let s = &d[0].segments;
        assert_eq!(s.iter().filter(|s| matches!(s, Segment::MoveTo(_))).count(), lines, "{handler}: {s:?}");
        let pts = points(s);
        // the first line starts after the first point and ends before the second
        assert!(pts[0].0 > 0.0 && pts[0].1 > 0.0 && pts[1].0 < 1.0 && pts[1].1 < 2.0, "{handler}: {pts:?}");
    }
}

/// The tension of smooth plots.
#[test]
fn plot_tension() {
    let loose = stream(r"\pgfplothandlercurveto", "stroke");
    let tight = stream(r"\pgfsetplottension{0.2}\pgfplothandlercurveto", "stroke");
    assert_ne!(loose[0].segments, tight[0].segments);
    assert_eq!(points(&loose[0].segments), points(&tight[0].segments));
}

/// A plot goes on from the path's point with `\pgfsetlinetofirstplotpoint`; it moves there again
/// with `\pgfsetmovetofirstplotpoint`.
#[test]
fn first_plot_point() {
    let start = format!(r"\pgfpathmoveto{{{}}}", p(5.0, 5.0));
    let d = stream(&format!(r"{start}\pgfsetlinetofirstplotpoint\pgfplothandlerlineto"), "stroke");
    assert!(matches!(d[0].segments[1], Segment::LineTo(_)) && d[0].segments.iter().filter(|s| matches!(s, Segment::MoveTo(_))).count() == 1, "{:?}", d[0].segments);
    let d = stream(&format!(r"{start}\pgfsetlinetofirstplotpoint\pgfsetmovetofirstplotpoint\pgfplothandlerlineto"), "stroke");
    assert_eq!(d[0].segments.iter().filter(|s| matches!(s, Segment::MoveTo(_))).count(), 2);
}

/// Marks: the handler's code at each point (or at the points listed), shifted there.
#[test]
fn mark_handlers() {
    let dot = r"\pgfpathcircle{\pgfpointorigin}{2pt}\pgfusepath{fill}";
    let centers = |d: &[Drawn]| d.iter().map(|d| bounds(&d.segments)).map(|(a, b)| ((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)).collect::<Vec<_>>();
    let d = stream(&format!(r"\pgfplothandlermark{{{dot}}}"), "discard");
    let c = centers(&d);
    assert!(c.len() == 3 && near(c[1], (1.0, 2.0)), "{c:?}");
    let d = stream(&format!(r"\pgfplothandlermarklisted{{{dot}}}{{1,3}}"), "discard");
    let c = centers(&d);
    assert!(c.len() == 2 && near(c[0], (0.0, 0.0)) && near(c[1], (2.0, 1.0)), "{c:?}");
}

/// Plot marks by name, their size, and declared ones.
#[test]
fn plot_marks() {
    let d = drawn(r"\pgfsetplotmarksize{3pt}\pgfuseplotmark{*}");
    let (a, b) = bounds(&d[0].segments);
    assert!(close(b.x - a.x, 6.0 * PT), "{a:?} {b:?}");
    let d = drawn(r"\pgfdeclareplotmark{bar}{\pgfpathmoveto{\pgfpointorigin}\pgfpathlineto{\pgfpoint{0cm}{1cm}}\pgfusepath{stroke}}\pgfuseplotmark{bar}");
    assert_eq!(points(&d[0].segments), vec![(0.0, 0.0), (0.0, 1.0)]);
}

/// `\pgfplothandlerrecord`: the stream kept in a macro, played again later with another handler.
#[test]
fn recorded_plots() {
    let d = drawn(&format!(
        r"\pgfplothandlerrecord{{\kept}}\pgfplotstreamstart\pgfplotstreampoint{{{}}}\pgfplotstreampoint{{{}}}\pgfplotstreamend\pgfplothandlerlineto\kept\pgfusepath{{stroke}}",
        p(1.0, 1.0),
        p(2.0, 3.0)
    ));
    assert!(near(points(&d[0].segments)[0], (1.0, 1.0)) && near(points(&d[0].segments)[1], (2.0, 3.0)), "{:?}", d[0].segments);
}

/// `\pgfplotfunction`: a point for each item of the list.
#[test]
fn plotted_functions() {
    let pts = first_points(r"\pgfplothandlerlineto\pgfplotfunction{\x}{0,1,2}{\pgfpointxy{\x}{\x*\x}}\pgfusepath{stroke}");
    assert_eq!(pts.len(), 3);
    assert!(near(pts[2], (2.0, 4.0)), "{pts:?}");
    // a variable that is not a macro: the same point each time
    let pts = first_points(r"\pgfplothandlerlineto\pgfplotfunction{x}{0,1}{\pgfpoint{1cm}{1cm}}\pgfusepath{stroke}");
    assert_eq!(pts, vec![(1.0, 1.0), (1.0, 1.0)]);
}

/// A declared plot handler: its code at each point, the point as `##1`, its arguments as `#1`….
#[test]
fn declared_plot_handlers() {
    let handler = r"\pgfdeclareplothandler{\dots}{#1}{point={\pgfpathcircle{##1}{#1}}}";
    let d = stream(&format!(r"{handler}\dots{{4pt}}"), "fill");
    let (a, b) = bounds(&d[0].segments);
    assert!(near((a.x, a.y), (-4.0 * PT, -4.0 * PT)) && near((b.x, b.y), (2.0 + 4.0 * PT, 2.0 + 4.0 * PT)), "{a:?} {b:?}");
    assert_eq!(d[0].segments.iter().filter(|s| matches!(s, Segment::MoveTo(_))).count(), 3);
    // no macro to declare: nothing happens
    assert!(warned(r"\pgfdeclareplothandler{dots}{#1}{point={}}").is_empty());
}

// --- utilities ---

/// The text of a node, from the SVG's title.
fn typeset(document_body: &str) -> String {
    let r = no_warnings(&format!("\\makeatletter\\begin{{tikzpicture}}{document_body}\\end{{tikzpicture}}"));
    r.alt
}

/// `\pgfmathdectobase`: digits in a base, padded to `\pgfmathbasenumberlength`.
#[test]
fn numbers_in_other_bases() {
    for (setup, n, base, digits) in
        [("", "10", "2", "1010"), ("", "255", "16", "ff"), ("", "0", "8", "0"), ("", "-5", "2", "101"), (r"\pgfmathsetbasenumberlength{6}", "10", "2", "001010")]
    {
        let alt = typeset(&format!(r"{setup}\pgfmathdectobase{{\r}}{{{n}}}{{{base}}}\node {{[\r]}};"));
        assert!(alt.contains(&format!("[{digits}]")), "{n} in base {base}: {alt}");
    }
    // a target that is not a macro: nothing set
    assert!(typeset(r"\def\r{none}\pgfmathdectobase{r}{10}{2}\node {[\r]};").contains("[none]"));
}

/// `\pgfmathtodigitlist`: a number's digits as a list.
#[test]
fn digit_lists() {
    assert!(typeset(r"\pgfmathtodigitlist{\l}{205}\node {[\l]};").contains("[2,0,5]"));
    assert!(typeset(r"\def\l{none}\pgfmathtodigitlist{l}{205}\node {[\l]};").contains("[none]"));
}

/// `\pgfgetlastxy`: the last point of the path, in pt.
#[test]
fn last_point() {
    let pts = first_points(&format!(r"\pgfpathmoveto{{{}}}\pgfgetlastxy{{\x}}{{\y}}\pgfpathlineto{{\pgfqpoint{{\y}}{{\x}}}}\pgfusepath{{stroke}}", p(1.0, 2.0)));
    assert!(near(pts[1], (2.0, 1.0)), "{pts:?}");
    // after the path is used: its last point still; a target that is not a macro is skipped
    let d = drawn(&format!(
        r"\pgfpathmoveto{{{}}}\pgfpathlineto{{{}}}\pgfusepath{{stroke}}\def\x{{0pt}}\pgfgetlastxy{{x}}{{\y}}\pgfpathmoveto{{\pgfqpoint{{\x}}{{\y}}}}\pgfpathlineto{{\pgfpointorigin}}\pgfusepath{{stroke}}",
        p(0.0, 0.0),
        p(3.0, 1.0)
    ));
    assert!(near(points(&d[1].segments)[0], (0.0, 1.0)), "{:?}", d[1].segments);
}

/// keyval's `\setkeys` runs the code `\define@key` gave each key.
#[test]
fn keyval_keys() {
    let alt = typeset(r"\define@key{fam}{size}{\def\size{#1}}\define@key{fam}{flag}{\def\flag{on#1}}\setkeys{fam}{size=7, flag}\node {[\size][\flag]};");
    assert!(alt.contains("[7][on]"), "{alt}");
}
