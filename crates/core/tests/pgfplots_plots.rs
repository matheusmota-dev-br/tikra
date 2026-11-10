//! pgfplots' `\addplot`: its syntax and inputs (coordinates, expressions, files, patch tables),
//! the cycle lists, coordinate filters and styles, point metas, data coordinate systems and the
//! sampling of domains.

mod common;
use tikra::canvas::{Canvas, Fill, Item, Paint};
use tikra::scene::Segment;

/// A picture with an axis (`axis`, or the environment before `:::`) of these options around a body.
fn picture(options: &str, body: &str) -> String {
    let (env, options) = options.split_once(":::").unwrap_or(("axis", options));
    format!("\\begin{{tikzpicture}}\\begin{{{env}}}[{options}]\n{body}\n\\end{{{env}}}\\end{{tikzpicture}}")
}

/// The figure's description and warnings.
fn describe(options: &str, body: &str) -> (String, Vec<String>) {
    let r = common::figure(&picture(options, body));
    (r.description, r.warnings)
}

/// The description of a figure that must have no warnings.
fn described(options: &str, body: &str) -> String {
    let (d, w) = describe(options, body);
    assert!(w.is_empty(), "warnings: {w:?}\n{body}");
    d
}

/// The warnings of a figure.
fn warnings(options: &str, body: &str) -> Vec<String> {
    describe(options, body).1
}

fn has(w: &[String], fragment: &str) -> bool {
    w.iter().any(|w| w.contains(fragment))
}

/// The canvas of an axis.
fn drawn(options: &str, body: &str) -> Canvas {
    let (c, w) = tikra::engine::run_document(&picture(options, body)).unwrap();
    assert!(w.is_empty(), "warnings: {w:?}\n{body}");
    c
}

/// The colors of the filled paths (marks, patches), as `#rrggbb`.
fn fills(c: &Canvas) -> Vec<String> {
    c.items
        .iter()
        .filter_map(|i| match i {
            Item::Path { fill: Some(Fill { paint: Paint::Solid(col), .. }), .. } => Some(col.hex()),
            _ => None,
        })
        .collect()
}

/// The stroked paths: their color and whether they are dashed.
fn strokes(c: &Canvas) -> Vec<(String, bool)> {
    c.items
        .iter()
        .filter_map(|i| match i {
            Item::Path { stroke: Some(s), .. } => Some((s.color.hex(), s.dash.is_some())),
            _ => None,
        })
        .collect()
}

fn texts(c: &Canvas) -> usize {
    c.items.iter().filter(|i| matches!(i, Item::Text { .. })).count()
}

// ---------------------------------------------------------------------------------------------
// the syntax of \addplot

#[test]
fn plot_with_behavior_options() {
    // `\addplot plot[options]`: the old syntax, the options after the user's
    let d = described("", r"\addplot plot[only marks] coordinates {(0,0) (1,1)};");
    assert!(d.starts_with("Scatter plot.") && d.contains("(0, 0), (1, 1)"), "{d}");
    // `\addplot expression[options] {…}`
    let d = described("", r"\addplot expression[domain=0:2, samples=3] {x^2};");
    assert!(d.contains("3 points — (0, 0), (1, 1), (2, 4)"), "{d}");
}

#[test]
fn inputs_that_are_not_supported() {
    // an unknown input, a word too long to be one, a number: the plot is left out
    for body in [r"\addplot pizza;", r"\addplot coordinatesabcdefghijabcdefghijabcdefghijabcdefghij {(0,0)};", r"\addplot 5;"] {
        let w = warnings("", body);
        assert!(w.iter().any(|w| w.starts_with("\\addplot: figure: \\addplot input '") && w.ends_with("not supported; plot ignored")), "{body}: {w:?}");
    }
    let w = warnings("", r"\addplot expression x;");
    assert!(has(&w, "expression without a formula; plot ignored"), "{w:?}");
    let w = warnings("", r"\addplot fill foo;");
    assert!(has(&w, "\\addplot fill foo not supported"), "{w:?}");
    // external programs: the plot has no points, the next one is drawn
    for (input, name) in [(r"gnuplot {x^2}", "gnuplot"), (r"shell[prefix=a] {ls}", "shell"), (r"function {x}", "function")] {
        let (d, w) = describe("", &format!("\\addplot {input}; \\addplot coordinates {{(0,0)}};"));
        assert_eq!(w, vec![format!("\\addplot {name} not supported: it needs an external program or file")]);
        assert!(d.contains("Series 2: one point — (0, 0)"), "{d}");
    }
}

#[test]
fn the_trailing_path() {
    // a group after the input (a node's text) is part of the trailing path
    let c = drawn("hide axis", r"\addplot coordinates {(0,0) (1,1)} node[pos=1] {end};");
    assert_eq!(texts(&c), 1);
    // a plot that ends with the group it is in, without `;`
    let d = described("", r"{\addplot coordinates {(0,0) (1,1)}}");
    assert!(d.contains("2 points — (0, 0), (1, 1)"), "{d}");
}

#[test]
fn an_addplot_cut_short() {
    // the document ends after `\addplot`, `\addplot3` or `\addplot plot`: nothing to draw, no failure
    for end in [r"\addplot", r"\addplot3", r"\addplot plot", r"\addplot+"] {
        let r = tikra::convert(&format!(r"\begin{{tikzpicture}}\begin{{axis}}\addplot coordinates {{(0,0) (1,1)}}; {end}"));
        let r = r.unwrap_or_else(|e| panic!("{end}: {e}"));
        assert!(r.warnings.iter().any(|w| w.contains("plot ignored")), "{end}: {:?}", r.warnings);
    }
    // \addplot outside an axis: ignored, up to its `;`
    let r = common::figure(r"\begin{tikzpicture}\addplot coordinates {(0,0)}; \draw (0,0) -- (1,0);\end{tikzpicture}");
    assert_eq!(r.warnings, vec!["\\addplot outside an axis: 'coordinates {(0,0)}' ignored".to_string()]);
    // \addlegendimage too
    let r = common::figure(r"\begin{tikzpicture}\addlegendimage{red} \draw (0,0) -- (1,0);\end{tikzpicture}");
    assert!(r.warnings[0].starts_with("\\addlegendimage outside an axis"), "{:?}", r.warnings);
}

#[test]
fn execute_at_begin_and_end_of_a_plot() {
    // the code at the plot's end adds its legend entry
    let d = described("", r"\addplot[execute at begin plot={\def\tikrafoo{1}}, execute at end plot={\addlegendentry{seen}}] coordinates {(0,0) (1,1)};");
    assert!(d.contains("“seen”: 2 points"), "{d}");
}

// ---------------------------------------------------------------------------------------------
// cycle lists

/// The plot lines of an axis without its own lines: color and dashing, one per plot.
fn plot_lines(options: &str, plots: usize) -> Vec<(String, bool)> {
    let body: String = (0..plots).map(|k| format!("\\addplot coordinates {{(0,{k}) (1,{k})}};")).collect();
    strokes(&drawn(&format!("hide axis, no markers, {options}"), &body))
}

#[test]
fn cycle_lists_that_are_not_there_or_empty() {
    let w = warnings("cycle list name=nope", r"\addplot coordinates {(0,0) (1,1)};");
    assert_eq!(w, vec!["cycle list 'nope' not defined".to_string()]);
    // an empty list gives nothing: the plot is black
    let lines = plot_lines("cycle list={}", 1);
    assert_eq!(lines, vec![("#000000".to_string(), false)]);
}

#[test]
fn cycle_multi_lists() {
    let red = "#ff0000".to_string();
    let blue = "#0000ff".to_string();
    let green = "#00ff00".to_string();
    // one entry of each list, the last one fastest
    let lines = plot_lines(r"cycle multi list={red,blue\nextlist solid,dashed}", 4);
    assert_eq!(lines, vec![(red.clone(), false), (red.clone(), true), (blue.clone(), false), (blue.clone(), true)]);
    // the same index into each list, the shorter one padded with nothing...
    let lines = plot_lines(r"cycle multiindex list={red,blue,green\nextlist solid,dashed}", 3);
    assert_eq!(lines, vec![(red.clone(), false), (blue.clone(), true), (green.clone(), false)]);
    // ... or restarted
    let lines = plot_lines(r"cycle multiindex* list={red,blue,green\nextlist dashed,solid}", 3);
    assert_eq!(lines, vec![(red.clone(), true), (blue.clone(), false), (green.clone(), true)]);
    // a list by name, or its first entries (`[2 of]`): the color list is blue, red, …
    let lines = plot_lines(r"cycle multi list={[2 of]color\nextlist solid}", 3);
    assert_eq!(lines.iter().map(|l| l.0.as_str()).collect::<Vec<_>>(), vec!["#0000ff", "#ff0000", "#0000ff"]);
    let lines = plot_lines(r"cycle multi list={color\nextlist solid}", 3);
    // (its third entry is brown!60!black)
    assert_eq!(lines[2].0, "#734d26", "{lines:?}");
}

#[test]
fn scatter_classes_have_a_legend_entry_each() {
    let c = drawn(
        "hide axis",
        r"\addplot[scatter, only marks, point meta=explicit symbolic, scatter/classes={a={mark=square*,blue},b={mark=triangle*,red}}] coordinates {(0,0)[a] (1,1)[b]}; \legend{A,B}",
    );
    assert_eq!(texts(&c), 2);
    // the marks: blue for a, red for b (and their legend images)
    let f = fills(&c);
    assert!(f.iter().filter(|c| *c == "#0000ff").count() >= 2 && f.iter().filter(|c| *c == "#ff0000").count() >= 2, "{f:?}");
}

// ---------------------------------------------------------------------------------------------
// inputs

#[test]
fn coordinates_syntax() {
    // expressions in parentheses, z as an expression
    let d = described("", r"\addplot3 coordinates {(0,0,1+1) ((1+1),2,3)};");
    assert!(d.contains("(0, 0, 2), (2, 2, 3)"), "{d}");
    // `()` is a jump: the line is broken there
    let c = drawn("hide axis, unbounded coords=jump", r"\addplot coordinates {(0,0) () (1,1) (2,0)};");
    let moves = c.items.iter().map(|i| if let Item::Path { segments, .. } = i { segments.iter().filter(|s| matches!(s, Segment::MoveTo(_))).count() } else { 0 }).max();
    assert_eq!(moves, Some(2));
    // `+= (…)` and `-= (…)`: each side of the error bar on its own; `+-` without parentheses is not one
    let d = described("", r"\addplot+[error bars/.cd, y dir=both, y explicit] coordinates {(0,0) += (0,1) -= (0,2) (1,1) +- 3 [4]};");
    assert!(d.contains("Vertical axis, from -2 to 1."), "{d}");
}

#[test]
fn contours_prepared() {
    // in 3d, the level is z
    let d = described("", r"\addplot3[contour prepared] coordinates {(0,0,1) (1,0,1) (1,1,1)};");
    assert!(d.starts_with("Contour plot.") && d.contains("Colors stand for values from 0.5 to 1.5"), "{d}");
    // matlab's format: (level, count) before each line
    let d = described("", r"\addplot[contour prepared, contour prepared format=matlab] coordinates {(2,3) (0,0) (1,0) (1,1) (5,2) (0,1) (1,2)};");
    assert!(d.contains("Colors stand for values from 2 to 5. 5 points — (0, 0), (1, 0), (1, 1), (0, 1), (1, 2)"), "{d}");
}

#[test]
fn smith_chart_coordinates() {
    // `is smithchart cs`: the points are reflection coefficients; 0.5 is the impedance 3
    let d = described("smithchart:::", r"\addplot[is smithchart cs] coordinates {(0.5,0)};");
    assert!(d.contains("one point — (3, 0)"), "{d}");
}

fn input_dir() {
    static DIR: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("tikra-pgfplots-plots-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("faces.dat"), "0 1 2\n").unwrap();
        std::env::set_var("TIKRA_INPUT_DIR", format!("{}:.", dir.display()));
    });
}

#[test]
fn patch_tables() {
    input_dir();
    let points = r"coordinates {(0,0) (1,0) (0,1) (1,1)};";
    // the patches by the points' indices: the second triangle shares two points with the first
    let d = described("", &format!(r"\addplot[patch, patch table={{0 1 2\\1 2 3}}] {points}"));
    assert!(d.contains("6 points — (0, 0), (1, 0), (0, 1), (1, 0), (0, 1), (1, 1)"), "{d}");
    // a color for each patch, or for each of its corners
    let d = described("", &format!(r"\addplot[patch, patch table with point meta={{0 1 2 5\\1 2 3 7}}] {points}"));
    assert!(d.contains("Colors stand for values from 5 to 7"), "{d}");
    let d = described("", &format!(r"\addplot[patch, patch table with individual point meta={{0 1 2 5 6 7\\1 2 3 7 8 9}}] {points}"));
    assert!(d.contains("Colors stand for values from 5 to 9"), "{d}");
    // a point that is not there is left out
    let d = described("", &format!(r"\addplot[patch, patch table={{0 1 9}}] {points}"));
    assert!(d.contains("2 points — (0, 0), (1, 0)"), "{d}");
    // in a file
    let d = described("", &format!(r"\addplot[patch, patch table=faces.dat] {points}"));
    assert!(d.contains("3 points — (0, 0), (1, 0), (0, 1)"), "{d}");
    let (d, w) = describe("", &format!(r"\addplot[patch, patch table=missing.dat] {points}"));
    assert_eq!(w, vec!["patch table: file 'missing.dat' not found".to_string()]);
    assert!(d.contains("4 points"), "{d}");
}

// ---------------------------------------------------------------------------------------------
// sampling

#[test]
fn degenerate_domains() {
    // one value: three samples of it
    let d = described("", r"\addplot[domain=1:1, samples=3] {x};");
    assert!(d.contains("3 points — (1, 1), (1, 1), (1, 1)"), "{d}");
    let d = described("xmode=log", r"\addplot[domain=1:1, samples=3] {x};");
    assert!(d.contains("3 points — (1, 1), (1, 1), (1, 1)"), "{d}");
    // a step pgf's fpu cannot write (it keeps eight digits): equal steps all the same
    let d = described("", r"\addplot[domain=100000000:100000000.001, samples=3] {x};");
    assert!(d.contains("(100000000, 100000000), (100000000.0005, 100000000.0005), (100000000.001, 100000000.001)"), "{d}");
}

#[test]
fn patch_type_sampling() {
    // each interval gives the points of one patch, where its type has them in the unit interval
    let d = described("", r"\addplot[patch, patch type=line, patch type sampling, domain=0:1, samples=3] {x^2};");
    assert!(d.contains("4 points — (0, 0), (0.5, 0.25), (0.5, 0.25), (1, 1)"), "{d}");
    let d = described("", r"\addplot[patch, patch type=quadratic spline, patch type sampling, domain=0:2, samples=2] {x};");
    assert!(d.contains("3 points — (0, 0), (2, 2), (1, 1)"), "{d}");
    let d = described("", r"\addplot[patch, patch type=cubic spline, patch type sampling, domain=0:3, samples=2] {x};");
    assert!(d.contains("4 points — (0, 0), (3, 3), (1, 1), (2, 2)"), "{d}");
    // and each cell of the square, in 3d
    let d = described("", r"\addplot3[surf, patch type=bilinear, patch type sampling, domain=0:1, samples=2, y domain=0:1, samples y=2] {x*y};");
    assert!(d.contains("4 points — (0, 0, 0), (1, 0, 0), (1, 1, 1), (0, 1, 0)"), "{d}");
    for (kind, n) in [("triangle quadr", 12), ("biquadratic", 9), ("bicubic", 16)] {
        let d = described("", &format!(r"\addplot3[surf, patch type={kind}, patch type sampling, domain=0:1, samples=2, y domain=0:1, samples y=2] (x,y,{{x+y}});"));
        assert!(d.contains(&format!("{n} points")), "{kind}: {d}");
    }
    // a type without points in the unit square samples as usual
    let d = described("", r"\addplot[patch, patch type=polygon, vertex count=3, patch type sampling, domain=0:2, samples=3] {x};");
    assert!(d.contains("3 points — (0, 0), (1, 1), (2, 2)"), "{d}");
}

// ---------------------------------------------------------------------------------------------
// coordinate filters and styles

#[test]
fn coordinate_filters() {
    // code: `\pgfmathresult` in and out
    let d = described("", r"\addplot[x filter/.code={\pgfmathparse{#1*10}}] coordinates {(1,1) (2,2)};");
    assert!(d.contains("(10, 1), (20, 2)"), "{d}");
    // an empty result drops the point
    let d = described("", r"\addplot[y filter/.code={\ifnum\coordindex=1 \def\pgfmathresult{}\fi}] coordinates {(1,1) (2,2) (3,3)};");
    assert!(d.contains("2 points — (1, 1), (3, 3)"), "{d}");
    // expressions: of the coordinate, its index, the value so far
    let d = described("", r"\addplot[x filter/.expression={x+\coordindex}] coordinates {(1,1) (2,2)};");
    assert!(d.contains("(1, 1), (3, 2)"), "{d}");
    let d = described("", r"\addplot[y filter/.expression={\pgfmathresult*2}] coordinates {(1,1) (2,2)};");
    assert!(d.contains("(1, 2), (2, 4)"), "{d}");
    // `restrict expr to domain`
    let d = described("", r"\addplot[restrict expr to domain={x}{0:1.5}] coordinates {(1,1) (2,2)};");
    assert!(d.contains("one point — (1, 1)"), "{d}");
}

#[test]
fn coordinate_styles() {
    // the marks of a scatter plot, drawn one by one; the first one in the colormap's first color
    let scatter = "scatter, only marks, mark=*, coordinate style";
    // `.condition`: the coordinates where it holds
    let c = drawn("hide axis", &format!(r"\addplot[{scatter}/.condition={{x>1}}{{red}}] coordinates {{(1,1) (2,2)}};"));
    assert_eq!(fills(&c), vec!["#0000ff", "#ff0000"]);
    // two conditions that both hold: both styles, the later one last
    let c = drawn("hide axis", &format!(r"\addplot[{scatter}/.condition={{x>1}}{{red}}, {scatter}/.condition={{y>1}}{{green}}] coordinates {{(1,1) (2,2)}};"));
    assert_eq!(fills(&c), vec!["#0000ff", "#00ff00"]);
    // `.from`: a table's cell, or a style for every point
    let c = drawn("hide axis", &format!(r"\addplot[{scatter}/.from=\thisrow{{s}}] table[row sep=\\] {{x y s\\ 1 1 red\\ 2 2 {{blue}}\\}};"));
    assert_eq!(fills(&c), vec!["#ff0000", "#0000ff"]);
    let c = drawn("hide axis", &format!(r"\addplot[{scatter}/.from={{green}}] coordinates {{(1,1) (2,2)}};"));
    assert_eq!(fills(&c), vec!["#00ff00", "#00ff00"]);
}

#[test]
fn visualization_depends_on_a_value() {
    // `value …`: the text as written, here the point's number in its node
    let c = drawn("hide axis", r"\addplot[only marks, visualization depends on={value \coordindex \as \idx}, nodes near coords=\idx] coordinates {(1,1) (2,2)};");
    assert_eq!(texts(&c), 2);
}

#[test]
fn scatter_classes_style_their_points() {
    let c = drawn("hide axis", r"\addplot[only marks, scatter, point meta=explicit symbolic, scatter/classes={a={mark=*,red}, b={mark=*,blue}}] coordinates {(1,1)[a] (2,2)[b]};");
    assert_eq!(fills(&c), vec!["#ff0000", "#0000ff"]);
}

// ---------------------------------------------------------------------------------------------
// tables of special plots

#[test]
fn quiver_from_a_table() {
    // `quiver/u` and `quiver/v` read from each row
    let d = described("", r"\addplot[quiver={u=\thisrow{u}, v=\thisrow{v}}, -stealth] table[row sep=\\] {x y u v\\ 0 0 1 0\\ 1 1 0 1\\};");
    assert!(d.starts_with("Vector field.") && d.contains("Vertical axis, from 0 to 2."), "{d}");
}

#[test]
fn tie_lines_of_a_ternary_diagram() {
    // the tie's other end: the columns three after x, y and z, or the ones named
    let tables = [
        (r"table[row sep=\\] {a b c d e f\\ 0.2 0.3 0.5 0.6 0.3 0.1\\}", ""),
        (r"table[row sep=\\, tie end x=d, tie end y=e, tie end z=f] {a b c d e f\\ 0.2 0.3 0.5 0.6 0.3 0.1\\}", ""),
        (r"table[row sep=\\, tie end x index=3, tie end y index=4, tie end z index=5] {a b c d e f\\ 0.2 0.3 0.5 0.6 0.3 0.1\\}", ""),
    ];
    for (table, _) in tables {
        let d = described("ternaryaxis:::", &format!(r"\addplot3[tieline] {table};"));
        assert!(d.contains("2 points"), "{table}: {d}");
    }
    // no columns for them
    let w = warnings("ternaryaxis:::", r"\addplot3[tieline] table[row sep=\\] {a b c\\ 0.2 0.3 0.5\\};");
    assert!(has(&w, "tieline: the tie ends' columns are not found"), "{w:?}");
}

#[test]
fn box_plot_data_filters() {
    // `boxplot/data filter/.code`: each value through the code (here doubled): the box of the
    // doubled values
    let doubled = described("", r"\addplot+[boxplot] table[row sep=\\, y index=0] {2\\4\\6\\8\\10\\};");
    for (data, index) in [("", "y index=0"), ("data=x, ", "x index=0")] {
        let d = described("", &format!(r"\addplot+[boxplot={{{data}data filter/.code={{\pgfmathparse{{#1*2}}}}}}] table[row sep=\\, {index}] {{1\\2\\3\\4\\5\\}};"));
        assert_eq!(d, doubled, "{data}");
    }
    // `data=z` in 3d, one more
    let plus_one = described("", r"\addplot+[boxplot] table[row sep=\\, y index=0] {2\\3\\4\\5\\6\\};");
    let d = described("", r"\addplot3+[boxplot={data=z, data filter/.code={\pgfmathparse{#1+1}}}] table[row sep=\\, z index=0, x index=0, y index=0] {1\\2\\3\\4\\5\\};");
    assert_eq!(d, plus_one);
}

// ---------------------------------------------------------------------------------------------
// point meta

#[test]
fn symbolic_point_metas() {
    // the patches' colors (`flat corner`: each triangle in its first corner's)
    let patch = "patch, shader=flat corner, mesh/color input";
    // a color for each point, as written...
    let c = drawn("hide axis", &format!(r"\addplot[{patch}=explicit, point meta={{symbolic={{0.5,0.2,0.1}}}}] coordinates {{(0,0) (1,0) (0,1)}};"));
    assert_eq!(fills(&c), vec!["#80331a"]);
    // ... or computed from the point
    let c = drawn("hide axis", &format!(r"\addplot[{patch}=explicit mathparse, point meta={{symbolic={{x,y,0}}}}] coordinates {{(1,1) (1,0) (0,1)}};"));
    assert_eq!(fills(&c), vec!["#ffff00"]);
    // a table's cells, and a color model
    let table = r"table[row sep=\\] {x y r g b\\ 0 1 0 0 1\\ 1 0 1 0 0\\ 1 1 1 0 0\\ 0 1 0 1 0\\ 1 0 1 0 0\\ 0 0 1 0 0\\}";
    let c = drawn("hide axis", &format!(r"\addplot[{patch}=explicit mathparse, point meta={{symbolic={{\thisrow{{r}},\thisrow{{g}},\thisrow{{b}}}}}}] {table};"));
    assert_eq!(fills(&c), vec!["#0000ff", "#00ff00"]);
    let c = drawn("hide axis", &format!(r"\addplot[{patch}=explicit mathparse, point meta={{symbolic={{rgb=x,0,y}}}}] {table};"));
    assert_eq!(fills(&c), vec!["#0000ff", "#0000ff"]);
}

#[test]
fn point_meta_of_the_index() {
    let d = described("", r"\addplot[scatter, point meta=\coordindex*10] coordinates {(0,0) (1,1) (2,2)};");
    assert!(d.contains("Colors stand for values from 0 to 20"), "{d}");
}

// ---------------------------------------------------------------------------------------------
// coordinate systems and filters of the axis

#[test]
fn data_coordinate_systems() {
    // polar data in a cartesian axis: (angle, radius)
    for (cs, quarter) in [("polar", "90"), ("polarrad", "1.5707963267949")] {
        let d = described("", &format!(r"\addplot[data cs={cs}] coordinates {{(0,1) ({quarter},2)}};"));
        assert!(d.contains("2 points — (1, 0), (0, 2)"), "{cs}: {d}");
    }
    // cartesian data in a polar axis
    let d = described("polaraxis:::", r"\addplot[data cs=cart] coordinates {(1,0) (0,2)};");
    assert!(d.contains("2 points — (0, 1), (90, 2)"), "{d}");
}

#[test]
fn log_axes_leave_out_what_has_no_logarithm() {
    let d = described("ymode=log", r"\addplot coordinates {(0,1) (1,0) (2,10)};");
    assert!(d.contains("2 points — (0, 1), (2, 10)"), "{d}");
}

#[test]
fn mesh_rows() {
    // `mesh/rows`: the lattice's rows, its columns the rest
    let c = drawn("hide axis", r"\addplot3[mesh, mesh/rows=2] coordinates {(0,0,0) (1,0,0) (2,0,0) (0,1,1) (1,1,1) (2,1,1)};");
    // three columns of two rows: two cells, each drawn round
    let cells = |c: &Canvas| c.items.iter().filter(|i| matches!(i, Item::Path { stroke: Some(_), segments, .. } if segments.contains(&Segment::Close))).count();
    assert_eq!(cells(&c), 2);
    // (without, a line of six points)
    let c = drawn("hide axis", r"\addplot3[mesh] coordinates {(0,0,0) (1,0,0) (2,0,0) (0,1,1) (1,1,1) (2,1,1)};");
    assert_eq!(cells(&c), 0);
}

// ---------------------------------------------------------------------------------------------
// where tables come from

#[test]
fn table_sources() {
    input_dir();
    // a file the document wrote (\pgfplotscolormaptodatafile): position, red, green, blue
    let d = described(
        "",
        r"\pgfplotsset{colormap={bw}{gray(0cm)=(0); gray(1cm)=(1)}}
          \pgfplotscolormaptodatafile{bw}{cm.dat}
          \addplot table[col sep=comma, header=false] {cm.dat};",
    );
    assert!(d.contains("2 points — (0, 0), (1000, 1)"), "{d}");
    // a file name with a macro in it
    let d = described("", "\\def\\name{faces}\\addplot table[header=false] {\\name.dat};");
    assert!(d.contains("one point — (0, 1)"), "{d}");
    // the data on the next line
    let d = described("", "\\addplot table\n{a b\n5 6\n};");
    assert!(d.contains("one point — (5, 6)"), "{d}");
    let d = described("", "\\addplot table[header=true]\n\n{a b\n5 7\n};");
    assert!(d.contains("one point — (5, 7)"), "{d}");
    // a single token for the data, `f` that does not start `from`: a file's name
    let w = warnings("", r"\addplot table f; \addplot table \relax;");
    assert!(has(&w, "\\addplot table: file 'f' not found"), "{w:?}");
    // `from` with the macro in braces
    let d = described("", "\\pgfplotstableread{p q\n1 6\n2 5}\\kept\n\\addplot table from {\\kept};");
    assert!(d.contains("(1, 6), (2, 5)"), "{d}");
    // \pgfplotstableread followed by no macro
    let w = warnings("", "\\pgfplotstableread{a b\n1 2} x \\addplot coordinates {(0,0)};");
    assert!(has(&w, "\\pgfplotstableread without a macro to keep the table: ignored"), "{w:?}");
    // the document ends after `table`, `table f` or `table from`
    for end in ["table", "table f", "table from"] {
        let r = tikra::convert(&format!(r"\begin{{tikzpicture}}\begin{{axis}}\addplot coordinates {{(0,0) (1,1)}}; \addplot {end}"));
        assert!(r.is_ok(), "{end}: {r:?}");
    }
}

#[test]
fn this_row_in_box_plot_filters_and_metas() {
    // the filter's `\thisrow{w}`: the row's cell as written; a row without a value stays out
    let expected = described("", r"\addplot+[boxplot] table[row sep=\\, y index=0] {10\\20\\40\\};");
    let d = described("", r"\addplot+[boxplot={data filter/.code={\edef\pgfmathresult{\thisrow{w}}}}] table[row sep=\\, y index=0] {v w\\ 1 10\\ 2 20\\ nan 30\\ 4 40\\};");
    assert_eq!(d, expected);
    // a column name with a group in it
    let d = described("", "\\addplot table[x expr=\\thisrow{x_{1}}, y=b] {x_{1} b\n3 4\n};");
    assert!(d.contains("one point — (3, 4)"), "{d}");
    // point meta from a column
    let d = described("", "\\addplot[scatter, point meta=\\thisrow{m}] table {x y m\n0 0 5\n1 1 7\n};");
    assert!(d.contains("Colors stand for values from 5 to 7"), "{d}");
}

#[test]
fn more_coordinate_inputs() {
    // a domain without its colon restricts nothing
    let d = described("", r"\addplot[restrict expr to domain={x}{5}] coordinates {(1,1) (2,2)};");
    assert!(d.contains("2 points"), "{d}");
    // error bars whose parentheses never close: the point, and nothing after
    let d = described("", r"\addplot coordinates {(0,0) +- (1};");
    assert!(d.contains("one point — (0, 0)"), "{d}");
    // radians in a polar axis
    let d = described("polaraxis:::", r"\addplot[data cs=polarrad] coordinates {(1.5707963267949,1)};");
    assert!(d.contains("one point — (90, 1)"), "{d}");
    // samples of a domain through 0 on a log axis: those at 0 or below are left out
    let d = described("xmode=log", r"\addplot[domain=0:2, samples=3] {x};");
    assert!(d.contains("2 points — (1, 1), (2, 2)"), "{d}");
    // a trailing path of more tokens than are read: the plot is drawn, the rest is ignored
    let long = "x".repeat(1_000_100);
    let r = common::figure(&picture("", &format!(r"\addplot coordinates {{(0,0) (1,1)}} {long};")));
    assert!(r.description.contains("2 points"), "{}", r.description);
}

// ---------------------------------------------------------------------------------------------
// limits and errors inside an axis

/// A macro that opens scopes without end.
const DEEP: &str = r"\def\deep{\begin{scope}\deep\end{scope}}";

/// Past the nesting limit the figure stops, wherever in the axis the code runs (TeX would run out
/// of memory).
#[test]
fn limits_inside_an_axis_stop_the_figure() {
    for (options, body) in [
        // the options of a plot, read while the axis is surveyed
        ("", r"\addplot[/utils/exec={\deep}] coordinates {(0,0) (1,1)};"),
        // a statement kept for when the axis is drawn
        ("", r"\addplot coordinates {(0,0) (1,1)}; \path[/utils/exec={\deep}] (0,0);"),
        // the code at the end of the axis' body
        (r"before end axis/.code={\deep}", r"\addplot coordinates {(0,0) (1,1)};"),
        // the code run around each scatter mark, when the plot is drawn
        ("", r"\addplot[scatter, scatter/@pre marker code/.code={\deep}] coordinates {(0,0) (1,1)};"),
        // the code run before a plot is drawn
        ("", r"\addplot[execute at begin plot visualization={\deep}] coordinates {(0,0) (1,1)};"),
    ] {
        let tikz = format!("{DEEP}{}", picture(options, body));
        let err = tikra::convert(&tikz).expect_err("a limit stops the figure");
        assert!(err.to_string().contains("nesting went past"), "{err}\n{body}");
    }
}

/// Errors that are no limit are reported and the axis is drawn: an endless macro in `before end
/// axis`, an error in the code before a plot (that plot is left out), a table macro that is not
/// defined.
#[test]
fn errors_inside_an_axis_are_reported() {
    let w = warnings(r"before end axis/.code={\loop}", r"\def\loop{\loop}\addplot coordinates {(0,0) (1,1)};");
    assert!(has(&w, "before end axis: figure: macro expansion did not end"), "{w:?}");
    let body = r"\addplot[red] coordinates {(0,0) (1,1)}; \addplot[blue, execute at begin plot visualization={\csname}] coordinates {(0,1) (1,0)};";
    let (c, w) = tikra::engine::run_document(&picture("", body)).unwrap();
    assert!(has(&w, "\\addplot: figure: \\csname without \\endcsname; plot not drawn"), "{w:?}");
    let colors: Vec<String> = strokes(&c).into_iter().map(|(color, _)| color).collect();
    assert!(colors.contains(&"#ff0000".to_string()) && !colors.contains(&"#0000ff".to_string()), "{colors:?}");
    let (d, w) = describe("", r"\addplot coordinates {(0,0) (1,1)}; \addplot table {\nosuchtable};");
    assert!(has(&w, "\\addplot table") && has(&w, "\\nosuchtable"), "{w:?}");
    assert!(d.contains("2 points"), "{d}");
}
