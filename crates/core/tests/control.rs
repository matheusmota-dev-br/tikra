//! The statement reader: `\foreach`, `scope` and transforms, macros, environments and commands
//! that are not drawing.

mod common;
use common::*;
use tikra::reader::{expand_list, format_number, invert, line_width, Env, Reader};
use tikra::scene::Transform;

#[test]
fn foreach_lists_ellipsis_and_counter() {
    assert_eq!(expand_list("0,1,...,4"), ["0", "1", "2", "3", "4"]);
    assert_eq!(expand_list("0,0.5,...,2"), ["0", "0.5", "1", "1.5", "2"]);
    assert_eq!(expand_list("5,...,1"), ["5"], "with no two terms the step is 1: going up from 5, nothing reaches 1");
    assert_eq!(expand_list("5,4,...,1"), ["5", "4", "3", "2", "1"]);
    assert_eq!(expand_list("1,1,...,5"), ["1", "1"], "a zero step does not repeat");
    assert_eq!(expand_list("a,b,c"), ["a", "b", "c"]);
    assert_eq!(expand_list("...,3"), ["...", "3"], "a leading ellipsis stays as text");
    let c = scene(r"\foreach \x [count=\i] in {a,b,c} { \node at (\i,0) {\x}; }");
    assert_eq!(nodes(&c).len(), 3);
    assert!(close(nodes(&c)[2].center.x, 3.0));
    // slash variables, a body without braces and a glued in
    let c = scene(r"\foreach \p/\q in{1/2, 3} \draw (\p,0) -- (\q,1);");
    assert_eq!(paths(&c).len(), 2);
    // loop variables do not leak
    let (_, w) = scene_and_warnings(r"\foreach \k in {1} {} \draw (\k,0) -- (1,1);");
    assert!(!w.is_empty());
}

#[test]
fn nested_foreach() {
    let c = scene(r"\foreach \x in {0,1,2} \foreach \y in {0,1} { \fill (\x,\y) circle (1pt); }");
    assert_eq!(paths(&c).len(), 6);
}

#[test]
fn scopes_and_transforms() {
    let c = scene(
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
    assert!(close(t[0].style.line_width, 0.8 * tikra::PT), "option inherited from the scope");
    assert!(close_to(points(&t[1].segments)[0], (1.0, 1.0)) && close_to(points(&t[1].segments)[1], (3.0, 1.0)));
    assert!(close_to(points(&t[2].segments)[1], (1.0, 3.0)));
    assert!(close_to(points(&t[3].segments)[1], (2.0, 3.0)));
}

#[test]
fn errors_in_scope_options_are_figure_errors() {
    for op in ["scale=zz", "xscale=zz", "yscale=zz", "rotate=zz", "xshift=zz", "yshift=zz", "shift={(zz)}", "x=zz", "y={(zz)}"] {
        let mut env = Env::new();
        let r = Reader::new(&format!(r"\begin{{scope}}[{op}] \end{{scope}}"), &mut env).run(None);
        assert!(r.is_err(), "{op}");
    }
}

#[test]
fn pop_scope_never_leaves_no_scope() {
    let mut env = Env::default();
    let mut r = Reader::new("", &mut env);
    r.pop_scope();
    r.pop_scope();
    assert_eq!(r.env.scopes.len(), 1);
}

#[test]
fn macros_def_newcommand_and_pgfmath() {
    let c = scene(
        r"\def\a{1} \gdef\b{2} \edef\cc{3} \xdef\dd{4}
          \newcommand{\ee}{5} \renewcommand{\ff}[1]{6} \providecommand\gg{7}
          \pgfmathsetmacro{\total}{\a+\b+\cc+\dd+\ee+\ff+\gg}
          \pgfmathsetlengthmacro{\len}{2cm}
          \pgfmathparse{1+1}
          \draw (0,0) -- (\total,\len);",
    );
    assert!(close_to(points(&paths(&c)[0].segments)[1], (28.0, 2.0)));
    let (_, w) = scene_and_warnings(r"\pgfmathsetmacro{\x}{\doesnotexist}");
    assert!(w.iter().any(|a| a.contains("pgfmathsetmacro")), "{w:?}");
}

#[test]
fn environments_and_end_without_begin() {
    let (c, w) = scene_and_warnings("\\begin{pgfonlayer}{background}\n\\draw (0,0) -- (1,0); \\end{pgfonlayer}");
    assert_eq!(paths(&c).len(), 1);
    assert!(w.iter().any(|a| a.contains("pgfonlayer")));
    let mut env = Env::new();
    Reader::new(r"\begin{scope} \end{other} \draw (0,0) -- (1,0); \end{scope}", &mut env).run(None).unwrap();
    assert!(env.warnings.iter().any(|a| a.contains("without a matching \\begin")), "{:?}", env.warnings);
    let mut env = Env::new();
    Reader::new(r"\begin{scope} \end{ \draw (0,0) -- (1,0);", &mut env).run(None).unwrap();
    assert!(env.warnings.iter().any(|a| a.contains("without '}'")), "{:?}", env.warnings);
}

#[test]
fn ignored_and_unknown_commands() {
    let (c, w) = scene_and_warnings(r"\small \bfseries \centering \usetikzlibrary{calc} \usepackage[x]{y} \pgfplotsset{compat=1.18} \usepgfplotslibrary{fillbetween} \draw (0,0) -- (1,0);");
    assert_eq!(paths(&c).len(), 1);
    assert!(w.is_empty(), "{w:?}");
    let (_, w) = scene_and_warnings(r"\shadow (0,0);  \pic {x}; \matrix {a};");
    assert!(w.iter().any(|a| a.contains("shadow")), "{w:?}");
    assert!(w.iter().any(|a| a.contains("pic")), "{w:?}");
    let (_, w) = scene_and_warnings("loose text here\n\\draw (0,0) -- (1,0);\n;\n");
    assert!(w.iter().any(|a| a.contains("text outside")), "{w:?}");
    // the same warning is not repeated
    let (_, w) = scene_and_warnings(r"\shadow; \shadow;");
    assert_eq!(w.len(), 1);
}

#[test]
fn statement_without_semicolon_runs_to_the_end() {
    let c = scene(r"\draw (0,0) -- (1,0)");
    assert_eq!(paths(&c).len(), 1);
}

#[test]
fn declare_function_and_globals() {
    let c = scene(r"\begin{scope}[declare function={f(\x)=2*\x; g(\a,\b)=\a+\b; bad; noparens=1}] \draw (0,0) -- ({f(2)},{g(1,2)}); \end{scope}");
    assert!(close_to(points(&paths(&c)[0].segments)[1], (4.0, 3.0)));
}

#[test]
fn numeric_helpers() {
    assert_eq!(format_number(2.0), "2");
    assert_eq!(format_number(0.1 + 0.2), "0.3");
    assert_eq!(format_number(1e20), "100000000000000000000");
    assert_eq!(line_width("thick"), Some(0.8 * tikra::PT));
    assert_eq!(line_width("chunky"), None);
    // singular matrix: identity
    assert_eq!(invert(&Transform::scaling(0.0, 1.0)), Transform::default());
    let t = Transform::rotation(30.0).compose(&Transform::translation(1.0, 2.0));
    let p = invert(&t).apply(t.apply(tikra::scene::Point::new(3.0, 4.0)));
    assert!(close_to((p.x, p.y), (3.0, 4.0)));
    assert!(close(Transform::scaling(2.0, 8.0).factor(), 4.0));
}

#[test]
fn block_and_comments() {
    use tikra::reader::{block, strip_comments};
    let c: Vec<char> = r"(a, {b)c}) rest".chars().collect();
    assert_eq!(block(&c, 0).unwrap().0, "a, {b)c}");
    let c: Vec<char> = r"{a\}b}".chars().collect();
    assert_eq!(block(&c, 0).unwrap().0, r"a\}b");
    assert!(block(&['x'], 0).is_none());
    assert!(block(&['('], 0).is_none());
    let c: Vec<char> = "({".chars().collect();
    assert!(block(&c, 0).is_none(), "an unmatched brace inside parentheses");
    assert_eq!(strip_comments("a % b\nc \\% d\\"), "a \nc \\% d\\\n");
}

#[test]
fn errors_inside_environments_and_loops_propagate() {
    for t in [r"\begin{whatever} \begin{scope}[scale=zz] \end{scope} \end{whatever}", r"\foreach \x in {1} { \begin{scope}[scale=zz] \end{scope} }"] {
        let mut env = Env::new();
        assert!(Reader::new(t, &mut env).run(None).is_err(), "{t}");
    }
}
