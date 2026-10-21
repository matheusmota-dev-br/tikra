//! What happens to a path once built (`\pgfusepath`): decoration, rounded corners, shortening,
//! clipping, filling and stroking, arrow tips, `double`, and the picture's bounding box.

use crate::canvas::{Fill, Item, Paint, Stroke};
use crate::scene::{Point, Segment};
use crate::Error;

use super::arrows::{EndTips, TipsMode, Widths};
use super::path::{valid_segment, MAX_COORDINATE, MAX_DECORATION_STEPS};
use super::Engine;
use crate::PT;

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
        let mode = self.st.mode.clone();
        // the bounding box grows as the path is built: before any shortening
        if mode.bounding_box {
            self.canvas.include_path(&segs);
            self.canvas_frozen = true;
        } else if !self.canvas_frozen {
            self.canvas.include_path(&segs);
            // a stroked path adds half its line width around its own box (\pgfusepath)
            if mode.draw {
                let w = match self.st.double {
                    Some(_) => 2.0 * self.st.line_width + self.st.double_distance,
                    None => self.st.line_width,
                };
                let mut path = crate::canvas::Canvas::default();
                path.include_path(&segs);
                if let (Some(lo), Some(hi)) = (path.min, path.max) {
                    self.canvas.include(Point::new(lo.x - w / 2.0, lo.y - w / 2.0));
                    self.canvas.include(Point::new(hi.x + w / 2.0, hi.y + w / 2.0));
                }
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
        // `tips=true`/`proper`: tips on a path that is neither filled nor stroked
        let tips_only = stroke.is_none() && fill.is_none() && !mode.clip && matches!(self.st.tips_mode, TipsMode::True | TipsMode::Proper);
        // a clipping path has no tips and is not shortened
        let mut tips = if drawn && !mode.clip && (stroke.is_some() || tips_only) { self.arrow_ends(&mut segs) } else { Tips::default() };
        if tips_only {
            self.push_tips(std::mem::take(&mut tips));
        }
        if drawn && (fill.is_some() || stroke.is_some()) {
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
            self.push_tips(tips);
        }
        if mode.clip {
            self.open_in_scope(Item::Clip { segments: segs, even_odd: self.st.even_odd });
        }
        Ok(())
    }

    /// Shortens the path for its arrow tips and `shorten <`/`shorten >`, and draws the tips, as
    /// `\pgf@prepare@end@of@path` and `\pgf@prepare@start@of@path` do: the end first, then the start
    /// of the path as the end left it. Nothing on a closed path.
    fn arrow_ends(&mut self, segs: &mut [Segment]) -> Tips {
        let mut out = Tips::default();
        if segs.iter().any(|s| matches!(s, Segment::Close)) || self.st.tips_mode == TipsMode::Never {
            return out;
        }
        // `tips=proper`, `on proper draw`: not on a path that stays at one point
        if matches!(self.st.tips_mode, TipsMode::Proper | TipsMode::OnProperDraw) && !proper(segs) {
            return out;
        }
        let s = self.stroke();
        let widths = match self.st.double {
            Some(_) => Widths { line: (2.0 * s.width + self.st.double_distance) / PT, inner: self.st.double_distance / PT },
            None => Widths { line: s.width / PT, inner: 0.0 },
        };
        let user = self.st.tips.clone();
        let mut warn = Vec::new();
        let (end_spec, start_spec) = (self.st.end_arrow.clone().unwrap_or_default(), self.st.start_arrow.clone().unwrap_or_default());
        let (shorten_end, shorten_start) = (self.st.shorten_end / PT, self.st.shorten_start / PT);
        let end = EndTips::new(self, &end_spec, false, &user, widths, shorten_end, &mut warn);
        let start = EndTips::new(self, &start_spec, true, &user, widths, shorten_start, &mut warn);
        for w in warn {
            self.warn(w);
        }
        let mut at_end = None;
        if !end.is_empty() || end.shortening != 0.0 {
            if let Some((k, movable, last, reference)) = path_end(segs) {
                if movable && end.shortening != 0.0 {
                    let p = toward(last, reference, end.shortening * PT);
                    if let Segment::LineTo(q) | Segment::CurveTo(_, _, q) = &mut segs[k] {
                        *q = p;
                    }
                }
                at_end = Some((last, reference));
            }
        }
        let mut at_start = None;
        if !start.is_empty() || start.shortening != 0.0 {
            if let Some((m, movable, first, reference)) = path_start(segs) {
                if movable && start.shortening != 0.0 {
                    segs[m] = Segment::MoveTo(toward(first, reference, start.shortening * PT));
                }
                at_start = Some((first, reference));
            }
        }
        // the stroke color and line width, without the dash; fills with the fill opacity
        let stroke = Stroke { width: widths.line * PT, ..s };
        let fill_opacity = self.st.fill_opacity;
        for (tips, at) in [(&start, at_start), (&end, at_end)] {
            if let Some((p, from)) = at {
                let hull = tips.draw(p, from, &stroke, fill_opacity, &mut out.items);
                out.hull.extend(hull);
            }
        }
        out
    }

    fn push_tips(&mut self, tips: Tips) {
        if !self.canvas_frozen {
            for p in tips.hull {
                self.canvas.include(p);
            }
        }
        self.canvas.items.extend(tips.items);
    }
}

/// The drawing of a path's tips, and the points of their hulls (for the bounding box).
#[derive(Default)]
struct Tips {
    items: Vec<Item>,
    hull: Vec<Point>,
}

/// The point at `distance` from `p` toward `q` (`\pgfpointlineatdistance`). When they coincide pgf's
/// normalized vector points up.
fn toward(p: Point, q: Point, distance: f64) -> Point {
    let d = q.minus(p);
    let u = if d.norm() > 1e-12 { d.scale(1.0 / d.norm()) } else { Point::new(0.0, 1.0) };
    p.plus(u.scale(distance))
}

fn point_of(s: &Segment) -> Option<Point> {
    match s {
        Segment::MoveTo(p) | Segment::LineTo(p) | Segment::CurveTo(_, _, p) => Some(*p),
        Segment::Close => None,
    }
}

/// Where the last subpath (the only one with tips, as pgf splits the path) begins.
fn last_subpath(segs: &[Segment]) -> Option<usize> {
    segs.iter().rposition(|s| matches!(s, Segment::MoveTo(_)))
}

/// The end of the path (`\pgf@parse@end`): the last segment, if it may be shortened (not a lone
/// point), its end point, and the point the tip is aimed from: the previous point, or the last
/// control point of a curve, the one before if they coincide. pgf moves only the end point.
fn path_end(segs: &[Segment]) -> Option<(usize, bool, Point, Point)> {
    let k = segs.len().checked_sub(1)?;
    match segs[k] {
        Segment::MoveTo(p) => Some((k, false, p, p)),
        Segment::LineTo(last) => Some((k, true, last, point_of(&segs[k.checked_sub(1)?])?)),
        Segment::CurveTo(c1, c2, last) => {
            let r = if c2 != last {
                c2
            } else if c1 != last {
                c1
            } else {
                point_of(&segs[k.checked_sub(1)?])?
            };
            Some((k, true, last, r))
        }
        Segment::Close => None,
    }
}

/// The start of the last subpath (`\pgf@parse@start`): its index, if it may be shortened, the first
/// point and the point the tip is aimed from: the next point, or the first control point of a curve
/// (then the second, then its end, while they coincide).
fn path_start(segs: &[Segment]) -> Option<(usize, bool, Point, Point)> {
    let m = last_subpath(segs)?;
    let first = point_of(&segs[m])?;
    match segs.get(m + 1) {
        None => Some((m, false, first, first)),
        Some(&Segment::LineTo(p)) => Some((m, true, first, p)),
        Some(&Segment::CurveTo(c1, c2, p)) => Some((
            m,
            true,
            first,
            if c1 != first {
                c1
            } else if c2 != first {
                c2
            } else {
                p
            },
        )),
        _ => None,
    }
}

/// A path that leaves its first point (`\pgf@path@check@proper`).
fn proper(segs: &[Segment]) -> bool {
    let points: Vec<Point> = segs
        .iter()
        .flat_map(|s| match *s {
            Segment::MoveTo(p) | Segment::LineTo(p) => vec![p],
            Segment::CurveTo(a, b, p) => vec![a, b, p],
            Segment::Close => vec![],
        })
        .collect();
    points.iter().any(|p| Some(p) != points.first())
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
fn round_corners(segs: &[Segment], r: f64) -> Vec<Segment> {
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
