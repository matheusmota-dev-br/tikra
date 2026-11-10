//! What a path's use leaves on the canvas (inner lines, fadings, tips on odd paths, rounded
//! corners), named intersections, pics, overlays, the SVG writer's rarer patterns and animations,
//! the canvas transformed, keys lists, and outside the browser: slides joined, the pen and its
//! plotter files, the construction animation as GIF and APNG.

mod common;
use common::*;
use tikra::canvas::{Canvas, Item, Paint};
use tikra::engine::{run_document, run_document_mapped, Run};
use tikra::scene::Segment;

const PT: f64 = 2.54 / 72.27;

fn picture(body: &str) -> (Canvas, Vec<String>) {
    run_document(&format!("\\begin{{tikzpicture}}{body}\\end{{tikzpicture}}")).unwrap_or_else(|e| panic!("{e}\n{body}"))
}

fn quiet(body: &str) -> Canvas {
    let (c, w) = picture(body);
    assert!(w.is_empty(), "warnings: {w:?}\n{body}");
    c
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

fn svg(body: &str) -> String {
    let r = tikra::convert(&format!("\\begin{{tikzpicture}}{body}\\end{{tikzpicture}}")).unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    r.svg
}

mod using_paths {
    use super::*;

    /// The basic layer's inner line: drawn over the stroke, in its own width and color.
    #[test]
    fn inner_line() {
        let c = quiet(r"\pgfsetinnerlinewidth{1pt}\pgfsetinnerstrokecolor{red}\pgfpathmoveto{\pgfpointorigin}\pgfpathlineto{\pgfpoint{1cm}{0cm}}\pgfusepath{stroke}");
        let p = paths(&c);
        assert_eq!(p.len(), 2);
        let inner = p[1].stroke.as_ref().unwrap();
        assert!(near(inner.width, PT) && near(inner.color.r, 1.0) && near(inner.color.g, 0.0), "{inner:?}");
    }

    /// A fading nobody declared: said, and the path drawn without it.
    #[test]
    fn an_unknown_fading() {
        let (c, w) = picture(r"\fill[path fading=nosuch] (0,0) rectangle (1,1);");
        assert_eq!(w, ["fading 'nosuch' not supported: drawn without it"]);
        assert!(!c.items.iter().any(|i| matches!(i, Item::Fade { .. })));
        assert_eq!(paths(&c).len(), 1);
    }

    /// `tips=proper`: no tips on a path that stays at one point.
    #[test]
    fn proper_tips() {
        let c = quiet(r"\draw[tips=proper, ->] (1,1) -- (1,1); \draw[tips=proper, ->] (0,0) -- (1,0);");
        assert_eq!(paths(&c).len(), 3);
    }

    /// Tips on paths with lone points and degenerate curves: drawn where pgf has them.
    #[test]
    fn tips_on_odd_paths() {
        // a curve whose control points are its end: the tip is aimed from the point before
        let c = quiet(r"\draw[->] (0,0) .. controls (1,0) and (1,0) .. (1,0);");
        let tip = points(&paths(&c)[1].segments);
        assert!(tip.iter().all(|q| q.0 <= 1.0 + 0.05) && tip.iter().any(|q| q.1 > 0.02), "{tip:?}");
        // bars at both ends of the line
        let c = quiet(r"\draw[|-|] (0,0) -- (1,0);");
        assert_eq!(paths(&c).len(), 3);
        // no path at all: no tips either
        let c = quiet(r"\pgfsetarrows{->}\pgfusepath{stroke}");
        assert!(paths(&c).is_empty());
        // the last subpath a lone point: a start tip there, pointing nowhere in particular
        let c = quiet(r"\draw[<-] (0,0) -- (1,0) (2,0);");
        assert!(!paths(&c).is_empty());
    }

    /// Rounded corners: at a side of no length there is no corner; a curve's end is cut to the arc.
    #[test]
    fn rounded_corners() {
        let c = quiet(r"\draw[rounded corners=2mm] (0,0) -- (0,0) -- (1,0) -- (1,1);");
        let s = &paths(&c)[0].segments;
        assert_eq!(s.iter().filter(|x| matches!(x, Segment::CurveTo(..))).count(), 1, "{s:?}");
        let c = quiet(r"\draw[rounded corners=2mm] (0,0) .. controls (0.5,0.5) .. (1,0) -- (2,0) -- (2,1);");
        let s = &paths(&c)[0].segments;
        assert!(s.iter().filter(|x| matches!(x, Segment::CurveTo(..))).count() >= 3, "{s:?}");
    }

    /// A nonlinear transformation bends the path as it is used.
    #[test]
    fn nonlinear_path() {
        let c = quiet(r"\makeatletter\pgftransformnonlinear{\pgf@y=\pgf@y\advance\pgf@y by 0.1\pgf@x}\draw (0,0) -- (2,0);");
        let p = points(&paths(&c)[0].segments);
        assert!(p.last().unwrap().1 - p[0].1 > 0.1, "{p:?}");
    }
}

mod name_intersections {
    use super::*;

    const PATHS: &str = r"\path[name path=a] (0,0) -- (2,2); \path[name path=b] (0,2) -- (2,0); \path[name path=c] (0,1) -- (2,3); \path[name path=d] (5,5);";

    /// `by` names them (options first in brackets), `total` counts them, `sort by` orders them.
    #[test]
    fn by_total_and_sort_by() {
        let body = format!(
            r"{PATHS}\path[name intersections={{of=a and b, by={{[label=above:x]m}}, total=\nint, sort by=b}}] node at (5,0) {{\nint}}; \draw (m) -- (intersection-1); \path[name intersections={{of=a and b, by=n}}]; \draw (n) -- (m);"
        );
        let c = quiet(&body);
        for d in paths(&c) {
            let p = points(&d.segments);
            assert_eq!(p[0], p[1]);
        }
        assert_eq!(paths(&c).len(), 2);
        assert_eq!(texts(&c).len(), 2);
    }

    /// A curve meets a line where its flattened pieces cross it.
    #[test]
    fn a_curve() {
        let c = quiet(
            r"\path[name path=e] (0,0) .. controls (0,2) and (2,2) .. (2,0); \path[name path=f] (1,0) -- (1,3); \path[name intersections={of=e and f}]; \draw (1,0) -- (intersection-1);",
        );
        let p = points(&paths(&c)[0].segments);
        assert!((p[1].1 - p[0].1 - 1.5).abs() < 0.01, "{p:?}");
    }

    /// Parallel paths, and a path that is a point, have none; keys it does not know are said; no
    /// `of`, or a path that does not exist, are errors.
    #[test]
    fn none_and_errors() {
        let (_, w) = picture(&format!(r"{PATHS}\path[name intersections={{of=a and c, total=\nint}}] node {{\nint}};"));
        assert!(w.is_empty(), "{w:?}");
        let (_, w) = picture(&format!(r"{PATHS}\path[name intersections={{of=a and d, foo, bar=1}}];"));
        assert_eq!(w, ["name intersections: 'foo' not supported: ignored", "name intersections: 'bar' not supported: ignored"]);
        let (_, w) = picture(&format!(r"{PATHS}\path[name intersections={{by=x}}];"));
        assert_eq!(w, ["name intersections without of=a and b: ignored"]);
        let (_, w) = picture(&format!(r"{PATHS}\path[name intersections={{of=a and zz}}];"));
        assert!(w[0].contains("name intersections: no path named 'zz'"), "{w:?}");
    }
}

mod pics {
    use super::*;

    /// A pic on a segment at `pos`, `sloped`: placed and turned along it.
    #[test]
    fn a_sloped_pic_on_a_segment() {
        let c = quiet(r"\tikzset{tick/.pic={\draw (0,0) -- (1,0);}}\draw (0,0) -- (2,2) pic[pos=0.5, sloped] {tick};");
        let tick = paths(&c).iter().map(|d| points(&d.segments)).find(|p| near((p[1].0 - p[0].0).hypot(p[1].1 - p[0].1), 1.0)).expect("the pic's line");
        // from the segment's middle, along it
        assert!(near(tick[0].0, tick[0].1) && near(tick[1].0 - tick[0].0, 0.5f64.sqrt()) && near(tick[1].1 - tick[0].1, 0.5f64.sqrt()), "{tick:?}");
        // not sloped: at `pos`, not turned
        let c = quiet(r"\tikzset{tick/.pic={\draw (0,0) -- (1,0);}}\draw (0,0) -- (2,2) pic[pos=0.5] {tick};");
        let tick = paths(&c).iter().map(|d| points(&d.segments)).find(|p| near((p[1].0 - p[0].0).hypot(p[1].1 - p[0].1), 1.0)).expect("the pic's line");
        assert!(near(tick[0].0, tick[0].1) && near(tick[1].1, tick[0].1), "{tick:?}");
        // on a segment that goes left, turned the other way round: not upside down
        let c = quiet(r"\tikzset{tick/.pic={\draw (0,0) -- (1,0);}}\draw (2,2) -- (0,0) pic[pos=0.5, sloped] {tick};");
        let tick = paths(&c).iter().map(|d| points(&d.segments)).find(|p| near((p[1].0 - p[0].0).hypot(p[1].1 - p[0].1), 1.0)).expect("the pic's line");
        assert!(tick[1].0 > tick[0].0 && tick[1].1 > tick[0].1, "{tick:?}");
    }

    /// `pics/…/setup code` runs before the pic's code, outside its scope.
    #[test]
    fn setup_code() {
        let c = quiet(r"\tikzset{pics/thick line/.style={setup code={\tikzset{line width=2pt}}, code={\draw (0,0) -- (1,0);}}}\pic {thick line};");
        assert!(near(paths(&c)[0].stroke.as_ref().unwrap().width, 2.0 * PT));
    }

    /// The angles library: three points expected and known; a radius of no length is 12 pt; the
    /// arc drawn the short way when the end comes first; a pic that only fills.
    #[test]
    fn angles() {
        let points = r"\usetikzlibrary{angles}\coordinate (a) at (1,0); \coordinate (b) at (0,0); \coordinate (c) at (0,1);";
        let (_, w) = picture(&format!(r"{points}\pic {{angle=a--b}};"));
        assert_eq!(w, ["\\pic: figure: angle=a--b: three points A--B--C expected; statement ignored"]);
        let (_, w) = picture(&format!(r"{points}\pic {{angle=a--b--zz}};"));
        assert_eq!(w, ["\\pic: figure: angle: unknown coordinate (zz); statement ignored"]);
        let c = quiet(&format!(r"{points}\pic[draw, angle radius=0pt] {{angle=a--b--c}};"));
        let (lo, hi) = bounds(&paths(&c)[0].segments);
        assert!(near(hi.x.max(hi.y) - lo.x.min(lo.y), 12.0 * PT), "{lo:?} {hi:?}");
        let c = quiet(&format!(r"\coordinate (d) at (0,-1); {points}\pic[draw] {{angle=c--b--d}};"));
        assert!(!paths(&c).is_empty());
        let c = quiet(&format!(r"{points}\pic[fill=red] {{angle=a--b--c}};"));
        assert!(paths(&c).iter().all(|d| d.stroke.is_none()));
    }

    /// A name with the scope's prefix, when the node was made before the prefix.
    #[test]
    fn a_name_without_the_prefix() {
        let c = quiet(r"\coordinate (x) at (1,1); \begin{scope}[name prefix=p-]\draw (0,0) -- (x);\end{scope}");
        let p = points(&paths(&c)[0].segments);
        assert!(near(p[1].0 - p[0].0, 1.0));
    }
}

mod overlays {
    use tikra::{slides, Options};

    fn count(code: &str) -> Vec<usize> {
        slides(&format!(r"\begin{{tikzpicture}}\draw (0,0) -- (1,0);{code}\end{{tikzpicture}}"), &Options::default())
            .unwrap()
            .iter()
            .map(|s| s.svg.matches(r#"<g opacity="0">"#).count())
            .collect()
    }

    /// `\pause[3]`, an empty term in a specification, `\onslide<…>{…}` braced on and off.
    #[test]
    fn more_overlays() {
        assert_eq!(count(r"\pause[3]\draw (0,1) -- (1,1);"), [1, 1, 0]);
        assert_eq!(count(r"\only<1,,3>{\draw (0,1) -- (1,1);}\onslide<2>{\draw (0,2) -- (1,2);}"), [1, 0, 1]);
    }

    /// A specification it cannot read is said, and shown on every slide.
    #[test]
    fn a_specification_not_understood() {
        let r = tikra::convert(r"\begin{tikzpicture}\draw<x> (0,0) -- (1,0);\end{tikzpicture}").unwrap();
        assert_eq!(r.warnings, ["overlay specification <x> not understood: shown on every slide"]);
    }

    /// Overlays outside a picture, or at the end of the input, do nothing.
    #[test]
    fn outside_pictures() {
        let s = slides(r"\onslide<2>{x}\uncover<2>{}\begin{tikzpicture}\draw (0,0) -- (1,0);\uncover<2>{}\end{tikzpicture}\onslide", &Options::default()).unwrap();
        assert_eq!(s.len(), 2);
        assert!(s.iter().all(|o| !o.svg.contains(r#"<g opacity="0">"#)));
    }
}

mod the_svg {
    use super::*;

    /// A canvas made by hand: a path with no segments, or with neither fill nor stroke, is not
    /// written.
    #[test]
    fn paths_that_draw_nothing() {
        use tikra::canvas::{Fill, Stroke};
        use tikra::scene::Point;
        let segments = vec![Segment::MoveTo(Point::new(0.0, 0.0)), Segment::LineTo(Point::new(1.0, 1.0))];
        let (c, _) = picture(r"\draw (0,0) -- (1,1);");
        let stroke: Stroke = paths(&c)[0].stroke.clone().unwrap();
        let mut canvas = c.clone();
        canvas.items = vec![
            Item::Path { segments: Vec::new(), fill: Some(Fill { paint: Paint::Solid(tikra::color::Color::BLACK), opacity: 1.0, even_odd: false }), stroke: None },
            Item::Path { segments: segments.clone(), fill: None, stroke: None },
            Item::Path { segments, fill: None, stroke: Some(stroke) },
        ];
        let out = tikra::driver::svg(&canvas, "", &tikra::describe::Description::default());
        assert_eq!(out.split("</metadata>").last().unwrap().split("</defs>").last().unwrap().matches("<path").count(), 1, "{out}");
    }

    /// Bars whose path has another number of pieces than points: the path is written as one, with
    /// no points.
    #[test]
    fn bars_that_do_not_match_their_path() {
        use tikra::canvas::DataPoint;
        use tikra::scene::Point;
        let (c, _) = picture(r"\draw (0,0) -- (1,1);");
        let stroke = paths(&c)[0].stroke.clone();
        let mut canvas = c.clone();
        let p = Point::new;
        canvas.items = vec![
            Item::Bars { points: vec![DataPoint { x: "1".into(), y: "2".into(), at: p(0.0, 0.0) }], vertical: true },
            Item::Path {
                segments: vec![Segment::MoveTo(p(0.0, 0.0)), Segment::LineTo(p(0.0, 1.0)), Segment::MoveTo(p(1.0, 0.0)), Segment::LineTo(p(1.0, 1.0))],
                fill: None,
                stroke,
            },
        ];
        let out = tikra::driver::svg(&canvas, "", &tikra::describe::Description::default());
        let body = out.split("</metadata>").last().unwrap().split("</defs>").last().unwrap();
        assert_eq!(body.matches("<path").count(), 1, "{out}");
        assert!(!body.contains("tr-point"), "{out}");
    }

    /// A dash pattern of no lengths is a solid line.
    #[test]
    fn a_dash_of_nothing() {
        assert!(!svg(r"\draw[dash pattern=on 0pt off 0pt] (0,0) -- (1,0);").contains("stroke-dasharray"));
    }

    /// The patterns of the patterns.meta library with fixed colors: their tiles.
    #[test]
    fn colored_patterns() {
        for name in ["horizontal lines dark blue", "crosshatch dots gray", "crosshatch dots light steel blue"] {
            let s = svg(&format!(r"\fill[pattern={name}] (0,0) rectangle (1,1);"));
            assert!(s.contains("<pattern"), "{name}");
            assert!(s.contains(if name.contains("dots") { "<circle" } else { "<rect" }), "{name}");
        }
    }

    /// Played animations of a scaling, and of a node's text.
    #[test]
    fn played_animations() {
        let r = tikra::convert_with(
            r#"\usetikzlibrary{animations}\begin{tikzpicture}\node[:scale={0s="1", 2s="2"}] {a}; \draw[:opacity={0s="1", 2s="0"}] (0,0) -- (1,0);\node[draw, :color={0s="red", 2s="blue"}] at (2,0) {b};\end{tikzpicture}"#,
            &tikra::Options::default().play(),
        )
        .unwrap();
        assert!(r.svg.contains(r#"type="scale""#), "{}", r.svg);
        assert!(r.svg.contains("<animate"), "{}", r.svg);
    }
}

mod the_canvas_transformed {
    use super::*;

    /// A picture in `\scalebox`: its dashes, fadings, canvas transformations and faded pictures
    /// scaled with it.
    #[test]
    fn scaled_items() {
        let code = r"\scalebox{2}{\tikz{\draw[dash pattern=on 1pt off 1pt] (0,0) -- (1,0); \fill[path fading=north] (0,0) rectangle (1,1); \draw[transform canvas={shift={(1,0)}}] (0,0) -- (1,0);
                      \pgfdeclarefading{f}{\tikz\fill[white] (0,0) rectangle (1,1);} \fill[path fading=f] (0,0) rectangle (1,1);}}";
        let (c, w) = run_document(code).unwrap();
        assert!(w.is_empty(), "{w:?}");
        let dash = paths(&c)[0].stroke.as_ref().unwrap().dash.clone().unwrap();
        assert!(near(dash.0[0], 2.0 * PT), "{dash:?}");
        assert!(c.items.iter().any(|i| matches!(i, Item::Fade { mask: tikra::canvas::Shading::Axial { transform, .. } } if transform.a > 0.0)));
        assert!(c.items.iter().any(|i| matches!(i, Item::Transform { transform } if near(transform.e, 2.0))), "{:?}", c.items);
        assert!(c.items.iter().any(|i| matches!(i, Item::FadePicture { .. })));
    }

    /// Played animations and the points of a plot, scaled with the picture.
    #[test]
    fn scaled_animations_and_points() {
        let code = r#"\usetikzlibrary{animations}\scalebox{2}{\tikz{\node[:rotate={0s="0", 2s="90"}] at (1,1) {a};}}"#;
        let (c, _, _) = run_document_mapped(code, &[], Run { play: true, ..Run::default() }).unwrap();
        let center = c.items.iter().find_map(|i| match i {
            Item::Animated { center, .. } => Some(*center),
            _ => None,
        });
        assert!(center.is_some(), "{:?}", c.items);
        let code = r"\scalebox{2}{\begin{tikzpicture}\begin{axis}[ybar]\addplot coordinates {(0,1) (1,2)};\end{axis}\end{tikzpicture}}";
        let (c, _, _) = run_document_mapped(code, &[], Run { points: true, ..Run::default() }).unwrap();
        assert!(c.items.iter().any(|i| matches!(i, Item::Bars { .. })));
    }

    /// Two tiles are the same when they are the same list.
    #[test]
    fn tiles_compare_by_their_list() {
        use tikra::canvas::TileItems;
        let a = TileItems(std::rc::Rc::new(Vec::new()));
        assert_eq!(a, a.clone());
        assert_ne!(a, TileItems(std::rc::Rc::new(Vec::new())));
        let _ = Paint::Solid(tikra::color::Color::BLACK);
    }
}

mod keys {
    use super::*;

    /// An empty item in braces is no key; a key at the root (`/foo`) is unknown there.
    #[test]
    fn empty_and_root_keys() {
        let c = quiet(r"\draw[{}, red] (0,0) -- (1,0);");
        assert!(near(paths(&c)[0].stroke.as_ref().unwrap().color.r, 1.0));
        let (_, w) = picture(r"\pgfkeys{/foo}");
        assert_eq!(w, ["option 'foo' not supported: ignored"]);
    }

    /// The parent of a key's path: the path without its last part, `/` for a key at the root, and
    /// nothing for a name with no path.
    #[test]
    fn parents() {
        use tikra::keys::parent;
        assert_eq!((parent("/tikz/draw"), parent("/foo"), parent("draw")), ("/tikz", "/", ""));
    }
}

mod slides_joined {
    use tikra::{click_through, slides, Options};

    /// The ids of each slide are its own in the SVG of all of them (the glyphs' are shared).
    #[test]
    fn ids_apart() {
        let s = slides(r"\begin{tikzpicture}\clip (0,0) rectangle (2,2); \draw (0,0) -- (3,3); \only<2>{\node {x};}\end{tikzpicture}", &Options::default()).unwrap();
        let all = click_through(&s).unwrap();
        assert!(all.contains(r#"id="s1-"#) && all.contains(r#"id="s2-"#), "{all}");
        assert!(all.contains("url(#s1-") && all.contains("url(#s2-"));
    }
}

mod pen {
    use tikra::pen::{gcode, strokes, Pen};

    /// A quadratic curve becomes short lines; what is neither a path nor a group draws nothing.
    #[test]
    fn strokes_of_an_svg() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="96" height="96" viewBox="0 0 96 96"><path d="M0 0 Q48 96 96 0" fill="none" stroke="black"/><image x="0" y="0" width="1" height="1" href="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="/></svg>"#;
        let (s, size) = strokes(svg).unwrap();
        assert_eq!(s.len(), 1);
        assert!(s[0].len() > 10);
        assert!((size.0 - 25.4).abs() < 1e-6);
        // the curve's middle is half its control point's height
        let top = s[0].iter().map(|p| p.1).fold(f64::MIN, f64::max);
        assert!((top - 25.4).abs() < 0.1 || (top - 12.7).abs() < 0.1, "{top}");
    }

    /// On paper, the strokes are centered on it.
    #[test]
    fn on_paper() {
        let svg = tikra::convert(r"\begin{tikzpicture}\draw (0,0) -- (1,0);\end{tikzpicture}").unwrap().svg;
        let pen = Pen { paper: Some((100.0, 100.0)), ..Pen::default() };
        let code = gcode(&svg, &pen).unwrap();
        let first = code.lines().find(|l| l.starts_with("G0 X")).unwrap();
        let x: f64 = first.split_whitespace().nth(1).unwrap()[1..].parse().unwrap();
        assert!(x > 40.0, "{code}");
    }
}

#[cfg(feature = "motion")]
mod motion {
    use tikra::motion::{apng, gif, pen_apng, pen_gif, Motion};
    use tikra::pen::Pen;

    fn animated(body: &str) -> String {
        tikra::convert_with(&format!(r"\begin{{tikzpicture}}{body}\end{{tikzpicture}}"), &tikra::Options::default().animated()).unwrap().svg
    }

    /// No width or no frames per second, a still SVG, a size past the pixel cap: refused.
    #[test]
    fn refused() {
        let svg = animated(r"\draw (0,0) -- (1,0);");
        for m in [Motion { width: 0, fps: 10 }, Motion { width: 100, fps: 0 }] {
            assert!(gif(&svg, m).unwrap_err().to_string().contains("must be over 0"));
            assert!(pen_gif(&svg, &Pen::default(), m).unwrap_err().to_string().contains("must be over 0"));
        }
        assert!(apng(&svg, Motion { width: 100_000, fps: 1 }).unwrap_err().to_string().contains("too large"));
    }

    /// A pen drawing the figure, as GIF and APNG frames.
    #[test]
    fn pen_frames() {
        let svg = tikra::convert(r"\begin{tikzpicture}\draw (0,0) -- (1,1); \draw (2,0) circle (1);\end{tikzpicture}").unwrap().svg;
        let pen = Pen { seconds: 1.0, ..Pen::default() };
        let m = Motion { width: 60, fps: 4 };
        let g = pen_gif(&svg, &pen, m).unwrap();
        assert!(g.starts_with(b"GIF89a") && u16::from_le_bytes([g[6], g[7]]) == 60);
        let a = pen_apng(&svg, &pen, m).unwrap();
        let at = a.windows(4).position(|w| w == b"acTL").unwrap();
        assert_eq!(u32::from_be_bytes(a[at + 4..at + 8].try_into().unwrap()), 5);
    }

    /// A large detailed figure: its GIF's codes fill LZW's table, which starts over.
    #[test]
    fn a_long_gif() {
        let svg =
            animated(r"\foreach \i in {1,...,40} { \draw[red!\i!blue, line width=\i/10 pt] (rnd*4,rnd*4) -- (rnd*4,rnd*4); \node[text=green!\i!black] at (rnd*4, rnd*4) {\i}; }");
        let g = gif(&svg, Motion { width: 400, fps: 2 }).unwrap();
        assert!(g.starts_with(b"GIF89a") && g.ends_with(&[0x3b]));
    }

    /// An animated SVG written by hand: a path with other commands, and no rules to take out.
    #[test]
    fn a_hand_written_animation() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10" width="10pt" height="10pt" data-steps="1"><g style="animation-delay:calc(var(--i)*0.2s)"><path class="tr-d" style="--i:0" d="M 0 0 H 10 V 10" stroke="black" fill="none"/></g></svg>"#;
        let g = gif(svg, Motion { width: 20, fps: 5 }).unwrap();
        assert!(g.starts_with(b"GIF89a"));
    }
}
