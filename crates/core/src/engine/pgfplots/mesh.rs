//! `mesh`, `surf` and `patch` plots (`pgfplotsmeshplothandler.code.tex`): a lattice or a list of
//! patches, each colored by the colormap at its point meta, drawn back to front in 3d
//! (`z buffer`), with the shaders `faceted` (fill and `faceted color` edges), `flat` (one color,
//! edges too), `flat corner` and `interp` (the patch's mean color).
//!
//! Colors given with `mapped color` (`draw=mapped color!80!black`) are linear in it: each is found
//! by setting the style with `mapped color` black, then white.

use crate::canvas::{Fill, Item, Paint, Stroke};
use crate::color::Color;
use crate::scene::{Point, Segment};
use crate::tex::Token;
use crate::Error;

use super::draw::Frame;
use super::keys;
use super::plot::Plot;
use super::Engine;

/// A color made from `mapped color`: `base + mapped × slope`, channel by channel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Linear {
    pub base: Color,
    pub slope: Color,
}

impl Linear {
    pub fn from(black: Color, white: Color) -> Linear {
        Linear { base: black, slope: Color { r: white.r - black.r, g: white.g - black.g, b: white.b - black.b } }
    }
    pub fn at(&self, m: Color) -> Color {
        let c = |b: f64, s: f64, m: f64| (b + s * m).clamp(0.0, 1.0);
        Color { r: c(self.base.r, self.slope.r, m.r), g: c(self.base.g, self.slope.g, m.g), b: c(self.base.b, self.slope.b, m.b) }
    }
}

/// The state's colors as functions of `mapped color`: (draw, fill) after `f` runs in a group.
pub fn mapped_colors(e: &mut Engine, mut f: impl FnMut(&mut Engine) -> Result<(), Error>) -> Result<(Linear, Linear, super::super::GState), Error> {
    let mut states = Vec::new();
    for m in [Color::BLACK, Color::WHITE] {
        e.colors.insert("mapped color".into(), m);
        e.begin_group();
        let r = f(e);
        let st = e.st.clone();
        e.end_group();
        r?;
        states.push(st);
    }
    // what is drawn later with `mapped color` (nodes near coords) gets TeX's black
    e.colors.insert("mapped color".into(), Color::BLACK);
    let draw = Linear::from(states[0].stroke_color(), states[1].stroke_color());
    let fill = Linear::from(states[0].fill_paint_color(), states[1].fill_paint_color());
    Ok((draw, fill, states.swap_remove(1)))
}

/// A color as a function of `mapped color`.
fn mapped_named(e: &mut Engine, name: &str) -> Option<Linear> {
    e.colors.insert("mapped color".into(), Color::BLACK);
    let a = e.color(name)?;
    e.colors.insert("mapped color".into(), Color::WHITE);
    let b = e.color(name);
    e.colors.insert("mapped color".into(), Color::BLACK);
    Some(Linear::from(a, b?))
}

/// The colormap's color for meta value `m`.
pub fn map(f: &Frame, m: f64) -> Color {
    let t = (m - f.meta[0]) / (f.meta[1] - f.meta[0]);
    match &f.colormap {
        Some(c) => c.at(t),
        None => Color::BLACK,
    }
}

/// The patches of a plot: corner indices.
fn patches(p: &Plot, n: usize, kind: &str) -> Vec<Vec<usize>> {
    if let Some((rows, cols)) = p.lattice {
        let mut v = Vec::new();
        for r in 0..rows.saturating_sub(1) {
            for c in 0..cols.saturating_sub(1) {
                let k = r * cols + c;
                v.push(vec![k, k + 1, k + cols + 1, k + cols]);
            }
        }
        return v;
    }
    if p.patch {
        let (size, corners): (usize, usize) = match kind {
            "rectangle" | "bilinear" => (4, 4),
            "biquadratic" => (9, 4),
            "quadratic triangle" => (6, 3),
            "triangle quadr" => (6, 3),
            "cubic spline" => (4, 2),
            "line" => (2, 2),
            "bicubic" => (16, 4),
            "polygon" => (4, 4),
            _ => (3, 3),
        };
        return (0..n / size).map(|i| (0..corners).map(|k| i * size + k).collect()).collect();
    }
    // a line of points: segments
    (0..n.saturating_sub(1)).map(|i| vec![i, i + 1]).collect()
}

/// Draws a mesh, surf or patch plot (in the plot's group, its style set).
pub fn draw(e: &mut Engine, p: &Plot, pts: &[[f64; 3]], f: &Frame, style: &[Token]) -> Result<(), Error> {
    let (draw, fill, st) = mapped_colors(e, |e| {
        e.st.mode = Default::default();
        e.process_keys(style.to_vec(), Some("/pgfplots"))
    })?;
    let surf = p.mesh == Some('s');
    let shader = keys::value(e, "/pgfplots/shader");
    let faceted_name = keys::value(e, "/pgfplots/faceted color");
    let faceted = mapped_named(e, &faceted_name).unwrap_or(draw);
    let kind = keys::value(e, "/pgfplots/patch type");
    let zbuffer = keys::value(e, "/pgfplots/z buffer");
    let width = st.line_width;
    let opacity = (st.draw_opacity, st.fill_opacity);
    let canvas: Vec<Option<Point>> = pts.iter().map(|l| if l.iter().take(if f.three_d { 3 } else { 2 }).all(|v| v.is_finite()) { Some(f.at(*l)) } else { None }).collect();
    let mut list = patches(p, pts.len(), &kind);
    // 2d lines through a lattice: a mesh of a one dimensional line has no area
    if f.three_d && zbuffer != "none" {
        let az = keys::number(e, "/pgfplots/view/az").unwrap_or(25.0).to_radians();
        let el = keys::number(e, "/pgfplots/view/el").unwrap_or(30.0).to_radians();
        let d = [az.sin() * el.cos(), -az.cos() * el.cos(), el.sin()];
        let norm = |i: usize, v: f64| (v - f.min[i]) / (f.max[i] - f.min[i]).abs().max(1e-300) * if f.e[i].norm() == 0.0 { 0.0 } else { 1.0 };
        let depth = |q: &Vec<usize>| -> f64 {
            let mut s = 0.0;
            for &k in q {
                let l = pts[k];
                s += (0..3).map(|i| d[i] * norm(i, l[i])).sum::<f64>();
            }
            s / q.len() as f64
        };
        list.sort_by(|a, b| depth(a).total_cmp(&depth(b)));
    }
    let closes = list.first().is_some_and(|q| q.len() > 2);
    let explicit = keys::value(e, "/pgfplots/mesh/color input").starts_with("explicit");
    for q in list {
        let Some(corners) = q.iter().map(|&k| canvas.get(k).copied().flatten()).collect::<Option<Vec<Point>>>() else { continue };
        let metas: Vec<f64> = q.iter().map(|&k| p.coords.get(k).map_or(f64::NAN, |c| c.meta)).filter(|m| m.is_finite()).collect();
        let m = if shader == "flat corner" { metas.first().copied().unwrap_or(f64::NAN) } else { metas.iter().sum::<f64>() / metas.len().max(1) as f64 };
        // `mesh/color input=explicit`: each point has its color
        let vertex: Option<Vec<Color>> = explicit.then(|| q.iter().map(|&k| vertex_color(e, p, k)).collect());
        let mapped = match &vertex {
            Some(v) if shader == "flat corner" => v[0],
            Some(v) => mean(v),
            None if m.is_finite() => map(f, m),
            None => Color::BLACK,
        };
        // interp: a large patch is shaded across, drawn as small pieces interpolated between its
        // corners
        let interp_ok = vertex.is_some() || metas.len() == corners.len();
        if closes && shader == "interp" && interp_ok && (corners.len() == 4 || corners.len() == 3) {
            let size = (0..corners.len()).map(|i| corners[i].minus(corners[(i + 1) % corners.len()]).norm()).fold(0.0, f64::max);
            let n = if size > 12.0 * crate::PT { ((size / (3.0 * crate::PT)).ceil() as usize).clamp(1, 64) } else { 1 };
            if n > 1 {
                // a point and its color from weights of the corners
                let at = |w: &[f64]| -> (Point, Color) {
                    let mut pt = Point::default();
                    for k in 0..corners.len() {
                        pt = pt.plus(corners[k].scale(w[k]));
                    }
                    let c = match &vertex {
                        Some(v) => blend(v, w),
                        None => map(f, (0..corners.len()).map(|k| metas[k] * w[k]).sum()),
                    };
                    (pt, c)
                };
                let mut pieces: Vec<Vec<(Point, Color)>> = Vec::new();
                let nf = n as f64;
                if corners.len() == 4 {
                    let bil = |u: f64, v: f64| [(1.0 - u) * (1.0 - v), u * (1.0 - v), u * v, (1.0 - u) * v];
                    for i in 0..n {
                        for j in 0..n {
                            let (u0, u1, v0, v1) = (i as f64 / nf, (i + 1) as f64 / nf, j as f64 / nf, (j + 1) as f64 / nf);
                            pieces.push([bil(u0, v0), bil(u1, v0), bil(u1, v1), bil(u0, v1)].iter().map(|w| at(w)).collect());
                        }
                    }
                } else {
                    let bar = |i: usize, j: usize| {
                        let (a, b) = (i as f64 / nf, j as f64 / nf);
                        [1.0 - a - b, a, b]
                    };
                    for i in 0..n {
                        for j in 0..n - i {
                            pieces.push([bar(i, j), bar(i + 1, j), bar(i, j + 1)].iter().map(|w| at(w)).collect());
                            if i + j + 1 < n {
                                pieces.push([bar(i + 1, j), bar(i + 1, j + 1), bar(i, j + 1)].iter().map(|w| at(w)).collect());
                            }
                        }
                    }
                }
                for piece in pieces {
                    let c = fill.at(mean(&piece.iter().map(|q| q.1).collect::<Vec<_>>()));
                    let mut segs = vec![Segment::MoveTo(piece[0].0)];
                    segs.extend(piece[1..].iter().map(|q| Segment::LineTo(q.0)));
                    segs.push(Segment::Close);
                    if !e.canvas_frozen {
                        e.canvas.include_path(&segs);
                    }
                    let stroke = Stroke { color: c, width: 0.5 * crate::PT, dash: None, cap: st.cap, join: st.join, miter_limit: st.miter_limit, opacity: opacity.0 };
                    e.canvas.push(Item::Path { segments: segs, fill: Some(Fill { paint: Paint::Solid(c), opacity: opacity.1, even_odd: false }), stroke: Some(stroke) });
                }
                continue;
            }
        }
        let mut segs = vec![Segment::MoveTo(corners[0])];
        segs.extend(corners[1..].iter().map(|c| Segment::LineTo(*c)));
        if closes {
            segs.push(Segment::Close);
        }
        if !e.canvas_frozen {
            e.canvas.include_path(&segs);
        }
        let stroke =
            |c: Color| Stroke { color: c, width, dash: st.dash.clone().filter(|d| !d.0.is_empty()), cap: st.cap, join: st.join, miter_limit: st.miter_limit, opacity: opacity.0 };
        let (fill_c, stroke_c) = if closes && (surf || p.patch) {
            match shader.as_str() {
                // pgfplots shades each patch; flat patches with a hairline of their color have
                // no seams between them either
                "interp" => (Some(fill.at(mapped)), Some(fill.at(mapped))),
                "flat" | "flat mean" | "flat corner" => (Some(fill.at(mapped)), Some(fill.at(mapped))),
                _ => (Some(fill.at(mapped)), Some(faceted.at(mapped))),
            }
        } else {
            (None, Some(draw.at(mapped)))
        };
        let fill_item = fill_c.map(|c| Fill { paint: Paint::Solid(c), opacity: opacity.1, even_odd: false });
        let mut stroke_item = stroke_c.map(stroke);
        if let (Some(st), true) = (stroke_item.as_mut(), surf && closes && shader == "interp") {
            st.width = 0.2 * crate::PT;
        }
        e.canvas.push(Item::Path { segments: segs, fill: fill_item, stroke: stroke_item });
    }
    Ok(())
}

fn mean(v: &[Color]) -> Color {
    let n = v.len().max(1) as f64;
    Color { r: v.iter().map(|c| c.r).sum::<f64>() / n, g: v.iter().map(|c| c.g).sum::<f64>() / n, b: v.iter().map(|c| c.b).sum::<f64>() / n }
}

fn blend(v: &[Color], w: &[f64]) -> Color {
    let mut c = Color { r: 0.0, g: 0.0, b: 0.0 };
    for (k, x) in v.iter().zip(w) {
        c.r += k.r * x;
        c.g += k.g * x;
        c.b += k.b * x;
    }
    c
}

/// The color of point `k` of an explicit color input: `color=red`, `rgb=…`, `R,G,B`, `rgb255=`,
/// `cmyk=`, `gray=`, `wave=` (nanometres), as written or computed by the survey.
fn vertex_color(e: &Engine, p: &Plot, k: usize) -> Color {
    let Some(label) = p.coords.get(k).filter(|c| c.label != super::plot::NO_LABEL).and_then(|c| p.labels.get(c.label as usize)) else { return Color::BLACK };
    explicit_color(e, &crate::tex::tokens_to_string(label)).unwrap_or(Color::BLACK)
}

pub fn explicit_color(e: &Engine, s: &str) -> Option<Color> {
    let s = s.trim();
    let (model, rest) = match s.split_once('=') {
        Some((m, r)) => (m.trim(), r.trim()),
        None => ("rgb", s),
    };
    // (values of \thisrow come in parentheses)
    let nums: Vec<f64> = rest.split(',').filter_map(|x| x.trim().trim_start_matches('(').trim_end_matches(')').trim().parse().ok()).collect();
    let n = |i: usize| nums.get(i).copied().unwrap_or(0.0).clamp(0.0, 1.0);
    Some(match model {
        "color" => return e.color(rest),
        "rgb" => Color { r: n(0), g: n(1), b: n(2) },
        "rgb255" => Color { r: nums.first()? / 255.0, g: nums.get(1)? / 255.0, b: nums.get(2)? / 255.0 },
        "gray" => Color { r: n(0), g: n(0), b: n(0) },
        "cmyk" => Color { r: (1.0 - n(0)) * (1.0 - n(3)), g: (1.0 - n(1)) * (1.0 - n(3)), b: (1.0 - n(2)) * (1.0 - n(3)) },
        "cmy" => Color { r: 1.0 - n(0), g: 1.0 - n(1), b: 1.0 - n(2) },
        "wave" => wave(*nums.first()?),
        _ => return None,
    })
}

/// xcolor's `wave` model: a wavelength in nm to RGB (Bruton's approximation, as xcolor does).
fn wave(l: f64) -> Color {
    let (r, g, b) = if l < 440.0 {
        ((440.0 - l) / 60.0, 0.0, 1.0)
    } else if l < 490.0 {
        (0.0, (l - 440.0) / 50.0, 1.0)
    } else if l < 510.0 {
        (0.0, 1.0, (510.0 - l) / 20.0)
    } else if l < 580.0 {
        ((l - 510.0) / 70.0, 1.0, 0.0)
    } else if l < 645.0 {
        (1.0, (645.0 - l) / 65.0, 0.0)
    } else {
        (1.0, 0.0, 0.0)
    };
    let f = if l < 420.0 {
        0.3 + 0.7 * (l - 380.0) / 40.0
    } else if l > 700.0 {
        0.3 + 0.7 * (780.0 - l) / 80.0
    } else {
        1.0
    };
    let c = |v: f64| (v * f).clamp(0.0, 1.0).powf(0.8);
    Color { r: c(r), g: c(g), b: c(b) }
}
