//! Data visualizations: how the data is given (`\datavisualization`'s blocks, the table, named and
//! function formats, data groups and scopes) and how the axes show it (scalings, logarithmic axes,
//! tick placement, tick texts and their number formats, axis labels).

mod common;

use common::{drawn_and_warnings, paths, points, Drawn};
use tikra::canvas::Canvas;
use tikra::PT;

/// A data visualization's canvas, without warnings (moved so that its box starts at (0, 0)).
fn chart(body: &str) -> Canvas {
    let (c, w) = drawn_and_warnings(body);
    assert!(w.is_empty(), "{w:?}\n{body}");
    c
}

/// The warnings of a picture body.
fn warnings(body: &str) -> Vec<String> {
    drawn_and_warnings(body).1
}

/// Requires a warning containing `fragment`.
fn warns(body: &str, fragment: &str) {
    let w = warnings(body);
    assert!(w.iter().any(|w| w.contains(fragment)), "expected '{fragment}' in {w:?}\n{body}");
}

/// The texts of a picture, in drawing order, as its description reads them out.
fn words(body: &str) -> Vec<String> {
    let r = tikra::convert(&format!("\\begin{{tikzpicture}}{body}\\end{{tikzpicture}}")).unwrap_or_else(|e| panic!("{e}\n{body}"));
    assert!(r.warnings.is_empty(), "{:?}\n{body}", r.warnings);
    let Some((_, list)) = r.description.split_once("Text in the chart: ") else { return Vec::new() };
    list.strip_suffix('.').unwrap_or(list).split("; ").map(str::to_string).collect()
}

/// The paths drawn semithick (what the visualizers draw), in order.
fn data_paths(c: &Canvas) -> Vec<Drawn> {
    paths(c).into_iter().filter(|p| p.stroke.as_ref().is_some_and(|s| (s.width - 0.6 * PT).abs() < 1e-9)).collect()
}

/// The points of the first data path, from its first point, in cm.
fn data_line(c: &Canvas) -> Vec<(f64, f64)> {
    let line = data_paths(c).into_iter().next().expect("no data line");
    let p = points(&line.segments);
    p.iter().map(|q| (q.0 - p[0].0, q.1 - p[0].1)).collect()
}

fn near(a: (f64, f64), b: (f64, f64)) -> bool {
    (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3
}

fn same_line(got: &[(f64, f64)], want: &[(f64, f64)]) -> bool {
    got.len() == want.len() && got.iter().zip(want).all(|(a, b)| near(*a, *b))
}

/// The x positions of the short vertical paths (ticks) at the bottom of a scientific axes box,
/// left to right, from the first.
fn bottom_ticks(c: &Canvas) -> Vec<f64> {
    let mut xs: Vec<f64> = paths(c)
        .iter()
        .map(|p| points(&p.segments))
        .filter(|p| p.len() == 2 && (p[0].0 - p[1].0).abs() < 1e-9 && (p[0].1 - p[1].1).abs() < 0.1 && p[0].1.min(p[1].1) < 0.5)
        .map(|p| p[0].0)
        .collect();
    xs.sort_by(f64::total_cmp);
    xs.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    xs.iter().map(|x| x - xs[0]).collect()
}

/// The y positions of the short horizontal paths (ticks) at the left of a scientific axes box,
/// bottom to top, from the first.
fn left_ticks(c: &Canvas) -> Vec<f64> {
    let mut ys: Vec<f64> = paths(c)
        .iter()
        .map(|p| points(&p.segments))
        .filter(|p| p.len() == 2 && (p[0].1 - p[1].1).abs() < 1e-9 && (p[0].0 - p[1].0).abs() < 0.1 && p[0].0.min(p[1].0) < 1.2)
        .map(|p| p[0].1)
        .collect();
    ys.sort_by(f64::total_cmp);
    ys.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    ys.iter().map(|y| y - ys[0]).collect()
}

const TWO_POINTS: &str = "data point [x=0, y=0] data point [x=3, y=1]";

// ---------------------------------------------------------------- the blocks of \datavisualization

/// A blank line between the blocks is a `\par`, which does not end the data visualization.
#[test]
fn a_blank_line_between_blocks_is_ignored() {
    let c = chart("\\datavisualization [xy Cartesian, visualize as line] data point [x=0, y=0]\n\n data point [x=2, y=1];");
    assert!(same_line(&data_line(&c), &[(0.0, 0.0), (2.0, 1.0)]));
}

/// `data point={…}` among the options adds a data point, as `data point […]` does.
#[test]
fn the_data_point_key_adds_a_point() {
    let c = chart("\\datavisualization [xy Cartesian, visualize as line, data point={x=0, y=0}, data point={x=1, y=2}];");
    assert!(same_line(&data_line(&c), &[(0.0, 0.0), (1.0, 2.0)]));
}

/// What does not belong in a data visualization is an error (the statement is dropped) or a warning.
#[test]
fn malformed_blocks_are_reported() {
    for (blocks, message) in [
        // no semicolon before the end of the picture
        ("data point [x=0, y=0]", "semicolon expected before '\\end'"),
        ("data point [x=0, y=0] foo;", "semicolon expected before 'foo'"),
        ("data point [x=0, y=0] = ;", "semicolon expected before '='"),
        ("data pint [x=0, y=0];", "'data pint' not understood"),
        ("data gap [x=0, y=0];", "'data gap' not understood"),
        ("data [format=table] x;", "opening brace expected"),
        ("data [read from file=points.csv];", "external data files are not read"),
        ("scope [/data point/y=1] data point [x=0];", "scope without '{'"),
        ("scope [/data point/y=1] { data point [x=0] data point [x=1] ;", "a scope or data group is not closed"),
        ("data group {g} = data point [x=0, y=0];", "data group without '{'"),
        ("data group {nowhere};", "undefined data group 'nowhere'"),
        ("data [use group=nowhere];", "undefined data group 'nowhere'"),
        ("data [format=binary] { 1 2 };", "unknown data format 'binary'"),
    ] {
        warns(&format!("\\datavisualization [xy Cartesian, visualize as line] {blocks}"), message);
    }
    // the statement is dropped: no data line
    let (c, _) = drawn_and_warnings("\\datavisualization [xy Cartesian, visualize as line] data point [x=0, y=0] data point [x=1, y=1] foo;");
    assert!(data_paths(&c).is_empty());
}

/// A document that ends in the middle of the blocks: `info` without its code, a data group's name
/// without anything after it.
#[test]
fn blocks_cut_off_by_the_end_of_the_document() {
    for (doc, message) in [
        (r"\begin{tikzpicture}\datavisualization [xy Cartesian] info", "missing its argument"),
        (r"\begin{tikzpicture}\datavisualization [xy Cartesian] data group {g}", "';' missing"),
    ] {
        let (_, w) = tikra::engine::run_document(doc).unwrap();
        assert!(w.iter().any(|w| w.contains(message)), "{doc}: {w:?}");
    }
}

/// `+=` to a group not made yet starts it.
#[test]
fn adding_to_a_new_data_group() {
    let c = chart(r"\datavisualization [xy Cartesian, visualize as line] data group {fresh} += { data point [x=0, y=0] data point [x=1, y=1] } data group {fresh};");
    assert!(same_line(&data_line(&c), &[(0.0, 0.0), (1.0, 1.0)]));
}

/// An attribute set to `\relax` is not set (as `\pgfkeysgetvalue` gives `\relax` for a key without
/// a value): the point is not moved along y.
#[test]
fn an_attribute_set_to_relax_is_unset() {
    let c = chart(r"\datavisualization [xy Cartesian, visualize as line] data point [x=0, y=1] data point [x=1, y=\relax] data point [x=2, y=3];");
    assert!(same_line(&data_line(&c), &[(0.0, 0.0), (1.0, -1.0), (2.0, 2.0)]), "{:?}", data_line(&c));
}

/// `\pgfkeysaddvalue{key}{before}{after}` puts text around a key's value.
#[test]
fn adding_to_a_keys_value() {
    let r = tikra::convert(r"\begin{tikzpicture} \pgfkeyssetvalue{/my/key}{b} \pgfkeysaddvalue{/my/key}{a}{c} \node {\pgfkeysvalueof{/my/key}}; \end{tikzpicture}").unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    assert_eq!(r.description, "Text in the figure: abc.");
}

/// The `\datavisualization` command is not called from inside itself without end.
#[test]
fn data_visualizations_nest_eight_deep_at_most() {
    warns(r"\def\nest{\datavisualization [xy Cartesian, visualize as line] info {\nest} data point [x=0, y=0];} \nest", "nested too deeply");
}

/// A scope's options apply to the data points in it, not to those after it.
#[test]
fn a_scope_sets_its_points_attributes() {
    let c = chart(r"\datavisualization [xy Cartesian, visualize as line] scope [/data point/y=1] { data point [x=0] data point [x=2] } data point [x=3, y=0];");
    assert!(same_line(&data_line(&c), &[(0.0, 0.0), (2.0, 0.0), (3.0, -1.0)]), "{:?}", data_line(&c));
}

/// `data group {name} = {…}` keeps data, `+=` adds to it; `data group {name}` and `data [use
/// group=name]` use it where they are.
#[test]
fn data_groups_are_kept_and_used() {
    let c = chart(
        r"\datavisualization [xy Cartesian, visualize as line]
          data group {peak} = { data point [x=0, y=0] data point [x=1, y=2] }
          data group {peak} += { data point [x=2, y=0] }
          data group {peak}
          data [use group=peak];",
    );
    // the group is used twice: there and back again
    let want = [(0.0, 0.0), (1.0, 2.0), (2.0, 0.0), (0.0, 0.0), (1.0, 2.0), (2.0, 0.0)];
    assert!(same_line(&data_line(&c), &want), "{:?}", data_line(&c));
}

/// `info {…}` is drawn after the data, `info' {…}` before it; `visualization cs` places a point as
/// a data point would be.
#[test]
fn info_is_drawn_after_or_before_the_data() {
    let c = chart(
        r"\datavisualization [xy Cartesian, visualize as line] data point [x=0, y=0] data point [x=2, y=1]
          info { \draw (visualization cs:x=2, y=0) -- (visualization cs:x=2, y=1); }
          info' { \draw[red] (visualization cs:x=0, y=1) -- (visualization cs:x=2, y=1); };",
    );
    let all = paths(&c);
    let [before, line, after] = &all[..] else { panic!("{all:?}") };
    assert!(before.stroke.as_ref().unwrap().color != line.stroke.as_ref().unwrap().color);
    let (b, l, a) = (points(&before.segments), points(&line.segments), points(&after.segments));
    // the red line along y = 1, the black one down at x = 2
    assert!(near((b[1].0 - b[0].0, b[0].1 - l[0].1), (2.0, 1.0)), "{b:?} {l:?}");
    assert!(near((a[0].0 - l[0].0, a[1].1 - a[0].1), (2.0, 1.0)), "{a:?} {l:?}");
}

/// `visualization cs` needs a data visualization to place its point in.
#[test]
fn visualization_cs_outside_a_data_visualization() {
    warns(r"\draw (visualization cs:x=1, y=1) -- (1,1);", "visualization cs outside a data visualization");
}

/// `name=` makes a coordinate at the data point.
#[test]
fn a_named_data_point_is_a_coordinate() {
    let c = chart(
        r"\datavisualization [xy Cartesian, visualize as line] data point [x=0, y=0, name=start] data point [x=3, y=1, name=end] info { \draw[red] (start) -- (end |- start); };",
    );
    let all = paths(&c);
    let red = points(&all[1].segments);
    assert!(near((red[1].0 - red[0].0, red[1].1 - red[0].1), (3.0, 0.0)), "{red:?}");
}

/// What was drawn before the data visualization stays in the picture's box; with nothing drawn, its
/// `data bounding box` is a point at the origin.
#[test]
fn bounding_boxes() {
    let c = chart(&format!(r"\draw (5,5) -- (6,6); \datavisualization [xy Cartesian, visualize as line] {TWO_POINTS};"));
    assert!(c.max.unwrap().x > 6.0 && c.max.unwrap().y > 6.0, "{:?}", c.max);
    let c = chart(r"\datavisualization [xy Cartesian]; \draw (data bounding box.north east) -- (1,1);");
    let line = points(&paths(&c)[0].segments);
    assert!(near((line[1].0 - line[0].0, line[1].1 - line[0].1), (1.0, 1.0)), "{line:?}");
}

// ---------------------------------------------------------------- formats

/// `separator=\space` splits a table's cells at spaces, and `headline=` names its columns.
#[test]
fn tables_with_spaces_or_a_given_head_line() {
    let c = chart("\\datavisualization [xy Cartesian, visualize as line] data [separator=\\space] {\n x y\n 0 0\n\n 1 2\n};");
    assert!(same_line(&data_line(&c), &[(0.0, 0.0), (1.0, 2.0)]));
    let c = chart("\\datavisualization [xy Cartesian, visualize as line] data [headline={x, y}] {\n 0, 0\n 1, 2\n};");
    assert!(same_line(&data_line(&c), &[(0.0, 0.0), (1.0, 2.0)]));
}

/// `format=named`: `key=value` pairs, a list of values giving a data point for each.
#[test]
fn named_values_give_a_point_for_each_combination() {
    let c = chart("\\datavisualization [xy Cartesian, visualize as scatter] data [format=named] {\n x={0,1,2}, y={1,2}\n\n x=5, y=5\n};");
    // the marks (x: two strokes), by their first stroke's middle
    let mut centers: Vec<(f64, f64)> = paths(&c)
        .iter()
        .map(|p| {
            let q = points(&p.segments);
            ((q[0].0 + q[1].0) / 2.0, (q[0].1 + q[1].1) / 2.0)
        })
        .collect();
    let first = centers.iter().fold((f64::MAX, f64::MAX), |m, p| (m.0.min(p.0), m.1.min(p.1)));
    for p in &mut centers {
        *p = (p.0 - first.0, p.1 - first.1);
    }
    for want in [(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (0.0, 1.0), (1.0, 1.0), (2.0, 1.0), (5.0, 4.0)] {
        assert!(centers.iter().any(|p| near(*p, want)), "{want:?} in {centers:?}");
    }
    assert_eq!(centers.len(), 7);
}

/// `format=function`: a variable over a list or an interval with a step; an interval that runs
/// backwards gives no points.
#[test]
fn function_variables_over_lists_and_steps() {
    let c = chart(r"\datavisualization [xy Cartesian, visualize as line] data [format=function] { var x : {0,2,3}; func y = 2*\value x; };");
    assert!(same_line(&data_line(&c), &[(0.0, 0.0), (2.0, 4.0), (3.0, 6.0)]));
    let c = chart(r"\datavisualization [xy Cartesian, visualize as line] data [format=function] { var x : interval [0:1] step 0.25; func y = \value x; };");
    assert!(same_line(&data_line(&c), &[(0.0, 0.0), (0.25, 0.25), (0.5, 0.5), (0.75, 0.75), (1.0, 1.0)]));
    let c = chart(r"\datavisualization [xy Cartesian, visualize as line] data [format=function] { var x : interval [1:0] samples 3; func y = \value x; };");
    assert!(paths(&c).is_empty());
}

/// Statements of the function format that are not understood, and functions that fail, are errors.
#[test]
fn bad_functions_are_reported() {
    for (statements, message) in [
        (r"var x : interval [0:1] samples 3; func y = nosuch(\value x);", "unknown function: nosuch"),
        ("var x : 3;", "'var x : 3' not understood"),
        ("x = 3;", "'x = 3' not understood"),
    ] {
        warns(&format!("\\datavisualization [xy Cartesian, visualize as line] data [format=function] {{ {statements} }};"), message);
    }
}

// ---------------------------------------------------------------- scalings

/// `unit length=1cm per 2 units`: two units of the attribute are a centimeter.
#[test]
fn unit_lengths() {
    let c =
        chart(r"\datavisualization [xy Cartesian, visualize as line, x axis={unit length=1cm per 2 units}, y axis={unit length=2cm}] data point [x=0, y=0] data point [x=4, y=1];");
    assert!(same_line(&data_line(&c), &[(0.0, 0.0), (2.0, 2.0)]));
}

/// `unit vector=` turns an axis: here x goes up and y goes right.
#[test]
fn unit_vectors_turn_the_axes() {
    let c = chart(
        r"\datavisualization [xy Cartesian, visualize as line, x axis={unit vector={(0pt,1pt)}}, y axis={unit vector={(1pt,0pt)}}] data point [x=0, y=0] data point [x=4, y=1];",
    );
    assert!(same_line(&data_line(&c), &[(0.0, 0.0), (1.0, 4.0)]));
}

/// `new polar axes` with `unit vectors=` for 0 and 90 degrees: here 0 degrees is up.
#[test]
fn polar_unit_vectors() {
    let c = chart(
        r"\datavisualization [new polar axes, angle axis={unit vectors={(0pt,1pt)}{(1pt,0pt)}, degrees}, radius axis={unit length=1cm}, visualize as line] data point [angle=0, radius=1] data point [angle=90, radius=1];",
    );
    // from (0, 1) to (1, 0), as an arc or a line
    let line = data_line(&c);
    assert!(near(*line.last().unwrap(), (1.0, -1.0)), "{line:?}");
}

/// A deforming function: `\pgfdvmathunaryop{…}{sqrt}{…}` puts the values at their square roots.
#[test]
fn a_square_root_axis() {
    let c = chart(
        r"\datavisualization [scientific axes, visualize as line, x axis={function=\pgfdvmathunaryop{\pgfvalue}{sqrt}{\pgfvalue}}] data point [x=0, y=0] data point [x=4, y=1] data point [x=9, y=2];",
    );
    // 0, 4 and 9 at 0, 2/3 and 1 of the width
    let line = data_line(&c);
    assert!(near(line[1], (5.0 * 2.0 / 3.0, 0.618 * 5.0 / 2.0)), "{line:?}");
    // the ticks, 0 to 8 by 2, at their square roots
    let ticks = bottom_ticks(&c);
    let want: Vec<f64> = [0.0f64, 2.0, 4.0, 6.0, 8.0].iter().map(|v| v.sqrt() / 3.0 * 5.0).collect();
    assert!(ticks.len() == 5 && ticks.iter().zip(&want).all(|(a, b)| (a - b).abs() < 1e-3), "{ticks:?}");
}

// ---------------------------------------------------------------- logarithmic axes

/// A logarithmic axis: its ticks at powers of ten, each decade the same length.
#[test]
fn logarithmic_axes_tick_powers_of_ten() {
    let body = r"\datavisualization [scientific axes, visualize as line, y axis={logarithmic}] data point [x=0, y=1] data point [x=1, y=1000];";
    assert_eq!(words(body)[6..], ["1", "10", "100", "1 comma 000"]);
    let ys = left_ticks(&chart(body));
    let h = 0.618 * 5.0;
    assert!(ys.len() == 4 && ys.iter().enumerate().all(|(i, y)| (y - h * i as f64 / 3.0).abs() < 1e-3), "{ys:?}");
}

/// About n ticks on a logarithmic axis: steps of one, three or six decades (or one decade when the
/// range is less than n decades).
#[test]
fn logarithmic_steps_of_one_three_or_six_decades() {
    for (ticks, top, want) in [
        ("few", "1e5", &["1", "10", "100", "1 comma 000", "10 comma 000", "1 times 10 to the 5"][..]),
        ("many", "2", &["1"][..]),
        ("some", "1e25", &["1", "1 comma 000", "1 times 10 to the 6", "1 times 10 to the 9"][..]),
        ("few", "1e18", &["1", "1 times 10 to the 6", "1 times 10 to the 12", "1 times 10 to the 18"][..]),
        ("few", "1e40", &["1", "1 times 10 to the 10", "1 times 10 to the 20", "1 times 10 to the 30", "1 times 10 to the 40"][..]),
    ] {
        let bottom = if top == "2" { "0.5" } else { "1" };
        let w = words(&format!(
            r"\datavisualization [scientific axes, visualize as line, y axis={{logarithmic, ticks={ticks}}}] data point [x=0, y={bottom}] data point [x=1, y={top}];"
        ));
        assert_eq!(&w[6..6 + want.len()], want, "{ticks} {top}: {w:?}");
    }
}

/// `phase=` moves the ticks of a logarithmic axis off the powers of ten.
#[test]
fn a_logarithmic_phase() {
    let w = words(r"\datavisualization [scientific axes, visualize as line, y axis={logarithmic, ticks={step=1, phase=0.5}}] data point [x=0, y=1] data point [x=1, y=1000];");
    assert_eq!(w[6..], ["3.16", "31.62", "316.23"]);
}

/// Minor ticks on a logarithmic axis: `minor steps between steps=8` puts them at 2, 3, … 9 times
/// the powers of ten.
#[test]
fn logarithmic_minor_ticks() {
    let c =
        chart(r"\datavisualization [scientific axes, visualize as line, y axis={logarithmic, ticks={minor steps between steps=8}}] data point [x=0, y=2] data point [x=1, y=50];");
    let h = 0.618 * 5.0;
    let at = |v: f64| (v / 2.0f64).ln() / 25.0f64.ln() * h;
    let ys = left_ticks(&c);
    for v in [3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 20.0, 30.0, 40.0] {
        assert!(ys.iter().any(|y| (y - at(v)).abs() < 1e-3), "{v} in {ys:?}");
    }
}

// ---------------------------------------------------------------- tick placement

/// Minor ticks between the major ones, on a linear axis.
#[test]
fn linear_minor_ticks() {
    let c = chart(r"\datavisualization [scientific axes, visualize as line, x axis={ticks={step=1, minor steps between steps=3}}] data point [x=0, y=0] data point [x=2, y=1];");
    let xs = bottom_ticks(&c);
    for v in [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0] {
        assert!(xs.iter().any(|x| (x - v * 2.5).abs() < 1e-3), "{v} in {xs:?}");
    }
}

/// An axis whose values are all the same, or a step of 0: one tick, at that value.
#[test]
fn one_value_one_tick() {
    let w = words(r"\datavisualization [scientific axes, visualize as line] data point [x=1, y=0] data point [x=1, y=1];");
    assert_eq!(w, ["1", "0", "0.2", "0.4", "0.6", "0.8", "1"]);
    let w = words(r"\datavisualization [scientific axes, visualize as line, x axis={ticks={step=0}}] data point [x=1, y=0] data point [x=2, y=1];");
    assert_eq!(w, ["1", "0", "0.2", "0.4", "0.6", "0.8", "1"]);
}

/// An axis no data point gives a value to has no ticks.
#[test]
fn an_axis_without_values_has_no_ticks() {
    let w = words(r"\datavisualization [scientific axes, visualize as line] data point [x=0] data point [x=1];");
    assert_eq!(w, ["0", "0.2", "0.4", "0.6", "0.8", "1"]);
}

/// The parts of an about strategy that are not `threshold/digit` are passed over: 7 in about 5
/// ticks is then a step of 1.4.
#[test]
fn about_strategies_skip_what_they_do_not_understand() {
    let w = words(r"\datavisualization [scientific axes, visualize as line, x axis={ticks={about strategy={junk, 2/x, 1.2/1}}}] data point [x=0, y=0] data point [x=7, y=1];");
    assert_eq!(w[..6], ["0", "1.4", "2.8", "4.2", "5.6", "7"]);
}

/// `at={value as [options] text}`: a tick with its own text and options; `options at=` gives a
/// value's tick options.
#[test]
fn ticks_with_their_own_text_and_options() {
    let body = r"\datavisualization [scientific axes, visualize as line, x axis={ticks={major={at={1 as one, {2} as [style=red] two}}, options at=0 as [no tick text]}}] data point [x=0, y=0] data point [x=3, y=1];";
    assert_eq!(words(body)[..2], ["one", "two"]);
    let c = chart(body);
    let red: Vec<Drawn> = paths(&c).into_iter().filter(|p| p.stroke.as_ref().is_some_and(|s| s.color.r > 0.9 && s.color.g < 0.1)).collect();
    // the tick at 2, below and above the box
    assert_eq!(red.len(), 2);
    assert!(red.iter().all(|p| (points(&p.segments)[0].0 - points(&data_paths(&c)[0].segments)[0].0 - 2.0 / 3.0 * 5.0).abs() < 1e-3));
}

/// Ticks without a direction axis stand across a horizontal axis; `visualize ticks` without ticks
/// asked for draws none.
#[test]
fn ticks_without_a_direction_axis() {
    let c = chart(
        r"\datavisualization [xy Cartesian, visualize as line, x axis={ticks={step=1}, visualize ticks={major={tick text at low}}}] data point [x=0, y=0] data point [x=3, y=1];",
    );
    let ticks: Vec<Vec<(f64, f64)>> = paths(&c).iter().map(|p| points(&p.segments)).filter(|p| p.len() == 2 && (p[0].0 - p[1].0).abs() < 1e-9).collect();
    assert_eq!(ticks.len(), 4);
    // from -2pt to 2pt
    assert!(ticks.iter().all(|t| (t[1].1 - t[0].1 - 4.0 * PT).abs() < 1e-6), "{ticks:?}");
    assert_eq!(
        words(
            r"\datavisualization [xy Cartesian, visualize as line, x axis={ticks={step=1}, visualize ticks={major={tick text at low}}}] data point [x=0, y=0] data point [x=3, y=1];"
        ),
        ["0", "1", "2", "3"]
    );
    let c = chart(&format!(r"\datavisualization [xy Cartesian, visualize as line, x axis={{visualize ticks}}] {TWO_POINTS};"));
    assert_eq!(paths(&c).len(), 1);
}

/// `ticks and grid=` sets both: the ticks and a grid line at each.
#[test]
fn ticks_and_grid() {
    let c = chart(
        r"\datavisualization [xy Cartesian, visualize as line, x axis={ticks and grid={step=1}, visualize ticks, visualize grid={y axis={goto=min}}}] data point [x=0, y=0] data point [x=2, y=1];",
    );
    // three ticks, three grid lines and the data
    assert_eq!(paths(&c).len(), 7);
}

// ---------------------------------------------------------------- tick texts

/// A tick typesetter's code runs: its assignments and computations are not printed, what it prints
/// is the tick's text.
#[test]
fn tick_typesetters_compute() {
    let w = words(
        r"\datavisualization [scientific axes, visualize as line, x axis={ticks={tick typesetter/.code={\def\v{#1}\relax\pgfmathparse{2*\v}\pgfmathprintnumber{\pgfmathresult}}, step=1}}] data point [x=0, y=0] data point [x=3, y=1];",
    );
    assert_eq!(w[..4], ["0", "2", "4", "6"]);
    // \pgfmathapproxequalto, and \pgfmathprintnumber with options
    let w = words(
        r"\datavisualization [scientific axes, visualize as line, x axis={ticks={step=1, tick typesetter/.code={\pgfmathapproxequalto{#1}{2}\ifdim\pgfmathresult pt>0pt two\else\pgfmathprintnumber[fixed, precision=1, fixed zerofill]{#1}\fi}}}] data point [x=0, y=0] data point [x=3, y=1];",
    );
    assert_eq!(w[..4], ["0.0", "1.0", "two", "3.0"]);
}

/// A tick typesetter that never stops printing is cut off (TeX would run out of memory): its text
/// is too long for a node.
#[test]
fn a_tick_typesetter_without_end() {
    warns(
        r"\datavisualization [xy Cartesian, visualize as line, x axis={ticks={step=1, tick typesetter/.code={\def\again{x\again}\again}}, visualize ticks={major={tick text at low}}}] data point [x=0, y=0] data point [x=1, y=1];",
        "argument too long",
    );
}

/// Tick texts as fractions (`/pgf/number format/frac`).
#[test]
fn fractions_as_tick_texts() {
    let w = words(
        r"\datavisualization [scientific axes, visualize as line, x axis={ticks={node style={/pgf/number format/frac}, step=0.25}}] data point [x=0, y=0] data point [x=1, y=1];",
    );
    assert_eq!(w[..5], ["0", "1 over 4", "1 over 2", "3 over 4", "1"]);
}

/// The polar axes in radians write multiples of π: π itself, and -π.
#[test]
fn multiples_of_pi() {
    let w = words(r"\datavisualization [scientific polar axes={-pi to pi}, visualize as line] data point [angle=0, radius=1] data point [angle=3, radius=2];");
    assert_eq!(w[..2], ["negative pi", "negative 11 over 12 pi"]);
    assert!(w.contains(&"0".to_string()) && w.contains(&"1 over 2 pi".to_string()), "{w:?}");
    let w = words(r"\datavisualization [scientific polar axes={0 to pi}, visualize as line] data point [angle=0, radius=1] data point [angle=3, radius=2];");
    assert_eq!(w[..13].last().unwrap(), "pi");
}

/// `\pgfmathprintnumber` in a picture but outside a node prints text where no text goes, as TeX
/// would.
#[test]
fn a_number_printed_outside_a_node() {
    warns(&format!(r"\datavisualization [xy Cartesian, visualize as line] {TWO_POINTS} info {{ \pgfmathprintnumber{{3.5}} }};"), "text outside a statement ignored");
}

// ---------------------------------------------------------------- axis labels

/// `label` alone is the attribute's name; `label={[options]text}` styles the label's node.
#[test]
fn axis_labels() {
    let body =
        r"\datavisualization [scientific axes, visualize as line, x axis={label={[node style={text=red}]time}}, y axis={label}] data point [x=0, y=0] data point [x=7, y=1];";
    let w = words(body);
    assert_eq!(w[w.len() - 2..], ["time", "y"]);
    let c = chart(body);
    let red = common::texts(&c).iter().filter(|(l, _)| l.color.r > 0.9 && l.color.g < 0.1).count();
    assert_eq!(red, 1);
}
