//! The basic layer's declared shadings (`\pgfuseshading`, `\pgfshadepath`), fadings drawn as
//! pictures (`\pgfdeclarefading`, `tikzfadingfrompicture`, `\pgfsetfading`…), declared and meta
//! patterns (`\pgfdeclarepatternformonly`, `\tikzdeclarepattern`, `Lines[…]`, `\pgfsetfillpattern`),
//! old-style arrow tips (`\pgfarrowsdeclare`) and plot marks (`plot[mark=…]`, `\pgfuseplotmark`).

mod common;
use common::*;
use tikra::canvas::{Canvas, Fill, Item, Paint, Shading};
use tikra::color::Color;
use tikra::scene::Segment;

const PT: f64 = tikra::PT;
const BP: f64 = 2.54 / 72.0;

const RED: Color = Color::rgb(1.0, 0.0, 0.0);
const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);
const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);

/// The paths filled with a shading: their segments and the shading.
fn shaded(c: &Canvas) -> Vec<(Vec<Segment>, Shading)> {
    paths(c)
        .into_iter()
        .filter_map(|d| match d.fill {
            Some(Fill { paint: Paint::Shading(s), .. }) => Some((d.segments, s)),
            _ => None,
        })
        .collect()
}

fn stops(s: &Shading) -> Vec<(f64, Color)> {
    match s {
        Shading::Axial { stops, .. } | Shading::Radial { stops, .. } => stops.clone(),
    }
}

/// Whether a bounding box is the one given, to 1e-6 cm.
fn boxed(segs: &[Segment], lo: (f64, f64), hi: (f64, f64)) -> bool {
    let (a, b) = bounds(segs);
    close_to((a.x, a.y), lo) && close_to((b.x, b.y), hi)
}

// ---------------------------------------------------------------------------------------------
// shadings

/// A horizontal shading is as long as its stops and as high as declared, centered at the origin.
#[test]
fn use_a_horizontal_shading() {
    let (c, w) = canvas_and_warnings(r"\pgfdeclarehorizontalshading{h}{1cm}{rgb(0cm)=(1,0,0); color(2cm)=(blue)} \pgfuseshading{h}");
    assert!(w.is_empty(), "{w:?}");
    let s = shaded(&c);
    assert_eq!(s.len(), 1);
    assert!(boxed(&s[0].0, (-1.0, -0.5), (1.0, 0.5)), "{:?}", bounds(&s[0].0));
    assert!(matches!(s[0].1, Shading::Axial { .. }));
    assert_eq!(stops(&s[0].1), vec![(0.0, RED), (1.0, BLUE)]);
}

/// A vertical shading is as wide as declared and as high as its stops; gray stops are grays.
#[test]
fn use_a_vertical_shading() {
    let (c, w) = canvas_and_warnings(r"\pgfdeclareverticalshading{v}{3cm}{gray(1cm)=(1); gray(0cm)=(0.5)} \pgfuseshading{v}");
    assert!(w.is_empty(), "{w:?}");
    let s = shaded(&c);
    assert!(boxed(&s[0].0, (-1.5, -0.5), (1.5, 0.5)), "{:?}", bounds(&s[0].0));
    // the stops in order of position
    assert_eq!(stops(&s[0].1), vec![(0.0, Color::rgb(0.5, 0.5, 0.5)), (1.0, Color::rgb(1.0, 1.0, 1.0))]);
}

/// A radial shading paints the circle of its last radius.
#[test]
fn use_a_radial_shading() {
    let (c, w) = canvas_and_warnings(r"\pgfdeclareradialshading{r}{\pgfpoint{0cm}{0cm}}{color(0cm)=(red); color(1cm)=(blue)} \pgfuseshading{r}");
    assert!(w.is_empty(), "{w:?}");
    let s = shaded(&c);
    assert!(boxed(&s[0].0, (-1.0, -1.0), (1.0, 1.0)), "{:?}", bounds(&s[0].0));
    assert!(s[0].0.iter().any(|g| matches!(g, Segment::CurveTo(..))));
    assert!(matches!(s[0].1, Shading::Radial { .. }));
}

/// A shading declared with the colors it depends on takes them as they are where it is used.
#[test]
fn a_shading_takes_its_colors_when_used() {
    let body = r"\colorlet{c1}{red} \pgfdeclarehorizontalshading[c1]{d}{1cm}{color(0cm)=(c1); rgb(1cm)=(0,0,1)} \colorlet{c1}{green} \pgfuseshading{d}";
    let s = shaded(&canvas(body));
    assert_eq!(stops(&s[0].1), vec![(0.0, GREEN), (1.0, BLUE)]);
}

/// A declaration without a stop declares nothing.
#[test]
fn a_shading_without_stops() {
    let (c, w) = canvas_and_warnings(r"\pgfdeclarehorizontalshading{e}{1cm}{red; (blue)} \pgfuseshading{e}");
    assert!(shaded(&c).is_empty());
    assert!(w.iter().any(|a| a.contains("shading 'e' not declared")), "{w:?}");
}

/// A functional shading and TikZ's own (`axis`, `ball`) fill their 100 bp box.
#[test]
fn use_a_functional_or_a_tikz_shading() {
    let half = 50.0 * BP;
    let (c, w) = drawn_and_warnings(r"\pgfuseshading{color wheel}");
    assert!(w.is_empty(), "{w:?}");
    assert!(!paths(&c).is_empty());
    let (lo, hi) = (c.min.unwrap(), c.max.unwrap());
    assert!(close(hi.x - lo.x, 2.0 * half) && close(hi.y - lo.y, 2.0 * half), "{lo:?} {hi:?}");
    // in a picture whose size is set, the same cells, centered at the origin
    let p = paths(&canvas(r"\pgfuseshading{color wheel}"));
    let (lo, hi) = bounds(&p.iter().flat_map(|d| d.segments.clone()).collect::<Vec<_>>());
    assert!(close_to((lo.x, lo.y), (-half, -half)) && close_to((hi.x, hi.y), (half, half)), "{lo:?} {hi:?}");
    for name in ["axis", "radial", "ball"] {
        let (c, w) = canvas_and_warnings(&format!(r"\tikzset{{top color=red, bottom color=blue}} \pgfuseshading{{{name}}}"));
        assert!(w.is_empty(), "{name}: {w:?}");
        let s = shaded(&c);
        assert!(boxed(&s[0].0, (-half, -half), (half, half)), "{name}: {:?}", bounds(&s[0].0));
    }
    let (c, w) = canvas_and_warnings(r"\pgfuseshading{nothing}");
    assert!(shaded(&c).is_empty());
    assert!(w.iter().any(|a| a.contains("shading 'nothing' not declared")), "{w:?}");
}

/// Outside a picture, `\pgfuseshading` is a box of its own, as large as the shading.
#[test]
fn use_a_shading_outside_a_picture() {
    for text in ["", "Some text "] {
        for (declare, name, wd, ht) in [
            (r"\pgfdeclarehorizontalshading{h}{1cm}{color(0cm)=(red); color(2cm)=(blue)}", "h", 2.0, 1.0),
            (r"\pgfdeclarefunctionalshading{f}{\pgfpoint{0bp}{0bp}}{\pgfpoint{20bp}{10bp}}{}{pop pop 1 0 0}", "f", 20.0 * BP, 10.0 * BP),
        ] {
            let (c, w) = tikra::engine::run_document(&format!(r"{declare}{text}\pgfuseshading{{{name}}}")).unwrap();
            assert!(w.is_empty(), "{w:?}");
            let p = paths(&c);
            assert!(!p.is_empty(), "{name}");
            let (lo, hi) = bounds(&p.iter().flat_map(|d| d.segments.clone()).collect::<Vec<_>>());
            assert!(close(hi.x - lo.x, wd) && close(hi.y - lo.y, ht), "{name}: {lo:?} {hi:?}");
        }
    }
    let (c, w) = tikra::engine::run_document(r"\pgfuseshading{nothing}").unwrap();
    assert!(paths(&c).is_empty());
    assert!(w.iter().any(|a| a.contains("shading 'nothing' not declared")), "{w:?}");
}

/// `\pgfshadepath` fills the path being built with the shading, fitted to it.
#[test]
fn shade_the_path() {
    let body = r"\pgfdeclarehorizontalshading{h}{1cm}{color(0cm)=(red); color(2cm)=(blue)}
        \pgfpathrectangle{\pgfpoint{0cm}{0cm}}{\pgfpoint{2cm}{1cm}} \pgfshadepath{h}{90}";
    let (c, w) = canvas_and_warnings(body);
    assert!(w.is_empty(), "{w:?}");
    let s = shaded(&c);
    assert!(boxed(&s[0].0, (0.0, 0.0), (2.0, 1.0)), "{:?}", bounds(&s[0].0));
    assert_eq!(stops(&s[0].1), vec![(0.0, RED), (1.0, BLUE)]);
    let (c, w) = canvas_and_warnings(r"\pgfpathrectangle{\pgfpoint{0cm}{0cm}}{\pgfpoint{2cm}{1cm}} \pgfshadepath{nothing}{0}");
    assert!(shaded(&c).is_empty());
    assert!(w.iter().any(|a| a.contains("shading 'nothing' not declared")), "{w:?}");
}

// ---------------------------------------------------------------------------------------------
// fadings

/// The items of the masks a canvas has: picture fadings' items, or `None` for a shading's.
fn masks(c: &Canvas) -> Vec<Option<Vec<Item>>> {
    c.items
        .iter()
        .filter_map(|i| match i {
            Item::FadePicture { items } => Some(Some(items.clone())),
            Item::Fade { .. } => Some(None),
            _ => None,
        })
        .collect()
}

/// A picture fading's paths, as boxes.
fn mask_box(items: &[Item]) -> (tikra::scene::Point, tikra::scene::Point) {
    let segs: Vec<Segment> = items.iter().filter_map(|i| if let Item::Path { segments, .. } = i { Some(segments.clone()) } else { None }).flatten().collect();
    bounds(&segs)
}

/// `\pgfdeclarefading` takes a picture, a shading or text; `\pgfsetfading` centers it at the origin.
#[test]
fn declare_and_set_a_fading() {
    // (code, whether it draws a path whose box can be checked)
    let cases = [
        (r"\tikz \fill[white] (0,0) rectangle (2,1);", true),
        (r"\pgfuseshading{axis}", true),
        (r"\tikzset{top color=white} \pgfuseshading{axis}", true),
        (r"\pgftext{A}", false),
        ("Faded", false),
    ];
    for (code, has_path) in cases {
        let body = format!(
            r"\pgfdeclarefading{{fa}}{{{code}}} \begin{{scope}} \pgfsetfading{{fa}}{{\pgftransformshift{{\pgfpoint{{3cm}}{{0cm}}}}}} \fill (0,0) rectangle (1,1); \end{{scope}}"
        );
        let (c, w) = canvas_and_warnings(&body);
        assert!(w.is_empty(), "{code}: {w:?}");
        let m = masks(&c);
        assert_eq!(m.len(), 1, "{code}");
        let items = m[0].clone().expect("a picture fading");
        assert!(!items.is_empty(), "{code}");
        if has_path {
            // centered at (3,0)
            let (lo, hi) = mask_box(&items);
            assert!(close((lo.x + hi.x) / 2.0, 3.0) && close((lo.y + hi.y) / 2.0, 0.0), "{code}: {lo:?} {hi:?}");
        } else {
            assert!(items.iter().any(|i| matches!(i, Item::Text { .. })), "{code}");
        }
    }
}

/// A fading whose picture is left open: the picture is closed with the fading, and it fades.
#[test]
fn fading_with_an_open_picture() {
    let body = r"\pgfdeclarefading{fa}{\begin{pgfpicture}\pgfpathrectangle{\pgfpointorigin}{\pgfpoint{2cm}{1cm}}\pgfusepath{fill}}
        \begin{scope} \pgfsetfading{fa}{} \fill (0,0) rectangle (1,1); \end{scope}";
    let (c, w) = canvas_and_warnings(body);
    assert!(w.iter().any(|a| a.contains(r"\begin{pgfpicture} without \end")), "{w:?}");
    let m = masks(&c);
    let (lo, hi) = mask_box(m[0].as_ref().expect("a picture fading"));
    assert!(close_to((lo.x, lo.y), (-1.0, -0.5)) && close_to((hi.x, hi.y), (1.0, 0.5)), "{lo:?} {hi:?}");
}

/// `tikzfadingfrompicture` declares a fading from a picture with its options.
#[test]
fn fading_from_picture() {
    let body = r"\begin{tikzfadingfrompicture}[name=fp, x=2cm, yshift=1cm]
        \begin{scope}[y=3cm] \fill[white] (0,0) rectangle (1,1); \end{scope}
        \end{tikzfadingfrompicture}
        \begin{scope} \pgfsetfading{fp}{} \fill (0,0) rectangle (1,1); \end{scope}";
    let (c, w) = canvas_and_warnings(body);
    assert!(w.is_empty(), "{w:?}");
    let m = masks(&c);
    let (lo, hi) = mask_box(m[0].as_ref().unwrap());
    // 2cm wide (x=2cm), 3cm high (y=3cm), centered at the origin
    assert!(close_to((lo.x, lo.y), (-1.0, -1.5)) && close_to((hi.x, hi.y), (1.0, 1.5)), "{lo:?} {hi:?}");
}

/// `\pgfsetfadingforcurrentpath` fits a picture fading to the path's box: 50 bp of it become the box.
#[test]
fn fading_fitted_to_the_path() {
    let declare = r"\pgfdeclarefading{fa}{\tikz \fill[white] (0,0) rectangle (50bp,50bp);}";
    let body = format!(
        r"{declare} \begin{{scope}} \pgfpathrectangle{{\pgfpoint{{1cm}}{{1cm}}}}{{\pgfpoint{{2cm}}{{4cm}}}} \pgfsetfadingforcurrentpath{{fa}}{{}} \pgfusepath{{fill}} \end{{scope}}"
    );
    let (c, w) = canvas_and_warnings(&body);
    assert!(w.is_empty(), "{w:?}");
    let (lo, hi) = mask_box(masks(&c)[0].as_ref().unwrap());
    assert!(close_to((lo.x, lo.y), (1.0, 1.0)) && close_to((hi.x, hi.y), (3.0, 5.0)), "{lo:?} {hi:?}");
    // stroked: the box widened by half the line width (1pt here)
    let body = format!(
        r"{declare} \begin{{scope}} \pgfsetlinewidth{{2pt}} \pgfpathrectangle{{\pgfpoint{{1cm}}{{1cm}}}}{{\pgfpoint{{2cm}}{{4cm}}}} \pgfsetfadingforcurrentpathstroked{{fa}}{{}} \pgfusepath{{stroke}} \end{{scope}}"
    );
    let (lo, hi) = mask_box(masks(&canvas(&body))[0].as_ref().unwrap());
    assert!(close_to((lo.x, lo.y), (1.0 - PT, 1.0 - PT)) && close_to((hi.x, hi.y), (3.0 + PT, 5.0 + PT)), "{lo:?} {hi:?}");
    // a library fading is placed as it is
    let body = r"\begin{scope} \pgfpathrectangle{\pgfpoint{1cm}{1cm}}{\pgfpoint{2cm}{4cm}} \pgfsetfadingforcurrentpath{south}{} \pgfusepath{fill} \end{scope}";
    let m = masks(&canvas(body));
    assert!(m.len() == 1 && m[0].is_none());
    // without a path, nothing
    let (c, w) = canvas_and_warnings(&format!(r"{declare} \begin{{scope}} \pgfsetfadingforcurrentpath{{fa}}{{}} \end{{scope}}"));
    assert!(masks(&c).is_empty() && w.is_empty(), "{w:?}");
}

/// `scope fading` with a picture fading: fitted to the path, or at its natural size.
#[test]
fn scope_fading_of_a_picture_fading() {
    let declare = r"\pgfdeclarefading{fa}{\tikz \fill[white] (0,0) rectangle (50bp,50bp);}";
    let (lo, hi) = mask_box(masks(&canvas(&format!(r"{declare} \fill[scope fading=fa] (0,0) rectangle (2,2);")))[0].as_ref().unwrap());
    assert!(close_to((lo.x, lo.y), (0.0, 0.0)) && close_to((hi.x, hi.y), (2.0, 2.0)), "{lo:?} {hi:?}");
    let (lo, hi) = mask_box(masks(&canvas(&format!(r"{declare} \fill[scope fading=fa, fit fading=false] (0,0) rectangle (2,2);")))[0].as_ref().unwrap());
    assert!(close_to((lo.x, lo.y), (-25.0 * BP, -25.0 * BP)) && close_to((hi.x, hi.y), (25.0 * BP, 25.0 * BP)), "{lo:?} {hi:?}");
    // a path that draws nothing hands it, fitted to itself, to the next one
    let c = canvas(&format!(r"{declare} \path[scope fading=fa] (0,0) rectangle (2,2); \fill (5,5) circle (1);"));
    let (lo, hi) = mask_box(masks(&c)[0].as_ref().unwrap());
    assert!(close_to((lo.x, lo.y), (0.0, 0.0)) && close_to((hi.x, hi.y), (2.0, 2.0)), "{lo:?} {hi:?}");
    // a library fading fitted to the path is a shading mask
    let m = masks(&canvas(r"\fill[scope fading=south] (0,0) rectangle (2,2);"));
    assert!(m.len() == 1 && m[0].is_none());
}

/// A fading made by `\tikzfading` is set as a shading mask; an unknown one is warned about.
#[test]
fn tikz_fading_and_unknown_fading() {
    let body = r"\tikzfading[name=mine, top color=transparent!0, bottom color=transparent!100]
        \begin{scope} \pgfsetfading{mine}{} \fill (0,0) rectangle (1,1); \end{scope}";
    let (c, w) = canvas_and_warnings(body);
    assert!(w.is_empty(), "{w:?}");
    let m = masks(&c);
    assert!(m.len() == 1 && m[0].is_none());
    let (c, w) = canvas_and_warnings(r"\begin{scope} \pgfsetfading{nothing}{} \fill (0,0) rectangle (1,1); \end{scope}");
    assert!(masks(&c).is_empty());
    assert!(w.iter().any(|a| a.contains("fading 'nothing' not supported")), "{w:?}");
}

// ---------------------------------------------------------------------------------------------
// patterns

/// The tile of the first path filled with a pattern: its items, origin, size and transformation.
fn tile(c: &Canvas) -> (Vec<Item>, (f64, f64), (f64, f64), tikra::scene::Transform) {
    paths(c)
        .into_iter()
        .find_map(|d| match d.fill {
            Some(Fill { paint: Paint::Tile { items, origin, size, transform }, .. }) => Some(((*items.0).clone(), (origin.x, origin.y), (size.x, size.y), transform)),
            _ => None,
        })
        .expect("a pattern fill")
}

/// The colors a tile's items are filled and stroked with.
fn tile_colors(items: &[Item]) -> Vec<Color> {
    items
        .iter()
        .filter_map(|i| match i {
            Item::Path { fill, stroke, .. } => Some(fill.as_ref().and_then(|f| if let Paint::Solid(c) = f.paint { Some(c) } else { None }).or(stroke.as_ref().map(|s| s.color))?),
            _ => None,
        })
        .collect()
}

/// A form-only pattern is drawn in the color it is used with, fills and strokes alike.
#[test]
fn form_only_pattern_takes_the_color() {
    let body = r"\pgfdeclarepatternformonly{dd}{\pgfpointorigin}{\pgfpoint{4pt}{4pt}}{\pgfpoint{4pt}{4pt}}
        {\pgfpathcircle{\pgfpoint{2pt}{2pt}}{1pt}\pgfusepath{fill}\pgfpathmoveto{\pgfpointorigin}\pgfpathlineto{\pgfpoint{4pt}{4pt}}\pgfusepath{stroke}}
        \fill[pattern=dd, pattern color=red] (0,0) rectangle (1,1);";
    let (c, w) = canvas_and_warnings(body);
    assert!(w.is_empty(), "{w:?}");
    let (items, origin, size, _) = tile(&c);
    assert_eq!(tile_colors(&items), vec![RED, RED]);
    assert!(close_to(origin, (0.0, 0.0)) && close_to(size, (4.0 * PT, 4.0 * PT)), "{origin:?} {size:?}");
    // an inherently colored one keeps its colors
    let body = r"\pgfdeclarepatterninherentlycolored{cc}{\pgfpointorigin}{\pgfpoint{4pt}{4pt}}{\pgfpoint{4pt}{4pt}}
        {\pgfsetfillcolor{blue}\pgfpathcircle{\pgfpoint{2pt}{2pt}}{1pt}\pgfusepath{fill}}
        \fill[pattern=cc, pattern color=red] (0,0) rectangle (1,1);";
    assert_eq!(tile_colors(&tile(&canvas(body)).0), vec![BLUE]);
}

/// `\tikzdeclarepattern`: `bounding box` gives the tile's corner; without a corner it is the origin.
#[test]
fn tikz_pattern_corner() {
    for (corner, want) in [("bounding box={(1pt,2pt) and (5pt,6pt)},", (PT, 2.0 * PT)), ("", (0.0, 0.0))] {
        let body = format!(
            r"\tikzdeclarepattern{{name=bb, type=uncolored, {corner} top right={{(4pt,4pt)}}, tile size={{(4pt,4pt)}}, code={{\draw (0,0) -- (4pt,4pt);}}}}
            \fill[pattern=bb, pattern color=blue] (0,0) rectangle (1,1);"
        );
        let (c, w) = canvas_and_warnings(&body);
        assert!(w.is_empty(), "{corner}: {w:?}");
        let (items, origin, size, _) = tile(&c);
        assert!(close_to(origin, want) && close_to(size, (4.0 * PT, 4.0 * PT)), "{corner}: {origin:?} {size:?}");
        assert_eq!(tile_colors(&items), vec![BLUE]);
    }
}

/// A basic-layer pattern with variables is drawn where it is used, with their values there.
#[test]
fn basic_pattern_with_variables() {
    let declare = r"\pgfdeclarepatternformonly[\mysize]{vr}{\pgfpointorigin}{\pgfpoint{\mysize}{\mysize}}{\pgfpoint{\mysize}{\mysize}}
        {\pgfpathcircle{\pgfpointorigin}{1pt}\pgfusepath{fill}}";
    for size in [5.0, 8.0] {
        let (c, w) = canvas_and_warnings(&format!(r"{declare} \def\mysize{{{size}pt}} \fill[pattern=vr] (0,0) rectangle (1,1);"));
        assert!(w.is_empty(), "{w:?}");
        let (_, _, s, _) = tile(&c);
        assert!(close_to(s, (size * PT, size * PT)), "{s:?}");
    }
}

/// `\pgfdeclarepattern`: points as `\pgfpoint`, the tile transformation as basic-layer code.
#[test]
fn basic_declare_pattern() {
    let body = r"\pgfdeclarepattern{name=pp, type=uncolored, bottom left=\pgfpoint{1pt}{1pt}, tile size=\pgfpoint{4pt}{4pt},
        tile transformation={\pgftransformrotate{30}}, code={\pgfpathcircle{\pgfpoint{2pt}{2pt}}{1pt}\pgfusepath{fill}}}
        \fill[pattern=pp, pattern color=red] (0,0) rectangle (1,1);";
    let (c, w) = canvas_and_warnings(body);
    assert!(w.is_empty(), "{w:?}");
    let (items, origin, size, t) = tile(&c);
    assert!(close_to(origin, (PT, PT)) && close_to(size, (4.0 * PT, 4.0 * PT)), "{origin:?} {size:?}");
    let r = tikra::scene::Transform::rotation(30.0);
    assert!(close(t.a, r.a) && close(t.b, r.b) && close(t.c, r.c) && close(t.d, r.d), "{t:?}");
    assert_eq!(tile_colors(&items), vec![RED]);
}

/// TikZ's pattern with parameters: its defaults, then the options after its name.
#[test]
fn tikz_pattern_with_parameters() {
    let declare = r"\tikzdeclarepattern{name=st, type=uncolored, parameters={\psize}, defaults={psize/.store in=\psize, psize=3pt},
        tile size={(\psize,\psize)}, tile transformation={rotate=45}, code={\fill (0,0) circle (1pt); \pgftext{x}}}";
    for (name, size) in [("st", 3.0), ("{st[psize=6pt]}", 6.0)] {
        let (c, w) = canvas_and_warnings(&format!(r"{declare} \fill[pattern={name}] (0,0) rectangle (1,1);"));
        assert!(w.is_empty(), "{name}: {w:?}");
        let (items, _, s, t) = tile(&c);
        assert!(close_to(s, (size * PT, size * PT)), "{name}: {s:?}");
        let r = tikra::scene::Transform::rotation(45.0);
        assert!(close(t.a, r.a) && close(t.b, r.b), "{t:?}");
        // the text of the tile is left as it is
        assert!(items.iter().any(|i| matches!(i, Item::Text { .. })));
    }
}

/// A pattern that cannot be drawn where it is used is warned about.
#[test]
fn pattern_that_cannot_be_drawn() {
    let declare = r"\tikzdeclarepattern{name=st, parameters={\psize}, defaults={psize/.store in=\psize, psize=3pt},
        tile size={(\psize,\psize)}, code={\fill (0,0) circle (1pt);}}";
    let (_, w) = canvas_and_warnings(&format!(r"{declare} \fill[pattern={{st[psize=(]}}] (0,0) rectangle (1,1);"));
    assert!(w.iter().any(|a| a.contains("pattern 'st'")), "{w:?}");
}

/// The meta patterns `Hatch` and `Stars`: a cross, a star of `points` points.
#[test]
fn hatch_and_stars() {
    let c = canvas(r"\fill[pattern=Hatch] (0,0) rectangle (1,1);");
    let (items, _, _, _) = tile(&c);
    let Item::Path { segments, .. } = &items[0] else { panic!() };
    assert_eq!(segments.iter().filter(|s| matches!(s, Segment::MoveTo(_))).count(), 2);
    let c = canvas(r"\fill[pattern={Stars[points=6, radius=2mm]}] (0,0) rectangle (1,1);");
    let (items, _, _, _) = tile(&c);
    let Item::Path { segments, .. } = &items[0] else { panic!() };
    // the top point, then 6 inner and 6 outer points
    assert_eq!(points(segments).len(), 13);
    assert!(close_to(points(segments)[0], (0.0, 0.2)), "{:?}", points(segments)[0]);
}

/// The patterns library's own patterns are left to the output, by name and color.
#[test]
fn library_pattern_by_name() {
    let (c, w) = canvas_and_warnings(r"\fill[pattern=north east lines, pattern color=red] (0,0) rectangle (1,1);");
    assert!(w.is_empty(), "{w:?}");
    let f = paths(&c).into_iter().find_map(|d| d.fill).unwrap();
    assert_eq!(f.paint, Paint::Pattern { name: "north east lines".into(), color: RED });
}

/// A meta pattern ignores options it does not know.
#[test]
fn meta_pattern_unknown_option() {
    let (c, w) = canvas_and_warnings(r"\fill[pattern={Lines[distance=4pt, color=red]}] (0,0) rectangle (1,1);");
    assert!(w.is_empty(), "{w:?}");
    let (_, _, size, _) = tile(&c);
    assert!(close_to(size, (4.0 * PT, 4.0 * PT)), "{size:?}");
}

/// `\pgfsetfillpattern{name}{color}`: the basic layer's fills use the pattern, in the color.
#[test]
fn set_fill_pattern() {
    let (c, w) = canvas_and_warnings(r"\pgfsetfillpattern{Dots}{red} \pgfpathrectangle{\pgfpointorigin}{\pgfpoint{1cm}{1cm}} \pgfusepath{fill}");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(tile_colors(&tile(&c).0), vec![RED]);
    // a color it does not know is black
    let c = canvas(r"\pgfsetfillpattern{Dots}{nocolor} \pgfpathrectangle{\pgfpointorigin}{\pgfpoint{1cm}{1cm}} \pgfusepath{fill}");
    assert_eq!(tile_colors(&tile(&c).0), vec![Color::BLACK]);
}

// ---------------------------------------------------------------------------------------------
// old-style arrow tips

/// `\pgfarrowsdeclare`: the tip is drawn as its code draws it, at the end of the line.
#[test]
fn arrows_declare() {
    let body = r"\pgfarrowsdeclare{tri}{tri}{\pgfarrowsleftextend{-2pt}\pgfarrowsrightextend{1pt}}
        {\pgfpathmoveto{\pgfpoint{-2pt}{-2pt}}\pgfpathlineto{\pgfpoint{1pt}{0pt}}\pgfpathlineto{\pgfpoint{-2pt}{2pt}}\pgfusepathqfill \pgftext{x}}
        \draw[-tri] (0,0) -- (2,0);";
    let (c, w) = canvas_and_warnings(body);
    assert!(w.is_empty(), "{w:?}");
    let p = paths(&c);
    let tip = p.iter().find(|d| d.fill.is_some()).expect("a filled tip");
    let (lo, hi) = bounds(&tip.segments);
    // 3pt long and 4pt high, its right end at the line's end
    assert!(close(hi.x - lo.x, 3.0 * PT) && close(hi.y - lo.y, 4.0 * PT), "{lo:?} {hi:?}");
    assert!((hi.x - 2.0).abs() < 2.0 * PT, "{hi:?}");
    // the text the code sets is not part of the tip
    assert!(texts(&c).is_empty());
    // a tip without extents is drawn too
    let body = r"\pgfarrowsdeclare{bar}{bar}{}{\pgfpathmoveto{\pgfpoint{0pt}{-2pt}}\pgfpathlineto{\pgfpoint{0pt}{2pt}}\pgfusepathqstroke}
        \draw[-bar] (0,0) -- (2,0);";
    let (c, w) = canvas_and_warnings(body);
    assert!(w.is_empty(), "{w:?}");
    assert!(paths(&c).len() >= 2);
}

// ---------------------------------------------------------------------------------------------
// plot marks

/// The paths a mark draws at the origin, with mark size 1cm: (filled, stroked) for each.
fn mark_paths(options: &str) -> (Vec<Drawn>, Vec<String>) {
    let (c, w) = canvas_and_warnings(&format!(r"\path plot[mark size=1cm, {options}] coordinates {{(0,0)}};"));
    (paths(&c), w)
}

/// Each of pgf's marks and the plotmarks library's: how many pieces, filled or stroked.
#[test]
fn every_mark() {
    // (name, [(filled, stroked)])
    let s = (false, true);
    let fs = (true, true);
    let f = (true, false);
    let cases: Vec<(&str, Vec<(bool, bool)>)> = vec![
        ("*", vec![fs]),
        ("+", vec![s]),
        ("x", vec![s]),
        ("o", vec![s]),
        ("-", vec![s]),
        ("|", vec![s]),
        ("Mercedes star", vec![s]),
        ("Mercedes star flipped", vec![s]),
        ("asterisk", vec![s]),
        ("star", vec![s]),
        ("10-pointed star", vec![s]),
        ("oplus", vec![s]),
        ("oplus*", vec![fs]),
        ("otimes", vec![s]),
        ("otimes*", vec![fs]),
        ("square", vec![s]),
        ("square*", vec![fs]),
        ("triangle", vec![s]),
        ("triangle*", vec![fs]),
        ("diamond", vec![s]),
        ("diamond*", vec![fs]),
        ("pentagon", vec![s]),
        ("pentagon*", vec![fs]),
        ("halfcircle", vec![f, s]),
        ("halfcircle*", vec![f, f, s]),
        ("halfdiamond*", vec![f, f, s]),
        ("halfsquare*", vec![f, f, s]),
        ("halfsquare right*", vec![f, f, s]),
        ("halfsquare left*", vec![f, f, s]),
        ("heart", vec![fs]),
    ];
    for (name, want) in cases {
        let (p, w) = mark_paths(&format!("mark={name}"));
        assert!(w.is_empty(), "{name}: {w:?}");
        let got: Vec<(bool, bool)> = p.iter().map(|d| (d.fill.is_some(), d.stroke.is_some())).collect();
        assert_eq!(got, want, "{name}");
    }
}

/// The marks' shapes, with radius 1cm.
#[test]
fn mark_shapes() {
    let outline = |name: &str| {
        let (p, _) = mark_paths(&format!("mark={name}"));
        let (lo, hi) = bounds(&p.last().unwrap().segments);
        ((lo.x, lo.y), (hi.x, hi.y))
    };
    let c30 = 3f64.sqrt() / 2.0;
    assert!(close_to(outline("square").0, (-1.0, -1.0)) && close_to(outline("square").1, (1.0, 1.0)));
    assert!(close_to(outline("triangle").0, (-c30, -0.5)) && close_to(outline("triangle").1, (c30, 1.0)), "{:?}", outline("triangle"));
    assert!(close_to(outline("diamond").0, (-0.75, -1.0)) && close_to(outline("diamond").1, (0.75, 1.0)), "{:?}", outline("diamond"));
    assert!(close(outline("pentagon").1 .1, 1.0) && close(outline("pentagon").1 .0, 18f64.to_radians().cos()), "{:?}", outline("pentagon"));
    assert!(close_to(outline("halfdiamond*").0, (-0.75, -1.0)), "{:?}", outline("halfdiamond*"));
    // the heart reaches 1.75 r down
    assert!(close(outline("heart").0 .1, -1.75), "{:?}", outline("heart"));
}

/// The upper half of a half-filled mark is white, in `mark color`, or not drawn with `none`.
#[test]
fn mark_color() {
    let color = |opts: &str| {
        let (p, w) = mark_paths(opts);
        assert!(w.is_empty(), "{opts}: {w:?}");
        p.iter().filter_map(|d| d.fill.as_ref().map(|f| f.paint.clone())).collect::<Vec<_>>()
    };
    let fill = |c: Color| Paint::Solid(c);
    let black = Color::BLACK;
    assert_eq!(color("mark=halfsquare*"), vec![fill(black), fill(Color::WHITE)]);
    assert_eq!(color("mark=halfsquare*, mark options={mark color=red}"), vec![fill(black), fill(RED)]);
    assert_eq!(color("mark=halfsquare*, mark options={mark color=nocolor}"), vec![fill(black), fill(Color::WHITE)]);
    assert_eq!(color("mark=halfsquare*, mark options={mark color=none}"), vec![fill(black)]);
}

/// `mark indices` and `mark repeat` choose the points that get a mark.
#[test]
fn chosen_points() {
    let marks = |opts: &str| {
        let c = canvas(&format!(r"\path plot[mark=square*, mark size=1pt, {opts}] coordinates {{(0,0) (1,0) (2,0) (3,0)}};"));
        paths(&c).iter().map(|d| (bounds(&d.segments).0.x + PT).round() as i64).collect::<Vec<_>>()
    };
    assert_eq!(marks("mark indices={2,4}"), vec![1, 3]);
    assert_eq!(marks("mark repeat=2"), vec![0, 2]);
    assert_eq!(marks("mark repeat=2, mark phase=2"), vec![1, 3]);
}

/// The marks count for the picture's size, half the line width around a stroked one.
#[test]
fn marks_count_for_the_size() {
    let (c, w) = drawn_and_warnings(r"\path plot[mark=square, mark size=1cm, line width=2pt] coordinates {(0,0)};");
    assert!(w.is_empty(), "{w:?}");
    let (lo, hi) = (c.min.unwrap(), c.max.unwrap());
    assert!(close(hi.x - lo.x, 2.0 + 2.0 * PT), "{lo:?} {hi:?}");
    let (c, _) = drawn_and_warnings(r"\path plot[mark=ball, mark size=1cm] coordinates {(0,0)};");
    let (lo, hi) = (c.min.unwrap(), c.max.unwrap());
    assert!(close(hi.x - lo.x, 2.0), "{lo:?} {hi:?}");
}

/// A mark nobody declared is warned about.
#[test]
fn unknown_mark() {
    let (p, w) = mark_paths("mark=nothing");
    assert!(p.is_empty());
    assert!(w.iter().any(|a| a.contains("plot mark 'nothing' not supported")), "{w:?}");
}

/// A label's glyphs, to compare labels by their text.
fn glyphs(l: &tikra::scene::Label) -> String {
    format!("{:?}", l.lines)
}

/// `mark=ball` is a shaded circle; `mark=text` sets `text mark` as a node.
#[test]
fn ball_and_text_marks() {
    let (c, w) = canvas_and_warnings(r"\path plot[mark=ball, mark size=1cm, mark indices={2}] coordinates {(5,5) (0,0)};");
    assert!(w.is_empty(), "{w:?}");
    let s = shaded(&c);
    assert_eq!(s.len(), 1);
    assert!(boxed(&s[0].0, (-1.0, -1.0), (1.0, 1.0)), "{:?}", bounds(&s[0].0));
    // without `text mark`, a `p` at each chosen point
    let (c, w) = canvas_and_warnings(r"\path plot[mark=text, mark indices={1}] coordinates {(0,0) (2,0)};");
    assert!(w.is_empty(), "{w:?}");
    let t = texts(&c);
    assert_eq!(t.len(), 1);
    assert_eq!(glyphs(&t[0].0), glyphs(&texts(&canvas(r"\node {p};"))[0].0));
}

/// A mark of `\pgfdeclareplotmark` runs its code at each chosen point.
#[test]
fn declared_mark() {
    let body = r"\pgfdeclareplotmark{dot}{\pgfpathcircle{\pgfpointorigin}{0.5cm}\pgfusepathqfill}
        \path plot[mark=dot, mark repeat=2] coordinates {(0,0) (1,0) (2,0)};";
    let (c, w) = canvas_and_warnings(body);
    assert!(w.is_empty(), "{w:?}");
    let centers: Vec<f64> = paths(&c).iter().map(|d| bounds(&d.segments).0.x + 0.5).collect();
    assert_eq!(centers.len(), 2);
    assert!(close(centers[0], 0.0) && close(centers[1], 2.0), "{centers:?}");
}

/// `\pgfuseplotmark` draws the mark at the origin of the coordinate system.
#[test]
fn use_plot_mark() {
    let (c, w) = canvas_and_warnings(r"\pgftransformshift{\pgfpoint{2cm}{1cm}} \pgfsetplotmarksize{1cm} \pgfuseplotmark{square}");
    assert!(w.is_empty(), "{w:?}");
    let p = paths(&c);
    assert_eq!(p.len(), 1);
    assert!(boxed(&p[0].segments, (1.0, 0.0), (3.0, 2.0)), "{:?}", bounds(&p[0].segments));
}
