//! More of the decorations: the library's less common automata (lineto over corners, curveto,
//! bent, triangles, the circle connection bar, the Koch curves, straight zigzag), decorations and
//! meta-decorations declared in the document with every state option, markings, shape
//! backgrounds spread along the path, text along a path and text effects, `reverse path`, the
//! basic layer's decoration commands, and `path picture`.

use tikra::canvas::Item;
use tikra::engine::run_document;
use tikra::scene::{Point, Segment};
use tikra::PT;

/// The items and warnings of a picture whose coordinates are those of the page (cm).
fn picture(body: &str) -> (Vec<Item>, Vec<String>) {
    let (c, w) = run_document(&format!("\\begin{{tikzpicture}}[baseline=0pt]\\useasboundingbox (0,-5) rectangle (20,5);{body}\\end{{tikzpicture}}"))
        .unwrap_or_else(|e| panic!("{e}\n{body}"));
    (c.items, w)
}

/// A picture that must draw without warnings.
fn clean(body: &str) -> Vec<Item> {
    let (items, w) = picture(body);
    assert!(w.is_empty(), "{w:?}\n{body}");
    items
}

/// A picture that must warn with `fragment`.
fn warned(body: &str, fragment: &str) -> Vec<Item> {
    let (items, w) = picture(body);
    assert!(w.iter().any(|w| w.contains(fragment)), "expected '{fragment}' in {w:?}\n{body}");
    items
}

fn paths(items: &[Item]) -> Vec<Vec<Segment>> {
    items.iter().filter_map(|i| if let Item::Path { segments, .. } = i { Some(segments.clone()) } else { None }).collect()
}

/// The places (x, y of the baseline start) of the texts.
fn text_places(items: &[Item]) -> Vec<(f64, f64)> {
    items.iter().filter_map(|i| if let Item::Text { place, .. } = i { Some((place.e, place.f)) } else { None }).collect()
}

fn ends(segs: &[Segment]) -> Vec<Point> {
    segs.iter()
        .filter_map(|s| match s {
            Segment::MoveTo(p) | Segment::LineTo(p) | Segment::CurveTo(_, _, p) => Some(*p),
            Segment::Close => None,
        })
        .collect()
}

fn near(p: Point, x: f64, y: f64) -> bool {
    (p.x - x).abs() < 1e-4 && (p.y - y).abs() < 1e-4
}

fn closes(segs: &[Segment]) -> usize {
    segs.iter().filter(|s| matches!(s, Segment::Close)).count()
}

/// The closed pieces of a path (from a moveto to its closepath), as their end points.
fn pieces(segs: &[Segment]) -> Vec<Vec<Point>> {
    let mut out = Vec::new();
    let mut cur = Vec::new();
    for s in segs {
        match s {
            Segment::MoveTo(p) => cur = vec![*p],
            Segment::LineTo(p) | Segment::CurveTo(_, _, p) => cur.push(*p),
            Segment::Close => out.push(std::mem::take(&mut cur)),
        }
    }
    out
}

/// Smallest and largest x and y of points.
fn extent(pts: &[Point]) -> (f64, f64, f64, f64) {
    let f = |g: fn(&Point) -> f64, min: bool| pts.iter().map(g).fold(if min { f64::MAX } else { f64::MIN }, |a, b| if min { a.min(b) } else { a.max(b) });
    (f(|p| p.x, true), f(|p| p.x, false), f(|p| p.y, true), f(|p| p.y, false))
}

// ---------------------------------------------------------------- the library's automata

/// `lineto` follows each input segment, corners included.
#[test]
fn lineto_follows_every_corner() {
    let p = ends(&paths(&clean(r"\draw[decorate, decoration=lineto] (0,0) -- (1,0) -- (1,1) -- (3,1);"))[0]);
    assert_eq!(p.len(), 4, "{p:?}");
    for (q, (x, y)) in p.iter().zip([(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (3.0, 1.0)]) {
        assert!(near(*q, x, y), "{p:?}");
    }
}

/// `curveto` replaces a curve by many short lines on it.
#[test]
fn curveto_is_many_lines_on_the_curve() {
    // a quarter of a circle of radius 2 drawn as `arc`: each line ends on the circle
    let p = ends(&paths(&clean(r"\draw[decorate, decoration=curveto] (2,0) arc[start angle=0, end angle=90, radius=2];"))[0]);
    assert!(p.len() > 50, "{}", p.len());
    for q in &p {
        assert!((q.x.hypot(q.y) - 2.0).abs() < 2e-3, "{q:?}");
    }
    assert!(near(*p.last().unwrap(), 0.0, 2.0), "{p:?}");
}

/// `bent` is one curve per segment, its controls at the amplitude and `aspect` along.
#[test]
fn bent_bows_each_segment() {
    let s = &paths(&clean(r"\draw[decorate, decoration={bent, amplitude=10pt, aspect=0.25}] (0,0) -- (4,0);"))[0];
    match s[1] {
        Segment::CurveTo(a, b, q) => {
            assert!(near(a, 1.0, 10.0 * PT) && near(b, 3.0, 10.0 * PT), "{s:?}");
            assert!(near(q, 4.0, 0.0), "{s:?}");
        }
        _ => panic!("{s:?}"),
    }
}

/// `triangles`: closed triangles pointing along the path, `shape start width` long.
#[test]
fn triangles_point_along_the_path() {
    let s = &paths(&clean(r"\draw[decorate, decoration={triangles, segment length=5mm, shape start width=2mm, shape start height=4mm}] (0,0) -- (2,0);"))[0];
    let tri = pieces(s);
    assert!(tri.len() >= 4, "{tri:?}");
    // the first: its back edge across the path at the start, its tip 2mm ahead
    assert!(near(tri[0][0], 0.0, 0.2) && near(tri[0][1], 0.2, 0.0) && near(tri[0][2], 0.0, -0.2), "{:?}", tri[0]);
    // one every segment length
    assert!(near(tri[1][1], 0.7, 0.0), "{:?}", tri[1]);
}

/// The mindmap library's `circle connection bar`: a start cap, the bar and an end cap.
#[test]
fn circle_connection_bar_has_three_closed_pieces() {
    let s = &paths(&clean(r"\draw[decorate, decoration={circle connection bar, start radius=4mm, end radius=6mm, amplitude=2mm}] (0,0) -- (4,0);"))[0];
    let p = pieces(s);
    assert_eq!(p.len(), 3, "{s:?}");
    // the caps sit around the ends: inside the circles of the two radii
    let (x0, x1, ..) = extent(&p[0]);
    assert!(x0 > -0.41 && x1 < 0.41, "{:?}", p[0]);
    let (x0, x1, ..) = extent(&p[2]);
    assert!(x0 > 4.0 - 0.61 && x1 < 4.61, "{:?}", p[2]);
    // the bar between them is as thick as the amplitude
    let (_, _, y0, y1) = extent(&p[1]);
    assert!((y1 - y0 - 0.2).abs() < 1e-3, "{:?}", p[1]);
}

/// `Koch curve type 2` replaces a line by eight of a quarter its length, up then down.
#[test]
fn koch_curve_type_2() {
    let p = ends(&paths(&clean(r"\draw[decorate, decoration=Koch curve type 2] (0,0) -- (4,0);"))[0]);
    let want = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (2.0, 1.0), (2.0, 0.0), (2.0, -1.0), (3.0, -1.0), (3.0, 0.0), (4.0, 0.0)];
    assert_eq!(p.len(), want.len(), "{p:?}");
    for (q, (x, y)) in p.iter().zip(want) {
        assert!(near(*q, x, y), "{p:?}");
    }
}

/// The meta-decoration `straight zigzag`: straight stretches and zigzags in turn, each a
/// meta-segment long.
#[test]
fn straight_zigzag_alternates() {
    for body in [
        r"\draw[decorate, decoration={straight zigzag, meta-segment length=1cm}] (0,0) -- (3,0);",
        r"\pgfmetadecoration{straight zigzag}\pgfpathmoveto{\pgfpointorigin}\pgfpathlineto{\pgfpoint{3cm}{0cm}}\endpgfmetadecoration\pgfusepath{stroke}",
    ] {
        let p = ends(&paths(&clean(body))[0]);
        let off: Vec<&Point> = p.iter().filter(|q| q.y.abs() > 1e-6).collect();
        // the zigzag only between 1cm and 2cm
        assert!(!off.is_empty(), "{p:?}");
        assert!(off.iter().all(|q| q.x > 1.0 - 1e-6 && q.x < 2.0 + 1e-6), "{off:?}");
        assert!(near(*p.last().unwrap(), 3.0, 0.0), "{p:?}");
    }
}

/// `saw` on a path of two closed parts: each part ends with a closepath where it ends.
#[test]
fn saw_closes_each_closed_part() {
    let s = &paths(&clean(r"\draw[decorate, decoration={saw, segment length=2mm, amplitude=1mm}] (0,0) -- (1,0) -- (1,1) -- cycle (2,0) -- (3,0) -- (3,1) -- cycle;"))[0];
    assert_eq!(closes(s), 2, "{s:?}");
    // each closepath comes back to the start of its part
    let i = s.iter().position(|x| matches!(x, Segment::Close)).unwrap();
    let before = ends(&s[..i]);
    assert!(near(*before.last().unwrap(), 0.0, 0.0) || before.last().unwrap().y > 0.0, "{before:?}");
    assert!(s[i + 1..].iter().any(|x| matches!(x, Segment::MoveTo(q) if near(*q, 2.0, 0.0)) || matches!(x, Segment::LineTo(q) if q.x >= 2.0)), "{s:?}");
}

/// Footprints of an animal pgf does not know are a human's, with a warning.
#[test]
fn footprints_of_an_unknown_animal() {
    let human = paths(&clean(r"\fill[decorate, decoration={footprints, foot of=human}] (0,0) -- (2,0);"));
    let unknown = paths(&warned(r"\fill[decorate, decoration={footprints, foot of=dragon}] (0,0) -- (2,0);", "footprints of 'dragon' not known"));
    assert_eq!(format!("{human:?}"), format!("{unknown:?}"));
    // the gnome's feet are lines, unlike the human's curves
    let gnome = &paths(&clean(r"\fill[decorate, decoration={footprints, foot of=gnome}] (0,0) -- (2,0);"))[0];
    assert!(gnome.iter().any(|s| matches!(s, Segment::LineTo(_))), "{gnome:?}");
    // an empty foot angle is the initial 10 degrees
    let a = paths(&clean(r"\fill[decorate, decoration={footprints, foot angle=10}] (0,0) -- (2,0);"));
    let b = paths(&clean(r"\fill[decorate, decoration={footprints, foot angle=}] (0,0) -- (2,0);"));
    assert_eq!(format!("{a:?}"), format!("{b:?}"));
}

/// `pre` and `post` name the decorations of the stretches before and after; an unknown one is a
/// line, with a warning. An empty length is no stretch; a length that is not one is reported.
#[test]
fn pre_and_post_decorations() {
    for key in ["pre", "post"] {
        let s = &paths(&warned(
            &format!(r"\draw[decorate, decoration={{zigzag, {key}=nothing, {key} length=5mm}}] (0,0) -- (2,0);"),
            "decoration 'nothing' not supported: drawn as lineto",
        ))[0];
        let p = ends(s);
        // a straight stretch of 5mm at that end
        if key == "pre" {
            assert!(near(p[1], 0.5, 0.0), "{p:?}");
        } else {
            let n = p.len();
            assert!(near(p[n - 1], 2.0, 0.0) && p[n - 2].x < 1.5 + 1e-6, "{p:?}");
        }
    }
    let plain = paths(&clean(r"\draw[decorate, decoration=zigzag] (0,0) -- (2,0);"));
    let empty = paths(&clean(r"\draw[decorate, decoration={zigzag, pre length={}}] (0,0) -- (2,0);"));
    assert_eq!(ends(&plain[0]).iter().filter(|q| q.y.abs() > 1e-6).count(), ends(&empty[0]).iter().filter(|q| q.y.abs() > 1e-6).count());
    let s = &paths(&warned(r"\draw[decorate, decoration={zigzag, pre length=wide}] (0,0) -- (2,0);", "decoration: pre length"))[0];
    assert!(ends(s).iter().any(|q| q.y.abs() > 1e-6), "{s:?}");
}

/// `show path construction` runs the code of each kind of segment, moveto and closepath too.
#[test]
fn show_path_construction_moveto_and_closepath() {
    let items = clean(
        r"\draw[decorate, decoration={show path construction,
            moveto code={\fill[red] (\tikzinputsegmentfirst) circle (1pt);},
            closepath code={\draw[blue] (\tikzinputsegmentfirst) -- (\tikzinputsegmentlast);}}]
          (0,0) -- (1,0) -- (1,1) -- cycle;",
    );
    let p = paths(&items);
    // a dot at the moveto, and the closepath from (1,1) back to (0,0)
    let dot = p.iter().find(|s| s.iter().any(|x| matches!(x, Segment::CurveTo(..)))).expect("the dot");
    let (x0, x1, y0, y1) = extent(&ends(dot)[1..]);
    assert!(((x0 + x1) / 2.0).abs() < 1e-6 && ((y0 + y1) / 2.0).abs() < 1e-6, "{dot:?}");
    assert!(p.iter().any(|s| s.len() == 2 && near(ends(s)[0], 1.0, 1.0) && near(ends(s)[1], 0.0, 0.0)), "{p:?}");
}

/// Marks: a step that is not positive is reported, a negative position counts from the end,
/// and a connection node that does not exist is reported.
#[test]
fn markings_edge_cases() {
    let items = warned(
        r"\draw[postaction={decorate}, decoration={markings, mark=between positions 0 and 1 step 0pt with {\fill circle (1pt);}}] (0,0) -- (2,0);",
        "the step must be positive",
    );
    assert_eq!(paths(&items).len(), 1);
    // -0.25: a quarter of the path before the end
    let items = clean(r"\draw[postaction={decorate}, decoration={markings, mark=at position -0.25 with {\coordinate (m);}}] (0,0) -- (2,0); \fill (m) circle (1pt);");
    let dot = &paths(&items)[1];
    let (x0, x1, ..) = extent(&ends(dot));
    assert!(((x0 + x1) / 2.0 - 1.5).abs() < 1e-4, "{dot:?}");
    warned(
        r"\draw[postaction={decorate}, decoration={markings, mark connection node=nowhere, mark=at position 0.5 with {\node {x};}}] (0,0) -- (2,0);",
        "mark connection node: no node named 'nowhere'",
    );
}

/// `\arrow[options]{tip}` in a mark: the options (here a color and a scale) apply to the tip.
#[test]
fn an_arrow_in_a_mark_takes_options() {
    let items = clean(r"\draw[postaction={decorate}, decoration={markings, mark=at position 0.5 with {\arrow[red, scale=2]{>}}}] (0,0) -- (2,0);");
    let small = clean(r"\draw[postaction={decorate}, decoration={markings, mark=at position 0.5 with {\arrow{>}}}] (0,0) -- (2,0);");
    let tip = |items: &[Item]| {
        let p = &paths(items)[1];
        let (x0, x1, y0, y1) = extent(&ends(p));
        (x1 - x0, y1 - y0)
    };
    let (w2, h2) = tip(&items);
    let (w1, h1) = tip(&small);
    assert!((h2 / h1 - 2.0).abs() < 1e-2 && (w2 / w1 - 2.0).abs() < 1e-2, "{w1} {h1} {w2} {h2}");
    let red = items.iter().any(|i| matches!(i, Item::Path { stroke: Some(s), .. } if s.color.r == 1.0 && s.color.g == 0.0));
    assert!(red, "{items:?}");
}

// ---------------------------------------------------------------- shape backgrounds

/// Rectangles of `shape backgrounds`, as (left, right, bottom, top).
fn boxes(body: &str) -> Vec<(f64, f64, f64, f64)> {
    let items = clean(body);
    paths(&items).first().map(|s| pieces(s).iter().map(|p| extent(p)).collect()).unwrap_or_default()
}

/// `shape evenly spread`: so many shapes, the ends at the ends of the path (`by borders`) or
/// their centers; one is at the middle, none is no shape.
#[test]
fn shapes_evenly_spread() {
    // four growing squares, as far apart from border to border
    let b = boxes(
        r"\fill[decorate, decoration={shape backgrounds, shape=rectangle, shape start size=2mm, shape end size=6mm, shape scaled, shape evenly spread={4, by borders}}] (0,0) -- (4,0);",
    );
    assert_eq!(b.len(), 4, "{b:?}");
    for (i, w) in [0.2, 0.2 + 0.4 / 3.0, 0.2 + 0.8 / 3.0, 0.6].iter().enumerate() {
        assert!((b[i].1 - b[i].0 - w).abs() < 1e-4, "{b:?}");
    }
    let gaps: Vec<f64> = b.windows(2).map(|w| w[1].0 - w[0].1).collect();
    assert!(gaps.iter().all(|g| (g - gaps[0]).abs() < 1e-3) && gaps[0] > 0.9, "{gaps:?}");
    // one shape: at the middle, both ways
    for spread in ["{1, by borders}", "1"] {
        let b = boxes(&format!(r"\fill[decorate, decoration={{shape backgrounds, shape=rectangle, shape size=2mm, shape evenly spread={spread}}}] (0,0) -- (4,0);"));
        assert_eq!(b.len(), 1, "{b:?}");
        assert!(((b[0].0 + b[0].1) / 2.0 - 2.0).abs() < 1e-4, "{b:?}");
    }
    // none
    for spread in ["{0, by borders}", "0"] {
        let b = boxes(&format!(r"\fill[decorate, decoration={{shape backgrounds, shape=rectangle, shape size=2mm, shape evenly spread={spread}}}] (0,0) -- (4,0);"));
        assert!(b.is_empty(), "{b:?}");
    }
}

/// `shape sep={…, between borders}` with `shape scaled`: the gap is from border to border while
/// the shapes grow.
#[test]
fn scaled_shapes_between_borders() {
    let b = boxes(
        r"\fill[decorate, decoration={shape backgrounds, shape=rectangle, shape start size=2mm, shape end size=6mm, shape scaled, shape sep={5mm, between borders}}] (0,0) -- (4,0);",
    );
    assert!(b.len() >= 4, "{b:?}");
    for w in b.windows(2) {
        assert!((w[1].0 - w[0].1 - 0.5).abs() < 1e-3, "{b:?}");
        assert!(w[1].1 - w[1].0 > w[0].1 - w[0].0, "{b:?}");
    }
}

/// `shape sloped=false` keeps the shapes upright on a slanted path.
#[test]
fn shapes_not_sloped_stay_upright() {
    let upright = boxes(r"\fill[decorate, decoration={shape backgrounds, shape=rectangle, shape size=2mm, shape sloped=false}] (0,0) -- (2,2);");
    assert!((upright[0].1 - upright[0].0 - 0.2).abs() < 1e-6 && (upright[0].3 - upright[0].2 - 0.2).abs() < 1e-6, "{upright:?}");
    let sloped = boxes(r"\fill[decorate, decoration={shape backgrounds, shape=rectangle, shape size=2mm}] (0,0) -- (2,2);");
    assert!((sloped[0].1 - sloped[0].0 - 0.2 * 2f64.sqrt()).abs() < 1e-6, "{sloped:?}");
}

/// Other node shapes are drawn with their own background path; an unknown one is a rectangle.
#[test]
fn shapes_from_the_node_shapes() {
    let items = clean(r"\usetikzlibrary{shapes.misc}\fill[decorate, decoration={shape backgrounds, shape=rounded rectangle, shape width=6mm, shape height=3mm}] (0,0) -- (2,0);");
    let s = &paths(&items)[0];
    let first = &pieces(s)[0];
    let (x0, x1, y0, y1) = extent(first);
    assert!((x1 - x0 - 0.6).abs() < 1e-3 && (y1 - y0 - 0.3).abs() < 2e-2, "{first:?}");
    assert!(s.iter().any(|x| matches!(x, Segment::CurveTo(..))), "{s:?}");
    let b = {
        let items = warned(r"\fill[decorate, decoration={shape backgrounds, shape=no such shape, shape size=4mm}] (0,0) -- (2,0);", "shape 'no such shape' not supported");
        pieces(&paths(&items)[0]).iter().map(|p| extent(p)).collect::<Vec<_>>()
    };
    assert!((b[0].1 - b[0].0 - 0.4).abs() < 1e-6 && (b[0].3 - b[0].2 - 0.4).abs() < 1e-6, "{b:?}");
}

// ---------------------------------------------------------------- text along path

/// `fit to path` spreads the letters to the end of the path; `stretching spaces` only widens the
/// spaces; a text too long for the path is set as it is.
#[test]
fn text_fitted_to_the_path() {
    let fit = text_places(&clean(r"\path[decorate, decoration={text along path, text={ab cd}, text align={fit to path}}] (0,0) -- (4,0);"));
    let spaces = text_places(&clean(r"\path[decorate, decoration={text along path, text={ab cd}, text align={fit to path stretching spaces}}] (0,0) -- (4,0);"));
    let plain = text_places(&clean(r"\path[decorate, decoration={text along path, text={ab cd}}] (0,0) -- (4,0);"));
    assert_eq!(fit.len(), 4);
    // fitted: the last letter ends at the end of the path; every gap the same
    let gaps: Vec<f64> = fit.windows(2).map(|w| w[1].0 - w[0].0).collect();
    assert!(gaps[0] > 0.5 && fit[3].0 > 3.7, "{fit:?}");
    // stretched spaces: the letters of a word stay together, as without fitting
    assert!((spaces[1].0 - spaces[0].0 - (plain[1].0 - plain[0].0)).abs() < 1e-6, "{spaces:?}");
    assert!((spaces[3].0 - spaces[2].0 - (plain[3].0 - plain[2].0)).abs() < 1e-6, "{spaces:?}");
    assert!(spaces[2].0 > 3.5, "{spaces:?}");
    // too long: as wide as without fitting, and cut at the end of the path
    let long = text_places(&clean(r"\path[decorate, decoration={text along path, text={abcdefghijklmnopqrstuvwxyz}, text align={fit to path}}] (0,0) -- (1,0);"));
    let unfitted = text_places(&clean(r"\path[decorate, decoration={text along path, text={abcdefghijklmnopqrstuvwxyz}}] (0,0) -- (1,0);"));
    assert_eq!(long, unfitted);
    assert!(long.len() < 26 && long.last().unwrap().0 < 1.0, "{long:?}");
}

/// The text is scanned as pgf scans it: a group is one box, `$…$` is math, `|…|` sets the format
/// (`|+…|` adds to it), `\ ` is a space, a macro is expanded, other commands are boxes.
#[test]
fn text_along_path_scans_groups_math_formats_and_macros() {
    let items = clean(r"\path[decorate, decoration={text along path, text={a{bc}$x$|\bfseries|d|+\itshape|e\ f\relax}}] (0,0) -- (6,0);");
    let labels: Vec<f64> = items.iter().filter_map(|i| if let Item::Text { label, .. } = i { Some(label.width()) } else { None }).collect();
    // a, bc, x, d, e, f and the empty box of \relax (the space is no text)
    assert_eq!(labels.len(), 7, "{labels:?}");
    assert!(labels[1] > labels[0] * 1.5, "the group is one box: {labels:?}");
    assert!(labels[6].abs() < 1e-9, "{labels:?}");
    let places = text_places(&items);
    // the space between e and f
    assert!(places[5].0 - places[4].0 > labels[4] + 0.05, "{places:?}");
    // a macro: its letters
    let items = clean(r"\def\word{xy}\path[decorate, decoration={text along path, text={a\word b}}] (0,0) -- (6,0);");
    assert_eq!(text_places(&items).len(), 4);
    // other delimiters for the format
    let items = clean(r"\path[decorate, decoration={text along path, text format delimiters={[}{]}, text={a[\bfseries]b}}] (0,0) -- (6,0);");
    assert_eq!(text_places(&items).len(), 2);
}

/// `text align=right` and `center` move the text along the path.
#[test]
fn text_along_path_aligned() {
    let left = text_places(&clean(r"\path[decorate, decoration={text along path, text={abc}}] (0,0) -- (4,0);"));
    let right = text_places(&clean(r"\path[decorate, decoration={text along path, text={abc}, text align=right}] (0,0) -- (4,0);"));
    let center = text_places(&clean(r"\path[decorate, decoration={text along path, text={abc}, text align=center}] (0,0) -- (4,0);"));
    let shift = right[0].0 - left[0].0;
    assert!(shift > 3.0, "{right:?}");
    assert!((center[0].0 - left[0].0 - shift / 2.0).abs() < 1e-4, "{center:?}");
}

// ---------------------------------------------------------------- text effects

/// The characters of text effects, as their nodes' places along the path.
fn letters(body: &str) -> Vec<(f64, f64)> {
    text_places(&clean(body))
}

/// A group is one character; `scale text to path` and `fit text to path` spread the characters
/// to the end of the path; `align=center` and `right` move them.
#[test]
fn text_effects_fit_scale_and_align() {
    let group = letters(r"\path[decoration={text effects along path, text={a{bc}d}}, decorate] (0,0) -- (4,0);");
    assert_eq!(group.len(), 3, "{group:?}");
    let plain = letters(r"\path[decoration={text effects along path, text={abc}}, decorate] (0,0) -- (4,0);");
    for key in ["fit text to path", "scale text to path"] {
        let l = letters(&format!(r"\path[decoration={{text effects along path, text={{abc}}, text effects/{key}}}, decorate] (0,0) -- (4,0);"));
        assert_eq!(l.len(), 3, "{key}");
        assert!(l[2].0 - l[0].0 > 2.0 * (plain[2].0 - plain[0].0), "{key}: {l:?}");
    }
    let right = letters(r"\path[decoration={text effects along path, text={abc}, text align=right}, decorate] (0,0) -- (4,0);");
    let center = letters(r"\path[decoration={text effects along path, text={abc}, text align=center}, decorate] (0,0) -- (4,0);");
    let shift = right[0].0 - plain[0].0;
    assert!(shift > 2.5, "{right:?}");
    assert!((center[0].0 - plain[0].0 - shift / 2.0).abs() < 1e-3, "{center:?}");
}

/// `group letters` makes a node of each word; `reverse text` the characters backwards;
/// `repeat text` the text again while there is path.
#[test]
fn text_effects_group_reverse_and_repeat() {
    let items = clean(r"\path[decoration={text effects along path, text={ab cd}, text effects/group letters}, decorate] (0,0) -- (4,0);");
    // two words and the space
    assert_eq!(text_places(&items).len(), 3);
    let fwd = clean(r"\path[decoration={text effects along path, text={ab}}, decorate] (0,0) -- (4,0);");
    let rev = clean(r"\path[decoration={text effects along path, text={ab}, text effects/reverse text}, decorate] (0,0) -- (4,0);");
    let widths = |items: &[Item]| items.iter().filter_map(|i| if let Item::Text { label, .. } = i { Some(label.width()) } else { None }).collect::<Vec<_>>();
    let (f, r) = (widths(&fwd), widths(&rev));
    assert!((f[0] - r[1]).abs() < 1e-9 && (f[1] - r[0]).abs() < 1e-9 && (f[0] - f[1]).abs() > 1e-6, "{f:?} {r:?}");
    let once = letters(r"\path[decoration={text effects along path, text={ab}}, decorate] (0,0) -- (4,0);");
    let twice = letters(r"\path[decoration={text effects along path, text={ab}, text effects/repeat text=1}, decorate] (0,0) -- (4,0);");
    assert_eq!((once.len(), twice.len()), (2, 4));
}

/// `character command` wraps each character; `style characters` gives some characters options;
/// `segment from width=false` spaces them by their height.
#[test]
fn text_effects_commands_styles_and_heights() {
    let items = clean(r"\path[decoration={text effects along path, text={ab}, text effects/character command=\bfseries}, decorate] (0,0) -- (4,0);");
    let normal = clean(r"\path[decoration={text effects along path, text={ab}}, decorate] (0,0) -- (4,0);");
    let w = |items: &[Item]| items.iter().filter_map(|i| if let Item::Text { label, .. } = i { Some(label.width()) } else { None }).sum::<f64>();
    assert!(w(&items) > w(&normal), "bold is wider");
    // the b drawn with a border (a stroked path)
    let items = clean(r"\path[decoration={text effects along path, text={abc}, text effects/style characters=b with {draw}}, decorate] (0,0) -- (4,0);");
    assert_eq!(items.iter().filter(|i| matches!(i, Item::Path { stroke: Some(_), .. })).count(), 1);
    // nodes 1cm high: 1cm apart
    let tall = letters(
        r"\path[decoration={text effects along path, text={abc}, text effects/.cd, segment from width=false, every character/.style={minimum height=1cm}}, decorate] (0,0) -- (4,0);",
    );
    assert!((tall[1].0 - tall[0].0 - 1.0).abs() < 0.1 && (tall[2].0 - tall[1].0 - 1.0).abs() < 0.1, "{tall:?}");
}

// ---------------------------------------------------------------- reverse path, long curves

/// `reverse path` decorates the path from its end: the subpaths backwards, a closed one and a
/// curve too.
#[test]
fn reverse_path_starts_at_the_end() {
    let s = &paths(&clean(r"\draw[decorate, decoration={lineto, reverse path}] (0,0) -- (1,0) .. controls (2,1) and (3,1) .. (4,0) -- cycle (5,0) -- (6,0);"))[0];
    let p = ends(s);
    // the last subpath backwards, then the closed one from its start: the closing line, the
    // curve (a line for `lineto`) back to (1,0)
    for (i, (x, y)) in [(6.0, 0.0), (5.0, 0.0), (0.0, 0.0), (4.0, 0.0), (1.0, 0.0)].into_iter().enumerate() {
        assert!(near(p[i], x, y), "{p:?}");
    }
    assert!(matches!(s[2], Segment::MoveTo(_)), "{s:?}");
    let z = &paths(&clean(r"\draw[decorate, decoration={zigzag, reverse path}] (0,0) -- (3,0);"))[0];
    assert!(near(ends(z)[0], 3.0, 0.0), "{z:?}");
}

/// Decorations along long curves (pgf walks them in coarser steps) still end at the end.
#[test]
fn long_curves_are_decorated_to_their_end() {
    for x in [10.0, 30.0] {
        let p =
            ends(&paths(&clean(&format!(r"\draw[decorate, decoration=zigzag] (0,0) .. controls ({},{}) and ({},{}) .. ({},0);", x / 3.0, x / 3.0, 2.0 * x / 3.0, x / 3.0, x)))[0]);
        assert!(near(*p.last().unwrap(), x, 0.0), "{x}: {:?}", p.last());
        assert!(p.len() > 20, "{}", p.len());
    }
}

// ---------------------------------------------------------------- the basic layer

/// `\pgfdecoration` with an unknown decoration leaves that stretch out; `\endpgfdecoration`
/// alone and an empty path are reported; `\pgfdecoratecurrentpath` decorates the path so far.
#[test]
fn basic_layer_commands() {
    let (items, w) = picture(r"\begin{pgfdecoration}{{nothing}{1cm}}\pgfpathmoveto{\pgfpointorigin}\pgfpathlineto{\pgfpoint{3cm}{0cm}}\end{pgfdecoration}\pgfusepath{stroke}");
    assert!(w.iter().any(|w| w.contains("decoration 'nothing' not supported: left out")), "{w:?}");
    assert!(paths(&items).is_empty());
    let (_, w) = picture(r"\endpgfdecoration");
    assert!(w.iter().any(|w| w.contains("\\endpgfdecoration without \\pgfdecoration")), "{w:?}");
    warned(r"\begin{pgfdecoration}{{zigzag}{1cm}}\end{pgfdecoration}", "I cannot decorate an empty path");
    warned(
        r"\pgfmetadecoration{no such meta}\pgfpathmoveto{\pgfpointorigin}\pgfpathlineto{\pgfpoint{1cm}{0cm}}\endpgfmetadecoration\pgfusepath{stroke}",
        "meta-decoration 'no such meta' not supported",
    );
    let p = ends(&paths(&clean(r"\pgfpathmoveto{\pgfpointorigin}\pgfpathlineto{\pgfpoint{3cm}{0cm}}\pgfdecoratecurrentpath{zigzag}\pgfusepath{stroke}"))[0]);
    assert!(p.iter().any(|q| q.y > 2.0 * PT) && near(*p.last().unwrap(), 3.0, 0.0), "{p:?}");
    // a closepath decorated: the path goes on from where it closed
    let p = ends(
        &paths(&clean(
            r"\pgfpathmoveto{\pgfpointorigin}\pgfpathlineto{\pgfpoint{1cm}{0cm}}\pgfpathclose\pgfdecoratecurrentpath{lineto}\pgfpathlineto{\pgfpoint{0cm}{2cm}}\pgfusepath{stroke}",
        ))[0],
    );
    assert!(near(p[2], 0.0, 0.0) && near(p[3], 0.0, 2.0), "{p:?}");
}

/// A lone point decorated is a tiny line; rounded corners are made before decorating; an
/// unknown soft clip path is reported.
#[test]
fn lone_points_rounded_corners_and_soft_clip() {
    let p = ends(&paths(&clean(r"\draw[decorate, decoration=zigzag] (1,1);"))[0]);
    assert!(near(p[0], 1.0, 1.0) && (p[1].x - 1.0) < 1e-3, "{p:?}");
    // the corner at (2,0) is rounded: no point of the zigzag is beyond x = 2 + amplitude near y = 0
    let s = &paths(&clean(r"\draw[rounded corners=5mm, decorate, decoration=lineto] (0,0) -- (2,0) -- (2,2);"))[0];
    assert!(s.iter().any(|x| matches!(x, Segment::CurveTo(..))), "{s:?}");
    let s = &paths(&warned(r"\draw[decorate, decoration={soft clip, soft clip path={domain=0:1}}] (0,0) -- (2,0);", "soft clip path 'domain=0:1' not understood"))[0];
    assert!(near(ends(s)[1], 2.0, 0.0));
}

/// `path picture`: the fill, then the picture clipped to the path with the node `path picture
/// bounding box` around it, then the stroke.
#[test]
fn path_picture() {
    let items = clean(r"\draw[path picture={\fill[red] (path picture bounding box.center) circle (2mm);}, fill=blue] (0,0) rectangle (2,1);");
    let kinds: Vec<&str> = items
        .iter()
        .map(|i| match i {
            Item::Path { fill: Some(_), stroke: None, .. } => "fill",
            Item::Path { stroke: Some(_), fill: None, .. } => "stroke",
            Item::Clip { .. } => "clip",
            _ => "other",
        })
        .collect();
    assert_eq!(kinds.first(), Some(&"fill"));
    assert_eq!(kinds.last(), Some(&"stroke"));
    assert!(kinds.contains(&"clip"), "{kinds:?}");
    // the dot is at the center of the rectangle
    let dot = paths(&items).into_iter().find(|s| s.iter().any(|x| matches!(x, Segment::CurveTo(..)))).unwrap();
    let (x0, x1, y0, y1) = extent(&ends(&dot)[1..]);
    assert!(((x0 + x1) / 2.0 - 1.0).abs() < 1e-6 && ((y0 + y1) / 2.0 - 0.5).abs() < 1e-6, "{dot:?}");
    // only a picture: the bounding box node is the circle's
    let items = clean(r"\path[path picture={\draw (path picture bounding box.south west) -- (path picture bounding box.north east);}] (0,0) circle (1);");
    let p = paths(&items);
    assert_eq!(p.len(), 1);
    assert!(near(ends(&p[0])[0], -1.0, -1.0) && near(ends(&p[0])[1], 1.0, 1.0), "{p:?}");
}

// ---------------------------------------------------------------- declared decorations

/// A declared decoration with `repeat state` and `switch if less than`: each state is used as
/// often as it says.
#[test]
fn declared_decoration_repeats_and_switches() {
    let p = ends(
        &paths(&clean(
            r"\pgfdeclaredecoration{steps}{up}{
                \state{up}[width=5pt, next state=down, repeat state=1]{\pgfpathlineto{\pgfpoint{0pt}{3pt}}}
                \state{down}[switch if less than=10pt to final, width=5pt, next state=up]{\pgfpathlineto{\pgfpoint{0pt}{-3pt}}}
                \state{final}{\pgfpathlineto{\pgfpointdecoratedpathlast}}}
              \draw[decorate, decoration=steps] (0,0) -- (40pt,0);",
        ))[0],
    );
    let ys: Vec<f64> = p.iter().map(|q| (q.y / PT).round()).collect();
    assert_eq!(ys, vec![0.0, 3.0, 3.0, -3.0, 3.0, 3.0, -3.0, 3.0, 3.0, 0.0], "{p:?}");
    let xs: Vec<f64> = p.iter().map(|q| (q.x / PT).round()).collect();
    assert_eq!(xs, vec![0.0, 0.0, 5.0, 10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 40.0], "{p:?}");
}

/// The persistent pre- and postcomputations run before and after each use of the state.
#[test]
fn declared_decoration_persistent_computations() {
    let p = ends(
        &paths(&clean(
            r"\newcount\n
              \pgfdeclaredecoration{stairs}{step}{
                \state{step}[width=10pt, persistent precomputation={\global\advance\n by 1\relax}, persistent postcomputation={\global\advance\n by 10\relax}]
                  {\pgfpathlineto{\pgfpoint{0pt}{\the\n pt}}}
                \state{final}{\pgfpathlineto{\pgfpointdecoratedpathlast}}}
              \draw[decorate, decoration=stairs] (0,0) -- (30pt,0);",
        ))[0],
    );
    let ys: Vec<f64> = p.iter().map(|q| (q.y / PT).round()).collect();
    assert_eq!(ys, vec![0.0, 1.0, 12.0, 23.0, 0.0], "{p:?}");
}

/// `switch if input segment less than`, `auto corner on length`, `auto end on length` and `if
/// input segment is closepath`, on a path with corners and a closed one.
#[test]
fn declared_decoration_input_segments_and_auto_options() {
    // dashes that stop at each corner: the last piece of a segment goes straight to its end
    let p = ends(
        &paths(&clean(
            r"\pgfdeclaredecoration{dashes}{dash}{
                \state{dash}[switch if input segment less than=4pt to corner, width=4pt]{\pgfpathlineto{\pgfpoint{2pt}{2pt}}\pgfpathlineto{\pgfpoint{4pt}{0pt}}}
                \state{corner}[width=\pgfdecoratedinputsegmentremainingdistance, next state=dash]{\pgfpathlineto{\pgfpointdecoratedinputsegmentlast}}
                \state{final}{\pgfpathlineto{\pgfpointdecoratedpathlast}}}
              \draw[decorate, decoration=dashes] (0,0) -- (10pt,0) -- (10pt,10pt);",
        ))[0],
    );
    // the corner is a point of the path
    assert!(p.iter().any(|q| near(*q, 10.0 * PT, 0.0)), "{p:?}");
    assert!(near(*p.last().unwrap(), 10.0 * PT, 10.0 * PT), "{p:?}");
    // auto corner and auto end: a line to each corner and to the end
    let body = |corners: &str| {
        format!(
            r"\pgfdeclaredecoration{{hops}}{{hop}}{{
                \state{{hop}}[auto end on length=6pt, auto corner on length=6pt, width=6pt]{{\pgfpathlineto{{\pgfpoint{{3pt}}{{3pt}}}}\pgfpathlineto{{\pgfpoint{{6pt}}{{0pt}}}}}}
                \state{{final}}{{}}}}
              \draw[decorate, decoration={{hops{corners}}}] (0,0) -- (20pt,0) -- (20pt,20pt);"
        )
    };
    let p = ends(&paths(&clean(&body(", path has corners")))[0]);
    assert!(p.iter().any(|q| near(*q, 20.0 * PT, 0.0)), "{p:?}");
    assert!(near(*p.last().unwrap(), 20.0 * PT, 20.0 * PT), "{p:?}");
    // a closed path: the options of `if input segment is closepath` hold on the closing segment
    let p = ends(
        &paths(&clean(
            r"\pgfdeclaredecoration{marks}{mark}{
                \state{mark}[width=5pt, if input segment is closepath={width=1pt}]{\pgfpathlineto{\pgfpoint{0pt}{1pt}}\pgfpathmoveto{\pgfpoint{0pt}{0pt}}}
                \state{final}{}}
              \draw[decorate, decoration=marks] (0,0) -- (10pt,0) -- (10pt,10pt) -- cycle;",
        ))[0],
    );
    // 2 + 2 marks on the lines, then the closing segment (about 14pt) every 1pt
    let marks = p.windows(2).filter(|w| (w[1].minus(w[0]).norm() - PT).abs() < 1e-6).count();
    assert!(marks > 14, "{marks}: {p:?}");
}

/// Options a state does not know, lengths that are not lengths, and a state that never ends.
#[test]
fn declared_decoration_problems() {
    warned(
        r"\pgfdeclaredecoration{odd}{a}{\state{a}[width=5pt, wobble=3]{\pgfpathlineto{\pgfpoint{0pt}{1pt}}}\state{final}{}}
          \draw[decorate, decoration=odd] (0,0) -- (1,0);",
        "decoration state option 'wobble' not supported",
    );
    warned(
        r"\pgfdeclaredecoration{odd}{a}{\state{a}[width=5pt, switch if less than=wide to final]{\pgfpathlineto{\pgfpoint{0pt}{1pt}}}\state{final}{}}
          \draw[decorate, decoration=odd] (0,0) -- (1,0);",
        "decoration state option 'switch if less than'",
    );
    warned(
        r"\pgfdeclaredecoration{still}{a}{\state{a}[width=0pt]{}}
          \draw[decorate, decoration=still] (0,0) -- (1,0);",
        "too many steps",
    );
}

/// `\pgfsetdecorationsegmenttransformation` and the segment length and amplitude registers set in
/// the code before a decoration of `\pgfdecoration`.
#[test]
fn code_before_a_decoration_sets_its_transformation_and_sizes() {
    let p = ends(
        &paths(&clean(
            r"\begin{pgfdecoration}{{zigzag}{2cm}{\pgfsetdecorationsegmenttransformation{\pgftransformyshift{1cm}}\pgfdecorationsegmentamplitude=5pt}}
              \pgfpathmoveto{\pgfpointorigin}\pgfpathlineto{\pgfpoint{2cm}{0cm}}\end{pgfdecoration}\pgfusepath{stroke}",
        ))[0],
    );
    let (.., y0, y1) = extent(&p[1..p.len() - 1]);
    assert!((y0 - (1.0 - 5.0 * PT)).abs() < 1e-4 && (y1 - (1.0 + 5.0 * PT)).abs() < 1e-4, "{p:?}");
}

// ---------------------------------------------------------------- declared meta-decorations

/// A declared meta-decoration: each state names its decoration and the code before and after
/// it; `\pgfmetadecoratedpathlength` and the widths split the path.
#[test]
fn declared_meta_decoration() {
    let body = r"\pgfdeclaremetadecoration{thirds}{first}{
        \state{first}[width=\pgfmetadecoratedpathlength/3, next state=second]{\decoration{zigzag}}
        \state{second}[width=\pgfmetadecoratedpathlength/3, next state=final]
          {\decoration{lineto}\beforedecoration{\pgfpathlineto{\pgfpoint{0pt}{10pt}}}\afterdecoration{\pgfpathlineto{\pgfpoint{0pt}{-10pt}}}}
        \state{final}{\decoration{moveto}}}
      \draw[decorate, decoration=thirds] (0,0) -- (3,0);";
    let p = ends(&paths(&clean(body))[0]);
    // the zigzag in the first third only
    let zig: Vec<&Point> = p.iter().filter(|q| q.y.abs() > 1e-6 && q.y.abs() < 5.0 * PT).collect();
    assert!(!zig.is_empty() && zig.iter().all(|q| q.x <= 1.0 + 1e-4), "{p:?}");
    // the code before and after the second third: in the page's coordinates, at the origin
    assert!(p.iter().any(|q| near(*q, 0.0, 10.0 * PT)) && p.iter().any(|q| near(*q, 0.0, -10.0 * PT)), "{p:?}");
    // the last third is moved over: the path ends with a moveto to the end
    let s = &paths(&clean(body))[0];
    assert!(matches!(s.last(), Some(Segment::MoveTo(q)) if near(*q, 3.0, 0.0)), "{s:?}");
}

/// The switches of a meta state, an unknown decoration, no decoration at all, an unknown option,
/// and a meta state that sets the decoration's keys.
#[test]
fn declared_meta_decoration_switches_and_problems() {
    // `switch if less than`: the path is shorter than 5cm, so the first state goes to `short`
    let p = ends(
        &paths(&clean(
            r"\pgfdeclaremetadecoration{choose}{start}{
                \state{start}[switch if less than=5cm to short, width=1cm, next state=final]{\decoration{zigzag}}
                \state{short}[switch if input segment less than=5cm to final, width=1cm]{\decoration{lineto}}
                \state{final}{\decoration{lineto}}}
              \draw[decorate, decoration=choose] (0,0) -- (3,0);",
        ))[0],
    );
    assert!(p.iter().all(|q| q.y.abs() < 1e-6), "no zigzag: {p:?}");
    assert!(near(*p.last().unwrap(), 3.0, 0.0), "{p:?}");
    // a state's code sets the amplitude for its decoration
    let p = ends(
        &paths(&clean(
            r"\pgfdeclaremetadecoration{big}{one}{
                \state{one}[width=2cm, next state=final]{\pgfset{/pgf/decoration/amplitude=8pt}\decoration{zigzag}}
                \state{final}{\decoration{lineto}}}
              \draw[decorate, decoration=big] (0,0) -- (3,0);",
        ))[0],
    );
    let top = p.iter().map(|q| q.y).fold(f64::MIN, f64::max);
    assert!((top - 8.0 * PT).abs() < 1e-4, "{top}");
    let (items, w) = picture(
        r"\pgfdeclaremetadecoration{bad}{one}{
            \state{one}[width=1cm, wobble=2, next state=two]{\decoration{nothing}}
            \state{two}[width=1cm, next state=final]{}
            \state{final}{\decoration{lineto}}}
          \draw[decorate, decoration=bad] (0,0) -- (3,0);",
    );
    assert!(w.iter().any(|w| w.contains("meta-decoration state option 'wobble' not supported")), "{w:?}");
    assert!(w.iter().any(|w| w.contains("decoration 'nothing' not supported: moved over")), "{w:?}");
    // the first two centimeters are moved over: the only line is the last centimeter
    let s = &paths(&items)[0];
    let lines: Vec<usize> = (0..s.len()).filter(|&i| matches!(s[i], Segment::LineTo(_))).collect();
    assert_eq!(lines.len(), 1, "{s:?}");
    let i = lines[0];
    assert!(matches!((s[i - 1], s[i]), (Segment::MoveTo(a), Segment::LineTo(b)) if near(a, 2.0, 0.0) && near(b, 3.0, 0.0)), "{s:?}");
}

// ---------------------------------------------------------------- more edge cases

/// A key with no value keeps what it had; an empty text of text effects draws nothing; a shape
/// with no background path draws no shapes; a path of movetos only has nothing to decorate.
#[test]
fn keys_without_values_and_empty_things() {
    let a = paths(&clean(r"\draw[decorate, decoration={zigzag, amplitude=4pt}] (0,0) -- (1,0);"));
    let b = paths(&clean(r"\draw[decorate, decoration={zigzag, amplitude=4pt, amplitude}] (0,0) -- (1,0);"));
    assert_eq!(format!("{a:?}"), format!("{b:?}"));
    assert!(clean(r"\path[decoration={text effects along path, text={}}, decorate] (0,0) -- (6,0);").is_empty());
    assert!(paths(&clean(r"\fill[decorate, decoration={shape backgrounds, shape=coordinate}] (0,0) -- (2,0);")).is_empty());
    assert!(paths(&clean(r"\draw[decorate, decoration=zigzag] (0,0) (1,1);")).is_empty());
}

/// pgfplots' `soft clip` with a rectangle: the path cut to it, starting where the path started.
#[test]
fn soft_clip_to_a_rectangle() {
    let s = &paths(&clean(r"\draw[decorate, decoration={soft clip, soft clip path={(1,-1) rectangle (2,1)}}] (0,0) -- (3,0);"))[0];
    assert!(matches!(s[0], Segment::MoveTo(q) if near(q, 0.0, 0.0)), "{s:?}");
    let p = ends(&s[1..]);
    assert!(near(p[0], 1.0, 0.0) && near(p[1], 2.0, 0.0) && p.len() == 2, "{s:?}");
}

/// Endless decorations stop with a warning: a meta-segment of zero length, a meta state that
/// switches to itself, a segment transformation whose macro never ends.
#[test]
fn endless_decorations_stop() {
    warned(r"\draw[decorate, decoration={straight zigzag, meta-segment length=0pt}] (0,0) -- (2,0);", "decoration: too many steps, stopped");
    warned(
        r"\pgfdeclaremetadecoration{loop}{a}{\state{a}[switch if less than=10cm to a]{}} \draw[decorate, decoration=loop] (0,0) -- (2,0);",
        "meta-decoration: too many steps, stopped",
    );
    let items = warned(
        r"\begin{pgfdecoration}{{zigzag}{2cm}{\pgfsetdecorationsegmenttransformation{\def\x{\x}\x}}}\pgfpathmoveto{\pgfpointorigin}\pgfpathlineto{\pgfpoint{2cm}{0cm}}\end{pgfdecoration}\pgfusepath{stroke}",
        "decoration segment transformation",
    );
    // the zigzag is drawn without it
    let p = ends(&paths(&items)[0]);
    assert!(near(p[1], 2.5 * PT, 2.5 * PT), "{p:?}");
}

/// `\pgfdecoratecurrentpath` with a decoration that closes its pieces: the path goes on from the
/// last piece.
#[test]
fn decorating_the_current_path_with_closed_pieces() {
    let s = &paths(&clean(
        r"\pgfpathmoveto{\pgfpointorigin}\pgfpathlineto{\pgfpoint{3cm}{0cm}}\pgfdecoratecurrentpath{triangles}\pgfpathlineto{\pgfpoint{3cm}{1cm}}\pgfusepath{stroke}",
    ))[0];
    assert!(closes(s) > 5, "{s:?}");
    assert!(matches!(s.last(), Some(Segment::LineTo(q)) if near(*q, 3.0, 1.0)), "{s:?}");
}

/// Text along a path: a group inside a group is one box, a macro that expands to itself ends the
/// text, a math command that cannot be set is reported.
#[test]
fn text_along_path_nested_groups_and_loops() {
    let items = clean(r"\path[decorate, decoration={text along path, text={a{b{c}d}e}}] (0,0) -- (6,0);");
    assert_eq!(text_places(&items).len(), 3);
    let items = clean(r"\def\again{\again}\path[decorate, decoration={text along path, text={b\again}}] (0,0) -- (6,0);");
    assert_eq!(text_places(&items).len(), 1);
    let (_, w) = picture(r"\path[decorate, decoration={text along path, text={$\qqq$b}}] (0,0) -- (6,0);");
    assert!(w.iter().any(|w| w.contains("qqq")), "{w:?}");
}

/// `shape scaled` with the shapes a distance apart from center to center: they grow from the
/// start size to the end size along the path.
#[test]
fn scaled_shapes_between_centers() {
    let b = boxes(r"\fill[decorate, decoration={shape backgrounds, shape=rectangle, shape start size=2mm, shape end size=6mm, shape scaled, shape sep=1cm}] (0,0) -- (4,0);");
    assert_eq!(b.len(), 5, "{b:?}");
    for (i, x) in b.iter().enumerate() {
        let want = 0.2 + 0.4 * i as f64 / 4.0;
        assert!((x.1 - x.0 - want).abs() < 1e-3, "{b:?}");
        assert!(((x.0 + x.1) / 2.0 - i as f64).abs() < 1e-3, "{b:?}");
    }
}

/// A `name path` in a postaction names the path the postaction uses.
#[test]
fn name_path_in_a_postaction() {
    let items = clean(
        r"\usetikzlibrary{intersections}\path[postaction={name path=a}] (0,0) -- (2,2); \path[name path=b] (0,2) -- (2,0); \fill[name intersections={of=a and b}] (intersection-1) circle (1pt);",
    );
    let dot = &paths(&items)[0];
    let (x0, x1, y0, y1) = extent(&ends(dot));
    assert!(((x0 + x1) / 2.0 - 1.0).abs() < 1e-4 && ((y0 + y1) / 2.0 - 1.0).abs() < 1e-4, "{dot:?}");
}

/// A state that has already switched does not end the path with `auto end on length`.
#[test]
fn auto_end_after_a_switch() {
    let p = ends(
        &paths(&clean(
            r"\pgfdeclaredecoration{early}{a}{
                \state{a}[switch if less than=1cm to final, auto end on length=5mm, width=5mm]{\pgfpathlineto{\pgfpoint{0pt}{2pt}}}
                \state{final}{}}
              \draw[decorate, decoration=early] (0,0) -- (3,0);",
        ))[0],
    );
    // marks at 0, 5mm, …, 2cm; nothing from 2.5cm on
    let last = p.last().unwrap();
    assert!(near(*last, 2.0, 2.0 * PT), "{p:?}");
}

/// `reverse path` of a closed path that starts with a curve, and of a path that ends with one.
#[test]
fn reverse_path_with_curves_at_the_ends() {
    let p = ends(&paths(&clean(r"\draw[decorate, decoration={lineto, reverse path}] (0,0) .. controls (1,1) and (2,1) .. (3,0) -- cycle;"))[0]);
    assert!(near(p[0], 0.0, 0.0) && near(p[1], 3.0, 0.0) && near(p[2], 0.0, 0.0), "{p:?}");
    let p = ends(&paths(&clean(r"\draw[decorate, decoration={lineto, reverse path}] (0,0) -- (1,0) .. controls (2,1) and (3,1) .. (4,0);"))[0]);
    assert!(near(p[0], 4.0, 0.0) && near(p[1], 1.0, 0.0), "{p:?}");
}

/// A curve longer than 2048pt is walked in the finest steps and still decorated to its end.
#[test]
fn a_very_long_curve() {
    let p = ends(&paths(&clean(r"\draw[decorate, decoration={zigzag, segment length=1cm}] (0,0) .. controls (30,30) and (60,30) .. (90,0);"))[0]);
    assert!(near(*p.last().unwrap(), 90.0, 0.0), "{:?}", p.last());
}

/// Characters of text effects whose node is a coordinate: no text and no room taken.
#[test]
fn text_effects_characters_as_coordinates() {
    let items = clean(r"\path[decoration={text effects along path, text={abc}, text effects/every character/.style={shape=coordinate}}, decorate] (0,0) -- (6,0);");
    assert!(text_places(&items).is_empty() && paths(&items).is_empty(), "{items:?}");
}

/// A text that never ends (a macro that expands to a letter and itself) is measured up to a
/// limit and set as far as the path goes.
#[test]
fn text_along_path_that_never_ends() {
    let items = clean(r"\def\more{x\more}\path[decorate, decoration={text along path, text={\more}}] (0,0) -- (3,0);");
    let places = text_places(&items);
    assert!(places.len() > 10, "{places:?}");
    assert!(places.iter().all(|p| p.0 < 3.0), "{places:?}");
}

/// A tight loop decorated in tiny steps: a step can run past the end of the curve, and the
/// decoration still ends at the end of the path.
#[test]
fn tiny_steps_on_a_tight_loop() {
    let p = ends(&paths(&clean(r"\draw[decorate, decoration={zigzag, segment length=0.7pt}] (0,0) .. controls (1,1) and (-1,1) .. (0.1,0);"))[0]);
    assert!(near(*p.last().unwrap(), 0.1, 0.0), "{:?}", p.last());
    assert!(p.len() > 50, "{}", p.len());
}

/// A segment of no length in the path is walked over: the zigzag goes on after it.
#[test]
fn a_segment_of_no_length() {
    let p = ends(&paths(&clean(r"\draw[decorate, decoration=zigzag] (0,0) -- (1,0) -- (1,0) -- (2,0);"))[0]);
    assert!(near(*p.last().unwrap(), 2.0, 0.0), "{p:?}");
    assert!(p.iter().any(|q| q.x > 1.2 && q.y.abs() > 1e-6), "{p:?}");
}

/// A path picture on a path with no points has nothing to clip to: the picture is left out.
#[test]
fn a_path_picture_on_an_empty_path() {
    let items = clean(r"\path[path picture={\fill (0,0) circle (1pt);}];");
    assert!(paths(&items).is_empty(), "{items:?}");
}

/// An arrow tip drawn by a mark counts in the picture's size.
#[test]
fn an_arrow_in_a_mark_counts_in_the_size() {
    let (c, w) =
        run_document(r"\begin{tikzpicture}\draw[postaction={decorate}, decoration={markings, mark=at position 0.5 with {\arrow[scale=10]{>}}}] (0,0) -- (2,0);\end{tikzpicture}")
            .unwrap();
    assert!(w.is_empty(), "{w:?}");
    let (lo, hi) = (c.min.unwrap(), c.max.unwrap());
    // the line alone is 0.4pt high; the big tip much more
    assert!(hi.y - lo.y > 0.2, "{lo:?} {hi:?}");
}

/// Text effects: a group inside a group is one character, and a separator first in the text
/// ends no word.
#[test]
fn text_effects_nested_groups_and_leading_separators() {
    let items = clean(r"\path[decoration={text effects along path, text={a{b{c}d}e}}, decorate] (0,0) -- (6,0);");
    assert_eq!(text_places(&items).len(), 3);
    let items = clean(r"\path[decoration={text effects along path, text={-ab}, text effects/.cd, word separator=-, every letter/.style={fill=blue}}, decorate] (0,0) -- (6,0);");
    assert_eq!(text_places(&items).len(), 3);
    assert_eq!(items.iter().filter(|i| matches!(i, Item::Path { fill: Some(_), .. })).count(), 2);
}
