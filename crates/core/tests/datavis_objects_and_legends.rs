//! Data visualizations: the objects behind them (`new object`, the methods of mappers,
//! transformers, intervals, counters, accumulators and of the data visualization itself, the phases'
//! hooks), what the visualizers draw (lines, cycles, marks, rectangles, labels and pins in the
//! data, style sheets and color series) and the legends (their entries, samples and arrangements).

mod common;

use common::{drawn_and_warnings, paths, points, Drawn};
use tikra::canvas::Canvas;
use tikra::color::Color;
use tikra::PT;

/// A data visualization's canvas, without warnings (moved so that its box starts at (0, 0)).
fn chart(body: &str) -> Canvas {
    let (c, w) = drawn_and_warnings(body);
    assert!(w.is_empty(), "{w:?}\n{body}");
    c
}

/// Requires a warning containing `fragment`.
fn warns(body: &str, fragment: &str) {
    let w = drawn_and_warnings(body).1;
    assert!(w.iter().any(|w| w.contains(fragment)), "expected '{fragment}' in {w:?}\n{body}");
}

/// The texts of a picture, in drawing order, as its description reads them out.
fn words(body: &str) -> Vec<String> {
    let r = tikra::convert(&format!("\\begin{{tikzpicture}}{body}\\end{{tikzpicture}}")).unwrap_or_else(|e| panic!("{e}\n{body}"));
    assert!(r.warnings.is_empty(), "{:?}\n{body}", r.warnings);
    let d = r.description;
    let list = d.split_once("Text in the chart: ").or_else(|| d.split_once("Text in the figure: ")).map(|(_, l)| l).unwrap_or("");
    list.strip_suffix('.').unwrap_or(list).split("; ").filter(|s| !s.is_empty()).map(str::to_string).collect()
}

/// The paths drawn semithick (what the visualizers draw), in order.
fn data_paths(c: &Canvas) -> Vec<Drawn> {
    paths(c).into_iter().filter(|p| p.stroke.as_ref().is_some_and(|s| (s.width - 0.6 * PT).abs() < 1e-9)).collect()
}

/// The points of a path, from its first point, in cm.
fn relative(d: &Drawn) -> Vec<(f64, f64)> {
    let p = points(&d.segments);
    p.iter().map(|q| (q.0 - p[0].0, q.1 - p[0].1)).collect()
}

/// The points of the first data path, from its first point.
fn data_line(c: &Canvas) -> Vec<(f64, f64)> {
    relative(&data_paths(c)[0])
}

fn near(a: (f64, f64), b: (f64, f64)) -> bool {
    (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3
}

fn same_line(got: &[(f64, f64)], want: &[(f64, f64)]) -> bool {
    got.len() == want.len() && got.iter().zip(want).all(|(a, b)| near(*a, *b))
}

fn stroke_color(d: &Drawn) -> Color {
    d.stroke.as_ref().expect("a stroked path").color
}

fn close_color(c: Color, r: f64, g: f64, b: f64) -> bool {
    (c.r - r).abs() < 1e-3 && (c.g - g).abs() < 1e-3 && (c.b - b).abs() < 1e-3
}

const TWO_POINTS: &str = "data point [x=0, y=0] data point [x=3, y=1]";

// ---------------------------------------------------------------- the phases and new object

/// The phases' hooks run in their order; the later ones (`at end …`, `after …`) put their code
/// before what they already have.
#[test]
fn hooks_run_in_the_phases_order() {
    let w = words(&format!(
        r"\gdef\order{{}}\def\add#1{{\xdef\order{{\order#1}}}}
          \datavisualization [xy Cartesian, visualize as line, before survey={{\add a}}, at start survey={{\add b}},
            at end survey={{\add d}}, at end survey={{\add c}}, after survey={{\add e}}, before visualization={{\add f}},
            at start visualization={{\add g}}, at end visualization={{\add h}}, after visualization={{\add i}}] {TWO_POINTS};
          \node at (0,-2) {{\order}};"
    ));
    assert_eq!(w, ["abcdefghi"]);
}

/// The data visualization object's own methods, through its handle: hooks added by method (the
/// object is made before the first survey, so a second one runs `before survey`), the number of
/// the survey, and a survey more.
#[test]
fn the_data_visualizations_own_methods() {
    let w = words(&format!(
        r"\makeatletter
          \datavisualization [xy Cartesian, visualize as line, new object={{class=interval, store=/tikz/data visualization/hooked, after creation={{%
            \tikra@dv@main.do an additional survey()%
            \tikra@dv@main.before survey(\gdef\a{{A}})\tikra@dv@main.at start survey(\gdef\b{{B}})%
            \tikra@dv@main.at end survey(\gdef\c{{C}})\tikra@dv@main.after survey(\gdef\d{{D}})%
            \tikra@dv@main.before visualization(\gdef\e{{E}})\tikra@dv@main.at start visualization(\gdef\f{{F}})%
            \tikra@dv@main.at end visualization(\gdef\g{{G}})\tikra@dv@main.after visualization(\gdef\h{{H}})%
            \tikra@dv@main.at no time(\gdef\h{{X}})\tikra@dv@main.anything()}}}}] {TWO_POINTS};
          \node at (0,-2) {{\a\b\c\d\e\f\g\h}};"
    ));
    assert_eq!(w, ["ABCDEFGH"]);
    // two surveys more: the hooks of the survey run three times
    let w = words(&format!(
        r"\makeatletter \newcount\surveys
          \datavisualization [xy Cartesian, visualize as line, before survey={{\global\advance\surveys by 1\relax}},
            at end survey={{\tikra@dv@main.get survey count(\n)\ifnum\n<3\relax\tikra@dv@main.do an additional survey()\fi}}]
            {TWO_POINTS} info {{ \node at (0,-1) {{\the\surveys}}; }};"
    ));
    assert_eq!(w, ["3"]);
}

/// A scaling mapper's methods: its in range (an interval, with its min and max), its out
/// attribute and its function.
#[test]
fn a_scaling_mappers_methods() {
    let x = r"\pgfkeysvalueof{/tikz/data visualization/x axis/scaling mapper}";
    let y = r"\pgfkeysvalueof{/tikz/data visualization/y axis/scaling mapper}";
    let w = words(&format!(
        r"\datavisualization [xy Cartesian, visualize as line, y axis={{logarithmic}}] data point [x=0, y=1] data point [x=3, y=10]
          info {{ {x}.get in range interval() \pgfdvinrangeinterval.get min and max() {x}.get out() {x}.get function(\f) {y}.get function(\g)
            \node at (0,-1) {{\pgfdvmin/\pgfdvmax, \pgfdvout, \ifx\f\empty linear\else log\fi, \ifx\g\empty linear\else log\fi}}; }};"
    ));
    assert_eq!(w, ["0/3, x/scaled, linear, log"]);
    // `set in to(max)` sets the attribute to the end of the range
    let w = words(&format!(
        r"\datavisualization [xy Cartesian, visualize as line] {TWO_POINTS} info {{ \begingroup {x}.set in to(max) \node at (0,-1) {{\pgfkeysvalueof{{/data point/x}}}}; \endgroup }};"
    ));
    assert_eq!(w, ["3"]);
}

/// `include in value(10)` before the survey widens the range: the axis goes to 10.
#[test]
fn a_mapper_includes_a_value() {
    let c = chart(&format!(
        r"\datavisualization [xy Cartesian, visualize as line, before survey={{\pgfkeysvalueof{{/tikz/data visualization/x axis/scaling mapper}}.include in value(10)}}, x axis={{visualize axis}}] {TWO_POINTS};"
    ));
    let axis = paths(&c).into_iter().find(|p| p.stroke.as_ref().is_some_and(|s| (s.width - 0.6 * PT).abs() > 1e-9)).unwrap();
    assert!(near(relative(&axis)[1], (10.0, 0.0)), "{:?}", relative(&axis));
}

/// `new object`: an interval made and stored, its ends set by method (one argument of two kept);
/// a second object for the same key is not made.
#[test]
fn new_objects_are_made_once_per_key() {
    let w = words(&format!(
        r"\datavisualization [xy Cartesian, visualize as line,
            new object={{class=interval, store=/tikz/data visualization/my range, after creation={{\tikzdvobj.set min(2)\tikzdvobj.let max(7, 9)\gdef\who{{first}}}}}},
            new object={{class=interval, store=/tikz/data visualization/my range, after creation={{\gdef\who{{second}}}}}}]
          {TWO_POINTS} info {{ \pgfkeysvalueof{{/tikz/data visualization/my range}}.get min and max() \node at (0,-1) {{\pgfdvmin..\pgfdvmax, \who}}; }};"
    ));
    assert_eq!(w, ["2..7, first"]);
}

/// An argument with a handler (`arg1/.expanded=`) is expanded where it is given, the others when the
/// object is made: here after `before creation` made the macro z. Summing y, the running total
/// goes up; summing z, y has no sums and the line stays flat.
#[test]
fn expanded_arguments_of_new_objects() {
    for (arg, end) in [("arg1/.expanded", (1.0, 2.0)), ("arg1", (1.0, 0.0))] {
        let c = chart(&format!(
            r"\def\summed{{y}}
              \datavisualization [xy Cartesian, visualize as line, new object={{class=accumulator, {arg}=\summed, before creation=\def\summed{{z}}}}, y axis={{attribute=y/sum}}]
              data point [x=0, y=1, z=10] data point [x=1, y=2, z=20];"
        ));
        assert!(same_line(&data_line(&c), &[(0.0, 0.0), end]), "{arg}: {:?}", data_line(&c));
    }
}

/// `new object`'s other keys: `before creation`, an argument from a key, an object made later
/// (`when=`); keys that are not arguments are passed over, and an object without a class is not made.
#[test]
fn new_object_options() {
    let w = words(&format!(
        r"\datavisualization [xy Cartesian, visualize as line,
            new object={{class=accumulator, arg1 from key=/tikz/data visualization/y axis/attribute, arg9=x, colour=red, when=after survey,
              before creation=\gdef\made{{before}}, store=/tikz/data visualization/acc}},
            new object={{store=/tikz/data visualization/nothing}}]
          {TWO_POINTS} info {{ \node at (0,-1) {{\made, \pgfkeysvalueof{{/data point/y/sum}}}}; }};"
    ));
    // the data points' sums are local to them: back to 0 after the visualization
    assert_eq!(w, ["before, 0"]);
}

/// What `new object` and method calls cannot do is told.
#[test]
fn unknown_classes_methods_and_objects() {
    warns(&format!(r"\datavisualization [xy Cartesian, visualize as line, new object={{class=spline}}] {TWO_POINTS};"), "class 'spline' not supported");
    warns(
        &format!(r"\datavisualization [xy Cartesian, visualize as line, new object={{class=interval, after creation={{\tikzdvobj.frobnicate()}}}}] {TWO_POINTS};"),
        "method 'frobnicate' not supported",
    );
    warns(
        &format!(r"\datavisualization [xy Cartesian, visualize as line, new object={{class=interval, after creation={{\tikzdvobj\relax}}}}] {TWO_POINTS};"),
        "method '' not supported",
    );
    warns(&format!(r"\makeatletter \datavisualization [xy Cartesian, visualize as line] {TWO_POINTS} info {{ \tikra@dv@obj{{99}}.get out() }};"), "no object 99 for .get out()");
    let long = "x".repeat(120);
    warns(
        &format!(r"\datavisualization [xy Cartesian, visualize as line, new object={{class=interval, after creation={{\tikzdvobj.{long}}}}}] {TWO_POINTS};"),
        "method call without '('",
    );
}

/// A counter counts the data points from its start value by its step; set to a value, it goes on
/// from there.
#[test]
fn counters() {
    let w = words(
        r"\datavisualization [xy Cartesian, visualize as line,
            new object={class=count, arg1=c, store=/tikz/data visualization/counter, after creation={\tikzdvobj.set start value(10)\tikzdvobj.set step(5)}},
            at start visualization={\pgfkeysvalueof{/tikz/data visualization/counter}.set value(100)}]
          data point [x=0, y=0] data point [x=3, y=1] data point [x=4, y=2]
          info { \pgfkeysvalueof{/tikz/data visualization/counter}.get max(\m) \node at (0,-1) {\m}; };",
    );
    assert_eq!(w, ["115"]);
}

/// An accumulator sums an attribute over the data points: on an axis of the sums, a running total
/// (reset by its method).
#[test]
fn accumulators() {
    let c = chart(
        "\\datavisualization [xy Cartesian, visualize as line, new object={class=accumulator, arg1=y, store=/tikz/data visualization/acc}, y axis={attribute=y/sum}]\n\
         data [format=TeX code] {\n\
           \\pgfkeys{/data point/x=0, /data point/y=1}\\pgfdatapoint\n\
           \\pgfkeys{/data point/x=1, /data point/y=2}\\pgfdatapoint\n\
           \\pgfkeysvalueof{/tikz/data visualization/acc}.reset()\n\
           \\pgfkeys{/data point/x=2, /data point/y=4}\\pgfdatapoint\n\
         };",
    );
    // sums 1, 3, then 4
    assert!(same_line(&data_line(&c), &[(0.0, 0.0), (1.0, 2.0), (2.0, 3.0)]), "{:?}", data_line(&c));
}

/// A linear transformer of one's own adds an attribute's direction (here z, upward); its origin
/// moves where it puts the points; a unit vector that is not a point moves nothing.
#[test]
fn linear_transformers() {
    let c = chart(
        r"\datavisualization [xy Cartesian, visualize as line, new axis base=z axis, z axis={attribute=z, scaling=0 at 0pt and 1 at 1cm},
            new object={class=linear transformer, arg1=z/scaled, arg2=\pgfqpoint{0pt}{1pt}}, new object={class=linear transformer, arg1=x/scaled, arg2=nothing}]
          data point [x=0, y=0, z=0] data point [x=1, y=0, z=2];",
    );
    assert!(same_line(&data_line(&c), &[(0.0, 0.0), (1.0, 2.0)]), "{:?}", data_line(&c));
    let c = chart(&format!(
        r"\datavisualization [xy Cartesian, visualize as line] {TWO_POINTS} info {{ \pgfkeysvalueof{{/tikz/data visualization/x axis/linear transformer}}.set origin(\pgfqpoint{{0pt}}{{28.45274pt}}) \draw (visualization cs:x=0,y=0) -- (visualization cs:x=3,y=0); }};"
    ));
    let all = paths(&c);
    let (line, moved) = (points(&all[0].segments), points(&all[1].segments));
    assert!(near((moved[0].0 - line[0].0, moved[0].1 - line[0].1), (0.0, 1.0)), "{moved:?} {line:?}");
}

/// A new axis base: a mapper without a transformer, its range surveyed.
#[test]
fn an_axis_base_has_a_range() {
    let w = words(
        r"\datavisualization [xy Cartesian, visualize as line, new axis base=size axis, size axis={attribute=size}]
          data point [x=0, y=0, size=2] data point [x=3, y=1, size=5]
          info { \pgfkeysvalueof{/tikz/data visualization/size axis/scaling mapper}.get in range interval() \pgfdvinrangeinterval.get min and max() \node at (0,-1) {\pgfdvmin-\pgfdvmax}; };",
    );
    assert_eq!(w, ["2-5"]);
}

/// A polar transformer tells its attributes.
#[test]
fn a_polar_transformers_attributes() {
    let t = r"\pgfkeysvalueof{/tikz/data visualization/angle axis/polar transformer}";
    let w = words(&format!(
        r"\datavisualization [new polar axes, angle axis={{degrees}}, radius axis={{unit length=1cm}}, visualize as line] data point [angle=0, radius=1] data point [angle=90, radius=2]
          info {{ {t}.get angle attribute(\a) {t}.get radius attribute(\r) \node at (0,-1) {{\a, \r}}; }};"
    ));
    assert_eq!(w, ["angle/scaled, radius/scaled"]);
}

/// Axes made twice are one axis each: `all axes` gives each its options once.
#[test]
fn axes_made_twice() {
    let c = chart(&format!(r"\datavisualization [xy Cartesian, xy Cartesian, visualize as line, visualize as line, all axes={{visualize axis}}] {TWO_POINTS};"));
    // the data, and the two axes
    assert_eq!(paths(&c).len(), 3);
    assert_eq!(data_paths(&c).len(), 1);
}

/// A key of the data visualization it does not know is a TikZ key, for what it draws; one TikZ does
/// not know either is told.
#[test]
fn unknown_keys_are_tikz_keys() {
    let c = chart(&format!(r"\datavisualization [xy Cartesian, visualize as line, dashed] {TWO_POINTS};"));
    assert!(data_paths(&c)[0].stroke.as_ref().unwrap().dash.is_some());
    warns(&format!(r"\datavisualization [xy Cartesian, visualize as line, no such key] {TWO_POINTS};"), "no such key");
}

/// `data/…` keys are the data's: `data/format=named` reads the data sets as named values.
#[test]
fn data_keys_among_the_options() {
    let c = chart("\\datavisualization [xy Cartesian, visualize as line, data/format=named] data {\n x=0, y=0\n x=1, y=2\n};");
    assert!(same_line(&data_line(&c), &[(0.0, 0.0), (1.0, 2.0)]));
}

// ---------------------------------------------------------------- visualizers

/// `visualize as smooth cycle`: a closed curve through the points (a closed polygon for two).
#[test]
fn smooth_cycles() {
    let c = chart(r"\datavisualization [xy Cartesian, visualize as smooth cycle] data point [x=0, y=0] data point [x=1, y=0] data point [x=1, y=1] data point [x=0, y=1];");
    let line = &data_paths(&c)[0];
    let curves = line.segments.iter().filter(|s| matches!(s, tikra::scene::Segment::CurveTo(..))).count();
    assert_eq!(curves, 4);
    assert!(matches!(line.segments.last(), Some(tikra::scene::Segment::Close)));
    let c = chart(r"\datavisualization [xy Cartesian, visualize as smooth cycle] data point [x=0, y=0] data point [x=1, y=0];");
    let line = &data_paths(&c)[0];
    assert!(same_line(&relative(line), &[(0.0, 0.0), (1.0, 0.0)]) && matches!(line.segments.last(), Some(tikra::scene::Segment::Close)));
}

/// A smooth line through one point draws nothing; outliers are not drawn.
#[test]
fn nothing_to_draw() {
    let c = chart(r"\datavisualization [xy Cartesian, visualize as smooth line] data point [x=0, y=0];");
    assert!(data_paths(&c).is_empty());
    let c = chart(r"\datavisualization [xy Cartesian, visualize as line] data point [x=0, y=0, outlier] data point [x=1, y=0, outlier];");
    assert!(paths(&c).is_empty());
}

/// `straight cycle` closes the line; `no lines` leaves only the marks.
#[test]
fn cycles_and_marks_only() {
    let c = chart(r"\datavisualization [xy Cartesian, visualize as line=a, a={straight cycle}] data point [x=0, y=0] data point [x=1, y=0] data point [x=1, y=1];");
    let line = &data_paths(&c)[0];
    assert!(same_line(&relative(line), &[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)]));
    assert!(matches!(line.segments.last(), Some(tikra::scene::Segment::Close)));
    let c = chart(r"\datavisualization [xy Cartesian, visualize as line=a, a={no lines, style={mark=x}}] data point [x=0, y=0] data point [x=1, y=0] data point [x=1, y=1];");
    // three marks (x: two strokes each), no line
    assert_eq!(paths(&c).len(), 3);
    assert!(paths(&c).iter().all(|p| points(&p.segments).len() == 4 && relative(p).iter().all(|q| q.0.abs() < 0.2 && q.1.abs() < 0.2)));
}

/// A visualizer's filter and its projection code, set by method: points with x above 1 left out,
/// and every point put on the x axis.
#[test]
fn filters_and_projections() {
    let v = r"\pgfkeysvalueof{/tikz/data visualization/visualizers/line}";
    let c = chart(&format!(
        r"\datavisualization [xy Cartesian, visualize as line, before visualization={{{v}.set filter(\ifdim\pgfkeysvalueof{{/data point/x}}pt>1pt\relax\pgfdvfilterpassedfalse\fi)}}]
          data point [x=0, y=0] data point [x=1, y=1] data point [x=2, y=0];"
    ));
    assert!(same_line(&data_line(&c), &[(0.0, 0.0), (1.0, 1.0)]), "{:?}", data_line(&c));
    let c = chart(&format!(
        r"\datavisualization [xy Cartesian, visualize as line, before visualization={{{v}.set projection code(\pgfkeys{{/data point/canvas y=0pt}})}}]
          data point [x=0, y=0] data point [x=1, y=1] data point [x=2, y=0];"
    ));
    assert!(same_line(&data_line(&c), &[(0.0, 0.0), (1.0, 0.0), (2.0, 0.0)]), "{:?}", data_line(&c));
}

/// `visualize as rectangles`: a rectangle from each data point's `min` and `max` of x and y, and in
/// the legend a small rectangle.
#[test]
fn rectangles() {
    let c = chart("\\datavisualization [xy Cartesian, visualize as rectangles=bars, bars={label in legend={text=bars}}] data [headline={x/min, x/max, y/min, y/max}] {\n 0, 1, 0, 2\n 2, 3, 0, 1\n};");
    let all = data_paths(&c);
    assert_eq!(all.len(), 3);
    assert!(same_line(&relative(&all[0]), &[(0.0, 0.0), (0.0, 2.0), (1.0, 2.0), (1.0, 0.0), (0.0, 0.0)]));
    assert!(same_line(&relative(&all[1]), &[(0.0, 0.0), (0.0, 1.0), (1.0, 1.0), (1.0, 0.0), (0.0, 0.0)]));
    // the second one 2cm right of the first
    assert!((points(&all[1].segments)[0].0 - points(&all[0].segments)[0].0 - 2.0).abs() < 1e-6);
    // the legend's: 1ex wide
    let sample = relative(&all[2]);
    assert_eq!(sample.len(), 4);
    assert!(sample.iter().all(|p| p.0 >= -1e-9 && p.0 < 0.2 && p.1 >= -1e-9), "{sample:?}");
    assert_eq!(
        words(
            "\\datavisualization [xy Cartesian, visualize as rectangles=bars, bars={label in legend={text=bars}}] data [headline={x/min, x/max, y/min, y/max}] {\n 0, 1, 0, 2\n};"
        )[0],
        "bars"
    );
}

// ---------------------------------------------------------------- styles

/// `ignore style sheets`: the visualizer keeps its own color while the others take the sheet's.
#[test]
fn ignoring_style_sheets() {
    let c = chart(
        r"\datavisualization [xy Cartesian, visualize as line=a, visualize as line=b, a={ignore style sheets}, style sheet=strong colors] data point [x=0, y=0, set=a] data point [x=1, y=1, set=a] data point [x=0, y=1, set=b] data point [x=1, y=0, set=b];",
    );
    let lines = data_paths(&c);
    let a = lines.iter().find(|l| relative(l)[1].1 > 0.0).unwrap();
    let b = lines.iter().find(|l| relative(l)[1].1 < 0.0).unwrap();
    assert!(close_color(stroke_color(a), 0.0, 0.0, 0.0));
    assert!(!close_color(stroke_color(b), 0.0, 0.0, 0.0), "{:?}", stroke_color(b));
}

/// `/data point/set/.style sheet=…` is a style sheet for the visualizers, as `style sheet=` is.
#[test]
fn a_style_sheet_for_an_attribute() {
    let c = chart(
        r"\datavisualization [xy Cartesian, visualize as line/.list={a,b}, /data point/set/.style sheet=strong colors] data point [x=0, y=0, set=a] data point [x=1, y=0, set=a] data point [x=0, y=1, set=b] data point [x=1, y=1, set=b];",
    );
    let colors: Vec<Color> = data_paths(&c).iter().map(stroke_color).collect();
    // red!80!black for the second
    assert!(colors.iter().any(|c| close_color(*c, 0.8, 0.0, 0.0)), "{colors:?}");
}

/// xcolor's hsb as rgb.
fn hsb(h: f64, s: f64, b: f64) -> (f64, f64, f64) {
    let h = (h - h.floor()) * 6.0;
    let i = h.floor();
    let f = h - i;
    let (p, q, t) = (b * (1.0 - s), b * (1.0 - s * f), b * (1.0 - s * (1.0 - f)));
    match i as i32 {
        0 => (b, t, p),
        1 => (q, b, p),
        2 => (p, b, t),
        3 => (p, q, b),
        4 => (t, p, b),
        _ => (b, p, q),
    }
}

/// `vary hue`: the n-th visualizer's hue is .4 + .213 n (around the circle, through the six
/// sectors of hsb), saturation .9, brightness .8.
#[test]
fn vary_hue_goes_around_the_circle() {
    let names = ["a", "b", "c", "d", "e", "f", "g"];
    let mut body = String::from(r"\datavisualization [xy Cartesian, visualize as line/.list={a,b,c,d,e,f,g}, style sheet=vary hue]");
    for (i, n) in names.iter().enumerate() {
        body.push_str(&format!(" data point [x=0, y={i}, set={n}] data point [x=1, y={i}, set={n}]"));
    }
    body.push(';');
    let c = chart(&body);
    let lines = data_paths(&c);
    let bottom = points(&lines.iter().min_by(|a, b| points(&a.segments)[0].1.total_cmp(&points(&b.segments)[0].1)).unwrap().segments)[0].1;
    for line in &lines {
        let n = (points(&line.segments)[0].1 - bottom).round() + 1.0;
        let (r, g, b) = hsb(0.4 + 0.213 * n, 0.9, 0.8);
        assert!(close_color(stroke_color(line), r, g, b), "{n}: {:?} vs {r} {g} {b}", stroke_color(line));
    }
}

/// A color series of one's own in rgb or gray: start plus n steps, each part taken modulo 1.
#[test]
fn color_series_in_rgb_and_gray() {
    for (model, start, step, first, second) in [("rgb", "0.5,0,1", "0.75,0.5,0", (0.25, 0.5, 1.0), (0.0, 1.0, 1.0)), ("gray", "0.2", "0.3", (0.5, 0.5, 0.5), (0.8, 0.8, 0.8))] {
        let c = chart(&format!(
            r"\tikzdvdeclarestylesheetcolorseries{{mine}}{{{model}}}{{{start}}}{{{step}}}
              \datavisualization [xy Cartesian, visualize as line/.list={{a,b}}, style sheet=mine] data point [x=0, y=0, set=a] data point [x=1, y=0, set=a] data point [x=0, y=1, set=b] data point [x=1, y=1, set=b];"
        ));
        let lines = data_paths(&c);
        // b above a, drawn first
        let (b, a) = (stroke_color(&lines[0]), stroke_color(&lines[1]));
        assert!(close_color(a, first.0, first.1, first.2), "{model}: {a:?}");
        assert!(close_color(b, second.0, second.1, second.2), "{model}: {b:?}");
    }
}

// ---------------------------------------------------------------- labels and pins in data

/// `label in data`: by `index=`, by `pos=` (a fraction of the points) or by default; a second label
/// of the same visualizer counts the same points.
#[test]
fn labels_in_data() {
    for how in ["index=2", "pos=0.5"] {
        let c = chart(&format!(
            r"\datavisualization [xy Cartesian, visualize as line=a, a={{label in data={{text=here, {how}}}}}] data point [x=0, y=0, set=a] data point [x=1, y=1, set=a] data point [x=2, y=0, set=a];"
        ));
        // beside the peak, on the left of the way up
        let (_, place) = common::texts(&c)[0].clone();
        let peak = points(&data_paths(&c)[0].segments)[1];
        assert!(place.e < peak.0 && place.f > peak.1 - 0.5, "{how}: {place:?} {peak:?}");
    }
    let w = words(
        r"\datavisualization [xy Cartesian, visualize as line=a, a={label in data={text=A}, label in data={text=B}}] data point [x=0, y=0, set=a] data point [x=3, y=1, set=a];",
    );
    assert_eq!(w, ["A", "B"]);
}

/// `pin in data`: the label a pin's length off the line, on its left (on its right with `text'`)
/// or at `pin angle`, with an edge to the place; at the first point, where the line has no
/// direction yet, there is no edge.
#[test]
fn pins_in_data() {
    let thin = |c: &Canvas| paths(c).into_iter().filter(|p| p.stroke.as_ref().is_some_and(|s| (s.width - 0.6 * PT).abs() > 1e-9)).collect::<Vec<_>>();
    let data = "data point [x=0, y=0, set=a] data point [x=1, y=1, set=a] data point [x=2, y=0, set=a];";
    // where the edge starts, from the peak: up left of the way up, down right, or well above
    fn up_left(d: (f64, f64)) -> bool {
        d.0 < 0.0 && d.1 > 0.0
    }
    fn down_right(d: (f64, f64)) -> bool {
        d.0 > 0.0 && d.1 < 0.0
    }
    fn above(d: (f64, f64)) -> bool {
        d.1 > 0.8
    }
    type Side = fn((f64, f64)) -> bool;
    let cases: [(&str, Side); 3] =
        [("text=pinned, when=x is 1", up_left), ("text'=pinned, when=x is 1", down_right), ("text=pinned, when=x is 1, pin angle=90, pin length=1cm", above)];
    for (options, side) in cases {
        let c = chart(&format!(r"\datavisualization [xy Cartesian, visualize as line=a, a={{pin in data={{{options}}}}}] {data}"));
        let edge = thin(&c);
        assert_eq!(edge.len(), 1, "{options}");
        let e = points(&edge[0].segments);
        let peak = points(&data_paths(&c)[0].segments)[1];
        // from the label to the peak
        assert!(near(e[1], peak), "{options}: {e:?} {peak:?}");
        let d = (e[0].0 - e[1].0, e[0].1 - e[1].1);
        assert!(side(d), "{options}: {d:?}");
    }
    let c = chart(r"\datavisualization [xy Cartesian, visualize as line=a, a={pin in data={text=pinned, when=x is 0}}] data point [x=0, y=0, set=a] data point [x=1, y=1, set=a];");
    assert!(thin(&c).is_empty());
    assert_eq!(common::texts(&c).len(), 1);
}

// ---------------------------------------------------------------- legends

/// The texts of four legend entries, `1` to `4`, with the legend's options, in the order they are
/// drawn (row by row).
fn legend_order(options: &str, entries: usize) -> Vec<String> {
    let list: Vec<String> = (1..=entries).map(|i| format!("new legend entry={{text={i}}}")).collect();
    let w = words(&format!(r"\datavisualization [xy Cartesian, visualize as line, {}, legend={{{options}}}] {TWO_POINTS};", list.join(", ")));
    w[..entries.min(w.len())].to_vec()
}

/// The eight ways to fill a legend's rows and columns.
#[test]
fn legend_strategies() {
    for (strategy, order) in [
        ("down then right", "1324"),
        ("right then down", "1234"),
        ("up then right", "2413"),
        ("right then up", "3412"),
        ("left then up", "4321"),
        ("up then left", "4231"),
        ("left then down", "2143"),
        ("down then left", "3142"),
    ] {
        assert_eq!(legend_order(&format!("columns=2, {strategy}"), 4).concat(), order, "{strategy}");
    }
}

/// `max rows=2`: five entries in two rows of three columns; `rows=`; `max columns=2`.
#[test]
fn legend_rows_and_columns() {
    assert_eq!(legend_order("max rows=2", 5).concat(), "13524");
    assert_eq!(legend_order("rows=1", 3).concat(), "123");
    // two columns: three rows, filled down then right
    assert_eq!(legend_order("max columns=2", 5).concat(), "14253");
}

/// Entries with only their text, or with their text on the left of three marks.
#[test]
fn legend_entry_styles() {
    let c = chart(
        r"\datavisualization [xy Cartesian, visualize as line=a, a={label in legend={text=A, text only}}, visualize as scatter=b, b={label in legend={text=B, text left, label in legend three marks}}]
          data point [x=0, y=0, set=a] data point [x=3, y=1, set=a] data point [x=0, y=1, set=b];",
    );
    // the line and its one mark in the data; three marks in the legend, no line
    assert_eq!(data_paths(&c).len(), 1);
    assert_eq!(paths(&c).len(), 5);
    let texts = common::texts(&c);
    let marks: Vec<f64> = paths(&c).iter().skip(2).map(|p| points(&p.segments)[0].0).collect();
    // B's text on the left of its marks
    let b = texts.iter().map(|(_, t)| t.e).fold(f64::MAX, f64::min);
    assert!(marks.iter().all(|m| *m > b), "{marks:?} {b}");
}

/// `new visualizer={name}{options}{legend options}` makes a visualizer's keys (not its object): its
/// legend entry is drawn.
#[test]
fn new_visualizers_have_legend_entries() {
    let w = words(&format!(
        r"\datavisualization [xy Cartesian, visualize as line, new visualizer={{points}}{{style={{mark=*}}}}{{}}, points={{label in legend={{text=P}}}}] {TWO_POINTS};"
    ));
    assert_eq!(w[0], "P");
}

/// A legend's arrangement, by method: its entries in rows and columns, here drawn a second time in
/// a matrix of one's own.
#[test]
fn a_legends_arrangement() {
    let body = r"\datavisualization [xy Cartesian, visualize as line=a, a={label in legend={text=A}}, visualize as line=b, b={label in legend={text=B}}]
        data point [x=0, y=0, set=a] data point [x=3, y=1, set=a]";
    assert_eq!(common::texts(&chart(&format!("{body};"))).len(), 2);
    let c = chart(&format!(r"{body} info {{ \pgfkeysvalueof{{/tikz/data visualization/main legend/obj}}.get arrangement(\entries) \matrix at (0,-2) {{\entries}}; }};"));
    assert_eq!(common::texts(&c).len(), 4);
}

/// The methods that get something do nothing without a macro to put it in.
#[test]
fn getting_into_nothing() {
    let m = r"\pgfkeysvalueof{/tikz/data visualization/x axis/scaling mapper}";
    let t = r"\pgfkeysvalueof{/tikz/data visualization/angle axis/polar transformer}";
    let l = r"\pgfkeysvalueof{/tikz/data visualization/main legend/obj}";
    let c = chart(&format!(
        r"\datavisualization [new polar axes, xy Cartesian, angle axis={{degrees}}, radius axis={{unit length=1cm}}, visualize as line, new legend,
            new object={{class=count, arg1=c, after creation={{\tikzdvobj.get max()}}}}]
          data point [x=0, y=0, angle=0, radius=0] data point [x=1, y=1, angle=0, radius=0]
          info {{ {m}.get function() {t}.get angle attribute() {t}.get radius attribute() {l}.get arrangement() }};"
    ));
    assert!(same_line(&data_line(&c), &[(0.0, 0.0), (1.0, 1.0)]), "{:?}", data_line(&c));
    assert!(common::texts(&c).is_empty());
}

/// Rectangles and lines in one data visualization: each draws its own set; a data point without all
/// four ends of its rectangle draws none; a legend rectangle without corners is not drawn.
#[test]
fn rectangles_among_other_visualizers() {
    let c = chart(
        "\\datavisualization [xy Cartesian, visualize as rectangles=bars, visualize as line=trend, bars={label in legend={text=bars, label in legend rectangle coordinates=}}]\n\
         data [set=bars, headline={x/min, x/max, y/min, y/max}] {\n 0, 1, 0, 2\n 2, 3, 0, 1\n}\n\
         data [set=bars, headline={x/min, x/max, y/min}] {\n 4, 5, 0\n}\n\
         data point [x=0, y=0, set=trend] data point [x=3, y=3, set=trend];",
    );
    let all = data_paths(&c);
    // two rectangles and the line; no sample in the legend
    assert_eq!(all.len(), 3, "{all:?}");
    assert_eq!(relative(&all[0]).len(), 5);
    assert_eq!(relative(&all[1]).len(), 5);
    assert!(same_line(&relative(&all[2]), &[(0.0, 0.0), (3.0, 3.0)]));
    assert_eq!(common::texts(&c).len(), 1);
}

/// Visualizers and axes asked for twice are made once.
#[test]
fn made_once() {
    let rects = "data [headline={x/min, x/max, y/min, y/max}] {\n 0, 1, 0, 2\n 2, 3, 0, 1\n};";
    let once = chart(&format!("\\datavisualization [xy Cartesian, visualize as rectangles=r, r={{style={{red}}}}] {rects}"));
    let twice = chart(&format!("\\datavisualization [xy Cartesian, visualize as rectangles=r, visualize as rectangles=r, r={{style={{red}}}}] {rects}"));
    assert_eq!(data_paths(&once).len(), 2);
    assert_eq!(data_paths(&twice).len(), 2);
    // one polar transformer: a quarter circle of radius 1cm, not drawn twice as far
    let polar = "angle axis={degrees}, radius axis={unit length=1cm}, visualize as line] data point [angle=0, radius=1] data point [angle=90, radius=1];";
    let twice = chart(&format!(r"\datavisualization [new polar axes, new polar axes, {polar}"));
    assert_eq!(paths(&twice).len(), 1);
    assert!(near(*data_line(&twice).last().unwrap(), (-1.0, 1.0)), "{:?}", data_line(&twice));
}

/// A legend asked for in a scope (where its keys are local) and again after it is still one legend.
#[test]
fn one_legend_from_a_scope_and_after_it() {
    let c = chart(
        r"\datavisualization [xy Cartesian, visualize as line=a]
          scope [/tikz/data visualization/a={label in legend={text=A}}] { data point [x=0, y=0, set=a] }
          data point [x=3, y=1, set=a] [legend={below}];",
    );
    assert_eq!(common::texts(&c).len(), 1);
}

/// A label in data `when=size is 5`: at the first point whose size reaches 5 (the points without a
/// size do not count).
#[test]
fn a_label_when_an_attribute_reaches_a_value() {
    let c = chart(
        r"\datavisualization [xy Cartesian, visualize as line=a, a={label in data={text=L, when=size is 5}}] data point [x=0, y=0, set=a] data point [x=1, y=1, size=5, set=a] data point [x=2, y=0, size=7, set=a];",
    );
    let (_, place) = common::texts(&c)[0].clone();
    let line = points(&data_paths(&c)[0].segments);
    // beside the second point, not the third
    assert!((place.e - line[1].0).abs() < 1.0 && place.e < line[2].0 - 0.5, "{place:?} {line:?}");
}

/// tikra's own command that makes the objects keys ask for warns about a kind it does not know.
#[test]
fn an_object_of_no_kind() {
    warns(&format!(r"\makeatletter \datavisualization [xy Cartesian, visualize as line] {TWO_POINTS} info {{ \tikra@dv@make{{spline}} }};"), "no object 'spline'");
}
