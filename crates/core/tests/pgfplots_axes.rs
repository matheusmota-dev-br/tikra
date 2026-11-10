//! pgfplots' axes: their limits and size (enlarged, scaled, of polar and smith charts), where
//! they are placed, their lines, ticks and tick labels (scaled, logarithmic, symbolic), the
//! coordinate systems of statements inside them, and `\addplot graphics`.

mod common;
use std::collections::HashMap;
use tikra::canvas::{Canvas, Fill, Item, Paint};
use tikra::scene::Segment;

/// 1pt in the SVG's units (bp).
const PT: f64 = 72.0 / 72.27;
/// 1cm in the SVG's units.
const CM: f64 = 72.0 / 2.54;

/// A picture with an axis (`axis`, or the environment before `:::`) of these options around a body.
fn picture(options: &str, body: &str) -> String {
    let (env, options) = options.split_once(":::").unwrap_or(("axis", options));
    format!("\\begin{{tikzpicture}}\\begin{{{env}}}[{options}]\n{body}\n\\end{{{env}}}\\end{{tikzpicture}}")
}

fn warnings(options: &str, body: &str) -> Vec<String> {
    common::figure(&picture(options, body)).warnings
}

fn has(w: &[String], fragment: &str) -> bool {
    w.iter().any(|w| w.contains(fragment))
}

/// The named points of a figure, in the SVG's units (y down).
fn named(code: &str) -> HashMap<String, (f64, f64)> {
    let r = common::figure(code);
    assert!(r.warnings.is_empty(), "warnings: {:?}\n{code}", r.warnings);
    r.points.into_iter().map(|p| (p.name, (p.x, p.y))).collect()
}

/// Where coordinates are in the axis box: 0 at its left (bottom) side, 1 at its right (top).
fn rel(options: &str, body: &str, coords: &[&str]) -> Vec<(f64, f64)> {
    let mut b = format!("{body} \\coordinate (o) at (rel axis cs:0,0); \\coordinate (x) at (rel axis cs:1,0); \\coordinate (y) at (rel axis cs:0,1);");
    for (k, c) in coords.iter().enumerate() {
        b.push_str(&format!("\\coordinate (p{k}) at ({c});"));
    }
    let p = named(&picture(options, &b));
    let (o, x, y) = (p["o"], p["x"], p["y"]);
    (0..coords.len())
        .map(|k| {
            let q = p[&format!("p{k}")];
            ((q.0 - o.0) / (x.0 - o.0), (q.1 - o.1) / (y.1 - o.1))
        })
        .collect()
}

/// The axis box's width and height, in pt.
fn box_size(options: &str, body: &str) -> (f64, f64) {
    let p = named(&picture(options, &format!("{body} \\coordinate (o) at (rel axis cs:0,0); \\coordinate (t) at (rel axis cs:1,1);")));
    let (o, t) = (p["o"], p["t"]);
    ((t.0 - o.0).abs() / PT, (t.1 - o.1).abs() / PT)
}

fn close(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() < tol
}

/// The canvas of a figure.
fn drawn(options: &str, body: &str) -> Canvas {
    let (c, w) = tikra::engine::run_document(&picture(options, body)).unwrap();
    assert!(w.is_empty(), "warnings: {w:?}\n{body}");
    c
}

fn texts(c: &Canvas) -> usize {
    c.items.iter().filter(|i| matches!(i, Item::Text { .. })).count()
}

/// The drawing's size, in cm.
fn size(c: &Canvas) -> (f64, f64) {
    let (a, b) = (c.min.unwrap(), c.max.unwrap());
    (b.x - a.x, b.y - a.y)
}

const LINE: &str = r"\addplot coordinates {(0,0) (10,1)};";

// ---------------------------------------------------------------------------------------------
// limits

#[test]
fn enlarged_limits() {
    // the data from 0 to 10, enlarged by 10% of that on the sides asked for
    let at = |opt: &str| rel(opt, LINE, &["axis cs:0,0", "axis cs:10,1"]);
    let r = at("enlarge x limits=true");
    assert!(close(r[0].0, 1.0 / 12.0, 1e-3) && close(r[1].0, 11.0 / 12.0, 1e-3), "{r:?}");
    let r = at("enlarge x limits=upper");
    assert!(close(r[0].0, 0.0, 1e-3) && close(r[1].0, 10.0 / 11.0, 1e-3), "{r:?}");
    let r = at("enlarge x limits=lower");
    assert!(close(r[0].0, 1.0 / 11.0, 1e-3) && close(r[1].0, 1.0, 1e-3), "{r:?}");
    // a part that is not understood does not enlarge
    let r = at("enlarge x limits={foo=1}");
    assert!(close(r[0].0, 0.0, 1e-3) && close(r[1].0, 1.0, 1e-3), "{r:?}");
    // `abs=1cm`: the axis keeps its length, the data leave a centimeter on each side
    let p = named(&picture(
        "enlarge x limits={abs=1cm}",
        &format!(r"{LINE} \coordinate (o) at (rel axis cs:0,0); \coordinate (d) at (axis cs:0,0); \coordinate (e) at (axis cs:10,0); \coordinate (x) at (rel axis cs:1,0);"),
    ));
    assert!(close(p["d"].0 - p["o"].0, CM, 0.01) && close(p["x"].0 - p["e"].0, CM, 0.01), "{p:?}");
    let (w, _) = box_size("enlarge x limits={abs=1cm}", LINE);
    assert!(close(w, 240.0 - 45.0, 0.01), "{w}");
}

#[test]
fn limits_of_log_axes() {
    // one value on a log axis: a decade on each side
    let r = rel("xmode=log", r"\addplot coordinates {(10,1)};", &["axis cs:10,1"]);
    assert!(close(r[0].0, 0.5, 1e-6), "{r:?}");
    // no data on a log axis: the default window, with a warning
    let w = warnings("xmode=log", "");
    assert_eq!(w, vec!["axis without data: drew the default window from 0 to 1".to_string()]);
}

#[test]
fn stacked_plots_of_different_lengths() {
    // the third point of the second plot has nothing below it; the third plot stacks on it
    let c = drawn(
        "hide axis, scale only axis, width=10cm, height=10cm, xmin=0, xmax=2, ymin=0, ymax=4, stack plots=y",
        r"\addplot coordinates {(0,1) (1,1)}; \addplot coordinates {(0,1) (1,1) (2,1)}; \addplot coordinates {(0,1) (1,1) (2,1)};",
    );
    let lines: Vec<Vec<(f64, f64)>> = c
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Path { segments, stroke: Some(_), fill: None } => Some(common::points(segments)),
            _ => None,
        })
        .collect();
    let last = lines.last().unwrap();
    // y = 3 at x = 0, y = 2 at x = 2: 2.5cm apart
    assert!(close(last[0].1 - last[2].1, 2.5, 1e-6), "{lines:?}");
}

// ---------------------------------------------------------------------------------------------
// size

#[test]
fn sizes_given_in_part() {
    // only the height: the width keeps the default ratio (240pt × 207pt), less the labels' 45pt
    let (w, h) = box_size("height=5cm", LINE);
    let full = 5.0 / 2.54 * 72.27;
    assert!(close(h, full - 45.0, 0.01) && close(w, full * 240.0 / 207.0 - 45.0, 0.01), "{w} × {h}");
    // `x post scale`: the x unit vector, scaled after all
    let (w, h) = box_size("x post scale=2", LINE);
    assert!(close(w, 2.0 * (240.0 - 45.0), 0.01) && close(h, 207.0 - 45.0, 0.01), "{w} × {h}");
}

#[test]
fn sizes_of_round_charts() {
    // a smith chart or a polar axis: a circle of the size given, the other side as large
    for env in ["smithchart", "polaraxis"] {
        let width = |opt: &str| size(&drawn(&format!("{env}:::{opt}"), r"\addplot coordinates {(0.5,0)};")).0;
        // both: stretched to them
        let square = width("width=5cm, height=5cm");
        let both = width("width=4cm, height=5cm");
        // (a smith chart's labels around the circle move with it)
        assert!(close(square - both, 1.0, 0.05), "{env}: {square} {both}");
        // the height alone: as wide as high
        let h = width("height=5cm");
        assert!(close(h, square, 1e-6), "{env}: {h} {square}");
        // the width alone, of a polar axis: as high as wide
        if env == "polaraxis" {
            assert!(close(width("width=5cm"), square, 1e-6), "{env}");
        }
    }
}

#[test]
fn unit_vector_ratio_keeps_the_size() {
    // equal units on both axes in a box that is wider than high: the x range grows
    let r = rel("unit vector ratio=1 1, unit rescale keep size=true, xmin=0, xmax=10, ymin=0, ymax=10", LINE, &["axis cs:0,0", "axis cs:10,10"]);
    assert!(r[0].0 > 0.05 && r[1].0 < 0.95 && close(r[0].0, 1.0 - r[1].0, 1e-6), "{r:?}");
    assert!(close(r[0].1, 0.0, 1e-6) && close(r[1].1, 1.0, 1e-6), "{r:?}");
}

// ---------------------------------------------------------------------------------------------
// where the axis goes

#[test]
fn the_axis_anchor() {
    // where the box's lower left corner lands, from the picture's origin (SVG units, y down)
    let corner = |anchor: &str| {
        let p = named(&format!(
            r"\begin{{tikzpicture}}\begin{{axis}}[anchor={anchor}, title=T, title style={{name=t}}, name=a]{LINE}\end{{axis}}\coordinate (o) at (a.south west);\coordinate (zero) at (0,0);\end{{tikzpicture}}"
        ));
        (p["o"].0 - p["zero"].0, p["o"].1 - p["zero"].1)
    };
    // `below north west`: the box's left side, the bottom of everything drawn (below the box)
    let (dx, dy) = corner("below north west");
    assert!(close(dx, 0.0, 1e-6) && dy < -5.0, "{dx} {dy}");
    // an anchor that is not one: the box's lower left corner
    let (dx, dy) = corner("nowhere");
    assert!(close(dx, 0.0, 1e-6) && close(dy, 0.0, 1e-6), "{dx} {dy}");
    // a node drawn in the axis: its anchor (the title's top is above the box)
    let (_, title) = corner("t.north");
    let (_, top) = corner("north west");
    assert!(title > top + 5.0, "{title} {top}");
}

#[test]
fn the_axis_at_a_point_that_is_not_one() {
    let w = warnings("at={(nonsense)}", LINE);
    assert_eq!(w, vec!["axis at=(nonsense): figure: unknown coordinate: (nonsense)".to_string()]);
}

// ---------------------------------------------------------------------------------------------
// ticks

/// How many texts an axis draws.
fn labels(options: &str, body: &str) -> usize {
    texts(&drawn(options, body))
}

#[test]
fn tick_lists() {
    // `xtickmin`, `xtickmax`: no tick outside (0, 2, …, 10 are six labels, 4 and 6 two)
    assert_eq!(labels("ytick=\\empty", LINE), 6);
    assert_eq!(labels("ytick=\\empty, xtickmin=3, xtickmax=7", LINE), 2);
    // `ytickten` on a log axis: ticks at 10^1 and 10^2
    assert_eq!(labels("xtick=\\empty, ymode=log, ytickten={1,2}", r"\addplot coordinates {(0,1) (10,1000)};"), 2);
    // names of a symbolic axis, and a number among them
    assert_eq!(labels("ytick=\\empty, symbolic x coords={a,b,c}, xtick={a,c}", r"\addplot coordinates {(a,1) (b,2) (c,3)};"), 2);
    assert_eq!(labels("ytick=\\empty, symbolic x coords={a,b,c}, xtick={a,1,c}", r"\addplot coordinates {(a,1) (b,2) (c,3)};"), 3);
    // a list item that is no number is left out
    assert_eq!(labels("ytick=\\empty, xtick={2,x,4}", LINE), 2);
    // powers of two
    let ten = labels("xtick=\\empty, ymode=log", r"\addplot coordinates {(0,1) (10,1024)};");
    let two = labels("xtick=\\empty, ymode=log, log basis y=2", r"\addplot coordinates {(0,1) (10,1024)};");
    assert!(two > ten, "{two} {ten}");
}

#[test]
fn symbolic_coordinates_by_number() {
    // `[normalized]0.5`: the number itself, between a and b
    let r = rel("symbolic x coords={a,b,c}, enlargelimits=false", r"\addplot coordinates {([normalized]0.5,1) (c,3)};", &["axis cs:[normalized]0.5,1"]);
    assert!(close(r[0].0, 0.0, 1e-6), "{r:?}");
}

#[test]
fn scaled_tick_labels() {
    // one more text: the factor the labels are divided by
    let plain = labels("scaled x ticks=false", r"\addplot coordinates {(0,0) (10000,1)};");
    for opt in
        ["scaled x ticks=base 10:3", "scaled x ticks=real:1000", r"scaled x ticks=manual:{$\cdot 10^3$}{\pgfmathparse{#1/1000}}", r"xtick scale label code/.code={$x 10^{#1}$}"]
    {
        assert_eq!(labels(opt, r"\addplot coordinates {(0,0) (10000,1)};"), plain + 1, "{opt}");
    }
    // `typeset ticklabels with strut`: each label as high and deep as a strut
    let highest = |opt: &str| drawn(opt, LINE).items.iter().filter_map(|i| if let Item::Text { label, .. } = i { Some(label.height()) } else { None }).fold(0.0, f64::max);
    let (low, tall) = (highest(""), highest("typeset ticklabels with strut"));
    assert!(tall > low + 0.01, "{low} {tall}");
}

#[test]
fn axis_line_shift_and_discontinuities() {
    // `axis x line shift`: a number is in units of the other axis; a length is a length
    let plain = size(&drawn("axis lines=left", LINE));
    let pt = size(&drawn("axis lines=left, axis x line shift=10pt", LINE));
    assert!(pt.1 > plain.1 + 0.1, "{plain:?} {pt:?}");
    // half the y unit (the box is 162pt high for 1): 81pt
    let unit = size(&drawn("axis lines=left, axis x line shift=0.5", LINE));
    assert!(unit.1 > pt.1 + 1.0, "{pt:?} {unit:?}");
    // `axis x discontinuity`: a zigzag (`crunch`) or two bars (`parallel`) near the start
    // (the axis lines: paths of one move and only lines, or of four moves and four lines)
    let shapes = |opt: &str| -> Vec<(usize, usize)> {
        drawn(&format!("axis lines=left, {opt}"), r"\addplot coordinates {(0,5) (10,6)};")
            .items
            .iter()
            .filter_map(|i| if let Item::Path { segments, .. } = i { Some(segments.clone()) } else { None })
            .filter(|s| s.iter().all(|s| matches!(s, Segment::MoveTo(_) | Segment::LineTo(_))))
            .map(|s| (s.iter().filter(|s| matches!(s, Segment::MoveTo(_))).count(), s.len()))
            .collect()
    };
    assert!(shapes("axis x discontinuity=crunch").contains(&(1, 6)));
    assert!(!shapes("").contains(&(1, 6)));
    assert!(shapes("axis y discontinuity=parallel").contains(&(4, 8)));
    assert!(!shapes("").contains(&(4, 8)));
}

#[test]
fn axis_background_and_units() {
    // `axis background/.style`: a rectangle behind the axis
    let c = drawn("axis background/.style={fill=gray!20}", LINE);
    let gray = c.items.iter().any(|i| matches!(i, Item::Path { fill: Some(Fill { paint: Paint::Solid(col), .. }), .. } if col.hex() == "#e6e6e6"));
    assert!(gray);
    // the units library: the label is followed by ` [km]`
    let width = |opt: &str| {
        drawn(&format!("xlabel=Distance, {opt}"), LINE).items.iter().filter_map(|i| if let Item::Text { label, .. } = i { Some(label.width()) } else { None }).fold(0.0, f64::max)
    };
    assert!(width("use units, x unit=m, x unit prefix=k") > width("") + 0.3);
}

#[test]
fn colormaps_that_are_not_there() {
    let w = warnings("colormap name=nope", r"\addplot[scatter] coordinates {(0,0) (10,1)};");
    assert_eq!(w, vec!["colormap 'nope' not defined".to_string()]);
}

// ---------------------------------------------------------------------------------------------
// statements in the axis

#[test]
fn data_point_coordinates_in_a_node() {
    // `\pgfplotspointgetcoordinates`: the axis' coordinates of a point, as /data point/x
    let d =
        common::figure(&picture("", &format!(r"{LINE} \node at (axis cs:5,0.5) {{\pgfplotspointgetcoordinates{{(axis cs:5,0.5)}}\pgfkeysvalueof{{/data point/x}}}};"))).description;
    assert!(d.ends_with("Other text: 5."), "{d}");
}

/// The named points of an axis' body.
fn points_in(options: &str, body: &str) -> HashMap<String, (f64, f64)> {
    named(&picture(options, body))
}

fn same(a: (f64, f64), b: (f64, f64)) -> bool {
    close(a.0, b.0, 1e-6) && close(a.1, b.1, 1e-6)
}

#[test]
fn cartesian_coordinates_of_ternary_and_smith_charts() {
    // a ternary axis: `cartesian cs`, and plain coordinates, are the same plane
    let p = points_in(
        "ternaryaxis:::",
        r"\addplot3 coordinates {(0.2,0.3,0.5)}; \coordinate (c) at (cartesian cs:0.5,0.25); \coordinate (p) at (0.5,0.25); \coordinate (o) at (cartesian cs:0,0);",
    );
    assert!(same(p["c"], p["p"]) && !same(p["c"], p["o"]), "{p:?}");
    // a smith chart: (0,0) of the unit circle's plane is the impedance 1
    let p = points_in("smithchart:::", r"\addplot coordinates {(0.5,0)}; \coordinate (c) at (cartesian cs:0,0); \coordinate (z) at (axis cs:1,0);");
    assert!(same(p["c"], p["z"]), "{p:?}");
}

#[test]
fn box_plot_and_direction_coordinates() {
    // the box plot's systems: along its box (the data axis) and across it
    let p = points_in(
        "",
        r"\addplot+[boxplot] table[row sep=\\, y index=0] {1\\2\\3\\4\\5\\};
          \coordinate (a) at (boxplot box cs:0,0.5); \coordinate (b) at (boxplot box cs:1,0.5); \coordinate (w) at (boxplot whisker cs:1,0);",
    );
    assert!(!same(p["a"], p["b"]) && !same(p["b"], p["w"]), "{p:?}");
    // `axis direction cs`: a difference of two points
    let p = points_in("", &format!(r"{LINE} \path (axis cs:1,0) ++(axis direction cs:2,0.5) coordinate (d); \coordinate (e) at (axis cs:3,0.5);"));
    assert!(same(p["d"], p["e"]), "{p:?}");
    // a system of TikZ's, inside an axis
    let p = points_in("", &format!(r"{LINE} \coordinate (k) at (canvas cs:x=1cm,y=0cm); \coordinate (z) at (canvas cs:x=0cm,y=0cm);"));
    assert!(close(p["k"].0 - p["z"].0, CM, 1e-6), "{p:?}");
}

#[test]
fn box_plot_coordinates_outside_a_box_plot() {
    // no box plot drawn yet (in this thread: the last box plot's numbers stay for the next figure)
    let w = warnings("", r"\addplot coordinates {(0,0)}; \draw (boxplot cs:0,0) -- (1,1);");
    assert_eq!(w, vec!["\\draw: figure: boxplot cs outside a box plot; statement ignored".to_string()]);
}

#[test]
fn tick_label_coordinates() {
    // `xticklabel cs`: on the x tick labels' line, moved out by the extra length
    let p = points_in(
        "axis lines=middle",
        r"\addplot coordinates {(-1,-1) (10,1)}; \coordinate (a) at (xticklabel cs:0.5); \coordinate (b) at (xticklabel cs:0.5,5pt);
          \coordinate (s) at (xticklabel* cs:0.5); \coordinate (zero) at (axis cs:0,0); \coordinate (low) at (rel axis cs:0,0);",
    );
    assert!(close(p["b"].1 - p["a"].1, 5.0 * PT, 1e-6), "{p:?}");
    // the line is the x axis' (y = 0 with `axis lines=middle`), not the box's lower side
    assert!(p["s"].1 >= p["zero"].1 && p["a"].1 < p["low"].1, "{p:?}");
}

#[test]
fn plain_coordinates_before_compat_1_11() {
    // the data scale transformation's units: x is 10 times the data (0 to 10 on 240pt), y 100
    // times; an empty y is the lower limit
    let p = points_in(
        "compat=1.10",
        &format!(r"{LINE} \coordinate (p) at (5,{{}}); \coordinate (q) at (axis cs:0.5,0); \coordinate (r) at (5,50); \coordinate (s) at (axis cs:0.5,0.5);"),
    );
    assert!(same(p["p"], p["q"]) && same(p["r"], p["s"]), "{p:?}");
    // a length stays TikZ's
    let p = points_in("compat=1.10", &format!(r"{LINE} \coordinate (u) at (1cm,0pt); \coordinate (c) at (canvas cs:x=1cm,y=0cm);"));
    assert!(same(p["u"], p["c"]), "{p:?}");
    // a log axis has no transformation: the natural logarithm of the data
    let p = points_in("compat=1.10, xmode=log", r"\addplot coordinates {(1,0) (100,1)}; \coordinate (p) at (2,50); \coordinate (q) at (axis cs:7.38905609893065,0.5);");
    assert!(close(p["p"].0, p["q"].0, 1e-6), "{p:?}");
}

/// The texts' widths, widest first.
fn widths(options: &str, body: &str) -> Vec<f64> {
    let mut w: Vec<f64> = drawn(options, body).items.iter().filter_map(|i| if let Item::Text { label, .. } = i { Some(label.width()) } else { None }).collect();
    w.sort_by(|a, b| b.total_cmp(a));
    w
}

#[test]
fn tick_label_texts() {
    // `\pgfmathprintnumber\tick`, unbraced, and with options and a space: the labels as usual
    let plain = widths("", LINE);
    assert_eq!(widths(r"xticklabel={\pgfmathprintnumber\tick}", LINE), plain);
    assert_eq!(widths(r"xticklabel={\pgfmathprintnumber[fixed] {\tick}}", LINE), plain);
    // the interval's ends (`\nexttick`): one label per interval, wider
    let intervals = widths("xticklabel interval boundaries, xtick={0,5,10}, ytick=\\empty", LINE);
    assert_eq!(intervals.len(), 2);
    assert!(intervals[0] > plain[0], "{intervals:?} {plain:?}");
}

#[test]
fn axes_on_top() {
    // the plot first, then the axis
    let last_is_plot = |opt: &str, body: &str| {
        let c = drawn(opt, body);
        let last = c.items.iter().rev().find_map(|i| if let Item::Path { stroke: Some(s), .. } = i { Some(s.color.hex()) } else { None }).unwrap();
        last == "#0000ff"
    };
    for (env, body) in [
        ("polaraxis", r"\addplot[blue, no marks] coordinates {(0,1) (90,2)};"),
        ("ternaryaxis", r"\addplot3[blue, no marks] coordinates {(0.2,0.3,0.5) (0.5,0.3,0.2)};"),
        ("smithchart", r"\addplot[blue, no marks] coordinates {(0.5,0) (1,1)};"),
    ] {
        assert!(last_is_plot(&format!("{env}:::"), body), "{env}");
        assert!(!last_is_plot(&format!("{env}:::axis on top"), body), "{env}");
    }
    // a smith chart's origin, marked
    let c = |opt: &str| drawn(&format!("smithchart:::{opt}"), r"\addplot coordinates {(0.5,0)};").items.len();
    assert!(c("show origin") > c(""));
}

#[test]
fn more_of_the_axis() {
    // units of the axis' own (`x=1cm`): one unit is a centimeter
    let p = points_in("x=1cm, y=2cm", r"\addplot coordinates {(0,0) (3,1)}; \coordinate (a) at (axis cs:0,0); \coordinate (b) at (axis cs:1,1);");
    assert!(close(p["b"].0 - p["a"].0, CM, 1e-6) && close(p["a"].1 - p["b"].1, 2.0 * CM, 1e-6), "{p:?}");
    // a view in radians, with `trig format=rad`
    let deg = size(&drawn("view={30}{20}", r"\addplot3 coordinates {(0,0,0) (1,1,1)};"));
    let (c, w) = tikra::engine::run_document(r"\begin{tikzpicture}\pgfkeys{/pgf/trig format=rad}\begin{axis}[view={0.5235987755982988}{0.3490658503988659}]\addplot3 coordinates {(0,0,0) (1,1,1)};\end{axis}\end{tikzpicture}").unwrap();
    assert!(w.is_empty(), "{w:?}");
    let rad = size(&c);
    assert!(close(deg.0, rad.0, 1e-6) && close(deg.1, rad.1, 1e-6), "{deg:?} {rad:?}");
    // group plots: the inner axes without their tick labels
    let body = r"\nextgroupplot \addplot coordinates {(0,0) (1,1)}; \nextgroupplot \addplot coordinates {(0,0) (1,1)};";
    let all = texts(&drawn("groupplot:::group style={group size=2 by 1}", body));
    let edge = texts(&drawn("groupplot:::group style={group size=2 by 1, y descriptions at=edge left}", body));
    assert!(edge < all, "{edge} {all}");
    // `extra description` and `after end axis`: code drawn with the axis
    let before = texts(&drawn("", LINE));
    let after = texts(&drawn(r"extra description/.code={\node at (0.5,0.5) {m};}, after end axis/.code={\node at (0,0) {e};}", LINE));
    assert_eq!(after, before + 2);
    // an axis line that is not drawn
    let paths = |opt: &str| drawn(opt, LINE).items.iter().filter(|i| matches!(i, Item::Path { .. })).count();
    assert!(paths("axis x line=none, axis y line=left") < paths("axis x line=bottom, axis y line=left"));
}

// ---------------------------------------------------------------------------------------------
// keys and commands of the axis

/// The colors of the stroked paths.
fn stroke_colors(c: &Canvas) -> Vec<String> {
    c.items.iter().filter_map(|i| if let Item::Path { stroke: Some(s), .. } = i { Some(s.color.hex()) } else { None }).collect()
}

#[test]
fn colors_of_a_colormap() {
    let map = r"\pgfplotsset{colormap={rg}{rgb=(1,0,0) rgb=(0,1,0)}, colormap={s}{samples of colormap=(3 of rg)}}";
    let color = |opt: &str| {
        let (c, w) = tikra::engine::run_document(&format!(
            r"{map}\begin{{tikzpicture}}\begin{{axis}}[hide axis, colormap name=rg]\addplot[no marks, {opt}] coordinates {{(0,0) (1,1)}};\end{{axis}}\end{{tikzpicture}}"
        ))
        .unwrap();
        assert!(w.is_empty(), "{opt}: {w:?}");
        stroke_colors(&c)[0].clone()
    };
    // a position of 0 to 1000 in the colormap, its colors by index, piecewise constant
    assert_eq!(color("color of colormap={0 of rg}"), "#ff0000");
    assert_eq!(color("color of colormap={500 of rg}"), "#808000");
    assert_eq!(color("color of colormap={1000}"), "#00ff00");
    assert_eq!(color("index of colormap={1 of rg}"), "#00ff00");
    assert_eq!(color("const color of colormap={0 of rg}"), "#ff0000");
    assert_eq!(color("const color of colormap={1000 of rg}"), "#00ff00");
    // a colormap of samples of another: three colors, the middle one between red and green
    assert_eq!(color("index of colormap={1 of s}"), "#808000");
    // a colormap's style
    let (c, _) = tikra::engine::run_document(r"\begin{tikzpicture}\begin{axis}[hide axis, colormap/cool]\addplot[no marks, color of colormap={1000}] coordinates {(0,0) (1,1)};\addplot[no marks, color of colormap={1000 of colormap/cool}] coordinates {(0,0) (1,1)};\end{axis}\end{tikzpicture}").unwrap();
    let s = stroke_colors(&c);
    assert_eq!(s[0], s[1]);
    // a colormap that is not there
    let w = warnings("", r"\addplot[color of colormap={10 of nope}] coordinates {(0,0) (1,1)};");
    assert!(has(&w, "color of colormap: colormap 'nope' not defined"), "{w:?}");
}

#[test]
fn si_prefixes() {
    let width = |opt: &str| widths(&format!("xlabel=D, x unit=m, use units, {opt}"), LINE)[0];
    // the label's unit gets the prefix
    assert!(width("x SI prefix=kilo") > width(""), "{} {}", width("x SI prefix=kilo"), width(""));
    // not a prefix
    let w = warnings("x SI prefix=lots", LINE);
    assert_eq!(w, vec!["x SI prefix=lots not supported: ignored".to_string()]);
    // `change x base`: the tick labels in that unit (10 m is 0.01 km), without a scale label
    let plain = labels("ytick=\\empty", LINE);
    assert_eq!(labels("ytick=\\empty, change x base, x SI prefix=kilo", LINE), plain);
    assert_ne!(widths("ytick=\\empty, change x base, x SI prefix=kilo", LINE), widths("ytick=\\empty", LINE));
}

#[test]
fn nodes_sloped_like_an_axis() {
    // the angle of the node's text
    let angle = |opt: &str, node: &str| {
        let c = drawn(&format!("hide axis, {opt}"), &format!(r"{LINE} \node[{node}] at (axis cs:5,0.5) {{a}};"));
        c.items.iter().find_map(|i| if let Item::Text { place, .. } = i { Some(place.b.atan2(place.a).to_degrees()) } else { None }).unwrap()
    };
    // the x axis of a slanted axis: atan(1/2)
    let slant = "x={(1cm,0.5cm)}, y={(0cm,1cm)}";
    assert!(close(angle(slant, "sloped like x axis"), 26.565051177, 1e-6));
    assert!(close(angle(slant, "sloped like y axis"), 90.0, 1e-6));
    // reversed: turned upright, unless upside down is allowed
    let reversed = &format!("{slant}, x dir=reverse");
    assert!(close(angle(reversed, "sloped like x axis"), 26.565051177, 1e-6));
    assert!(close(angle(reversed, "sloped like x axis={allow upside down}"), 26.565051177 - 180.0, 1e-6));
    assert!(close(angle(reversed, r"sloped like x axis={execute for upside down={\tikzset{anchor=west}}}"), 26.565051177, 1e-6));
    // outside an axis: nothing
    let c = tikra::engine::run_document(r"\begin{tikzpicture}\node[sloped like x axis] {a};\end{tikzpicture}").unwrap().0;
    let a = c.items.iter().find_map(|i| if let Item::Text { place, .. } = i { Some(place.b) } else { None }).unwrap();
    assert_eq!(a, 0.0);
}

#[test]
fn axes_in_axes_and_outside_pictures() {
    let r = common::figure(r"\begin{tikzpicture}\begin{axis}\begin{axis}\addplot coordinates {(0,0) (1,1)};\end{axis}\end{axis}\end{tikzpicture}");
    assert!(r.warnings.iter().any(|w| w.starts_with("nested axis environments are not supported")), "{:?}", r.warnings);
    // an axis beside a picture, outside one: a picture of its own
    let r = common::no_warnings(r"\tikz \draw (0,0) -- (1,0); \begin{axis}\addplot coordinates {(0,0) (1,1)};\end{axis}");
    assert!(r.description.contains("2 points"), "{}", r.description);
    // a group the body leaves open is closed with the axis
    let r = common::no_warnings(r"\begin{tikzpicture}\begin{axis}\begingroup\addplot coordinates {(0,0) (1,1)};\end{axis}\draw (0,0) -- (1,0);\end{tikzpicture}");
    assert!(r.description.contains("2 points"), "{}", r.description);
}

#[test]
fn ternary_axes_options() {
    // relative limits: percentages; a reversed direction is not drawn reversed
    assert!(warnings("ternaryaxis:::ternary limits relative", r"\addplot3 coordinates {(0.2,0.3,0.5)};").is_empty());
    let w = warnings("ternaryaxis:::x dir=reverse", r"\addplot3 coordinates {(0.2,0.3,0.5)};");
    assert_eq!(w, vec!["ternaryaxis: 'x dir=reverse' not supported: drawn forward".to_string()]);
}

#[test]
fn axis_commands() {
    // code at the axis' begin and end
    let n = texts(&drawn(r"execute at begin axis={\def\tikraname{B}}, execute at end axis={\node at (0,0) {\tikraname};}", LINE));
    assert_eq!(n, texts(&drawn("", LINE)) + 1);
    // a legend image of its own, and entries with options and groups
    let c = drawn("", r"\addlegendimage{red} \addlegendentry{r} \addplot coordinates {(0,0) (1,1)};");
    assert!(stroke_colors(&c).iter().any(|c| c == "#ff0000"));
    let legend = |body: &str| texts(&drawn("xtick=\\empty, ytick=\\empty", body));
    assert_eq!(legend(r"\addplot coordinates {(0,0)}; \addplot coordinates {(1,1)}; \legend{[red]a,{b,c}}"), 2);
    // a stray \closedcycle is nothing
    assert!(warnings("", &format!(r"{LINE} \closedcycle")).is_empty());
    // \pgfplotstransformcoordinatex: in the axis' transformed units; outside an axis, the number
    let d = common::no_warnings(r"\begin{tikzpicture}\pgfplotstransformcoordinatez{3}\node {\pgfmathresult};\end{tikzpicture}").description;
    assert_eq!(d, "Text in the figure: 3.");
    // (when the axis is drawn: in \pgfplotsextra; x is 10 times the data, 0 to 10 on 240pt)
    let d = common::figure(&picture("", &format!(r"{LINE} \pgfplotsextra{{\pgfplotstransformcoordinatex{{5}}\node at (axis cs:5,0.5) {{\pgfmathresult}};}}"))).description;
    assert!(d.ends_with("Other text: 50."), "{d}");
    // a colormap as a shading's colors
    let d = common::no_warnings(
        r"\pgfplotsset{colormap={rg}{rgb=(1,0,0) rgb=(0,1,0)}}\pgfplotscolormaptoshadingspec{rg}{100pt}\spec \begin{tikzpicture}\node {\spec};\end{tikzpicture}",
    )
    .description;
    assert_eq!(d, "Text in the figure: rgb(0pt)=(1,0,0); rgb(100pt)=(0,1,0).");
    // \tikzfillbetween of two named paths
    let c = tikra::engine::run_document(
        r"\begin{tikzpicture}\path[name path=a] (0,0) -- (1,0); \path[name path=b] (0,1) -- (1,1); \tikzfillbetween[of=a and b]{red};\end{tikzpicture}",
    )
    .unwrap()
    .0;
    assert!(c.items.iter().any(|i| matches!(i, Item::Path { fill: Some(Fill { paint: Paint::Solid(col), .. }), .. } if col.hex() == "#ff0000")));
}

// ---------------------------------------------------------------------------------------------
// \addplot graphics

/// A folder tikra reads files from, with a PNG of 40 × 20 pixels (72 dpi: as many pt) and a file
/// that is no PNG.
fn image_dir() {
    static DIR: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="40pt" height="20pt" viewBox="0 0 40 20"><rect width="40" height="20" fill="red"/></svg>"#;
        let dir = std::env::temp_dir().join(format!("tikra-pgfplots-axes-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("img.png"), tikra::png(svg, 72.0, false).unwrap()).unwrap();
        std::fs::write(dir.join("text.png"), "not an image").unwrap();
        std::env::set_var("TIKRA_INPUT_DIR", format!("{}:.", dir.display()));
    });
}

/// The canvas and warnings of an axis with an image.
fn with_image(options: &str, body: &str) -> (Canvas, Vec<String>) {
    image_dir();
    tikra::engine::run_document(&picture(options, body)).unwrap()
}

fn image_place(c: &Canvas) -> Option<tikra::scene::Transform> {
    c.items.iter().find_map(|i| if let Item::Image { place, .. } = i { Some(*place) } else { None })
}

#[test]
fn images_that_cannot_be_placed() {
    let (_, w) = with_image("", r"\addplot graphics[xmin=0, xmax=1, ymin=0, ymax=1] {text.png};");
    assert!(has(&w, "\\addplot graphics: 'text.png' is not a PNG image; only PNG images are drawn"), "{w:?}");
    // a corner without its place
    let (_, w) = with_image("", r"\addplot graphics[xmin=0, xmax=1, ymin=0] {img.png};");
    assert!(has(&w, "the key 'ymax' is missing: no place for the image"), "{w:?}");
    // includegraphics' options other than trim and clip
    let (_, w) = with_image("", r"\addplot graphics[xmin=0, xmax=1, ymin=0, ymax=1, includegraphics={scale=2}] {img.png};");
    assert!(has(&w, "includegraphics option 'scale' ignored"), "{w:?}");
    // two mapped points at the same image x
    let (c, w) = with_image("", r"\addplot graphics[points={(0,0) => (0,0) (1,1) => (0,10)}] {img.png};");
    assert!(has(&w, "the first two points with '=>' need different image x and y coordinates"), "{w:?}");
    assert!(image_place(&c).is_none());
    // one point, no corners: too few coordinates
    let (c, w) = with_image("", r"\addplot graphics[points={(0,0)}] {img.png};");
    assert!(has(&w, "too few coordinates"), "{w:?}");
    assert!(image_place(&c).is_none());
    // trimmed away entirely
    let (c, w) = with_image("", r"\addplot graphics[xmin=0, xmax=1, ymin=0, ymax=1, includegraphics={trim=30 0 30 0}] {img.png};");
    assert!(w.is_empty(), "{w:?}");
    assert!(image_place(&c).is_none());
}

#[test]
fn trimmed_and_clipped_images() {
    // the part left after trimming (in bp, or a length) fills the box, the whole image scaled
    // alike; clipped to the box
    let opt = "scale only axis, width=4cm, height=2cm, enlargelimits=false, hide axis";
    let place = |trim: &str| {
        let (c, w) = with_image(opt, &format!(r"\addplot graphics[xmin=0, xmax=1, ymin=0, ymax=1, includegraphics={{trim={trim}, clip}}] {{img.png}};"));
        assert!(w.is_empty(), "{w:?}");
        assert!(c.items.iter().any(|i| matches!(i, Item::Clip { .. })));
        image_place(&c).unwrap().a
    };
    // untrimmed, the image is the box; trimmed by t, it is 4cm · n / (n - t), n its natural width
    assert!(close(place("0 0 0 0"), 4.0, 1e-9));
    let bp = 2.54 / 72.0;
    let a = place("10 0 0 0");
    let natural = a * 10.0 * bp / (a - 4.0);
    // 40 pixels at 72 dpi
    assert!(close(natural, 40.0 * bp, 1e-6), "{natural}");
    assert!(close(place("0 0 0.5cm 0"), 4.0 * natural / (natural - 0.5), 1e-6));
}

#[test]
fn points_of_an_image() {
    // points without image coordinates count in the limits; one of two parts is no mapping
    let (c, w) = with_image("hide axis", r"\addplot graphics[points={(0,0) => (0,0) (2,1) => (40,20) (5,5) => (1) ((1+1),3)}] {img.png};");
    assert!(w.is_empty(), "{w:?}");
    assert!(image_place(&c).is_some());
    // in 3d: a point without z is not one of the axis'; fewer than four mapped points, or one
    // without z, leave the view as it is
    for points in ["(0,0,0) => (0,0) (1,0,0) => (40,5) (0,1)", "(0,0,0) => (0,0) (1,0,0) => (40,5) (0,1,0) => (5,20) (0,0) => (3,3)"] {
        let (c, w) = with_image("hide axis", &format!(r"\addplot3 graphics[points={{{points}}}] {{img.png}};"));
        assert!(w.is_empty(), "{points}: {w:?}");
        assert!(image_place(&c).is_some(), "{points}");
    }
    // four points in a plane: no view makes them so
    let (_, w) = with_image("hide axis", r"\addplot3 graphics[points={(0,0,0) => (0,0) (1,0,0) => (40,0) (2,0,0) => (80,0) (3,0,0) => (120,0)}] {img.png};");
    assert!(has(&w, "the points are not linearly independent"), "{w:?}");
    // a reversed axis keeps its direction
    let (c, w) = with_image("hide axis, x dir=reverse", r"\addplot3 graphics[points={(0,0,0) => (0,0) (1,0,0) => (40,5) (0,1,0) => (-5,15) (0,0,1) => (2,20)}] {img.png};");
    assert!(w.is_empty(), "{w:?}");
    assert!(image_place(&c).is_some());
    // a slanted z stays slanted (`snap z` only straightens a nearly vertical one)
    let (c, w) = with_image("hide axis", r"\addplot3 graphics[points={(0,0,0) => (0,0) (1,0,0) => (40,5) (0,1,0) => (-5,15) (0,0,1) => (20,20)}] {img.png};");
    assert!(w.is_empty(), "{w:?}");
    assert!(image_place(&c).is_some());
    // `=>` with no image point after it
    let (_, w) = with_image("", r"\addplot graphics[xmin=0, xmax=1, ymin=0, ymax=1, points={(0,0) => 1}] {img.png};");
    assert!(w.is_empty(), "{w:?}");
    // no options: no place
    let (_, w) = with_image("", r"\addplot graphics {img.png};");
    assert!(has(&w, "the key 'xmin' is missing"), "{w:?}");
    // \includegraphics of a file that is no PNG
    image_dir();
    let r = common::figure(r"\begin{tikzpicture}\node {\includegraphics{text.png}};\end{tikzpicture}");
    assert!(has(&r.warnings, "'text.png' is not a PNG image"), "{:?}", r.warnings);
    // an unclosed parenthesis ends the list
    let (_, w) = with_image("", r"\addplot graphics[xmin=0, xmax=1, ymin=0, ymax=1, points={(0,0) (1,}] {img.png};");
    assert!(w.is_empty(), "{w:?}");
}

// ---------------------------------------------------------------------------------------------
// more of the axis' corners

#[test]
fn coordinates_of_other_axes() {
    // a polar axis before compat 1.11: plain coordinates are TikZ's
    let p = points_in("polaraxis:::compat=1.10", r"\addplot coordinates {(0,1)}; \coordinate (p) at (1,1); \coordinate (c) at (canvas cs:x=1cm,y=1cm);");
    assert!(same(p["p"], p["c"]), "{p:?}");
    // no data scaling: plain coordinates are the data
    let p = points_in("compat=1.10, disabledatascaling", &format!(r"{LINE} \coordinate (p) at (5,0.5); \coordinate (q) at (axis cs:5,0.5);"));
    assert!(same(p["p"], p["q"]), "{p:?}");
    // TikZ's other systems in an axis
    let p = points_in("", &format!(r"{LINE} \coordinate (k) at (canvas polar cs:angle=0,radius=1cm); \coordinate (c) at (canvas cs:x=1cm,y=0cm);"));
    assert!(same(p["k"], p["c"]), "{p:?}");
    // a 3d axis of given units: z is up
    let p = points_in("x=1cm, y=1cm, z=2cm", r"\addplot3 coordinates {(0,0,0) (1,1,1)}; \coordinate (a) at (axis cs:0,0,0); \coordinate (b) at (axis cs:0,0,1);");
    assert!(close(p["a"].1 - p["b"].1, 2.0 * CM, 1e-6) && close(p["a"].0, p["b"].0, 1e-6), "{p:?}");
    // the data coordinates of a point of a 3d axis are not known: nothing is printed
    let d = common::figure(&picture(
        "",
        r"\addplot3 coordinates {(0,0,0) (1,1,1)}; \node at (axis cs:1,1,1) {\pgfplotspointgetcoordinates{(axis cs:1,1,1)}\pgfkeysvalueof{/data point/x}};",
    ))
    .description;
    assert!(!d.contains("Other text"), "{d}");
}

#[test]
fn more_tick_labels() {
    // `\pgfmathprintnumber\tick` followed by `--`: the number, then the dashes
    let w = |code: &str| widths(&format!("ytick=\\empty, xticklabel={{{code}}}"), LINE);
    assert_eq!(w(r"\pgfmathprintnumber\tick--"), w(r"\pgfmathprintnumber{\tick}--"));
    // a group inside the number's argument
    assert_eq!(w(r"\pgfmathprintnumber{{\tick}}"), w(r"\pgfmathprintnumber{\tick}"));
    // a manual scale whose code gives no number: the labels as they are, and the scale's label
    let plain = labels("scaled x ticks=false", r"\addplot coordinates {(0,0) (10000,1)};");
    assert_eq!(labels(r"scaled x ticks=manual:{L}{\def\pgfmathresult{x}}", r"\addplot coordinates {(0,0) (10000,1)};"), plain + 1);
    // `real:0` divides by nothing
    assert_eq!(labels("scaled x ticks=real:0", r"\addplot coordinates {(0,0) (10000,1)};"), plain);
    // a date axis' own labels
    let dates = r"\addplot coordinates {(2020-01-01,1) (2020-01-05,2)};";
    assert_eq!(labels("date coordinates in=x, xticklabel={\\day.\\month.}, ytick=\\empty", dates), labels("date coordinates in=x, ytick=\\empty", dates));
    // ticks hidden by a crossing axis line: at its end (`right`) or its start (`left`), none
    // without it
    let lines = |y: &str| labels(&format!("ytick=\\empty, axis x line=middle, axis y line={y}"), r"\addplot coordinates {(0,-1) (10,1)};");
    assert_eq!(lines("right"), lines("left"));
    assert_eq!(lines("none"), lines("left") + 1);
    // `every axis` in 3d: its keys count for the description
    let (c, w) = tikra::engine::run_document(
        r"\pgfplotsset{every axis/.append style={xlabel=X}}\begin{tikzpicture}\begin{axis}\addplot3 coordinates {(0,0,0) (1,1,1)};\end{axis}\end{tikzpicture}",
    )
    .unwrap();
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(texts(&c), texts(&drawn("", r"\addplot3 coordinates {(0,0,0) (1,1,1)};")) + 1);
}

// ---------------------------------------------------------------------------------------------
// `scale mode=scale uniformly` (the scaling of 3d axes)

mod uniformly {
    use super::close;
    use tikra::engine::pgfplots::scaling::{uniform, Input, KeepSize, Strategy};
    use tikra::scene::Point;

    fn p(x: f64, y: f64) -> Point {
        Point::new(x, y)
    }

    /// A unit box of these unit vectors brought to `width` × `height`: the unit vectors and the
    /// limits (from 0 to 1, those of `auto` computed from the data).
    fn scale(e: [Point; 3], (width, height): (f64, f64), strategy: Strategy, ratio: &str, ratio_axis: &str, keep: KeepSize, auto: [bool; 2]) -> ([Point; 3], [f64; 3], [f64; 3]) {
        let inp = Input { e, width, height, unit_box: true, three_d: true, strategy, ratio, ratio_axis, keep, auto_min: [auto[0]; 3], auto_max: [auto[1]; 3] };
        let (mut min, mut max) = ([0.0; 3], [1.0; 3]);
        let ev = uniform(&inp, &mut min, &mut max);
        (ev, min, max)
    }

    const FLAT: [Point; 3] = [Point { x: 1.0, y: 0.0 }, Point { x: 0.0, y: 1.0 }, Point { x: 0.0, y: 0.0 }];

    #[test]
    fn units_only() {
        // one scale that fits the box, the limits as they are
        let (ev, min, max) = scale(FLAT, (10.0, 5.0), Strategy::UnitsOnly, "", "", KeepSize::True, [true, true]);
        assert!(close(ev[0].x, 5.0, 1e-9) && close(ev[1].y, 5.0, 1e-9), "{ev:?}");
        assert_eq!((min, max), ([0.0; 3], [1.0; 3]));
        let (ev, ..) = scale(FLAT, (4.0, 5.0), Strategy::UnitsOnly, "", "", KeepSize::True, [true, true]);
        assert!(close(ev[0].x, 4.0, 1e-9), "{ev:?}");
    }

    #[test]
    fn the_strategies_change_the_limits_of_one_direction() {
        // twice as wide as high: x grows to twice its range (the computed side, or both)
        let (ev, min, max) = scale(FLAT, (10.0, 5.0), Strategy::ChangeHorizontal, "", "", KeepSize::True, [false, true]);
        assert!(close(ev[0].x, 5.0, 1e-9) && close(max[0] - min[0], 2.0, 1e-9) && min[0] == 0.0, "{ev:?} {min:?} {max:?}");
        let (_, min, max) = scale(FLAT, (10.0, 5.0), Strategy::ChangeHorizontal, "", "", KeepSize::True, [true, false]);
        assert!(close(min[0], -1.0, 1e-9) && max[0] == 1.0, "{min:?} {max:?}");
        // limits given on both sides: kept (`unless limits declared`), or grown around the center
        let (_, min, max) = scale(FLAT, (10.0, 5.0), Strategy::ChangeHorizontal, "", "", KeepSize::UnlessLimitsDeclared, [false, false]);
        assert_eq!((min[0], max[0]), (0.0, 1.0));
        let (_, min, max) = scale(FLAT, (10.0, 5.0), Strategy::ChangeHorizontal, "", "", KeepSize::True, [false, false]);
        assert!(close(min[0], -0.5, 1e-9) && close(max[0], 1.5, 1e-9), "{min:?} {max:?}");
        // twice as high: y grows
        let (ev, min, max) = scale(FLAT, (5.0, 10.0), Strategy::ChangeVertical, "", "", KeepSize::True, [true, true]);
        assert!(close(ev[0].x, 5.0, 1e-9) && close(max[1] - min[1], 2.0, 1e-9), "{ev:?} {min:?} {max:?}");
        // a slanted y (or z) has no horizontal or vertical limits to change: units only
        let slanted = [p(1.0, 0.0), p(0.5, 0.5), p(0.0, 0.0)];
        let skew = [p(1.0, 0.0), p(0.0, 1.0), p(0.5, 0.5)];
        for (e, strategy) in [(slanted, Strategy::ChangeVertical), (slanted, Strategy::ChangeHorizontal), (skew, Strategy::ChangeVertical), (skew, Strategy::ChangeHorizontal)] {
            let (_, min, max) = scale(e, (10.0, 5.0), strategy, "", "", KeepSize::True, [true, true]);
            assert_eq!((min, max), ([0.0; 3], [1.0; 3]), "{e:?} {strategy:?}");
        }
    }

    #[test]
    fn unit_vector_ratios() {
        // the unit vectors' lengths in the ratio asked, whichever axis is the reference
        let cube = [p(1.0, -0.3), p(0.6, 0.4), p(0.0, 1.0)];
        for axis in ["", "x", "y", "z"] {
            let (ev, ..) = scale(cube, (10.0, 10.0), Strategy::Auto, "1 2 3", axis, KeepSize::True, [true, true]);
            let len: Vec<f64> = (0..3).map(|a| ev[a].norm() / cube[a].norm()).collect();
            assert!(close(len[1] / len[0], 2.0, 1e-9) && close(len[2] / len[0], 3.0, 1e-9), "{axis}: {len:?}");
        }
        // an axis without a unit vector keeps none
        let (ev, ..) = scale([p(1.0, 0.0), p(0.0, 1.0), p(0.0, 0.0)], (10.0, 10.0), Strategy::Auto, "1 2", "", KeepSize::True, [true, true]);
        assert!(close(ev[1].norm() / ev[0].norm(), 2.0, 1e-9) && ev[2].norm() == 0.0, "{ev:?}");
    }
}
