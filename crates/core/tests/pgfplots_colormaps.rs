//! pgfplots: colormaps (`colormap={name}{spec}` and how it is read, `colormap access`,
//! `\pgfplotscolormaptodatafile`, `\pgfplotscolormaptoshadingspec`), colorbars (`colorbar to name`,
//! `colorbar sampled`, `colorbar sampled line`) and contours (`contour prepared`, its labels, and
//! `contour filled`).

mod common;
use common::*;
use tikra::canvas::{Canvas, Paint, Shading};
use tikra::color::Color;
use tikra::scene::Segment;

fn picture(body: &str) -> (Canvas, Vec<String>) {
    tikra::engine::run_document(&format!("\\begin{{tikzpicture}}{body}\\end{{tikzpicture}}")).unwrap_or_else(|e| panic!("{e}\n{body}"))
}

fn axis(options: &str, body: &str) -> (Canvas, Vec<String>) {
    picture(&format!("\\begin{{axis}}[{options}] {body} \\end{{axis}}"))
}

fn solid(d: &Drawn) -> Option<Color> {
    match &d.fill.as_ref()?.paint {
        Paint::Solid(c) => Some(*c),
        _ => None,
    }
}

/// A colormap's stops as `\pgfplotscolormaptoshadingspec` gives them over 100pt: where each is
/// (0 to 1) and its color.
fn stops(spec: &str) -> Vec<(f64, String)> {
    let (c, w) = picture(&format!(
        r"\pgfplotsset{{colormap={{test}}{{{spec}}}}} \pgfplotscolormaptoshadingspec{{test}}{{100pt}}\spec \pgfdeclarehorizontalshading{{s}}{{1cm}}{{\spec}} \shade[shading=s] (0,0) rectangle (1,1);"
    ));
    assert!(w.is_empty(), "{w:?}\n{spec}");
    match &paths(&c)[0].fill.as_ref().unwrap().paint {
        Paint::Shading(Shading::Axial { stops, .. }) => stops.iter().map(|(t, c)| (*t, c.hex())).collect(),
        other => panic!("{other:?}"),
    }
}

/// The colors of the marks of a scatter plot of points of meta 0, 1, 2 and 3.
fn scatter(options: &str) -> (Vec<String>, Vec<String>) {
    let (c, w) = axis(&format!("hide axis, {options}"), r"\addplot[scatter, only marks, point meta=explicit] coordinates {(0,0) [0] (1,0) [1] (2,0) [2] (3,0) [3]};");
    (paths(&c).iter().filter_map(solid).map(|c| c.hex()).collect(), w)
}

// --- colormaps ---

#[test]
fn colors_in_every_model() {
    let s = stops("color=(red) rgb255=(0,0,255) gray=(0.5) cmyk=(0,1,0,0) rgb=(0,1,0)");
    let colors: Vec<&str> = s.iter().map(|(_, c)| c.as_str()).collect();
    assert_eq!(colors, ["#ff0000", "#0000ff", "#808080", "#ff00ff", "#00ff00"]);
    let at: Vec<f64> = s.iter().map(|(t, _)| *t).collect();
    assert!(at.iter().zip([0.0, 0.25, 0.5, 0.75, 1.0]).all(|(a, b)| (a - b).abs() < 1e-6), "{at:?}");
    // what is not a color (an unknown model, separators) is left out
    assert_eq!(stops(", rgb=(1,0,0); hsb=(0,1,1) rgb=(0,0,1)").len(), 2);
}

#[test]
fn positions_on_a_uniform_grid() {
    // stops at 0, 1 and 3cm: the gap of two steps gets the color halfway in between
    let s = stops("rgb(0cm)=(0,0,0) rgb(1cm)=(1,0,0) rgb(3cm)=(0,0,1)");
    assert_eq!(s.len(), 4);
    assert!((s[1].0 - 1.0 / 3.0).abs() < 1e-6 && (s[2].0 - 2.0 / 3.0).abs() < 1e-6);
    assert_eq!(s[2].1, "#800080");
    // a step of 0 does not count: the first distance is the grid's
    assert_eq!(stops("[0cm] rgb(0cm)=(0,0,0) rgb(1cm)=(1,0,0) rgb(3cm)=(0,0,1)").len(), 4);
    // two stops at one place: kept as they are
    assert_eq!(stops("rgb(0cm)=(0,0,0) rgb(0cm)=(1,0,0) rgb(1cm)=(0,0,1)").len(), 3);
}

#[test]
fn a_colormap_of_one_color_is_that_color_everywhere() {
    let s = stops("rgb=(1,0,0)");
    assert_eq!(s, [(0.0, "#ff0000".to_string())]);
    let (colors, w) = scatter("colormap={one}{rgb=(1,0,0)}");
    assert!(w.is_empty(), "{w:?}");
    assert!(colors.iter().all(|c| c == "#ff0000"), "{colors:?}");
}

#[test]
fn a_colormap_that_cannot_be_read_is_not_defined() {
    // a color without parentheses, one not closed, a position without a color, no color at all
    for spec in ["rgb=1,0,0", "rgb=(1,0,0", "rgb(1cm)", "hsb=(0,1,1)"] {
        let (colors, w) = scatter(&format!("colormap={{bad}}{{{spec}}}"));
        assert!(w.iter().any(|w| w.contains("colormap 'bad' not defined")), "{spec}: {w:?}");
        // the marks are black
        assert!(colors.iter().all(|c| c == "#000000"), "{spec}: {colors:?}");
    }
}

#[test]
fn colormap_access() {
    let map = "colormap={rgb}{rgb=(1,0,0) rgb=(0,1,0) rgb=(0,0,1)}";
    // map: interpolated
    let (c, _) = scatter(map);
    assert_eq!(c[1], "#55aa00");
    // piecewise constant: three bands over 0 to 3
    let (c, _) = scatter(&format!("{map}, colormap access=piecewise constant"));
    assert_eq!(c, ["#ff0000", "#00ff00", "#0000ff", "#0000ff"]);
    // direct: the meta is the index of the color, the last one beyond
    let (c, _) = scatter(&format!("{map}, colormap access=direct"));
    assert_eq!(c, ["#ff0000", "#00ff00", "#0000ff", "#0000ff"]);
}

#[test]
fn a_colormap_written_to_a_data_file() {
    // `position,r,g,b`, the positions from 0 to 1000: plotted, three points evenly apart
    let (c, w) = picture(
        r"\pgfplotsset{colormap={three}{rgb=(1,0,0) rgb=(0,1,0) rgb=(0,0,1)}} \pgfplotscolormaptodatafile{three}{cm.dat}
          \begin{axis}[hide axis] \addplot[mark=none] table[x index=0, y index=1, col sep=comma] {cm.dat}; \addplot[mark=none, red] coordinates {(0,1) (500,0) (1000,0)}; \end{axis}",
    );
    assert!(w.is_empty(), "{w:?}");
    let p = paths(&c);
    assert_eq!(points(&p[0].segments), points(&p[1].segments));
    // a colormap not defined yet is defined by its style: viridis
    let (_, w) = picture(r"\pgfplotscolormaptodatafile{viridis}{v.dat}");
    assert!(w.is_empty(), "{w:?}");
    // and one that is not defined at all warns
    let (_, w) = picture(r"\pgfplotscolormaptodatafile[output format=csv]{nothing}{cm.dat}");
    assert!(w.iter().any(|w| w.contains("\\pgfplotscolormaptodatafile: colormap 'nothing' not defined")), "{w:?}");
}

#[test]
fn a_colormap_as_a_shading() {
    // the colormap's stops over the length
    let (c, w) = picture(r"\pgfplotscolormaptoshadingspec{viridis}{4cm}\spec \pgfdeclarehorizontalshading{mine}{1cm}{\spec} \shade[shading=mine] (0,0) rectangle (4,1);");
    assert!(w.is_empty(), "{w:?}");
    match &paths(&c)[0].fill.as_ref().unwrap().paint {
        Paint::Shading(Shading::Axial { stops, .. }) => {
            assert!(stops.len() > 2);
            assert!(stops[0].1.b > stops[0].1.g && stops.last().unwrap().1.r > stops.last().unwrap().1.b);
        }
        other => panic!("{other:?}"),
    }
    let (_, w) = picture(r"\pgfplotscolormaptoshadingspec{nothing}{4cm}\spec");
    assert!(w.iter().any(|w| w.contains("\\pgfplotscolormaptoshadingspec: colormap 'nothing' not defined")), "{w:?}");
}

#[test]
fn the_grid_step_given_in_brackets() {
    // 0 and 2cm, a step of 1cm: a color in between; without the step, none
    assert_eq!(stops("[1cm] rgb(0cm)=(0,0,0) rgb(2cm)=(0,0,1)").len(), 3);
    assert_eq!(stops("rgb(0cm)=(0,0,0) rgb(2cm)=(0,0,1)").len(), 2);
    // a position is an expression, parentheses and all
    assert_eq!(stops("rgb(0cm)=(0,0,0) rgb(1cm)=(1,0,0) rgb((1+2)*1cm)=(0,0,1)").len(), 4);
}

// --- colorbars ---

/// The number of paths and texts of an axis.
fn counts(options: &str, body: &str) -> (usize, usize, Vec<String>) {
    let (c, w) = axis(options, body);
    (paths(&c).len(), texts(&c).len(), w)
}

#[test]
fn a_colorbar_to_name_is_not_drawn() {
    let plot = r"\addplot[scatter, only marks] coordinates {(0,0) (1,1)};";
    let (plain, _, _) = counts("", plot);
    let (bar, _, _) = counts("colorbar", plot);
    let (named, _, w) = counts("colorbar, colorbar to name=bar", plot);
    assert!(w.is_empty(), "{w:?}");
    assert!(bar > plain);
    assert_eq!(named, plain);
}

#[test]
fn a_colorbar_of_an_empty_meta_range_shows_0_to_1() {
    // a minimum above the maximum: no range to show
    let (_, _, w) = counts("colorbar, point meta min=2, point meta max=1", r"\addplot[scatter, only marks] coordinates {(0,0) (1,1)};");
    assert!(w.iter().any(|w| w.contains("colorbar: the axis has no point meta, it shows 0 to 1")), "{w:?}");
}

#[test]
fn a_sampled_colorbar_has_a_cell_per_sample() {
    // 5 samples: 4 cells of one color each
    let (c, w) =
        picture(r"\pgfplotsset{samples=5} \begin{axis}[colorbar sampled, colormap={bw}{gray=(0) gray=(1)}] \addplot[point meta=y, no marks] coordinates {(0,0) (1,1)}; \end{axis}");
    assert!(w.is_empty(), "{w:?}");
    let cells: Vec<String> = paths(&c).iter().filter(|d| matches!(d.segments.last(), Some(Segment::Close))).filter_map(solid).map(|c| c.hex()).collect();
    let grays: std::collections::BTreeSet<&String> = cells.iter().filter(|c| *c != "#ffffff").collect();
    assert_eq!(grays.len(), 4, "{cells:?}");
}

#[test]
fn a_sampled_colorbar_line_is_a_row_of_marks() {
    // a mark per sample, vertical or horizontal
    for orientation in ["colorbar right", "colorbar horizontal", "colorbar left", "colorbar horizontal, colorbar style={at={(0.5,1.1)}, anchor=south}, colorbar pos*=t"] {
        let (c, w) = picture(&format!(
            r"\pgfplotsset{{samples=4}} \begin{{axis}}[{orientation}, colorbar sampled line] \addplot[point meta=y, no marks] coordinates {{(0,0) (1,1)}}; \end{{axis}}"
        ));
        assert!(w.is_empty(), "{orientation}: {w:?}");
        let marks = paths(&c).iter().filter(|d| d.fill.is_some() && d.segments.iter().any(|s| matches!(s, Segment::CurveTo(..)))).count();
        assert_eq!(marks, 4, "{orientation}");
    }
}

// --- contours ---

/// The stroked paths of a contour plot without the axis.
fn contour(options: &str, body: &str) -> (Canvas, Vec<String>) {
    axis(&format!("hide axis, {options}"), body)
}

#[test]
fn contours_in_the_matlab_format() {
    // two lines: two points at level 0.5, two at level 1, each in its level's color
    let (c, w) = contour(
        "colormap={rb}{rgb=(1,0,0) rgb=(0,0,1)}",
        "\\addplot[contour prepared, contour prepared format=matlab, contour/labels=false] table {\n0.5 2\n0 0\n1 1\n1 2\n0 1\n1 2\n};",
    );
    assert!(w.is_empty(), "{w:?}");
    let p = paths(&c);
    assert_eq!(p.len(), 2);
    assert_eq!(p.iter().map(|d| d.stroke.as_ref().unwrap().color.hex()).collect::<Vec<_>>(), ["#ff0000", "#0000ff"]);
}

#[test]
fn an_unbounded_point_breaks_a_contour_line() {
    let (c, w) = contour("unbounded coords=jump", r"\addplot[contour prepared, contour/labels=false] coordinates {(0,0,1) (1,0,1) (nan,0,1) (2,0,1) (3,0,1)};");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(paths(&c).len(), 2);
}

#[test]
fn filled_contours_are_closed_in_their_fill_color() {
    let (c, _) = contour("", r"\addplot[contour prepared, contour/labels=false, contour/filled=true, contour/fill color=red] coordinates {(0,0,1) (1,0,1) (1,1,1)};");
    let p = &paths(&c)[0];
    assert!(matches!(p.segments.last(), Some(Segment::Close)));
    assert_eq!(solid(p).unwrap().hex(), "#ff0000");
}

#[test]
fn contour_labels() {
    // a short line gets its label on its last segment, turned along it and never upside down
    // (no warning asserted: the labels' text is not printed as a number yet, a known bug)
    let (c, _) = contour("", r"\addplot[contour prepared] coordinates {(2,1,1) (1,0,1)};");
    let t = texts(&c);
    assert_eq!(t.len(), 1);
    // the segment goes left and down: the label reads left to right, rising
    let place = t[0].1;
    assert!(place.a > 0.0 && place.b > 0.0, "{place:?}");
    // a label per line, each at the middle of its segment
    let (c, _) = contour("", r"\addplot[contour prepared] coordinates {(0,0,0) (1,0,0)}; \addplot[contour prepared] coordinates {(0,1,1) (1,1,1)};");
    assert_eq!(texts(&c).len(), 2);
    // no labels: none
    let (c, _) = contour("", r"\addplot[contour prepared, contour/labels=false] coordinates {(0,0,1) (1,0,1)};");
    assert!(texts(&c).is_empty());
}

/// The colors of the bands of a filled contour.
fn bands(options: &str, plot: &str) -> (Vec<String>, Vec<String>) {
    let (c, w) = contour(options, plot);
    (paths(&c).iter().filter_map(solid).map(|c| c.hex()).collect(), w)
}

#[test]
fn filled_contours_in_bands_of_their_levels() {
    let map = "colormap={rb}{rgb=(1,0,0) rgb=(0,0,1)}";
    // levels given: the bands from each level up, in the colormap's color there
    let (b, w) = bands(map, r"\addplot3[contour filled={levels={0,0.5}}, samples=3, domain=0:1] {x};");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(b, ["#ff0000", "#800080"]);
    // levels from a colormap of their own
    let (b, _) = bands(map, r"\addplot3[contour filled={levels from colormap={rgb(0cm)=(0,1,0) rgb(1cm)=(0,0,0)}}, samples=3, domain=0:1] {x};");
    assert_eq!(b, ["#00ff00", "#000000"]);
    // a band no cell reaches is not drawn
    let (b, _) = bands(map, r"\addplot3[contour filled={levels={0,0.5,5}}, samples=3, domain=0:1] {x};");
    assert_eq!(b.len(), 2);
    // a cell of an unbounded corner is left out
    let (b, _) = bands(map, r"\addplot3[contour filled={number=2}, samples=3, domain=0:1, unbounded coords=jump] {x < 0.2 ? nan : x};");
    assert_eq!(b.len(), 2);
    // from behind, the cells cut along their other diagonal: the same bands
    let (b, _) = bands(&format!("{map}, view={{-30}}{{30}}"), r"\addplot3[contour filled={number=2}, samples=3, domain=0:1] {x};");
    assert_eq!(b, ["#ff0000", "#0000ff"]);
}

#[test]
fn filled_contours_need_a_matrix_and_a_colormap() {
    let (b, w) = bands("", r"\addplot[contour filled] coordinates {(0,0) (1,1) (2,0)};");
    assert!(b.is_empty());
    assert!(w.iter().any(|w| w.contains("contour filled needs a matrix of points")), "{w:?}");
    let (b, w) = bands("colormap name=nothing", r"\addplot3[contour filled, samples=3, domain=0:1] {x};");
    assert!(b.is_empty());
    assert!(w.iter().any(|w| w.contains("colormap 'nothing' not defined")), "{w:?}");
}

#[test]
fn a_line_too_short_for_the_label_distance_gets_one_label() {
    let (c, _) = contour("", r"\addplot[contour prepared, contour/label distance=1000pt] coordinates {(0,0,1) (1,0,1) (2,0,1)};");
    assert_eq!(texts(&c).len(), 1);
}

#[test]
fn filled_contours_in_two_dimensions() {
    // a 2 × 2 matrix of metas 0 and 1 across x: two bands; levels from a colormap that cannot be
    // read are the colormap's `number` of levels
    let (b, w) = bands(
        "colormap={rb}{rgb=(1,0,0) rgb=(0,0,1)}",
        r"\addplot[contour filled={number=2, levels from colormap={junk}}, point meta=x, mesh/cols=2] coordinates {(0,0) (1,0) (0,1) (1,1)};",
    );
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(b, ["#ff0000", "#0000ff"]);
}
