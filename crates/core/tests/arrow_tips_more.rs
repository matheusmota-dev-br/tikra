//! More arrow tips: the harpoon, reversed and round forms of the barbs and hooks, the keys that
//! scale, slant and size a tip from its length, values that need evaluating, the older tips of
//! the `arrows` library, tips declared with `\pgfarrowsdeclare`, the `tips` modes, flexing on a
//! curve, and specifications that cannot be read.

use tikra::canvas::Item;
use tikra::scene::{Point, Segment};
use tikra::PT;

/// A path of the canvas: segments, filled, stroked.
type Drawn = (Vec<Segment>, bool, bool);

/// The paths of a figure (segments, filled, stroked) and its warnings, in the picture's own
/// coordinates (a fixed bounding box keeps them).
fn draw(body: &str) -> (Vec<Drawn>, Vec<String>) {
    let (c, w) = tikra::engine::run_document(&format!("\\begin{{tikzpicture}}[baseline=0pt]\\useasboundingbox (0,-5) rectangle (10,5);{body}\\end{{tikzpicture}}"))
        .unwrap_or_else(|e| panic!("{e}\n{body}"));
    let paths =
        c.items.iter().filter_map(|i| if let Item::Path { segments, fill, stroke } = i { Some((segments.clone(), fill.is_some(), stroke.is_some())) } else { None }).collect();
    (paths, w)
}

/// Every point of segments, control points included.
fn all_points(segs: &[Segment]) -> Vec<Point> {
    segs.iter()
        .flat_map(|s| match *s {
            Segment::MoveTo(p) | Segment::LineTo(p) => vec![p],
            Segment::CurveTo(a, b, p) => vec![a, b, p],
            Segment::Close => vec![],
        })
        .collect()
}

/// The box (min x, max x, min y, max y) of the tip's paths, in pt, for a horizontal line from
/// (0,0) to (3,0) with the end tip `spec` (the line is the first path).
fn tip_box(spec: &str) -> (f64, f64, f64, f64) {
    let (p, w) = draw(&format!(r"\draw[line width=1pt, -{{{spec}}}] (0,0) -- (3,0);"));
    assert!(w.is_empty(), "{spec}: {w:?}");
    assert!(p.len() > 1, "{spec}: no tip");
    let pts: Vec<Point> = p[1..].iter().flat_map(|d| all_points(&d.0)).collect();
    let f = |g: fn(&Point) -> f64, min: bool| pts.iter().map(g).fold(if min { f64::MAX } else { f64::MIN }, |a, b| if min { a.min(b) } else { a.max(b) });
    (f(|p| p.x, true) / PT, f(|p| p.x, false) / PT, f(|p| p.y, true) / PT, f(|p| p.y, false) / PT)
}

/// The tip drawings of a spec, for comparing two specs.
fn tip_paths(spec: &str) -> String {
    let (p, w) = draw(&format!(r"\draw[line width=1pt, -{{{spec}}}] (0,0) -- (3,0);"));
    assert!(w.is_empty(), "{spec}: {w:?}");
    format!("{:?}", &p[1..])
}

fn close(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() < tol
}

/// A tip with its option list left open for more options: `Hooks[arc=120` → `Hooks[arc=120, `.
fn open_options(tip: &str) -> String {
    match tip.strip_suffix(']') {
        Some(t) => format!("{t}, "),
        None => format!("{tip}["),
    }
}

// ---------------------------------------------------------------- harpoons, reversed, round

/// The barbs are symmetric across the line; as harpoons only their left half is drawn (the
/// upper half, for a line going right), and `swap` draws the other one.
#[test]
fn harpoons_draw_one_half() {
    let kinds = [
        "Hooks",
        "Hooks[arc=120]",
        "Hooks[arc=200]",
        "Hooks[arc=300]",
        "Arc Barb",
        "Arc Barb[arc=200]",
        "Arc Barb[arc=260]",
        "Arc Barb[arc=320]",
        "Tee Barb",
        "Tee Barb[inset=1pt]",
        "Tee Barb[inset=-1pt]",
        "Tee Barb[length=2pt, inset=3pt]",
        "Classical TikZ Rightarrow",
        "Computer Modern Rightarrow",
        "Latex",
        "Stealth",
        "Rays",
    ];
    for k in kinds {
        let (_, _, y0, y1) = tip_box(k);
        assert!(close(y0, -y1, 0.5) && y1 > 1.0, "{k}: {y0} {y1}");
        for more in ["harpoon", "harpoon, reversed", "left", "harpoon, round"] {
            let spec = format!("{}{more}]", open_options(k));
            let (_, _, y0, y1) = tip_box(&spec);
            assert!(!close(y0, -y1, 0.6) && y1 > 1.0 && (y0 > -1.0 || k.contains("Hooks[arc=")), "{spec}: {y0} {y1}");
        }
        let spec = format!("{}right]", open_options(k));
        let (_, _, y0, y1) = tip_box(&spec);
        assert!(y0 < -1.0 && (y1 < 1.0 || k.contains("Hooks[arc=")), "{spec}: {y0} {y1}");
    }
}

/// Reversed and round forms of the tips: still symmetric; reversed ones differ from the plain tip
/// (a round cap or join only changes how the tip's line is stroked).
#[test]
fn reversed_and_round_tips() {
    for k in ["Hooks", "Hooks[arc=180]", "Hooks[arc=270]", "Arc Barb", "Arc Barb[arc=100]", "Classical TikZ Rightarrow", "Computer Modern Rightarrow", "Latex", "Stealth", "Rays"] {
        let open = open_options(k);
        let plain = tip_paths(k);
        for more in ["reversed", "round", "reversed, round"] {
            let spec = format!("{open}{more}]");
            if more.starts_with("reversed") {
                assert_ne!(tip_paths(&spec), plain, "{spec}");
            }
            let (_, _, y0, y1) = tip_box(&spec);
            assert!(close(y0, -y1, 0.5), "{spec}: {y0} {y1}");
        }
    }
    // a reversed hook opens toward the end of the line: it reaches past the plain one
    let (_, x1, ..) = tip_box("Hooks");
    let (_, r1, ..) = tip_box("Hooks[reversed]");
    assert!(r1 > x1, "{x1} {r1}");
}

/// Reversed tips on a double line end the line inside them.
#[test]
fn reversed_tips_on_double_lines() {
    for spec in ["Latex[reversed]", "Stealth[reversed]"] {
        let (p, w) = draw(&format!(r"\draw[double, -{{{spec}}}] (0,0) -- (3,0);"));
        assert!(w.is_empty(), "{w:?}");
        // the two strokes of the double line, then the tip
        assert!(p.len() >= 2, "{spec}: {p:?}");
        let end = all_points(&p[0].0).iter().map(|q| q.x).fold(f64::MIN, f64::max);
        assert!(end < 3.0 && end > 2.8, "{spec}: {end}");
    }
}

// ---------------------------------------------------------------- keys

/// `scale length`, `scale width`, `slant`, `width'`, `inset'`, `line width'`, `angle'` and
/// `cap angle` size the tip as pgf's keys do.
#[test]
fn sizing_keys() {
    let (x0, x1, y0, y1) = tip_box("Latex[length=4pt, width=4pt]");
    let (lx0, lx1, ly0, ly1) = tip_box("Latex[length=4pt, width=4pt, scale length=2]");
    assert!(close(lx1 - lx0, 2.0 * (x1 - x0), 0.6) && close(ly1 - ly0, y1 - y0, 0.6), "{x0} {x1} {lx0} {lx1}");
    let (_, _, wy0, wy1) = tip_box("Latex[length=4pt, width=4pt, scale width=2]");
    assert!(close(wy1 - wy0, 2.0 * (y1 - y0), 0.6), "{y0} {y1} {wy0} {wy1}");
    // slanted: the top corner moves forward
    assert_ne!(tip_paths("Kite[slant=0.5]"), tip_paths("Kite"));
    // the width from the length: `width'=0pt 1` is as wide as long
    let (x0, x1, y0, y1) = tip_box("Kite[length=6pt, width'=0pt 1, line width=0pt]");
    assert!(close(x1 - x0, y1 - y0, 0.2), "{x0} {x1} {y0} {y1}");
    // `angle'=90`: twice as wide as long
    let (x0, x1, y0, y1) = tip_box("Straight Barb[length=3pt, angle'=90, line width=0pt]");
    assert!(close(y1 - y0, 2.0 * (x1 - x0), 0.3), "{x0} {x1} {y0} {y1}");
    // `inset'` and `line width'` from the length
    assert_ne!(tip_paths("Stealth[length=6pt, inset'=0pt 0.5]"), tip_paths("Stealth[length=6pt]"));
    assert_ne!(tip_paths("Straight Barb[length=6pt, line width'=0pt 0.2]"), tip_paths("Straight Barb[length=6pt]"));
    // `cap angle` makes a triangle cap longer as it gets sharper
    let (a1, ..) = tip_box("Triangle Cap[cap angle=60]");
    let (b1, ..) = tip_box("Triangle Cap[cap angle=120]");
    assert!(a1 < b1, "{a1} {b1}");
}

/// Values that are expressions or in other units are evaluated: `2*3pt` is `6pt`, `1+1` is 2.
#[test]
fn evaluated_values() {
    assert_eq!(tip_paths("Latex[length=2*3pt]"), tip_paths("Latex[length=6pt]"));
    assert_eq!(tip_paths("Latex[scale=1+1]"), tip_paths("Latex[scale=2]"));
    // an em of the 10pt font
    let (a0, a1, ..) = tip_box("Latex[length=1em]");
    let (b0, b1, ..) = tip_box("Latex[length=10pt]");
    assert!(close(a1 - a0, b1 - b0, 1e-2), "{a0} {a1} {b0} {b1}");
}

/// A width of zero makes a flat tip; `fill` and `color` that are not colors are reported, and a
/// color makes the tip that color.
#[test]
fn widths_and_colors() {
    let (_, _, y0, y1) = tip_box("Latex[width=0pt, line width=0pt]");
    assert!(y1 - y0 < 1e-6, "{y0} {y1}");
    let (_, w) = draw(r"\draw[-{Latex[fill=nocolor]}] (0,0) -- (1,0);");
    assert!(w.iter().any(|w| w.contains("arrow tip fill 'nocolor' is not a color")), "{w:?}");
    let (_, w) = draw(r"\draw[-{Latex[color=nocolor]}] (0,0) -- (1,0);");
    assert!(w.iter().any(|w| w.contains("arrow tip color 'nocolor' is not a color")), "{w:?}");
    let (c, _) = tikra::engine::run_document(r"\begin{tikzpicture}\draw[-{Latex[color=red]}] (0,0) -- (1,0);\end{tikzpicture}").unwrap();
    let red = c.items.iter().any(|i| matches!(i, Item::Path { fill: Some(f), .. } if format!("{:?}", f.paint).contains("r: 1.0, g: 0.0")));
    assert!(red, "{:?}", c.items);
    // a color with braces in the option list: half red, half blue
    let (c, _) = tikra::engine::run_document(r"\begin{tikzpicture}\draw[-{Latex[color={red!50!blue}]}] (0,0) -- (1,0);\end{tikzpicture}").unwrap();
    let mix = c.items.iter().any(|i| matches!(i, Item::Path { fill: Some(f), .. } if format!("{:?}", f.paint).contains("r: 0.5, g: 0.0, b: 0.5")));
    assert!(mix, "{:?}", c.items);
}

// ---------------------------------------------------------------- the `arrows` library

/// Older tips: open triangles (pointing and reversed), the reversed triangle cap, the bar.
#[test]
fn older_tips() {
    for spec in ["open triangle 90", "open triangle 60 reversed", "open triangle 45 reversed", "triangle 90 cap", "triangle 90 cap reversed", "@bar"] {
        let (p, w) = draw(&format!(r"\usetikzlibrary{{arrows}}\draw[-{{{spec}}}] (0,0) -- (3,0);"));
        assert!(w.is_empty(), "{spec}: {w:?}");
        assert_eq!(p.len(), 2, "{spec}");
        let pts = all_points(&p[1].0);
        // drawn about the end of the line, symmetric across it
        let (y0, y1) = pts.iter().fold((f64::MAX, f64::MIN), |(a, b), q| (a.min(q.y), b.max(q.y)));
        assert!(close(y0, -y1, 1e-9) && y1 > 0.0, "{spec}: {pts:?}");
        assert!(pts.iter().all(|q| q.x > 2.5 && q.x < 3.2), "{spec}: {pts:?}");
    }
    // the open triangles are stroked, not filled
    let (p, _) = draw(r"\usetikzlibrary{arrows}\draw[-{open triangle 90}] (0,0) -- (3,0);");
    assert!(!p[1].1 && p[1].2);
}

/// A tip declared with `\pgfarrowsdeclare`: its drawing at the end, the line pulled back by its
/// left extent.
#[test]
fn tips_declared_the_old_way() {
    let (p, w) = draw(
        r"\pgfarrowsdeclare{dot}{dot}{\pgfarrowsleftextend{-2pt}\pgfarrowsrightextend{2pt}}{\pgfpathcircle{\pgfpointorigin}{2pt}\pgfusepathqfill}
          \draw[-dot] (0,0) -- (3,0);",
    );
    assert!(w.is_empty(), "{w:?}");
    let dot = &p[1];
    assert!(dot.1, "filled");
    let pts = all_points(&dot.0);
    let (x0, x1) = pts.iter().fold((f64::MAX, f64::MIN), |(a, b), q| (a.min(q.x), b.max(q.x)));
    assert!(close((x0 + x1) / 2.0, 3.0 - 2.0 * PT, 1e-4) && close(x1 - x0, 4.0 * PT, 1e-4), "{x0} {x1}");
}

// ---------------------------------------------------------------- tips modes and flexing

/// `tips=on proper draw` leaves out the tips of a path that stays at one point; `tips=proper`
/// draws them on a path that is not stroked.
#[test]
fn tips_modes() {
    let (p, _) = draw(r"\draw[->, tips=on proper draw] (1,1) -- (1,1);");
    assert_eq!(p.len(), 1);
    let (p, _) = draw(r"\draw[->] (1,1) -- (1,1);");
    assert_eq!(p.len(), 2);
    let (p, _) = draw(r"\path[->, tips=proper] (0,0) -- (1,0);");
    assert_eq!(p.len(), 1, "the tip only");
    let (_, w) = draw(r"\draw[->, tips=sometimes] (0,0) -- (1,0);");
    assert!(w.iter().any(|w| w.contains("sometimes")), "{w:?}");
    let (p, _) = draw(r"\path[->, tips=proper] (1,1) -- (1,1);");
    assert!(p.is_empty());
}

/// On a curve, `flex'` puts the tip's own ends on it, `quick` and `bend` its visual ends.
#[test]
fn flexed_tips_on_a_curve() {
    let tips: Vec<String> = ["flex'", "flex'=0.5", "quick", "bend", "flex"]
        .iter()
        .map(|f| {
            let (p, w) = draw(&format!(r"\usetikzlibrary{{bending}}\draw[-{{Computer Modern Rightarrow[length=5mm, {f}]}}] (0,0) to[bend left=60] (2,0);"));
            assert!(w.is_empty(), "{f}: {w:?}");
            format!("{:?}", p[1..].to_vec())
        })
        .collect();
    assert_ne!(tips[0], tips[1]);
    assert_ne!(tips[0], tips[4]);
}

// ---------------------------------------------------------------- reading specifications

/// Names in braces, a stray option list, a tip spec that ends with a dot, an unclosed option
/// list, shorthands that refer to each other, and unknown tips.
#[test]
fn reading_specifications() {
    assert_eq!(tip_paths("{Latex}[length=3mm]"), tip_paths("Latex[length=3mm]"));
    assert_eq!(tip_paths("Latex[length=3mm] [red]"), tip_paths("Latex[length=3mm]"));
    // a dot at the end marks where the line ends: the same two tips are drawn
    let (p, w) = draw(r"\draw[line width=1pt, ->>.] (0,0) -- (3,0);");
    let (q, _) = draw(r"\draw[line width=1pt, ->>] (0,0) -- (3,0);");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(format!("{:?}", &p[1..]), format!("{:?}", &q[1..]));
    let end = all_points(&p[0].0).iter().map(|q| q.x).fold(f64::MIN, f64::max);
    assert!(end < 3.0, "{end}");
    for (body, fragment) in [
        (r"\tikzset{unclosed/.tip={Latex[length=3mm}}\draw[-unclosed] (0,0) -- (1,0);", "option '-unclosed' not supported"),
        (r"\tikzset{ping/.tip=pong, pong/.tip=ping}\draw[-ping] (0,0) -- (1,0);", "not supported"),
        (r"\draw[-{Nonsense}] (0,0) -- (1,0);", "not supported"),
        (r"\draw[->, >=Nonsense] (0,0) -- (1,0);", "arrow tip '>' not supported"),
    ] {
        let (p, w) = draw(body);
        assert!(w.iter().any(|w| w.contains(fragment)), "{body}: {w:?}");
        assert_eq!(p.len(), 1, "{body}: the line only");
    }
}
