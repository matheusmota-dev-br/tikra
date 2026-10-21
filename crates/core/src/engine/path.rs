//! Path statements: `\path[options] <operations> ;`, read the way `tikz.code.tex` reads them, token
//! by token after expansion: coordinates, `--`, `-|`, `|-`, `.. controls ..`, `to`, `edge`,
//! `rectangle`, `circle`, `ellipse`, `arc`, `grid`, `plot`, `sin`, `cos`, `parabola`, `cycle`,
//! `let`, `node`, `coordinate`, `\foreach` and `\pgfextra` inside the path, options and `{…}` groups.

use crate::canvas::Item;
use crate::keys;
use crate::scene::{Point, Segment};
use crate::tex::{self, Cat, Meaning, Token};
use crate::Error;

use super::coords::{read_coordinate_tokens, Coord};
use super::nodes::{NodeSpec, OnPath};
use super::state::Mode;
use super::{macro_from, tikra_tokens, Engine};

/// Largest accepted coordinate, in cm (one kilometer): beyond that it is a typo or a bad computation.
pub const MAX_COORDINATE: f64 = 100_000.0;
/// Cap on `grid` lines in each direction.
pub const MAX_GRID_LINES: usize = 10_000;
/// Cap on decoration steps per segment.
pub const MAX_DECORATION_STEPS: usize = 10_000;

pub fn valid_point(p: Point) -> bool {
    p.x.is_finite() && p.y.is_finite() && p.x.abs() <= MAX_COORDINATE && p.y.abs() <= MAX_COORDINATE
}

pub fn valid_segment(s: &Segment) -> bool {
    match s {
        Segment::MoveTo(p) | Segment::LineTo(p) => valid_point(*p),
        Segment::CurveTo(a, b, p) => valid_point(*a) && valid_point(*b) && valid_point(*p),
        Segment::Close => true,
    }
}

/// A node written between an operation and its target (`-- node {a} (B)`), placed once the segment exists.
struct MidNode {
    options: Vec<Token>,
    name: Option<String>,
    text: Vec<Token>,
    coordinate: bool,
}

struct Builder {
    segs: Vec<Segment>,
    current: Option<Coord>,
    subpath_start: Option<Point>,
    last: Option<OnPath>,
    node_items: Vec<Item>,
    mid_nodes: Vec<MidNode>,
    /// `edge` paths, drawn after the main path: (segments, options)
    edges: Vec<(Vec<Segment>, Vec<Token>, Vec<Item>)>,
    /// the point a `+` coordinate is relative to (set by `+`, not moved)
    relative_base: Option<Point>,
    /// (0,0) of the coordinate system the path started in: where a node goes before any coordinate
    origin: Point,
}

impl Builder {
    fn point(&self) -> Point {
        self.current.as_ref().map_or(self.origin, |c| c.p)
    }

    fn move_to(&mut self, r: Coord) {
        self.segs.push(Segment::MoveTo(r.p));
        self.subpath_start = Some(r.p);
        self.last = Some(OnPath::Point(r.p));
        self.current = Some(r);
    }
}

/// A path statement (`\draw`, `\fill`, `\path`, `\node`…): `defaults` are the command's options.
pub fn statement(e: &mut Engine, defaults: &str) -> Result<(), Error> {
    e.begin_group();
    e.st.mode = Mode::default();
    e.st.pending = crate::scene::Transform::default();
    let r = statement_in_group(e, defaults);
    e.end_group();
    r
}

fn statement_in_group(e: &mut Engine, defaults: &str) -> Result<(), Error> {
    e.process_keys(tikra_tokens("every path/.try"), Some("/tikz"))?;
    if !defaults.is_empty() {
        e.process_keys(tikra_tokens(defaults), Some("/tikz"))?;
    }
    // options right after the command, possibly several groups
    loop {
        match e.tex.get_x_token()? {
            Some(t) if t.is_space() => continue,
            Some(t) if t.is_char('[') => {
                let o = e.tex.read_until_char(']', false)?;
                e.process_keys(o, Some("/tikz"))?;
            }
            Some(t) => {
                e.tex.back(t);
                break;
            }
            None => break,
        }
    }
    e.st.apply_pending();
    let mut b = Builder {
        segs: Vec::new(),
        current: None,
        subpath_start: None,
        last: None,
        node_items: Vec::new(),
        mid_nodes: Vec::new(),
        edges: Vec::new(),
        relative_base: None,
        origin: e.st.transform.apply(Point::default()),
    };
    operations(e, &mut b, None)?;
    // a node left between an operation and a target that never came
    place_mid_nodes(e, &mut b, None)?;
    let segs = std::mem::take(&mut b.segs);
    if let Some(name) = e.st.name_path.take() {
        e.named_paths.insert(name, segs.clone());
    }
    e.use_path(segs, "path")?;
    for (segs, options, items) in std::mem::take(&mut b.edges) {
        edge_path(e, segs, options)?;
        e.canvas.items.extend(items);
    }
    let items = std::mem::take(&mut b.node_items);
    e.canvas.items.extend(items);
    Ok(())
}

fn edge_path(e: &mut Engine, segs: Vec<Segment>, options: Vec<Token>) -> Result<(), Error> {
    e.begin_group();
    e.st.mode = Mode::default();
    let r = (|| {
        e.process_keys(tikra_tokens("every edge/.try"), Some("/tikz"))?;
        e.process_keys(options, Some("/tikz"))?;
        e.use_path(segs, "edge")
    })();
    e.end_group();
    r
}

/// The next token that is not a space, expanded.
fn next(e: &mut Engine, depth: Option<usize>) -> Result<Option<Token>, Error> {
    loop {
        let t = match depth {
            Some(d) => e.tex.get_x_token_until(d)?,
            None => e.tex.get_x_token()?,
        };
        match t {
            Some(t) if t.is_space() || t.is_cs("par") || t.is_cs("relax") => continue,
            other => return Ok(other),
        }
    }
}

fn peek(e: &mut Engine, depth: Option<usize>) -> Result<Option<Token>, Error> {
    let t = next(e, depth)?;
    if let Some(t) = &t {
        e.tex.back(t.clone());
    }
    Ok(t)
}

/// Reads a word of letters (`rectangle`, `node`), the first one already read.
fn word(e: &mut Engine, first: char) -> String {
    let mut w = String::new();
    w.push(first);
    while let Some(t) = e.tex.get_token() {
        match t {
            Token::Char(c, Cat::Letter) => w.push(c),
            other => {
                e.tex.back(other);
                break;
            }
        }
        if w.len() > 64 {
            break;
        }
    }
    w
}

/// `(…)` read and evaluated (the `(` already read).
fn coordinate(e: &mut Engine) -> Result<Coord, Error> {
    let tokens = read_coordinate_tokens(e)?;
    let text = e.math_string(tokens)?;
    e.coordinate(&text)
}

/// A point that must follow: skips spaces and reads `(…)`, with an optional `+`/`++`.
fn target(e: &mut Engine, b: &mut Builder, depth: Option<usize>, what: &str) -> Result<Coord, Error> {
    match next(e, depth)? {
        Some(t) if t.is_char('(') => coordinate(e),
        Some(t) if t.is_char('+') => relative(e, b, depth),
        _ => Err(Error::Figure(format!("{what} without a point"))),
    }
}

/// `+(x,y)` or `++(x,y)` (the first `+` already read).
fn relative(e: &mut Engine, b: &mut Builder, depth: Option<usize>) -> Result<Coord, Error> {
    let double = match next(e, depth)? {
        Some(t) if t.is_char('+') => true,
        Some(t) => {
            e.tex.back(t);
            false
        }
        None => false,
    };
    match next(e, depth)? {
        Some(t) if t.is_char('(') => {}
        _ => return Err(Error::Figure("'+' without a point".into())),
    }
    let r = coordinate(e)?;
    // the offset uses only the linear part of the transformation
    let origin = e.st.transform.apply(Point::default());
    let base = b.relative_base.unwrap_or_else(|| b.point());
    let p = base.plus(r.p.minus(origin));
    if double {
        b.relative_base = None;
    } else {
        b.relative_base = Some(base);
    }
    Ok(Coord { p, node: None })
}

/// Runs path operations until `;` (or the end of the frame above `depth`).
fn operations(e: &mut Engine, b: &mut Builder, depth: Option<usize>) -> Result<(), Error> {
    // the operation waiting for its target: "--", "-|", "|-", or "to" with its options
    let mut pending_op: Option<String> = None;
    let mut to_options: Vec<Token> = Vec::new();
    loop {
        let Some(t) = next(e, depth)? else {
            if depth.is_none() {
                e.warn("path without ';' at the end");
            }
            return Ok(());
        };
        e.steps += 1;
        if e.steps > super::MAX_STEPS {
            return Err(Error::Figure(format!("figure too large: more than {} statements", super::MAX_STEPS)));
        }
        match &t {
            Token::Char(';', _) => return Ok(()),
            Token::Char('(', _) => {
                let r = coordinate(e)?;
                b.relative_base = None;
                apply_operation(e, b, pending_op.take(), std::mem::take(&mut to_options), r)?;
            }
            Token::Char('+', _) => {
                let r = relative(e, b, depth)?;
                apply_operation(e, b, pending_op.take(), std::mem::take(&mut to_options), r)?;
            }
            Token::Char('-', _) => match e.tex.get_token() {
                Some(n) if n.is_char('-') => pending_op = Some("--".into()),
                Some(n) if n.is_char('|') => pending_op = Some("-|".into()),
                other => {
                    if let Some(o) = other {
                        e.tex.back(o);
                    }
                    e.warn("unrecognized path fragment: '-'");
                    return skip_rest(e, depth);
                }
            },
            Token::Char('|', _) => match e.tex.get_token() {
                Some(n) if n.is_char('-') => pending_op = Some("|-".into()),
                other => {
                    if let Some(o) = other {
                        e.tex.back(o);
                    }
                    e.warn("unrecognized path fragment: '|'");
                    return skip_rest(e, depth);
                }
            },
            Token::Char('.', _) => {
                // .. controls (a) and (b) ..
                match e.tex.get_token() {
                    Some(n) if n.is_char('.') => {}
                    _ => {
                        e.warn("unrecognized path fragment: '.'");
                        return skip_rest(e, depth);
                    }
                }
                let w = match next(e, depth)? {
                    Some(Token::Char(c, Cat::Letter)) => word(e, c),
                    _ => String::new(),
                };
                if w != "controls" {
                    e.warn("'..' without 'controls'");
                    return skip_rest(e, depth);
                }
                let c1 = target(e, b, depth, "controls")?.p;
                let mut c2 = c1;
                if let Some(Token::Char('a', Cat::Letter)) = peek(e, depth)? {
                    let _ = next(e, depth)?;
                    let w = word(e, 'a');
                    if w == "and" {
                        c2 = target(e, b, depth, "controls")?.p;
                    }
                }
                // the closing ..
                for _ in 0..2 {
                    match next(e, depth)? {
                        Some(t) if t.is_char('.') => {}
                        Some(t) => {
                            e.tex.back(t);
                            break;
                        }
                        None => break,
                    }
                }
                let r = target(e, b, depth, "controls")?;
                curve_to(e, b, c1, c2, r);
                place_mid_nodes(e, b, Some(0.5))?;
            }
            Token::Char('[', _) => {
                let o = e.tex.read_until_char(']', false)?;
                e.process_keys(o, Some("/tikz"))?;
                e.st.apply_pending();
            }
            Token::Char(_, Cat::Begin) => e.begin_group(),
            Token::Char(_, Cat::End) => e.end_group(),
            Token::Char(c, Cat::Letter) => {
                let w = word(e, *c);
                match w.as_str() {
                    "to" | "edge" => {
                        let opts = match peek(e, depth)? {
                            Some(t) if t.is_char('[') => {
                                let _ = next(e, depth)?;
                                e.tex.read_until_char(']', false)?
                            }
                            _ => Vec::new(),
                        };
                        if w == "edge" && opts.is_empty() && peek_words(e, depth, &["from", "parent"])? {
                            // a tree's edge from parent: options and label nodes, drawn after the child
                            let o = match peek(e, depth)? {
                                Some(t) if t.is_char('[') => {
                                    let _ = next(e, depth)?;
                                    e.tex.read_until_char(']', false)?
                                }
                                _ => Vec::new(),
                            };
                            let mut labels = Vec::new();
                            while let Some(Token::Char('n', Cat::Letter)) = peek(e, depth)? {
                                let _ = next(e, depth)?;
                                let w = word(e, 'n');
                                if w != "node" {
                                    e.tex.push_tokens(tex::string_tokens(&w));
                                    break;
                                }
                                let (options, _, _, text) = read_node(e, depth, false)?;
                                labels.extend(tikra_tokens("node["));
                                labels.extend(options);
                                labels.extend(tikra_tokens("]"));
                                labels.push(Token::Char('{', Cat::Begin));
                                labels.extend(text);
                                labels.push(Token::Char('}', Cat::End));
                            }
                            e.st.tree_edge = Some((o, labels));
                            e.tree_edge_out = e.st.tree_edge.clone();
                        } else if w == "edge" {
                            // nodes can follow, then the target
                            let mids = read_mid_nodes(e, depth)?;
                            let r = target(e, b, depth, "edge")?;
                            edge(e, b, opts, r, mids)?;
                        } else {
                            to_options = opts;
                            pending_op = Some("to".into());
                        }
                    }
                    "node" | "coordinate" => {
                        let coordinate = w == "coordinate";
                        let (options, name, at, text) = read_node(e, depth, coordinate)?;
                        if at.is_some() || pending_op.is_none() {
                            let place = b.last.unwrap_or(OnPath::Point(b.point()));
                            let items = e.node(NodeSpec { options, name, at, text, place, point: b.point(), coordinate, default_pos: None })?;
                            b.node_items.extend(items);
                        } else {
                            b.mid_nodes.push(MidNode { options, name, text, coordinate });
                        }
                    }
                    "cycle" => {
                        if let Some(p0) = b.subpath_start {
                            if pending_op.take().is_some() {
                                let from = b.point();
                                b.segs.push(Segment::Close);
                                b.last = Some(OnPath::Segment(from, None, p0));
                                b.current = Some(Coord { p: p0, node: None });
                                place_mid_nodes(e, b, Some(0.5))?;
                            }
                        }
                    }
                    "rectangle" => {
                        let r = target(e, b, depth, "rectangle")?;
                        let a = b.point();
                        let t = e.st.transform;
                        let inv = crate::reader::invert(&t);
                        let (la, lb) = (inv.apply(a), inv.apply(r.p));
                        b.segs.push(Segment::MoveTo(a));
                        for k in [Point::new(lb.x, la.y), lb, Point::new(la.x, lb.y)] {
                            b.segs.push(Segment::LineTo(t.apply(k)));
                        }
                        b.segs.push(Segment::Close);
                        b.subpath_start = Some(a);
                        b.last = Some(OnPath::Segment(a, None, r.p));
                        b.current = Some(Coord { p: r.p, node: None });
                        pending_op = None;
                        place_mid_nodes(e, b, Some(0.5))?;
                    }
                    "circle" | "ellipse" => circle(e, b, depth)?,
                    "arc" => arc(e, b, depth)?,
                    "grid" => grid(e, b, depth)?,
                    "plot" => plot(e, b, depth, pending_op.take().is_some())?,
                    "sin" | "cos" => {
                        let r = target(e, b, depth, &w)?;
                        let a = b.point();
                        let d = r.p.minus(a);
                        // \pgfpathsine and \pgfpathcosine
                        let (c1, c2) = if w == "sin" {
                            (Point::new(0.3260 * d.x, 0.5120 * d.y), Point::new(0.6380 * d.x, d.y))
                        } else {
                            (Point::new(0.3620 * d.x, 0.0), Point::new(0.6740 * d.x, 0.4880 * d.y))
                        };
                        curve_to(e, b, a.plus(c1), a.plus(c2), r);
                    }
                    "parabola" => parabola(e, b, depth)?,
                    "let" => let_operation(e, depth)?,
                    "in" => {}
                    "pic" => {
                        let (options, name, at, content) = read_node(e, depth, false)?;
                        let at = at.unwrap_or_else(|| b.point());
                        e.pic(super::pics::PicSpec { options, name, at, content })?;
                    }
                    "child" => {
                        let parent = e.last_node_name.clone();
                        e.children(parent)?;
                    }
                    "svg" | "decorate" => {
                        e.warn(format!("path operation '{w}' not supported: rest of the path ignored"));
                        return skip_rest(e, depth);
                    }
                    other => {
                        e.warn(format!("unrecognized path fragment: '{other}'"));
                        return skip_rest(e, depth);
                    }
                }
            }
            Token::Cs(name) if &**name == "foreach" => path_foreach(e, b)?,
            Token::Cs(name) if &**name == "pgfextra" => {
                let code = e.argument()?;
                e.run_tokens(code)?;
            }
            Token::Cs(name) if matches!(e.tex.meaning(&t), Meaning::Native(_)) && matches!(&**name, "node" | "coordinate") => {
                // `\node` inside a path reads as `node`
                e.tex.push_tokens(tikra_tokens(&format!("{name} ")));
            }
            other => {
                let s = tex::show_token(other);
                e.warn(format!("unrecognized path fragment: '{s}'"));
                return skip_rest(e, depth);
            }
        }
    }
}

/// Whether the next words are `words` (read if so, left if not).
fn peek_words(e: &mut Engine, depth: Option<usize>, words: &[&str]) -> Result<bool, Error> {
    let mut read: Vec<Token> = Vec::new();
    for w in words {
        let first = match next(e, depth)? {
            Some(t) => t,
            None => {
                e.tex.push_tokens(read);
                return Ok(false);
            }
        };
        let ok = matches!(&first, Token::Char(c, Cat::Letter) if w.starts_with(*c));
        if !ok {
            e.tex.back(first);
            e.tex.push_tokens(read);
            return Ok(false);
        }
        let Token::Char(c, _) = first else { unreachable!() };
        let got = word(e, c);
        read.extend(tex::string_tokens(&format!("{got} ")));
        if got != *w {
            e.tex.push_tokens(read);
            return Ok(false);
        }
    }
    Ok(true)
}

/// After an error inside a path: skips to its `;`.
fn skip_rest(e: &mut Engine, depth: Option<usize>) -> Result<(), Error> {
    let mut level = 0usize;
    loop {
        let t = match depth {
            Some(d) => e.tex.get_token_until(d),
            None => e.tex.get_token(),
        };
        let Some(t) = t else { return Ok(()) };
        match t {
            Token::Char(_, Cat::Begin) => level += 1,
            Token::Char(_, Cat::End) => level = level.saturating_sub(1),
            Token::Char(';', _) if level == 0 => return Ok(()),
            _ => {}
        }
    }
}

fn apply_operation(e: &mut Engine, b: &mut Builder, op: Option<String>, to_opts: Vec<Token>, r: Coord) -> Result<(), Error> {
    match op.as_deref() {
        Some("--") => {
            line_to(e, b, r);
            place_mid_nodes(e, b, Some(0.5))?;
        }
        Some("-|") | Some("|-") => {
            let a = b.point();
            let corner = if op.as_deref() == Some("-|") { Point::new(r.p.x, a.y) } else { Point::new(a.x, r.p.y) };
            let start = a;
            line_to(e, b, Coord { p: corner, node: None });
            line_to(e, b, r.clone());
            // pos runs along both legs: 0.5, the default, is the corner
            b.last = Some(OnPath::Corner(start, corner, b.point()));
            place_mid_nodes(e, b, Some(0.5))?;
        }
        Some("to") => {
            to(e, b, to_opts, r)?;
            place_mid_nodes(e, b, Some(0.5))?;
        }
        _ => {
            b.move_to(r);
            place_mid_nodes(e, b, Some(0.5))?;
        }
    }
    Ok(())
}

/// A line to `r`, stopping at the border of the nodes on both ends.
fn line_to(e: &mut Engine, b: &mut Builder, r: Coord) {
    let Some(a) = b.current.clone() else {
        b.move_to(r);
        return;
    };
    let mut pa = a.p;
    let mut pb = r.p;
    if let Some(n) = a.node.as_ref().and_then(|n| e.nodes.get(n)) {
        let target = r.node.as_ref().and_then(|m| e.nodes.get(m)).map_or(r.p, |m| m.center());
        pa = n.border_toward(target);
        if let Some(Segment::MoveTo(m)) = b.segs.last_mut() {
            *m = pa;
        }
    }
    if let Some(n) = r.node.as_ref().and_then(|n| e.nodes.get(n)) {
        let from = a.node.as_ref().and_then(|m| e.nodes.get(m)).map_or(a.p, |m| m.center());
        pb = n.border_toward(from);
    }
    if !matches!(b.segs.last(), Some(Segment::MoveTo(_)) | Some(Segment::LineTo(_)) | Some(Segment::CurveTo(..))) {
        b.segs.push(Segment::MoveTo(pa));
    }
    b.segs.push(Segment::LineTo(pb));
    b.last = Some(OnPath::Segment(pa, None, pb));
    b.current = Some(r);
}

fn curve_to(e: &mut Engine, b: &mut Builder, c1: Point, c2: Point, r: Coord) {
    let a = b.current.clone().map_or(r.p, |a| a.p);
    if b.current.is_none() {
        b.segs.push(Segment::MoveTo(a));
    }
    if let Some(n) = b.current.as_ref().and_then(|c| c.node.as_ref()).and_then(|n| e.nodes.get(n)) {
        let pa = n.border_toward(c1);
        if let Some(Segment::MoveTo(m)) = b.segs.last_mut() {
            *m = pa;
        }
    }
    let mut pb = r.p;
    if let Some(n) = r.node.as_ref().and_then(|n| e.nodes.get(n)) {
        pb = n.border_toward(c2);
    }
    let start = match b.segs.last() {
        Some(Segment::MoveTo(q) | Segment::LineTo(q) | Segment::CurveTo(_, _, q)) => *q,
        _ => a,
    };
    b.segs.push(Segment::CurveTo(c1, c2, pb));
    b.last = Some(OnPath::Segment(start, Some((c1, c2)), pb));
    b.current = Some(r);
}

/// The control points of a `to` from `a` to `r` with the current `out`, `in`, `bend` and `looseness`.
fn to_controls(e: &mut Engine, a: Point, r: Point) -> Option<(Point, Point)> {
    let d = r.minus(a);
    let dist = d.norm();
    let base = d.y.atan2(d.x).to_degrees();
    let (out, inn) = match (e.st.bend, e.st.to_out, e.st.to_in) {
        (Some(g), _, _) => (base + g, base + 180.0 - g),
        (None, None, None) => return None,
        (None, o, i) => (o.unwrap_or(base), i.unwrap_or(base + 180.0)),
    };
    let lo = e.st.out_looseness.unwrap_or(e.st.looseness);
    let li = e.st.in_looseness.unwrap_or(e.st.looseness);
    let (so, co) = out.to_radians().sin_cos();
    let (si, ci) = inn.to_radians().sin_cos();
    let ko = 0.3915 * dist * lo;
    let ki = 0.3915 * dist * li;
    Some((a.plus(Point::new(co * ko, so * ko)), r.plus(Point::new(ci * ki, si * ki))))
}

fn to(e: &mut Engine, b: &mut Builder, opts: Vec<Token>, r: Coord) -> Result<(), Error> {
    e.begin_group();
    e.st.quotes_on_path = true;
    e.st.quoted.clear();
    let res = (|| {
        e.process_keys(tikra_tokens("every to/.try"), Some("/tikz"))?;
        e.process_keys(opts, Some("/tikz"))?;
        for (text, options) in std::mem::take(&mut e.st.quoted) {
            b.mid_nodes.push(MidNode { options, name: None, text, coordinate: false });
        }
        let a = b.point();
        let rp = r.node.as_ref().and_then(|n| e.nodes.get(n)).map_or(r.p, |n| n.center());
        let from = b.current.as_ref().and_then(|c| c.node.as_ref()).and_then(|n| e.nodes.get(n)).map_or(a, |n| n.center());
        Ok(to_controls(e, from, rp))
    })();
    e.end_group();
    match res? {
        Some((c1, c2)) => curve_to(e, b, c1, c2, r),
        None => line_to(e, b, r),
    }
    Ok(())
}

/// `edge[options] node{…} (target)`: a separate path, drawn after this one; the current point stays.
fn edge(e: &mut Engine, b: &mut Builder, opts: Vec<Token>, r: Coord, mids: Vec<MidNode>) -> Result<(), Error> {
    let saved_current = b.current.clone();
    let mut sub = Builder {
        segs: Vec::new(),
        current: None,
        subpath_start: None,
        last: None,
        node_items: Vec::new(),
        mid_nodes: mids,
        edges: Vec::new(),
        relative_base: None,
        origin: e.st.transform.apply(Point::default()),
    };
    if let Some(c) = &saved_current {
        sub.move_to(c.clone());
    }
    e.begin_group();
    e.st.quotes_on_path = true;
    e.st.quoted.clear();
    let res = (|| {
        e.process_keys(tikra_tokens("every edge/.try"), Some("/tikz"))?;
        e.process_keys(opts.clone(), Some("/tikz"))?;
        for (text, options) in std::mem::take(&mut e.st.quoted) {
            sub.mid_nodes.push(MidNode { options, name: None, text, coordinate: false });
        }
        let a = sub.point();
        let rp = r.node.as_ref().and_then(|n| e.nodes.get(n)).map_or(r.p, |n| n.center());
        let from = sub.current.as_ref().and_then(|c| c.node.as_ref()).and_then(|n| e.nodes.get(n)).map_or(a, |n| n.center());
        let controls = to_controls(e, from, rp);
        match controls {
            Some((c1, c2)) => curve_to(e, &mut sub, c1, c2, r.clone()),
            None => line_to(e, &mut sub, r.clone()),
        }
        place_mid_nodes(e, &mut sub, Some(0.5))
    })();
    e.end_group();
    res?;
    b.edges.push((sub.segs, opts, sub.node_items));
    b.current = saved_current;
    Ok(())
}

/// Nodes between `edge` and its target.
fn read_mid_nodes(e: &mut Engine, depth: Option<usize>) -> Result<Vec<MidNode>, Error> {
    let mut v = Vec::new();
    while let Some(Token::Char('n' | 'c', Cat::Letter)) = peek(e, depth)? {
        let Some(Token::Char(c, _)) = next(e, depth)? else { break };
        let w = word(e, c);
        if w != "node" && w != "coordinate" {
            e.tex.push_tokens(tex::string_tokens(&w));
            break;
        }
        let coordinate = w == "coordinate";
        let (options, name, _, text) = read_node(e, depth, coordinate)?;
        v.push(MidNode { options, name, text, coordinate });
    }
    Ok(v)
}

fn place_mid_nodes(e: &mut Engine, b: &mut Builder, default_pos: Option<f64>) -> Result<(), Error> {
    for m in std::mem::take(&mut b.mid_nodes) {
        let place = b.last.unwrap_or(OnPath::Point(b.point()));
        let items = e.node(NodeSpec { options: m.options, name: m.name, at: None, text: m.text, place, point: b.point(), coordinate: m.coordinate, default_pos })?;
        b.node_items.extend(items);
    }
    Ok(())
}

/// `node[options] (name) at (x,y) {text}`, the word already read.
#[allow(clippy::type_complexity)]
fn read_node(e: &mut Engine, depth: Option<usize>, coordinate: bool) -> Result<(Vec<Token>, Option<String>, Option<Point>, Vec<Token>), Error> {
    let mut options = Vec::new();
    let mut name = None;
    let mut at = None;
    loop {
        match next(e, depth)? {
            Some(t) if t.is_char('[') => {
                if !options.is_empty() {
                    options.push(Token::other(','));
                }
                options.extend(e.tex.read_until_char(']', false)?);
            }
            Some(t) if t.is_char('(') => {
                let n = read_coordinate_tokens(e)?;
                name = Some(e.tex.expand_to_string(n)?.trim().to_string());
            }
            Some(Token::Char('a', Cat::Letter)) => {
                // `at (x,y)`
                match e.tex.get_token() {
                    Some(Token::Char('t', Cat::Letter)) => match next(e, depth)? {
                        Some(t) if t.is_char('(') => at = Some(coordinate_point(e)?),
                        _ => return Err(Error::Figure("at without a point".into())),
                    },
                    other => {
                        if let Some(o) = other {
                            e.tex.back(o);
                        }
                        e.tex.back(Token::Char('a', Cat::Letter));
                        break;
                    }
                }
            }
            Some(Token::Char(_, Cat::Begin)) if !coordinate => {
                let text = e.tex.read_group_body()?;
                return Ok((options, name, at, text));
            }
            Some(t) => {
                e.tex.back(t);
                break;
            }
            None => break,
        }
    }
    if !coordinate {
        e.warn("node without text: '{…}' missing");
    }
    Ok((options, name, at, Vec::new()))
}

fn coordinate_point(e: &mut Engine) -> Result<Point, Error> {
    Ok(coordinate(e)?.p)
}

/// `circle (r)`, `circle (rx and ry)`, `circle[radius=…]`, `ellipse[x radius=…, y radius=…]`.
fn circle(e: &mut Engine, b: &mut Builder, depth: Option<usize>) -> Result<(), Error> {
    e.begin_group();
    let res = (|| -> Result<(f64, f64), Error> {
        if let Some(t) = peek(e, depth)? {
            if t.is_char('[') {
                let _ = next(e, depth)?;
                let o = e.tex.read_until_char(']', false)?;
                e.process_keys(o, Some("/tikz"))?;
            }
        }
        let (mut rx, mut ry) = radii(e)?;
        if let Some(t) = peek(e, depth)? {
            if t.is_char('(') {
                let _ = next(e, depth)?;
                let tokens = read_coordinate_tokens(e)?;
                let text = e.math_string(tokens)?;
                if let Some((a, bb)) = text.split_once(" and ") {
                    rx = e.radius_length(a)?;
                    ry = e.radius_length(bb)?;
                } else {
                    rx = e.radius_length(&text)?;
                    ry = rx;
                }
            }
        }
        Ok((rx, ry))
    })();
    e.end_group();
    let (rx, ry) = res?;
    let t = e.st.transform;
    let center = b.point();
    let mut v = arc_segments(center, rx, ry, 0.0, 360.0, &t);
    v.insert(0, Segment::MoveTo(center.plus(t.linear(Point::new(rx, 0.0)))));
    v.push(Segment::Close);
    b.segs.extend(v);
    b.subpath_start = Some(center);
    Ok(())
}

/// `radius`, `x radius`, `y radius` from the keys.
fn radii(e: &mut Engine) -> Result<(f64, f64), Error> {
    let get = |e: &mut Engine, k: &str| -> Result<Option<f64>, Error> {
        match keys::value(&e.tex, k) {
            Some(v) if !keys::trim(&v).is_empty() => {
                let s = e.math_string(v)?;
                Ok(Some(e.radius_length(&s)?))
            }
            _ => Ok(None),
        }
    };
    let r = get(e, "/tikz/radius")?;
    let rx = get(e, "/tikz/x radius")?.or(r).unwrap_or(0.0);
    let ry = get(e, "/tikz/y radius")?.or(r).unwrap_or(0.0);
    Ok((rx, ry))
}

impl Engine {
    /// A radius: with a unit in cm, without one in the x unit (as TikZ scales circle radii).
    fn radius_length(&mut self, text: &str) -> Result<f64, Error> {
        let (v, unit) = crate::expr::eval_length(crate::options::strip_braces(text.trim()), &self.vars())?;
        Ok(if unit { v } else { v * self.st.x.norm() })
    }
}

/// An elliptical arc from `a0` to `a1` degrees as cubic curves, cut the way `\pgfpatharc` cuts it:
/// 90° pieces while more than 115° are left, 60° pieces while more than 90° are, then the rest.
pub fn arc_segments(c: Point, rx: f64, ry: f64, a0: f64, a1: f64, t: &crate::scene::Transform) -> Vec<Segment> {
    let mut a = a0;
    let mut b = a1;
    if (b - a).abs() > 720.0 {
        // more than two turns draw the same ellipse: keep one turn and the rest
        b = a + (b - a).signum() * (360.0 + (b - a).abs() % 360.0);
    }
    let piece = |from: f64, to: f64| -> Segment {
        let delta = (to - from).abs();
        let k = if (delta - 90.0).abs() < 1e-12 { 0.55228475 } else { 4.0 / 3.0 * (delta.to_radians() / 4.0).tan() };
        let side = if to > from { 90.0 } else { -90.0 };
        let polar = |angle: f64, sx: f64, sy: f64| {
            let (s, co) = angle.to_radians().sin_cos();
            Point::new(sx * co, sy * s)
        };
        let p0 = polar(from, rx, ry);
        let p3 = polar(to, rx, ry);
        let c1 = p0.plus(polar(from + side, k * rx, k * ry));
        let c2 = p3.plus(polar(to - side, k * rx, k * ry));
        Segment::CurveTo(c.plus(t.linear(c1)), c.plus(t.linear(c2)), c.plus(t.linear(p3)))
    };
    let mut v = Vec::new();
    while (a - b).abs() > 90.0 && v.len() < 100 {
        let step = if (a - b).abs() > 115.0 { 90.0 } else { 60.0 };
        let next = if b > a { a + step } else { a - step };
        v.push(piece(a, next));
        a = next;
    }
    v.push(piece(a, b));
    v
}

/// `arc (start:end:radius)`, `arc (start:end:rx and ry)`, `arc[start angle=…, end angle=…, radius=…]`.
fn arc(e: &mut Engine, b: &mut Builder, depth: Option<usize>) -> Result<(), Error> {
    e.begin_group();
    let res = (|| -> Result<(f64, f64, f64, f64), Error> {
        let (mut a0, mut a1, mut rx, mut ry) = (0.0, 90.0, 0.0, 0.0);
        match peek(e, depth)? {
            Some(t) if t.is_char('(') => {
                let _ = next(e, depth)?;
                let tokens = read_coordinate_tokens(e)?;
                let text = e.math_string(tokens)?;
                let p: Vec<&str> = text.split(':').collect();
                if p.len() == 3 {
                    a0 = e.number(p[0])?;
                    a1 = e.number(p[1])?;
                    if let Some((x, y)) = p[2].split_once(" and ") {
                        rx = e.radius_length(x)?;
                        ry = e.radius_length(y)?;
                    } else {
                        rx = e.radius_length(p[2])?;
                        ry = rx;
                    }
                }
            }
            Some(t) if t.is_char('[') => {
                let _ = next(e, depth)?;
                let o = e.tex.read_until_char(']', false)?;
                e.process_keys(o, Some("/tikz"))?;
                let (x, y) = radii(e)?;
                rx = x;
                ry = y;
                let get = |e: &mut Engine, k: &str| -> Result<Option<f64>, Error> {
                    match keys::value(&e.tex, k) {
                        Some(v) if !keys::trim(&v).is_empty() => Ok(Some(e.number_of(v)?)),
                        _ => Ok(None),
                    }
                };
                let s = get(e, "/tikz/start angle")?;
                let en = get(e, "/tikz/end angle")?;
                let d = get(e, "/tikz/delta angle")?;
                match (s, en, d) {
                    (Some(s), Some(en), _) => {
                        a0 = s;
                        a1 = en;
                    }
                    (Some(s), None, Some(d)) => {
                        a0 = s;
                        a1 = s + d;
                    }
                    (None, Some(en), Some(d)) => {
                        a0 = en - d;
                        a1 = en;
                    }
                    (Some(s), None, None) => a0 = s,
                    _ => {}
                }
            }
            _ => {
                let (x, y) = radii(e)?;
                rx = x;
                ry = y;
            }
        }
        Ok((a0, a1, rx, ry))
    })();
    e.end_group();
    let (a0, a1, rx, ry) = res?;
    let t = e.st.transform;
    let p = b.point();
    if b.current.is_none() {
        b.segs.push(Segment::MoveTo(p));
    }
    let (s0, c0) = a0.to_radians().sin_cos();
    let center = p.minus(t.linear(Point::new(rx * c0, ry * s0)));
    let segs = arc_segments(center, rx, ry, a0, a1, &t);
    let (s1, c1) = a1.to_radians().sin_cos();
    let end = center.plus(t.linear(Point::new(rx * c1, ry * s1)));
    if let Some(Segment::CurveTo(c1p, c2p, _)) = segs.first() {
        b.last = Some(OnPath::Segment(p, Some((*c1p, *c2p)), end));
    }
    b.segs.extend(segs);
    b.current = Some(Coord { p: end, node: None });
    Ok(())
}

/// Multiples of `step` between `start` and `end`. None if the step is unusable.
fn ticks(start: f64, end: f64, step: f64) -> Option<Vec<f64>> {
    if !(step > 0.0 && step.is_finite() && start.is_finite() && end.is_finite()) {
        return None;
    }
    let first = (start / step - 1e-9).ceil();
    let count = ((end + 1e-9) / step).floor() - first + 1.0;
    if count > MAX_GRID_LINES as f64 {
        return None;
    }
    Some((0..count.max(0.0) as usize).map(|k| (first + k as f64) * step).collect())
}

fn grid(e: &mut Engine, b: &mut Builder, depth: Option<usize>) -> Result<(), Error> {
    e.begin_group();
    let res = (|| -> Result<(f64, f64), Error> {
        if let Some(t) = peek(e, depth)? {
            if t.is_char('[') {
                let _ = next(e, depth)?;
                let o = e.tex.read_until_char(']', false)?;
                e.process_keys(o, Some("/tikz"))?;
            }
        }
        Ok(e.st.grid_step)
    })();
    e.end_group();
    let step = res?;
    let r = target(e, b, depth, "grid")?;
    let t = e.st.transform;
    let inv = crate::reader::invert(&t);
    let (la, lb) = (inv.apply(b.point()), inv.apply(r.p));
    let (x0, x1) = (la.x.min(lb.x), la.x.max(lb.x));
    let (y0, y1) = (la.y.min(lb.y), la.y.max(lb.y));
    match (ticks(x0, x1, step.0), ticks(y0, y1, step.1)) {
        (Some(xs), Some(ys)) => {
            for x in xs {
                b.segs.push(Segment::MoveTo(t.apply(Point::new(x, y0))));
                b.segs.push(Segment::LineTo(t.apply(Point::new(x, y1))));
            }
            for y in ys {
                b.segs.push(Segment::MoveTo(t.apply(Point::new(x0, y))));
                b.segs.push(Segment::LineTo(t.apply(Point::new(x1, y))));
            }
        }
        _ => e.warn(format!("grid: step {} × {} is invalid or too fine (more than {MAX_GRID_LINES} lines): grid ignored", step.0, step.1)),
    }
    b.last = Some(OnPath::Point(r.p));
    b.current = Some(r);
    Ok(())
}

/// `plot[options] coordinates {…}` and `plot[options] (expression)`.
fn plot(e: &mut Engine, b: &mut Builder, depth: Option<usize>, continues: bool) -> Result<(), Error> {
    e.begin_group();
    let res = (|| -> Result<(Vec<Point>, bool), Error> {
        if let Some(t) = peek(e, depth)? {
            if t.is_char('[') {
                let _ = next(e, depth)?;
                let o = e.tex.read_until_char(']', false)?;
                e.process_keys(o, Some("/tikz"))?;
            }
        }
        let (d0, d1) = e.st.domain;
        let n = e.st.samples.max(2);
        let var = e.st.variable.clone();
        let smooth = e.st.smooth;
        let mut points = Vec::new();
        match next(e, depth)? {
            Some(Token::Char('c', Cat::Letter)) => {
                let w = word(e, 'c');
                if w != "coordinates" {
                    return Err(Error::Figure(format!("plot {w} not supported")));
                }
                let list = match next(e, depth)? {
                    Some(Token::Char(_, Cat::Begin)) => e.tex.read_group_body()?,
                    _ => return Err(Error::Figure("plot coordinates without a list".into())),
                };
                let depth0 = e.tex.depth();
                e.tex.push_frame(list);
                while let Some(t) = e.tex.get_x_token_until(depth0)? {
                    if t.is_char('(') {
                        let r = coordinate(e)?;
                        points.push(r.p);
                    }
                }
            }
            Some(Token::Char('f', Cat::Letter)) => {
                let w = word(e, 'f');
                let _ = e.argument();
                return Err(Error::Figure(format!("plot {w} (external data) not supported")));
            }
            Some(t) if t.is_char('(') => {
                let tokens = read_coordinate_tokens(e)?;
                let name = Token::cs(&var);
                for k in 0..n {
                    let v = d0 + (d1 - d0) * k as f64 / (n - 1) as f64;
                    e.tex.set_meaning(&name, macro_from(&super::pgf_number(v)), false);
                    let text = e.math_string(tokens.clone())?;
                    match e.coordinate(&text) {
                        Ok(r) if r.p.x.is_finite() && r.p.y.is_finite() => points.push(r.p),
                        Ok(_) => {}
                        Err(err) => {
                            e.warn(format!("plot: {err}"));
                            break;
                        }
                    }
                }
            }
            _ => return Err(Error::Figure("plot without an expression".into())),
        }
        Ok((points, smooth))
    })();
    e.end_group();
    let (points, smooth) = res?;
    if points.is_empty() {
        return Ok(());
    }
    if continues && b.current.is_some() {
        b.segs.push(Segment::LineTo(points[0]));
    } else {
        b.segs.push(Segment::MoveTo(points[0]));
        b.subpath_start = Some(points[0]);
    }
    if smooth && points.len() > 2 {
        for k in 0..points.len() - 1 {
            let p0 = points[k.saturating_sub(1)];
            let p1 = points[k];
            let p2 = points[k + 1];
            let p3 = points[(k + 2).min(points.len() - 1)];
            let c1 = p1.plus(p2.minus(p0).scale(1.0 / 6.0));
            let c2 = p2.minus(p3.minus(p1).scale(1.0 / 6.0));
            b.segs.push(Segment::CurveTo(c1, c2, p2));
        }
    } else {
        for p in &points[1..] {
            b.segs.push(Segment::LineTo(*p));
        }
    }
    let n = points.len();
    b.last = Some(OnPath::Segment(points[n.saturating_sub(2)], None, points[n - 1]));
    b.current = Some(Coord { p: points[n - 1], node: None });
    Ok(())
}

/// `parabola (b)`, `parabola bend (c) (b)`, `parabola[bend at end] (b)`.
fn parabola(e: &mut Engine, b: &mut Builder, depth: Option<usize>) -> Result<(), Error> {
    let mut bend_at_end = false;
    if let Some(t) = peek(e, depth)? {
        if t.is_char('[') {
            let _ = next(e, depth)?;
            let o = e.tex.read_until_char(']', false)?;
            bend_at_end = tex::tokens_to_string(&o).contains("bend at end");
        }
    }
    let mut bend = None;
    if let Some(Token::Char('b', Cat::Letter)) = peek(e, depth)? {
        let _ = next(e, depth)?;
        let w = word(e, 'b');
        if w == "bend" {
            bend = Some(target(e, b, depth, "parabola bend")?.p);
        }
    }
    let r = target(e, b, depth, "parabola")?;
    let a = b.point();
    if b.current.is_none() {
        b.segs.push(Segment::MoveTo(a));
    }
    // \pgfpathparabola: to the bend with its vertex there, then on to the end with its vertex at the bend
    let (bend_vector, end_vector) = match bend {
        Some(v) => (v.minus(a), r.p.minus(v)),
        None if bend_at_end => (r.p.minus(a), Point::default()),
        None => (Point::default(), r.p.minus(a)),
    };
    let mut from = a;
    if bend_vector.norm() > 1e-12 {
        let c = bend_vector;
        b.segs.push(Segment::CurveTo(from.plus(Point::new(0.1125 * c.x, 0.225 * c.y)), from.plus(Point::new(0.5 * c.x, c.y)), from.plus(c)));
        from = from.plus(c);
    }
    if end_vector.norm() > 1e-12 {
        let c = end_vector;
        b.segs.push(Segment::CurveTo(from.plus(Point::new(0.5 * c.x, 0.0)), from.plus(Point::new(0.8875 * c.x, 0.775 * c.y)), from.plus(c)));
    }
    b.current = Some(r);
    Ok(())
}

/// `let \p1 = (a), \n1 = {expr} in`: defines `\p1`, `\x1`, `\y1`, `\n1` for the rest of the path.
fn let_operation(e: &mut Engine, depth: Option<usize>) -> Result<(), Error> {
    loop {
        let Some(t) = next(e, depth)? else { return Ok(()) };
        let name = match &t {
            Token::Cs(n) => n.to_string(),
            Token::Char('i', Cat::Letter) => {
                let w = word(e, 'i');
                if w == "in" {
                    return Ok(());
                }
                return Err(Error::Figure(format!("let: '{w}' unexpected")));
            }
            Token::Char(',', _) => continue,
            other => return Err(Error::Figure(format!("let: '{}' unexpected", tex::show_token(other)))),
        };
        // the number after \p, \n: \p1, \p{name}
        let index = match next(e, depth)? {
            Some(Token::Char(_, Cat::Begin)) => tex::tokens_to_string(&e.tex.read_group_body()?),
            Some(Token::Char(c, _)) => c.to_string(),
            _ => String::new(),
        };
        match next(e, depth)? {
            Some(t) if t.is_char('=') => {}
            _ => return Err(Error::Figure("let: '=' missing".into())),
        }
        match name.as_str() {
            "p" => {
                let p = match next(e, depth)? {
                    Some(t) if t.is_char('(') => coordinate(e)?.p,
                    _ => return Err(Error::Figure("let \\p: a point is missing".into())),
                };
                let (x, y) = (p.x / crate::PT, p.y / crate::PT);
                e.tex.set_meaning(&Token::cs(&format!("p{index}")), macro_from(&format!("tikra cs:{x}pt,{y}pt")), false);
                e.tex.set_meaning(&Token::cs(&format!("x{index}")), macro_from(&format!("{x}pt")), false);
                e.tex.set_meaning(&Token::cs(&format!("y{index}")), macro_from(&format!("{y}pt")), false);
                // \p1 used alone as a coordinate reads as a point
                let _ = Point::default();
            }
            "n" => {
                let v = match next(e, depth)? {
                    Some(Token::Char(_, Cat::Begin)) => e.tex.read_group_body()?,
                    Some(t) => vec![t],
                    None => Vec::new(),
                };
                let text = e.math_string(v)?;
                let (val, unit) = crate::expr::eval_length(&text, &e.vars())?;
                let s = if unit { format!("{}pt", super::pgf_number(val / crate::PT)) } else { super::pgf_number(val) };
                e.tex.set_meaning(&Token::cs(&format!("n{index}")), macro_from(&s), false);
            }
            other => return Err(Error::Figure(format!("let: \\{other} is not \\p or \\n"))),
        }
    }
}

/// `\foreach` inside a path: its body is more path operations.
fn path_foreach(e: &mut Engine, b: &mut Builder) -> Result<(), Error> {
    let mut vars: Vec<Token> = Vec::new();
    loop {
        let Some(t) = e.tex.get_token() else { return Err(Error::Figure("\\foreach without 'in'".into())) };
        if t.is_char('[') {
            let _ = e.tex.read_until_char(']', false)?;
            continue;
        }
        if t.is_char('i') {
            if let Some(n) = e.tex.get_token() {
                if n.is_char('n') {
                    break;
                }
                e.tex.back(n);
            }
        }
        vars.push(t);
    }
    let names: Vec<Token> = vars.iter().filter(|t| matches!(t, Token::Cs(_))).cloned().collect();
    let list_tokens = match e.tex.next_nonspace() {
        Some(Token::Char(_, Cat::Begin)) => e.tex.read_group_body()?,
        Some(t) => vec![t],
        None => Vec::new(),
    };
    let list = e.tex.expand_fully(list_tokens)?;
    let body = match e.tex.next_nonspace() {
        Some(Token::Char(_, Cat::Begin)) => e.tex.read_group_body()?,
        Some(t) => vec![t],
        None => Vec::new(),
    };
    for item in super::foreach_items(&list) {
        e.begin_group();
        let parts: Vec<Vec<Token>> = item.split(|t| t.is_char('/')).map(|p| keys::trim(p).to_vec()).collect();
        for (i, n) in names.iter().enumerate() {
            let v = parts.get(i).or(parts.last()).cloned().unwrap_or_default();
            e.tex.set_meaning(n, super::macro_tokens(v), false);
        }
        let depth = e.tex.depth();
        e.tex.push_frame(body.clone());
        let r = operations(e, b, Some(depth));
        e.tex.drain_to(depth);
        // the path's settings stay; only the loop variables end with the iteration
        let st = e.st.clone();
        e.end_group();
        e.st = st;
        r?;
    }
    Ok(())
}
