//! pgfplots' libraries and special axes: dates (`date coordinates in`), legends (positions,
//! columns, `transpose legend`, the images of bar plots), box plots, Smith charts, polar and
//! ternary axes, group plots, and pgfplots' own plot marks.

mod common;
use common::*;
use tikra::canvas::{Canvas, Item, Paint};
use tikra::color::Color;
use tikra::scene::{Label, Point, Segment};

const PT: f64 = tikra::PT;

/// An environment of pgfplots in a picture: its canvas and warnings.
fn env(name: &str, options: &str, body: &str) -> (Canvas, Vec<String>) {
    drawn_and_warnings(&format!("\\begin{{{name}}}[{options}] {body} \\end{{{name}}}"))
}

fn axis(options: &str, body: &str) -> (Canvas, Vec<String>) {
    env("axis", options, body)
}

/// A label's glyphs, to compare labels by their text.
fn glyphs(l: &Label) -> String {
    format!("{:?}", l.lines)
}

/// The labels of a canvas, as glyphs.
fn labels(c: &Canvas) -> Vec<String> {
    texts(c).iter().map(|(l, _)| glyphs(l)).collect()
}

/// The paths of a canvas with a solid fill of `color`.
fn filled_with(c: &Canvas, color: Color) -> Vec<Drawn> {
    paths(c).into_iter().filter(|d| matches!(&d.fill, Some(f) if f.paint == Paint::Solid(color))).collect()
}

/// The paths stroked in `color`.
fn stroked_with(c: &Canvas, color: Color) -> Vec<Drawn> {
    paths(c).into_iter().filter(|d| matches!(&d.stroke, Some(s) if s.color == color)).collect()
}

const RED: Color = Color::rgb(1.0, 0.0, 0.0);
const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);

// ---------------------------------------------------------------------------------------------
// dates

/// The tick labels of a date axis read `\year`, `\month`, `\day`, `\hour` and `\minute` of the
/// tick, counted in days (and fractions of a day) from `date ZERO`.
#[test]
fn date_tick_labels() {
    let common = "xmin=0, xmax=3, ymin=0, ymax=1, hide y axis, xtick={0,0.25,0.5104,0.9999,2}";
    let (c, w) = axis(
        &format!(r"{common}, date coordinates in=x, date ZERO=2023-12-31, xticklabel={{\year/\month/\day~\hour:\minute~\tick}}"),
        r"\addplot coordinates {(2024-01-01,0) (2024-01-02 12:00,1)};",
    );
    assert!(w.is_empty(), "{w:?}");
    // the same labels, written out; 0.5104 of a day is 12:14.98, rounded to 12:15; 0.9999 rounds up
    // to 24:00
    let (want, w) = axis(
        &format!(
            r"{common}, xticklabels={{2023/12/31~00:00~2023-12-31, 2023/12/31~06:00~2023-12-31, 2023/12/31~12:15~2023-12-31, 2023/12/31~24:00~2023-12-31, 2024/01/02~00:00~2024-01-02}}"
        ),
        r"\addplot coordinates {(0,0) (1.5,1)};",
    );
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(labels(&c), labels(&want));
}

/// Dates with a time are fractions of a day; the first date the axis reads is day 0 when `date
/// ZERO` is not given; `24:00` is the next day.
#[test]
fn date_values() {
    let points = |opts: &str, coords: &str| {
        let (c, w) = axis(&format!("x=1cm, y=1cm, hide axis, date coordinates in=x, {opts}"), &format!(r"\addplot[mark=none] coordinates {{{coords}}};"));
        assert!(w.is_empty(), "{w:?}");
        let p = paths(&c);
        let pts = points(&p[0].segments);
        pts.iter().map(|q| q.0 - pts[0].0).collect::<Vec<f64>>()
    };
    let xs = points("", "(2024-03-01,0) (2024-03-02 06:00,0) (2024-03-03 24:00,0) (2024-03-05,0)");
    for (x, want) in xs.iter().zip([0.0, 1.25, 3.0, 4.0]) {
        assert!((x - want).abs() < 1e-4, "{xs:?}");
    }
    // date ZERO=… that is not a date: the first date read
    let xs = points("date ZERO={}", "(2024-03-01,0) (2024-03-03,0)");
    assert!((xs[1] - 2.0).abs() < 1e-6, "{xs:?}");
}

// ---------------------------------------------------------------------------------------------
// legends

/// The legend entries: texts, the plot images and the frame.
fn legend(options: &str, body: &str) -> (Canvas, Vec<String>) {
    axis(&format!("xmin=0, xmax=1, ymin=0, ymax=1, hide axis, {options}"), body)
}

const TWO: &str = r"\addplot[red] coordinates {(0,0) (1,1)}; \addlegendentry{a} \addplot[blue] coordinates {(0,1) (1,0)}; \addlegendentry{b}";

/// Where each legend text is, by its glyphs.
fn text_positions(c: &Canvas) -> Vec<(String, Point)> {
    texts(c).into_iter().map(|(l, t)| (glyphs(&l), t.apply(Point::default()))).collect()
}

/// `legend plot pos=right` puts the text before the image; `none` leaves the image out.
#[test]
fn legend_plot_pos() {
    let image_x = |opts: &str| {
        let (c, w) = legend(opts, TWO);
        assert!(w.is_empty(), "{opts}: {w:?}");
        let red = stroked_with(&c, RED);
        // the plot and its image
        (red.len(), red.last().map(|d| bounds(&d.segments).0.x), texts(&c)[0].1.apply(Point::default()).x)
    };
    let (n, img, txt) = image_x("");
    assert_eq!(n, 2);
    assert!(img.unwrap() < txt, "{img:?} {txt}");
    let (n, img, txt) = image_x("legend plot pos=right");
    assert_eq!(n, 2);
    assert!(img.unwrap() > txt, "{img:?} {txt}");
    let (n, _, _) = image_x("legend plot pos=none");
    assert_eq!(n, 1);
}

/// `transpose legend` fills the legend column by column; `legend columns` rows then.
#[test]
fn transpose_legend() {
    let body = r"\addplot coordinates {(0,0)}; \addlegendentry{a} \addplot coordinates {(0,0)}; \addlegendentry{b} \addplot coordinates {(0,0)}; \addlegendentry{c}";
    let rows = |opts: &str| {
        let (c, w) = legend(opts, body);
        assert!(w.is_empty(), "{opts}: {w:?}");
        let mut ys: Vec<f64> = text_positions(&c).iter().map(|(_, p)| p.y).collect();
        ys.sort_by(f64::total_cmp);
        // the texts are centered in their cells: a row's baselines are within a few points
        1 + ys.windows(2).filter(|w| w[1] - w[0] > 0.2).count()
    };
    // one column: three rows; transposed: one row; transposed with two "columns": two rows
    assert_eq!(rows(""), 3);
    assert_eq!(rows("transpose legend"), 1);
    assert_eq!(rows("transpose legend, legend columns=2"), 2);
    assert_eq!(rows("legend columns=2"), 2);
}

/// An empty entry is skipped; an empty legend draws nothing; `legend to name` draws none.
#[test]
fn empty_legend_entries() {
    let (c, w) = legend("", r"\addplot[red] coordinates {(0,0) (1,1)}; \addlegendentry{} \addplot[blue] coordinates {(0,1) (1,0)}; \addlegendentry{b}");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(texts(&c).len(), 1);
    assert_eq!(stroked_with(&c, RED).len(), 1, "no image for the empty entry");
    let (c, _) = legend("", r"\addplot[red] coordinates {(0,0) (1,1)}; \addlegendentry{}");
    assert!(texts(&c).is_empty());
    assert!(filled_with(&c, Color::WHITE).is_empty(), "no frame");
    let (c, w) = legend("legend to name=mylegend", TWO);
    assert!(w.is_empty(), "{w:?}");
    assert!(texts(&c).is_empty());
    // an entry with options but no text is drawn: its image, and an empty text
    let (c, w) = legend("legend entries={{[text=red]},b}", r"\addplot[red] coordinates {(0,0) (1,1)}; \addplot[blue] coordinates {(0,1) (1,0)};");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(stroked_with(&c, RED).len(), 2);
    // only marks: the image is the mark alone
    let (c, w) = legend("", r"\addplot[red, only marks, mark=*] coordinates {(0.5,0.5)}; \addlegendentry{a}");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(stroked_with(&c, RED).len(), 2);
    assert!(stroked_with(&c, RED).iter().all(|d| d.fill.is_some()));
    // a text that does not count for the size takes no room
    let (c, w) = legend("", r"\addplot[red] coordinates {(0,0) (1,1)}; \addlegendentry[overlay]{a}");
    assert!(w.is_empty(), "{w:?}");
    let frame = filled_with(&c, Color::WHITE);
    let (lo, hi) = bounds(&frame[0].segments);
    let (c2, _) = legend("", r"\addplot[red] coordinates {(0,0) (1,1)}; \addlegendentry{a}");
    let (lo2, hi2) = bounds(&filled_with(&c2, Color::WHITE)[0].segments);
    assert!(hi.x - lo.x < hi2.x - lo2.x, "{lo:?} {hi:?} / {lo2:?} {hi2:?}");
    // a plot with an empty legend image: its text only
    let (c, w) = legend("", r"\addplot[red, empty legend] coordinates {(0,0) (1,1)}; \addlegendentry{a}");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(stroked_with(&c, RED).len(), 1);
    assert_eq!(texts(&c).len(), 1);
}

/// `legend entries` as a list: braces keep a comma, `...` counts as `\foreach` does.
#[test]
fn legend_entries_lists() {
    let three = r"\addplot coordinates {(0,0)}; \addplot coordinates {(0,0)}; \addplot coordinates {(0,0)};";
    let (c, w) = legend("legend entries={1,...,3}", three);
    assert!(w.is_empty(), "{w:?}");
    let (want, _) = legend("legend entries={1,2,3}", three);
    assert_eq!(labels(&c), labels(&want));
    let (c, _) = legend("legend entries={{a,b},c}", r"\addplot coordinates {(0,0)}; \addplot coordinates {(0,0)};");
    assert_eq!(texts(&c).len(), 2);
    let (want, _) = legend("", r"\addplot coordinates {(0,0)}; \addlegendentry{a,b} \addplot coordinates {(0,0)}; \addlegendentry{c}");
    assert_eq!(labels(&c), labels(&want));
}

/// A legend anchor that is not a compass direction is warned about, and the legend is centered.
#[test]
fn legend_anchor_not_supported() {
    let (c, w) = legend("legend style={anchor=text}", TWO);
    assert!(w.iter().any(|a| a.contains("legend anchor 'text' not supported")), "{w:?}");
    assert_eq!(texts(&c).len(), 2);
}

/// The legend images of interval bars and of bars with a line legend.
#[test]
fn legend_images_of_bars() {
    let image = |opts: &str| {
        let (c, w) = legend("", &format!(r"\addplot[{opts}, fill=red, draw=none] coordinates {{(0,0.5) (0.5,0.7) (1,0.2)}}; \addlegendentry{{a}}"));
        assert!(w.is_empty(), "{opts}: {w:?}");
        let f = filled_with(&c, RED);
        // the plot, then the image
        let img = &f.last().unwrap().segments;
        (img.iter().filter(|s| matches!(s, Segment::Close)).count(), bounds(img))
    };
    // two bars, the first 0.8em high and the second 0.6em, upwards or to the right
    let (n, (lo, hi)) = image("ybar interval");
    assert_eq!(n, 2);
    assert!(close(hi.y - lo.y, 8.0 * PT), "{lo:?} {hi:?}");
    let (n, (lo, hi)) = image("xbar interval");
    assert_eq!(n, 2);
    assert!(close(hi.x - lo.x, 8.0 * PT), "{lo:?} {hi:?}");
    // a line legend of a bar plot: three bars of the bar width on the line's points
    let (n, (lo, hi)) = image("ybar, bar width=4pt, line legend");
    assert_eq!(n, 3);
    assert!(close(hi.x - lo.x, 0.6 + 4.0 * PT), "{lo:?} {hi:?}");
}

// ---------------------------------------------------------------------------------------------
// box plots

/// A box plot's box: (lower quartile, upper quartile, median) as x on the canvas, and the
/// whiskers' ends; values on x, one unit a cm.
fn box_geometry(c: &Canvas) -> Vec<Vec<(f64, f64)>> {
    paths(c).iter().filter(|d| d.stroke.is_some()).map(|d| points(&d.segments)).collect()
}

/// The quartiles by each estimator, as pgfplots computes them, on 1…6.
#[test]
fn boxplot_estimators() {
    let quartiles = |estimator: &str, n: usize| {
        let data: String = (1..=n).map(|k| format!("{k}\\\\")).collect();
        let (c, w) =
            axis(&format!("x=1cm, y=1cm, hide axis, xmin=0, xmax=7, boxplot/estimator={estimator}"), &format!(r"\addplot[boxplot] table[row sep=\\, y index=0] {{{data}}};"));
        assert!(w.is_empty(), "{estimator}: {w:?}");
        // the box: a closed path; its x extent are the quartiles (from the origin of the axis)
        let p = paths(&c);
        let b = p.iter().find(|d| d.segments.iter().any(|s| matches!(s, Segment::Close))).expect("a box");
        let (lo, hi) = bounds(&b.segments);
        let origin = bounds(&p.iter().flat_map(|d| d.segments.clone()).collect::<Vec<_>>()).0.x;
        let whisker_lo = p.iter().flat_map(|d| points(&d.segments)).map(|q| q.0).fold(f64::INFINITY, f64::min);
        (lo.x - whisker_lo, hi.x - whisker_lo, origin)
    };
    // R2: the mean of the values at p·n + 0.5 rounded down and up; legacy*: 0-based, p·(n−1), the mean
    // of the two values around it, or the value there
    for (est, n, lq, uq) in [("R2", 6, 2.0, 5.0), ("legacy*", 6, 2.5, 4.5), ("legacy*", 5, 2.0, 4.0), ("legacy", 6, 1.5, 4.5)] {
        let (a, b, _) = quartiles(est, n);
        // measured from the lower whisker, the value 1
        assert!(close(a, lq - 1.0) && close(b, uq - 1.0), "{est} {n}: {a} {b}");
    }
}

/// `boxplot/average=auto` draws the average with the `every average` mark.
#[test]
fn boxplot_average() {
    let (c, w) = axis(
        "x=1cm, y=1cm, hide axis, boxplot/average=auto, boxplot/every average/.style={mark=square*, mark size=2pt, fill=red}",
        r"\addplot[boxplot] table[row sep=\\, y index=0] {1\\2\\3\\4\\10\\};",
    );
    assert!(w.is_empty(), "{w:?}");
    let marks = filled_with(&c, RED);
    assert_eq!(marks.len(), 1);
    // the average is 4: three units right of the lower whisker (1)
    let all: Vec<f64> = paths(&c).iter().flat_map(|d| points(&d.segments)).map(|q| q.0).collect();
    let min = all.iter().copied().fold(f64::INFINITY, f64::min);
    let (lo, hi) = bounds(&marks[0].segments);
    assert!(close((lo.x + hi.x) / 2.0 - min, 3.0), "{lo:?} {hi:?} {min}");
    // without a mark in `every average`, nothing
    let (c, _) = axis("hide axis, boxplot/average=auto, boxplot/every average/.style={mark=none}", r"\addplot[boxplot, fill=red] table[row sep=\\, y index=0] {1\\2\\3\\};");
    assert!(filled_with(&c, RED).len() <= 1);
}

/// A prepared box plot without quartiles: each whisker drawn from the other one, no box.
#[test]
fn boxplot_without_a_box() {
    let (c, w) = axis("x=1cm, y=1cm, hide axis", r"\addplot[boxplot prepared={lower whisker=1, upper whisker=5}] coordinates {};");
    assert!(w.is_empty(), "{w:?}");
    let p = paths(&c);
    assert!(!p.iter().any(|d| d.segments.iter().any(|s| matches!(s, Segment::Close))), "no box");
    let g = box_geometry(&c);
    assert_eq!(g.len(), 2, "{g:?}");
    // the lower whisker's line goes from 5 to 1, the upper one's from 1 to 5: 4 units each
    for w in &g {
        assert!(close((w[1].0 - w[0].0).abs(), 4.0), "{g:?}");
    }
    assert!(close(g[0][0].0, g[1][1].0) && close(g[0][1].0, g[1][0].0), "{g:?}");
}

/// A box plot without data draws nothing; `draw direction=y` puts the values upwards.
#[test]
fn boxplot_empty_and_vertical() {
    let (c, w) = axis("x=1cm, y=1cm, hide axis, xmin=0, xmax=1, ymin=0, ymax=1", r"\addplot[boxplot, red] coordinates {};");
    assert!(w.is_empty(), "{w:?}");
    assert!(stroked_with(&c, RED).is_empty());
    let (c, w) = axis(
        "x=1cm, y=1cm, hide axis, boxplot/draw direction=y",
        r"\addplot[boxplot prepared={lower whisker=1, lower quartile=2, median=3, upper quartile=4, upper whisker=5}, red] coordinates {};",
    );
    assert!(w.is_empty(), "{w:?}");
    let all: Vec<Segment> = stroked_with(&c, RED).iter().flat_map(|d| d.segments.clone()).collect();
    let (lo, hi) = bounds(&all);
    // from the lower whisker to the upper one: 4 units upwards
    assert!(close(hi.y - lo.y, 4.0) && hi.x - lo.x < 1.0, "{lo:?} {hi:?}");
}

/// `boxplot/variable width`: each box as wide as the square root of its sample size allows.
#[test]
fn boxplot_variable_width() {
    let (c, w) = axis(
        "x=1cm, y=1cm, hide axis, boxplot/variable width, boxplot/variable width min target=0.5",
        r"\addplot[boxplot] table[row sep=\\, y index=0] {1\\2\\3\\4\\};
          \addplot[boxplot] table[row sep=\\, y index=0] {1\\2\\3\\4\\5\\6\\7\\8\\9\\10\\11\\12\\13\\14\\15\\16\\};",
    );
    assert!(w.is_empty(), "{w:?}");
    let boxes: Vec<f64> = paths(&c)
        .iter()
        .filter(|d| d.segments.iter().any(|s| matches!(s, Segment::Close)))
        .map(|d| {
            let (lo, hi) = bounds(&d.segments);
            hi.y - lo.y
        })
        .collect();
    assert_eq!(boxes.len(), 2);
    // the smaller sample's box is half as wide (min target 0.5)
    assert!(close(boxes[0] * 2.0, boxes[1]), "{boxes:?}");
}

/// `boxplot cs`, `boxplot box cs` and `boxplot whisker cs` place points relative to the last box.
#[test]
fn boxplot_coordinate_systems() {
    let body = r"\addplot[boxplot prepared={lower whisker=1, lower quartile=2, median=3, upper quartile=4, upper whisker=5},
        boxplot/draw position=2, boxplot/box extend=0.8, boxplot/whisker extend=0.4] coordinates {};
        \draw[red] (boxplot cs:3,0) -- (boxplot box cs:3,1) -- (boxplot whisker cs:5,1);";
    let (c, w) = axis("x=1cm, y=1cm, hide axis", body);
    assert!(w.is_empty(), "{w:?}");
    let red = points(&stroked_with(&c, RED)[0].segments);
    let g = box_geometry(&c);
    // the median: from the box's bottom to its top, at 3
    let median = g.iter().find(|p| p.len() == 2 && close(p[0].0, p[1].0)).expect("the median");
    // at the draw position (the box's middle), the box's top, the top of the upper whisker
    assert!(close_to(red[0], (median[0].0, (median[0].1 + median[1].1) / 2.0)), "{red:?} {median:?}");
    assert!(close_to(red[1], median[1]), "{red:?} {median:?}");
    let upper = g.iter().find(|p| p.len() == 4 && close(p[1].0, median[0].0 + 2.0)).expect("the upper whisker");
    assert!(close_to(red[2], upper[3]), "{red:?} {upper:?}");
}

// ---------------------------------------------------------------------------------------------
// smith charts

fn smith(options: &str, body: &str) -> (Canvas, Vec<String>) {
    env("smithchart", options, body)
}

/// `is smithchart cs`: the coordinates are points of the unit circle's plane, drawn as they are.
#[test]
fn smithchart_cs_input() {
    let line = |opts: &str, coords: &str| {
        let (c, w) = smith("width=6cm", &format!(r"\addplot[red, mark=none, {opts}] coordinates {{{coords}}};"));
        assert!(w.is_empty(), "{w:?}");
        points(&stroked_with(&c, RED)[0].segments)
    };
    // the impedance 1 is the center; Γ = 0.5 is the impedance 3
    let a = line("is smithchart cs", "(0,0) (0.5,0)");
    let b = line("", "(1,0) (3,0)");
    assert!(close_to(a[0], b[0]) && close_to(a[1], b[1]), "{a:?} {b:?}");
}

/// The tick sets by the chart's size: few up to 14cm, many up to 20cm, dense beyond.
#[test]
fn smithchart_ticks_by_size() {
    let count = |width: &str| {
        let (c, w) = smith(&format!("width={width}"), "");
        assert!(w.is_empty(), "{w:?}");
        texts(&c).len()
    };
    let (few, many, dense) = (count("6cm"), count("16cm"), count("24cm"));
    assert!(few < many && many < dense, "{few} {many} {dense}");
}

/// `show origin` puts a small ring at the center, over everything.
#[test]
fn smithchart_show_origin() {
    let (c, w) = smith("width=6cm, show origin", "");
    assert!(w.is_empty(), "{w:?}");
    let p = paths(&c);
    let n = p.len();
    let (outer, inner) = (&p[n - 2], &p[n - 1]);
    let (lo, hi) = bounds(&outer.segments);
    assert!(close(hi.x - lo.x, 5.0 * PT), "{lo:?} {hi:?}");
    let (lo2, hi2) = bounds(&inner.segments);
    assert!(close(hi2.x - lo2.x, PT), "{lo2:?} {hi2:?}");
    assert!(close(lo.x + hi.x, lo2.x + hi2.x));
    assert_eq!(outer.fill.as_ref().unwrap().paint, Paint::Solid(Color::WHITE));
}

/// Hidden axes of a Smith chart draw no lines and no labels.
#[test]
fn smithchart_hidden_axes() {
    let (c, w) = smith("width=6cm, hide axis", "");
    assert!(w.is_empty(), "{w:?}");
    assert!(texts(&c).is_empty());
    let (all, _) = smith("width=6cm", "");
    let (x, _) = smith("width=6cm, hide x axis", "");
    let (y, _) = smith("width=6cm, hide y axis", "");
    assert!(texts(&x).len() < texts(&all).len() && texts(&y).len() < texts(&all).len());
}

/// The tick lines inside, outside or centered on the axis.
#[test]
fn smithchart_tick_align() {
    for align in ["inside", "outside", "center"] {
        let (c, w) = smith(&format!("width=6cm, xtick={{1}}, ytick={{1}}, tick align={align}, xmajorticks, ymajorticks"), "");
        assert!(w.is_empty(), "{align}: {w:?}");
        assert!(!paths(&c).is_empty());
    }
}

/// Grid lines stop by `grid each nth passes`, with `if <` limits and `[index]` lines; `stop at`.
#[test]
fn smithchart_grid_rules() {
    let grid = |opts: &str| {
        let (c, w) = smith(&format!("width=8cm, xtick={{0.5,1,2}}, ytick={{0.5,1,2,-0.5,-1,-2}}, xgrid each nth passes y={{{opts}}}, ygrid each nth passes x={{1:2 if < 2}}"), "");
        (paths(&c).len(), w)
    };
    let (n, w) = grid("1:2 if <0.6, [index]1:3");
    assert!(w.is_empty(), "{w:?}");
    assert!(n > 0);
    let (_, w) = grid("[index]9");
    assert!(w.iter().any(|a| a.contains("no grid line '[index]9'")), "{w:?}");
    let (_, w) = grid("nothing");
    assert!(w.is_empty(), "{w:?}");
    let stopped = |stops: &str| {
        let (c, w) = smith(&format!("width=8cm, xtick={{0.5,1}}, ytick={{0.5,1}}, yminorgrids=false, xminorgrids=false, {stops}"), "");
        assert!(w.is_empty(), "{w:?}");
        // the grid's segments, as a measure of how far its lines go
        paths(&c).iter().filter(|d| d.stroke.as_ref().is_some_and(|s| s.color != Color::BLACK)).map(|d| d.segments.len()).sum::<usize>()
    };
    let free = stopped("");
    let some = stopped("ygrid stop at x={0.5:0.5, 1:0.2}, xgrid stop at y={0.5:1}");
    assert_ne!(free, some);
    // entries without a stop, or not numbers, stop nothing
    assert_eq!(stopped("ygrid stop at x={0.5, 1:abc}"), free);
}

/// Minor ticks are placed between the major ones; `minor xtick` given lists them.
#[test]
fn smithchart_minor_ticks() {
    let (c, w) = smith("width=8cm, xtick={1,2}, minor x tick num=1, xminorgrids, xminorticks", "");
    assert!(w.is_empty(), "{w:?}");
    assert!(!paths(&c).is_empty());
}

/// `sloped like y axis` with its options turns the y labels along the circle.
#[test]
fn smithchart_sloped_labels() {
    for opts in [
        "sloped like y axis",
        "sloped like y axis={allow upside down}",
        "sloped like y axis={execute for upside down={anchor=west}, reset nontranslations=false}",
        "sloped like y axis={allow upside down=false, something=1}",
    ] {
        let (c, w) = smith(&format!("width=6cm, ytick={{-1,1}}, yticklabel style={{{opts}}}"), "");
        assert!(w.is_empty(), "{opts}: {w:?}");
        assert!(!texts(&c).is_empty());
    }
    // a tick that is not major gets no label
    let (c, w) = smith("width=6cm, ytick={1}, extra y ticks={2}", "");
    assert!(w.is_empty(), "{w:?}");
    assert!(!texts(&c).is_empty());
}

/// `yticklabel cs` in a Smith chart: its shift goes out of the circle.
#[test]
fn smithchart_yticklabel_cs() {
    let (c, w) = smith("width=6cm", r"\draw[red] (yticklabel* cs:0.5,1cm) -- (yticklabel* cs:0.5);");
    assert!(w.is_empty(), "{w:?}");
    let p = points(&stroked_with(&c, RED)[0].segments);
    // at y = 0 (the chart's left end) the circle's outward normal points left
    assert!(close_to((p[1].0 - p[0].0, p[1].1 - p[0].1), (1.0, 0.0)), "{p:?}");
}

/// A y tick outside the limits gets no label.
#[test]
fn smithchart_tick_outside_the_limits() {
    let (inside, w) = smith("width=6cm, ymin=-2, ymax=2, ytick={1}, xtick={1}", "");
    assert!(w.is_empty(), "{w:?}");
    let (outside, _) = smith("width=6cm, ymin=-2, ymax=2, ytick={1,5}, xtick={1}", "");
    assert_eq!(texts(&inside).len(), texts(&outside).len());
}

/// `cartesian cs` in a Smith chart: the unit circle's plane; (−1,0) to (1,0) is the diameter.
#[test]
fn smithchart_cartesian_cs() {
    let (c, w) = smith("width=6cm", r"\draw[red] (cartesian cs:-1,0) -- (cartesian cs:1,0);");
    assert!(w.is_empty(), "{w:?}");
    let p = points(&stroked_with(&c, RED)[0].segments);
    let diameter = paths(&c).into_iter().map(|d| points(&d.segments)).find(|q| q.len() == 2 && close_to(q[0], p[0])).expect("the x axis line");
    assert!(close_to(diameter[1], p[1]), "{p:?} {diameter:?}");
}

// ---------------------------------------------------------------------------------------------
// polar axes

fn polar(options: &str, body: &str) -> (Canvas, Vec<String>) {
    env("polaraxis", options, body)
}

/// A sector's outline: to the origin without an inner radius, round both radii with one.
#[test]
fn polar_sectors() {
    for (opts, closed) in [("xmin=0, xmax=90, ymin=0, ymax=1", true), ("xmin=0, xmax=90, ymin=0.5, ymax=1", true), ("ymin=0.5, ymax=1", true)] {
        let (c, w) = polar(&format!("{opts}, axis on top, clip=true"), r"\addplot[red, fill=red] coordinates {(10,0.7) (80,0.7) (45,2)} \closedcycle;");
        assert!(w.is_empty(), "{opts}: {w:?}");
        let has_clip = c.items.iter().any(|i| matches!(i, Item::Clip { .. }));
        assert_eq!(has_clip, closed, "{opts}");
    }
}

/// Marks outside the sector are not drawn; the sector may go round 0.
#[test]
fn polar_marks_in_the_sector() {
    let count = |opts: &str, coords: &str| {
        let (c, w) = polar(&format!("ymin=0, ymax=2, {opts}"), &format!(r"\addplot[only marks, mark=*, red] coordinates {{{coords}}};"));
        assert!(w.is_empty(), "{opts}: {w:?}");
        filled_with(&c, RED).len()
    };
    // radius 3 is outside; 45 degrees inside a sector of 0 to 90
    assert_eq!(count("xmin=0, xmax=90", "(45,1) (45,3) (180,1)"), 1);
    // a sector from 300 to 60 goes round 0
    assert_eq!(count("xmin=300, xmax=420", "(10,1) (330,1) (180,1)"), 2);
    // a sector with an inner radius
    assert_eq!(count("xmin=0, xmax=90, ymin=1", "(45,0.5) (45,1.5)"), 1);
    // an axis of no size has no sector to tell inside from outside: its marks are all kept, at
    // its center
    assert_eq!(count("xmin=0, xmax=90, scale only axis, width=0pt, height=0pt", "(45,1) (45,3) (180,1)"), 3);
}

/// The angles' ticks: as given, or by the arc's length.
#[test]
fn polar_ticks() {
    let labels_of = |opts: &str| {
        let (c, w) = polar(&format!("hide y axis, {opts}"), "");
        assert!(w.is_empty(), "{opts}: {w:?}");
        labels(&c)
    };
    assert_eq!(labels_of("xtick={0,90}").len(), 2);
    // `xtick distance`: ticks as a linear axis has them
    assert_eq!(labels_of("xmin=0, xmax=90, xtick distance=30"), labels_of("xmin=0, xmax=90, xtick={0,30,60,90}"));
    // at least four ticks on a short arc, at a step of 90, 45, 30, 15, 10 or 5 degrees
    for (range, ticks) in [("xmin=0, xmax=90", "0,45,90"), ("xmin=0, xmax=40", "0,15,30"), ("xmin=0, xmax=20", "0,10,20"), ("xmin=0, xmax=12", "0,5,10")] {
        let auto = labels_of(&format!("{range}, width=4cm"));
        assert_eq!(auto, labels_of(&format!("{range}, width=4cm, xtick={{{ticks}}}")), "{range}");
    }
}

/// Hidden polar axes, grids and `sloped like x axis` labels.
#[test]
fn polar_axis_parts() {
    let (all, w) = polar("xmajorgrids, ymajorgrids", "");
    assert!(w.is_empty(), "{w:?}");
    // the grids are drawn by default; without them, fewer paths
    let (bare, _) = polar("xmajorgrids=false, ymajorgrids=false", "");
    assert!(paths(&bare).len() < paths(&all).len());
    let (x, _) = polar("hide x axis", "");
    let (y, _) = polar("hide y axis", "");
    let (none, _) = polar("hide axis", "");
    assert!(texts(&none).is_empty());
    assert!(texts(&x).len() < texts(&all).len() && texts(&y).len() < texts(&all).len());
    let (c, w) = polar("xticklabel style={sloped like x axis}", "");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(texts(&c).len(), texts(&polar("", "").0).len());
    let (c, _) = polar("xmajorticks=false", "");
    assert!(texts(&c).len() < texts(&polar("", "").0).len());
}

// ---------------------------------------------------------------------------------------------
// ternary axes

fn ternary(options: &str, body: &str) -> (Canvas, Vec<String>) {
    env("ternaryaxis", options, body)
}

/// A point with two coordinates takes what they leave of one as its third.
#[test]
fn ternary_missing_z() {
    let line = |coords: &str| {
        let (c, w) = ternary("hide axis", &format!(r"\draw[red] {coords};"));
        assert!(w.is_empty(), "{w:?}");
        points(&stroked_with(&c, RED)[0].segments)
    };
    let two = line("(axis cs:0.2,0.3) -- (axis cs:0.5,0.1)");
    let three = line("(axis cs:0.2,0.3,0.5) -- (axis cs:0.5,0.1,0.4)");
    assert!(close_to(two[0], three[0]) && close_to(two[1], three[1]), "{two:?} {three:?}");
}

/// Plain coordinates in a ternary axis are cartesian, in units of the side.
#[test]
fn ternary_cartesian() {
    let (c, w) = ternary("width=6cm, hide axis", r"\draw[red] (0,0) -- (1,0); \draw[blue] (cartesian cs:0,0) -- (cartesian cs:1,0);");
    assert!(w.is_empty(), "{w:?}");
    let a = points(&stroked_with(&c, RED)[0].segments);
    let b = points(&stroked_with(&c, BLUE)[0].segments);
    assert!(close_to(a[0], b[0]) && close_to(a[1], b[1]), "{a:?} {b:?}");
    assert!(a[1].0 - a[0].0 > 1.0, "{a:?}");
}

/// Ternary grids, minor grids, tick alignments and hidden axes.
#[test]
fn ternary_axis_parts() {
    for opts in ["minor tick num=1, xminorgrids, yminorgrids, zminorgrids, xmajorgrids", "tick align=center", "tick align=inside", "tick align=outside"] {
        let (c, w) = ternary(opts, "");
        assert!(w.is_empty(), "{opts}: {w:?}");
        assert!(!paths(&c).is_empty());
    }
    let (all, _) = ternary("", "");
    let (bare, _) = ternary("xmajorgrids=false, ymajorgrids=false, zmajorgrids=false", "");
    assert!(paths(&bare).len() < paths(&all).len());
    let (unlabelled, _) = ternary("xmajorticks=false", "");
    assert!(texts(&unlabelled).len() < texts(&all).len());
    let sides = |c: &Canvas| paths(c).iter().filter(|d| d.segments.len() == 2).count();
    let (x, w) = ternary("hide x axis", "");
    assert!(w.is_empty(), "{w:?}");
    // the triangle drawn side by side: the two sides left
    assert!(texts(&x).len() < texts(&all).len());
    assert!(sides(&x) >= 2);
    // relative limits show percentages; absolute ones the values
    let (rel, _) = ternary("xmin=0, xmax=0.5, ternary limits relative", "");
    let (abs, _) = ternary("xmin=0, xmax=0.5, ternary limits relative=false", "");
    assert_ne!(labels(&rel), labels(&abs));
}

// ---------------------------------------------------------------------------------------------
// group plots

/// A group plot outside a picture opens one; `group style` and the plots' options.
#[test]
fn groupplot_outside_a_picture() {
    let doc = r"\begin{groupplot}[group style={group size=2 by 1, horizontal sep=1cm}, width=3cm, height=3cm, scale only axis, hide axis]
        \nextgroupplot[xmin=0, xmax=1] \addplot coordinates {(0,0) (1,1)};
        \nextgroupplot \addplot coordinates {(0,0) (1,1)};
        \end{groupplot}";
    let (c, w) = tikra::engine::run_document(doc).unwrap();
    assert!(w.is_empty(), "{w:?}");
    let (lo, hi) = (c.min.unwrap(), c.max.unwrap());
    // two 3cm plots 1cm apart
    assert!((hi.x - lo.x - 7.0).abs() < 0.2, "{lo:?} {hi:?}");
}

/// What a group plot cannot do is warned about.
#[test]
fn groupplot_warnings() {
    let (_, w) = drawn_and_warnings(r"\nextgroupplot");
    assert!(w.iter().any(|a| a.contains("outside a groupplot")), "{w:?}");
    let (_, w) = drawn_and_warnings(
        r"\begin{groupplot}[group style={group size=1 by 1}] \nextgroupplot \addplot coordinates {(0,0)}; \nextgroupplot \addplot coordinates {(1,1)}; \end{groupplot}",
    );
    assert!(w.iter().any(|a| a.contains("more plots than the group size (1 by 1)")), "{w:?}");
    let (_, w) = drawn_and_warnings(r"\begin{axis} \begin{groupplot} \nextgroupplot \end{groupplot} \end{axis}");
    assert!(w.iter().any(|a| a.contains("groupplot inside a groupplot or an axis")), "{w:?}");
    // a group left open inside is closed with the group plot: the color set inside is gone after it
    let (c, w) = drawn_and_warnings(r"\begin{groupplot} \begingroup \color{red} \end{groupplot} \draw (0,0) -- (1,0);");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(stroked_with(&c, Color::BLACK).len(), 1);
}

/// Tick labels and labels at the edges, and a style for one plot.
#[test]
fn groupplot_edges() {
    let group = |style: &str| {
        let body = format!(
            r"\begin{{groupplot}}[group style={{group size=2 by 2, {style}}}, width=3cm, height=3cm, xlabel=X, ylabel=Y]
            \nextgroupplot \addplot coordinates {{(0,0) (1,1)}};
            \nextgroupplot \addplot coordinates {{(0,0) (1,1)}};
            \nextgroupplot \addplot coordinates {{(0,0) (1,1)}};
            \nextgroupplot \addplot coordinates {{(0,0) (1,1)}};
            \end{{groupplot}}"
        );
        let (c, w) = drawn_and_warnings(&body);
        assert!(w.is_empty(), "{style}: {w:?}");
        texts(&c).len()
    };
    let all = group("");
    for style in [
        "xticklabels at=edge top",
        "xticklabels at=edge bottom",
        "yticklabels at=edge left",
        "yticklabels at=edge right",
        "xlabels at=edge top",
        "xlabels at=edge bottom",
        "ylabels at=edge left",
        "ylabels at=edge right",
    ] {
        assert!(group(style) < all, "{style}");
    }
    // a plot's own style after the edges' settings
    let (c, w) = drawn_and_warnings(
        r"\begin{groupplot}[group style={group size=2 by 1, yticklabels at=edge left, plot c2r1/.style={hide axis}}, width=3cm, height=3cm]
        \nextgroupplot \addplot coordinates {(0,0) (1,1)};
        \nextgroupplot \addplot coordinates {(0,0) (1,1)};
        \end{groupplot}",
    );
    assert!(w.is_empty(), "{w:?}");
    let (c2, _) = drawn_and_warnings(
        r"\begin{groupplot}[group style={group size=2 by 1, yticklabels at=edge left}, width=3cm, height=3cm]
        \nextgroupplot \addplot coordinates {(0,0) (1,1)};
        \nextgroupplot \addplot coordinates {(0,0) (1,1)};
        \end{groupplot}",
    );
    // the second plot's x tick labels are gone too
    assert!(texts(&c).len() < texts(&c2).len());
    // `group/plot c1r1/.style` and empty group style
    let (c, w) = drawn_and_warnings(
        r"\begin{groupplot}[group style={group size=2 by 1, plot c1r1/.style={hide axis}}, group style, width=3cm, height=3cm]
        \nextgroupplot \addplot coordinates {(0,0) (1,1)};
        \nextgroupplot \addplot coordinates {(0,0) (1,1)};
        \end{groupplot}",
    );
    assert!(w.is_empty(), "{w:?}");
    let (c2, _) = drawn_and_warnings(
        r"\begin{groupplot}[group style={group size=2 by 1}, width=3cm, height=3cm]
        \nextgroupplot \addplot coordinates {(0,0) (1,1)};
        \nextgroupplot \addplot coordinates {(0,0) (1,1)};
        \end{groupplot}",
    );
    assert!(texts(&c).len() < texts(&c2).len());
}

// ---------------------------------------------------------------------------------------------
// pgfplots' marks

/// pgfplots' `oplus`, `oplus*` and `halfcircle` marks: a circle with a plus, a half disk.
#[test]
fn pgfplots_marks() {
    for (mark, filled, segments) in [("oplus", false, 9), ("oplus*", true, 9), ("halfcircle", false, 0), ("halfcircle*", true, 0)] {
        let (c, w) = axis("hide axis", &format!(r"\addplot[only marks, mark={mark}, red, fill=red] coordinates {{(0,0)}};"));
        assert!(w.is_empty(), "{mark}: {w:?}");
        let m = stroked_with(&c, RED);
        assert_eq!(m.len(), 1, "{mark}");
        assert_eq!(m[0].fill.is_some(), filled, "{mark}");
        if segments > 0 {
            // the circle (4 curves) and the plus (two lines)
            assert_eq!(m[0].segments.len(), segments + 1, "{mark}: {:?}", m[0].segments);
        } else {
            // the upper half of a circle, closed
            let (lo, hi) = bounds(&m[0].segments);
            assert!(close(hi.x - lo.x, 2.0 * (hi.y - lo.y)), "{mark}: {lo:?} {hi:?}");
        }
    }
}
