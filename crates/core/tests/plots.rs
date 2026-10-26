//! pgfplots: the axis' size, its ticks and labels, plots by expression, coordinates and table,
//! the cycle list and marks, bars, legends, log axes and TikZ statements inside an axis.

mod common;
use common::*;
use tikra::canvas::{Canvas, Item, Paint};
use tikra::scene::Segment;

const PT: f64 = tikra::PT;

fn canvas(body: &str) -> (Canvas, Vec<String>) {
    tikra::engine::run_document(&format!("\\begin{{tikzpicture}}{body}\\end{{tikzpicture}}")).unwrap_or_else(|e| panic!("{e}\n{body}"))
}

fn axis(options: &str, body: &str) -> (Canvas, Vec<String>) {
    canvas(&format!("\\begin{{axis}}[{options}] {body} \\end{{axis}}"))
}

/// Width and height of the drawing, in pt.
fn size(c: &Canvas) -> (f64, f64) {
    let (a, b) = (c.min.unwrap(), c.max.unwrap());
    ((b.x - a.x) / PT, (b.y - a.y) / PT)
}

fn texts(c: &Canvas) -> usize {
    c.items.iter().filter(|i| matches!(i, Item::Text { .. })).count()
}

fn filled(c: &Canvas) -> Vec<&Vec<Segment>> {
    c.items.iter().filter_map(|i| if let Item::Path { segments, fill: Some(_), .. } = i { Some(segments) } else { None }).collect()
}

#[test]
fn default_size_is_pgfplots() {
    // 240pt × 207pt with the labels: TeX draws this one 233.6pt × 182.9pt (empty labels included)
    let (c, w) = axis("", r"\addplot+[sharp plot] coordinates {(0,0) (1,2) (2,3)};");
    assert!(w.is_empty(), "{w:?}");
    let (wd, ht) = size(&c);
    assert!((wd - 233.6).abs() < 2.0 && (ht - 182.9).abs() < 2.0, "{wd} × {ht}");
    // scale only axis: the box is exactly the size
    let (c, _) = axis("scale only axis, width=5cm, height=3cm, hide axis", r"\addplot coordinates {(0,0) (1,1)};");
    let (wd, ht) = size(&c);
    assert!(wd >= 5.0 / 2.54 * 72.27 - 1.0, "{wd} {ht}");
}

#[test]
fn axis_line_styles() {
    for lines in ["left", "middle", "center", "none", "box", "right"] {
        let (c, w) = axis(&format!("xmin=-1, xmax=1, ymin=-1, ymax=1, axis lines={lines}"), r"\addplot {x};");
        assert!(w.is_empty(), "{lines}: {w:?}");
        assert!(!c.items.is_empty(), "{lines}");
    }
}

#[test]
fn ticks_follow_the_algorithm() {
    // x from -6 to 6 (domain -5:5 enlarged by 10%): ticks every 2, seven labels; y: 0, 10, 20 (as TeX draws it)
    let (c, w) = axis("", r"\addplot[samples=3] {x^2};");
    assert!(w.is_empty(), "{w:?}");
    // 7 x labels, 3 y labels, and the two empty label nodes draw nothing
    assert_eq!(texts(&c), 10, "{}", texts(&c));
    // explicit ticks and labels
    let (c, w) = axis("xtick={0,1,2}, xticklabels={a,b,c}, ytick=\\empty", r"\addplot coordinates {(0,0) (2,1)};");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(texts(&c), 3);
}

#[test]
fn expressions_tables_and_coordinates() {
    let (_, w) =
        axis("domain=0:2, samples=5, xlabel={$x$}, ylabel={$y$}, title={Title}", r"\addplot[red, thick] {x^2}; \addplot+[domain=-1:1, samples=3] {1/x}; \addplot (x, {sqrt(x)});");
    assert!(w.is_empty(), "{w:?}");
    let (c, w) = axis("", "\\addplot table {\nx y\n0 1\n1 3\n2 2\n};");
    assert!(w.is_empty(), "{w:?}");
    // three marks (filled circles) of the first cycle list entry
    assert!(!filled(&c).is_empty());
    let (_, w) = axis("", r"\addplot table[x=b, y expr=\thisrow{a}*2, row sep=\\] {a b \\ 1 2 \\ 3 4 \\};");
    assert!(w.is_empty(), "{w:?}");
}

#[test]
fn cycle_list_and_marks() {
    let (c, _) = axis("", r"\addplot coordinates {(0,0) (1,1)}; \addplot coordinates {(0,1) (1,0)};");
    let svg = tikra::driver::svg(&c, "", &Default::default());
    // blue line, red line; marks filled blue!80!black and red!80!black
    assert!(svg.contains("#0000ff") && svg.contains("#ff0000"), "{svg}");
    assert!(svg.contains("#0000cc") && svg.contains("#cc0000"), "{svg}");
    // a plot with its own options does not take marks from the cycle list
    let (c, _) = axis("", r"\addplot[blue] coordinates {(0,0) (1,1)};");
    assert!(filled(&c).is_empty());
    let (c, _) = axis("", r"\addplot[only marks, mark=square*] coordinates {(0,0) (1,1) (2,0)};");
    assert!(!filled(&c).is_empty());
}

#[test]
fn bars_and_closed_cycles() {
    let (c, w) = axis("ybar, ymin=0", r"\addplot coordinates {(0,3) (1,2) (2,4)};");
    assert!(w.is_empty(), "{w:?}");
    let bars = filled(&c);
    assert_eq!(bars.len(), 1, "the bars are one path");
    assert_eq!(bars[0].iter().filter(|s| matches!(s, Segment::Close)).count(), 3);
    let (c, w) = axis("", r"\addplot[fill=blue] {x^2} \closedcycle;");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(filled(&c).len(), 1);
}

#[test]
fn legend_frame_and_positions() {
    let legend_box = |opts: &str| {
        let (c, w) = axis(&format!("width=10cm, height=6cm, xmin=0, xmax=1, ymin=0, ymax=1, {opts}"), r"\addplot coordinates {(0,0) (1,1)}; \addlegendentry{data}");
        assert!(w.is_empty(), "{opts}: {w:?}");
        let frame = c
            .items
            .iter()
            .find_map(|i| match i {
                Item::Path { segments, fill: Some(f), .. } if matches!(f.paint, Paint::Solid(col) if col == tikra::color::Color::WHITE) => Some(points(segments)),
                _ => None,
            })
            .expect("a white legend frame");
        let xs: Vec<f64> = frame.iter().map(|p| p.0).collect();
        let ys: Vec<f64> = frame.iter().map(|p| p.1).collect();
        (
            xs.iter().cloned().fold(f64::INFINITY, f64::min),
            ys.iter().cloned().fold(f64::INFINITY, f64::min),
            xs.iter().cloned().fold(f64::MIN, f64::max),
            ys.iter().cloned().fold(f64::MIN, f64::max),
        )
    };
    let (_, _, right, top) = legend_box("");
    let (left_nw, _, _, _) = legend_box("legend pos=north west");
    let (_, bottom_se, right_se, _) = legend_box("legend pos=south east");
    let (left_out, _, _, _) = legend_box("legend pos=outer north east");
    assert!(left_nw < right / 2.0, "{left_nw}");
    assert!(bottom_se < top / 2.0 && right_se < right, "{bottom_se} {right_se}");
    assert!(left_out > right, "outer north east is outside the box: {left_out} {right}");
    // \legend and legend entries
    let (_, w) = axis("legend entries={a,b}", r"\addplot {x}; \addplot {2*x};");
    assert!(w.is_empty(), "{w:?}");
    let (_, w) = axis("", r"\addplot {x}; \addplot {2*x}; \legend{$x$,$2x$}");
    assert!(w.is_empty(), "{w:?}");
}

#[test]
fn log_axes() {
    for env in ["semilogyaxis", "semilogxaxis", "loglogaxis"] {
        let (c, w) = canvas(&format!(r"\begin{{{env}}} \addplot coordinates {{(1,1) (10,100) (100,1000)}}; \end{{{env}}}"));
        assert!(w.is_empty(), "{env}: {w:?}");
        assert!(texts(&c) >= 3, "{env}");
    }
    let (_, w) = axis("ymode=log, xmin=1, xmax=10", r"\addplot[domain=1:10] {exp(x)};");
    assert!(w.is_empty(), "{w:?}");
}

#[test]
fn tikz_statements_inside_an_axis() {
    let (c, w) = axis("xmin=0, xmax=10, ymin=0, ymax=10", r"\draw[red] (axis cs:0,0) -- (axis cs:10,10); \node at (rel axis cs:0.5,0.5) {mid};");
    assert!(w.is_empty(), "{w:?}");
    assert!(texts(&c) >= 1);
    // compat=1.11 on: plain coordinates are axis coordinates
    let (_, w) = axis("compat=1.18, xmin=0, xmax=1, ymin=0, ymax=1", r"\draw (0,0) -- (1,1); \coordinate (a) at (0.5,0.5);");
    assert!(w.is_empty(), "{w:?}");
    // named axes and their anchors
    let (_, w) = canvas(r"\begin{axis}[name=a] \addplot {x}; \end{axis} \draw (a.outer north east) -- (a.origin) -- (a.south west);");
    assert!(w.is_empty(), "{w:?}");
}

#[test]
fn degenerate_input_does_not_fail() {
    let (_, w) = axis("", "");
    assert!(w.iter().any(|x| x.contains("without data")), "{w:?}");
    let (_, w) = axis("xmin=2, xmax=2, ymin=1, ymax=1", r"\addplot coordinates {(2,1)};");
    assert!(w.is_empty(), "{w:?}");
    // the step vanishes in precision or the range is huge: no hang
    axis("xmin=1e17, xmax=1.0000000000000001e17, ymin=0, ymax=1", r"\addplot coordinates {(1e17,0)};");
    axis("xmin=0, xmax=1e-300, ymin=0, ymax=1", "");
    axis("ymin=-1e308, ymax=1e308", "");
    let (_, w) = axis("", r"\addplot coordinates;");
    assert!(!w.is_empty());
    let (_, w) = axis("", r"\addplot table {no-such-file.dat};");
    assert!(w.iter().any(|x| x.contains("not found")), "{w:?}");
    let (_, w) = axis("", r"\addplot table {../../etc/passwd};");
    assert!(w.iter().any(|x| x.contains("refused")), "{w:?}");
    // unknown keys are reported
    let (_, w) = axis("no such key=1", r"\addplot {x};");
    assert!(w.iter().any(|x| x.contains("no such key")), "{w:?}");
    // an axis outside tikzpicture is a picture of its own
    let r = tikra::convert(r"\begin{axis} \addplot {x}; \end{axis}").unwrap();
    assert!(r.svg.contains("<path"));
}

#[test]
fn three_d_axis_and_surface() {
    // view {25}{30}: x and y not enlarged, three ticks wanted at least (`every 3d description`)
    let (c, w) = axis("", r"\addplot3[surf,shader=interp] {x*y};");
    assert!(w.is_empty(), "{w:?}");
    // as TeX: x -4, -2, 0, 2, 4 (the longer axis on paper); y -5, 0, 5; z -20, 0, 20
    assert_eq!(texts(&c), 11);
    // 25 × 25 samples: 24 × 24 patches, each shaded as two triangles
    assert_eq!(filled(&c).len(), 2 * 24 * 24);
    // 3d box=complete draws the edges in front too: more lines than the background box alone
    let strokes = |c: &Canvas| c.items.iter().filter(|i| matches!(i, Item::Path { fill: None, stroke: Some(_), .. })).count();
    let (c2, w) = axis("3d box=complete", r"\addplot3[surf,shader=interp] {x*y};");
    assert!(w.is_empty(), "{w:?}");
    assert!(strokes(&c2) > strokes(&c), "{} {}", strokes(&c2), strokes(&c));
}

/// The gray of an axial shading at canvas point `p` (its stops interpolated, padded at the ends).
fn shading_gray(paint: &Paint, p: tikra::scene::Point) -> f64 {
    let Paint::Shading(tikra::canvas::Shading::Axial { transform, stops }) = paint else { panic!("{paint:?}") };
    let t = (transform.inverse().apply(p).y + 50.0) / 100.0;
    let at = |k: usize| (stops[k].0, stops[k].1.r);
    if t <= at(0).0 {
        return at(0).1;
    }
    for k in 1..stops.len() {
        let ((a, ca), (b, cb)) = (at(k - 1), at(k));
        if t <= b {
            return ca + (cb - ca) * (t - a) / (b - a).max(1e-12);
        }
    }
    at(stops.len() - 1).1
}

#[test]
fn interp_shades_each_triangle_smoothly() {
    // a cell of a surface is shaded as two triangles (a lattice shading), its color linear in the
    // point meta between the corners: a gradient on each triangle, however small, not one color
    let body = r"\addplot3[surf,shader=interp,samples=3,domain=0:1,y domain=0:1] {x+2*y};";
    let (c, w) = axis("colormap/blackwhite", body);
    assert!(w.is_empty(), "{w:?}");
    let tris: Vec<(&Vec<Segment>, &Paint)> =
        c.items.iter().filter_map(|i| if let Item::Path { segments, fill: Some(f), .. } = i { Some((segments, &f.paint)) } else { None }).collect();
    assert_eq!(tris.len(), 2 * 2 * 2);
    // at each corner, the gray of its meta: z = x + 2y in 0, 0.5, … 3, grays in steps of 1/6
    let mut grays = Vec::new();
    for (segs, paint) in &tris {
        assert!(matches!(paint, Paint::Shading(_)), "{paint:?}");
        for s in segs.iter() {
            if let Segment::MoveTo(p) | Segment::LineTo(p) = s {
                grays.push(shading_gray(paint, *p));
            }
        }
    }
    for g in &grays {
        assert!((g * 6.0 - (g * 6.0).round()).abs() < 1e-6, "{g} {grays:?}");
    }
    assert!(grays.iter().any(|g| g.abs() < 1e-6) && grays.iter().any(|g| (g - 1.0).abs() < 1e-6), "{grays:?}");
    // drawn without antialiasing, as viewers draw shadings: no seams between the triangles
    assert!(c.items.iter().any(|i| matches!(i, Item::Crisp)));
    let svg = tikra::convert(&format!("\\begin{{tikzpicture}}\\begin{{axis}}[colormap/blackwhite] {body} \\end{{axis}}\\end{{tikzpicture}}")).unwrap().svg;
    assert!(svg.contains(r#"shape-rendering="crispEdges""#));
    // `faceted interp`: the same shading, each patch's edges on top (antialiased, out of the group)
    let (c, _) = axis("colormap/blackwhite", &body.replace("shader=interp", "shader=faceted interp"));
    let kinds: Vec<char> = c
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Path { fill: Some(_), .. } => Some('s'),
            Item::Path { fill: None, stroke: Some(_), segments } if segments.len() == 5 => Some('e'),
            _ => None,
        })
        .collect();
    let s: String = kinds.into_iter().filter(|k| *k == 's' || *k == 'e').collect();
    assert!(s.contains("sse") && s.matches('e').count() >= 4, "{s}");
}

#[test]
fn interp_image_cells_shade_between_their_corners() {
    // `matrix plot, shader=interp`: the corners' metas as pgfplots finds them sweeping the
    // scanlines (pgfplots manual, the matrix with a nan in its middle)
    let metas = [0.0, 1.0, 2.0, 3.0, f64::NAN, 5.0, 6.0, 7.0, 8.0];
    let cells = tikra::engine::pgfplots::mesh::image_interp_metas(&metas, 3, 3, 0.0);
    // the first corner its own; the inner corner after a nan the mean of the four around
    // (nan as the lowest meta); the first row's from its centers; the last column extrapolated
    assert_eq!(cells[0], [0.0, 1.0, 1.0, 3.0]);
    assert_eq!(cells[2], [2.0, -1.0, 2.0, 5.0]);
    assert_eq!(cells[5], [5.0, 2.0, 5.0, 8.0]);
    assert_eq!(cells[6], [6.0, 7.0, 6.0, 5.0]);
    assert_eq!(cells[8], [8.0, 5.0, 8.0, 7.0]);
    // drawn as two shaded triangles each, the nan cell left out
    let (c, w) = axis(
        "colorbar=false",
        r"\addplot [matrix plot, shader=interp, point meta=explicit] coordinates {
        (0,0) [0] (1,0) [1] (2,0) [2]

        (0,1) [3] (1,1) [nan] (2,1) [5]

        (0,2) [6] (1,2) [7] (2,2) [8]
    };",
    );
    assert!(w.is_empty(), "{w:?}");
    let shaded = c.items.iter().filter(|i| matches!(i, Item::Path { fill: Some(f), .. } if matches!(f.paint, Paint::Shading(_)))).count();
    assert_eq!(shaded, 8 * 2);
}

#[test]
fn group_plots() {
    // 2 by 2: four axes side by side and below each other, 1cm apart
    let (c, w) = canvas(
        r"\begin{groupplot}[group style={group size=2 by 2, x descriptions at=edge bottom}, width=4cm, height=4cm, xtick=\empty, ytick=\empty, xlabel=t]
        \nextgroupplot \addplot coordinates {(0,0) (1,1)};
        \nextgroupplot \addplot coordinates {(0,0) (1,1)};
        \nextgroupplot \addplot coordinates {(0,0) (1,1)};
        \nextgroupplot \addplot coordinates {(0,0) (1,1)};
        \end{groupplot}",
    );
    assert!(w.is_empty(), "{w:?}");
    // the x labels of the bottom row only
    assert_eq!(texts(&c), 2);
    let (wd, ht) = size(&c);
    // two axes of 4cm less 45pt each and 1cm between them
    let box_pt = 4.0 / 2.54 * 72.27 - 45.0;
    // (with the empty y label node 35pt left of the first axis, as TeX draws it)
    assert!(wd > 2.0 * box_pt + 28.0 && wd < 2.0 * box_pt + 80.0, "{wd}");
    assert!(ht > 2.0 * box_pt + 28.0, "{ht}");
}

#[test]
fn nodes_near_coords() {
    // one node per point with its y value, above the points and below for negative ones
    let (c, w) = axis("xtick=\\empty, ytick=\\empty, nodes near coords", r"\addplot coordinates {(0,1) (1,-1) (2,0.5)};");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(texts(&c), 3);
    // symbolic point meta is shown as written
    let (c, w) = axis("xtick=\\empty, ytick=\\empty", r"\addplot[nodes near coords, point meta=explicit symbolic] coordinates {(0,1) [a] (1,2) [b]};");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(texts(&c), 2);
    // and the text can be anything, with \coordindex
    let (c, w) = axis("xtick=\\empty, ytick=\\empty", r"\addplot+[ybar, nodes near coords={(\coordindex)}] coordinates {(0,1) (1,2)};");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(texts(&c), 2);
}

#[test]
fn statistics_and_colorbar() {
    // hist: 3 bins of 12 values as ybar intervals
    let (c, w) = axis("xtick=\\empty, ytick=\\empty", r"\addplot+[hist={bins=3}] coordinates {(0,1) (0,2) (0,1) (0,5) (0,4) (0,10) (0,7) (0,10) (0,9) (0,8) (0,9) (0,9)};");
    assert!(w.is_empty(), "{w:?}");
    assert!(!filled(&c).is_empty());
    // a prepared box plot: a box, two whiskers, a median
    let (c, w) =
        axis("xtick=\\empty, ytick=\\empty", r"\addplot[boxplot prepared={lower whisker=1, lower quartile=2, median=3, upper quartile=4, upper whisker=5}] coordinates {};");
    assert!(w.is_empty(), "{w:?}");
    let strokes = c.items.iter().filter(|i| matches!(i, Item::Path { stroke: Some(_), .. })).count();
    assert!(strokes >= 4, "{strokes}");
    // a colorbar is an axis of its own next to the plot
    let (c, w) = axis("colorbar", r"\addplot[mesh] {x};");
    assert!(w.is_empty(), "{w:?}");
    let (wd, _) = size(&c);
    assert!(wd > 260.0, "{wd}");
}

#[test]
fn a_scaled_picture_scales_its_axis_text_and_all() {
    let plain = size(&axis("title=T", r"\addplot coordinates {(0,0) (1,1)};").0);
    let (c, w) = tikra::engine::run_document(r"\begin{tikzpicture}[scale=0.5]\begin{axis}[title=T] \addplot coordinates {(0,0) (1,1)}; \end{axis}\end{tikzpicture}").unwrap();
    assert!(w.is_empty(), "{w:?}");
    let half = size(&c);
    assert!((half.0 - plain.0 / 2.0).abs() < 0.5 && (half.1 - plain.1 / 2.0).abs() < 0.5, "{plain:?} {half:?}");
    // the title's text too
    let scale = |c: &Canvas| c.items.iter().find_map(|i| if let Item::Text { place, .. } = i { Some(place.d) } else { None }).unwrap();
    assert!((scale(&c) - scale(&axis("title=T", r"\addplot coordinates {(0,0) (1,1)};").0) / 2.0).abs() < 1e-9);
}
