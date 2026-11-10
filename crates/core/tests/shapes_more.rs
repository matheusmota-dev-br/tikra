//! More node shapes and their options: the split shapes (circle solidus, ellipse and diamond
//! split, horizontal rectangle splits), `shape border uses incircle`, minimum sizes and the
//! stretching keys of the geometric shapes, the symbols (starburst, signal, tape, magnetic tape,
//! cloud), rounded and chamfered rectangles, callouts, logic gates, arrow shapes and the
//! electrical ones.

mod common;

use common::{canvas, close, paths, point};
use tikra::canvas::Item;
use tikra::scene::{Point, Segment};
use tikra::PT;

/// A node `s` with these options and text, after `libs`.
fn node(libs: &str, opts: &str, text: &str) -> String {
    let libs = if libs.is_empty() { String::new() } else { format!("\\usetikzlibrary{{{libs}}}") };
    format!("{libs}\\node[{opts}] (s) {{{text}}};")
}

/// An anchor of the node `s`, in cm.
fn at(body: &str, anchor: &str) -> Point {
    point(body, &format!("s.{anchor}"))
}

/// Width (east to west) and height (north to south) of the node `s`, in cm.
fn extent(body: &str) -> (f64, f64) {
    (at(body, "east").x - at(body, "west").x, at(body, "north").y - at(body, "south").y)
}

/// The picture's warnings.
fn warnings(body: &str) -> Vec<String> {
    tikra::engine::run_document(&format!("\\begin{{tikzpicture}}{body}\\end{{tikzpicture}}")).unwrap().1
}

fn clean(body: &str) {
    let w = warnings(body);
    assert!(w.is_empty(), "{w:?}\n{body}");
}

/// The straight lines (moveto then lineto) of the drawn paths.
fn lines(body: &str) -> Vec<(Point, Point)> {
    let mut out = Vec::new();
    for d in paths(&canvas(body)) {
        for w in d.segments.windows(2) {
            if let [Segment::MoveTo(a), Segment::LineTo(b)] = w {
                out.push((*a, *b));
            }
        }
    }
    out
}

fn near(p: Point, x: f64, y: f64) -> bool {
    (p.x - x).abs() < 1e-4 && (p.y - y).abs() < 1e-4
}

// ---------------------------------------------------------------- split shapes

/// The circle solidus puts the lower part below and right of the upper one, with a slash
/// between them; the ellipse and diamond split put it below a horizontal line.
#[test]
fn circle_solidus_ellipse_and_diamond_split() {
    let body = node("shapes.multipart", "circle solidus, draw", r"1\nodepart{lower}2");
    clean(&body);
    let (t, l) = (at(&body, "text"), at(&body, "lower"));
    assert!(l.x > t.x && l.y < t.y, "{t:?} {l:?}");
    let slash = lines(&body).into_iter().find(|(a, b)| (b.x - a.x).abs() > 1e-6 && close(b.x - a.x, b.y - a.y)).expect("a slash at 45 degrees");
    assert!(slash.1.x > slash.0.x, "{slash:?}");
    // round: as high as wide
    let (w, h) = extent(&body);
    assert!(close(w, h), "{w} {h}");
    for shape in ["ellipse split", "diamond split"] {
        let body = node("shapes.multipart", &format!("{shape}, draw"), r"upper\nodepart{lower}down");
        clean(&body);
        let (t, l, c) = (at(&body, "text"), at(&body, "lower"), at(&body, "center"));
        assert!(l.y < c.y && c.y < t.y + 0.3, "{shape}: {t:?} {l:?} {c:?}");
        // the line across, through the center, from the west to the east of the shape
        let across = lines(&body).into_iter().find(|(a, b)| close(a.y, b.y) && close(a.y, c.y)).expect("the line across");
        assert!(across.0.x < c.x && across.1.x > c.x, "{shape}: {across:?}");
        let (w, h) = extent(&body);
        assert!(w > 0.5 && h > 0.5, "{shape}: {w} {h}");
    }
    // a diamond split as wide as the minimum width, with a flat aspect
    let body = node("shapes.multipart", "diamond split, draw, minimum width=4cm, aspect=0", r"a\nodepart{lower}b");
    let (w, _) = extent(&body);
    assert!((w - 4.0).abs() < 0.05, "{w}");
}

/// Horizontal rectangle splits: parts side by side, aligned as asked, filled part by part,
/// empty parts left out.
#[test]
fn horizontal_rectangle_splits() {
    let opts = "rectangle split, rectangle split horizontal, rectangle split parts=4, draw, rectangle split part align={top, bottom, base, center}";
    let body = node("shapes.multipart", opts, r"a\nodepart{two}Ag\nodepart{three}y\nodepart{four}X");
    clean(&body);
    let xs: Vec<f64> = ["one", "two", "three", "four"].iter().map(|p| at(&body, p).x).collect();
    assert!(xs.windows(2).all(|w| w[0] < w[1]), "{xs:?}");
    // a split between the first two parts, from the bottom to the top
    let (n, s) = (at(&body, "one split north"), at(&body, "one split south"));
    assert!(close(n.x, s.x) && n.y > s.y && n.x > xs[0] && n.x < xs[1], "{n:?} {s:?}");
    assert!(close(at(&body, "two north").y, at(&body, "north").y));
    // top: the part's top at the top; bottom: its depth at the bottom
    let (top, bottom) = (at(&body, "one"), at(&body, "two"));
    assert!(top.y > bottom.y, "{top:?} {bottom:?}");
    // three splits drawn: vertical lines
    let vertical = lines(&body).into_iter().filter(|(a, b)| close(a.x, b.x) && (a.y - b.y).abs() > 0.1).count();
    assert!(vertical >= 3, "{vertical}");
    // fills: the first and the last part
    let filled = node(
        "shapes.multipart",
        "rectangle split, rectangle split horizontal, rectangle split parts=3, draw, rectangle split part fill={red, none, blue}",
        r"a\nodepart{two}b\nodepart{three}c",
    );
    let fills = canvas(&filled).items.iter().filter(|i| matches!(i, Item::Path { fill: Some(_), stroke: None, .. })).count();
    assert_eq!(fills, 2);
    // an empty middle part left out: one split line, and the third part right after the first
    let skip = node(
        "shapes.multipart",
        "rectangle split, rectangle split horizontal, rectangle split parts=3, draw, rectangle split ignore empty parts",
        r"a\nodepart{two}\nodepart{three}c",
    );
    clean(&skip);
    let vertical = lines(&skip).into_iter().filter(|(a, b)| close(a.x, b.x) && (a.y - b.y).abs() > 0.1).count();
    assert_eq!(vertical, 1);
    let full = node("shapes.multipart", "rectangle split, rectangle split horizontal, rectangle split parts=3, draw", r"a\nodepart{two}\nodepart{three}c");
    assert!(extent(&skip).0 < extent(&full).0);
    // vertical, with an empty middle part left out too: the bottom of the outline and one split
    let skip = node("shapes.multipart", "rectangle split, rectangle split parts=3, draw, rectangle split ignore empty parts", r"a\nodepart{two}\nodepart{three}c");
    let horizontal = lines(&skip).into_iter().filter(|(a, b)| close(a.y, b.y) && (a.x - b.x).abs() > 0.1).count();
    assert_eq!(horizontal, 2);
}

/// `rectangle split draw splits=false`: the outline alone, no line between the parts, though the
/// parts stay where they are.
#[test]
fn rectangle_split_without_its_split_lines() {
    let with = node("shapes.multipart", "rectangle split, rectangle split parts=2, draw", r"a\nodepart{two}b");
    let without = node("shapes.multipart", "rectangle split, rectangle split parts=2, draw, rectangle split draw splits=false", r"a\nodepart{two}b");
    clean(&without);
    let horizontal = |body: &str| lines(body).into_iter().filter(|(a, b)| close(a.y, b.y) && (a.x - b.x).abs() > 0.1).count();
    // the split, and the outline's sides (moveto then lineto: its bottom)
    assert_eq!(horizontal(&with), horizontal(&without) + 1);
    assert_eq!(paths(&canvas(&without)).len(), 1);
    assert!(close(at(&with, "two").y, at(&without, "two").y));
}

// ---------------------------------------------------------------- geometric shapes

/// An empty node with an inner sep of 3mm by 1mm and no outer sep.
const BOX: &str = "draw, inner xsep=3mm, inner ysep=1mm, outer sep=0pt";

/// `shape border uses incircle`: the shape is built around the circle through the corners of the
/// text box, so it is larger than without, and as large across as high where it is symmetric.
#[test]
fn shapes_around_the_incircle() {
    for shape in ["trapezium", "semicircle", "isosceles triangle", "kite", "dart", "circular sector", "cylinder", "starburst", "single arrow", "double arrow"] {
        let plain = node("shapes.geometric, shapes.symbols, shapes.arrows", &format!("{shape}, {BOX}"), "");
        let round = node("shapes.geometric, shapes.symbols, shapes.arrows", &format!("{shape}, {BOX}, shape border uses incircle"), "");
        clean(&round);
        let (pw, ph) = extent(&plain);
        let (rw, rh) = extent(&round);
        assert!(rh > ph + 1e-3, "{shape}: {pw} {ph} {rw} {rh}");
    }
    // a trapezium of right angles around the incircle: a square of side 2·√2·3mm
    let body = node("shapes.geometric", &format!("trapezium, trapezium angle=90, {BOX}, shape border uses incircle"), "");
    let (w, h) = extent(&body);
    let side = 2.0 * 2f64.sqrt() * 0.3;
    assert!((w - side).abs() < 1e-3 && (h - side).abs() < 1e-3, "{w} {h}");
}

/// A negative rotation of the border is turned into a positive one.
#[test]
fn negative_border_rotations() {
    // (pgf rounds the angle with integer arithmetic on its remainder: -180 is 270)
    let a = node("shapes.geometric", "isosceles triangle, draw, shape border rotate=-180", "x");
    let b = node("shapes.geometric", "isosceles triangle, draw, shape border rotate=270", "x");
    let (pa, pb) = (at(&a, "apex"), at(&b, "apex"));
    assert!(near(pa, pb.x, pb.y), "{pa:?} {pb:?}");
    assert!(pa.y < at(&a, "center").y, "the apex points down: {pa:?}");
    // negative trapezium angles are taken modulo 360
    let t = node("shapes.geometric", "trapezium, draw, trapezium left angle=-60, trapezium right angle=300", "x");
    clean(&t);
    let u = node("shapes.geometric", "trapezium, draw, trapezium left angle=300, trapezium right angle=300", "x");
    assert!(near(at(&t, "bottom left corner"), at(&u, "bottom left corner").x, at(&u, "bottom left corner").y));
}

/// Minimum sizes: the shapes grow to them; the trapezium and isosceles triangle stretch as their
/// keys say.
#[test]
fn minimum_sizes_of_the_geometric_shapes() {
    for (shape, w, h) in [
        ("trapezium", 5.0, 0.0),
        ("trapezium, trapezium stretches", 5.0, 0.0),
        ("trapezium, trapezium stretches body", 5.0, 0.0),
        ("kite", 0.0, 4.0),
        ("kite", 4.0, 0.0),
        ("dart", 0.0, 4.0),
        ("circular sector", 4.0, 0.0),
        ("circular sector", 0.0, 4.0),
        ("isosceles triangle, isosceles triangle stretches", 4.0, 0.0),
        ("isosceles triangle, isosceles triangle stretches", 0.0, 4.0),
        ("isosceles triangle", 0.0, 4.0),
        ("starburst", 6.0, 6.0),
        ("single arrow", 3.0, 0.0),
    ] {
        let body = node("shapes.geometric, shapes.symbols, shapes.arrows", &format!("{shape}, draw, outer sep=0pt, minimum width={w}cm, minimum height={h}cm"), "x");
        clean(&body);
        let small = node("shapes.geometric, shapes.symbols, shapes.arrows", &format!("{shape}, draw, outer sep=0pt"), "x");
        let (bw, bh) = extent(&body);
        let (sw, sh) = extent(&small);
        assert!(bw > sw + 0.5 || bh > sh + 0.5, "{shape} {w} {h}: {bw} {bh} vs {sw} {sh}");
    }
    // stretched or not, a trapezium is as wide as its minimum width
    for keys in ["", ", trapezium stretches", ", trapezium stretches body"] {
        let body = node("shapes.geometric", &format!("trapezium{keys}, draw, outer sep=0pt, minimum width=5cm"), "x");
        let l = at(&body, "bottom left corner");
        let r = at(&body, "bottom right corner");
        assert!((r.x - l.x - 5.0).abs() < 1e-3, "{keys}: {l:?} {r:?}");
    }
    // the height of a trapezium scaled to its width grows, stretched ones keep it
    let scaled = extent(&node("shapes.geometric", "trapezium, draw, outer sep=0pt, minimum width=5cm", "x")).1;
    let stretched = extent(&node("shapes.geometric", "trapezium, trapezium stretches, draw, outer sep=0pt, minimum width=5cm", "x")).1;
    assert!(scaled > stretched + 0.1, "{scaled} {stretched}");
    // an isosceles triangle of an empty box: as long as its minimum height
    let body = node("shapes.geometric", "isosceles triangle, draw, inner sep=0pt, outer sep=0pt, minimum height=2cm", "");
    let (a, l) = (at(&body, "apex"), at(&body, "left corner"));
    assert!((a.x - l.x - 2.0).abs() < 1e-3, "{a:?} {l:?}");
}

/// An empty dart with no inner sep grows to its minimum height; an empty tape without bends keeps
/// the height of a bottom bend (pgf always counts it), and with a bend height of zero it is flat.
#[test]
fn empty_darts_and_flat_tapes() {
    let body = node("shapes.geometric", "dart, draw, inner sep=0pt, outer sep=0pt, minimum height=2cm", "");
    clean(&body);
    let (tip, tail) = (at(&body, "tip"), at(&body, "left tail"));
    assert!(tip.x - tail.x > 1.0, "{tip:?} {tail:?}");
    let body = node("shapes.symbols", "tape, draw, tape bend=none, inner sep=0pt, outer sep=0pt", "");
    clean(&body);
    let (n, s) = (at(&body, "north"), at(&body, "south"));
    assert!(close(n.y - s.y, 5.0 * PT), "{n:?} {s:?}");
    let body = node("shapes.symbols", "tape, draw, tape bend=none, tape bend height=0pt, inner sep=0pt, outer sep=0pt, minimum height=0pt, minimum width=1cm", "");
    clean(&body);
    let (n, s) = (at(&body, "north"), at(&body, "south"));
    assert!(close(n.y, s.y), "{n:?} {s:?}");
}

/// A cylinder with custom fills: its body and its end are filled with their colors.
#[test]
fn cylinder_custom_fills() {
    let body = node("shapes.geometric", "cylinder, draw, cylinder uses custom fill, cylinder body fill=red, cylinder end fill=blue", "x");
    clean(&body);
    let fills: Vec<_> =
        canvas(&body).items.iter().filter_map(|i| if let Item::Path { fill: Some(f), stroke: None, .. } = i { Some(format!("{:?}", f.paint)) } else { None }).collect();
    assert_eq!(fills.len(), 2, "{fills:?}");
    assert!(fills[0].contains("r: 1.0, g: 0.0, b: 0.0") && fills[1].contains("r: 0.0, g: 0.0, b: 1.0"), "{fills:?}");
}

// ---------------------------------------------------------------- symbols

/// A starburst turned a quarter is built around the text turned, then turned back: still wide,
/// but not the same shape.
#[test]
fn starburst_turned() {
    let plain = node("shapes.symbols", "starburst, draw, random starburst=0, outer sep=0pt", "wide text");
    let turned = node("shapes.symbols", "starburst, draw, random starburst=0, outer sep=0pt, shape border rotate=90", "wide text");
    clean(&turned);
    let (pw, ph) = extent(&plain);
    let (tw, th) = extent(&turned);
    assert!(pw > ph && tw > th && !close(pw, tw), "{pw} {ph} {tw} {th}");
}

/// A signal to a direction it does not know has no pointer.
#[test]
fn signal_to_an_unknown_direction() {
    let odd = node("shapes.symbols", "signal, draw, signal to=sideways", "x");
    let none = node("shapes.symbols", "signal, draw, signal to=nowhere", "x");
    clean(&odd);
    assert_eq!(format!("{:?}", paths(&canvas(&odd))[0].segments), format!("{:?}", paths(&canvas(&none))[0].segments));
}

/// A signal pointing south: the pointer at the bottom, the sides flat.
#[test]
fn signal_to_the_south() {
    let body = node("shapes.symbols", "signal, draw, signal to=south", "x");
    clean(&body);
    let (s, c) = (at(&body, "south"), at(&body, "center"));
    let plain = node("shapes.symbols", "signal, draw, signal to=nowhere", "x");
    assert!(c.y - s.y > at(&plain, "center").y - at(&plain, "south").y + 0.05, "{s:?} {c:?}");
}

/// Tape bends: in and out, out and in, or none (a straight side).
#[test]
fn tape_bends() {
    for (top, bottom) in [("in and out", "none"), ("none", "out and in"), ("none", "none"), ("out and in", "in and out")] {
        let body = node("shapes.symbols", &format!("tape, draw, tape bend top={top}, tape bend bottom={bottom}"), "x");
        clean(&body);
        let s = paths(&canvas(&body))[0].segments.clone();
        let curves = s.iter().filter(|x| matches!(x, Segment::CurveTo(..))).count();
        let bends = [top, bottom].iter().filter(|b| **b != "none").count();
        assert_eq!(curves > 0, bends > 0, "{top} / {bottom}: {s:?}");
    }
    // a straight top: the north corners at the top of the tape
    let body = node("shapes.symbols", "tape, draw, tape bend top=none, outer sep=2pt", "x");
    assert!(close(at(&body, "north east").y, at(&body, "north west").y));
    // a bent top: the outer sep is mitered differently at the two corners
    let body = node("shapes.symbols", "tape, draw, tape bend top=in and out, outer sep=2pt", "x");
    assert!(!close(at(&body, "north east").y, at(&body, "north west").y));
}

/// The magnetic tape: a circle with its tail at the bottom right, and the tail's anchors.
#[test]
fn magnetic_tape() {
    let body = node("shapes.symbols", "magnetic tape, draw, outer sep=0pt, magnetic tape tail extend=5mm, magnetic tape tail=0.2", "xy");
    clean(&body);
    let (c, n, s, e) = (at(&body, "center"), at(&body, "north"), at(&body, "south"), at(&body, "east"));
    let r = n.y - c.y;
    assert!(close(c.y - s.y, r) && close(e.x - c.x, r), "{c:?} {n:?} {s:?} {e:?}");
    let tail = at(&body, "tail south east");
    assert!(close(tail.x - c.x, r + 0.5) && close(tail.y, s.y), "{tail:?}");
    let top = at(&body, "tail north east");
    assert!(close(top.y - tail.y, 0.2 * r), "{top:?} {tail:?}");
    for a in ["tail east", "north east", "south west", "north west", "base east", "mid west"] {
        at(&body, a);
    }
}

/// A cloud that ignores the aspect follows the text; a tall text in a wide cloud makes it tall.
#[test]
fn clouds_and_their_aspect() {
    let free = node("shapes.symbols", "cloud, draw, cloud ignores aspect", "wide wide text");
    let kept = node("shapes.symbols", "cloud, draw, aspect=1", "wide wide text");
    clean(&free);
    let (fw, fh) = extent(&free);
    let (kw, kh) = extent(&kept);
    assert!(fw / fh > kw / kh + 0.5, "{fw} {fh} {kw} {kh}");
    let tall = node("shapes.symbols", "cloud, draw, aspect=4, text width=2mm", r"a\\b\\c\\d");
    let (tw, th) = extent(&tall);
    assert!(th > 1.0 && tw / th < 4.0, "{tw} {th}");
}

// ---------------------------------------------------------------- rounded and chamfered

/// Rounded rectangles with concave arcs and with no arc (a straight side), and a tall one whose
/// convex arcs meet.
#[test]
fn rounded_rectangle_arcs() {
    let body = node("shapes.misc", "rounded rectangle, draw, rounded rectangle west arc=concave, rounded rectangle east arc=none, outer sep=0pt", "text");
    clean(&body);
    // the straight east side: a vertical line
    let s = paths(&canvas(&body))[0].segments.clone();
    let e = at(&body, "east").x;
    assert!(s.windows(2).any(|w| matches!(w, [Segment::LineTo(a), Segment::LineTo(b)] if close(a.x, e) && close(b.x, e))), "{s:?}");
    // concave: the west point of the shape is inside its corners
    let (w, nw) = (at(&body, "west"), at(&body, "north west"));
    assert!(w.x > nw.x - 1e-6, "{w:?} {nw:?}");
    let body = node("shapes.misc", "rounded rectangle, draw, rounded rectangle east arc=concave, rounded rectangle arc length=120", "text");
    clean(&body);
    // tall and narrow: a circle
    let body = node("shapes.misc", "rounded rectangle, draw, minimum height=2cm, outer sep=0pt", "i");
    let (w, h) = extent(&body);
    assert!((w - h).abs() < 0.05, "{w} {h}");
}

/// Chamfered rectangles: a chamfer larger than the text is cut to it; a minimum height.
#[test]
fn chamfered_rectangles() {
    let body = node("shapes.misc", "chamfered rectangle, draw, inner sep=0pt, chamfered rectangle ysep=1cm", "i");
    clean(&body);
    let tall = node("shapes.misc", "chamfered rectangle, draw, minimum height=3cm, outer sep=0pt", "x");
    let (_, h) = extent(&tall);
    assert!((h - 3.0).abs() < 1e-3, "{h}");
}

// ---------------------------------------------------------------- callouts

/// Callouts pointing every way: the pointer anchor is where the pointer was asked to go.
#[test]
fn callouts_point_every_way() {
    for shape in ["ellipse callout", "rectangle callout", "cloud callout"] {
        for (x, y) in [(3.0, 0.0), (3.0, 0.2), (0.2, 3.0), (-3.0, 0.2), (-0.2, -3.0), (3.0, -0.3), (2.0, 2.0)] {
            let body = node("shapes.callouts", &format!("{shape}, draw, outer sep=0pt, callout absolute pointer={{({x},{y})}}"), "some text");
            clean(&body);
            let p = at(&body, "pointer");
            assert!(near(p, x, y), "{shape} ({x},{y}): {p:?}");
        }
    }
    // the puffs of a cloud callout's pointer: sizes given as lengths
    for size in ["4pt and 6pt", "5pt"] {
        let body = node("shapes.callouts", &format!("cloud callout, draw, callout pointer start size={size}, callout absolute pointer={{(3,0)}}"), "x");
        clean(&body);
        assert!(near(at(&body, "pointer"), 3.0, 0.0));
    }
}

// ---------------------------------------------------------------- logic gates

/// US gates as wide as their minimum width, a not gate with an inverted input, and the IEC
/// symbol in each corner.
#[test]
fn logic_gates() {
    for gate in ["and gate US", "or gate US", "nand gate US", "xor gate US"] {
        let body = node("shapes.gates.logic.US", &format!("{gate}, draw, logic gate inputs=nn, minimum width=3cm, outer sep=0pt"), "");
        clean(&body);
        let w = at(&body, "output").x - at(&body, "input 1").x;
        assert!(w > 2.0, "{gate}: {w}");
        // the border in every direction
        for a in 0..12 {
            at(&body, &format!("{}", a * 30));
        }
    }
    let inverted = node("shapes.gates.logic.US", "not gate US, draw, logic gate inputs=i", "");
    let plain = node("shapes.gates.logic.US", "not gate US, draw, logic gate inputs=n", "");
    clean(&inverted);
    let circles = |b: &str| paths(&canvas(b)).iter().map(|d| d.segments.iter().filter(|s| matches!(s, Segment::CurveTo(..))).count()).sum::<usize>();
    assert_eq!(circles(&inverted), circles(&plain) + 4);
    let at_input = at(&inverted, "input");
    assert!(at_input.x < at(&plain, "input").x, "the bubble moves the input out");
    let mut places = Vec::new();
    for align in ["top", "bottom", "left", "right", "top, right", "middle"] {
        let body = format!("\\usetikzlibrary{{shapes.gates.logic.IEC}}\\node[and gate IEC, draw, minimum size=2cm, logic gate IEC symbol align={{{align}}}] (s) {{}};");
        clean(&body);
        let (c, _) = tikra::engine::run_document(&format!("\\begin{{tikzpicture}}{body}\\end{{tikzpicture}}")).unwrap();
        let place = c.items.iter().find_map(|i| if let Item::Text { place, .. } = i { Some((place.e, place.f)) } else { None }).expect("the symbol");
        places.push(place);
    }
    let (top, bottom, left, right) = (places[0], places[1], places[2], places[3]);
    assert!(top.1 > bottom.1 + 1.0 && right.0 > left.0 + 1.0, "{places:?}");
    assert!(close(places[4].0, right.0) && close(places[4].1, top.1), "{places:?}");
    // a word that is no side leaves the symbol at the center
    let middle = places[5];
    assert!(middle.0 > left.0 + 0.3 && middle.0 < right.0 - 0.3 && middle.1 < top.1 - 0.3 && middle.1 > bottom.1 + 0.3, "{places:?}");
}

// ---------------------------------------------------------------- arrow shapes

/// A single arrow turned a quarter, an arrow box with a list of arrows and one with none.
#[test]
fn arrow_shapes() {
    let up = node("shapes.arrows", "single arrow, draw, shape border rotate=90", "x");
    clean(&up);
    let (tip, c) = (at(&up, "tip"), at(&up, "center"));
    assert!(tip.y > c.y + 0.2 && close(tip.x, c.x), "{tip:?} {c:?}");
    let listed = node("shapes.arrows", "arrow box, draw, arrow box arrows={east:1cm, north}", "x");
    clean(&listed);
    let c = at(&listed, "center");
    let (e, n, w) = (at(&listed, "east arrow tip"), at(&listed, "north arrow tip"), at(&listed, "west arrow tip"));
    // east and north 1cm from the center, west none
    assert!(e.x - c.x > 0.9 && n.y - c.y > 0.9 && c.x - w.x < 0.5, "{c:?} {e:?} {n:?} {w:?}");
    let none = node("shapes.arrows", "arrow box, draw, arrow box west arrow=none", "x");
    let all = node("shapes.arrows", "arrow box, draw", "x");
    assert!(at(&none, "west").x > at(&all, "west").x + 0.3);
}

// ---------------------------------------------------------------- electrical shapes

/// `direction ee` with another tip: a box as long as that tip, which is drawn; the wave of an
/// alternating source; the code of a generic circle.
#[test]
fn electrical_shapes_with_other_tips_and_marks() {
    let body = node("circuits.ee.IEC", r"direction ee, draw, /pgf/direction ee arrow={Latex[length=4mm]}", "");
    clean(&body);
    let (i, o) = (at(&body, "input"), at(&body, "output"));
    assert!((o.x - i.x - 0.4).abs() < 0.05, "{i:?} {o:?}");
    let filled = canvas(&body).items.iter().filter(|x| matches!(x, Item::Path { fill: Some(_), .. })).count();
    assert!(filled >= 1);
    // a tip that does not exist: the box of `direction ee` itself, 4pt long
    let body = node("circuits.ee.IEC", r"direction ee, draw, /pgf/direction ee arrow={Nonsense}", "");
    let (i, o) = (at(&body, "input"), at(&body, "output"));
    assert!((o.x - i.x - 4.0 * PT).abs() < 1e-4, "{i:?} {o:?}");
    let body = r"\usetikzlibrary{circuits.ee.IEC}\draw[circuit ee IEC] (0,0) to [ac source] (3,0);";
    clean(body);
    assert!(paths(&canvas(body)).iter().any(|d| d.segments.iter().filter(|s| matches!(s, Segment::CurveTo(..))).count() == 2), "the wave");
    let body = r"\usetikzlibrary{circuits.ee.IEC}\node[generic circle IEC, draw, /pgf/generic circle IEC/before background={\pgfpathmoveto{\pgfpoint{-1pt}{0pt}}\pgfpathlineto{\pgfpoint{1pt}{0pt}}\pgfusepathqstroke}] (s) {};";
    clean(body);
    let c = at(body, "center");
    assert!(lines(body).iter().any(|(a, b)| close(a.y, c.y) && close(b.y, c.y) && b.x > a.x), "the code's line");
}

// ---------------------------------------------------------------- shapes in general

/// An unknown shape is reported; an empty anchor is the center.
#[test]
fn unknown_shapes_and_empty_anchors() {
    let w = warnings(r"\node[shape=blob, draw] {x};");
    assert!(w.iter().any(|w| w.contains("blob")), "{w:?}");
    // a node placed by an empty anchor: by its center
    let body = r"\node[draw, anchor={}] (s) at (1,2) {x};";
    let c = at(body, "center");
    assert!(near(c, 1.0, 2.0), "{c:?}");
}
