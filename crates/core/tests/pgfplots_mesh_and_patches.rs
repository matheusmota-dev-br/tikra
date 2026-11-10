//! pgfplots: `mesh`, `surf` and `patch` plots — their shaders (`flat`, `flat corner`, `faceted`,
//! `interp`, `faceted interp`), colors given per point (`mesh/color input=explicit`), the patch
//! types of the `patchplots` library, `patch refines`, `patch to triangles`, the z buffer, the
//! interior colormap and piecewise constant colormaps; and the box of a 3d axis they are drawn in
//! (its faces in front from every view, tick positions, minor and extra ticks, centered and
//! shifted lines, `3d box=complete`).

mod common;
use common::*;
use tikra::canvas::{Paint, Shading};
use tikra::color::Color;
use tikra::scene::{Point, Segment};

/// The paths of a plot in an axis without its lines and ticks (`hide axis`).
fn drawn(options: &str, body: &str) -> Vec<Drawn> {
    let (c, w) = tikra::engine::run_document(&format!("\\begin{{tikzpicture}}\\begin{{axis}}[hide axis, {options}] {body} \\end{{axis}}\\end{{tikzpicture}}"))
        .unwrap_or_else(|e| panic!("{e}\n{body}"));
    assert!(w.is_empty(), "{w:?}\n{body}");
    paths(&c)
}

fn solid(d: &Drawn) -> Option<Color> {
    match &d.fill.as_ref()?.paint {
        Paint::Solid(c) => Some(*c),
        _ => None,
    }
}

fn axial(d: &Drawn) -> bool {
    matches!(&d.fill, Some(f) if matches!(f.paint, Paint::Shading(Shading::Axial { .. })))
}

fn corners(d: &Drawn) -> usize {
    d.segments.iter().filter(|s| !matches!(s, Segment::Close)).count()
}

fn near(a: Color, b: Color) -> bool {
    (a.r - b.r).abs() < 0.02 && (a.g - b.g).abs() < 0.02 && (a.b - b.b).abs() < 0.02
}

// --- colors given per point ---

#[test]
fn explicit_colors_in_every_model() {
    let red = Color::rgb(1.0, 0.0, 0.0);
    let cases = [
        ("color=red", red),
        ("rgb=0,1,0", Color::rgb(0.0, 1.0, 0.0)),
        ("1,0,0", red),
        ("rgb255=0,0,255", Color::rgb(0.0, 0.0, 1.0)),
        ("gray=0.5", Color::rgb(0.5, 0.5, 0.5)),
        ("cmyk=0,1,0,0", Color::rgb(1.0, 0.0, 1.0)),
        ("cmy=1,0,0", Color::rgb(0.0, 1.0, 1.0)),
        // a red of 700nm
        ("wave=700", red),
        // a model xcolor does not know: black
        ("hsv=0,1,1", Color::BLACK),
    ];
    for (spec, want) in cases {
        let d = drawn("", &format!(r"\addplot[patch, mesh/color input=explicit, shader=flat corner] coordinates {{(0,0) [{spec}] (1,0) [color=blue] (0,1) [color=blue]}};"));
        assert_eq!(d.len(), 1, "{spec}");
        let c = solid(&d[0]).unwrap();
        assert!(near(c, want), "{spec}: {c:?}");
    }
}

#[test]
fn wavelengths_go_through_the_spectrum() {
    let at = |nm: u32| {
        let d = drawn("", &format!(r"\addplot[patch, mesh/color input=explicit, shader=flat corner] coordinates {{(0,0) [wave={nm}] (1,0) [color=blue] (0,1) [color=blue]}};"));
        solid(&d[0]).unwrap()
    };
    // violet, dimmed at the end of the range; blue to cyan; cyan to green; green to yellow; yellow
    // to red; a dim red past 700nm
    let v = at(400);
    assert!(v.r > 0.0 && v.g == 0.0 && v.b > v.r && v.b < 1.0, "{v:?}");
    let b = at(460);
    assert!(b.r == 0.0 && b.g > 0.0 && b.g < 1.0 && close(b.b, 1.0), "{b:?}");
    let c = at(500);
    assert!(c.r == 0.0 && close(c.g, 1.0) && c.b > 0.0 && c.b < 1.0, "{c:?}");
    let g = at(550);
    assert!(g.r > 0.0 && g.r < 1.0 && close(g.g, 1.0) && g.b == 0.0, "{g:?}");
    let o = at(600);
    assert!(close(o.r, 1.0) && o.g > 0.0 && o.g < 1.0 && o.b == 0.0, "{o:?}");
    let r = at(750);
    assert!(r.r > 0.0 && r.r < 1.0 && r.g == 0.0 && r.b == 0.0, "{r:?}");
}

#[test]
fn a_point_without_a_color_leaves_its_patch_out() {
    let d =
        drawn("", r"\addplot[patch, mesh/color input=explicit] coordinates {(0,0) [] (1,0) [color=red] (0,1) [color=red] (2,0) [color=red] (3,0) [color=red] (2,1) [color=red]};");
    assert_eq!(d.len(), 1);
}

#[test]
fn explicit_colors_flat_mean_and_shaded() {
    // the mean of the corners' colors
    let d = drawn("", r"\addplot[patch, mesh/color input=explicit, shader=flat mean] coordinates {(0,0) [color=red] (1,0) [color=blue] (0,1) [color=blue]};");
    assert!(near(solid(&d[0]).unwrap(), Color::rgb(1.0 / 3.0, 0.0, 2.0 / 3.0)), "{:?}", solid(&d[0]));
    // shaded: small triangles, red at the first corner, blue at the others
    let d = drawn("", r"\addplot[patch, mesh/color input=explicit, shader=interp] coordinates {(0,0) [color=red] (1,0) [color=blue] (0,1) [color=blue]};");
    assert!(d.len() > 100, "{}", d.len());
    assert!(d.iter().all(|p| corners(p) == 3));
    assert!(solid(&d[0]).unwrap().r > 0.9 && solid(d.last().unwrap()).unwrap().b > 0.9);
    // a rectangle: small quadrilaterals, the colors bilinear between the corners
    let d = drawn(
        "",
        r"\addplot[patch, patch type=rectangle, mesh/color input=explicit, shader=interp] coordinates {(0,0) [color=red] (1,0) [color=red] (1,1) [color=blue] (0,1) [color=blue]};",
    );
    assert!(d.len() > 100 && d.iter().all(|p| corners(p) == 4));
    let first = solid(&d[0]).unwrap();
    assert!(first.r > 0.9 && first.b < 0.1, "{first:?}");
}

#[test]
fn a_matrix_of_explicit_colors_is_shaded_as_two_triangles_per_cell() {
    // from every side the z buffer reverses rows or columns: each cell is still cut in triangles
    for az in [25, -25, 155, -155] {
        let d = drawn(
            &format!("view={{{az}}}{{30}}"),
            r"\addplot3[surf, shader=interp, mesh/color input=explicit, mesh/rows=2] coordinates {(0,0,0) [color=red] (1,0,0) [color=red] (0,1,0) [color=blue] (1,1,0) [color=blue]};",
        );
        assert!(d.len() > 100 && d.iter().all(|p| corners(p) == 3), "{az}");
        assert!(d.iter().any(|p| solid(p).unwrap().r > 0.9) && d.iter().any(|p| solid(p).unwrap().b > 0.9), "{az}");
    }
}

#[test]
fn explicit_colors_of_each_patch_type() {
    // a patch of one color per type: as many corners as the type has (the curves are lines)
    let cases = [
        ("rectangle", 4, 4),
        ("bilinear", 4, 4),
        ("biquadratic", 9, 4),
        ("triangle quadr", 6, 3),
        ("quadratic triangle", 6, 3),
        ("line", 2, 2),
        ("cubic spline", 4, 2),
        ("triangle", 3, 3),
    ];
    for (kind, points, want) in cases {
        let coords: String = (0..points).map(|k| format!("({},{}) [color=red] ", k % 3, k / 3)).collect();
        let d = drawn("", &format!(r"\addplot[patch, patch type={kind}, mesh/color input=explicit, shader=flat] coordinates {{{coords}}};"));
        assert_eq!(d.len(), 1, "{kind}");
        assert_eq!(corners(&d[0]), want, "{kind}");
    }
    // a polygon of four
    let d =
        drawn("", r"\addplot[patch, patch type=polygon, vertex count=4, mesh/color input=explicit, shader=flat] coordinates {(0,0) [red] (1,0) [red] (1,1) [red] (0,1) [red]};");
    assert_eq!(corners(&d[0]), 4);
}

// --- matrices ---

#[test]
fn a_mesh_of_lines_follows_the_scanlines_only() {
    let d = drawn("", r"\addplot3[mesh, patch type=line, samples=3, domain=0:1] {x*y};");
    // 3 scanlines of 2 segments each
    assert_eq!(d.len(), 6);
    assert!(d.iter().all(|p| corners(p) == 2 && p.fill.is_none()));
}

#[test]
fn piecewise_constant_colormaps_cut_the_shading_into_bands() {
    // the colors of the bands are the colormap's own, whatever the view
    for az in [25, 155] {
        let d = drawn(
            &format!("view={{{az}}}{{30}}, colormap={{two}}{{rgb=(1,0,0) rgb=(0,0,1)}}, colormap access=piecewise constant"),
            r"\addplot3[surf, shader=interp, samples=3, domain=0:1] {x+y};",
        );
        assert!(!d.is_empty());
        for p in &d {
            let c = solid(p).unwrap();
            assert!(c.hex() == "#ff0000" || c.hex() == "#0000ff", "{az}: {}", c.hex());
        }
        assert!(d.iter().any(|p| solid(p).unwrap().hex() == "#ff0000") && d.iter().any(|p| solid(p).unwrap().hex() == "#0000ff"));
    }
}

#[test]
fn bilinear_cells_are_shaded_in_small_triangles() {
    // each cell: its mean color under small linear shadings, its edges on top
    let d = drawn("", r"\addplot3[surf, patch type=bilinear, shader=faceted interp, samples=2, domain=0:1] {x*y};");
    assert!(d.iter().filter(|p| axial(p)).count() >= 2);
    assert!(d.iter().any(|p| solid(p).is_some()));
    let edges: Vec<&Drawn> = d.iter().filter(|p| p.fill.is_none() && p.stroke.is_some()).collect();
    assert_eq!(edges.len(), 1);
    assert_eq!(corners(edges[0]), 4);
}

#[test]
fn a_flat_cell_whose_color_changes_one_way_is_one_shading() {
    // z = 0: each cell a parallelogram; the color changes along x or along y
    for meta in ["x", "y"] {
        let d = drawn("", &format!(r"\addplot3[surf, shader=interp, point meta={meta}, samples=3, domain=0:1] {{0}};"));
        assert_eq!(d.len(), 4, "{meta}");
        assert!(d.iter().all(axial), "{meta}");
    }
}

#[test]
fn the_interior_colormap_shows_the_inside() {
    // a bowl seen from above: its inside in the interior colormap's colors (here all green), from
    // every side
    for az in [25, -25, 155, -155] {
        let d =
            drawn(&format!("view={{{az}}}{{60}}"), r"\addplot3[surf, shader=flat, samples=4, domain=-1:1, mesh/interior colormap={greens}{rgb=(0,1,0) rgb=(0,1,0)}] {x^2+y^2};");
        assert!(d.iter().any(|p| solid(p).is_some_and(|c| c.hex() == "#00ff00")), "{az}");
    }
    // seen from below, its outside
    let d = drawn("view={25}{-60}", r"\addplot3[surf, shader=flat, samples=4, domain=-1:1, mesh/interior colormap={greens}{rgb=(0,1,0) rgb=(0,1,0)}] {x^2+y^2};");
    assert!(d.iter().all(|p| solid(p).is_none_or(|c| c.hex() != "#00ff00")));
    // with a threshold, only the patches turned away by more than it
    let d = drawn(
        "view={25}{60}",
        r"\addplot3[surf, shader=flat, samples=4, domain=-1:1, mesh/interior colormap={greens}{rgb=(0,1,0) rgb=(0,1,0)}, mesh/interior colormap thresh=-2] {x^2+y^2};",
    );
    assert!(d.iter().all(|p| solid(p).is_none_or(|c| c.hex() != "#00ff00")));
}

#[test]
fn point_meta_per_plot_spans_the_colormap_over_one_plot() {
    // two plots of metas 0 to 1 and 10 to 11, the first corner's color: per plot, both at the
    // colormap's start
    let colors = |rel: &str| {
        let d = drawn(
            &format!("point meta rel={rel}, colormap={{bw}}{{gray=(0) gray=(1)}}"),
            r"\addplot3[surf, shader=flat corner, point meta=explicit, mesh/rows=2] coordinates {(0,0,0) [0] (1,0,0) [1] (0,1,0) [1] (1,1,0) [1]};
              \addplot3[surf, shader=flat corner, point meta=explicit, mesh/rows=2] coordinates {(2,0,0) [10] (3,0,0) [11] (2,1,0) [11] (3,1,0) [11]};",
        );
        d.iter().filter_map(solid).map(|c| c.hex()).collect::<Vec<_>>()
    };
    assert_eq!(colors("per plot"), ["#000000", "#000000"]);
    assert_eq!(colors("axis wide")[0], "#000000");
    assert_ne!(colors("axis wide")[1], "#000000");
}

// --- lists of patches ---

/// The closed paths of a patch plot (the patches).
fn patches(options: &str, body: &str) -> Vec<Drawn> {
    drawn(options, body).into_iter().filter(|d| matches!(d.segments.last(), Some(Segment::Close))).collect()
}

/// `n` patches of `size` points each: x and y on a small grid, z = 0.
fn points(n: usize, size: usize) -> String {
    let side = (size as f64).sqrt().ceil() as usize;
    let mut s = String::new();
    for p in 0..n {
        for k in 0..size {
            s.push_str(&format!("({},{}) ", p * 4 + k % side, k / side));
        }
    }
    s
}

#[test]
fn unbounded_patches_are_left_out() {
    let d = patches("unbounded coords=jump", r"\addplot[patch] coordinates {(0,0) (1,0) (nan,1) (2,0) (3,0) (2,1)};");
    assert_eq!(d.len(), 1);
}

#[test]
fn patches_of_unbounded_point_meta_are_left_out() {
    let d = patches("", r"\addplot[patch, point meta=explicit] coordinates {(0,0) [nan] (1,0) [1] (0,1) [1] (2,0) [1] (3,0) [1] (2,1) [1]};");
    assert_eq!(d.len(), 1);
}

#[test]
fn patches_refined_and_cut_into_triangles() {
    // (type, points, patches after one refinement, triangles)
    let cases = [("triangle", 3, 2, 1), ("rectangle", 4, 4, 2), ("bilinear", 4, 4, 2), ("biquadratic", 9, 4, 8), ("triangle quadr", 6, 4, 4), ("coons", 12, 1, 1)];
    for (kind, size, refined, triangles) in cases {
        let coords = points(1, size);
        assert_eq!(patches("", &format!(r"\addplot[patch, patch type={kind}, patch refines=1] coordinates {{{coords}}};")).len(), refined, "{kind}");
        let t = patches("", &format!(r"\addplot[patch, patch type={kind}, patch to triangles] coordinates {{{coords}}};"));
        assert_eq!(t.len(), triangles, "{kind}");
        if triangles > 1 {
            assert!(t.iter().all(|p| p.segments.iter().filter(|s| matches!(s, Segment::LineTo(_))).count() == 3), "{kind}");
        }
    }
}

#[test]
fn curves_refined_are_drawn_in_pieces() {
    // (type, points, pieces after one refinement)
    for (kind, size, refined) in [("line", 2, 2), ("quadratic spline", 3, 2), ("cubic spline", 4, 3), ("bezier spline", 4, 1)] {
        let coords: String = (0..size).map(|k| format!("({k},{}) ", k % 2)).collect();
        let d = drawn("", &format!(r"\addplot[patch, patch type={kind}, patch refines=1] coordinates {{{coords}}};"));
        assert_eq!(d.len(), refined, "{kind}");
        assert!(d.iter().all(|p| p.fill.is_none() && p.stroke.is_some()), "{kind}");
    }
}

#[test]
fn the_outline_of_each_patch_type() {
    // (type, points, lines, curves)
    let cases = [
        ("triangle", 3, 3, 0),
        ("rectangle", 4, 4, 0),
        ("polygon, vertex count=5", 5, 5, 0),
        ("triangle quadr", 6, 0, 3),
        ("biquadratic", 9, 0, 4),
        ("bicubic", 16, 0, 4),
        ("coons", 12, 0, 4),
        ("tensor bezier", 16, 0, 4),
    ];
    for (kind, size, lines, curves) in cases {
        let d = patches("", &format!(r"\addplot[patch, patch type={kind}] coordinates {{{}}};", points(1, size)));
        assert_eq!(d.len(), 1, "{kind}");
        let s = &d[0].segments;
        assert_eq!(s.iter().filter(|x| matches!(x, Segment::LineTo(_))).count(), lines, "{kind}");
        assert_eq!(s.iter().filter(|x| matches!(x, Segment::CurveTo(..))).count(), curves, "{kind}");
        // faceted: filled in its color, its edges darker
        let (f, e) = (solid(&d[0]).unwrap(), d[0].stroke.as_ref().unwrap().color);
        assert!(close(e.r, 0.8 * f.r) && close(e.g, 0.8 * f.g), "{kind}");
    }
}

#[test]
fn a_mesh_of_patches_draws_their_outlines() {
    let d = drawn("", r"\addplot[patch, mesh, patch type=rectangle] coordinates {(0,0) (1,0) (1,1) (0,1)};");
    assert_eq!(d.len(), 1);
    assert!(d[0].fill.is_none() && d[0].stroke.is_some());
}

#[test]
fn patches_in_three_dimensions_are_drawn_back_to_front() {
    // seen from above, the triangle at z=1 is in front
    let d = patches("view={25}{60}", r"\addplot3[patch] coordinates {(0,0,1) (1,0,1) (0,1,1) (0,0,0) (1,0,0) (0,1,0)};");
    assert_eq!(d.len(), 2);
    assert!(bounds(&d[1].segments).1.y > bounds(&d[0].segments).1.y);
    // without the z buffer, in their order
    let d = patches("view={25}{60}", r"\addplot3[patch, z buffer=none] coordinates {(0,0,1) (1,0,1) (0,1,1) (0,0,0) (1,0,0) (0,1,0)};");
    assert!(bounds(&d[0].segments).1.y > bounds(&d[1].segments).1.y);
}

#[test]
fn flat_patches_take_their_corners_color() {
    // flat: the edges in the fill's color; flat corner: the first point's
    let d = patches(
        "colormap={bw}{gray=(0) gray=(1)}",
        r"\addplot[patch, shader=flat, point meta=explicit] coordinates {(0,0) [0] (1,0) [1] (0,1) [1] (2,0) [0] (3,0) [0] (2,1) [0]};",
    );
    assert_eq!(solid(&d[0]).unwrap().hex(), d[0].stroke.as_ref().unwrap().color.hex());
    let d = patches(
        "colormap={bw}{gray=(0) gray=(1)}",
        r"\addplot[patch, shader=flat corner, point meta=explicit] coordinates {(0,0) [0] (1,0) [1] (0,1) [1] (2,0) [1] (3,0) [0] (2,1) [0]};",
    );
    assert_eq!(solid(&d[0]).unwrap().hex(), "#000000");
    assert_eq!(solid(&d[1]).unwrap().hex(), "#ffffff");
}

/// The fills of a shaded patch plot: (axial shadings, solid pieces).
fn shading(options: &str, body: &str) -> (usize, usize) {
    let d = drawn(options, body);
    (d.iter().filter(|p| axial(p)).count(), d.iter().filter(|p| solid(p).is_some()).count())
}

#[test]
fn shaded_patches_of_higher_order() {
    // curved patches: small triangles each a linear shading, over the patch in its mean color
    for (kind, size) in [("biquadratic", 9), ("bicubic", 16), ("coons", 12), ("tensor bezier", 16), ("bilinear", 4)] {
        let (axial, solids) = shading("", &format!(r"\addplot[patch, patch type={kind}, shader=interp] coordinates {{{}}};", points(1, size)));
        assert!(axial >= 2 && solids >= 1, "{kind}: {axial} {solids}");
    }
    // the same in 3d, sorted back to front
    let (axial, solids) =
        shading("", r"\addplot3[patch, patch type=biquadratic, shader=interp] coordinates {(0,0,0) (2,0,1) (2,2,2) (0,2,1) (1,0,0.5) (2,1,1.5) (1,2,1.5) (0,1,0.5) (1,1,1)};");
    assert!(axial >= 2 && solids >= 1, "{axial} {solids}");
    // a rectangle: two exact triangles
    assert_eq!(shading("", r"\addplot[patch, patch type=rectangle, shader=interp] coordinates {(0,0) (1,0) (1,1) (0,1)};"), (2, 0));
}

#[test]
fn shaded_patches_in_pieces() {
    // a quadratic triangle, a polygon, and any patch with a piecewise constant colormap: small
    // pieces of one color each
    for (options, kind, size) in [
        ("", "triangle quadr", 6),
        ("", "polygon, vertex count=4", 4),
        ("colormap access=piecewise constant", "triangle", 3),
        ("colormap access=piecewise constant", "rectangle", 4),
        ("colormap access=piecewise constant", "biquadratic", 9),
    ] {
        let (axial, solids) = shading(options, &format!(r"\addplot[patch, patch type={kind}, shader=interp] coordinates {{{}}};", points(1, size)));
        assert!(axial == 0 && solids > 4, "{kind}: {axial} {solids}");
    }
    // in 3d too
    let (axial, solids) = shading("", r"\addplot3[patch, patch type=triangle quadr, shader=interp] coordinates {(0,0,0) (2,0,1) (0,2,1) (1,0,0.5) (1,1,1) (0,1,0.5)};");
    assert!(axial == 0 && solids > 4, "{axial} {solids}");
}

#[test]
fn faceted_interp_draws_the_outline_on_the_shading() {
    let d = drawn("", r"\addplot[patch, patch type=biquadratic, shader=faceted interp] coordinates {(0,0) (2,0) (2,2) (0,2) (1,0) (2,1) (1,2) (0,1) (1,1)};");
    let last = d.last().unwrap();
    assert!(last.fill.is_none() && last.stroke.is_some());
    assert_eq!(last.segments.iter().filter(|s| matches!(s, Segment::CurveTo(..))).count(), 4);
}

#[test]
fn a_shaded_patch_with_an_unbounded_meta_inside_is_flat() {
    // the middle point's meta is nan: the patch is filled in its corners' mean color
    let d = patches(
        "colormap={bw}{gray=(0) gray=(1)}",
        r"\addplot[patch, patch type=biquadratic, shader=interp, point meta=explicit] coordinates {(0,0) [0] (2,0) [1] (2,2) [1] (0,2) [0] (1,0) [0] (2,1) [1] (1,2) [0] (0,1) [0] (1,1) [nan]};",
    );
    assert_eq!(d.len(), 1);
    let c = solid(&d[0]).unwrap();
    assert!((c.r - 0.5).abs() < 0.01, "{c:?}");
    assert_eq!(c.hex(), d[0].stroke.as_ref().unwrap().color.hex());
}

#[test]
fn shaded_cells_are_two_linear_shadings_from_every_side() {
    // a saddle: each cell cut in two triangles as the z buffer turns it, each its linear shading
    // (one of a single meta, solid)
    for az in [25, -25, 155, -155] {
        let d = drawn(&format!("view={{{az}}}{{30}}"), r"\addplot3[surf, shader=interp, samples=3, domain=0:1] {x*y};");
        assert_eq!(d.len(), 8, "{az}");
        assert!(d.iter().all(|p| corners(p) == 3 && p.fill.is_some()) && d.iter().filter(|p| axial(p)).count() >= 6, "{az}");
    }
}

#[test]
fn small_cells_of_a_direct_colormap_are_flat_with_a_hairline() {
    // `colormap access=direct`: the meta is a color's index, each cell in its color
    let d = drawn(
        "scale only axis, width=8pt, height=8pt, colormap={three}{rgb=(1,0,0) rgb=(0,1,0) rgb=(0,0,1)}, colormap access=direct",
        r"\addplot3[surf, shader=interp, point meta=explicit, mesh/rows=2] coordinates {(0,0,0) [0] (1,0,0) [0] (0,1,0) [0] (1,1,0) [0] };",
    );
    assert_eq!(d.len(), 1);
    let s = d[0].stroke.as_ref().unwrap();
    assert_eq!(solid(&d[0]).unwrap().hex(), "#ff0000");
    assert_eq!(s.color.hex(), "#ff0000");
    assert!(close(s.width, 0.2 * tikra::PT));
}

// --- the 3d axis box ---

/// A 3d axis of the unit cube with its corners marked in red: the canvas, its warnings, the
/// corners and the tick labels' centers.
fn cube(options: &str) -> (tikra::canvas::Canvas, Vec<String>, Vec<Point>, Vec<Point>) {
    let corners: String = (0..8).map(|k| format!("({},{},{}) ", k & 1, (k >> 1) & 1, (k >> 2) & 1)).collect();
    let (c, w) = tikra::engine::run_document(&format!(
        "\\begin{{tikzpicture}}\\begin{{axis}}[xmin=0, xmax=1, ymin=0, ymax=1, zmin=0, zmax=1, {options}] \\addplot3[red, only marks, mark=*] coordinates {{{corners}}}; \\end{{axis}}\\end{{tikzpicture}}"
    ))
    .unwrap_or_else(|e| panic!("{e}\n{options}"));
    let marks: Vec<Point> = paths(&c)
        .iter()
        .filter(|d| d.fill.as_ref().is_some_and(|f| matches!(f.paint, Paint::Solid(k) if k.hex() == "#ff0000")))
        .map(|d| {
            let (a, b) = bounds(&d.segments);
            Point::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)
        })
        .collect();
    let labels = texts(&c)
        .iter()
        .map(|(l, t)| {
            let h = l.height() - l.depth();
            Point::new(t.e + l.width() / 2.0, t.f + h / 2.0)
        })
        .collect();
    (c, w, marks, labels)
}

/// Whether `p` is inside the convex hull of `pts`.
fn inside_hull(p: Point, pts: &[Point]) -> bool {
    let mut v: Vec<Point> = pts.to_vec();
    v.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    let cross = |o: Point, a: Point, b: Point| (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);
    let mut hull: Vec<Point> = Vec::new();
    for pass in 0..2 {
        let start = hull.len();
        let order: Vec<Point> = if pass == 0 { v.clone() } else { v.iter().rev().copied().collect() };
        for q in order {
            while hull.len() >= start + 2 && cross(hull[hull.len() - 2], hull[hull.len() - 1], q) <= 1e-12 {
                hull.pop();
            }
            hull.push(q);
        }
        hull.pop();
    }
    let n = hull.len();
    n >= 3 && (0..n).all(|i| cross(hull[i], hull[(i + 1) % n], p) > 1e-9)
}

#[test]
fn tick_labels_are_outside_the_box_from_every_side() {
    // pgfplots puts them on the lines of the box's outline, outwards
    for view in
        ["{25}{30}", "{-25}{30}", "{115}{30}", "{205}{30}", "{-115}{30}", "{25}{-30}", "{160}{-40}", "{-70}{60}", "{30}{0}", "{0}{0}", "{0}{90}", "{225}{-30}", "{-45}{-30}"]
    {
        let (_, w, corners, labels) = cube(&format!("view={view}, xtick={{0,1}}, ytick={{0,1}}, ztick={{0,1}}"));
        assert!(w.is_empty(), "{view}: {w:?}");
        assert_eq!(corners.len(), 8, "{view}");
        assert!(!labels.is_empty(), "{view}");
        for l in &labels {
            assert!(!inside_hull(*l, &corners), "{view}: {l:?} inside {corners:?}");
        }
    }
}

/// The paths of a 3d axis of the unit cube (without the corners' marks).
fn box_paths(options: &str) -> usize {
    cube(options).0.items.iter().filter(|i| matches!(i, tikra::canvas::Item::Path { .. })).count() - 8
}

#[test]
fn tick_positions_and_label_sides() {
    // ticks on one side of each face only: fewer tick marks; none: fewer still
    let both = box_paths("");
    let tick_marks = |o: &str| {
        let (c, _, _, _) = cube(o);
        paths(&c).iter().filter(|d| d.stroke.as_ref().is_some_and(|s| s.color.hex() == "#808080")).map(|d| d.segments.len()).sum::<usize>()
    };
    let all = tick_marks("");
    for pos in ["xtick pos=upper", "xtick pos=lower", "xtick pos=upper, y dir=reverse", "xtick pos=lower, y dir=reverse"] {
        let n = tick_marks(pos);
        assert!(n < all && n > 0, "{pos}: {n} {all}");
    }
    assert!(tick_marks("xtick pos=none") < tick_marks("xtick pos=upper"));
    // the labels on the upper side: elsewhere than on the lower one
    let (_, _, _, lower) = cube("xtick={0,1}, ytick=\\empty, ztick=\\empty");
    let (_, _, _, upper) = cube("xtick={0,1}, ytick=\\empty, ztick=\\empty, xticklabel pos=upper");
    let (_, _, _, by_ticks) = cube("xtick={0,1}, ytick=\\empty, ztick=\\empty, xtick pos=upper");
    assert!(lower[0].minus(upper[0]).norm() > 0.5, "{lower:?} {upper:?}");
    assert!(by_ticks[0].minus(upper[0]).norm() < 1e-6, "{by_ticks:?} {upper:?}");
    // axis lines on one side, their ticks on the upper one
    let (_, w, _, left) = cube("axis lines=left, xtick={0,1}, ytick=\\empty, ztick=\\empty, xtick pos=upper");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(left.len(), 2);
    assert!(box_paths("axis lines=left") < both);
}

#[test]
fn ticks_of_lines_on_one_side_take_their_labels() {
    // `axis lines*=left` (without the styles that set the tick positions again): the x ticks on
    // the upper side, their labels there too
    let (_, w, _, lower) = cube("axis lines*=left, xtick={0,1}, ytick=\\empty, ztick=\\empty, xtick pos=lower");
    assert!(w.is_empty(), "{w:?}");
    let (_, _, _, upper) = cube("axis lines*=left, xtick={0,1}, ytick=\\empty, ztick=\\empty, xtick pos=upper");
    assert_eq!((lower.len(), upper.len()), (2, 2));
    assert!(upper.iter().zip(&lower).all(|(u, l)| u.y > l.y + 1.0), "{upper:?} {lower:?}");
}

#[test]
fn unit_vectors_degenerate_on_paper() {
    // x and y along one line: no view direction solves them (taken as seen from above); y and z
    // opposite: the x axis' line has no outward side. Drawn all the same, the labels where the
    // ticks are.
    for units in ["x={(1cm,1cm)}, y={(2cm,2cm)}, z={(0cm,1cm)}", "x={(1cm,0cm)}, y={(0cm,1cm)}, z={(0cm,-1cm)}"] {
        let (_, w, corners, labels) = cube(&format!("{units}, xtick={{0,1}}, ytick=\\empty, ztick=\\empty"));
        assert!(w.is_empty(), "{units}: {w:?}");
        assert_eq!(corners.len(), 8, "{units}");
        assert_eq!(labels.len(), 2, "{units}");
        assert!(labels.iter().all(|l| l.x.is_finite() && l.y.is_finite()), "{units}: {labels:?}");
    }
}

#[test]
fn minor_ticks_grids_and_extra_ticks_in_3d() {
    let plain = box_paths("");
    assert!(box_paths("minor tick num=1") > plain);
    assert!(box_paths("minor tick num=1, grid=both") > box_paths("minor tick num=1, grid=major"));
    // extra ticks: their tick marks, grid lines and labels
    let (_, w, _, labels) = cube("xtick={0,1}, ytick={0,1}, ztick={0,1}, extra x ticks={0.5}, extra y ticks={0.5}, extra z ticks={0.5}, extra tick style={grid=major}");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(labels.len(), 9);
    assert!(box_paths("extra x ticks={0.5}, extra y ticks={0.5}, extra z ticks={0.5}") > plain);
}

#[test]
fn centered_axis_lines_in_3d() {
    // the lines through the zeros: three lines, their labels beside them
    let (c, w, _, labels) = cube("axis lines=center, xmin=-1, ymin=-1, zmin=-1, xtick={-1,1}, ytick={-1,1}, ztick={-1,1}");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(labels.len(), 6);
    assert!(paths(&c).len() > 8);
    // reversed: the labels on the other side of the line
    let (_, _, _, reversed) = cube("axis lines=center, xmin=-1, ymin=-1, zmin=-1, xtick={-1,1}, ytick={-1,1}, ztick={-1,1}, y dir=reverse");
    assert!(labels.iter().zip(&reversed).any(|(a, b)| a.minus(*b).norm() > 1e-3));
}

#[test]
fn shifted_axis_lines_in_3d() {
    // `axis x line shift`: the x lines and their ticks move outwards, 20pt
    // (their tick labels do not follow yet: a known bug)
    let options = "xtick={0,1}, ytick=\\empty, ztick=\\empty, xlabel=\\empty";
    let lines = |o: &str| -> Vec<Vec<(f64, f64)>> {
        let (c, w, _, _) = cube(o);
        assert!(w.is_empty(), "{w:?}");
        paths(&c).iter().filter(|d| d.fill.is_none() && d.segments.len() == 2).map(|d| common::points(&d.segments)).collect()
    };
    let (plain, moved) = (lines(options), lines(&format!("{options}, axis x line shift=20pt")));
    assert_eq!(plain.len(), moved.len());
    let shifted: Vec<f64> = plain.iter().zip(&moved).filter(|(a, b)| a != b).map(|(a, b)| ((a[0].0 - b[0].0).powi(2) + (a[0].1 - b[0].1).powi(2)).sqrt()).collect();
    assert!(!shifted.is_empty());
    assert!(shifted.iter().any(|d| (d - 20.0 * tikra::PT).abs() < 1e-6), "{shifted:?}");
}

#[test]
fn a_complete_box_with_lines_on_one_side_has_no_front() {
    // `3d box=complete` draws the faces in front only with a box of lines
    assert_eq!(box_paths("3d box=complete, axis lines=left"), box_paths("axis lines=left"));
    assert!(box_paths("3d box=complete") > box_paths(""));
}

#[test]
fn a_curved_cell_whose_color_changes_one_way_is_still_two_shadings() {
    // the meta changes along x only, but the saddle's cells are not flat: two triangles each
    let d = drawn("", r"\addplot3[surf, shader=interp, point meta=x, samples=3, domain=0:1] {x*y};");
    assert_eq!(d.len(), 8);
    assert!(d.iter().all(axial));
}

#[test]
fn bilinear_cells_shaded_without_their_edges() {
    let d = drawn("", r"\addplot3[surf, patch type=bilinear, shader=interp, samples=2, domain=0:1] {x*y};");
    assert!(d.iter().filter(|p| axial(p)).count() >= 2);
    assert!(d.iter().all(|p| p.fill.is_some() && p.stroke.is_none()));
}

#[test]
fn large_cells_of_a_direct_colormap_are_shaded_in_pieces() {
    // indices 0 and 1 across the cell: pieces between the first color and the second, none of
    // the third
    let d = drawn(
        "colormap={three}{rgb=(1,0,0) rgb=(0,1,0) rgb=(0,0,1)}, colormap access=direct",
        r"\addplot3[surf, shader=interp, point meta=explicit, mesh/rows=2] coordinates {(0,0,0) [0] (1,0,0) [1] (0,1,0) [0] (1,1,0) [1]};",
    );
    assert!(d.len() > 10);
    let colors: Vec<Color> = d.iter().filter_map(solid).collect();
    assert!(colors.iter().all(|c| c.b == 0.0 && close(c.r + c.g, 1.0)), "{colors:?}");
    assert!(colors.iter().any(|c| c.hex() == "#ff0000"));
}
