//! What happens to a path once built (`\pgfusepath`): decoration, rounded corners, shortening,
//! clipping, filling and stroking, arrow tips, `double`, and the picture's bounding box.

use crate::canvas::{Fill, Item, Paint, Stroke};
use crate::scene::{Point, Segment};
use crate::Error;

use super::path::{valid_segment, MAX_COORDINATE, MAX_DECORATION_STEPS};
use super::Engine;

impl Engine {
    /// Uses the path as the state says: clip, fill, draw, arrows.
    pub fn use_path(&mut self, segments: Vec<Segment>, command: &str) -> Result<(), Error> {
        if !segments.iter().all(valid_segment) {
            self.warn(format!("{command}: coordinate is not finite or beyond ±{MAX_COORDINATE} cm (division by zero?): path ignored"));
            return Ok(());
        }
        let mut segs = segments;
        if self.st.decorate && !self.st.decoration.is_empty() {
            let kind = self.st.decoration.clone();
            let mut warnings = Vec::new();
            segs = decorate(&segs, &kind, self.st.amplitude, self.st.segment_length, |w| warnings.push(w));
            for w in warnings {
                self.warn(w);
            }
        }
        if self.st.rounded > 0.0 {
            segs = round_corners(&segs, self.st.rounded);
        }
        if self.st.shorten_start > 0.0 || self.st.shorten_end > 0.0 {
            shorten_ends(&mut segs, self.st.shorten_start, self.st.shorten_end);
        }
        let mode = self.st.mode.clone();
        if mode.bounding_box {
            self.canvas.include_path(&segs);
            self.canvas_frozen = true;
        } else if !self.canvas_frozen {
            self.canvas.include_path(&segs);
            if mode.draw {
                // \pgfusepath adds half the line width around a stroked path
                self.canvas.include_path_margin(&segs, self.st.line_width / 2.0);
            }
        }
        let drawn = segs.iter().any(|s| !matches!(s, Segment::MoveTo(_)));
        let fill = if mode.fill || mode.pattern || mode.shade {
            let paint = if mode.pattern {
                Paint::Pattern {
                    name: self.st.pattern_name.clone().unwrap_or_else(|| "north east lines".into()),
                    color: self.st.pattern_color.unwrap_or(crate::color::Color::BLACK),
                }
            } else if mode.shade {
                // shadings come later: the fill color stands in
                Paint::Solid(self.st.fill_color.unwrap_or(crate::color::Color { r: 0.5, g: 0.5, b: 0.5 }))
            } else {
                Paint::Solid(self.st.fill_paint_color())
            };
            Some(Fill { paint, opacity: self.st.fill_opacity, even_odd: self.st.even_odd })
        } else {
            None
        };
        // a pattern is the fill: \fill[pattern=…] paints no solid color under it unless fill= names one
        let solid_under = if mode.pattern && mode.fill && self.st.fill_color.is_some() {
            Some(Fill { paint: Paint::Solid(self.st.fill_paint_color()), opacity: self.st.fill_opacity, even_odd: self.st.even_odd })
        } else {
            None
        };
        let stroke = mode.draw.then(|| self.stroke());
        if drawn && (fill.is_some() || stroke.is_some()) {
            let mut tips = Vec::new();
            if let Some(s) = &stroke {
                let default = self.st.default_arrow.clone();
                if let Some(spec) = self.st.end_arrow.clone() {
                    if let Some((p, d, k)) = end_point(&segs) {
                        let back = super::arrows::tip_items(&spec, &default, p, d, s, &mut tips);
                        if back > 0.0 {
                            pull_back_end(&mut segs, k, p, d, back);
                        }
                    }
                }
                if let Some(spec) = self.st.start_arrow.clone() {
                    if let Some((p, d)) = start_point(&segs) {
                        let back = super::arrows::tip_items(&spec, &default, p, d, s, &mut tips);
                        if back > 0.0 {
                            pull_back_start(&mut segs, p, d, back);
                        }
                    }
                }
            }
            if let Some(under) = solid_under {
                self.canvas.push(Item::Path { segments: segs.clone(), fill: Some(under), stroke: None });
            }
            match (self.st.double, &stroke) {
                (Some(core), Some(s)) => {
                    // two lines: the outer one, then the inner one in the double color
                    let outer = Stroke { width: 2.0 * s.width + self.st.double_distance, ..s.clone() };
                    let inner = Stroke { width: self.st.double_distance, color: core, ..s.clone() };
                    self.canvas.push(Item::Path { segments: segs.clone(), fill, stroke: Some(outer) });
                    self.canvas.push(Item::Path { segments: segs.clone(), fill: None, stroke: Some(inner) });
                }
                _ => self.canvas.push(Item::Path { segments: segs.clone(), fill, stroke }),
            }
            if !self.canvas_frozen {
                for t in &tips {
                    if let Item::Path { segments, .. } = t {
                        self.canvas.include_path(segments);
                    }
                }
            }
            self.canvas.items.extend(tips);
        }
        if mode.clip {
            self.open_in_scope(Item::Clip { segments: segs, even_odd: self.st.even_odd });
        }
        Ok(())
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
                    if p.minus(a).norm() > 1e-12 {
                        res = Some((*p, p.minus(a), k));
                    }
                }
                previous = Some(*p);
            }
            Segment::CurveTo(c1, c2, p) => {
                let d = if p.minus(*c2).norm() > 1e-9 {
                    p.minus(*c2)
                } else if p.minus(*c1).norm() > 1e-9 {
                    p.minus(*c1)
                } else {
                    p.minus(previous.unwrap_or(*p))
                };
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

fn pull_back_end(segs: &mut [Segment], k: usize, p: Point, d: Point, amount: f64) {
    let u = d.scale(1.0 / d.norm().max(1e-12));
    let new = p.minus(u.scale(amount));
    if let Segment::CurveTo(_, c2, q) = &mut segs[k] {
        *q = new;
        *c2 = c2.minus(u.scale(amount));
    } else {
        segs[k] = Segment::LineTo(new);
    }
}

fn pull_back_start(segs: &mut [Segment], p: Point, d: Point, amount: f64) {
    let u = d.scale(1.0 / d.norm().max(1e-12));
    segs[0] = Segment::MoveTo(p.minus(u.scale(amount)));
    if let Some(Segment::CurveTo(c1, _, _)) = segs.get_mut(1) {
        *c1 = c1.minus(u.scale(amount));
    }
}

/// Zigzag, snake (wave) or random steps over the straight segments.
fn decorate(segs: &[Segment], kind: &str, amp: f64, len_step: f64, mut warn: impl FnMut(String)) -> Vec<Segment> {
    if !matches!(kind, "zigzag" | "snake" | "random steps" | "saw" | "coil" | "bumps") {
        warn(format!("decoration '{kind}' not supported: path drawn undecorated"));
        return segs.to_vec();
    }
    let mut out = Vec::new();
    let mut current = Point::default();
    for s in segs {
        match s {
            Segment::LineTo(b) => {
                let a = current;
                let d = b.minus(a);
                let len = d.norm();
                if len < 1e-9 {
                    out.push(*s);
                    continue;
                }
                let u = d.scale(1.0 / len);
                let nrm = Point::new(-u.y, u.x);
                let n = if len_step > 0.0 { ((len / len_step).round() as usize).clamp(1, MAX_DECORATION_STEPS) } else { 1 };
                if kind == "random steps" {
                    // deterministic pseudo-random steps: the same input gives the same drawing
                    let mut seed: u32 = 0x9E37_79B9 ^ (out.len() as u32).wrapping_mul(2_654_435_761);
                    let mut random = || {
                        seed ^= seed << 13;
                        seed ^= seed >> 17;
                        seed ^= seed << 5;
                        (seed as f64 / u32::MAX as f64) * 2.0 - 1.0
                    };
                    for k in 1..=n {
                        let base = a.plus(d.scale(k as f64 / n as f64));
                        let p = if k == n { base } else { base.plus(Point::new(random() * amp, random() * amp)) };
                        out.push(Segment::LineTo(p));
                    }
                    current = *b;
                    continue;
                }
                let steps = n * if kind == "zigzag" || kind == "saw" { 2 } else { 8 };
                for k in 1..=steps {
                    let f = k as f64 / steps as f64;
                    let base = a.plus(d.scale(f));
                    let offset = match kind {
                        "zigzag" | "saw" => {
                            if k == steps {
                                0.0
                            } else if k % 2 == 1 {
                                amp
                            } else {
                                -amp
                            }
                        }
                        _ => amp * (f * n as f64 * std::f64::consts::TAU).sin(),
                    };
                    out.push(Segment::LineTo(base.plus(nrm.scale(offset))));
                }
                current = *b;
            }
            Segment::MoveTo(p) => {
                current = *p;
                out.push(*s);
            }
            Segment::CurveTo(_, _, p) => {
                current = *p;
                out.push(*s);
            }
            Segment::Close => out.push(*s),
        }
    }
    out
}

/// Rounded corners between straight segments.
pub(super) fn round_corners(segs: &[Segment], r: f64) -> Vec<Segment> {
    let mut points: Vec<Vec<Point>> = Vec::new();
    let mut closed: Vec<bool> = Vec::new();
    for s in segs {
        match s {
            Segment::MoveTo(p) => {
                points.push(vec![*p]);
                closed.push(false);
            }
            Segment::LineTo(p) => {
                if let Some(v) = points.last_mut() {
                    v.push(*p);
                }
            }
            Segment::Close => {
                if let Some(f) = closed.last_mut() {
                    *f = true;
                }
            }
            Segment::CurveTo(..) => return segs.to_vec(),
        }
    }
    let mut out = Vec::new();
    for (pts, is_closed) in points.iter().zip(closed) {
        let mut p: Vec<Point> = pts.clone();
        if is_closed && p.len() > 2 && p.first().map(|a| a.minus(*p.last().unwrap()).norm() <= 1e-9).unwrap_or(false) {
            p.pop();
        }
        let n = p.len();
        if n < 3 {
            out.push(Segment::MoveTo(p[0]));
            for q in &p[1..] {
                out.push(Segment::LineTo(*q));
            }
            if is_closed {
                out.push(Segment::Close);
            }
            continue;
        }
        let corner = |k: usize| -> (Point, Point, Point) {
            let prev = p[(k + n - 1) % n];
            let here = p[k];
            let next = p[(k + 1) % n];
            let d1 = prev.minus(here);
            let d2 = next.minus(here);
            let rr = r.min(d1.norm() / 2.0).min(d2.norm() / 2.0);
            (here.plus(d1.scale(rr / d1.norm().max(1e-12))), here, here.plus(d2.scale(rr / d2.norm().max(1e-12))))
        };
        let indices: Vec<usize> = if is_closed { (0..n).collect() } else { (1..n - 1).collect() };
        if is_closed {
            out.push(Segment::MoveTo(corner(0).2));
        } else {
            out.push(Segment::MoveTo(p[0]));
        }
        for &k in indices.iter().skip(if is_closed { 1 } else { 0 }) {
            let (a, v, b) = corner(k);
            out.push(Segment::LineTo(a));
            out.push(Segment::CurveTo(a.lerp(v, 0.5523), b.lerp(v, 0.5523), b));
        }
        if is_closed {
            let (a, v, b) = corner(0);
            out.push(Segment::LineTo(a));
            out.push(Segment::CurveTo(a.lerp(v, 0.5523), b.lerp(v, 0.5523), b));
            out.push(Segment::Close);
        } else {
            out.push(Segment::LineTo(p[n - 1]));
        }
    }
    out
}

fn shorten_ends(segs: &mut [Segment], start: f64, end: f64) {
    if segs.len() < 2 {
        return;
    }
    if let (Segment::MoveTo(a), Some(Segment::LineTo(b))) = (segs[0], segs.get(1).copied()) {
        let d = b.minus(a);
        let n = d.norm();
        if n > start && start > 0.0 {
            segs[0] = Segment::MoveTo(a.plus(d.scale(start / n)));
        }
    }
    let k = segs.len() - 1;
    if let (Segment::LineTo(b), Segment::MoveTo(a) | Segment::LineTo(a) | Segment::CurveTo(_, _, a)) = (segs[k], segs[k - 1]) {
        let d = b.minus(a);
        let n = d.norm();
        if n > end && end > 0.0 {
            segs[k] = Segment::LineTo(b.minus(d.scale(end / n)));
        }
    }
}
