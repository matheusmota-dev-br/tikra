//! The statement reader: `\foreach`, `scope` and transforms, macros, environments and commands
//! that are not drawing.

mod common;
use common::*;
use tikra::scene::{Point, Transform};

/// The x of the start of each path drawn by `\foreach \x in {list}`.
fn foreach_values(list: &str) -> Vec<f64> {
    let c = canvas(&format!(r"\foreach \x in {{{list}}} {{ \draw (\x,0) -- (\x,1); }}"));
    paths(&c).iter().map(|d| points(&d.segments)[0].0).collect()
}

/// The texts `\foreach \x in {list}` puts in nodes, as the number of them.
fn foreach_texts(list: &str) -> usize {
    texts(&canvas(&format!(r"\foreach \x in {{{list}}} {{ \node {{\x}}; }}"))).len()
}

#[test]
fn foreach_lists_ellipsis_and_counter() {
    assert_eq!(foreach_values("0,1,...,4"), [0.0, 1.0, 2.0, 3.0, 4.0]);
    assert_eq!(foreach_values("0,0.5,...,2"), [0.0, 0.5, 1.0, 1.5, 2.0]);
    assert_eq!(foreach_values("5,4,...,1"), [5.0, 4.0, 3.0, 2.0, 1.0]);
    assert_eq!(foreach_texts("a,b,c"), 3);
    let c = canvas(r"\foreach \x [count=\i] in {a,b,c} { \node (n\i) at (\i,0) {\x}; }");
    assert_eq!(texts(&c).len(), 3);
    let p = point(r"\foreach \x [count=\i] in {a,b,c} { \node (n\i) at (\i,0) {\x}; }", "n3.center");
    assert!(close_to((p.x, p.y), (3.0, 0.0)), "{p:?}");
    // slash variables, a body without braces and a glued in
    let c = canvas(r"\foreach \p/\q in{1/2, 3} \draw (\p,0) -- (\q,1);");
    assert_eq!(paths(&c).len(), 2);
    // loop variables do not leak
    let (_, w) = canvas_and_warnings(r"\foreach \k in {1} {} \draw (\k,0) -- (1,1);");
    assert!(!w.is_empty());
}

#[test]
fn nested_foreach() {
    let c = canvas(r"\foreach \x in {0,1,2} \foreach \y in {0,1} \fill (\x,\y) circle (1pt);");
    assert_eq!(paths(&c).len(), 6);
    let c = canvas(r"\foreach \x in {0,1,2} {\foreach \y in {0,1} { \fill (\x,\y) circle (1pt); }}");
    assert_eq!(paths(&c).len(), 6);
}

#[test]
#[ignore = "the engine reads the outer body up to a top-level `;`; pgffor collects a nested \\foreach and its braced body (\\pgffor@collectforeach)"]
fn nested_foreach_with_a_braced_inner_body() {
    let c = canvas(r"\foreach \x in {0,1,2} \foreach \y in {0,1} { \fill (\x,\y) circle (1pt); }");
    assert_eq!(paths(&c).len(), 6);
}

#[test]
fn scopes_and_transforms() {
    let c = canvas(
        r"\coordinate (A) at (1,1);
          \begin{scope}[xshift=1cm, yshift=2cm, xscale=2, yscale=3, thick]
            \draw (0,0) -- (1,1);
          \end{scope}
          \begin{scope}[shift={(A)}, scale=2] \draw (0,0) -- (1,0); \end{scope}
          \begin{scope}[x={(1cm,1cm)}, y={(0cm,2cm)}] \draw (0,0) -- (1,1); \end{scope}
          \begin{scope}[x=2cm, y=3cm] \draw (0,0) -- (1,1); \end{scope}",
    );
    let t = paths(&c);
    assert!(close_to(points(&t[0].segments)[1], (3.0, 5.0)), "{:?}", points(&t[0].segments));
    assert!(close(t[0].stroke.as_ref().unwrap().width, 0.8 * tikra::PT), "option inherited from the scope");
    assert!(close_to(points(&t[1].segments)[0], (1.0, 1.0)) && close_to(points(&t[1].segments)[1], (3.0, 1.0)));
    assert!(close_to(points(&t[2].segments)[1], (1.0, 3.0)));
    assert!(close_to(points(&t[3].segments)[1], (2.0, 3.0)));
}

#[test]
fn errors_in_scope_options_are_figure_errors() {
    for op in ["scale=zz", "xscale=zz", "yscale=zz", "rotate=zz", "xshift=zz", "yshift=zz", "shift={(zz)}", "x=zz", "y={(zz)}"] {
        let r = tikra::engine::run_document(&format!(r"\begin{{tikzpicture}}\begin{{scope}}[{op}] \end{{scope}}\end{{tikzpicture}}"));
        assert!(r.is_err(), "{op}");
    }
}

#[test]
fn ends_of_scopes_with_no_begin_do_no_harm() {
    let c = canvas(r"\end{scope} \end{scope} \draw (0,0) -- (1,0);");
    assert_eq!(paths(&c).len(), 1);
}

#[test]
fn macros_def_newcommand_and_pgfmath() {
    let c = canvas(
        r"\def\a{1} \gdef\b{2} \edef\cc{3} \xdef\dd{4}
          \newcommand{\ee}{5} \renewcommand{\ff}[1]{6} \providecommand\gg{7}
          \pgfmathsetmacro{\total}{\a+\b+\cc+\dd+\ee+\ff+\gg}
          \pgfmathsetlengthmacro{\len}{2cm}
          \pgfmathparse{1+1}
          \draw (0,0) -- (\total,\len);",
    );
    // \ff takes one argument: as in TeX it eats the `+` after it, so the sum is 1+2+3+4+5+67
    let end = points(&paths(&c)[0].segments)[1];
    assert!(close(end.0, 82.0) && (end.1 - 2.0).abs() < 1e-6, "{end:?}");
    let (_, w) = canvas_and_warnings(r"\pgfmathsetmacro{\x}{\doesnotexist}");
    assert!(w.iter().any(|a| a.contains("pgfmathsetmacro")), "{w:?}");
}

#[test]
fn environments_and_end_without_begin() {
    // layers are drawn now, with no warning
    let (c, w) = canvas_and_warnings("\\begin{pgfonlayer}{background}\n\\draw (0,0) -- (1,0); \\end{pgfonlayer}");
    assert_eq!(paths(&c).len(), 1);
    assert!(w.is_empty(), "{w:?}");
    let (_, w) = canvas_and_warnings(r"\begin{scope} \end{other} \draw (0,0) -- (1,0); \end{scope}");
    assert!(w.iter().any(|a| a.contains("without a matching \\begin")), "{w:?}");
    let (_, w) = canvas_and_warnings(r"\begin{scope} \end{ \draw (0,0) -- (1,0);");
    assert!(w.iter().any(|a| a.contains("never closes")), "{w:?}");
}

#[test]
fn ignored_and_unknown_commands() {
    let (c, w) = canvas_and_warnings(
        r"\small \bfseries \centering \usetikzlibrary{calc} \usepackage[x]{y} \pgfplotsset{compat=1.18} \usepgfplotslibrary{fillbetween} \draw (0,0) -- (1,0);",
    );
    assert_eq!(paths(&c).len(), 1);
    assert!(w.is_empty(), "{w:?}");
    // \pic and \matrix are drawn now: what is left unknown is the pic type `x`
    let (_, w) = canvas_and_warnings(r"\shadow (0,0);  \pic {x}; \matrix {a};");
    assert!(w.iter().any(|a| a.contains("shadow")), "{w:?}");
    assert!(w.iter().any(|a| a.contains("'x'")), "{w:?}");
    let (_, w) = canvas_and_warnings("loose text here\n\\draw (0,0) -- (1,0);\n;\n");
    assert!(w.iter().any(|a| a.contains("text outside")), "{w:?}");
    // the same warning is not repeated
    let (_, w) = canvas_and_warnings(r"\shadow; \shadow;");
    assert_eq!(w.len(), 1);
}

#[test]
fn statement_without_semicolon_runs_to_the_end() {
    let c = canvas(r"\draw (0,0) -- (1,0)");
    assert_eq!(paths(&c).len(), 1);
}

#[test]
fn declare_function_and_globals() {
    let c = canvas(r"\begin{scope}[declare function={f(\x)=2*\x; g(\a,\b)=\a+\b; bad; noparens=1}] \draw (0,0) -- ({f(2)},{g(1,2)}); \end{scope}");
    assert!(close_to(points(&paths(&c)[0].segments)[1], (4.0, 3.0)));
}

#[test]
fn line_widths_and_transforms() {
    assert!(close(first_path(r"\draw[thick] (0,0) -- (1,0);").stroke.unwrap().width, 0.8 * tikra::PT));
    let (_, w) = canvas_and_warnings(r"\draw[chunky] (0,0) -- (1,0);");
    assert!(w.iter().any(|a| a.contains("chunky")), "{w:?}");
    // singular matrix: identity
    assert_eq!(Transform::scaling(0.0, 1.0).inverse(), Transform::default());
    let t = Transform::rotation(30.0).compose(&Transform::translation(1.0, 2.0));
    let p = t.inverse().apply(t.apply(Point::new(3.0, 4.0)));
    assert!(close_to((p.x, p.y), (3.0, 4.0)));
    assert!(close(Transform::scaling(2.0, 8.0).factor(), 4.0));
}

#[test]
fn groups_and_comments() {
    // parentheses inside braces do not end a coordinate
    let d = first_path(r"\draw (0,0) -- ({(1)+1}, {max(1,2)});");
    assert!(close_to(points(&d.segments)[1], (2.0, 2.0)), "{:?}", d.segments);
    // a comment runs to the end of the line; \% is a percent sign
    let c = canvas("\\draw (0,0) -- (1,0); % \\draw (5,5) -- (6,6);\n\\node {50\\% off};");
    assert_eq!(paths(&c).len(), 1);
    assert_eq!(texts(&c).len(), 1);
    assert!(tikra::convert("% \\begin{tikzpicture} \\draw (0,0) -- (1,0); \\end{tikzpicture}").is_err(), "a picture in a comment is no picture");
}

#[test]
fn errors_inside_environments_and_loops_propagate() {
    for t in [r"\begin{whatever} \begin{scope}[scale=zz] \end{scope} \end{whatever}", r"\foreach \x in {1} { \begin{scope}[scale=zz] \end{scope} }"] {
        assert!(tikra::engine::run_document(&format!(r"\begin{{tikzpicture}}{t}\end{{tikzpicture}}")).is_err(), "{t}");
    }
}
