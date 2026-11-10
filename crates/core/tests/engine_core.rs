//! The statement reader at the heart of the engine: the text set around the pictures (formulas,
//! spaces, `\emph`, `\\`, `tabular`, glue), the commands it dispatches to the basic layer and the
//! libraries, groups and scopes, pgfmath as commands, `\tikzstyle`, `\scoped`, colors, packages,
//! environments, `\foreach` and the page the pictures are laid out on.

mod common;
use common::*;
use tikra::canvas::{Canvas, Item, Paint};
use tikra::engine::run_document;
use tikra::scene::{Point, Segment};

/// The canvas and warnings of a whole document.
fn document(code: &str) -> (Canvas, Vec<String>) {
    run_document(code).unwrap_or_else(|e| panic!("{e}\n{code}"))
}

/// The canvas of a whole document, which must not warn.
fn quiet(code: &str) -> Canvas {
    let (c, w) = document(code);
    assert!(w.is_empty(), "warnings: {w:?}\n{code}");
    c
}

/// Where each path of a canvas starts.
fn starts(c: &Canvas) -> Vec<Point> {
    paths(c)
        .iter()
        .map(|d| match d.segments[0] {
            Segment::MoveTo(p) => p,
            ref s => panic!("a path that starts with {s:?}"),
        })
        .collect()
}

/// Where each path of a canvas ends.
fn ends(c: &Canvas) -> Vec<Point> {
    paths(c).iter().map(|d| *points(&d.segments).last().map(|(x, y)| Point::new(*x, *y)).as_ref().unwrap()).collect()
}

/// The widths of the texts of a canvas, and where each starts, in cm.
fn words(c: &Canvas) -> Vec<(f64, Point)> {
    texts(c).iter().map(|(l, t)| (l.width(), Point::new(t.e, t.f))).collect()
}

fn near(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() < tolerance
}

/// One picture, a line 1 cm long: it is 1 cm and a line width wide.
const PIC: &str = r"\tikz\draw (0,0)--(1,0);";
/// TeX's interword space of the 10 pt font, in cm.
const SPACE: f64 = 3.33333 * 2.54 / 72.27;

mod text_around_the_pictures {
    use super::*;

    /// `$…$` between pictures is a formula set as a word, joined to the word it touches.
    #[test]
    fn a_formula_is_a_word() {
        let alone = quiet(&format!("{PIC} $a=b$"));
        let joined = quiet(&format!("{PIC} w$a=b$"));
        let (a, j) = (words(&alone), words(&joined));
        assert_eq!((a.len(), j.len()), (1, 1));
        assert!(j[0].0 > a[0].0 + 0.1, "{a:?} {j:?}");
        // one space after the picture
        assert!(near(a[0].1.x, 1.0 + 2.0 * 0.0070292 + SPACE, 1e-3), "{a:?}");
    }

    /// A formula with a picture inside is read as it comes: the picture is drawn, no word set.
    #[test]
    fn a_formula_with_a_picture_inside() {
        let c = quiet(&format!("{PIC} ${PIC}$"));
        assert_eq!(paths(&c).len(), 2);
        assert!(texts(&c).is_empty());
    }

    /// `~` and `\ ` are spaces between pictures; the spaces after `\ ` are skipped, as TeX does.
    #[test]
    fn tie_and_control_space() {
        let second = |code: String| starts(&quiet(&code))[1].x;
        let plain = second(format!("{PIC} {PIC}"));
        assert!(near(second(format!("{PIC} ~ {PIC}")) - plain, 2.0 * SPACE, 1e-3));
        assert!(near(second(format!(r"{PIC} \ {PIC}")) - plain, SPACE, 1e-3));
    }

    /// A character `\let` to a letter is a letter of the text.
    #[test]
    fn a_letter_by_another_name() {
        let w = words(&quiet(&format!(r"\let\x=a {PIC} \x")));
        assert_eq!(w.len(), 1);
        let a = words(&quiet(&format!("{PIC} a")));
        assert!(near(w[0].0, a[0].0, 1e-9));
    }

    /// `\emph{…}` is set inside the word it is in.
    #[test]
    fn emph_inside_a_word() {
        assert_eq!(words(&quiet(&format!(r"{PIC} x\emph{{k}}y"))).len(), 1);
        let w = words(&quiet(&format!(r"{PIC} \emph{{k}}y")));
        assert_eq!(w.len(), 1);
        assert!(w[0].0 > 0.3, "{w:?}");
    }

    /// `\\` (with or without a space asked for) breaks the line: the next picture is below.
    #[test]
    fn a_line_break() {
        for brk in [r"\\[2pt]", r"\\"] {
            let s = starts(&quiet(&format!("{PIC} a{brk} {PIC}")));
            assert!(near(s[1].x, s[0].x, 1e-9) && s[1].y < s[0].y - 0.3, "{brk}: {s:?}");
        }
    }

    /// The cells of a `tabular` are apart by `\tabcolsep` twice, the first one's in from the left.
    #[test]
    fn tabular_cells() {
        let s = starts(&quiet(&format!(r"\begin{{tabular}}{{cc}}{PIC} & \tikz\draw (0,0)--(2,0);\end{{tabular}}")));
        let tabcolsep = 6.0 * 2.54 / 72.27;
        assert!(near(s[0].x, tabcolsep + 0.0070292, 1e-3), "{s:?}");
        assert!(near(s[1].x - s[0].x, 1.0 + 2.0 * 0.0070292 + 2.0 * tabcolsep, 1e-3), "{s:?}");
    }

    /// `\hspace`, `\hskip` (its stretch and shrink not used) and `\qquad` are glue of their width.
    #[test]
    fn glue_between_pictures() {
        let second = |code: String| starts(&quiet(&code))[1].x;
        let plain = second(format!("{PIC} {PIC}"));
        assert!(near(second(format!(r"{PIC} \hspace*{{1cm}}{PIC}")) - plain, 1.0, 1e-6));
        assert!(near(second(format!(r"{PIC} \hspace{{1cm}}{PIC}")) - plain, 1.0, 1e-6));
        assert!(near(second(format!(r"{PIC} \hskip 1cm plus 2fill minus 1pt{PIC}")) - plain, 1.0, 1e-4));
        assert!(near(second(format!(r"{PIC} \hskip 1cm plus 1fil{PIC}")) - plain, 1.0, 1e-4));
        assert!(near(second(format!(r"{PIC}\qquad{PIC}")) - plain, 20.0 * 2.54 / 72.27 - SPACE, 1e-4));
        assert!(near(second(format!(r"{PIC}\quad{PIC}")) - plain, 10.0 * 2.54 / 72.27 - SPACE, 1e-4));
    }

    /// Glue, boxes and page commands inside a picture do not move what it draws.
    #[test]
    fn glue_and_boxes_inside_a_picture() {
        let c = quiet(
            r"\begin{tikzpicture}\draw (0,0) -- (1,0); \hspace{1cm} \hskip 1cm plus 1fil \vskip 2pt \quad
              \scalebox{2}{\draw (0,0) -- (1,1);} \resizebox{1cm}{!}{\draw (0,0) -- (3,1);} \fbox{\draw (0,0)--(1,2);}\end{tikzpicture}",
        );
        assert_eq!(paths(&c).len(), 4);
        assert!(close(c.max.unwrap().x, 3.0 + 0.0140584), "{:?}", c.max);
    }

    /// `\fbox` frames the picture `\fboxsep` away, with a line `\fboxrule` wide.
    #[test]
    fn fbox_frames_a_picture() {
        let c = quiet(&format!(r"\fbox{{{PIC}}}"));
        let p = paths(&c);
        assert_eq!(p.len(), 2);
        let frame = &p[1];
        let rule = 0.4 * 2.54 / 72.27;
        assert!(near(frame.stroke.as_ref().unwrap().width, rule, 1e-6));
        let (lo, hi) = bounds(&frame.segments);
        // 3 pt of \fboxsep on each side of the picture's 1 cm and a line width
        assert!(near(hi.x - lo.x, 1.0140584 + 2.0 * 3.0 * 2.54 / 72.27 + rule, 1e-3), "{lo:?} {hi:?}");
        assert!(matches!(frame.segments.last(), Some(Segment::Close)));
    }

    /// `\resizebox` to a size it cannot read (`\width`) says so and leaves the picture as it is.
    #[test]
    fn resizebox_to_a_size_it_does_not_know() {
        let (c, w) = document(&format!(r"\resizebox{{\width}}{{!}}{{{PIC}}}"));
        assert_eq!(w, ["\\resizebox: size '\\width' not understood: not resized to it"]);
        assert!(close(c.max.unwrap().x, 1.0140584));
    }
}

mod groups_and_scopes {
    use super::*;

    /// `local bounding box=b`: a node `b` over what the scope drew, line width included.
    #[test]
    fn local_bounding_box() {
        // a mark at (-3,-3) with no width, where the page starts
        let c = quiet(
            r"\begin{tikzpicture}\draw[line width=0pt] (-3,-3) -- (-3,-3.1);\begin{scope}[local bounding box=b] \draw[line width=4pt] (0,0)--(1,1); \end{scope}
              \draw[line width=0pt] (b.south west) -- (b.north east);\end{tikzpicture}",
        );
        let half = 2.0 * 2.54 / 72.27;
        let (sw, ne) = (starts(&c)[2], ends(&c)[2]);
        assert!(close_to((sw.x - 3.0, sw.y - 3.1), (-half, -half)), "{sw:?}");
        assert!(close_to((ne.x - 3.0, ne.y - 3.1), (1.0 + half, 1.0 + half)), "{ne:?}");
    }

    /// `\endpgfscope` with no scope open is an extra `}`.
    #[test]
    fn an_extra_end_of_scope() {
        let (c, w) = canvas_and_warnings(r"\draw (0,0)--(1,1); \endpgfscope \draw (0,0) -- (2,0);");
        assert_eq!(w, ["extra '}'"]);
        assert_eq!(paths(&c).len(), 2);
    }

    /// A fading set in a scope that has one replaces it, as pgf's graphic state holds one.
    #[test]
    fn a_fading_replaces_the_scope_fading() {
        let c = quiet(r"\begin{tikzpicture}\begin{scope}[scope fading=south] \pgfsetfading{north}{} \fill (0,0) rectangle (1,1); \end{scope}\end{tikzpicture}");
        let kinds: Vec<&str> = c
            .items
            .iter()
            .map(|i| match i {
                Item::Fade { .. } => "fade",
                Item::End => "end",
                Item::Path { .. } => "path",
                _ => "other",
            })
            .collect();
        assert_eq!(kinds, ["fade", "end", "fade", "path", "end"]);
    }

    /// `\scoped` and `\node` at the very end of the input: nothing more to read.
    #[test]
    fn commands_at_the_end_of_the_input() {
        let c = quiet(r"\begin{tikzpicture}\draw (0,0)--(1,0);\end{tikzpicture}\scoped");
        assert_eq!(paths(&c).len(), 1);
        let (c, w) = document(r"\begin{tikzpicture}\draw (0,0)--(1,0);\end{tikzpicture}\node");
        assert_eq!(paths(&c).len(), 1);
        assert!(w.iter().any(|w| w.contains("node without text")), "{w:?}");
    }
}

mod pgfmath_in_the_code {
    use super::*;

    /// `\pgf@sys@tonumber\reg` is the register without its unit; `2\pgf@xx` a product with the x
    /// vector's length; a count register before a unit is that many of it.
    #[test]
    fn registers_in_expressions() {
        let c = quiet(
            r"\makeatletter\begin{tikzpicture}\newdimen\d \d=2pt \pgfmathsetmacro\v{\pgf@sys@tonumber\d} \draw (0,0) -- (\v,0);
              \pgfmathsetmacro\w{2\pgf@xx} \draw (0,1) -- (\w pt,1);
              \newcount\c \c=3 \pgfmathsetlengthmacro\u{\c pt} \draw (0,2) -- (\u,2);\end{tikzpicture}",
        );
        let e = ends(&c);
        // 2 (a number: cm), 2 cm, 3 pt
        assert!(near(e[0].x, 2.0070292, 1e-6), "{e:?}");
        assert!(near(e[1].x, 2.0070292, 1e-5), "{e:?}");
        assert!(near(e[2].x, 0.0070292 + 3.0 * 2.54 / 72.27, 1e-5), "{e:?}");
    }

    /// `\tikzmath` assigns variables the code uses after it.
    #[test]
    fn tikzmath() {
        let c = quiet(r"\begin{tikzpicture}\tikzmath{\a = 3;} \draw (0,0) -- (\a,0);\end{tikzpicture}");
        assert!(near(ends(&c)[0].x, 3.0070292, 1e-6));
    }

    /// `\pgfmathsincos@{90}`: the cosine and sine in `\pgfmathresultx` and `\pgfmathresulty`.
    #[test]
    fn sincos() {
        let c = quiet(r"\makeatletter\begin{tikzpicture}\pgfmathsincos@{90} \draw (0,0) -- (\pgfmathresultx,\pgfmathresulty);\end{tikzpicture}");
        let e = ends(&c)[0];
        assert!(close_to((e.x, e.y), (0.0070292, 1.0070292)), "{e:?}");
    }
}

mod colors {
    use super::*;

    /// `.` in a color expression is the current color.
    #[test]
    fn the_current_color_is_a_dot() {
        let c = quiet(r"\begin{tikzpicture}\draw[red, draw=.!50!white] (0,0)--(1,0);\end{tikzpicture}");
        let s = paths(&c)[0].stroke.clone().unwrap();
        assert!(close(s.color.r, 1.0) && close(s.color.g, 0.5) && close(s.color.b, 0.5), "{:?}", s.color);
    }
}

mod animate_package {
    use super::*;

    /// `animateinline`: only the first frame is drawn; statements outside a picture in the frames
    /// after it are not drawn either, and `\newframe*[rate]` is a frame break too.
    #[test]
    fn only_the_first_frame() {
        let c = quiet(&format!(r"\begin{{animateinline}}{{2}}{PIC}\newframe\draw (0,0)--(2,0); \newframe*[4] \fill (0,0) circle (1pt);\end{{animateinline}}"));
        assert_eq!(paths(&c).len(), 1);
        assert!(texts(&c).is_empty());
    }
}

mod basic_layer_commands {
    use super::*;
    use tikra::canvas::Shading;

    /// `\node[matrix]` is a `\matrix`: its cells are laid out in a row.
    #[test]
    fn a_node_that_is_a_matrix() {
        let c = quiet(r"\begin{tikzpicture}\node[matrix, draw] {\node {a}; & \node {b};\\};\end{tikzpicture}");
        let w = words(&c);
        assert_eq!(w.len(), 2);
        assert!(w[1].1.x > w[0].1.x + w[0].0, "{w:?}");
        assert_eq!(paths(&c).len(), 1);
    }

    /// `\pgfmatrix`: the basic layer's matrix, its cells laid out as `\matrix` lays them out.
    #[test]
    fn pgfmatrix() {
        let c = quiet(r"\begin{tikzpicture}\pgfmatrix{rectangle}{center}{mm}{\pgfusepath{}}{\pgfpointorigin}{}{\node {a}; \pgfmatrixnextcell \node {b}; \\}\end{tikzpicture}");
        let w = words(&c);
        assert_eq!(w.len(), 2);
        assert!(w[1].1.x > w[0].1.x + w[0].0, "{w:?}");
    }

    /// `\pgfdeclarefading{name}{picture}`: the picture is the fading a path then uses.
    #[test]
    fn a_fading_from_a_picture() {
        let c = quiet(r"\begin{tikzpicture}\pgfdeclarefading{f}{\tikz\fill[white] (0,0) rectangle (1,1);} \fill[path fading=f] (0,0) rectangle (2,2);\end{tikzpicture}");
        assert!(matches!(c.items[0], Item::FadePicture { .. }), "{:?}", c.items[0]);
    }

    /// `\pgfsetfading` in a scope, and `\pgfsetfadingforcurrentpath` (stroked or not) on a path:
    /// what follows is faded, the fading moved by the code.
    #[test]
    fn fadings_set_by_the_basic_layer() {
        let c = quiet(
            r"\begin{tikzpicture}\begin{scope}\pgfsetfading{south}{\pgftransformshift{\pgfpoint{1cm}{0cm}}} \fill (0,0) rectangle (2,2);\end{scope}
              \pgfpathrectangle{\pgfpointorigin}{\pgfpoint{1cm}{1cm}} \pgfsetfadingforcurrentpath{north}{} \pgfusepath{fill}
              \pgfpathrectangle{\pgfpointorigin}{\pgfpoint{1cm}{1cm}} \pgfsetfadingforcurrentpathstroked{north}{} \pgfusepath{stroke}\end{tikzpicture}",
        );
        let fades: Vec<f64> = c
            .items
            .iter()
            .filter_map(|i| match i {
                Item::Fade { mask: Shading::Axial { transform, .. } } => Some(transform.e),
                _ => None,
            })
            .collect();
        assert_eq!(fades.len(), 3);
        assert!(near(fades[0] - fades[1], 1.0, 1e-9), "{fades:?}");
    }

    /// `\pgfdeclarepatterninherentlycolored`, then `\pgfsetfillpattern`: the fill is the pattern.
    #[test]
    fn a_pattern_of_the_basic_layer() {
        let c = quiet(
            r"\begin{tikzpicture}\pgfdeclarepatterninherentlycolored{dots}{\pgfpointorigin}{\pgfpoint{4pt}{4pt}}{\pgfpoint{4pt}{4pt}}{\pgfpathcircle{\pgfpoint{2pt}{2pt}}{1pt}\pgfusepath{fill}}
              \pgfsetfillpattern{dots}{red} \pgfpathrectangle{\pgfpointorigin}{\pgfpoint{1cm}{1cm}}\pgfusepath{fill}\end{tikzpicture}",
        );
        assert!(matches!(paths(&c)[0].fill.as_ref().unwrap().paint, Paint::Tile { .. }), "{:?}", paths(&c)[0].fill);
    }

    /// `\pgfdeclarepattern{name=…, …}`: a pattern TikZ's `pattern=` then fills with.
    #[test]
    fn a_pattern_declared_with_keys() {
        let c = quiet(
            r"\begin{tikzpicture}\pgfdeclarepattern{name=hat, bottom left={\pgfpoint{0pt}{0pt}}, top right={\pgfpoint{4pt}{4pt}}, tile size={\pgfpoint{4pt}{4pt}},
              code={\pgfpathmoveto{\pgfpointorigin}\pgfpathlineto{\pgfpoint{4pt}{4pt}}\pgfusepath{stroke}}} \fill[pattern=hat] (0,0) rectangle (1,1);\end{tikzpicture}",
        );
        assert!(matches!(paths(&c)[0].fill.as_ref().unwrap().paint, Paint::Tile { .. }), "{:?}", paths(&c)[0].fill);
    }

    /// `\pgfarrowsdeclare`: a tip of the old kind, drawn at the end of the line, which is shortened.
    #[test]
    fn an_arrow_of_the_old_kind() {
        let c = quiet(
            r"\begin{tikzpicture}\pgfarrowsdeclare{my}{my}{\pgfarrowsleftextend{-2pt}\pgfarrowsrightextend{1pt}}{\pgfpathmoveto{\pgfpoint{-2pt}{2pt}}\pgfpathlineto{\pgfpoint{1pt}{0pt}}\pgfpathlineto{\pgfpoint{-2pt}{-2pt}}\pgfusepathqstroke}
              \draw[-my] (0,0) -- (1,0);\end{tikzpicture}",
        );
        let p = paths(&c);
        assert_eq!(p.len(), 2);
        assert!(ends(&c)[0].x < 1.0, "{:?}", ends(&c));
        assert!(near(points(&p[1].segments)[1].0, 1.0070292, 1e-6));
    }

    /// `\pgfpositionnodelater` keeps the node of the group; `\pgfpositionnodenow` puts it there.
    #[test]
    fn a_node_positioned_later() {
        let c =
            quiet(r"\begin{tikzpicture}{\pgfpositionnodelater{\saved}\node[draw] (k) {k};}\pgfpositionnodenow{\pgfpoint{1cm}{2cm}} \draw (k.center) -- (0,0);\end{tikzpicture}");
        let s = starts(&c);
        assert_eq!(s.len(), 2);
        assert!(close_to((s[1].x, s[1].y), (1.0070292, 2.0070292)), "{s:?}");
    }

    /// `\pgfuseshading` draws a shading's box at the origin; `\pgfshadepath` fills the path with it.
    #[test]
    fn shadings_of_the_basic_layer() {
        let c = quiet(
            r"\begin{tikzpicture}\pgfdeclarehorizontalshading{h}{1cm}{color(0cm)=(red); color(2cm)=(blue)} \pgfuseshading{h}
              \pgfpathrectangle{\pgfpoint{1cm}{0cm}}{\pgfpoint{1cm}{1cm}} \pgfshadepath{h}{45} \pgfusepath{}\end{tikzpicture}",
        );
        let p = paths(&c);
        assert_eq!(p.len(), 2);
        assert!(p.iter().all(|d| matches!(d.fill.as_ref().map(|f| &f.paint), Some(Paint::Shading(_)))), "{p:?}");
    }

    /// A nonlinear transformation of the basic layer moves the points it is given.
    #[test]
    fn a_nonlinear_transformation() {
        let c = quiet(
            r"\makeatletter\begin{tikzpicture}\draw (0,0) -- (0,1);\begin{scope}\pgftransformnonlinear{\pgf@x=2\pgf@x} \pgfapproximatenonlineartransformation \pgfapproximatenonlineartranslation \draw (0,0)--(1,1);\end{scope}\end{tikzpicture}",
        );
        let e = ends(&c);
        assert!(near(e[1].x - e[0].x, 2.0, 1e-3), "{e:?}");
    }

    /// The curvilinear points of a Bézier curve, into `\pgf@x` and `\pgf@y`.
    #[test]
    fn curvilinear_points() {
        let c = quiet(
            r"\makeatletter\begin{tikzpicture}\pgfsetcurvilinearbeziercurve{\pgfpoint{0cm}{0cm}}{\pgfpoint{1cm}{0cm}}{\pgfpoint{2cm}{0cm}}{\pgfpoint{3cm}{0cm}}
              \pgfpointcurvilinearbezierorthogonal{1cm}{1cm} \draw (0,0) -- (\the\pgf@x,\the\pgf@y);
              \pgfpointcurvilinearbezierpolar{1cm}{0cm} \draw (0,0) -- (\the\pgf@x,\the\pgf@y);\end{tikzpicture}",
        );
        let e = ends(&c);
        assert!(near(e[0].x, 1.0070292, 1e-5) && near(e[0].y, 1.0070292, 1e-5), "{e:?}");
        assert!(near(e[1].x, 1.0070292, 1e-5) && near(e[1].y, 0.0070292, 1e-6), "{e:?}");
    }
}

mod keys_and_styles {
    use super::*;

    /// `\scoped[options]` applies to the next statement only; with animated options (`:rotate=…`)
    /// the statement is still drawn, as at the animation's start.
    #[test]
    fn scoped() {
        let c = quiet(
            r"\begin{tikzpicture}\scoped[red] \draw (0,0)--(1,0); \draw (0,1) -- (1,1); \scoped :rotate={0s=0, 2s=90} :opacity={0s=1, 1s=0} \draw (0,2) -- (1,2);\end{tikzpicture}",
        );
        let p = paths(&c);
        assert_eq!(p.len(), 3);
        let red = |d: &Drawn| d.stroke.as_ref().unwrap().color.r > 0.5;
        assert!(red(&p[0]) && !red(&p[1]) && !red(&p[2]));
        assert!(near(ends(&c)[2].y, 2.0070292, 1e-6));
    }

    /// `\pgfset` sets keys from `/pgf`; a full path reaches TikZ's.
    #[test]
    fn pgfset() {
        let c = quiet(r"\begin{tikzpicture}\pgfset{/tikz/line width=2pt} \draw (0,0)--(1,0);\end{tikzpicture}");
        assert!(near(paths(&c)[0].stroke.as_ref().unwrap().width, 2.0 * 2.54 / 72.27, 1e-9));
    }

    /// `\tikzstyle{a}=[…]` defines, `+=[…]` appends; the `=` may be left out.
    #[test]
    fn tikzstyle() {
        let c = quiet(
            r"\begin{tikzpicture}\tikzstyle{a}=[red] \tikzstyle{a}+=[ultra thick] \tikzstyle{b}=[blue] \tikzstyle{b}+[dashed] \tikzstyle{c}[green]
              \draw[a] (0,0)--(1,0); \draw[b] (0,1) -- (1,1); \draw[c] (0,2)--(1,2);\end{tikzpicture}",
        );
        let p = paths(&c);
        let s: Vec<_> = p.iter().map(|d| d.stroke.clone().unwrap()).collect();
        assert!(close(s[0].color.r, 1.0) && near(s[0].width, 1.6 * 2.54 / 72.27, 1e-9), "{:?}", s[0]);
        assert!(close(s[1].color.b, 1.0) && s[1].dash.is_some(), "{:?}", s[1]);
        assert!(close(s[2].color.g, 1.0) && close(s[2].color.r, 0.0), "{:?}", s[2]);
    }
}

mod colors_and_text_commands {
    use super::*;

    /// `\color{…}` in a picture is the color of what follows; an unknown one changes nothing.
    /// `\pgfqbox` is not drawn, and says so; `\textcolor` outside a node is TeX's text, not drawn.
    #[test]
    fn color_in_a_picture() {
        let (c, w) = document(r"\begin{tikzpicture}\color{red} \draw (0,0)--(1,0); \color{nosuchcolor} \draw (0,1) -- (1,1); \pgfqbox{\mybox} \textcolor{red}{x}\end{tikzpicture}");
        assert_eq!(w, ["\\pgfqbox{\\mybox}: the box is not drawn"]);
        let p = paths(&c);
        assert_eq!(p.len(), 2);
        assert!(p.iter().all(|d| close(d.stroke.as_ref().unwrap().color.r, 1.0) && close(d.stroke.as_ref().unwrap().color.g, 0.0)));
        assert!(texts(&c).is_empty());
    }

    /// A `\color` of the page's text with `varwidth` set: the page takes its whole width.
    #[test]
    fn color_with_varwidth() {
        let c = quiet(&format!(r"\documentclass[varwidth=3cm]{{standalone}}\begin{{document}}\color{{red}} {PIC}\end{{document}}"));
        assert!(close(c.max.unwrap().x, 3.0), "{:?}", c.max);
        // without its width: the pictures on one line, as they are
        let c = quiet(&format!(r"\documentclass[varwidth]{{standalone}}\begin{{document}}{PIC} {PIC} {PIC} {PIC}\end{{document}}"));
        let s = starts(&c);
        assert!(s.iter().all(|p| close(p.y, s[0].y)), "{s:?}");
        assert!(close(c.min.unwrap().x, 0.0) && c.max.unwrap().x > 4.0, "{:?}", c.max);
        // with a width and a border that is no length: the lines are broken at the width all the same
        let c = quiet(&format!(r"\documentclass[varwidth=3cm, border=abc]{{standalone}}\begin{{document}}{PIC} {PIC} {PIC} {PIC}\end{{document}}"));
        let s = starts(&c);
        assert!(s.iter().any(|p| !close(p.y, s[0].y)), "{s:?}");
        assert!(c.max.unwrap().x - c.min.unwrap().x <= 3.0 + 1e-9, "{:?} {:?}", c.min, c.max);
    }

    /// The font size commands make the text smaller or larger, in LaTeX's order, `\normalsize`
    /// between `\small` and `\large`.
    #[test]
    fn font_sizes() {
        let width = |cmd: &str| words(&quiet(&format!(r"\begin{{tikzpicture}}{cmd} \node {{x}};\end{{tikzpicture}}")))[0].0;
        let mut last = 0.0;
        for cmd in ["tiny", "scriptsize", "footnotesize", "small", "normalsize", "large", "Large", "LARGE", "huge", "Huge"] {
            let w = width(&format!("\\{cmd}"));
            assert!(w > last, "{cmd}: {w}");
            last = w;
        }
        assert!(near(width(r"\normalsize"), width(""), 1e-9));
    }
}

mod pgfmath_commands {
    use super::*;

    /// `\pgfmathneg` and `\pgfmathadd` as commands: `\pgfmathresult` is their value; an expression
    /// it cannot read is a warning.
    #[test]
    fn functions_as_commands() {
        let (c, w) = document(
            r"\begin{tikzpicture}\pgfmathadd{1}{2} \draw (0,0) -- (\pgfmathresult,0); \pgfmathsqrt{x} \pgfmathneg{2} \draw (0,1) -- (\pgfmathresult, 1);\end{tikzpicture}",
        );
        assert_eq!(w.len(), 1, "{w:?}");
        assert!(w[0].contains("function without parentheses: x"), "{w:?}");
        let (s, e) = (starts(&c), ends(&c));
        assert!(near(e[0].x - s[0].x, 3.0, 1e-9) && near(e[1].x - s[1].x, -2.0, 1e-9), "{s:?} {e:?}");
    }

    /// `\pgfmathdeclarefunction`: a `\pgfmathparse` body with its parameters (`#1`, `##2`) is a
    /// function; another body is not understood, and said so.
    #[test]
    fn declared_functions() {
        let (c, w) = document(
            r"\begin{tikzpicture}\pgfmathdeclarefunction{f}{1}{\edef\pgfmathresult{#1}} \pgfmathdeclarefunction{g}{2}{\pgfmathparse{#1+##2}} \draw (0,0) -- ({g(1,2)},0);\end{tikzpicture}",
        );
        assert_eq!(w, ["\\pgfmathdeclarefunction{f}: only a \\pgfmathparse body is supported"]);
        assert!(near(ends(&c)[0].x, 3.0070292, 1e-6));
    }

    /// `\pgfmathprintnumber`, in the number format given in its options: a shorter text.
    #[test]
    fn printnumber() {
        let w = words(&quiet(
            r"\begin{tikzpicture}\node {\pgfmathprintnumber{3.14159}}; \node at (0,1) {\pgfmathprintnumber[fixed, precision=1]{3.14159}}; \node at (0,2) {3.1};\end{tikzpicture}",
        ));
        assert!(w[0].0 > w[1].0, "{w:?}");
        assert!(near(w[1].0, w[2].0, 1e-9), "{w:?}");
    }
}

mod packages_and_classes {
    use super::*;

    /// `\usetikzlibrary` and `\documentclass[` cut short at the end of the input: nothing to draw,
    /// and no error.
    #[test]
    fn cut_short_at_the_end() {
        let c = quiet(&format!(r"{PIC}\usetikzlibrary"));
        assert_eq!(paths(&c).len(), 1);
        let c = quiet(r"\documentclass[a4paper");
        assert!(c.items.is_empty() && c.min.is_none());
    }

    /// Beamer's templates and colors are the theme's: ignored, whatever form they take.
    #[test]
    fn beamer_templates() {
        let c = quiet(&format!(
            r"\setbeamertemplate{{navigation symbols}}{{}} \setbeamertemplate{{footline}}[frame number] \setbeamertemplate{{x}}[a][b] \setbeamercolor{{a}}{{fg=red}}{PIC}"
        ));
        assert_eq!(paths(&c).len(), 1);
        assert!(texts(&c).is_empty());
    }

    /// `\tikzfading[name=…]` from shading options, `transparent!N` N percent transparent; one without
    /// a name is made and forgotten.
    #[test]
    fn tikzfading() {
        let c = quiet(
            r"\tikzfading[name=fade right, left color=transparent!0, right color=transparent!100]\begin{tikzpicture}\fill[path fading=fade right] (0,0) rectangle (2,1);\tikzfading[left color=red]\end{tikzpicture}",
        );
        let Item::Fade { mask: tikra::canvas::Shading::Axial { stops, .. } } = &c.items[0] else { panic!("{:?}", c.items[0]) };
        // black (transparent) at the left, white (opaque) at the right
        assert!(close(stops.first().unwrap().1.r, 0.0) && close(stops.last().unwrap().1.r, 1.0), "{stops:?}");
    }
}

mod the_bounding_box {
    use super::*;

    /// `\pgfresetboundingbox`: the picture's size is made anew by what follows.
    #[test]
    fn reset() {
        let (w, _) = size(r"\draw (0,0)--(0,3); \pgfresetboundingbox \draw (0,0)--(2,0);");
        assert!(near(w, 2.0140584, 1e-6));
        let c = quiet(r"\begin{tikzpicture}\draw (0,0)--(0,3); \pgfresetboundingbox \draw (0,0)--(2,0);\end{tikzpicture}");
        assert!(near(c.max.unwrap().y - c.min.unwrap().y, 0.0140584, 1e-6), "{:?} {:?}", c.min, c.max);
    }

    /// `pgfinterruptboundingbox`: what is drawn inside does not make the picture larger.
    #[test]
    fn interrupted() {
        let c = quiet(r"\begin{tikzpicture}\draw (0,0)--(1,0);\begin{pgfinterruptboundingbox}\draw (0,0)--(5,5);\end{pgfinterruptboundingbox}\end{tikzpicture}");
        assert_eq!(paths(&c).len(), 2);
        assert!(near(c.max.unwrap().x, 1.0140584, 1e-6), "{:?}", c.max);
    }

    /// `trim left` and `trim right`: the box starts and ends there (a point, a length, or the origin
    /// with no value); a point that does not exist leaves that side as it is.
    #[test]
    fn trimmed() {
        let c = quiet(r"\begin{tikzpicture}[trim left=(a), trim right=1cm]\coordinate (a) at (0.5,0);\draw (0,0)--(2,0);\end{tikzpicture}");
        assert!(near(c.max.unwrap().x - c.min.unwrap().x, 0.5, 1e-9), "{:?} {:?}", c.min, c.max);
        assert!(near(starts(&c)[0].x, -0.5, 1e-9));
        let c = quiet(r"\begin{tikzpicture}[trim left, trim right=(nowhere)]\draw (-1,0)--(2,0);\end{tikzpicture}");
        assert!(near(c.max.unwrap().x - c.min.unwrap().x, 2.0070292, 1e-6), "{:?} {:?}", c.min, c.max);
    }

    /// `baseline` (0pt with no value), `baseline=(point)` and `\pgfsetbaselinepointlater`: that
    /// height of the picture is on the line's baseline, with the next picture.
    #[test]
    fn baselines() {
        for first in [
            r"\tikz[baseline]\draw (0,-1)--(1,1);",
            r"\tikz[baseline=(a)]{\coordinate (a) at (0,1); \draw (0,-1)--(1,1);}",
            r"\tikz{\pgfsetbaselinepointlater{\pgfpoint{0cm}{1cm}}\draw (0,-1)--(1,1);}",
        ] {
            let c = quiet(&format!("{first} {PIC}"));
            let s = starts(&c);
            let baseline = s[1].y - 0.0070292;
            let first_bottom = s[0].y;
            let expected = if first.contains("[baseline]") { -1.0 } else { -2.0 };
            assert!(near(first_bottom - baseline, expected, 1e-6), "{first}: {s:?}");
        }
    }
}

mod environments {
    use super::*;

    /// `\pgfinterruptpath`: the path being built waits; an extra `\endpgfinterruptpath` is ignored.
    #[test]
    fn interrupted_paths() {
        let (c, w) = document(r"\begin{tikzpicture}\draw (0,0) -- (1,0); \pgfinterruptpath \draw (0,1) -- (1,1); \endpgfinterruptpath \endpgfinterruptpath\end{tikzpicture}");
        assert_eq!(w, ["\\endpgfinterruptpath without \\pgfinterruptpath: ignored"]);
        assert_eq!(paths(&c).len(), 2);
        let c = quiet(
            r"\begin{tikzpicture}\pgfpathmoveto{\pgfpointorigin}\begin{pgfinterruptpath}\draw (0,1)--(1,1);\end{pgfinterruptpath}\pgfpathlineto{\pgfpoint{2cm}{0cm}}\pgfusepath{stroke}\end{tikzpicture}",
        );
        let p = paths(&c);
        assert_eq!(p.len(), 2);
        // the path built around it: from the origin to (2,0), after the one drawn inside
        let q = points(&p[1].segments);
        assert_eq!(q.len(), 2);
        assert!(near(q[1].0 - q[0].0, 2.0, 1e-9) && near(q[1].1, q[0].1, 1e-9), "{q:?}");
    }

    /// `pgfscope` keeps what it sets to itself.
    #[test]
    fn pgfscope() {
        let c = quiet(r"\begin{tikzpicture}\begin{pgfscope}\pgfsetlinewidth{3pt}\draw (0,0)--(1,0);\end{pgfscope}\draw (0,1)--(1,1);\end{tikzpicture}");
        let w: Vec<f64> = paths(&c).iter().map(|d| d.stroke.as_ref().unwrap().width).collect();
        assert!(near(w[0], 3.0 * 2.54 / 72.27, 1e-9) && near(w[1], 0.4 * 2.54 / 72.27, 1e-9), "{w:?}");
    }

    /// Layers: drawn in the order of `\pgfsetlayers` (`main` last when it is not listed), those not
    /// listed after them; `\endpgfonlayer` with none open is said, one left open is closed with the
    /// picture.
    #[test]
    fn layers() {
        let c = quiet(
            r"\pgfdeclarelayer{back}\pgfdeclarelayer{front}\pgfsetlayers{back,front}\begin{tikzpicture}\begin{pgfonlayer}{front}\draw (0,0)--(1,0);\end{pgfonlayer}
              \draw (0,1)--(1,1);\pgfonlayer{back}\draw (0,2)--(1,2);\end{tikzpicture}",
        );
        let y: Vec<f64> = starts(&c).iter().map(|p| (p.y - 0.0070292).round()).collect();
        assert_eq!(y, [2.0, 0.0, 1.0]);
        let c = quiet(
            r"\pgfdeclarelayer{back}\pgfdeclarelayer{other}\pgfsetlayers{back,main}\begin{tikzpicture}\begin{pgfonlayer}{other}\draw (0,0)--(1,0);\end{pgfonlayer}
              \draw (0,1)--(1,1);\begin{pgfonlayer}{back}\draw (0,2)--(1,2);\end{pgfonlayer}\end{tikzpicture}",
        );
        let y: Vec<f64> = starts(&c).iter().map(|p| (p.y - 0.0070292).round()).collect();
        assert_eq!(y, [2.0, 1.0, 0.0]);
        let (_, w) = document(r"\begin{tikzpicture}\draw (0,0)--(1,0); \endpgfonlayer\end{tikzpicture}");
        assert_eq!(w, ["\\endpgfonlayer without \\pgfonlayer"]);
    }

    /// The backgrounds library's frame: around what the picture drew, once.
    #[test]
    fn background_frame() {
        let c = quiet(r"\usetikzlibrary{backgrounds}\begin{tikzpicture}[show background rectangle, framed]\draw (0,0)--(1,1);\end{tikzpicture}");
        let p = paths(&c);
        assert_eq!(p.len(), 3);
        let (lo, hi) = bounds(&p[0].segments);
        // 1 cm and a line width, and `inner frame xsep` (1 ex of the 10 pt font) on both sides
        assert!(near(hi.x - lo.x, 1.0140584 + 2.0 * 4.30554 * 2.54 / 72.27, 1e-3), "{lo:?} {hi:?}");
    }

    /// Code at the end of a picture that fails is said, and the picture is kept.
    #[test]
    fn execute_at_end_picture_that_fails() {
        let (c, w) = document(r"\begin{tikzpicture}[execute at end picture={\draw (0,0) -- (}]\draw (0,0)--(1,0);\end{tikzpicture}");
        assert_eq!(paths(&c).len(), 1);
        assert_eq!(w.len(), 1, "{w:?}");
    }

    /// `\tikz \datavisualization`: the visualization reads its data itself.
    #[test]
    fn tikz_datavisualization() {
        let c = quiet("\\usetikzlibrary{datavisualization}\\tikz \\datavisualization [school book axes, visualize as line] data {x, y\n0, 0\n1, 1\n};");
        assert!(paths(&c).len() >= 5);
        assert_eq!(texts(&c).len(), 3);
    }

    /// A picture inside a picture draws in it.
    #[test]
    fn nested_pictures() {
        let c = quiet(r"\begin{tikzpicture}\draw (0,0)--(1,0); \begin{tikzpicture}\draw (0,0)--(2,0);\end{tikzpicture}\end{tikzpicture}");
        let s = starts(&c);
        assert!(s.len() == 2 && near(s[0].x, s[1].x, 1e-9) && near(s[0].y, s[1].y, 1e-9), "{s:?}");
        assert!(near(c.max.unwrap().x, 2.0140584, 1e-6));
    }

    /// `tabular`: columns `l` and `r` (the narrower cell to the right), `\\` at the end of the last
    /// row adds none, glue in a cell is kept.
    #[test]
    fn tabular_columns() {
        let c = quiet(r"\begin{tabular}{lr}\tikz\draw (0,0)--(1,0); & \tikz\draw (0,0)--(2,0);\\ \tikz\draw (0,0)--(3,0); & a b\\\end{tabular}");
        let s = starts(&c);
        let e = ends(&c);
        assert!(near(s[0].x, s[2].x, 1e-9), "{s:?}");
        let w = words(&c);
        assert!(near(w[1].1.x + w[1].0, e[1].x + 0.0070292, 1e-3), "{w:?} {e:?}");
        // two rows
        assert!(s[2].y < s[0].y - 0.3);
        let w = words(&quiet(r"\begin{tabular}{l}a\hspace{1cm}b\\\end{tabular}"));
        assert!(near(w[1].1.x - w[0].1.x - w[0].0, 1.0, 1e-6), "{w:?}");
    }

    /// Beamer's `frame`, its overlay, options, title and subtitle; `figure` and `minipage` (the
    /// page as wide as the minipage); `center` around a picture and in one.
    #[test]
    fn document_environments() {
        for env in [r"\begin{frame}<1>[fragile]{Title}{Sub}", r"\begin{frame}\frametitle{x}"] {
            let c = quiet(&format!(r"{env}{PIC}\end{{frame}}"));
            assert_eq!(paths(&c).len(), 1);
            assert!(texts(&c).is_empty(), "{env}");
        }
        let c = quiet(&format!(r"\begin{{figure}}[h]\begin{{minipage}}{{3cm}}{PIC}\end{{minipage}}\end{{figure}}"));
        assert!(close(c.max.unwrap().x, 3.0), "{:?}", c.max);
        let c = quiet(&format!(r"\begin{{center}}{PIC}\end{{center}}"));
        assert_eq!(paths(&c).len(), 1);
        let c = quiet(r"\begin{tikzpicture}\begin{center}\draw (0,0)--(1,0);\end{center}\end{tikzpicture}");
        assert!(near(c.max.unwrap().x, 1.0140584, 1e-6));
    }

    /// An `\end` without its `\begin` is ignored; environments left open are closed with the one
    /// around them.
    #[test]
    fn unmatched_ends() {
        let (c, w) = document(r"\begin{tikzpicture}\draw (0,0)--(1,0);\end{animateinline}\end{tikzpicture}");
        assert_eq!(w, ["\\end{animateinline} without a matching \\begin: ignored"]);
        assert_eq!(paths(&c).len(), 1);
        let (c, w) = document(r"\begin{tikzpicture}\begin{scope}[red]\draw (0,0)--(1,0);\end{tikzpicture}");
        assert_eq!(w, ["\\begin{scope} without \\end: closed at \\end{tikzpicture}"]);
        assert_eq!(paths(&c).len(), 1);
    }
}

mod animations {
    use super::*;

    /// Frames after the first are not run: pictures in them skipped (those nested, and a `\begin`
    /// that is not an environment's, with them), one never ended said so; `begin` and `end` options
    /// make each frame a picture.
    #[test]
    fn frames_after_the_first() {
        let c = quiet(&format!(
            r"\begin{{animateinline}}{{2}}{PIC}\newframe\begin{{tikzpicture}}\begin{{scope}}\draw (0,0)--(2,0);\end{{scope}}\begin\relax\begin{{tikzpicture}}\end{{tikzpicture}}\end{{tikzpicture}}\end{{animateinline}}"
        ));
        assert_eq!(paths(&c).len(), 1);
        let (c, w) = document(&format!(r"\begin{{animateinline}}{{2}}{PIC}\newframe\begin{{tikzpicture}}\draw (0,0)--(2,0);"));
        assert_eq!(w, ["\\begin{tikzpicture} without \\end: skipped to the end"]);
        assert_eq!(paths(&c).len(), 1);
        let c = quiet(
            r"\begin{animateinline}[begin={\begin{tikzpicture}}, end={\end{tikzpicture}}, poster=first]{2}\draw (0,0)--(1,0);\newframe\draw (0,0)--(2,0);\end{animateinline}",
        );
        assert_eq!(paths(&c).len(), 1);
        assert!(near(c.max.unwrap().x, 1.0140584, 1e-6));
    }
}

mod foreach_statements {
    use super::*;

    /// `\tikz \foreach …`: the loop is the picture's statement, with options before or after its
    /// list, a list in a macro, a braced body; `\tikz \foreach` with nothing after it is said.
    #[test]
    fn foreach_as_the_statement_of_tikz() {
        for code in [
            r"\tikz \foreach \x [count=\i] in {1,2} \draw (\i,0)--(\i,1);",
            r"\tikz \foreach \x in {1,2} [count=\i] \draw (\i,0)--(\i,1);",
            r"\tikz \foreach \x in {1,2} {\draw (\x,0)--(\x,1);}",
            r"\def\l{1,2}\tikz \foreach \x in \l \draw (\x,0)--(\x,1);",
        ] {
            let c = quiet(code);
            let s = starts(&c);
            assert_eq!(s.len(), 2, "{code}");
            assert!(near(s[1].x - s[0].x, 1.0, 1e-9), "{code}: {s:?}");
        }
        let (_, w) = document(r"\tikz \foreach");
        assert_eq!(w, ["\\foreach: figure: \\foreach without 'in'; statement ignored"]);
        let c = quiet(r"\tikz");
        assert!(c.items.is_empty());
    }

    /// `\breakforeach`: the iteration it is in is the last.
    #[test]
    fn breakforeach() {
        let c = quiet(r"\begin{tikzpicture}\foreach \x in {1,2,3,4} {\draw (\x,0)--(\x,1); \ifnum\x=2 \breakforeach\fi}\end{tikzpicture}");
        assert_eq!(paths(&c).len(), 2);
    }

    /// `remember=\x as \last (initially 0)`: the value of the iteration before.
    #[test]
    fn remember() {
        let c = quiet(r"\begin{tikzpicture}\foreach \x [remember=\x as \last (initially 0)] in {1,2,3} \draw (\last,0)--(\x,1);\end{tikzpicture}");
        let (s, e) = (starts(&c), ends(&c));
        assert!(s.iter().zip(&e).all(|(a, b)| near(b.x - a.x, 1.0, 1e-9)), "{s:?} {e:?}");
        assert!(near(s[2].x - s[0].x, 2.0, 1e-9));
    }

    /// `evaluate`: the variable itself, `{\c=…;}` assignments, and `as \s using …` with a string or
    /// `ifthenelse` (nested) choosing one; `parse` and keys it does not know are ignored.
    #[test]
    fn evaluate() {
        let x = |code: &str| starts(&quiet(code)).iter().map(|p| (p.x - starts(&quiet(code))[0].x).round()).collect::<Vec<_>>();
        assert_eq!(x(r"\begin{tikzpicture}\foreach \x [evaluate=\x] in {1+1,2*2} \draw (\x,0)--(\x,1);\end{tikzpicture}"), [0.0, 2.0]);
        assert_eq!(x(r"\begin{tikzpicture}\foreach \x [evaluate={\c=\x*2;}] in {1,2} \draw (\c,0)--(\c,1);\end{tikzpicture}"), [0.0, 2.0]);
        for using in [r#"{ifthenelse(\x>1,"dashed","solid")}"#, r#"{ifthenelse(\x>1,ifthenelse(\x>5,"dotted","dashed"),"solid")}"#] {
            let c = quiet(&format!(r"\begin{{tikzpicture}}\foreach \x [evaluate=\x as \s using {using}] in {{1,2}} \draw[\s] (\x,0)--(\x,1);\end{{tikzpicture}}"));
            let dashes: Vec<bool> = paths(&c).iter().map(|d| d.stroke.as_ref().unwrap().dash.is_some()).collect();
            assert_eq!(dashes, [false, true], "{using}");
        }
        let c = quiet(r#"\begin{tikzpicture}\foreach \x [evaluate=\x as \s using "red", parse=true, other] in {1,2} \draw[\s] (\x,0)--(\x,1);\end{tikzpicture}"#);
        assert!(paths(&c).iter().all(|d| close(d.stroke.as_ref().unwrap().color.r, 1.0)));
    }

    /// Ranges of letters (`a,...,e`, `a,c,...,g`); a range whose ends are not numbers keeps its
    /// ends; one with no step stops.
    #[test]
    fn ranges() {
        let count = |list: &str| texts(&quiet(&format!(r"\begin{{tikzpicture}}\foreach \x in {{{list}}} \node at (0,0) {{\x}};\end{{tikzpicture}}"))).len();
        assert_eq!(count("a,...,e"), 5);
        assert_eq!(count("a,c,...,g"), 4);
        assert_eq!(count("foo,...,3"), 2);
        let c = quiet(r"\begin{tikzpicture}\foreach \x in {1,1,...,3} \draw (\x,0)--(\x,1);\end{tikzpicture}");
        assert_eq!(paths(&c).len(), 2);
    }

    /// Letters between the variables and `in` are not variables (`i` without `n` included); one at
    /// the very end is a `\foreach` with no `in`.
    #[test]
    fn letters_before_in() {
        for code in [r"\tikz \foreach \v i in {1,2} \draw (\v,0)--(\v,1);", r"\begin{tikzpicture}\foreach \v i in {1,2} \draw (\v,0)--(\v,1);\end{tikzpicture}"] {
            let s = starts(&quiet(code));
            assert!(s.len() == 2 && near(s[1].x - s[0].x, 1.0, 1e-9), "{code}: {s:?}");
        }
        for code in [r"\tikz\foreach \x i", r"\begin{tikzpicture}\foreach \x i"] {
            let (_, w) = document(code);
            assert_eq!(w, ["\\foreach: figure: \\foreach without 'in'; statement ignored"], "{code}");
        }
    }

    /// `\foreach` cut short: no list, or nothing after the list.
    #[test]
    fn cut_short() {
        for code in [r"\tikz\foreach \x in", r"\tikz\foreach \x in {1,2}"] {
            let c = quiet(code);
            assert!(c.items.is_empty(), "{code}");
        }
    }
}

mod more_text {
    use super::*;

    /// `\pgfmathprintnumber` in the text: the number as pgf prints it (two decimals), or as its
    /// options say.
    #[test]
    fn printnumber_in_the_text() {
        let w = words(&quiet(&format!(r"{PIC} \pgfmathprintnumber{{3.14159}} \pgfmathprintnumber[fixed,precision=1]{{3.14159}} 3.14 3.1")));
        assert_eq!(w.len(), 4);
        assert!(near(w[0].0, w[2].0, 1e-9) && near(w[1].0, w[3].0, 1e-9), "{w:?}");
    }

    /// `\pgfplotstabletypeset` beside a picture: the table is one box of text in the line.
    #[test]
    fn a_typeset_table_is_a_word() {
        let c = quiet(&format!("\\usepackage{{pgfplotstable}}{PIC} \\pgfplotstabletypeset{{a b\n1 2\n3 4\n}}"));
        let w = words(&c);
        assert_eq!(w.len(), 1, "{w:?}");
        assert!(near(w[0].1.x, 1.0140584 + SPACE, 1e-3) && w[0].0 > 0.3, "{w:?}");
    }

    /// `\pgf@sys@tonumber\reg` in the text: its value in pt, without the unit.
    #[test]
    fn tonumber_in_the_text() {
        let w = words(&quiet(&format!(r"\makeatletter\newdimen\d \d=2.5pt {PIC} \pgf@sys@tonumber\d")));
        let plain = words(&quiet(&format!(r"{PIC} 2.5")));
        assert!(near(w[0].0, plain[0].0, 1e-9), "{w:?} {plain:?}");
    }

    /// `\\` at the very end; `\\` then a paragraph's end leaves an empty line.
    #[test]
    fn breaks_at_the_end() {
        let c = quiet(&format!(r"{PIC} a\\"));
        assert_eq!(words(&c).len(), 1);
        let w = words(&quiet(&format!("{PIC} a\\\\\n\nb")));
        let one = words(&quiet(&format!("{PIC} a\\\\ b")));
        assert!(w[1].1.y < one[1].1.y - 0.3, "{w:?} {one:?}");
    }

    /// A `\color` of the page with `varwidth`: the line is as wide as it, and breaks at the spaces
    /// (glue between pictures included).
    #[test]
    fn lines_broken_at_the_width() {
        let c = quiet(&format!(r"\documentclass[varwidth=3cm]{{standalone}}\begin{{document}}\color{{red}} {PIC}\hspace{{1mm}}{PIC} {PIC}\ {PIC}\end{{document}}"));
        let s = starts(&c);
        assert!(near(s[0].y, s[1].y, 1e-9) && near(s[2].y, s[3].y, 1e-9) && s[2].y < s[0].y - 0.3, "{s:?}");
        assert!(near(s[1].x - s[0].x, 1.0140584 + 0.1, 1e-6) && near(s[2].x, s[0].x, 1e-9), "{s:?}");
    }

    /// A `tabular` after a picture is set beside it; one with a `minipage` takes its width; a
    /// paragraph's end in a cell is a space; column types other than `l`, `c`, `r` are skipped.
    #[test]
    fn more_tables() {
        let s = starts(&quiet(&format!(r"{PIC} \begin{{tabular}}{{c}}{PIC}\end{{tabular}}")));
        assert!(s[1].x > s[0].x + 1.0 && near(s[1].y, s[0].y, 0.01), "{s:?}");
        let c = quiet(&format!(r"\begin{{tabular}}{{c}}\begin{{minipage}}{{2cm}}{PIC}\end{{minipage}}\end{{tabular}}"));
        assert!(close(c.max.unwrap().x, 2.0), "{:?}", c.max);
        let w = words(&quiet("\\begin{tabular}{c}a\n\nb\\end{tabular}"));
        assert_eq!(w.len(), 2);
        assert!(near(w[1].1.y, w[0].1.y, 1e-9));
        let c = quiet(&format!(r"\begin{{tabular}}{{|c|p{{1cm}}}}{PIC} & a\end{{tabular}}"));
        assert_eq!(words(&c).len(), 1);
        assert!(words(&c)[0].1.x > starts(&c)[0].x + 1.0);
    }

    /// `\pattern` is a path whose fill is a pattern.
    #[test]
    fn pattern_command() {
        let c = quiet(r"\begin{tikzpicture}\pattern[pattern=north east lines] (0,0) rectangle (1,1);\end{tikzpicture}");
        let p = paths(&c);
        assert!(p[0].stroke.is_none() && matches!(p[0].fill.as_ref().unwrap().paint, Paint::Pattern { .. }), "{:?}", p[0].fill);
    }

    /// A `\chardef` number in a coordinate.
    #[test]
    fn chardef_in_a_coordinate() {
        let c = quiet(r"\begin{tikzpicture}\chardef\n=2 \draw (0,0) -- (\n,0);\end{tikzpicture}");
        assert!(near(ends(&c)[0].x, 2.0070292, 1e-6));
    }

    /// `\arrowreversed` in a marking, and `\pgfdecoratecurrentpath`.
    #[test]
    fn decoration_commands() {
        let c = quiet(
            r"\usetikzlibrary{decorations.markings}\begin{tikzpicture}\draw[decoration={markings, mark=at position 0.5 with {\arrowreversed{>}}}, postaction=decorate] (0,0)--(2,0);\end{tikzpicture}",
        );
        let p = paths(&c);
        assert_eq!(p.len(), 2);
        // the tip points back, toward the start: its tip is left of its ends
        let tip = points(&p[1].segments);
        assert!(tip.iter().any(|q| q.0 < tip[0].0 - 0.05), "{tip:?}");
        let c = quiet(
            r"\usetikzlibrary{decorations.pathmorphing}\begin{tikzpicture}\pgfpathmoveto{\pgfpointorigin}\pgfpathlineto{\pgfpoint{2cm}{0cm}}\pgfdecoratecurrentpath{zigzag}\pgfusepath{stroke}\end{tikzpicture}",
        );
        assert!(paths(&c)[0].segments.len() > 5);
    }

    /// Commands cut short at the end of the input.
    #[test]
    fn cut_short_commands() {
        for tail in [r"\tikzstyle{a}", r"\tikzstyle{a}+", r"\setbeamertemplate{x}"] {
            let c = quiet(&format!(r"{PIC}{tail}"));
            assert_eq!(paths(&c).len(), 1, "{tail}");
        }
        let c = quiet(&format!(r"\begin{{animateinline}}{{2}}{PIC}\newframe"));
        assert_eq!(paths(&c).len(), 1);
        let (_, w) = document(&format!(r"{PIC}\hspace"));
        assert_eq!(w, ["\\hspace: figure: a command is missing its argument; statement ignored"]);
        assert!(quiet(r"\begin{frame}").items.is_empty());
    }

    /// `\documentclass[border]` with no length; `trim left` on an empty picture; a `minipage` in a
    /// picture does not change the page.
    #[test]
    fn options_with_nothing_to_do() {
        let c = quiet(&format!(r"\documentclass[border]{{standalone}}\begin{{document}}{PIC}\end{{document}}"));
        let plain = quiet(&format!(r"\documentclass{{standalone}}\begin{{document}}{PIC}\end{{document}}"));
        assert!(close(c.margin, plain.margin), "{} {}", c.margin, plain.margin);
        let c = quiet(&format!(r"\begin{{tikzpicture}}[trim left=1cm]\end{{tikzpicture}}{PIC}"));
        assert_eq!(paths(&c).len(), 1);
        let c = quiet(r"\begin{tikzpicture}\begin{minipage}{2cm}\draw (0,0)--(1,0);\end{minipage}\end{tikzpicture}");
        assert!(near(c.max.unwrap().x, 1.0140584, 1e-6));
    }

    /// `\pgfmathsign` and `\pgfmathMod`; an expression they cannot read is said.
    #[test]
    fn native_pgfmath_functions() {
        let (c, w) = document(
            r"\begin{tikzpicture}\pgfmathsign{-3} \draw (0,0) -- (\pgfmathresult,0); \pgfmathMod{-1}{3} \draw (0,1) -- (\pgfmathresult,1); \pgfmathsign{x}\end{tikzpicture}",
        );
        assert_eq!(w.len(), 1, "{w:?}");
        assert!(w[0].contains("function without parentheses: x"), "{w:?}");
        let (s, e) = (starts(&c), ends(&c));
        assert!(near(e[0].x - s[0].x, -1.0, 1e-9) && near(e[1].x - s[1].x, 2.0, 1e-9), "{s:?} {e:?}");
    }
}

mod odds_and_ends {
    use super::*;

    /// A key's code used as a command does nothing outside a key list.
    #[test]
    fn a_key_used_as_a_command() {
        let c = quiet(r"\begin{tikzpicture}\expandafter\let\expandafter\k\csname pgfk@/tikz/xshift/.@cmd\endcsname \k \draw (0,0)--(1,0);\end{tikzpicture}");
        assert!(near(starts(&c)[0].x, 0.0070292, 1e-6));
    }

    /// Transformations of the canvas outside any picture: the pictures are drawn as they are.
    #[test]
    fn canvas_transformations_outside_pictures() {
        for before in [
            r"\tikzset{transform canvas={shift={(1,0)}}}",
            r"\pgflowlevel{\pgftransformshift{\pgfpoint{1cm}{0cm}}}",
            r"\pgflowlevelsynccm",
            r"\pgfsetfading{north}{}",
            r"\begin{pgfviewboxscope}{\pgfpointorigin}{\pgfpoint{1cm}{1cm}}{\pgfpointorigin}{\pgfpoint{2cm}{2cm}}{fill}\end{pgfviewboxscope}",
        ] {
            let c = quiet(&format!("{before}{PIC}"));
            assert_eq!(paths(&c).len(), 1, "{before}");
            assert!(near(c.max.unwrap().x, 1.0140584, 1e-6), "{before}: {:?}", c.max);
        }
    }

    /// The folding library's pics: a net of six squares for a cube.
    #[test]
    fn folding() {
        let c = quiet(r"\usetikzlibrary{folding}\begin{tikzpicture}\pic {cube folding};\end{tikzpicture}");
        let (lo, hi) = (c.min.unwrap(), c.max.unwrap());
        // a net 3 faces wide and 4 high, of 2 cm faces, with its tabs
        assert!(near(hi.y - lo.y, 8.0 + 0.4 + 0.0140584, 1e-6) && hi.x - lo.x > 6.0, "{lo:?} {hi:?}");
        assert!(paths(&c).len() > 20);
    }

    /// `\scoped :rotate = {…}`: in a snapshot, the scope turned as at that time, canvas and all.
    #[test]
    fn scoped_animation_in_a_snapshot() {
        let c = quiet(
            r#"\usetikzlibrary{animations}\begin{tikzpicture}[make snapshot of=1s]\scoped :rotate = {0s="0", 2s="90"} :opacity = {0s="1", 2s="0"} \draw (0,0) -- (1,0);\end{tikzpicture}"#,
        );
        let Some(Item::Transform { transform }) = c.items.first() else { panic!("{:?}", c.items) };
        assert!(near(transform.a, 0.5f64.sqrt(), 1e-9) && near(transform.b, 0.5f64.sqrt(), 1e-9), "{transform:?}");
    }

    /// `center` in a `minipage`: the line is centered in its width, the space between the pictures
    /// counted.
    #[test]
    fn centered_in_a_minipage() {
        let c = quiet(&format!(r"\begin{{minipage}}{{5cm}}\begin{{center}}{PIC} {PIC}\end{{center}}\end{{minipage}}"));
        let s = starts(&c);
        let natural = 2.0 * 1.0140584 + SPACE;
        assert!(near(s[0].x - 0.0070292, (5.0 - natural) / 2.0, 1e-4), "{s:?}");
    }

    /// Glue at the start of the paragraph stays, and a space after it.
    #[test]
    fn glue_at_the_start() {
        let c = quiet(&format!(r"\hspace{{1cm}} {PIC}"));
        assert!(near(starts(&c)[0].x - c.min.unwrap().x, 0.0070292, 1e-6));
        assert!(near(c.min.unwrap().x, 1.0, 1e-9), "{:?}", c.min);
    }

    /// A `\vbox` of another `\hsize` in a `tabular`: the table is as wide as it.
    #[test]
    fn a_vbox_in_a_table() {
        let c = quiet(&format!(r"\begin{{tabular}}{{c}}\vbox{{\hsize=4cm {PIC}}}\end{{tabular}}"));
        assert!(near(c.max.unwrap().x, 4.0, 1e-6), "{:?}", c.max);
    }

    /// The background's frame with its separation not a macro: no separation that way.
    #[test]
    fn a_frame_without_its_separation() {
        let c = quiet(r"\makeatletter\usetikzlibrary{backgrounds}\begin{tikzpicture}[show background rectangle]\let\tikz@framexsep\relax\draw (0,0)--(1,1);\end{tikzpicture}");
        assert!(near(c.max.unwrap().x - c.min.unwrap().x, 1.0281168, 1e-6), "{:?} {:?}", c.min, c.max);
        assert!(c.max.unwrap().y - c.min.unwrap().y > 1.3);
    }

    /// TeX errors in the code run at the end of a picture are said, and the picture is kept.
    #[test]
    fn errors_at_the_end_of_a_picture() {
        for (code, error) in [(r"\count0=\foo", "missing number"), (r"\csname foo", "\\csname without \\endcsname"), (r"\expandafter", "\\expandafter at the end")] {
            let (c, w) = document(&format!(r"\begin{{tikzpicture}}[execute at end picture={{{code}}}]\draw (0,0)--(1,0);\end{{tikzpicture}}"));
            assert_eq!(w, [format!("execute at end picture: figure: {error}")]);
            assert_eq!(paths(&c).len(), 1);
        }
    }

    /// Pictures nested past 64 levels stop the figure.
    #[test]
    fn pictures_nested_too_deep() {
        let code = format!("{}{}", r"\begin{tikzpicture}".repeat(70), r"\end{tikzpicture}".repeat(70));
        let e = run_document(&code).map(|(_, w)| w);
        match e {
            Err(e) => assert!(e.to_string().contains("nesting went past 64 levels"), "{e}"),
            Ok(w) => assert!(w.iter().any(|w| w.contains("nesting went past 64 levels")), "{w:?}"),
        }
    }

    /// An unknown command in a picture skips its statement, up to TeX's limit of tokens.
    #[test]
    fn a_statement_too_long_to_skip() {
        let code = format!(r"\begin{{tikzpicture}}\draw (0,0)--(1,0); \nosuchcommand {}; \draw (0,1)--(1,1);\end{{tikzpicture}}", "a".repeat(1_000_100));
        let (c, w) = document(&code);
        assert!(w[0].starts_with("\\nosuchcommand not supported"), "{:?}", &w[..1]);
        assert!(!paths(&c).is_empty());
    }

    /// A `\foreach` that never says `in`: an error, after a thousand variables.
    #[test]
    fn foreach_without_in() {
        let (_, w) = document(&format!(r"\begin{{tikzpicture}}\foreach {} \draw (0,0)--(1,0);\end{{tikzpicture}}", r"\x/".repeat(600)));
        assert!(w.iter().any(|w| w.contains("\\foreach without 'in'")), "{w:?}");
    }

    /// `\scoped :xshift = {…}`: in a snapshot, the scope moved as at that time, on the canvas.
    #[test]
    fn scoped_shift_in_a_snapshot() {
        let c = quiet(
            r#"\usetikzlibrary{animations}\begin{tikzpicture}[make snapshot of=1s]\draw (0,1) -- (1,1); \scoped :xshift = {0s="0cm", 2s="2cm"} \draw (0,0) -- (1,0);\end{tikzpicture}"#,
        );
        let shift = c.items.iter().find_map(|i| match i {
            Item::Transform { transform } => Some(transform.e),
            _ => None,
        });
        assert!(near(shift.expect("a transformation of the canvas"), 1.0, 1e-9), "{:?}", c.items);
    }

    /// `Engine::default()` is a fresh engine, as `Engine::new()` makes it.
    #[test]
    fn a_default_engine() {
        let (a, b) = (tikra::engine::Engine::default(), tikra::engine::Engine::new());
        assert_eq!(a.st.key_path, b.st.key_path);
        assert_eq!(a.st.key_path, "/");
    }
}

mod outside_libraries {
    use tikra::extension::{Context, Error, Library, Tokens};

    fn said(cx: &mut Context, _value: Option<Tokens>) -> Result<(), Error> {
        cx.warn("the key ran");
        Ok(())
    }

    static BROKEN: Library = Library { name: "broken definitions", definitions: || r"\count0=\nosuchnumber".into(), keys: &[("/tikz/said", said)] };

    fn convert(body: &str, library: &'static Library) -> tikra::Output {
        tikra::convert_with_libraries(&format!("\\begin{{tikzpicture}}{body}\\end{{tikzpicture}}"), &tikra::Options::default(), &[library]).unwrap()
    }

    /// Definitions that fail stop there; the library's keys are there all the same.
    #[test]
    fn definitions_that_fail() {
        let r = convert(r"\draw[said] (0,0) -- (1,0);", &BROKEN);
        assert!(r.warnings.iter().any(|w| w == "the key ran"), "{:?}", r.warnings);
    }

    /// Past the room for the native keys of the libraries, the rest are left out: unknown keys.
    #[test]
    fn too_many_keys() {
        let keys: Vec<(&'static str, tikra::extension::KeyFn)> =
            (0..26_000).map(|k| (&*Box::leak(format!("/tikz/k{k}").into_boxed_str()), said as tikra::extension::KeyFn)).collect();
        let library: &'static Library = Box::leak(Box::new(Library { name: "many keys", definitions: String::new, keys: Box::leak(keys.into_boxed_slice()) }));
        let r = convert(r"\draw[k0] (0,0) -- (1,0); \draw[k25999] (0,1) -- (1,1);", library);
        assert_eq!(r.warnings, ["library many keys: too many keys, 477 left out from /tikz/k25523 on", "the key ran", "option 'k25999' not supported: ignored"]);
    }
}
