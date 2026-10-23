//! Drawing one plot (`\pgfplots@stored@plotlist@visualize@selected@phases`): the plot handlers
//! (lines, `smooth`, `const plot`, `jump mark`, combs, `ybar`/`xbar`, intervals), `\closedcycle`,
//! `fill between`, error bars, and the marks drawn afterwards, as pgf's mark handler draws them.

use crate::canvas::{Fill, Item, Paint};
use crate::scene::{Point, Segment, Transform};
use crate::tex::{self, Token};
use crate::{Error, PT};

use super::super::nodes::{NodeSpec, OnPath};
use super::draw::Frame;
use super::keys;
use super::marks::{self, Use};
use super::plot::Plot;
use super::{toks, Engine};

/// The marks of a plot, drawn once all plots are: its style and the canvas points.
#[derive(Debug, Clone)]
pub struct Marks {
    pub style: Vec<Token>,
    /// Canvas points and their point meta.
    pub points: Vec<(Point, f64)>,
    /// `scatter`: each mark in the colormap's color for its point meta.
    pub scatter: bool,
    /// `nodes near coords`: what the nodes say, and the nodes.
    pub node_text: Vec<Token>,
    pub nodes: Vec<NearNode>,
}

/// A node of `nodes near coords`.
#[derive(Debug, Clone)]
pub struct NearNode {
    /// The point it belongs to (kept or dropped with it), and where the node is.
    pub point: Point,
    pub at: Point,
    /// Alignment options (`above`, `anchor=center`…).
    pub align: String,
    pub meta: f64,
    /// The point meta as written, for symbolic point meta.
    pub label: Option<Vec<Token>>,
    pub index: usize,
    /// `/data point/x` and its friends.
    pub data: [f64; 3],
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
    // the areas of `fill between` are filled, not drawn (`\tikzfillbetween`)
    let mut style = toks(if p.fill_between.is_some() { "/pgfplots/every fill between plot," } else { "/tikz/draw," });
    style.extend(p.style.clone());
    e.process_keys(style.clone(), Some("/pgfplots"))?;
    e.st.apply_pending();
    // `only marks` or `scatter` without a mark: `mark=*` (`\pgfplots@markers@check@for@default@plot@mark`)
    if (e.st.only_marks || p.scatter) && e.st.plot_mark.as_deref().is_none_or(|m| m.trim().is_empty() || m.trim() == "none") {
        e.process_keys(toks("/tikz/mark=*"), Some("/tikz"))?;
        style.extend(toks(",/tikz/mark=*"));
    }
    if let Some(code) = keys::value_tokens(e, "/pgfplots/execute at begin plot visualization") {
        e.run_tokens(code)?;
    }
    if let Some(spec) = &p.fill_between {
        super::fillbetween::draw(e, p, spec)?;
        return Ok(None);
    }
    if p.handler == "contour filled" {
        super::contour::draw_filled(e, p, pts, f)?;
        return Ok(None);
    }
    // one dimensional patches of one color are a single path (`supports global path`), which
    // `\closedcycle` closes down to the zero level
    if let Some(mut segs) = super::mesh::global_path(e, p, pts, f) {
        let ends = (pts.iter().find(|l| l[0].is_finite()), pts.iter().rev().find(|l| l[0].is_finite()));
        if let (true, (Some(first), Some(last))) = (p.post.iter().any(|t| t.is_cs("closedcycle")), ends) {
            segs.push(Segment::LineTo(f.at([last[0], f.zero[1], f.min[2]])));
            segs.push(Segment::LineTo(f.at([first[0], f.zero[1], f.min[2]])));
            segs.push(Segment::Close);
        }
        e.use_path(segs, "addplot")?;
        let (node_text, nodes) = near_nodes(e, p, pts, &[], f, "", 0.0)?;
        if e.st.plot_mark.is_none() && nodes.is_empty() {
            return Ok(None);
        }
        let cp = canvas_points(f, pts);
        let points = cp.iter().enumerate().filter_map(|(i, c)| c.map(|c| (c, p.coords.get(i).map_or(f64::NAN, |k| k.meta)))).collect();
        return Ok(Some(Marks { style, points, scatter: p.scatter, node_text, nodes }));
    }
    if p.mesh.is_some() {
        super::mesh::draw(e, p, pts, f, &style)?;
        // marks and nodes near coords on the mesh's points
        let (node_text, nodes) = near_nodes(e, p, pts, &[], f, "", 0.0)?;
        if e.st.plot_mark.is_none() && nodes.is_empty() {
            return Ok(None);
        }
        let cp = canvas_points(f, pts);
        let points = cp.iter().enumerate().filter_map(|(i, c)| c.map(|c| (c, p.coords.get(i).map_or(f64::NAN, |k| k.meta)))).collect();
        return Ok(Some(Marks { style, points, scatter: p.scatter, node_text, nodes }));
    }
    let handler = keys::value(e, "/tikz/@plot handler");
    let cp = canvas_points(f, pts);
    let mut segs: Vec<Segment> = Vec::new();
    let bar_width = keys::length(e, "/pgf/bar width").unwrap_or(10.0 * PT);
    define(e, "pgfplotbarwidth", &format!("{}pt", bar_width / PT));
    // stacked bars stand on each other: no shift
    let given = p.bar_shift_given || tex::tokens_to_string(&p.style).contains("bar shift");
    let bar_shift = if p.stack.is_some() && !given { 0.0 } else { keys::length(e, "/pgf/bar shift").unwrap_or(0.0) };
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
        // box plots draw their box; the points are the outliers' marks
        "boxplot" => super::boxplot::draw(e, p, f, index_of_type)?,
        "contour prepared" => super::contour::draw(e, p, pts, f)?,
        // rows give a point and its tie's other end: the curve through all of them (the points,
        // then the other ends backwards), then every nth tie
        "tieline" => {
            let ends: Vec<Point> = cp.iter().step_by(2).flatten().copied().collect();
            let others: Vec<Point> = cp.iter().skip(1).step_by(2).flatten().copied().collect();
            let curve: Vec<Point> = ends.iter().chain(others.iter().rev()).copied().collect();
            if let Some((first, rest)) = curve.split_first() {
                let mut segs = vec![Segment::MoveTo(*first)];
                segs.extend(rest.iter().map(|q| Segment::LineTo(*q)));
                e.begin_group();
                let r = (|| {
                    e.process_keys(toks("/pgfplots/tieline/every curve"), Some("/pgfplots"))?;
                    e.st.apply_pending();
                    e.use_path(segs, "addplot")
                })();
                e.end_group();
                r?;
            }
            e.process_keys(toks("/pgfplots/tieline/every tieline"), Some("/pgfplots"))?;
            e.st.apply_pending();
            let nth = keys::number(e, "/pgfplots/tieline/each nth tie").unwrap_or(1.0).max(1.0) as usize;
            if keys::value(e, "/tikz/@plot handler") == "tieline" {
                for (i, pair) in cp.chunks(2).enumerate() {
                    if let ([Some(a), Some(b)], true) = (pair, (i + 1) % nth == 0) {
                        segs.push(Segment::MoveTo(*b));
                        segs.push(Segment::LineTo(*a));
                    }
                }
            }
        }
        // an arrow per point, each a path of its own (its tips at its end)
        "quiver" => {
            let color = keys::value(e, "/pgfplots/quiver/arrow color");
            for (i, l) in pts.iter().enumerate() {
                let Some(c) = p.coords.get(i) else { continue };
                if !(l[0].is_finite() && l[1].is_finite()) || !c.uvw.iter().take(if f.three_d { 3 } else { 2 }).all(|v| v.is_finite()) {
                    continue;
                }
                let end = [f.logical(0, c.x + c.uvw[0]), f.logical(1, c.y + c.uvw[1]), if f.three_d { f.logical(2, c.z + c.uvw[2]) } else { f.min[2] }];
                let segs = vec![Segment::MoveTo(f.at(*l)), Segment::LineTo(f.at(end))];
                e.begin_group();
                let r = (|| -> Result<(), Error> {
                    if c.meta.is_finite() {
                        let t = (c.meta - f.meta[0]) / (f.meta[1] - f.meta[0]) * 1000.0;
                        define(e, "pgfplotspointmetatransformed", &super::super::pgf_number(if t.is_finite() { t } else { 0.0 }));
                        if f.colormap.is_some() {
                            e.colors.insert("mapped color".into(), super::mesh::map(f, c.meta));
                        }
                    }
                    e.st.pending = Transform::default();
                    let mut style = Vec::new();
                    if !color.is_empty() {
                        style.extend(toks(&format!("/tikz/color={color},")));
                    }
                    style.extend(toks("/pgfplots/quiver/every arrow"));
                    e.process_keys(style, Some("/pgfplots"))?;
                    e.st.apply_pending();
                    e.use_path(segs, "addplot")
                })();
                e.colors.insert("mapped color".into(), crate::color::Color::BLACK);
                e.end_group();
                r?;
            }
        }
        // rays from the center
        "polar comb" => {
            for l in pts {
                if l[0].is_finite() && l[1].is_finite() {
                    segs.push(Segment::MoveTo(f.at([l[0], 0.0, 0.0])));
                    segs.push(Segment::LineTo(f.at(*l)));
                }
            }
        }
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
                if e.st.plot_handler == "smooth" {
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
    // the rest of the path: nodes at the last point, those with `pos` along the plot
    let rest: Vec<Token> = p.post.iter().filter(|t| !t.is_cs("closedcycle")).cloned().collect();
    if rest.iter().any(|t| !t.is_space()) {
        post_nodes(e, p, pts, &cp, f, rest)?;
    }
    error_bars(e, p, pts, f)?;
    let (node_text, nodes) = near_nodes(e, p, pts, base, f, &handler, bar_shift)?;
    if mark.is_none() && nodes.is_empty() {
        return Ok(None);
    }
    let points = cp.iter().enumerate().filter_map(|(i, c)| c.map(|c| (c, p.coords.get(i).map_or(f64::NAN, |k| k.meta)))).collect();
    Ok(Some(Marks { style, points, scatter: p.scatter, node_text, nodes }))
}

/// The nodes after a plot (`\addplot … node[pos=0.3] {…};`): at the last point, or with `pos` at
/// that fraction of the plot (`\pgfplotspointplotattime`), on the segment there.
fn post_nodes(e: &mut Engine, p: &Plot, pts: &[[f64; 3]], cp: &[Option<Point>], f: &Frame, rest: Vec<Token>) -> Result<(), Error> {
    let (prefix, items) = split_nodes(&rest);
    let mut at_end: Vec<Token> = prefix.clone();
    let size = |e: &mut Engine, a: usize| keys::length(e, if a == 0 { "/pgfplots/width" } else { "/pgfplots/height" }).unwrap_or(if a == 0 { 240.0 * PT } else { 207.0 * PT }) / PT;
    let scale = [0, 1, 2].map(|a| {
        let [lo, hi] = f.data_range[a];
        if lo.is_finite() && hi.is_finite() {
            super::ticks::data_scale(lo, hi, size(e, a))
        } else {
            1.0
        }
    });
    let points: Vec<Option<[f64; 3]>> = pts.iter().map(|l| l.iter().take(if f.three_d { 3 } else { 2 }).all(|v| v.is_finite()).then_some(*l)).collect();
    let tl = super::attime::Timeline { points, scale, log: f.log, time: 1.0 };
    let mut placed = Vec::new();
    // the path goes on at the last point when no node has a `pos` (options, `--` and others too)
    let mut ends = items.is_empty();
    for item in items {
        match node_time(e, &item).and_then(|t| tl.at(t).map(|r| (t, r))) {
            Some((t, (_, a, b, frac))) => placed.push((item, t, a, b, frac)),
            None => {
                at_end.extend(item);
                ends = true;
            }
        }
    }
    super::attime::set(Some(tl));
    let r = (|| -> Result<(), Error> {
        let z = |l: [f64; 3]| if f.three_d { l } else { [l[0], l[1], f.min[2]] };
        for (item, t, a, b, frac) in placed {
            super::attime::set_time(t);
            let (pa, pb) = (f.at(z(a)), f.at(z(b)));
            let mut code = toks("\\path[draw=none,fill=none]");
            code.extend(prefix.clone());
            code.extend(toks(&format!(" (tikra cs:{}pt,{}pt) -- (tikra cs:{}pt,{}pt) ", pa.x / PT, pa.y / PT, pb.x / PT, pb.y / PT)));
            code.extend(with_pos(&item, frac));
            code.push(Token::other(';'));
            e.run_tokens(code)?;
        }
        if ends {
            super::attime::set_time(1.0);
            // a box plot without outliers starts its path at the origin
            let origin = (p.boxplot.is_some()).then(|| f.at(f.min));
            if let Some(last) = cp.iter().rev().flatten().next().copied().or(origin).as_ref() {
                let mut code = toks(&format!("\\path[draw=none,fill=none] (tikra cs:{}pt,{}pt) ", last.x / PT, last.y / PT));
                code.extend(at_end);
                code.push(Token::other(';'));
                e.run_tokens(code)?;
            }
        }
        Ok(())
    })();
    super::attime::set(None);
    r
}

/// The path's options before its first node, and each `node …` or `coordinate …` (with what
/// follows it up to the next one).
fn split_nodes(rest: &[Token]) -> (Vec<Token>, Vec<Vec<Token>>) {
    let word = |i: usize| -> Option<usize> {
        for w in ["coordinate", "node"] {
            let n = w.len();
            if i + n <= rest.len()
                && rest[i..i + n].iter().zip(w.chars()).all(|(t, c)| t.is_char(c))
                && (i == 0 || !matches!(rest[i - 1], Token::Char(ch, tex::Cat::Letter) if ch.is_alphabetic()))
                && !matches!(rest.get(i + n), Some(Token::Char(ch, tex::Cat::Letter)) if ch.is_alphabetic())
            {
                return Some(n);
            }
        }
        None
    };
    let mut prefix = Vec::new();
    let mut items: Vec<Vec<Token>> = Vec::new();
    let (mut braces, mut brackets) = (0i32, 0i32);
    let mut i = 0;
    while i < rest.len() {
        let t = &rest[i];
        if braces == 0 && brackets == 0 {
            if let Some(n) = word(i) {
                items.push(rest[i..i + n].to_vec());
                i += n;
                continue;
            }
        }
        match t {
            Token::Char(_, tex::Cat::Begin) => braces += 1,
            Token::Char(_, tex::Cat::End) => braces -= 1,
            Token::Char('[', _) if braces == 0 => brackets += 1,
            Token::Char(']', _) if braces == 0 => brackets -= 1,
            _ => {}
        }
        match items.last_mut() {
            Some(item) => item.push(t.clone()),
            None => prefix.push(t.clone()),
        }
        i += 1;
    }
    (prefix, items)
}

/// The options of a node item (`node [options] …`): where they are.
fn node_options(item: &[Token]) -> Option<(usize, usize)> {
    let mut j = item.iter().position(|t| !matches!(t, Token::Char(_, tex::Cat::Letter)))?;
    while j < item.len() && item[j].is_space() {
        j += 1;
    }
    if !item.get(j)?.is_char('[') {
        return None;
    }
    let mut depth = 0;
    for (k, t) in item.iter().enumerate().skip(j) {
        match t {
            Token::Char(_, tex::Cat::Begin) => depth += 1,
            Token::Char(_, tex::Cat::End) => depth -= 1,
            Token::Char(']', _) if depth == 0 => return Some((j, k)),
            _ => {}
        }
    }
    None
}

/// The `pos` a node item asks for (`pos=…`, `midway`, `near end` and their friends).
fn node_time(e: &mut Engine, item: &[Token]) -> Option<f64> {
    let (a, b) = node_options(item)?;
    let mut time = None;
    for it in crate::keys::split_list(&item[a + 1..b]) {
        let t = match it.key.trim() {
            "pos" => it.value.and_then(|v| e.number_of(v).ok()),
            "at start" => Some(0.0),
            "very near start" => Some(0.125),
            "near start" => Some(0.25),
            "midway" => Some(0.5),
            "near end" => Some(0.75),
            "very near end" => Some(0.875),
            "at end" => Some(1.0),
            _ => None,
        };
        if t.is_some() {
            time = t;
        }
    }
    time
}

/// A node item with its `pos` set to `frac` (of the segment it is placed on).
fn with_pos(item: &[Token], frac: f64) -> Vec<Token> {
    let Some((_, b)) = node_options(item) else { return item.to_vec() };
    let mut out = item[..b].to_vec();
    out.extend(toks(&format!(",pos={}", super::super::pgf_number(frac))));
    out.extend(item[b..].iter().cloned());
    out
}

/// The anchors `nodes near coords align` may name.
const ANCHORS: &[&str] =
    &["north", "south", "east", "west", "north east", "north west", "south east", "south west", "center", "base", "base east", "base west", "mid", "mid east", "mid west"];

/// `nodes near coords`: one node per point (`scatter/@post marker code`), aligned by
/// `nodes near coords/@@eval alignment`; bars move them by their bar shift, stacked bars (since
/// compat 1.9) put them in the middle of their bar.
#[allow(clippy::too_many_arguments)]
fn near_nodes(e: &mut Engine, p: &Plot, pts: &[[f64; 3]], base: &[f64], f: &Frame, handler: &str, bar_shift: f64) -> Result<(Vec<Token>, Vec<NearNode>), Error> {
    let Some(text) = keys::value_tokens(e, "/pgfplots/@nodes near coords") else { return Ok((Vec::new(), Vec::new())) };
    if crate::keys::trim(&text).is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    let source = keys::value(e, "/pgfplots/point meta");
    let symbolic = source == "explicit symbolic";
    let align = keys::value(e, "/pgfplots/nodes near coords align");
    let align = if align == "auto" {
        if source == "x" {
            "horizontal".to_string()
        } else {
            "vertical".to_string()
        }
    } else {
        align
    };
    let compat = keys::compat(e);
    let stacked = p.stack.is_some() && compat >= 109 && matches!(handler, "ybar" | "xbar");
    let offset = keys::number(e, "/pgfplots/nodes near coords bar offset").unwrap_or(0.5);
    let shift = match handler {
        "ybar" => unit(f.e[0]).scale(bar_shift),
        "xbar" => unit(f.e[1]).scale(bar_shift),
        _ => Point::default(),
    };
    let mut nodes = Vec::new();
    for (i, l) in pts.iter().enumerate() {
        if !l.iter().take(if f.three_d { 3 } else { 2 }).all(|v| v.is_finite()) {
            continue;
        }
        let point = f.at(*l);
        let c = p.coords.get(i).copied().unwrap_or_else(|| super::plot::Coord::new(f64::NAN, f64::NAN, f64::NAN));
        let label = (c.label != super::plot::NO_LABEL).then(|| p.labels.get(c.label as usize).cloned()).flatten();
        // before compat 1.9 the nodes of stacked plots show the stacked value (f(x) after stacking)
        let meta = match p.stack {
            // f(x) is y (2d): x stacked bars keep it
            Some(_) if !stacked && matches!(source.as_str(), "f(x)" | "y" | "x") => {
                if source == "x" {
                    l[0]
                } else {
                    l[1]
                }
            }
            _ => c.meta,
        };
        let at = if stacked {
            let b = base.get(i).copied().unwrap_or(0.0);
            if handler == "ybar" {
                f.at([l[0], offset * l[1] + (1.0 - offset) * b, l[2]])
            } else {
                f.at([offset * l[0] + (1.0 - offset) * b, l[1], l[2]])
            }
        } else {
            point
        };
        let a = if stacked {
            "anchor=center".to_string()
        } else {
            match align.as_str() {
                "horizontal" if symbolic => "above".into(),
                "horizontal" => (if meta < 0.0 { "left" } else { "right" }).into(),
                "vertical" if symbolic => "above".into(),
                "vertical" => (if meta < 0.0 { "below" } else { "above" }).into(),
                other if ANCHORS.contains(&other) => format!("anchor={other}"),
                other => other.to_string(),
            }
        };
        nodes.push(NearNode { point, at: at.plus(shift), align: a, meta, label, index: i, data: [c.x, c.y, c.z] });
    }
    Ok((text, nodes))
}

/// Draws the nodes of `nodes near coords`, in the plot's style.
pub fn draw_near_nodes(e: &mut Engine, m: &Marks) -> Result<(), Error> {
    if m.nodes.is_empty() {
        return Ok(());
    }
    e.begin_group();
    let r = (|| -> Result<(), Error> {
        e.st.mode = Default::default();
        e.st.pending = Transform::default();
        e.process_keys(m.style.clone(), Some("/pgfplots"))?;
        e.st.apply_pending();
        let default = tex::tokens_to_string(&m.node_text).replace(' ', "") == "\\pgfmathprintnumber\\pgfplotspointmeta";
        for n in &m.nodes {
            e.begin_group();
            let r = (|| -> Result<(), Error> {
                let meta = match &n.label {
                    Some(l) if !n.meta.is_finite() => tex::tokens_to_string(l),
                    _ => super::super::pgf_number(n.meta),
                };
                define(e, "pgfplotspointmeta", &meta);
                define(e, "coordindex", &n.index.to_string());
                for (k, c) in ['x', 'y', 'z'].iter().enumerate() {
                    super::keys::set(e, &format!("/data point/{c}"), &super::super::pgf_number(n.data[k]));
                }
                // symbolic point meta is shown as written
                let text = match (&n.label, default && !n.meta.is_finite()) {
                    (Some(l), true) => l.clone(),
                    _ => {
                        // `\pgfmathprintnumber\pgfplotspointmeta` takes the macro, then the rest
                        let t = super::draw::print_numbers(e, &m.node_text)?;
                        e.tex.expand_fully(t)?
                    }
                };
                let options = toks(&format!("{},/pgfplots/every node near coord", n.align));
                let items = e.node(NodeSpec { options, name: None, at: Some(n.at), text, place: OnPath::Point(n.at), point: n.at, coordinate: false, default_pos: None })?;
                e.canvas.items.extend(items);
                Ok(())
            })();
            e.end_group();
            r?;
        }
        Ok(())
    })();
    e.end_group();
    r
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

/// A path walked backwards.
pub(super) fn reverse(segs: &[Segment]) -> Vec<Segment> {
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
pub fn draw_marks(e: &mut Engine, m: &Marks, f: &Frame) -> Result<(), Error> {
    e.begin_group();
    let r = (|| -> Result<(), Error> {
        let setup = |e: &mut Engine| -> Result<(), Error> {
            e.st.mode = Default::default();
            e.st.pending = Transform::default();
            e.process_keys(m.style.clone(), Some("/pgfplots"))?;
            e.st.apply_pending();
            e.st.pending = Transform::default();
            e.process_keys(toks("every mark"), Some("/tikz"))?;
            if m.scatter {
                if let Some(v) = keys::value_tokens(e, "/pgfplots/scatter/use mapped color") {
                    e.process_keys(v, Some("/tikz"))?;
                }
            }
            Ok(())
        };
        let colors = if m.scatter { Some(super::mesh::mapped_colors(e, setup)?) } else { None };
        setup(e)?;
        let Some(name) = e.st.plot_mark.clone() else { return Ok(()) };
        let local = e.st.pending;
        let size = keys::length(e, "/tikz/mark size").unwrap_or(2.0 * PT);
        let cube = f.three_d && (name == "cube" || name == "cube*");
        // the native keys keep these in the state
        let repeat = e.st.mark_repeat.max(1);
        let phase = e.st.mark_phase.max(1);
        let indices: Vec<usize> = match keys::value_tokens(e, "/tikz/mark indices") {
            Some(v) => {
                let list = e.tex.expand_fully(v)?;
                super::super::foreach_items(&list).into_iter().filter_map(|i| tex::tokens_to_string(&i).trim().parse().ok()).collect()
            }
            None => Vec::new(),
        };
        let chosen = m.points.iter().copied().enumerate().filter(|(i, _)| {
            let k = i + 1;
            if !indices.is_empty() {
                return indices.contains(&k);
            }
            k >= phase && (k - phase).is_multiple_of(repeat)
        });
        match colors {
            Some((draw, fill, _)) => {
                for (_, (p, meta)) in chosen {
                    let mapped = super::mesh::map(f, meta);
                    e.st.draw_color = Some(draw.at(mapped));
                    e.st.fill_color = Some(fill.at(mapped));
                    if name == "ball" {
                        // `ball color=mapped color`: the shading's color, point by point
                        e.colors.insert("mapped color".into(), mapped);
                        if let Some(v) = keys::value_tokens(e, "/pgfplots/scatter/use mapped color") {
                            e.process_keys(v, Some("/tikz"))?;
                        }
                        e.colors.insert("mapped color".into(), crate::color::Color::BLACK);
                    }
                    if cube {
                        draw_cubes(e, &name, e.st.mark_size, f, std::iter::once(p))?;
                    } else {
                        draw_mark_set(e, &name, size, &local, std::iter::once(p))?;
                    }
                }
                Ok(())
            }
            None if cube => draw_cubes(e, &name, e.st.mark_size, f, chosen.map(|(_, (p, _))| p)),
            None => draw_mark_set(e, &name, size, &local, chosen.map(|(_, (p, _))| p)),
        }
    })();
    e.end_group();
    r
}

/// `mark=cube` and `cube*` in a 3d axis: a cube with edges `size` long along the axes
/// (`cube/size x=\pgfplotmarksize`); `cube*` fills the three faces in front.
fn draw_cubes(e: &mut Engine, name: &str, size: f64, f: &Frame, points: impl Iterator<Item = Point>) -> Result<(), Error> {
    let half = [0, 1, 2].map(|k| {
        let n = f.e[k].norm();
        if n > 0.0 {
            f.e[k].scale(0.5 * size / n)
        } else {
            Point::default()
        }
    });
    let stroke = e.stroke();
    let fill = Fill { paint: Paint::Solid(e.st.fill_paint_color()), opacity: e.st.fill_opacity, even_odd: false };
    for p in points {
        let at = |s: [f64; 3]| p.plus(half[0].scale(s[0])).plus(half[1].scale(s[1])).plus(half[2].scale(s[2]));
        let face = |k: usize, side: f64| -> Vec<Segment> {
            // the four corners of the face at `side` across axis k, around
            let (a, b) = ((k + 1) % 3, (k + 2) % 3);
            let mut c = Vec::new();
            for (u, v) in [(-1.0, -1.0), (-1.0, 1.0), (1.0, 1.0), (1.0, -1.0)] {
                let mut s = [0.0; 3];
                s[k] = side;
                s[a] = u;
                s[b] = v;
                c.push(at(s));
            }
            vec![Segment::MoveTo(c[0]), Segment::LineTo(c[1]), Segment::LineTo(c[2]), Segment::LineTo(c[3]), Segment::Close]
        };
        if name == "cube*" {
            for k in 0..3 {
                let segs = face(k, if f.fg[k][0] { -1.0 } else { 1.0 });
                if !e.canvas_frozen {
                    e.canvas.include_path(&segs);
                }
                e.canvas.push(Item::Path { segments: segs, fill: Some(fill.clone()), stroke: Some(stroke.clone()) });
            }
        } else {
            let mut segs = face(0, -1.0);
            segs.extend(face(0, 1.0));
            for (u, v) in [(-1.0, -1.0), (-1.0, 1.0), (1.0, 1.0), (1.0, -1.0)] {
                segs.push(Segment::MoveTo(at([1.0, u, v])));
                segs.push(Segment::LineTo(at([-1.0, u, v])));
            }
            if !e.canvas_frozen {
                e.canvas.include_path(&segs);
            }
            e.canvas.push(Item::Path { segments: segs, fill: None, stroke: Some(stroke.clone()) });
        }
    }
    Ok(())
}

/// Marks named `name` of size `size` at `points`, in the current state.
pub fn draw_mark_set(e: &mut Engine, name: &str, size: f64, local: &Transform, points: impl Iterator<Item = Point>) -> Result<(), Error> {
    // a shaded ball: the engine's mark, with its `ball color`
    if name == "ball" {
        let pts: Vec<Point> = points.collect();
        let _ = size;
        let items = e.plot_marks(&pts)?;
        e.canvas.items.extend(items);
        let _ = local;
        return Ok(());
    }
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
