//! From the scene to SVG. SVG units are points (1 cm = 28.4528 pt), y pointing down.

use std::collections::HashSet;
use std::fmt::Write;

use mathra::font::GlyphId;
use mathra::layout::Item;

use crate::color::Color;
use crate::scene::*;
use crate::PT;

const CM: f64 = 1.0 / PT; // points per cm

fn n(v: f64) -> String {
    let r = (v * 1000.0).round() / 1000.0;
    if r.fract() == 0.0 && r.abs() < 1e15 {
        format!("{}", r as i64)
    } else {
        format!("{r}")
    }
}

/// A scene point (cm, y up) in SVG coordinates.
fn sx(p: Point) -> (f64, f64) {
    (p.x * CM, -p.y * CM)
}

fn path_d(segs: &[Segment]) -> String {
    let mut d = String::new();
    for s in segs {
        match s {
            Segment::MoveTo(p) => {
                let (x, y) = sx(*p);
                let _ = write!(d, "M{} {} ", n(x), n(y));
            }
            Segment::LineTo(p) => {
                let (x, y) = sx(*p);
                let _ = write!(d, "L{} {} ", n(x), n(y));
            }
            Segment::CurveTo(a, b, p) => {
                let (ax, ay) = sx(*a);
                let (bx, by) = sx(*b);
                let (x, y) = sx(*p);
                let _ = write!(d, "C{} {} {} {} {} {} ", n(ax), n(ay), n(bx), n(by), n(x), n(y));
            }
            Segment::Close => d.push_str("Z "),
        }
    }
    d.trim_end().to_string()
}

struct Bounds {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}

impl Bounds {
    fn new() -> Bounds {
        Bounds { x0: f64::INFINITY, y0: f64::INFINITY, x1: f64::NEG_INFINITY, y1: f64::NEG_INFINITY }
    }
    fn include(&mut self, p: Point, pad: f64) {
        self.x0 = self.x0.min(p.x - pad);
        self.y0 = self.y0.min(p.y - pad);
        self.x1 = self.x1.max(p.x + pad);
        self.y1 = self.y1.max(p.y + pad);
    }
    fn valid(&self) -> bool {
        self.x0.is_finite() && self.x1 > self.x0 - 1e-9
    }
}

/// Arrow tip length, in cm.
fn tip_length(p: Tip, e: &Style) -> f64 {
    let lw = e.line_width;
    e.tip_scale
        * match p {
            Tip::Latex => 3.0 * PT + 2.5 * lw,
            Tip::Stealth => 3.0 * PT + 3.0 * lw,
            Tip::Default | Tip::Double => 1.8 * PT + 2.0 * lw,
            Tip::Bar | Tip::Circle => 1.5 * PT + 1.5 * lw,
        }
}

/// Last point of a path and the direction it arrives from.
fn end_point(segs: &[Segment]) -> Option<(Point, Point, usize)> {
    let mut previous = None;
    let mut res = None;
    for (k, s) in segs.iter().enumerate() {
        match s {
            Segment::MoveTo(p) => previous = Some(*p),
            Segment::LineTo(p) => {
                if let Some(a) = previous {
                    res = Some((*p, p.minus(a), k));
                }
                previous = Some(*p);
            }
            Segment::CurveTo(c1, c2, p) => {
                let d = if p.minus(*c2).norm() > 1e-9 { p.minus(*c2) } else { p.minus(*c1) };
                res = Some((*p, d, k));
                previous = Some(*p);
            }
            Segment::Close => return None,
        }
    }
    res
}

fn start_point(segs: &[Segment]) -> Option<(Point, Point)> {
    let Segment::MoveTo(a) = segs.first()? else { return None };
    match segs.get(1)? {
        Segment::LineTo(p) => Some((*a, a.minus(*p))),
        Segment::CurveTo(c1, c2, p) => {
            let target = if c1.minus(*a).norm() > 1e-9 {
                *c1
            } else if c2.minus(*a).norm() > 1e-9 {
                *c2
            } else {
                *p
            };
            Some((*a, a.minus(target)))
        }
        _ => None,
    }
}

/// Draws the tip with its point at `p`, pointing in direction `d`.
fn tip_svg(p: Point, d: Point, kind: Tip, e: &Style, color: &str) -> String {
    let len = tip_length(kind, e);
    let u = d.scale(1.0 / d.norm().max(1e-12));
    let nrm = Point::new(-u.y, u.x);
    let pts = |v: &[Point]| {
        v.iter()
            .map(|q| {
                let (x, y) = sx(*q);
                format!("{},{}", n(x), n(y))
            })
            .collect::<Vec<_>>()
            .join(" ")
    };
    match kind {
        Tip::Latex => {
            let base = p.minus(u.scale(len));
            let w = len * 0.42;
            format!(r#"<polygon points="{}" fill="{color}"/>"#, pts(&[p, base.plus(nrm.scale(w)), base.minus(nrm.scale(w))]))
        }
        Tip::Stealth => {
            let base = p.minus(u.scale(len));
            let w = len * 0.45;
            let mid = p.minus(u.scale(len * 0.7));
            format!(r#"<polygon points="{}" fill="{color}"/>"#, pts(&[p, base.plus(nrm.scale(w)), mid, base.minus(nrm.scale(w))]))
        }
        Tip::Default => {
            let base = p.minus(u.scale(len));
            let w = len * 0.75;
            format!(
                r#"<polyline points="{}" fill="none" stroke="{color}" stroke-width="{}" stroke-linecap="round" stroke-linejoin="round"/>"#,
                pts(&[base.plus(nrm.scale(w)), p, base.minus(nrm.scale(w))]),
                n(e.line_width * CM)
            )
        }
        Tip::Double => {
            let behind = p.minus(u.scale(len * 0.8));
            format!("{}{}", tip_svg(p, d, Tip::Default, e, color), tip_svg(behind, d, Tip::Default, e, color))
        }
        Tip::Bar => {
            let w = len;
            format!(r#"<polyline points="{}" fill="none" stroke="{color}" stroke-width="{}"/>"#, pts(&[p.plus(nrm.scale(w)), p.minus(nrm.scale(w))]), n(e.line_width * CM))
        }
        Tip::Circle => {
            let (x, y) = sx(p.minus(u.scale(len / 2.0)));
            format!(r#"<circle cx="{}" cy="{}" r="{}" fill="{color}"/>"#, n(x), n(y), n(len / 2.0 * CM))
        }
    }
}

/// Pulls back the end of the path (segment `k`, arriving at `p` in direction `d`) so the tip does not leak.
fn pull_back_end(segs: &mut [Segment], k: usize, p: Point, d: Point, amount: f64) {
    let u = d.scale(1.0 / d.norm().max(1e-12));
    let new = p.minus(u.scale(amount));
    // end_point only returns LineTo or CurveTo
    if let Segment::CurveTo(_, c2, q) = &mut segs[k] {
        *q = new;
        *c2 = c2.minus(u.scale(amount));
    } else {
        segs[k] = Segment::LineTo(new);
    }
}

/// Pulls back the start of the path (which leaves `p` in direction `-d`).
fn pull_back_start(segs: &mut [Segment], p: Point, d: Point, amount: f64) {
    let u = d.scale(1.0 / d.norm().max(1e-12));
    segs[0] = Segment::MoveTo(p.minus(u.scale(amount)));
    if let Some(Segment::CurveTo(c1, _, _)) = segs.get_mut(1) {
        *c1 = c1.minus(u.scale(amount));
    }
}

fn node_shape(node: &Node) -> Vec<Segment> {
    let c = node.center;
    let (w, h) = (node.half_width, node.half_height);
    let p = |x: f64, y: f64| c.plus(Point::new(x, y));
    match node.shape {
        Shape::Rectangle => {
            let r = node.style.rounded_corners.min(w).min(h);
            if r <= 0.0 {
                vec![Segment::MoveTo(p(-w, -h)), Segment::LineTo(p(w, -h)), Segment::LineTo(p(w, h)), Segment::LineTo(p(-w, h)), Segment::Close]
            } else {
                let k = r * 0.4477;
                vec![
                    Segment::MoveTo(p(-w + r, -h)),
                    Segment::LineTo(p(w - r, -h)),
                    Segment::CurveTo(p(w - k, -h), p(w, -h + k), p(w, -h + r)),
                    Segment::LineTo(p(w, h - r)),
                    Segment::CurveTo(p(w, h - k), p(w - k, h), p(w - r, h)),
                    Segment::LineTo(p(-w + r, h)),
                    Segment::CurveTo(p(-w + k, h), p(-w, h - k), p(-w, h - r)),
                    Segment::LineTo(p(-w, -h + r)),
                    Segment::CurveTo(p(-w, -h + k), p(-w + k, -h), p(-w + r, -h)),
                    Segment::Close,
                ]
            }
        }
        Shape::Diamond => vec![Segment::MoveTo(p(0.0, -h)), Segment::LineTo(p(w, 0.0)), Segment::LineTo(p(0.0, h)), Segment::LineTo(p(-w, 0.0)), Segment::Close],
        Shape::Circle | Shape::Ellipse => {
            let k = 0.5523;
            vec![
                Segment::MoveTo(p(w, 0.0)),
                Segment::CurveTo(p(w, h * k), p(w * k, h), p(0.0, h)),
                Segment::CurveTo(p(-w * k, h), p(-w, h * k), p(-w, 0.0)),
                Segment::CurveTo(p(-w, -h * k), p(-w * k, -h), p(0.0, -h)),
                Segment::CurveTo(p(w * k, -h), p(w, -h * k), p(w, 0.0)),
                Segment::Close,
            ]
        }
        Shape::Coordinate => Vec::new(),
    }
}

fn pattern_svg(name: &str, id: &str, color: &Color) -> String {
    let c = color.hex();
    let t = 3.0; // pt
    let line = |x1: f64, y1: f64, x2: f64, y2: f64| format!(r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{c}" stroke-width="0.4"/>"#, n(x1), n(y1), n(x2), n(y2));
    let body = match name {
        "north east lines" => line(0.0, t, t, 0.0) + &line(-t, t, t * 0.0, 0.0) + &line(t * 0.0, t * 2.0, t * 2.0, 0.0),
        "north west lines" => line(0.0, 0.0, t, t) + &line(-t, 0.0, 0.0, t) + &line(t * 0.0, -t, 2.0 * t, t),
        "horizontal lines" => line(0.0, t / 2.0, t, t / 2.0),
        "vertical lines" => line(t / 2.0, 0.0, t / 2.0, t),
        "grid" => line(0.0, t / 2.0, t, t / 2.0) + &line(t / 2.0, 0.0, t / 2.0, t),
        "crosshatch" => line(0.0, t, t, 0.0) + &line(0.0, 0.0, t, t),
        "dots" | "crosshatch dots" | "fivepointed stars" | "sixpointed stars" => format!(r#"<circle cx="{}" cy="{}" r="0.5" fill="{c}"/>"#, n(t / 2.0), n(t / 2.0)),
        _ => line(0.0, t, t, 0.0),
    };
    format!(r#"<pattern id="{id}" patternUnits="userSpaceOnUse" width="{}" height="{}">{body}</pattern>"#, n(t), n(t))
}

/// The items of a mathra box, placed with the baseline at `origin` (cm) and scale `scale` (cm per unit).
fn math_box_svg(c: &mathra::layout::MathBox, origin: Point, scale: f64, glyphs: &mut Vec<GlyphId>) -> String {
    let mut s = String::new();
    let k = scale * CM;
    for item in &c.items {
        match item {
            Item::Glyph { id, x, y, scale: gs, color } => {
                glyphs.push(*id);
                let (px, py) = sx(origin.plus(Point::new(*x as f64 * scale, *y as f64 * scale)));
                let f = *gs as f64 * k;
                let fill = color.as_ref().map(|c| format!(r#" fill="{c}""#)).unwrap_or_default();
                let _ = write!(s, r##"<use href="#tr-g{}" transform="translate({} {}) scale({} {})"{fill}/>"##, id.0, n(px), n(py), n(f), n(-f));
            }
            Item::Rule { x, y, width, height, color } => {
                let top = origin.plus(Point::new(*x as f64 * scale, (*y + *height) as f64 * scale));
                let (px, py) = sx(top);
                let fill = color.as_ref().map(|c| format!(r#" fill="{c}""#)).unwrap_or_default();
                let _ = write!(s, r#"<rect x="{}" y="{}" width="{}" height="{}"{fill}/>"#, n(px), n(py), n(*width as f64 * k), n(*height as f64 * k));
            }
            Item::Line { x1, y1, x2, y2, thickness, color } => {
                let (a, b) = (sx(origin.plus(Point::new(*x1 as f64 * scale, *y1 as f64 * scale))), sx(origin.plus(Point::new(*x2 as f64 * scale, *y2 as f64 * scale))));
                let _ = write!(
                    s,
                    r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}"/>"#,
                    n(a.0),
                    n(a.1),
                    n(b.0),
                    n(b.1),
                    color.as_deref().unwrap_or("currentColor"),
                    n(*thickness as f64 * k)
                );
            }
            Item::Ellipse { cx, cy, rx, ry, thickness, color } => {
                let (px, py) = sx(origin.plus(Point::new(*cx as f64 * scale, *cy as f64 * scale)));
                let _ = write!(
                    s,
                    r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="none" stroke="{}" stroke-width="{}"/>"#,
                    n(px),
                    n(py),
                    n(*rx as f64 * k),
                    n(*ry as f64 * k),
                    color.as_deref().unwrap_or("currentColor"),
                    n(*thickness as f64 * k)
                );
            }
        }
    }
    s
}

fn label_svg(node: &Node, glyphs: &mut Vec<GlyphId>) -> String {
    let Some(r) = &node.label else { return String::new() };
    let height = r.height();
    let total = height + r.depth();
    let width = r.width();
    let top = node.center.y + total / 2.0;
    let mut s = format!(r#"<g fill="{}">"#, r.color.hex());
    for (k, line) in r.lines.iter().enumerate() {
        let w = line.width as f64 * r.scale;
        let x = match r.align {
            'l' => node.center.x - width / 2.0,
            'r' => node.center.x + width / 2.0 - w,
            _ => node.center.x - w / 2.0,
        };
        let base = top - height - k as f64 * r.line_height;
        s.push_str(&math_box_svg(line, Point::new(x, base), r.scale, glyphs));
    }
    s.push_str("</g>");
    s
}

fn escape(t: &str) -> String {
    t.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

pub fn svg(scene: &Scene, source: &str, description: &str) -> String {
    // bounds
    let mut lim = Bounds::new();
    for el in &scene.elements {
        match el {
            Element::Path(t) => {
                let pad = t.style.line_width / 2.0 + if t.style.start_tip.is_some() || t.style.end_tip.is_some() { 2.0 * PT } else { 0.0 };
                // control points go past the stroke: the curves are sampled
                include_curves(&t.segments, &mut lim, pad);
            }
            Element::Node(node) => {
                let f = node.style.line_width / 2.0;
                let (w, h) = (node.half_width, node.half_height);
                lim.include(node.center.plus(Point::new(-w, -h)), f);
                lim.include(node.center.plus(Point::new(w, h)), f);
            }
            _ => {}
        }
    }
    if !lim.valid() {
        lim = Bounds { x0: 0.0, y0: 0.0, x1: 0.1, y1: 0.1 };
    }
    let m = scene.margin;
    let (x0, x1, y0, y1) = (lim.x0 - m, lim.x1 + m, lim.y0 - m, lim.y1 + m);
    let (width, height) = ((x1 - x0) * CM, (y1 - y0) * CM);

    let mut body = String::new();
    let mut defs = String::new();
    let mut glyphs = Vec::new();
    let mut n_pattern = 0;
    let mut n_clip = 0;
    for el in &scene.elements {
        match el {
            Element::Path(t) => {
                let e = &t.style;
                let mut segs = t.segments.clone();
                let stroke_color = e.stroke.map(|c| c.hex());
                let mut tips = String::new();
                if let Some(c) = &stroke_color {
                    if let Some(p) = e.end_tip {
                        if let Some((pt, d, k)) = end_point(&segs) {
                            tips.push_str(&tip_svg(pt, d, p, e, c));
                            if matches!(p, Tip::Latex | Tip::Stealth) {
                                pull_back_end(&mut segs, k, pt, d, tip_length(p, e) * 0.85);
                            }
                        }
                    }
                    if let Some(p) = e.start_tip {
                        if let Some((pt, d)) = start_point(&segs) {
                            tips.push_str(&tip_svg(pt, d, p, e, c));
                            if matches!(p, Tip::Latex | Tip::Stealth) {
                                pull_back_start(&mut segs, pt, d, tip_length(p, e) * 0.85);
                            }
                        }
                    }
                }
                let d = path_d(&segs);
                if let Some(c) = e.fill {
                    let op = if e.fill_opacity < 1.0 { format!(r#" fill-opacity="{}""#, n(e.fill_opacity)) } else { String::new() };
                    let _ = write!(body, r#"<path d="{d}" fill="{}"{op}/>"#, c.hex());
                }
                if let Some((name, color)) = &e.pattern {
                    n_pattern += 1;
                    let id = format!("tr-p{n_pattern}");
                    defs.push_str(&pattern_svg(name, &id, color));
                    let _ = write!(body, r#"<path d="{d}" fill="url(#{id})"/>"#);
                }
                if let Some(c) = &stroke_color {
                    let dash = e.dash.as_ref().map(|v| format!(r#" stroke-dasharray="{}""#, v.iter().map(|x| n(x * CM)).collect::<Vec<_>>().join(" "))).unwrap_or_default();
                    let op = if e.stroke_opacity < 1.0 { format!(r#" stroke-opacity="{}""#, n(e.stroke_opacity)) } else { String::new() };
                    let _ = write!(body, r#"<path d="{d}" fill="none" stroke="{c}" stroke-width="{}"{dash}{op} stroke-linejoin="round"/>"#, n(e.line_width * CM));
                    body.push_str(&tips);
                }
            }
            Element::Node(node) => {
                let segs = node_shape(node);
                let rot = if node.rotation != 0.0 {
                    let (cx, cy) = sx(node.center);
                    format!(r#" transform="rotate({} {} {})""#, n(-node.rotation), n(cx), n(cy))
                } else {
                    String::new()
                };
                let _ = write!(body, "<g{rot}>");
                if !segs.is_empty() {
                    let d = path_d(&segs);
                    if let Some(c) = node.style.fill {
                        let _ = write!(body, r#"<path d="{d}" fill="{}"/>"#, c.hex());
                    }
                    if let Some(c) = node.style.stroke {
                        let dash =
                            node.style.dash.as_ref().map(|v| format!(r#" stroke-dasharray="{}""#, v.iter().map(|x| n(x * CM)).collect::<Vec<_>>().join(" "))).unwrap_or_default();
                        let _ = write!(body, r#"<path d="{d}" fill="none" stroke="{}" stroke-width="{}"{dash}/>"#, c.hex(), n(node.style.line_width * CM));
                    }
                }
                body.push_str(&label_svg(node, &mut glyphs));
                body.push_str("</g>");
            }
            Element::Clip(segs) => {
                n_clip += 1;
                let _ = write!(defs, r#"<clipPath id="tr-c{n_clip}"><path d="{}"/></clipPath>"#, path_d(segs));
                let _ = write!(body, r#"<g clip-path="url(#tr-c{n_clip})">"#);
            }
            Element::EndClip => body.push_str("</g>"),
        }
    }
    let font = mathra::font::font();
    let mut seen = HashSet::new();
    for g in glyphs {
        if seen.insert(g.0) {
            let _ = write!(defs, r#"<path id="tr-g{}" d="{}"/>"#, g.0, font.outline(g));
        }
    }
    let mut s = String::new();
    let _ = write!(
        s,
        r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:tikra="urn:tikra:1" viewBox="{} {} {} {}" width="{}pt" height="{}pt" role="img" aria-labelledby="tr-title">"#,
        n(x0 * CM),
        n(-y1 * CM),
        n(width),
        n(height),
        n(width),
        n(height)
    );
    let _ = write!(s, r#"<title id="tr-title">{}</title>"#, escape(description));
    let _ = write!(s, r#"<metadata><tikra:source type="tikz">{}</tikra:source></metadata>"#, escape(source));
    let _ = write!(s, "<defs>{defs}</defs>{body}</svg>");
    s
}

/// Adds the curve points (sampled) to the bounds, not the control points.
fn include_curves(segs: &[Segment], lim: &mut Bounds, pad: f64) {
    let mut current = Point::default();
    for s in segs {
        match s {
            Segment::MoveTo(p) | Segment::LineTo(p) => {
                lim.include(*p, pad);
                current = *p;
            }
            Segment::CurveTo(a, b, p) => {
                for k in 1..8 {
                    let t = k as f64 / 8.0;
                    let mt = 1.0 - t;
                    let q = current.scale(mt * mt * mt).plus(a.scale(3.0 * mt * mt * t)).plus(b.scale(3.0 * mt * t * t)).plus(p.scale(t * t * t));
                    lim.include(q, pad);
                }
                lim.include(*p, pad);
                current = *p;
            }
            Segment::Close => {}
        }
    }
}
