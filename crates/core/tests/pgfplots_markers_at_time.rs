//! pgfplots: how one plot is drawn — its plot handler (steps, jumps, bars, combs, intervals, ties,
//! arrows, polar combs), `\closedcycle` and the path going on after it, error bars, its marks
//! (`ball`, `cube`, scatter marks, `mark indices`), `nodes near coords`, and nodes along it
//! (`\addplot … node[pos=t] {…};`, `\pgfplotspointplotattime`).

mod common;
use common::*;
use tikra::canvas::{Canvas, Paint, Shading};
use tikra::scene::{Point, Segment};

fn axis(options: &str, body: &str) -> (Canvas, Vec<String>) {
    environment("axis", options, body)
}

fn environment(env: &str, options: &str, body: &str) -> (Canvas, Vec<String>) {
    tikra::engine::run_document(&format!("\\begin{{tikzpicture}}\\begin{{{env}}}[{options}] {body} \\end{{{env}}}\\end{{tikzpicture}}")).unwrap_or_else(|e| panic!("{e}\n{body}"))
}

/// The paths stroked or filled in `color` (as `#rrggbb`).
fn in_color(c: &Canvas, color: &str) -> Vec<Drawn> {
    paths(c)
        .into_iter()
        .filter(|d| d.stroke.as_ref().is_some_and(|s| s.color.hex() == color) || matches!(&d.fill, Some(f) if matches!(f.paint, Paint::Solid(k) if k.hex() == color)))
        .collect()
}

const RED: &str = "#ff0000";
const BLUE: &str = "#0000ff";
const GREEN: &str = "#00ff00";

/// The points a path goes through (Close has none).
fn pts(d: &Drawn) -> Vec<Point> {
    points(&d.segments).into_iter().map(|(x, y)| Point::new(x, y)).collect()
}

fn same(a: Point, b: Point) -> bool {
    a.minus(b).norm() < 1e-6
}

/// The texts of a canvas that are no tick labels (those of the plot's nodes, and of the reference
/// nodes after them): their width, where they start and their rotation (the transform's `b`).
fn node_texts(c: &Canvas, ticks: usize) -> Vec<(f64, Point, f64)> {
    texts(c).into_iter().skip(ticks).map(|(l, t)| (l.width(), Point::new(t.e, t.f), t.b)).collect()
}

/// The number of tick labels of an axis with these options and a plot of these coordinates.
fn tick_labels(options: &str, plot: &str) -> usize {
    texts(&axis(options, &format!(r"\addplot coordinates {{{plot}}};")).0).len()
}

/// Asserts that the plot's nodes (the first half of the non-tick texts) are drawn where the
/// reference nodes (the second half) are, with the same width.
fn same_places(options: &str, plot: &str, nodes: &str, references: &str) -> Vec<(f64, Point, f64)> {
    let ticks = tick_labels(options, plot);
    let (c, w) = axis(options, &format!(r"\addplot coordinates {{{plot}}} {nodes}; {references}"));
    assert!(w.is_empty(), "{w:?}");
    let t = node_texts(&c, ticks);
    assert!(!t.is_empty() && t.len().is_multiple_of(2), "{t:?}");
    let n = t.len() / 2;
    for k in 0..n {
        let (a, b) = (t[k], t[n + k]);
        assert!(close(a.0, b.0) && a.1.minus(b.1).norm() < 1e-6, "node {k}: {a:?} is not at {b:?}\n{nodes}");
    }
    t
}

#[test]
fn pos_is_a_fraction_of_the_plots_length() {
    // two sides of equal length: a quarter of the way is the middle of the first
    same_places(
        "",
        "(0,0) (2,0) (2,2)",
        r"node[pos=0.25] {A} node[pos=0.75] {B} node[pos=0.6] {C}",
        r"\node at (axis cs:1,0) {A}; \node at (axis cs:2,1) {B}; \node at (axis cs:2,0.4) {C};",
    );
}

#[test]
fn the_named_positions_are_fractions_too() {
    let cases = [("at start", "0,0"), ("very near start", "0.5,0"), ("near start", "1,0"), ("midway", "2,0"), ("near end", "2,1"), ("very near end", "2,1.5"), ("at end", "2,2")];
    for (key, at) in cases {
        same_places("", "(0,0) (2,0) (2,2)", &format!("node[{key}] {{A}}"), &format!(r"\node at (axis cs:{at}) {{A}};"));
    }
}

#[test]
fn beyond_the_end_the_last_segment_goes_on() {
    same_places("", "(0,0) (1,0) (2,0)", r"node[pos=1.5] {A}", r"\node at (axis cs:3,0) {A};");
}

#[test]
fn a_plot_of_one_point_has_its_nodes_there() {
    same_places("xmin=0, xmax=2, ymin=0, ymax=2", "(1,1)", r"node[pos=0.5] {A}", r"\node at (axis cs:1,1) {A};");
}

#[test]
fn nodes_without_pos_go_at_the_last_point() {
    // a node without `pos`, and one with `pos` beside it
    same_places("", "(0,0) (2,0) (2,2)", r"node[pos=0.25] {A} node[above] {B}", r"\node at (axis cs:1,0) {A}; \node[above] at (axis cs:2,2) {B};");
}

#[test]
fn options_before_the_nodes_move_them() {
    same_places("", "(0,0) (2,0) (2,2)", r"[xshift=10pt] node[pos=0.25] {A}", r"\node[xshift=10pt] at (axis cs:1,0) {A};");
}

#[test]
fn a_coordinate_along_the_plot_can_be_used_later() {
    same_places("", "(0,0) (2,0) (2,2)", r"coordinate[pos=0.25] (m)", r"\node at (m) {A}; \node at (axis cs:1,0) {A};");
}

#[test]
fn sloped_nodes_follow_their_segment() {
    // the node on the rising segment is turned, the one on the flat segment is not
    let ticks = tick_labels("", "(0,0) (2,0) (4,2)");
    let (c, _) = axis("", r"\addplot coordinates {(0,0) (2,0) (4,2)} node[pos=0.2, sloped] {A} node[pos=0.8, sloped] {B};");
    let t = node_texts(&c, ticks);
    assert_eq!(t.len(), 2);
    assert!(close(t[0].2, 0.0), "{t:?}");
    assert!(t[1].2 > 0.3, "{t:?}");
}

#[test]
fn pos_segment_measures_one_piece_of_the_plot() {
    // the second piece (after the empty line) is from (4,0) to (4,4): its middle
    same_places("", "(0,0) (2,0)\n\n(4,0) (4,4)", r"[pos segment=1] node[pos=0.5] {A}", r"\node at (axis cs:4,2) {A};");
}

#[test]
fn nodes_of_a_scatter_plot_go_to_the_nearest_point() {
    let plot = "(0,0) (1,0) (2,0)";
    let ticks = texts(&axis("", &format!(r"\addplot[only marks] coordinates {{{plot}}};")).0).len();
    let (c, _) = axis("", &format!(r"\addplot[only marks] coordinates {{{plot}}} node[pos=0.4] {{A}}; \node at (axis cs:1,0) {{A}};"));
    let t = node_texts(&c, ticks);
    assert_eq!(t.len(), 2);
    assert!(t[0].1.minus(t[1].1).norm() < 1e-6, "{t:?}");
}

#[test]
fn the_point_at_time_sets_the_data_point() {
    // the node's own `pos`, or the time given to it: the text is the coordinate there
    same_places(
        "",
        "(0,0) (2,0) (2,2)",
        r"node[pos=0.25] {\pgfplotspointplotattime \pgfkeysvalueof{/data point/x}} node[pos=0.25] {\pgfplotspointplotattime{{0.75}}\pgfkeysvalueof{/data point/y}}",
        r"\node at (axis cs:1,0) {1}; \node at (axis cs:1,0) {1};",
    );
}

#[test]
fn the_point_at_time_on_a_log_axis_and_with_symbolic_coordinates() {
    // the middle of 1 to 100 on a log axis is 10
    same_places("ymode=log", "(1,1) (1,100)", r"node[pos=0.5] {\pgfplotspointplotattime\pgfkeysvalueof{/data point/y}}", r"\node at (axis cs:1,10) {10};");
    // a symbolic coordinate is told by its name
    same_places("symbolic x coords={a,b,c}", "(a,0) (c,0)", r"node[pos=0.5] {\pgfplotspointplotattime\pgfkeysvalueof{/data point/x}}", r"\node at (axis cs:b,0) {b};");
}

#[test]
fn the_point_at_time_outside_a_plot_warns() {
    with_warning(r"\begin{tikzpicture}\node {\pgfplotspointplotattime x};\end{tikzpicture}", "outside the nodes of a plot");
}

// --- plot handlers ---

#[test]
fn a_smooth_plot_of_two_points_is_a_line() {
    let (c, _) = axis("", r"\addplot[smooth, red] coordinates {(0,0) (1,1)};");
    let p = &in_color(&c, RED)[0];
    assert!(matches!(p.segments[..], [Segment::MoveTo(_), Segment::LineTo(_)]), "{:?}", p.segments);
}

#[test]
fn steps_and_jumps_between_the_points() {
    let data = "(0,0) (2,1) (4,0)";
    for handler in ["const plot mark mid", "jump mark left", "jump mark right", "jump mark mid"] {
        let (c, _) = axis("", &format!(r"\addplot[red, sharp plot] coordinates {{{data}}}; \addplot[blue, {handler}] coordinates {{{data}}};"));
        let p = pts(&in_color(&c, RED)[0]);
        let s = &in_color(&c, BLUE)[0].segments;
        let m = |a: Point, b: Point| (a.x + b.x) / 2.0;
        let want: Vec<Segment> = match handler {
            "const plot mark mid" => vec![
                Segment::MoveTo(p[0]),
                Segment::LineTo(Point::new(m(p[0], p[1]), p[0].y)),
                Segment::LineTo(Point::new(m(p[0], p[1]), p[1].y)),
                Segment::LineTo(p[1]),
                Segment::LineTo(Point::new(m(p[1], p[2]), p[1].y)),
                Segment::LineTo(Point::new(m(p[1], p[2]), p[2].y)),
                Segment::LineTo(p[2]),
            ],
            // the step is drawn, the riser is a jump
            "jump mark left" => {
                vec![Segment::MoveTo(p[0]), Segment::LineTo(Point::new(p[1].x, p[0].y)), Segment::MoveTo(p[1]), Segment::LineTo(Point::new(p[2].x, p[1].y)), Segment::MoveTo(p[2])]
            }
            "jump mark right" => {
                vec![Segment::MoveTo(p[0]), Segment::MoveTo(Point::new(p[0].x, p[1].y)), Segment::LineTo(p[1]), Segment::MoveTo(Point::new(p[1].x, p[2].y)), Segment::LineTo(p[2])]
            }
            _ => vec![
                Segment::MoveTo(p[0]),
                Segment::LineTo(Point::new(m(p[0], p[1]), p[0].y)),
                Segment::MoveTo(Point::new(m(p[0], p[1]), p[1].y)),
                Segment::LineTo(p[1]),
                Segment::LineTo(Point::new(m(p[1], p[2]), p[1].y)),
                Segment::MoveTo(Point::new(m(p[1], p[2]), p[2].y)),
                Segment::LineTo(p[2]),
            ],
        };
        assert_eq!(s.len(), want.len(), "{handler}: {s:?}");
        for (a, b) in s.iter().zip(&want) {
            let ok = match (a, b) {
                (Segment::MoveTo(x), Segment::MoveTo(y)) | (Segment::LineTo(x), Segment::LineTo(y)) => same(*x, *y),
                _ => false,
            };
            assert!(ok, "{handler}: {a:?} is not {b:?}");
        }
    }
}

#[test]
fn bars_combs_and_intervals_skip_unbounded_points() {
    // with `unbounded coords=jump` the point at nan stays in the plot, but has no bar, comb or interval
    let data = "(0,1) (1,nan) (2,2) (3,1)";
    let count = |handler: &str| {
        let (c, w) = axis("unbounded coords=jump", &format!(r"\addplot[red, {handler}] coordinates {{{data}}};"));
        assert!(w.is_empty(), "{w:?}");
        let s = &in_color(&c, RED)[0].segments;
        (s.iter().filter(|s| matches!(s, Segment::MoveTo(_))).count(), s.iter().filter(|s| matches!(s, Segment::Close)).count())
    };
    assert_eq!(count("ybar"), (3, 3));
    assert_eq!(count("ycomb"), (3, 0));
    assert_eq!(count("xcomb"), (3, 0));
    // an interval needs both its ends: only the one from 2 to 3
    assert_eq!(count("ybar interval"), (1, 1));
}

#[test]
fn xbar_intervals_go_from_the_zero_level_sideways() {
    let (c, _) = axis("", r"\addplot[red, sharp plot] coordinates {(1,0) (2,1) (3,2)}; \addplot[blue, xbar interval] coordinates {(1,0) (2,1) (3,2)};");
    let p = pts(&in_color(&c, RED)[0]);
    let s = &in_color(&c, BLUE)[0];
    let q = pts(s);
    // two bars, from y=0 to 1 and 1 to 2, as long as their x
    assert_eq!(q.len(), 8);
    assert!(close(q[0].y, p[0].y) && close(q[2].y, p[1].y) && close(q[2].x, p[0].x), "{q:?}");
    assert!(close(q[4].y, p[1].y) && close(q[6].y, p[2].y) && close(q[6].x, p[1].x), "{q:?}");
    // both from the zero level of x
    assert!(close(q[0].x, q[4].x));
}

#[test]
fn ties_join_each_point_to_its_other_end() {
    // each row: a point (a, b) and its tie's other end (d, e), three columns on
    let table = "a b c d e f\n0 0 0 1 1 0\n2 0 0 3 1 0\n4 0 0 5 1 0\n";
    let (c, w) =
        axis("", &format!("\\addplot[red, sharp plot] coordinates {{(0,0) (2,0) (4,0) (5,1) (3,1) (1,1)}}; \\addplot[blue, tieline] table[tie end z index=5] {{{table}}};"));
    assert!(w.is_empty(), "{w:?}");
    let reference = pts(&in_color(&c, RED)[0]);
    let blue = in_color(&c, BLUE);
    // the curve through the points, then the other ends backwards
    let curve = pts(&blue[0]);
    assert_eq!(curve.len(), 6);
    assert!(curve.iter().zip(&reference).all(|(a, b)| same(*a, *b)), "{curve:?}");
    // a tie per row, from its other end to the point
    let ties = &blue[1].segments;
    assert_eq!(ties.len(), 6);
    assert!(matches!(ties[0], Segment::MoveTo(p) if same(p, reference[5])) && matches!(ties[1], Segment::LineTo(p) if same(p, reference[0])), "{ties:?}");
    // every second tie only
    let (c, _) = axis("", &format!("\\addplot[blue, tieline={{each nth tie=2}}] table[tie end z index=5] {{{table}}};"));
    assert_eq!(in_color(&c, BLUE)[1].segments.len(), 2);
}

#[test]
fn quiver_arrows_in_their_color_without_unbounded_ones() {
    let (c, w) = axis("", "\\addplot[red, -, quiver={u=\\thisrow{u}, v=0, colored=blue}] table {x y u\n0 0 1\n0 1 nan\n0 2 2\n};");
    assert!(w.is_empty(), "{w:?}");
    // an arrow per bounded point, in the arrow color, none in the plot's
    assert_eq!(in_color(&c, BLUE).len(), 2);
    assert!(in_color(&c, RED).is_empty());
}

#[test]
fn polar_combs_are_rays_from_the_center() {
    let (c, w) = environment("polaraxis", "", r"\addplot[polar comb, red] coordinates {(0,1) (90,2)};");
    assert!(w.is_empty(), "{w:?}");
    let s = &in_color(&c, RED)[0];
    let p = pts(s);
    assert_eq!(p.len(), 4);
    // both start at the center; the one at 0° goes right, the one at 90° up, twice as long
    assert!(same(p[0], p[2]));
    let (a, b) = (p[1].minus(p[0]), p[3].minus(p[2]));
    assert!(a.x > 0.0 && close(a.y, 0.0) && close(b.x, 0.0) && (b.y - 2.0 * a.x).abs() < 1e-6, "{p:?}");
}

#[test]
fn a_stacked_step_plot_closes_along_the_one_below() {
    // the second plot's `\closedcycle` goes back along the first, with its steps
    let (c, _) = axis(
        "stack plots=y",
        r"\addplot[red, const plot mark mid] coordinates {(0,1) (1,1) (2,1)} \closedcycle; \addplot[blue, const plot mark mid] coordinates {(0,1) (1,2) (2,1)} \closedcycle;",
    );
    let top = pts(&in_color(&c, RED)[0]);
    let s = &in_color(&c, BLUE)[0];
    assert!(matches!(s.segments.last(), Some(Segment::Close)));
    // forward 7 points, then back over the lower plot's 7 (its middles included)
    let p = pts(s);
    assert_eq!(p.len(), 14);
    for k in 0..7 {
        assert!(same(p[7 + k], top[6 - k]), "{k}: {:?} {:?}", p[7 + k], top[6 - k]);
    }
}

#[test]
fn the_path_goes_on_after_the_plot() {
    // `--`, `|-` and `-|` after the bars (the path closed by its last bar)
    let (c, _) = axis("", r"\addplot[red, ybar] coordinates {(1,1) (2,2)} -- (axis cs:3,0) |- (axis cs:0,1) -| (axis cs:1,3);");
    let s = &in_color(&c, RED)[0].segments;
    let n = s.len();
    let at = |k: usize| match s[k] {
        Segment::LineTo(p) => p,
        other => panic!("{other:?}"),
    };
    assert!(matches!(s[n - 6], Segment::Close));
    let (a, b, c2) = (at(n - 5), at(n - 3), at(n - 1));
    assert!(same(at(n - 4), Point::new(a.x, b.y)) && same(at(n - 2), Point::new(c2.x, b.y)), "{s:?}");
    // `-- cycle` closes the line
    let (c, _) = axis("", r"\addplot[red] coordinates {(1,1) (2,2) (3,1)} -- cycle;");
    assert!(matches!(in_color(&c, RED)[0].segments.last(), Some(Segment::Close)));
}

#[test]
fn postactions_draw_the_plot_again() {
    let (c, _) = axis("", r"\addplot[red, postaction={draw=blue}] coordinates {(1,1) (2,2)};");
    let (r, b) = (in_color(&c, RED), in_color(&c, BLUE));
    assert_eq!((r.len(), b.len()), (1, 1));
    assert!(pts(&r[0]).iter().zip(pts(&b[0])).all(|(x, y)| same(*x, y)));
}

#[test]
fn code_at_the_beginning_of_the_plots_visualization_runs() {
    let (c, _) = axis("", r"\addplot[red, execute at begin plot visualization={\draw[green] (axis cs:1,2) -- (axis cs:2,1);}] coordinates {(1,1) (2,2)};");
    assert_eq!(in_color(&c, GREEN).len(), 1);
}

#[test]
fn one_dimensional_patches_without_point_meta_are_one_path() {
    // two lines meeting at (1,1): one path, closed down to the zero level by `\closedcycle`
    let (c, _) = axis(
        "",
        r"\addplot[red, sharp plot] coordinates {(0,0) (1,1) (2,0)}; \addplot[patch, patch type=line, point meta=none, blue] coordinates {(0,0) (1,1) (1,1) (2,0)} \closedcycle;",
    );
    let reference = pts(&in_color(&c, RED)[0]);
    let s = &in_color(&c, BLUE);
    assert_eq!(s.len(), 1);
    let p = pts(&s[0]);
    assert_eq!(s[0].segments.iter().filter(|x| matches!(x, Segment::MoveTo(_))).count(), 1);
    assert!(same(p[0], reference[0]) && same(p[1], reference[1]) && same(p[2], reference[2]), "{p:?}");
    assert!(matches!(s[0].segments.last(), Some(Segment::Close)));
    // two splines apart: two pieces
    let (c, _) = axis("", r"\addplot[patch, patch type=quadratic spline, point meta=none, blue] coordinates {(0,0) (2,0) (1,1) (3,0) (5,0) (4,1)};");
    let s = &in_color(&c, BLUE)[0].segments;
    assert_eq!(s.iter().filter(|x| matches!(x, Segment::MoveTo(_))).count(), 2);
    assert_eq!(s.iter().filter(|x| matches!(x, Segment::CurveTo(..))).count(), 2);
    // with marks, at each of the patches' points
    let (c, _) = axis("", r"\addplot[patch, patch type=line, point meta=none, blue, mark=*] coordinates {(0,0) (1,1) (1,1) (2,0)};");
    assert_eq!(in_color(&c, BLUE).into_iter().filter(|d| d.fill.is_some()).count(), 4);
}

#[test]
fn fill_between_smooth_plots_walks_one_backwards() {
    let (c, w) = axis(
        "",
        r"\addplot[smooth, name path=a] coordinates {(0,0) (1,1) (2,0)}; \addplot[smooth, name path=b] coordinates {(0,-1) (1,-2) (2,-1)}; \addplot[green] fill between[of=a and b];",
    );
    assert!(w.is_empty(), "{w:?}");
    let f = &in_color(&c, GREEN)[0];
    assert!(f.segments.iter().filter(|s| matches!(s, Segment::CurveTo(..))).count() >= 4, "{:?}", f.segments);
}

// --- error bars ---

/// The error bars' path of a plot (its segments in pairs), and the green reference lines.
fn error_bars(options: &str, plot: &str, reference: &str) -> (Vec<Segment>, Vec<Vec<Point>>) {
    let (c, w) = axis("", &format!(r"\addplot[blue, mark=none, error bars/error bar style={{red}}, {options}] {plot}; {reference}"));
    assert!(w.is_empty(), "{w:?}");
    let red = in_color(&c, RED);
    (red.first().map(|d| d.segments.clone()).unwrap_or_default(), in_color(&c, GREEN).iter().map(pts).collect())
}

#[test]
fn error_bars_relative_to_the_value() {
    // 50% of 2 below and above it
    let (s, r) = error_bars("error bars/.cd, y dir=both, y explicit relative", r"coordinates {(1,2) +- (0,0.5)}", r"\draw[green] (axis cs:1,1) -- (axis cs:1,3);");
    assert!(matches!((s[0], s[1]), (Segment::MoveTo(a), Segment::LineTo(b)) if same(a, r[0][0]) && same(b, r[0][1])), "{s:?}");
    // a fixed 10% of each value
    let (s, r) = error_bars("error bars/.cd, y dir=plus, y fixed relative=0.1", r"coordinates {(1,2) (2,4)}", r"\draw[green] (axis cs:2,4) -- (axis cs:2,4.4);");
    assert!(matches!((s[2], s[3]), (Segment::MoveTo(a), Segment::LineTo(b)) if same(a, r[0][0]) && same(b, r[0][1])), "{s:?}");
}

#[test]
fn points_without_an_error_have_no_bar() {
    // an unbounded point and a point of no error: one bar (and its two ends' marks)
    let (s, _) = error_bars("unbounded coords=jump, error bars/.cd, y dir=both, y explicit", r"coordinates {(0,1) +- (0,1) (1,nan) +- (0,1) (2,1) +- (0,0)}", "");
    assert_eq!(s.len(), 6, "{s:?}");
    // no error at all: no path
    let (s, _) = error_bars("error bars/.cd, y dir=both, y fixed=0", r"coordinates {(0,1) (1,2)}", "");
    assert!(s.is_empty(), "{s:?}");
}

// --- marks ---

fn marks_of(options: &str, plot: &str) -> (Vec<Drawn>, Vec<String>) {
    let (c, w) = axis("", &format!(r"\addplot[{options}] {plot};"));
    (paths(&c).into_iter().filter(|d| d.segments.len() > 3 && d.segments.iter().any(|s| matches!(s, Segment::CurveTo(..)))).collect(), w)
}

#[test]
fn only_marks_without_a_mark_draws_dots() {
    let (m, _) = marks_of("red, only marks, mark=none", "coordinates {(1,1) (2,2)}");
    assert_eq!(m.len(), 2);
    assert!(m.iter().all(|d| d.fill.is_some() && d.stroke.is_some()));
}

#[test]
fn filled_marks_and_balls() {
    // `heart` is filled, not stroked
    let (m, _) = marks_of("red, mark=heart", "coordinates {(1,1) (2,2)}");
    assert_eq!(m.len(), 2);
    assert!(m.iter().all(|d| d.fill.is_some() && d.stroke.is_none()));
    // a ball is shaded around
    let (m, _) = marks_of("red, mark=ball", "coordinates {(1,1) (2,2)}");
    assert_eq!(m.len(), 2);
    assert!(m.iter().all(|d| matches!(&d.fill, Some(f) if matches!(f.paint, Paint::Shading(Shading::Radial { .. })))));
    // in a scatter plot, each in its mapped color
    let (m, _) = marks_of("scatter, only marks, mark=ball, scatter/use mapped color={ball color=mapped color}", "coordinates {(1,1) (2,2)}");
    let stops = |d: &Drawn| match &d.fill {
        Some(f) => match &f.paint {
            Paint::Shading(Shading::Radial { stops, .. }) => stops.iter().map(|s| s.1.hex()).collect::<Vec<_>>(),
            _ => panic!(),
        },
        None => panic!(),
    };
    assert_eq!(m.len(), 2);
    assert_ne!(stops(&m[0]), stops(&m[1]));
}

#[test]
fn an_unknown_mark_is_a_dot() {
    let (m, w) = marks_of("red, mark=nonsense", "coordinates {(1,1) (2,2)}");
    assert!(w.iter().any(|w| w.contains("mark 'nonsense' not supported")), "{w:?}");
    assert_eq!(m.len(), 2);
}

#[test]
fn mark_indices_choose_the_points() {
    let (c, _) = axis("", r"\addplot[red, sharp plot] coordinates {(1,1) (2,2) (3,3)}; \addplot[blue, mark=*, mark indices={1,3}] coordinates {(1,1) (2,2) (3,3)};");
    let p = pts(&in_color(&c, RED)[0]);
    let marks: Vec<Drawn> = in_color(&c, BLUE).into_iter().filter(|d| d.fill.is_some()).collect();
    assert_eq!(marks.len(), 2);
    let center = |d: &Drawn| {
        let (a, b) = bounds(&d.segments);
        Point::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)
    };
    assert!(same(center(&marks[0]), p[0]) && same(center(&marks[1]), p[2]));
}

#[test]
fn cubes_in_a_three_dimensional_axis() {
    // a wire cube: two faces and four edges, a path per point
    let (c, _) = axis("", r"\addplot3[red, mark=cube] coordinates {(1,1,1) (2,2,2)};");
    let cubes: Vec<Drawn> = in_color(&c, RED).into_iter().skip(1).collect();
    assert_eq!(cubes.len(), 2);
    assert_eq!(cubes[0].segments.len(), 18);
    assert!(cubes[0].fill.is_none());
    // `cube*`: the three faces in front, filled
    let (c, _) = axis("", r"\addplot3[red, mark=cube*] coordinates {(1,1,1) (2,2,2)};");
    let faces: Vec<Drawn> = in_color(&c, RED).into_iter().filter(|d| d.fill.is_some()).collect();
    assert_eq!(faces.len(), 6);
    // in a scatter plot, each cube in its mapped color, its edges darker
    let (c, _) = axis("", r"\addplot3[scatter, only marks, mark=cube*] coordinates {(1,1,1) (2,2,2)};");
    let faces: Vec<Drawn> = paths(&c).into_iter().filter(|d| d.fill.is_some()).collect();
    let fill = |d: &Drawn| match &d.fill.as_ref().unwrap().paint {
        Paint::Solid(c) => *c,
        _ => panic!(),
    };
    assert_eq!(faces.len(), 6);
    assert_eq!(fill(&faces[0]).hex(), fill(&faces[2]).hex());
    assert_ne!(fill(&faces[0]).hex(), fill(&faces[3]).hex());
    let edge = faces[0].stroke.as_ref().unwrap().color;
    assert!(close(edge.r, 0.8 * fill(&faces[0]).r) && close(edge.b, 0.8 * fill(&faces[0]).b));
}

#[test]
fn scatter_marks_with_a_coordinate_style() {
    // the styled mark is drawn after the others, as its style says, in its mapped color
    let (c, w) = axis("", r"\addplot[scatter, only marks, coordinate style/.condition={x>1}{mark=square*}] coordinates {(1,1) (2,2)};");
    assert!(w.is_empty(), "{w:?}");
    let filled: Vec<Drawn> = paths(&c).into_iter().filter(|d| d.fill.is_some()).collect();
    assert_eq!(filled.len(), 2);
    assert!(filled[1].segments.iter().all(|s| !matches!(s, Segment::CurveTo(..))) && filled[0].segments.iter().any(|s| matches!(s, Segment::CurveTo(..))));
    // a shift in the style moves the mark
    let (c, _) = axis(
        "",
        r"\addplot[scatter, only marks, coordinate style/.condition={x>1}{xshift=10pt}] coordinates {(1,1) (2,2)}; \addplot[red, only marks, mark=*] coordinates {(1,1) (2,2)};",
    );
    // (the marks of the first plot, then those of the second)
    let filled: Vec<Drawn> = paths(&c).into_iter().filter(|d| d.fill.is_some()).collect();
    let (moved, plain) = (&filled[1], &filled[3]);
    let (a, b) = (bounds(&moved.segments).0, bounds(&plain.segments).0);
    assert!((a.x - b.x - 10.0 * tikra::PT).abs() < 1e-6 && close(a.y, b.y), "{a:?} {b:?}");
    // with `scatter/classes` the class colors the styled mark, not the colormap
    let (c, w) = axis(
        "",
        r"\addplot[scatter, only marks, point meta=explicit symbolic, scatter/classes={a={red}, b={blue}}, coordinate style/.condition={x>1}{mark=square*}] coordinates {(1,1) [a] (2,2) [b]};",
    );
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(in_color(&c, BLUE).into_iter().filter(|d| d.fill.is_some()).count(), 1);
}

// --- nodes near coords ---

/// The texts after the tick labels.
fn near_texts(options: &str, plot: &str, more: &str) -> Vec<(f64, Point, String)> {
    let ticks = texts(&axis(options, &format!(r"\addplot coordinates {{{plot}}};")).0).len();
    let (c, w) = axis(options, &format!(r"\addplot[{more}] coordinates {{{plot}}}; "));
    assert!(w.is_empty(), "{w:?}");
    texts(&c).into_iter().skip(ticks).map(|(l, t)| (l.width(), Point::new(t.e, t.f), l.color.hex())).collect()
}

#[test]
fn nodes_near_coords_aligned_as_asked() {
    let at = |align: &str| near_texts("", "(1,1) (2,2)", &format!("nodes near coords, nodes near coords align={{{align}}}"))[0].1.y;
    let (above, center, below) = (at("vertical"), at("center"), at("below"));
    assert!(above > center && center > below, "{above} {center} {below}");
}

#[test]
fn nodes_near_coords_without_text_are_none() {
    assert!(near_texts("", "(1,1) (2,2)", "nodes near coords={}").is_empty());
    // nor at an unbounded point
    assert_eq!(near_texts("unbounded coords=jump", "(1,1) (2,nan) (3,2)", "nodes near coords").len(), 2);
}

#[test]
fn nodes_near_coords_see_their_points_macros_and_style() {
    // `visualization depends on`: ten times y, "10" and "20"
    let t = near_texts("", "(1,1) (2,2)", r"visualization depends on={y*10 \as \ten}, nodes near coords=\ten");
    let ten = near_texts("", "(1,1) (2,2)", "nodes near coords=10");
    assert_eq!(t.len(), 2);
    assert!(close(t[0].0, ten[0].0) && close(t[1].0, ten[0].0));
    // `coordinate style`: the second node in red
    let t = near_texts("", "(1,1) (2,2)", r"nodes near coords, coordinate style/.condition={x>1}{red}");
    assert_eq!((t[0].2.as_str(), t[1].2.as_str()), ("#000000", RED));
}

#[test]
fn nodes_near_stacked_bars_are_in_their_middle() {
    // since compat 1.9, in the middle of each bar, centered
    for (handler, plots, reference) in [
        ("ybar stacked", "(1,2)", r"\node at (axis cs:1,1) {2}; \node at (axis cs:1,3) {2};"),
        ("xbar stacked", "(2,1)", r"\node at (axis cs:1,1) {2}; \node at (axis cs:3,1) {2};"),
    ] {
        let opts = format!("compat=1.9, {handler}, nodes near coords");
        let ticks = texts(&axis(&format!("compat=1.9, {handler}"), &format!(r"\addplot coordinates {{{plots}}}; \addplot coordinates {{{plots}}};")).0).len();
        let (c, w) = axis(&opts, &format!(r"\addplot coordinates {{{plots}}}; \addplot coordinates {{{plots}}}; {reference}"));
        assert!(w.is_empty(), "{w:?}");
        let t: Vec<(f64, Point)> = texts(&c).into_iter().skip(ticks).map(|(l, t)| (l.width(), Point::new(t.e, t.f))).collect();
        assert_eq!(t.len(), 4, "{handler}");
        // each node is where a reference node is, and no two at the same place
        assert!(t.iter().all(|a| t.iter().filter(|b| same(a.1, b.1)).count() == 2), "{handler}: {t:?}");
    }
}

#[test]
fn before_compat_1_9_stacked_nodes_show_the_stacked_value() {
    // `point meta=x` of x stacked bars: the second plot's node says 4 (2 on top of 2)
    let opts = "compat=1.8, xbar stacked, point meta=x, nodes near coords";
    let data = r"\addplot coordinates {(2,1)}; \addplot coordinates {(2,1)};";
    let ticks = texts(&axis(opts, data).0).len() - 2;
    let (c, _) = axis(opts, &format!(r"{data} \node {{4}}; \node {{2}};"));
    let w: Vec<f64> = texts(&c).into_iter().skip(ticks).map(|(l, _)| l.width()).collect();
    assert_eq!(w.len(), 4);
    assert!(close(w[0], w[3]) && close(w[1], w[2]), "{w:?}");
}

#[test]
fn a_closed_cycle_of_no_point_closes_nothing() {
    let (c, w) = axis("unbounded coords=jump, xmin=0, xmax=1, ymin=0, ymax=1", r"\addplot[red] coordinates {(nan,1) (nan,2)} \closedcycle;");
    assert!(w.is_empty(), "{w:?}");
    assert!(in_color(&c, RED).iter().all(|d| !d.segments.iter().any(|s| matches!(s, Segment::Close))));
}

#[test]
fn fill_between_closed_plots() {
    // the region between a triangle and a closed line below it spans both
    let (c, w) = axis(
        "",
        r"\addplot[name path=a] coordinates {(0,0) (1,1) (2,0)} -- cycle; \addplot[name path=b] coordinates {(0,-1) (2,-1)} -- cycle; \addplot[green] fill between[of=a and b]; \draw[blue] (axis cs:0,-1) -- (axis cs:2,1);",
    );
    assert!(w.is_empty(), "{w:?}");
    let fill = &in_color(&c, GREEN)[0];
    let (a, b) = bounds(&fill.segments);
    let (ra, rb) = bounds(&in_color(&c, BLUE)[0].segments);
    assert!(same(a, ra) && same(b, rb), "{a:?} {b:?} {ra:?} {rb:?}");
}

#[test]
fn scatter_marks_without_mapped_colors_keep_the_plots() {
    let (c, _) = axis("", r"\addplot[scatter, only marks, red, mark=*, scatter/use mapped color={}] coordinates {(1,1) (2,2)};");
    assert_eq!(in_color(&c, RED).into_iter().filter(|d| d.fill.is_some()).count(), 2);
    // balls too: both in the same color
    let (m, _) = marks_of("scatter, only marks, mark=ball, scatter/use mapped color={}", "coordinates {(1,1) (2,2)}");
    assert_eq!(m.len(), 2);
    assert_eq!(
        format!(
            "{:?}",
            m[0].fill.as_ref().map(|f| match &f.paint {
                Paint::Shading(Shading::Radial { stops, .. }) => stops.clone(),
                _ => panic!(),
            })
        ),
        format!(
            "{:?}",
            m[1].fill.as_ref().map(|f| match &f.paint {
                Paint::Shading(Shading::Radial { stops, .. }) => stops.clone(),
                _ => panic!(),
            })
        )
    );
}

#[test]
fn a_cube_on_a_flat_z_axis_is_a_square() {
    // z has no length on the canvas: the cube's faces across it are its square
    let (c, w) = axis("x={(1cm,0)}, y={(0,1cm)}, z={(0,0)}", r"\addplot3[red, mark=cube*] coordinates {(1,1,1) (2,2,2)};");
    assert!(w.is_empty(), "{w:?}");
    let faces: Vec<Drawn> = in_color(&c, RED).into_iter().filter(|d| d.fill.is_some()).collect();
    assert_eq!(faces.len(), 6);
    let sides: Vec<(f64, f64)> = faces[..3]
        .iter()
        .map(|d| {
            let (a, b) = bounds(&d.segments);
            (b.x - a.x, b.y - a.y)
        })
        .collect();
    // one face is the mark's square, the others are lines
    // (a cube is `mark size` wide)
    let side = 2.0 * tikra::PT;
    assert_eq!(sides.iter().filter(|(w, h)| (w - side).abs() < 1e-6 && (h - side).abs() < 1e-6).count(), 1, "{sides:?}");
}

#[test]
fn node_options_after_a_space_and_with_braces() {
    // `node [pos=…]` with a space, options holding a group, and a node without options at the end
    same_places("", "(0,0) (2,0) (2,2)", r"node [pos=0.25, font={\small}] {A} node {B}", r"\node[font={\small}] at (axis cs:1,0) {A}; \node at (axis cs:2,2) {B};");
}
