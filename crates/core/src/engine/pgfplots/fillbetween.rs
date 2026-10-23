//! `\addplot fill between[of=A and B, …]` (pgflibraryfillbetween.code.tex): the area between two
//! named paths, the first one followed by the second one backwards (or forwards: `reverse`, by
//! default whether both run the same way). With `split`, the area is cut where the paths cross and
//! each piece is filled on its own, in `every segment`, the plot's options, `every segment no k`,
//! `every odd segment` or `every even segment` and `every last segment` (`\tikzfillbetween`).

use crate::scene::{Point, Segment};
use crate::tex::{self, Token};
use crate::{Error, PT};

use super::plot::Plot;
use super::{toks, Engine};

/// Pieces a curve is flattened into when the paths are cut.
const CURVE_PIECES: usize = 32;

/// A path as one polyline: inner moves become lines (`inner moveto=connect`), curves are
/// flattened.
fn polyline(segs: &[Segment]) -> Vec<Point> {
    let mut out: Vec<Point> = Vec::new();
    let mut start = Point::default();
    for s in segs {
        match *s {
            Segment::MoveTo(p) => {
                if out.is_empty() {
                    start = p;
                }
                out.push(p);
            }
            Segment::LineTo(p) => out.push(p),
            Segment::CurveTo(c1, c2, p) => {
                let a = out.last().copied().unwrap_or(p);
                for k in 1..=CURVE_PIECES {
                    let t = k as f64 / CURVE_PIECES as f64;
                    let u = 1.0 - t;
                    out.push(a.scale(u * u * u).plus(c1.scale(3.0 * u * u * t)).plus(c2.scale(3.0 * u * t * t)).plus(p.scale(t * t * t)));
                }
            }
            Segment::Close => out.push(start),
        }
    }
    out.dedup_by(|a, b| a.minus(*b).norm() < 1e-9);
    out
}

/// Inner moves of a path joined by lines (`inner moveto=connect`), trailing moves dropped.
fn connected(segs: &[Segment]) -> Vec<Segment> {
    let mut out: Vec<Segment> = Vec::new();
    // a move waits: the next one replaces it, the end of the path drops it
    let mut moved: Option<Point> = None;
    for s in segs {
        match s {
            Segment::MoveTo(p) if out.is_empty() => out.push(*s),
            Segment::MoveTo(p) => moved = Some(*p),
            _ => {
                if let Some(q) = moved.take() {
                    out.push(Segment::LineTo(q));
                }
                out.push(*s);
            }
        }
    }
    out
}

/// The way a path starts: `+` when its second point is to the right (or above, at the same x)
/// of its first (`\pgfplotsfillbetween@heuristics@get@direction`).
fn direction(segs: &[Segment]) -> bool {
    let mut pts = segs.iter().filter_map(|s| match s {
        Segment::MoveTo(p) | Segment::LineTo(p) | Segment::CurveTo(_, _, p) => Some(*p),
        Segment::Close => None,
    });
    match (pts.next(), pts.next()) {
        (Some(a), Some(b)) => a.x < b.x || (a.x == b.x && a.y < b.y),
        _ => true,
    }
}

/// Where a point is on a polyline: the piece and the position along it.
fn locate(line: &[Point], p: Point) -> (usize, f64) {
    let mut best = (0, 0.0, f64::INFINITY);
    for (i, w) in line.windows(2).enumerate() {
        let d = w[1].minus(w[0]);
        let l2 = d.x * d.x + d.y * d.y;
        let t = if l2 > 0.0 { ((p.x - w[0].x) * d.x + (p.y - w[0].y) * d.y) / l2 } else { 0.0 };
        let t = t.clamp(0.0, 1.0);
        let q = w[0].plus(d.scale(t));
        let dist = q.minus(p).norm();
        if dist < best.2 - 1e-12 {
            best = (i, t, dist);
        }
    }
    (best.0, best.1)
}

/// A polyline cut at points on it, in its own order.
fn cut(line: &[Point], points: &[Point]) -> Vec<Vec<Point>> {
    let mut at: Vec<(usize, f64, Point)> = points
        .iter()
        .map(|p| {
            let (i, t) = locate(line, *p);
            (i, t, *p)
        })
        .collect();
    at.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
    let mut out = Vec::new();
    let mut current = vec![line[0]];
    let mut next = 0;
    for i in 0..line.len().saturating_sub(1) {
        while next < at.len() && at[next].0 == i {
            current.push(at[next].2);
            out.push(std::mem::replace(&mut current, vec![at[next].2]));
            next += 1;
        }
        current.push(line[i + 1]);
    }
    out.push(current);
    out
}

fn path_of(line: &[Point]) -> Vec<Segment> {
    let mut segs = Vec::new();
    if let Some((first, rest)) = line.split_first() {
        segs.push(Segment::MoveTo(*first));
        segs.extend(rest.iter().map(|p| Segment::LineTo(*p)));
    }
    segs
}

/// The area between `a` and `b` (`\pgfpathfillbetween@compute`): `a`, then `b` (backwards with
/// `reverse`) joined to it by a line, closed.
fn area(a: &[Segment], b: &[Segment], reverse: bool) -> Vec<Segment> {
    let mut segs: Vec<Segment> = a.iter().copied().filter(|s| !matches!(s, Segment::Close)).collect();
    let b = if reverse { super::visual::reverse(b) } else { b.to_vec() };
    for (i, s) in b.into_iter().filter(|s| !matches!(s, Segment::Close)).enumerate() {
        segs.push(match (i, s) {
            (0, Segment::MoveTo(p)) => Segment::LineTo(p),
            (_, s) => s,
        });
    }
    segs.push(Segment::Close);
    segs
}

/// Whether a piece is empty: its bounding box narrower or lower than 1pt.
fn empty(line: &[Point]) -> bool {
    let (mut lo, mut hi) = (Point::new(f64::INFINITY, f64::INFINITY), Point::new(f64::NEG_INFINITY, f64::NEG_INFINITY));
    for p in line {
        lo = Point::new(lo.x.min(p.x), lo.y.min(p.y));
        hi = Point::new(hi.x.max(p.x), hi.y.max(p.y));
    }
    hi.x - lo.x < PT || hi.y - lo.y < PT
}

/// Draws the plot: its segments, each filled in its styles.
pub fn draw(e: &mut Engine, p: &Plot, spec: &[Token]) -> Result<(), Error> {
    let mut of = String::new();
    let mut split = false;
    let mut reverse: Option<bool> = None;
    let mut soft_domain = false;
    let mut rest: Vec<Token> = Vec::new();
    for item in crate::keys::split_list(spec) {
        let value = item.value.clone().map(|v| tex::tokens_to_string(&v)).unwrap_or_default();
        match item.key.trim() {
            "of" => of = value,
            "split" => split = value.trim() != "false",
            "reverse" => {
                reverse = match value.trim() {
                    "false" => Some(false),
                    "auto" => None,
                    _ => Some(true),
                }
            }
            k @ ("soft clip" | "soft clip first" | "soft clip second") => {
                soft_domain |= value.trim_start().starts_with("domain");
                e.warn(format!("fill between: '{k}' not supported: the whole area is filled"))
            }
            _ => {
                rest.extend(toks(&item.key));
                if let Some(v) = item.value {
                    rest.extend(toks("={"));
                    rest.extend(v);
                    rest.extend(toks("}"));
                }
                rest.extend(toks(","));
            }
        }
    }
    let Some((a, b)) = of.split_once(" and ") else {
        e.warn("fill between: 'of=A and B' missing");
        return Ok(());
    };
    let (Some(pa), Some(pb)) = (e.named_paths.get(a.trim()).cloned(), e.named_paths.get(b.trim()).cloned()) else {
        e.warn(format!("fill between: no path named '{}' or '{}'", a.trim(), b.trim()));
        return Ok(());
    };
    // `every segment no 1/.style=…` and the other options
    if !rest.is_empty() {
        let saved = e.st.key_path.clone();
        let r = e.process_keys(rest, Some("/tikz/fill between"));
        e.st.key_path = saved;
        r?;
    }
    let (pa, pb) = (connected(&pa), connected(&pb));
    // a path of no length (a point, as `(\pgfkeysvalueof{/pgfplots/xmin},0)` makes with xmin unset)
    // has no intersections to split at or to clip: pgfplots fills nothing then
    let point = |s: &[Segment]| {
        let pts: Vec<Point> = s
            .iter()
            .filter_map(|g| match g {
                Segment::MoveTo(p) | Segment::LineTo(p) | Segment::CurveTo(_, _, p) => Some(*p),
                _ => None,
            })
            .collect();
        pts.windows(2).all(|w| w[0].minus(w[1]).norm() < 1e-9)
    };
    if (split || soft_domain) && (point(&pa) || point(&pb)) {
        return Ok(());
    }
    let reverse = reverse.unwrap_or_else(|| direction(&pa) == direction(&pb));
    let mut pieces: Vec<Vec<Segment>> = Vec::new();
    if split {
        // both run the same way, then are cut where they cross: the kth pieces make the kth segment
        let la = polyline(&pa);
        let lb = if reverse { polyline(&pb) } else { polyline(&super::visual::reverse(&pb)) };
        let crossings = super::super::intersections::intersections(&path_of(&la), &path_of(&lb));
        let (ca, cb) = (cut(&la, &crossings), cut(&lb, &crossings));
        if ca.len() != cb.len() {
            e.warn("fill between: the paths are cut into different numbers of pieces: filled as one");
            pieces.push(area(&pa, &pb, reverse));
        } else {
            for (x, y) in ca.iter().zip(cb.iter()) {
                if empty(x) && empty(y) {
                    continue;
                }
                pieces.push(area(&path_of(x), &path_of(y), true));
            }
        }
    } else {
        pieces.push(area(&pa, &pb, reverse));
    }
    let last = pieces.len().saturating_sub(1);
    for (k, segs) in pieces.into_iter().enumerate() {
        e.begin_group();
        let r = (|| -> Result<(), Error> {
            e.st.mode = Default::default();
            e.st.mode.fill = true;
            let mut style = toks("/tikz/fill between/every segment,");
            style.extend(p.style.clone());
            style.extend(toks(&format!(
                ",/tikz/fill between/every segment no {k}/.try,/tikz/fill between/every {} segment{}",
                if k % 2 == 1 { "odd" } else { "even" },
                if k == last { ",/tikz/fill between/every last segment" } else { "" }
            )));
            e.process_keys(style, Some("/pgfplots"))?;
            e.st.apply_pending();
            e.use_path(segs, "fill between")
        })();
        e.end_group();
        r?;
    }
    Ok(())
}

/// `intersection segments={of=A and B, sequence={L1 -- R2[reverse]}}`: pieces of two named paths,
/// cut where they cross, put into the path being built (`\tikzpathintersectionsegments`). `A0`,
/// `B1` count from 0, `L1`, `R1` from 1, negative numbers from the end, `*` is the whole path;
/// `--` joins a piece to the one before, `[reverse]` walks it backwards.
pub fn intersection_segments(e: &mut Engine, v: Option<Vec<Token>>) -> Result<(), Error> {
    let mut of = String::new();
    let mut sequence = "A0 -- B1".to_string();
    let mut reverse_all = false;
    for item in crate::keys::split_list(&v.unwrap_or_default()) {
        let value = item.value.map(|v| tex::tokens_to_string(&v)).unwrap_or_default();
        match item.key.trim() {
            "of" => of = value,
            "sequence" => sequence = value,
            "reverse" => reverse_all = value.trim() != "false",
            other => e.warn(format!("intersection segments: option '{other}' not supported: ignored")),
        }
    }
    let Some((a, b)) = of.split_once(" and ") else {
        e.warn("intersection segments: 'of=A and B' missing");
        return Ok(());
    };
    let (Some(pa), Some(pb)) = (e.named_paths.get(a.trim()).cloned(), e.named_paths.get(b.trim()).cloned()) else {
        e.warn(format!("intersection segments: no path named '{}' or '{}'", a.trim(), b.trim()));
        return Ok(());
    };
    let (la, lb) = (polyline(&pa), polyline(&pb));
    let crossings = super::super::intersections::intersections(&path_of(&la), &path_of(&lb));
    let pieces = [cut(&la, &crossings), cut(&lb, &crossings)];
    let whole = [la, lb];
    let mut out: Vec<Token> = Vec::new();
    let mut joined = false;
    let text: Vec<char> = sequence.chars().collect();
    let mut i = 0;
    while i < text.len() {
        let c = text[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c == '-' && text.get(i + 1) == Some(&'-') {
            joined = true;
            i += 2;
            continue;
        }
        let which = match c {
            'A' | 'L' => 0,
            'B' | 'R' => 1,
            _ => {
                e.warn(format!("intersection segments: unexpected '{}' in the sequence: the rest ignored", text[i..].iter().collect::<String>()));
                break;
            }
        };
        let one_based = matches!(c, 'L' | 'R');
        i += 1;
        let start = i;
        while i < text.len() && (text[i].is_ascii_digit() || (text[i] == '-' && i == start) || text[i] == '*') {
            i += 1;
        }
        let index: String = text[start..i].iter().collect();
        // `[reverse]`
        let mut reverse = reverse_all;
        while i < text.len() && text[i].is_whitespace() {
            i += 1;
        }
        if text.get(i) == Some(&'[') {
            let close = text[i..].iter().position(|ch| *ch == ']').map_or(text.len(), |p| i + p);
            let opts: String = text[i + 1..close].iter().collect();
            for o in opts.split(',') {
                match o.trim() {
                    "reverse" | "reverse=true" => reverse = true,
                    "reverse=false" => reverse = false,
                    "" => {}
                    other => e.warn(format!("intersection segments: option '{other}' not supported: ignored")),
                }
            }
            i = close + 1;
        }
        let line: Vec<Point> = if index == "*" || crossings.is_empty() {
            if index != "*" {
                e.warn("intersection segments: the paths do not cross: the whole path is used");
            }
            whole[which].clone()
        } else {
            let n = pieces[which].len() as i64;
            let mut k: i64 = index.parse().unwrap_or(0);
            if one_based && k >= 0 {
                k -= 1;
            }
            if k < 0 {
                k += n;
            }
            match usize::try_from(k).ok().and_then(|k| pieces[which].get(k)) {
                Some(p) => p.clone(),
                None => {
                    e.warn(format!("intersection segments: there is no piece '{c}{index}'"));
                    joined = false;
                    continue;
                }
            }
        };
        let line: Vec<Point> = if reverse { line.into_iter().rev().collect() } else { line };
        for (k, p) in line.iter().enumerate() {
            let at = format!("(tikra cs:{}pt,{}pt)", p.x / PT, p.y / PT);
            if k == 0 && !joined {
                out.extend(toks(&format!(" {at}")));
            } else {
                out.extend(toks(&format!(" -- {at}")));
            }
        }
        joined = false;
    }
    // as `insert path` does: the path goes on with them (`\tikra@insertpath`)
    let mut body = match e.tex.meaning_of("tikra@insertpath") {
        crate::tex::Meaning::Macro(m) => tex::substitute(&m.body, &[]),
        _ => Vec::new(),
    };
    body.extend(out);
    e.tex.set_meaning(&Token::cs("tikra@insertpath"), super::super::macro_tokens(body), false);
    Ok(())
}
