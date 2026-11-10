//! Coordinates and options: the coordinate systems (`canvas`, `xyz`, `intersection`,
//! `barycentric`, `node`, declared ones), polar and calc forms and their errors; pgfkeys' handlers
//! (`.search also`, `.unknown`, `.code 2 args`, `.list`, `.get`, `.add`…), the first char syntax,
//! stored values used as keys, and TikZ keys that only a few figures use.

mod common;
use common::*;
use tikra::canvas::Canvas;
use tikra::engine::run_document;
use tikra::scene::Segment;

/// Half the default line width, in cm: where a line from the origin starts on the page.
const HALF: f64 = 0.2 * 2.54 / 72.27;
const PT: f64 = 2.54 / 72.27;

/// The canvas and warnings of a picture body.
fn picture(body: &str) -> (Canvas, Vec<String>) {
    run_document(&format!("\\begin{{tikzpicture}}{body}\\end{{tikzpicture}}")).unwrap_or_else(|e| panic!("{e}\n{body}"))
}

/// The canvas of a picture body, which must not warn.
fn quiet(body: &str) -> Canvas {
    let (c, w) = picture(body);
    assert!(w.is_empty(), "warnings: {w:?}\n{body}");
    c
}

/// The points of the first path of a picture body, from the picture's lower left corner.
fn line(body: &str) -> Vec<(f64, f64)> {
    let c = quiet(body);
    points(&paths(&c)[0].segments)
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

/// The one warning a picture body gives.
fn warning(body: &str) -> String {
    let (_, w) = picture(body);
    assert_eq!(w.len(), 1, "{w:?}\n{body}");
    w[0].clone()
}

mod coordinate_systems {
    use super::*;

    /// `canvas cs` and `xyz cs`, with their parts or none (the origin).
    #[test]
    fn canvas_and_xyz() {
        let p = line(r"\draw (canvas cs:x=1cm,y=2cm) -- (xyz cs:x=1,z=0) -- (canvas cs:) -- (xyz cs:);");
        let q: Vec<(f64, f64)> = p.iter().map(|(x, y)| ((x - HALF), (y - HALF))).collect();
        assert!(near(q[0].0, 1.0) && near(q[0].1, 2.0) && near(q[1].0, 1.0) && near(q[1].1, 0.0) && near(q[2].0, 0.0) && near(q[3].1, 0.0), "{q:?}");
    }

    /// `intersection cs` of two lines; with one line only, an error.
    #[test]
    fn intersection_cs() {
        let p = line(r"\draw (0,0) -- (intersection cs:first line={(0,0)--(2,2)}, second line={(0,2)--(2,0)});");
        assert!(near(p[1].0 - p[0].0, 1.0) && near(p[1].1 - p[0].1, 1.0), "{p:?}");
        assert_eq!(warning(r"\draw (0,0) -- (intersection cs:first line={(0,0)--(2,2)});"), "\\draw: figure: intersection cs: only lines are supported; statement ignored");
    }

    /// `barycentric cs`: the nodes' centers weighted; weights that add up to zero, and a node that
    /// does not exist, are errors.
    #[test]
    fn barycentric() {
        let nodes = r"\node (a) at (0,0) {}; \node (b) at (2,0) {}; \node (c) at (0,2) {};";
        let p = point(nodes, "barycentric cs:a=1,b=1,c=2");
        assert!(close_to((p.x, p.y), (0.5, 1.0)), "{p:?}");
        assert_eq!(warning(&format!(r"{nodes}\draw (0,-1) -- (barycentric cs:a=1,b=-1);")), "\\draw: figure: barycentric cs: the weights add up to zero; statement ignored");
        assert_eq!(warning(r"\draw (0,-1) -- (barycentric cs:zz=1);"), "\\draw: figure: unknown node: zz; statement ignored");
    }

    /// `node cs`: an anchor, an angle, or the center; an unknown anchor or node is an error.
    #[test]
    fn node_cs() {
        let node = r"\node[draw, minimum width=2cm, minimum height=1cm, inner sep=0pt, outer sep=0pt] (a) at (1,1) {};";
        let n = point(node, "node cs:name=a, anchor=north");
        assert!(close_to((n.x, n.y), (1.0, 1.5)), "{n:?}");
        let e = point(node, "node cs:name=a, angle=0");
        assert!(close_to((e.x, e.y), (2.0, 1.0)), "{e:?}");
        let c = point(node, "node cs:name=a");
        assert!(close_to((c.x, c.y), (1.0, 1.0)), "{c:?}");
        assert_eq!(warning(&format!(r"{node}\draw (0,0) -- (node cs:name=a, anchor=nowhere);")), "\\draw: figure: unknown anchor: a.nowhere; statement ignored");
        assert_eq!(warning(r"\draw (0,0) -- (node cs:name=zz);"), "\\draw: figure: unknown node: zz; statement ignored");
    }

    /// `\tikzdeclarecoordinatesystem`: its code's last point at the top level is the coordinate,
    /// the code before it run; code with no point is an error, and so is a system nobody declared.
    #[test]
    fn declared_systems() {
        let p = line(
            r"\tikzdeclarecoordinatesystem{twice}{\pgfpoint{2cm}{#1}}\tikzdeclarecoordinatesystem{last}{{\pgfpoint{5cm}{0cm}}\def\q{3cm}\pgfpoint{1cm}{\q}}\draw (0,0) -- (twice cs:1cm) -- (last cs:);",
        );
        assert!(near(p[1].0 - p[0].0, 2.0) && near(p[1].1 - p[0].1, 1.0) && near(p[2].0 - p[0].0, 1.0) && near(p[2].1 - p[0].1, 3.0), "{p:?}");
        assert_eq!(warning(r"\tikzdeclarecoordinatesystem{none}{\relax}\draw (0,0) -- (none cs:);"), "\\draw: figure: coordinate system 'none' gives no point; statement ignored");
        assert_eq!(warning(r"\draw (0,0) -- (mystery cs:a=1);"), "\\draw: figure: coordinate system 'mystery' not supported; statement ignored");
    }
}

mod coordinate_forms {
    use super::*;

    /// `(intersection of A--B and C--D)`: where the lines meet; parallel lines, no `and`, or a line
    /// that is not `A--B` are errors.
    #[test]
    fn intersection_of() {
        let p = line(r"\draw (0,2) -- (intersection of 0,0--2,2 and 0,2--2,0);");
        assert!(near(p[1].0 - p[0].0, 1.0) && near(p[1].1 - p[0].1, -1.0), "{p:?}");
        for (spec, error) in [
            ("0,0--2,2 and 0,1--2,3", "intersection of parallel lines"),
            ("0,0--2,2 or 0,1--2,3", "intersection of: missing 'and'"),
            ("0,0 2,2 and 0,1--2,3", "intersection of: a line is A--B"),
        ] {
            assert_eq!(warning(&format!(r"\draw (0,2) -- (intersection of {spec});")), format!("\\draw: figure: {error}; statement ignored"));
        }
    }

    /// Polar coordinates with two radii, and in radians with `trig format=rad`.
    #[test]
    fn polar() {
        let p = point("", "90:1 and 2");
        assert!(close_to((p.x, p.y), (0.0, 2.0)), "{p:?}");
        let p = line(r"\begin{scope}[trig format=rad]\draw (0,0) -- (pi/2:1);\end{scope}");
        assert!(near(p[1].0, p[0].0) && near(p[1].1 - p[0].1, 1.0), "{p:?}");
    }

    /// A z part with a unit follows the z vector's direction, as long as the length says.
    #[test]
    fn z_with_a_unit() {
        let p = line(r"\draw (0,0,0) -- (0,0,1cm);");
        let (dx, dy) = (p[1].0 - p[0].0, p[1].1 - p[0].1);
        assert!(near(dx.hypot(dy), 1.0) && near(dx, dy) && dx < 0.0, "{p:?}");
    }

    /// `rand` in a coordinate: a number between -1 and 1, drawn twice as TikZ draws it.
    #[test]
    fn random() {
        let p = line(r"\draw (rand,0) -- (1,1);");
        let x = p[0].0 - (p[1].0 - 1.0);
        assert!((-1.0..=1.0).contains(&x), "{p:?}");
    }

    /// `(current subpath start)`: where the subpath began.
    #[test]
    fn current_subpath_start() {
        let p = line(r"\draw (1,1) -- (2,1) -- (current subpath start);");
        assert!(near(p[2].0, p[0].0) && near(p[2].1, p[0].1), "{p:?}");
    }

    /// calc: a factor in braces before a point, a function of numbers before a point, and the
    /// projection of a point on a line.
    #[test]
    fn calc_factors_and_projections() {
        let p = point("", "$(1,0) + {1+1}*(0,1)$");
        assert!(close_to((p.x, p.y), (1.0, 2.0)), "{p:?}");
        let p = point("", "$cos(0)*(0,1)$");
        assert!(close_to((p.x, p.y), (0.0, 1.0)), "{p:?}");
        let p = point("", "$(1,0)!(0,1)!(3,0)$");
        assert!(close_to((p.x, p.y), (0.0, 0.0)), "{p:?}");
    }

    /// Coordinates that cannot be read: a bracket left open, an empty part, parentheses that do
    /// not close in calc or ever, a projection with no second point.
    #[test]
    fn errors() {
        for (body, error) in [
            (r"\draw ([red 0,0) -- (1,1);", "unmatched bracket in ([red 0,0)"),
            (r"\draw (0,0) -- (,1);", "unknown coordinate: (,1)"),
            (r"\draw (0,0) -- ($(1,{(}0)$);", "unmatched parenthesis in calc"),
            (r"\draw (0,0) -- ($(1,0)!(0,1)!x$);", "calc: missing the second point"),
            (r"\draw (0,0) -- (", "unmatched parenthesis"),
        ] {
            assert_eq!(warning(body), format!("\\draw: figure: {error}; statement ignored"), "{body}");
        }
        // past TeX's limit of tokens, still looking for the `)`
        let long = format!(r"\draw (0,0) -- ({}", "a".repeat(1_000_100));
        assert_eq!(warning(&long), "\\draw: figure: unmatched parenthesis; statement ignored");
    }

    /// `handle active characters in code`: an active `!` in calc and in a color is the character.
    #[test]
    fn active_characters_in_code() {
        let c = quiet(
            r"\catcode`\!=13 \def!{x}\draw[handle active characters in code] (0,1) -- ($(0,0)!0.5!(2,0)$); \draw[handle active characters in code, draw=red!50!white] (0,0)--(1,0);",
        );
        let p = paths(&c);
        let q = points(&p[0].segments);
        assert!(near(q[1].0 - q[0].0, 1.0) && near(q[1].1 - q[0].1, -1.0), "{q:?}");
        let s = p[1].stroke.as_ref().unwrap();
        assert!(near(s.color.r, 1.0) && near(s.color.g, 0.5) && near(s.color.b, 0.5), "{:?}", s.color);
    }
}

mod pgfkeys {
    use super::*;

    /// `/handlers/first char syntax`: a key starting with a character that has a macro is that
    /// macro, called with the whole key (and its value).
    #[test]
    fn first_char_syntax() {
        let c =
            quiet(r"\pgfkeys{/handlers/first char syntax=true, /handlers/first char syntax/the character ;/.initial=\mysemi}\def\mysemi#1{\draw (0,0) -- (1,0);}\path[;thing=3];");
        assert_eq!(paths(&c).len(), 1);
        let c = quiet(
            r"\pgfkeys{/handlers/first char syntax=true}\pgfkeys{/handlers/first char syntax/the character !/.initial=\mybang}\def\mybang#1{\draw (0,0) -- (2,0);}\path[!thing];",
        );
        assert!(near(points(&paths(&c)[0].segments)[1].0, 2.0 + HALF));
    }

    /// A stored value used as a key: its value is inserted.
    #[test]
    fn a_value_used_as_a_key() {
        let c = quiet(r"\pgfkeys{/my/val/.initial={\draw (0,0)--(3,0);}}\pgfkeys{/my/val}");
        assert!(near(points(&paths(&c)[0].segments)[1].0, 3.0 + HALF));
    }

    /// `.unknown` handlers: set with `/.unknown/.code` or `/.unknown=`, they run for the keys of
    /// their path; one that is not code does not, and the key is unknown.
    #[test]
    fn unknown_handlers() {
        for set in [r"\pgfkeys{/my/.unknown/.code={\draw (0,0)--(1,0);}}", r"\pgfkeys{/foo/.unknown={\draw (0,0)--(1,0);}}"] {
            let path = if set.contains("/my/") { "/my" } else { "/foo" };
            let c = quiet(&format!(r"{set}\pgfkeys{{{path}/what=1}}"));
            assert_eq!(paths(&c).len(), 1, "{set}");
        }
        assert_eq!(warning(r"\expandafter\let\csname pgfk@/q/.unknown/.@cmd\endcsname\relax \pgfkeys{/q/zz}"), "option 'zz' not supported: ignored");
    }

    /// `.search also`: an unknown key of the path is looked for in the others.
    #[test]
    fn search_also() {
        let c = quiet(r"\tikzset{/a/.search also={/tikz}}\draw[/a/.cd, line width=3pt] (0,0)--(1,0);");
        assert!(near(paths(&c)[0].stroke.as_ref().unwrap().width, 3.0 * PT));
        assert_eq!(picture(r"\pgfkeys{/b/.search also={/c}}\pgfkeys{/b/zzz}").1, ["option 'zzz' not supported: ignored"]);
    }

    /// Styles and code: `.append style`, `.append code`, `.style 2 args` (a missing second argument
    /// is empty), `##` in a key that defines a key.
    #[test]
    fn styles_and_code() {
        let c = quiet(
            r"\tikzset{a/.style={red}, a/.append style={thick}, b/.code={\def\q##1{##1}}, b/.append code={}, c/.style 2 args={draw=#1, line width=#2}, c/.append style={dashed}}\draw[a] (0,0)--(1,0);\draw[c={blue}{2pt}] (0,1)--(1,1);",
        );
        let s: Vec<_> = paths(&c).iter().map(|d| d.stroke.clone().unwrap()).collect();
        assert!(near(s[0].color.r, 1.0) && near(s[0].width, 0.8 * PT), "{:?}", s[0]);
        assert!(near(s[1].color.b, 1.0) && near(s[1].width, 2.0 * PT) && s[1].dash.is_some(), "{:?}", s[1]);
        let c = quiet(r"\tikzset{e/.style 2 args={draw=#1, #2}}\draw[e={{red}}] (0,1)--(1,1);");
        assert!(near(paths(&c)[0].stroke.as_ref().unwrap().color.r, 1.0));
        for append in ["", ", outer/.append code={}"] {
            let c = quiet(&format!(r"\tikzset{{outer/.code={{\tikzset{{inner/.style={{line width=##1}}}}}}{append}}}\tikzset{{outer}}\draw[inner=3pt] (0,0)--(1,0);"));
            assert!(near(paths(&c)[0].stroke.as_ref().unwrap().width, 3.0 * PT), "{append}");
        }
    }

    /// pgfplots' filter handlers (`.condition`, `.from`, `.expression`, `.append expression`),
    /// `.prefix code`, `.code n args` and `.code args`.
    #[test]
    fn more_handlers() {
        let c = quiet(
            r"\pgfkeys{/f/x filter/.condition={1}{red}, /f/x filter/.condition={2}{blue}, /f/y/.from={a}, /f/z/.expression={x}, /f/z/.append expression={y},
              /f/w/.append expression={y}, /f/p/.prefix code={\relax}, /f/q/.code n args={3}{#1#2#3}, /f/r/.code args={#1 and #2}{\draw (#1,0)--(#2,0);}}\pgfkeys{/f/r=0 and 2}",
        );
        let p = points(&paths(&c)[0].segments);
        assert!(near(p[1].0 - p[0].0, 2.0), "{p:?}");
    }

    /// `.expand once`, `.get`, `.add`, `.prefix`, `.list` (of a key and of none), `.pic`, and a
    /// handler nobody defined.
    #[test]
    fn value_handlers() {
        let (c, w) = picture(
            r"\pgfkeys{/g/h/.expand once=\x, /g/k/.code={}, /g/k/.expand once={a}, /g/v/.initial=1, /g/v/.add={0}{2}, /g/v/.prefix={9}, /g/v/.get=\vv,
              /g/l/.list={a,b}, /g/m/.code={\draw (#1,1) -- (#1,2);}, /g/m/.list={1,2}, mypic/.pic={\draw (0,0)--(1,0);}}\pic {mypic};\node at (0,3) {\vv}; \pgfkeys{/g/v/.mystery handler}",
        );
        // `/g/h` and `/g/l` are no keys: expanded or listed, each value is an unknown key
        assert_eq!(
            w,
            ["option 'h' not supported: ignored", "option 'l' not supported: ignored", "key handler '.mystery handler' not supported (in '/g/v/.mystery handler'): ignored"]
        );
        // the two lines of `.list` and the pic's
        assert_eq!(paths(&c).len(), 3);
        // `\vv` is `9012`: four digits, wider than one
        let one = texts(&quiet(r"\node {9};"))[0].0.width();
        assert!(texts(&c)[0].0.width() > 3.5 * one);
    }
}

mod rare_keys {
    use super::*;

    /// `dash=… phase …`, `dash=solid`, `dash phase` alone, and a word that is not `on` or `off`.
    #[test]
    fn dashes() {
        let c = quiet(
            r"\draw[dash=on 2pt off 1pt phase 1pt] (0,0)--(1,0); \draw[dash=solid] (0,1)--(1,1); \draw[dash phase=1pt] (0,2)--(1,2); \draw[dash=on 2pt foo off 1pt] (0,3)--(1,3);",
        );
        let d: Vec<_> = paths(&c).iter().map(|p| p.stroke.as_ref().unwrap().dash.clone()).collect();
        let (on_off, phase) = d[0].clone().unwrap();
        assert!(near(on_off[0], 2.0 * PT) && near(on_off[1], PT) && near(phase, PT), "{d:?}");
        assert!(d[1].is_none());
        // a phase with no pattern: still solid
        assert!(d[2].is_none(), "{d:?}");
        assert!(d[3].as_ref().is_some_and(|(v, _)| v.len() == 2), "{d:?}");
    }

    /// `z=1cm` and `rotate around x/y/z`: the unit vectors turned about each axis.
    #[test]
    fn rotations_about_the_axes() {
        let p = line(r"\begin{scope}[rotate around z=90]\draw (0,0,0)--(1,0,0);\end{scope}");
        assert!(near(p[1].0 - p[0].0, 0.0) && near(p[1].1 - p[0].1, 1.0), "{p:?}");
        let p = line(r"\begin{scope}[rotate around x=90]\draw (0,0,0)--(0,1,0);\end{scope}");
        let z = line(r"\draw (0,0,0)--(0,0,1);");
        assert!(near(p[1].0 - p[0].0, z[1].0 - z[0].0) && near(p[1].1 - p[0].1, z[1].1 - z[0].1), "{p:?} {z:?}");
        let p = line(r"\begin{scope}[rotate around y=90]\draw (0,0,0)--(0,0,1);\end{scope}");
        assert!(near(p[1].0 - p[0].0, 1.0) && near(p[1].1 - p[0].1, 0.0), "{p:?}");
        let p = line(r"\begin{scope}[z=1cm]\draw (0,0,0)--(0,0,1);\end{scope}");
        assert!(near(p[1].0 - p[0].0, 1.0) && near(p[1].1 - p[0].1, 1.0), "{p:?}");
    }

    /// `above right=1cm of a`: a single distance along the diagonal is measured along it.
    #[test]
    fn diagonal_distance() {
        let body = r"\node[inner sep=0pt, outer sep=0pt, minimum size=0pt] (a) {}; \node[above right=1cm of a, inner sep=0pt, outer sep=0pt, minimum size=0pt] (b) {};";
        let b = point(&format!(r"\usetikzlibrary{{positioning}}{body}"), "b");
        assert!(close_to((b.x, b.y), (0.5f64.sqrt(), 0.5f64.sqrt())), "{b:?}");
    }

    /// `save path` and `use path`: the path drawn again by another statement.
    #[test]
    fn saved_paths() {
        let c = quiet(r"\draw[save path=\p] (0,0)--(1,1); \path[use path=\p, draw=red];");
        let p = paths(&c);
        assert_eq!(points(&p[0].segments), points(&p[1].segments));
        assert!(near(p[1].stroke.as_ref().unwrap().color.r, 1.0));
    }

    /// The distances of `to`'s control points: each set, then the curve.
    #[test]
    fn to_distances() {
        let c = quiet(
            r"\draw (0,0) to[out=90, in=90, in min distance=1cm, in max distance=2cm, out min distance=1cm, out max distance=2cm, in distance=1cm, out distance=1cm, min distance=1cm, max distance=1cm, distance=1cm, in min distance] (2,0);",
        );
        let Segment::CurveTo(a, b, _) = paths(&c)[0].segments[1] else { panic!("{:?}", paths(&c)[0].segments) };
        assert!(near(a.y - HALF, 1.0) && near(b.y - HALF, 1.0), "{a:?} {b:?}");
    }

    /// `ampersand replacement=\&`: the cells of a matrix apart by it; a replacement that is not a
    /// command leaves `&`.
    #[test]
    fn ampersand_replacement() {
        for code in [r"\matrix[ampersand replacement=\&] {\node{a}; \& \node{b};\\};", r"\matrix[ampersand replacement=x] {\node{a}; & \node{b};\\};"] {
            let c = quiet(code);
            let t = texts(&c);
            assert_eq!(t.len(), 2, "{code}");
            assert!(t[1].1.e > t[0].1.e + t[0].0.width(), "{code}");
        }
    }

    /// A positioning key with an empty value only sets the anchor: `above right=` puts the node's
    /// south west corner at its point.
    #[test]
    fn empty_positioning() {
        let b = point(r"\node[inner sep=0pt, outer sep=0pt, above right=] (c) at (1,1) {};", "c.south west");
        assert!(close_to((b.x, b.y), (1.0, 1.0)), "{b:?}");
    }

    /// The quotes library: a text whose closing `"` is missing is said, and the node drawn.
    #[test]
    fn quotes_not_closed() {
        let (c, w) = picture(r#"\usetikzlibrary{quotes}\node["abc] {x};"#);
        assert_eq!(w, ["quotes: closing '\"' missing in \"abc"]);
        assert_eq!(texts(&c).len(), 1);
    }

    /// An argument a `.style 2 args` is not given is empty; spaces between the arguments are
    /// skipped, as TeX skips them.
    #[test]
    fn style_arguments() {
        let c = quiet(r"\tikzset{f/.style 2 args={line width=#1pt#2}}\draw[f=3] (0,0)--(1,0);");
        assert!(near(paths(&c)[0].stroke.as_ref().unwrap().width, 3.0 * PT));
        let c = quiet(r"\tikzset{g/.style 2 args={draw=#1, line width=#2}}\draw[g={red} {2pt}] (0,0)--(1,0);");
        let s = paths(&c)[0].stroke.clone().unwrap();
        assert!(near(s.width, 2.0 * PT) && near(s.color.r, 1.0), "{s:?}");
    }

    /// An empty part of a plain coordinate in an axis is 0, as TeX recovers.
    #[test]
    fn an_empty_part_in_an_axis() {
        let first = |p: &str| {
            let c = quiet(&format!(r"\begin{{axis}}[xmin=0,xmax=2,ymin=0,ymax=2]\draw[red] {p} -- (2,1);\end{{axis}}"));
            paths(&c).into_iter().find(|d| d.stroke.as_ref().is_some_and(|s| near(s.color.r, 1.0) && s.color.g < 0.5)).map(|d| points(&d.segments)).unwrap()
        };
        assert_eq!(first("(,1)"), first("(0,1)"));
    }

    /// `view={… at …}` of the views library: a clip and a scaling; without `at`, nothing.
    #[test]
    fn views() {
        let c = quiet(
            r"\usetikzlibrary{views}\begin{scope}[view={(0,0) (1,1) at (0,0) (2,2)}]\draw (0,0)--(1,1);\end{scope}\begin{scope}[view={(0,0) (1,1)}]\draw (0,0)--(1,0);\end{scope}",
        );
        let kinds: Vec<bool> = c.items.iter().map(|i| matches!(i, tikra::canvas::Item::Clip { .. })).collect();
        assert_eq!(kinds.iter().filter(|k| **k).count(), 1);
        let t = c.items.iter().find_map(|i| match i {
            tikra::canvas::Item::Transform { transform } => Some(*transform),
            _ => None,
        });
        assert!(near(t.unwrap().a, 2.0));
    }

    /// `make snapshot after`, and `myself:` animations of a path's options in a snapshot.
    #[test]
    fn snapshots() {
        let c = quiet(r#"\usetikzlibrary{animations}\tikzset{make snapshot of=0.5s}\draw[myself:line width={0s="1pt", 1s="3pt"}] (0,0) -- (1,0);"#);
        assert!(near(paths(&c)[0].stroke.as_ref().unwrap().width, 2.0 * PT));
        // just after the end of a fade: over, back to the path's own opacity
        let c = quiet(r#"\usetikzlibrary{animations}\tikzset{make snapshot after=1s}\fill[:fill opacity={0s="1", 1s="0"}] (0,0) rectangle (1,1);"#);
        assert!(near(paths(&c)[0].fill.as_ref().unwrap().opacity, 1.0));
    }

    /// `plot[smooth cycle]`: a closed curve.
    #[test]
    fn smooth_cycle() {
        let c = quiet(r"\draw plot[smooth cycle] coordinates {(0,0) (1,1) (2,0)};");
        let s = &paths(&c)[0].segments;
        assert!(matches!(s.last(), Some(Segment::Close)) && s.iter().any(|x| matches!(x, Segment::CurveTo(..))), "{s:?}");
    }

    /// `overlay=false` grows the picture; `overlay` does not.
    #[test]
    fn overlay() {
        let c = quiet(r"\draw[overlay=false] (0,0)--(1,0); \draw[overlay] (0,0)--(5,0);");
        assert!(near(c.max.unwrap().x - c.min.unwrap().x, 1.0 + 2.0 * HALF), "{:?}", c.max);
    }

    /// `grow` by a compass direction or an angle.
    #[test]
    fn grow() {
        for (grow, angle) in [("north east", 45.0f64), ("30", 30.0), ("north west", 135.0), ("south east", -45.0), ("south west", -135.0)] {
            let c = quiet(&format!(r"\begin{{scope}}[grow={grow}]\node[inner sep=0pt] (r) {{}} child {{node[inner sep=0pt] {{}}}};\end{{scope}}"));
            let p = points(&paths(&c)[0].segments);
            let a = (p[1].1 - p[0].1).atan2(p[1].0 - p[0].0).to_degrees();
            assert!((a - angle).abs() < 1e-6, "{grow}: {a}");
        }
    }

    /// The corner colors of a bilinear shading; an unknown one is said.
    #[test]
    fn corner_colors() {
        let (c, w) = picture(r"\shade[upper left=red, upper right=nocolor, lower left=blue, lower right=green] (0,0) rectangle (1,1);");
        assert_eq!(w, ["upper right: unknown color 'nocolor'"]);
        assert!(paths(&c).len() > 100);
    }
}
