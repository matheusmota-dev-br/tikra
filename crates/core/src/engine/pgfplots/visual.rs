//! Drawing one plot (`\pgfplots@stored@plotlist@visualize@selected@phases`): the plot handlers
//! (lines, `smooth`, `const plot`, `jump mark`, combs, `ybar`/`xbar`, intervals), `\closedcycle`,
//! `fill between`, error bars, and the marks drawn afterwards, as pgf's mark handler draws them.

use crate::canvas::{Fill, Item, Paint};
use crate::scene::{Point, Segment, Transform};
use crate::tex::{self, Token};
use crate::{Error, PT};

use super::draw::Frame;
use super::keys;
use super::marks::{self, Use};
use super::plot::Plot;
use super::{toks, Engine};

/// The marks of a plot, drawn once all plots are: its style and the canvas points.
#[derive(Debug, Clone)]
pub struct Marks {
    pub style: Vec<Token>,
    pub points: Vec<Point>,
}

/// The canvas points of a plot (None at jumps).
fn canvas_points(f: &Frame, pts: &[[f64; 3]]) -> Vec<Option<Point>> {
    pts.iter()
        .map(|l| {
            let z = if f.three_d { l[2] } else { f.min[2] };
            if l[0].is_finite() && l[1].is_finite() && z.is_finite() {
                Some(f.at([l[0], l[1], z]))
            } else {
                None
            }
        })
        .collect()
}

/// pgf's curveto handler: support points at `tension` times the neighbours' difference.
fn smooth(points: &[Point], tension: f64, segs: &mut Vec<Segment>) {
    let t = 0.2775 * tension;
    if points.len() < 3 {
        for p in &points[1..] {
            segs.push(Segment::LineTo(*p));
        }
        return;
    }
    let mut first_support = points[0];
    for i in 1..points.len() - 1 {
        let d = points[i + 1].minus(points[i - 1]).scale(t);
        segs.push(Segment::CurveTo(first_support, points[i].minus(d), points[i]));
        first_support = points[i].plus(d);
    }
    let last = points[points.len() - 1];
    segs.push(Segment::CurveTo(first_support, last, last));
}

/// The pieces of a plot between jumps.
fn runs(points: &[Option<Point>]) -> Vec<Vec<Point>> {
    let mut out = vec![Vec::new()];
    for p in points {
        match p {
            Some(p) => out.last_mut().expect("a run").push(*p),
            None => {
                if !out.last().is_some_and(Vec::is_empty) {
                    out.push(Vec::new());
                }
            }
        }
    }
    out.into_iter().filter(|r| !r.is_empty()).collect()
}

/// Draws a plot; returns its marks, for later.
#[allow(clippy::too_many_arguments)]
pub fn draw_plot(e: &mut Engine, p: &Plot, pts: &[[f64; 3]], base: &[f64], f: &Frame, of_type: usize, index_of_type: usize) -> Result<Option<Marks>, Error> {
    e.begin_group();
    let r = draw_plot_in_group(e, p, pts, base, f, of_type, index_of_type);
    e.end_group();
    r
}

fn define(e: &mut Engine, name: &str, value: &str) {
    e.tex.set_meaning(&Token::cs(name), super::super::macro_from(value), false);
}

fn draw_plot_in_group(e: &mut Engine, p: &Plot, pts: &[[f64; 3]], base: &[f64], f: &Frame, of_type: usize, index_of_type: usize) -> Result<Option<Marks>, Error> {
    define(e, "numplotsofactualtype", &of_type.to_string());
    define(e, "plotnumofactualtype", &index_of_type.to_string());
    define(e, "plotnum", &p.index.to_string());
    e.st.mode = Default::default();
    e.st.pending = Transform::default();
    let mut style = toks("/tikz/draw,");
    style.extend(p.style.clone());
    e.process_keys(style.clone(), Some("/pgfplots"))?;
    e.st.apply_pending();
    if let Some(code) = keys::value_tokens(e, "/pgfplots/execute at begin plot visualization") {
        e.run_tokens(code)?;
    }
    let handler = keys::value(e, "/tikz/@plot handler");
    let cp = canvas_points(f, pts);
    let mut segs: Vec<Segment> = Vec::new();
    let bar_width = keys::length(e, "/pgf/bar width").unwrap_or(10.0 * PT);
    define(e, "pgfplotbarwidth", &format!("{}pt", bar_width / PT));
    let bar_shift = keys::length(e, "/pgf/bar shift").unwrap_or(0.0);
    define(e, "pgfplotbarshift", &format!("{}pt", bar_shift / PT));
    let stacked = p.stack.is_some();
    // the zero level bars and combs stand on
    let level = |i: usize, axis: usize| -> f64 {
        if stacked && p.stack == Some(['x', 'y'][axis]) {
            base.get(i).copied().unwrap_or(0.0)
        } else {
            f.zero[axis]
        }
    };
    let at = |x: f64, y: f64| f.at([x, y, f.min[2]]);
    match handler.as_str() {
        "ybar" | "xbar" => {
            let vertical = handler == "ybar";
            let (ua, _) = if vertical { (unit(f.e[0]), unit(f.e[1])) } else { (unit(f.e[1]), unit(f.e[0])) };
            for (i, l) in pts.iter().enumerate() {
                if !(l[0].is_finite() && l[1].is_finite()) {
                    continue;
                }
                let (from, to) = if vertical { (at(l[0], level(i, 1)), at(l[0], l[1])) } else { (at(level(i, 0), l[1]), at(l[0], l[1])) };
                let c0 = from.plus(ua.scale(bar_shift - bar_width / 2.0));
                let c1 = to.plus(ua.scale(bar_shift + bar_width / 2.0));
                rect(&mut segs, c0, c1);
            }
        }
        "ybar interval" | "xbar interval" => {
            let vertical = handler == "ybar interval";
            let w = keys::number(e, "/pgf/bar interval width").unwrap_or(1.0);
            let sh = keys::number(e, "/pgf/bar interval shift").unwrap_or(0.5);
            for i in 0..pts.len().saturating_sub(1) {
                let (l, n) = (pts[i], pts[i + 1]);
                if !(l[0].is_finite() && l[1].is_finite() && n[0].is_finite() && n[1].is_finite()) {
                    continue;
                }
                if vertical {
                    let d = n[0] - l[0];
                    let x0 = l[0] + d * (sh - w / 2.0);
                    rect(&mut segs, at(x0, level(i, 1)), at(x0 + d * w, l[1]));
                } else {
                    let d = n[1] - l[1];
                    let y0 = l[1] + d * (sh - w / 2.0);
                    rect(&mut segs, at(level(i, 0), y0), at(l[0], y0 + d * w));
                }
            }
        }
        "ycomb" | "xcomb" => {
            for (i, l) in pts.iter().enumerate() {
                if !(l[0].is_finite() && l[1].is_finite()) {
                    continue;
                }
                let from = if handler == "ycomb" { at(l[0], level(i, 1)) } else { at(level(i, 0), l[1]) };
                segs.push(Segment::MoveTo(from));
                segs.push(Segment::LineTo(at(l[0], l[1])));
            }
        }
        h if h.starts_with("const") || h.starts_with("jump") => {
            for run in runs(&cp) {
                segs.push(Segment::MoveTo(run[0]));
                for w in run.windows(2) {
                    let (a, b) = (w[0], w[1]);
                    match h {
                        "const" => {
                            segs.push(Segment::LineTo(Point::new(b.x, a.y)));
                            segs.push(Segment::LineTo(b));
                        }
                        "const right" => {
                            segs.push(Segment::LineTo(Point::new(a.x, b.y)));
                            segs.push(Segment::LineTo(b));
                        }
                        "const mid" => {
                            let m = (a.x + b.x) / 2.0;
                            segs.push(Segment::LineTo(Point::new(m, a.y)));
                            segs.push(Segment::LineTo(Point::new(m, b.y)));
                            segs.push(Segment::LineTo(b));
                        }
                        "jump left" => {
                            segs.push(Segment::LineTo(Point::new(b.x, a.y)));
                            segs.push(Segment::MoveTo(b));
                        }
                        "jump right" => {
                            segs.push(Segment::MoveTo(Point::new(a.x, b.y)));
                            segs.push(Segment::LineTo(b));
                        }
                        _ => {
                            let m = (a.x + b.x) / 2.0;
                            segs.push(Segment::LineTo(Point::new(m, a.y)));
                            segs.push(Segment::MoveTo(Point::new(m, b.y)));
                            segs.push(Segment::LineTo(b));
                        }
                    }
                }
            }
        }
        _ => {
            let tension = e.st.tension;
            for run in runs(&cp) {
                segs.push(Segment::MoveTo(run[0]));
                if e.st.smooth {
                    smooth(&run, tension, &mut segs);
                } else {
                    for q in &run[1..] {
                        segs.push(Segment::LineTo(*q));
                    }
                }
            }
        }
    }
    // \closedcycle: down to the zero level (or the previous stacked plot) and back
    let closed = p.post.iter().any(|t| t.is_cs("closedcycle"));
    if closed {
        if let (Some(first), Some(last)) = (pts.iter().find(|l| l[0].is_finite()), pts.iter().rev().find(|l| l[0].is_finite())) {
            if stacked && p.stack == Some('y') {
                for (i, l) in pts.iter().enumerate().rev() {
                    if l[0].is_finite() {
                        segs.push(Segment::LineTo(at(l[0], base.get(i).copied().unwrap_or(f.zero[1]))));
                    }
                }
            } else {
                segs.push(Segment::LineTo(at(last[0], f.zero[1])));
                segs.push(Segment::LineTo(at(first[0], f.zero[1])));
            }
            segs.push(Segment::Close);
        }
    }
    if let Some(spec) = &p.fill_between {
        segs = fill_between(e, spec)?;
    }
    if let Some(name) = e.st.name_path.take() {
        e.named_paths.insert(name, segs.clone());
    }
    let only_marks = e.st.only_marks;
    let mark = e.st.plot_mark.clone();
    if !only_marks && !segs.is_empty() {
        let draw = e.st.mode.draw;
        // a mark-only path does not need the line
        e.use_path(segs.clone(), "addplot")?;
        let _ = draw;
    }
    // the rest of the path: nodes at the last point
    let rest: Vec<Token> = p.post.iter().filter(|t| !t.is_cs("closedcycle")).cloned().collect();
    if rest.iter().any(|t| !t.is_space()) {
        if let Some(last) = cp.iter().rev().flatten().next() {
            let mut code = toks(&format!("\\path[draw=none,fill=none] (tikra cs:{}pt,{}pt) ", last.x / PT, last.y / PT));
            code.extend(rest);
            code.push(Token::other(';'));
            e.run_tokens(code)?;
        }
    }
    error_bars(e, p, pts, f)?;
    match mark {
        Some(_) => Ok(Some(Marks { style, points: cp.into_iter().flatten().collect() })),
        None => Ok(None),
    }
}

fn unit(p: Point) -> Point {
    p.scale(1.0 / p.norm().max(1e-12))
}

fn rect(segs: &mut Vec<Segment>, a: Point, b: Point) {
    segs.push(Segment::MoveTo(a));
    segs.push(Segment::LineTo(Point::new(b.x, a.y)));
    segs.push(Segment::LineTo(b));
    segs.push(Segment::LineTo(Point::new(a.x, b.y)));
    segs.push(Segment::Close);
}

/// `fill between[of=A and B]`: the area between two named paths.
fn fill_between(e: &mut Engine, spec: &[Token]) -> Result<Vec<Segment>, Error> {
    let s = tex::tokens_to_string(spec);
    let of = s.split(',').find_map(|p| p.trim().strip_prefix("of").map(|r| r.trim().trim_start_matches('=').trim().to_string())).unwrap_or_default();
    let Some((a, b)) = of.split_once(" and ") else {
        e.warn("fill between: 'of=A and B' missing");
        return Ok(Vec::new());
    };
    let (Some(pa), Some(pb)) = (e.named_paths.get(a.trim()).cloned(), e.named_paths.get(b.trim()).cloned()) else {
        e.warn(format!("fill between: no path named '{}' or '{}'", a.trim(), b.trim()));
        return Ok(Vec::new());
    };
    let mut segs = pa.clone();
    segs.retain(|s| !matches!(s, Segment::Close));
    let rev = reverse(&pb);
    for (i, s) in rev.into_iter().enumerate() {
        segs.push(if i == 0 {
            if let Segment::MoveTo(p) = s {
                Segment::LineTo(p)
            } else {
                s
            }
        } else {
            s
        });
    }
    segs.push(Segment::Close);
    Ok(segs)
}

/// A path walked backwards.
fn reverse(segs: &[Segment]) -> Vec<Segment> {
    let mut pts: Vec<(Point, Option<(Point, Point)>)> = Vec::new();
    for s in segs {
        match s {
            Segment::MoveTo(p) | Segment::LineTo(p) => pts.push((*p, None)),
            Segment::CurveTo(a, b, p) => pts.push((*p, Some((*a, *b)))),
            Segment::Close => {}
        }
    }
    let mut out = Vec::new();
    if let Some(last) = pts.last() {
        out.push(Segment::MoveTo(last.0));
    }
    for i in (1..pts.len()).rev() {
        match pts[i].1 {
            Some((a, b)) => out.push(Segment::CurveTo(b, a, pts[i - 1].0)),
            None => out.push(Segment::LineTo(pts[i - 1].0)),
        }
    }
    out
}

/// Error bars: a line from minus to plus error, with `error mark` at its ends.
fn error_bars(e: &mut Engine, p: &Plot, pts: &[[f64; 3]], f: &Frame) -> Result<(), Error> {
    let ydir = keys::value(e, "/pgfplots/error bars/y dir");
    let xdir = keys::value(e, "/pgfplots/error bars/x dir");
    if ydir == "none" && xdir == "none" {
        return Ok(());
    }
    let mut segs = Vec::new();
    let mut caps = Vec::new();
    for (axis, dir) in [(1usize, ydir), (0usize, xdir)] {
        if dir == "none" {
            continue;
        }
        let c = ['x', 'y'][axis];
        let mode = keys::value(e, &format!("/pgfplots/error bars/@{c} mode"));
        let fixed = keys::number(e, &format!("/pgfplots/error bars/@{c} value")).unwrap_or(0.0);
        for (i, l) in pts.iter().enumerate() {
            if !(l[0].is_finite() && l[1].is_finite()) {
                continue;
            }
            let raw = p.coords.get(i).map_or(f64::NAN, |c| [c.x, c.y][axis]);
            let (em, ep) = match mode.as_str() {
                "explicit" => (p.coords[i].err[axis * 2], p.coords[i].err[axis * 2 + 1]),
                "explicit relative" => (p.coords[i].err[axis * 2] * raw.abs(), p.coords[i].err[axis * 2 + 1] * raw.abs()),
                "fixed relative" => (fixed * raw.abs(), fixed * raw.abs()),
                _ => (fixed, fixed),
            };
            let (em, ep) = (if em.is_finite() { em.abs() } else { 0.0 }, if ep.is_finite() { ep.abs() } else { 0.0 });
            if em == 0.0 && ep == 0.0 {
                continue;
            }
            let lo = if dir == "plus" { raw } else { raw - em };
            let hi = if dir == "minus" { raw } else { raw + ep };
            let mut a = *l;
            let mut b = *l;
            a[axis] = f.logical(axis, lo);
            b[axis] = f.logical(axis, hi);
            let (pa, pb) = (f.at(a), f.at(b));
            segs.push(Segment::MoveTo(pa));
            segs.push(Segment::LineTo(pb));
            let d = unit(pb.minus(pa));
            let n = Point::new(-d.y, d.x).scale(2.0 * PT);
            for q in [pa, pb] {
                caps.push(Segment::MoveTo(q.minus(n)));
                caps.push(Segment::LineTo(q.plus(n)));
            }
        }
    }
    segs.extend(caps);
    if segs.is_empty() {
        return Ok(());
    }
    e.begin_group();
    let r = (|| {
        e.st.mode = Default::default();
        e.st.mode.draw = true;
        e.st.dash = None;
        e.process_keys(toks("/pgfplots/every error bar"), Some("/pgfplots"))?;
        e.use_path(segs, "error bars")
    })();
    e.end_group();
    r
}

/// Draws the marks of a plot, as `\pgfplothandlermark` does: `every mark` and `mark options`
/// apply, each mark is a path at the point.
pub fn draw_marks(e: &mut Engine, m: &Marks) -> Result<(), Error> {
    e.begin_group();
    let r = (|| -> Result<(), Error> {
        e.st.mode = Default::default();
        e.st.pending = Transform::default();
        e.process_keys(m.style.clone(), Some("/pgfplots"))?;
        e.st.apply_pending();
        let Some(name) = e.st.plot_mark.clone() else { return Ok(()) };
        e.st.pending = Transform::default();
        e.process_keys(toks("every mark"), Some("/tikz"))?;
        let local = e.st.pending;
        let size = keys::length(e, "/tikz/mark size").unwrap_or(2.0 * PT);
        let repeat = keys::number(e, "/tikz/mark repeat").unwrap_or(1.0).max(1.0) as usize;
        let phase = keys::number(e, "/tikz/mark phase").unwrap_or(1.0).max(1.0) as usize;
        let indices: Vec<usize> = match keys::value_tokens(e, "/tikz/mark indices") {
            Some(v) => {
                let list = e.tex.expand_fully(v)?;
                super::super::foreach_items(&list).into_iter().filter_map(|i| tex::tokens_to_string(&i).trim().parse().ok()).collect()
            }
            None => Vec::new(),
        };
        draw_mark_set(
            e,
            &name,
            size,
            &local,
            m.points
                .iter()
                .copied()
                .enumerate()
                .filter(|(i, _)| {
                    let k = i + 1;
                    if !indices.is_empty() {
                        return indices.contains(&k);
                    }
                    k >= phase && (k - phase).is_multiple_of(repeat)
                })
                .map(|(_, p)| p),
        )
    })();
    e.end_group();
    r
}

/// Marks named `name` of size `size` at `points`, in the current state.
pub fn draw_mark_set(e: &mut Engine, name: &str, size: f64, local: &Transform, points: impl Iterator<Item = Point>) -> Result<(), Error> {
    let Some((shape, usage)) = marks::mark(name, size) else {
        if name == "text" {
            return Ok(());
        }
        e.warn(format!("mark '{name}' not supported: drawn as *"));
        return draw_mark_set(e, "*", size, local, points);
    };
    let stroke = e.stroke();
    let fill = Fill { paint: Paint::Solid(e.st.fill_paint_color()), opacity: e.st.fill_opacity, even_odd: false };
    let lin = Transform { e: 0.0, f: 0.0, ..*local };
    for p in points {
        let t = Transform::translation(p.x, p.y).compose(&lin);
        let segs = crate::canvas::transform_segments(&shape, &t);
        if !e.canvas_frozen {
            e.canvas.include_path(&segs);
        }
        let (f, s) = match usage {
            Use::Stroke => (None, Some(stroke.clone())),
            Use::Fill => (Some(fill.clone()), None),
            Use::FillStroke => (Some(fill.clone()), Some(stroke.clone())),
        };
        e.canvas.push(Item::Path { segments: segs, fill: f, stroke: s });
    }
    Ok(())
}
