//! One path statement: `\draw[->, thick] (A) -- (B) node[midway, above] {$F$} -- cycle;`.

use crate::color::Color;
use crate::label;
use crate::options::{self, Opt};
use crate::reader::{block, invert, line_width, Env};
use crate::scene::*;
use crate::{Error, PT};

/// A point that was read, remembering whether it came from a node (so the line stops at its border).
#[derive(Debug, Clone)]
struct Ref {
    p: Point,
    node: Option<String>,
}

pub fn tip(spec: &str) -> Option<Tip> {
    let s = spec.trim().trim_start_matches('{').trim_end_matches('}');
    let name = s.split('[').next().unwrap_or("").trim();
    Some(match name {
        ">" | "to" | "" => Tip::Default,
        "<" => Tip::Default,
        "latex" | "Latex" | "latex'" | "Triangle" | "triangle 45" | "triangle 60" | "triangle 90" => Tip::Latex,
        "stealth" | "Stealth" | "stealth'" | "Straight Barb" | "Kite" => Tip::Stealth,
        "|" | "Bar" => Tip::Bar,
        ">>" | "<<" => Tip::Double,
        "o" | "Circle" | "*" => Tip::Circle,
        _ => return None,
    })
}

/// `->`, `<->`, `-latex`, `-{Stealth[length=3mm]}`, `|->`: (start, end). None if it is not an arrow.
fn arrows(key: &str, default: Tip) -> Option<(Option<Tip>, Option<Tip>)> {
    let c: Vec<char> = key.chars().collect();
    // find the separating hyphen, outside braces
    let mut level = 0;
    let mut cut = None;
    for (i, ch) in c.iter().enumerate() {
        match ch {
            '{' | '[' => level += 1,
            '}' | ']' => level -= 1,
            '-' if level == 0 => {
                cut = Some(i);
                // the middle hyphen is the one with something plausible on both sides; the first one will do
                break;
            }
            _ => {}
        }
    }
    let i = cut?;
    let (a, b): (String, String) = (c[..i].iter().collect(), c[i + 1..].iter().collect());
    // `>` at the start or `<` at the end reverse the tip; the drawing is the same
    let side = |s: &str| -> Option<Option<Tip>> {
        let s = s.trim();
        if s.is_empty() {
            return Some(None);
        }
        let p = tip(s)?;
        Some(Some(if matches!(s, ">" | "<") { default } else { p }))
    };
    let start = side(&a)?;
    let end = side(&b)?;
    if start.is_none() && end.is_none() {
        return None;
    }
    Some((start, end))
}

/// Path style from the (already expanded) options, with the command's default.
pub fn style(env: &Env, options: &[Opt], command: &str) -> Style {
    let mut e = Style::default();
    let mut color = Color::BLACK;
    let mut wants_stroke = matches!(command, "draw" | "filldraw" | "shadedraw");
    let mut wants_fill = matches!(command, "fill" | "filldraw" | "shade" | "shadedraw");
    let mut stroke_color: Option<Color> = None;
    let mut fill_color: Option<Color> = None;
    let mut pattern: Option<String> = None;
    let mut pattern_color = Color::BLACK;
    for o in options {
        let v = o.value.as_deref();
        match (o.key.as_str(), v) {
            ("draw", None) => wants_stroke = true,
            ("draw", Some("none")) => wants_stroke = false,
            ("draw", Some(c)) => {
                wants_stroke = true;
                stroke_color = env.color(c);
            }
            ("fill", None) => wants_fill = true,
            ("fill", Some("none")) => wants_fill = false,
            ("fill", Some(c)) => {
                wants_fill = true;
                fill_color = env.color(c);
            }
            ("color", Some(c)) => {
                if let Some(c) = env.color(c) {
                    color = c;
                }
            }
            ("line width", Some(w)) => e.line_width = env.length(w).unwrap_or(e.line_width),
            ("dashed", None) => e.dash = Some(vec![3.0 * PT, 3.0 * PT]),
            ("densely dashed", None) => e.dash = Some(vec![3.0 * PT, 2.0 * PT]),
            ("loosely dashed", None) => e.dash = Some(vec![3.0 * PT, 6.0 * PT]),
            ("dotted", None) => e.dash = Some(vec![0.4 * PT, 2.0 * PT]),
            ("densely dotted", None) => e.dash = Some(vec![0.4 * PT, 1.0 * PT]),
            ("loosely dotted", None) => e.dash = Some(vec![0.4 * PT, 4.0 * PT]),
            ("dash dot", None) => e.dash = Some(vec![3.0 * PT, 2.0 * PT, 0.4 * PT, 2.0 * PT]),
            ("solid", None) => e.dash = None,
            ("dash pattern", Some(p)) => {
                let nums: Vec<f64> = p
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .chunks(2)
                    .filter_map(|pair| pair.get(1).and_then(|m| env.length(m).ok()))
                    .collect();
                if !nums.is_empty() {
                    e.dash = Some(nums);
                }
            }
            ("opacity", Some(x)) => {
                let x = env.number(x).unwrap_or(1.0);
                e.stroke_opacity = x;
                e.fill_opacity = x;
            }
            ("draw opacity", Some(x)) => e.stroke_opacity = env.number(x).unwrap_or(1.0),
            ("fill opacity", Some(x)) => e.fill_opacity = env.number(x).unwrap_or(1.0),
            ("rounded corners", None) => e.rounded_corners = 4.0 * PT,
            ("rounded corners", Some(r)) => e.rounded_corners = env.length(r).unwrap_or(4.0 * PT),
            ("sharp corners", None) => e.rounded_corners = 0.0,
            ("pattern", Some(p)) => pattern = Some(p.trim().to_string()),
            ("pattern color", Some(c)) => pattern_color = env.color(c).unwrap_or(Color::BLACK),
            (key, None) => {
                if let Some(w) = line_width(key) {
                    e.line_width = w;
                } else if let Some((s, f)) = arrows(key, env.default_tip) {
                    e.start_tip = s;
                    e.end_tip = f;
                } else if let Some(c) = env.color(key) {
                    color = c;
                }
            }
            _ => {}
        }
    }
    if wants_stroke {
        e.stroke = Some(stroke_color.unwrap_or(color));
    }
    if wants_fill {
        e.fill = Some(fill_color.unwrap_or(color));
    }
    if let Some(h) = pattern {
        e.pattern = Some((h, pattern_color));
        // the pattern is the fill: \fill[pattern=…] does not paint a solid color underneath
        if fill_color.is_none() {
            e.fill = None;
        }
    }
    e
}

/// Effective options: the scope's and the statement's, with styles expanded.
fn effective_options(env: &Env, own: &str, context: &str) -> Vec<Opt> {
    let mut o = Vec::new();
    for e in &env.scopes {
        o.extend(e.options.iter().cloned());
    }
    o.extend(env.styles.context(context));
    o.extend(options::parse(own));
    env.styles.expand(o)
}

/// Where the `:` of a polar coordinate is, outside braces and parentheses: `({90+45*2}:2)`.
fn top_level_colon(t: &str) -> Option<usize> {
    let mut depth = 0i32;
    for (i, c) in t.char_indices() {
        match c {
            '{' | '(' | '[' => depth += 1,
            '}' | ')' | ']' => depth -= 1,
            ':' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// The content of `( … )` in canvas coordinates.
pub fn absolute_coordinate(env: &Env, inner: &str) -> Result<Point, Error> {
    Ok(coordinate(env, inner)?.p)
}

fn coordinate(env: &Env, inner: &str) -> Result<Ref, Error> {
    let t = inner.trim();
    let sc = env.scope();
    if let Some(calc) = t.strip_prefix('$').and_then(|s| s.strip_suffix('$')) {
        return Ok(Ref { p: calc_expr(env, calc)?, node: None });
    }
    // name, name.anchor, name.angle
    let top_comma = options::split_top_level(t).len() > 1;
    let colon = top_level_colon(t);
    let has_colon = colon.is_some();
    if !top_comma && !has_colon {
        let (name, anchor) = match t.split_once('.') {
            Some((n, a)) if !n.chars().next().is_some_and(|c| c.is_ascii_digit() || c == '-') => (n.trim(), Some(a.trim())),
            _ => (t, None),
        };
        if let Some(node) = env.nodes.get(name) {
            return Ok(match anchor {
                Some(a) => Ref { p: node.anchor(a).ok_or(Error::Figure(format!("unknown anchor: {name}.{a}")))?, node: None },
                None => Ref { p: node.center, node: Some(name.to_string()) },
            });
        }
        return Err(Error::Figure(format!("unknown coordinate: ({t})")));
    }
    let local = if has_colon && !top_comma {
        // polar: (angle:radius)
        let (a, r) = t.split_at(colon.unwrap_or_default());
        let r = &r[1..];
        let angle = env.number(options::strip_braces(a))?;
        let (radius, has_unit) = crate::expr::eval_length(&env.expand(options::strip_braces(r)), &env.vars)?;
        let (s, c) = angle.to_radians().sin_cos();
        if has_unit {
            // with a unit the radius is in cm; without one, in axis units
            return Ok(Ref { p: sc.transform.apply(Point::new(radius * c, radius * s)), node: None });
        }
        sc.ux.scale(radius * c).plus(sc.uy.scale(radius * s))
    } else {
        let parts = options::split_top_level(t);
        let read = |s: &str, axis: Point| -> Result<Point, Error> {
            let (v, has_unit) = crate::expr::eval_length(&env.expand(options::strip_braces(s)), &env.vars)?;
            Ok(if has_unit {
                // an explicit length in cm follows the axis direction, without the unit vector's scale
                let n = axis.norm().max(1e-12);
                axis.scale(v / n)
            } else {
                axis.scale(v)
            })
        };
        read(&parts[0], sc.ux)?.plus(read(&parts[1], sc.uy)?)
    };
    Ok(Ref { p: sc.transform.apply(local), node: None })
}

/// `$(A)!0.5!(B)$`, `$(A)+(1,0)$`, `$0.5*(A)+0.5*(B)$`, `$(A)!0.5!90:(B)$`.
fn calc_expr(env: &Env, text: &str) -> Result<Point, Error> {
    let c: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut total = Point::default();
    let mut sign = 1.0;
    let mut last: Option<Point> = None;
    let mut factor = 1.0;
    while i < c.len() {
        match c[i] {
            ' ' => i += 1,
            '+' => {
                if let Some(p) = last.take() {
                    total = total.plus(p.scale(sign * factor));
                }
                sign = 1.0;
                factor = 1.0;
                i += 1;
            }
            '-' => {
                if let Some(p) = last.take() {
                    total = total.plus(p.scale(sign * factor));
                }
                sign = -1.0;
                factor = 1.0;
                i += 1;
            }
            '(' => {
                let (inside, end) = block(&c, i).ok_or(Error::Figure("unmatched parenthesis in calc".into()))?;
                last = Some(coordinate(env, &inside)?.p);
                i = end;
            }
            '!' => {
                // (A)!t!(B) or (A)!t!angle:(B)
                let rest: String = c[i + 1..].iter().collect();
                let t_end = rest.find('!').ok_or(Error::Figure("calc: unmatched '!'".into()))?;
                let t_text = &rest[..t_end];
                let after = &rest[t_end + 1..];
                let (angle, target_text) = match after.find('(') {
                    Some(j) if after[..j].contains(':') => (env.number(after[..j].trim_end_matches(':')).ok(), &after[j..]),
                    Some(j) => (None, &after[j..]),
                    None => return Err(Error::Figure("calc: missing the second point".into())),
                };
                let tc: Vec<char> = target_text.chars().collect();
                let (inside, end) = block(&tc, 0).ok_or(Error::Figure("calc: unmatched parenthesis".into()))?;
                let b = coordinate(env, &inside)?.p;
                let a = last.unwrap_or_default();
                let mut target = b;
                if let Some(ang) = angle {
                    let d = b.minus(a);
                    target = a.plus(Transform::rotation(ang).apply(d));
                }
                last = Some(match crate::expr::eval_length(&env.expand(t_text), &env.vars) {
                    // (A)!1cm!(B): distance from A
                    Ok((v, true)) => a.plus(target.minus(a).scale(v / target.minus(a).norm().max(1e-12))),
                    Ok((v, false)) => a.lerp(target, v),
                    Err(e) => return Err(e),
                });
                i += 1 + t_end + 1 + (after.len() - target_text.len()) + tc[..end].iter().collect::<String>().len();
            }
            _ => {
                // factor: 0.5*(A)
                let start = i;
                while i < c.len() && c[i] != '*' && c[i] != '(' {
                    i += 1;
                }
                let f: String = c[start..i].iter().collect();
                factor = env.number(f.trim()).unwrap_or(1.0);
                if c.get(i) == Some(&'*') {
                    i += 1;
                }
            }
        }
    }
    if let Some(p) = last {
        total = total.plus(p.scale(sign * factor));
    }
    Ok(total)
}

/// A node read in the middle of a path, created after the path: (options, text, name, position, last segment).
type PendingNode = (String, String, Option<String>, Point, Option<(Point, Point)>);

struct Builder<'a> {
    env: &'a mut Env,
    segments: Vec<Segment>,
    current: Option<Ref>,
    subpath_start: Option<Point>,
    /// start and end of the last segment (for `node[midway]`)
    last: Option<(Point, Point)>,
    nodes: Vec<PendingNode>,
    plot: (f64, f64, usize, String),
    decoration: Option<(String, f64, f64)>,
    /// `bend left`, `out`, `in`, `looseness` given to the whole statement: defaults for every `to`
    to_defaults: Vec<Opt>,
    /// nodes written between an operation and its target (`-- node {a} (B)`): placed once the segment exists
    mid_nodes: Vec<(String, String, Option<String>)>,
}

impl Builder<'_> {
    fn move_to(&mut self, r: Ref) {
        self.segments.push(Segment::MoveTo(r.p));
        self.subpath_start = Some(r.p);
        self.current = Some(r);
    }

    /// A line to `r`, stopping at the border of the nodes on both ends.
    fn line_to(&mut self, r: Ref) {
        let Some(a) = self.current.clone() else {
            self.move_to(r);
            return;
        };
        let mut pa = a.p;
        let mut pb = r.p;
        if let Some(n) = a.node.as_ref().and_then(|n| self.env.nodes.get(n)) {
            // the line leaves from the border of the starting node
            pa = n.border_toward(r.p.minus(n.center));
            if let Some(Segment::MoveTo(m)) = self.segments.last_mut() {
                *m = pa;
            }
        }
        if let Some(n) = r.node.as_ref().and_then(|n| self.env.nodes.get(n)) {
            pb = n.border_toward(a.p.minus(n.center));
        }
        if !matches!(self.segments.last(), Some(Segment::MoveTo(_)) | Some(Segment::LineTo(_)) | Some(Segment::CurveTo(..))) {
            self.segments.push(Segment::MoveTo(pa));
        }
        self.segments.push(Segment::LineTo(pb));
        self.last = Some((pa, pb));
        self.current = Some(Ref { p: r.p, node: r.node });
    }

    fn curve_to(&mut self, c1: Point, c2: Point, r: Ref) {
        let a = self.current.clone().map_or(r.p, |a| a.p);
        if self.current.is_none() {
            self.segments.push(Segment::MoveTo(a));
        }
        let mut pb = r.p;
        if let Some(n) = r.node.as_ref().and_then(|n| self.env.nodes.get(n)) {
            pb = n.border_toward(c2.minus(n.center));
        }
        self.segments.push(Segment::CurveTo(c1, c2, pb));
        self.last = Some((a, pb));
        self.current = Some(r);
    }

    fn point(&self) -> Point {
        self.current.as_ref().map_or(Point::default(), |a| a.p)
    }
}

/// Runs a path statement, already without its command (`\draw`) and with macros expanded.
pub fn statement(env: &mut Env, command: &str, text: &str) -> Result<(), Error> {
    let c: Vec<char> = text.chars().collect();
    let mut i = 0;
    let skip = |i: &mut usize| {
        while *i < c.len() && c[*i].is_whitespace() {
            *i += 1;
        }
    };
    skip(&mut i);
    let mut own = String::new();
    while c.get(i) == Some(&'[') {
        let (o, end) = block(&c, i).ok_or(Error::Figure("unmatched bracket".into()))?;
        if !own.is_empty() {
            own.push(',');
        }
        own.push_str(&o);
        i = end;
        skip(&mut i);
    }
    let options = effective_options(env, &own, "every path");
    let mut st = style(env, &options, command);
    let mut domain = (-5.0, 5.0);
    let mut samples = 25usize;
    let mut variable = "x".to_string();
    let mut decoration: Option<(String, f64, f64)> = None;
    let mut decorated = false;
    let mut shorten = (0.0, 0.0);
    // the path's step applies to its grids (\draw[step=5mm] (0,0) grid (2,2))
    let mut grid_step = (1.0, 1.0);
    for o in &options {
        match (o.key.as_str(), o.value.as_deref()) {
            ("domain", Some(d)) => {
                if let Some((a, b)) = d.split_once(':') {
                    domain = (env.number(a).unwrap_or(-5.0), env.number(b).unwrap_or(5.0));
                }
            }
            ("samples", Some(n)) => samples = env.number(n).map_or(25, |v| v.max(2.0) as usize).min(2000),
            ("variable", Some(v)) => variable = v.trim().trim_start_matches('\\').to_string(),
            ("decorate", None) => decorated = true,
            ("decoration", Some(d)) => {
                let mut kind = "zigzag".to_string();
                let mut amp = 2.5 * PT;
                let mut seg = 10.0 * PT;
                for p in options::parse(d) {
                    match (p.key.as_str(), p.value.as_deref()) {
                        ("amplitude", Some(v)) => amp = env.length(v).unwrap_or(amp),
                        ("segment length", Some(v)) => seg = env.length(v).unwrap_or(seg),
                        (name, None) => kind = name.to_string(),
                        _ => {}
                    }
                }
                decoration = Some((kind, amp, seg));
            }
            ("shorten >", Some(v)) => shorten.1 = env.length(v).unwrap_or(0.0),
            ("shorten <", Some(v)) => shorten.0 = env.length(v).unwrap_or(0.0),
            ("step", Some(v)) => {
                let s = env.length(v).unwrap_or(1.0);
                grid_step = (s, s);
            }
            ("xstep", Some(v)) => grid_step.0 = env.length(v).unwrap_or(1.0),
            ("ystep", Some(v)) => grid_step.1 = env.length(v).unwrap_or(1.0),
            _ => {}
        }
    }
    let mut b = Builder {
        env,
        segments: Vec::new(),
        current: None,
        subpath_start: None,
        last: None,
        nodes: Vec::new(),
        plot: (domain.0, domain.1, samples, variable),
        decoration: if decorated { decoration.or(Some(("zigzag".into(), 2.5 * PT, 10.0 * PT))) } else { None },
        to_defaults: options.iter().filter(|o| matches!(o.key.as_str(), "out" | "in" | "bend left" | "bend right" | "looseness")).cloned().collect(),
        mid_nodes: Vec::new(),
    };
    let mut operation: Option<String> = None;
    let mut relative_base: Option<Point> = None;

    while i < c.len() {
        skip(&mut i);
        if i >= c.len() {
            break;
        }
        let rest: String = c[i..].iter().take(12).collect();
        let word = |p: &str| rest.starts_with(p) && !c.get(i + p.len()).is_some_and(|x| x.is_ascii_alphabetic());
        if rest.starts_with("--") {
            operation = Some("--".into());
            i += 2;
        } else if rest.starts_with("-|") || rest.starts_with("|-") {
            operation = Some(rest[..2].to_string());
            i += 2;
        } else if rest.starts_with("..") {
            // .. controls (a) and (b) ..
            i += 2;
            skip(&mut i);
            let r2: String = c[i..].iter().take(8).collect();
            if r2.starts_with("controls") {
                i += 8;
                skip(&mut i);
                let (a, end) = block(&c, i).ok_or(Error::Figure("controls without a point".into()))?;
                i = end;
                let pa = coordinate(b.env, &a)?.p;
                skip(&mut i);
                let mut pc = pa;
                let r3: String = c[i..].iter().take(3).collect();
                if r3 == "and" {
                    i += 3;
                    skip(&mut i);
                    let (bb, end) = block(&c, i).ok_or(Error::Figure("controls without a second point".into()))?;
                    i = end;
                    pc = coordinate(b.env, &bb)?.p;
                }
                skip(&mut i);
                if c.get(i) == Some(&'.') && c.get(i + 1) == Some(&'.') {
                    i += 2;
                }
                skip(&mut i);
                let (target, end) = block(&c, i).ok_or(Error::Figure("controls without a target".into()))?;
                i = end;
                let r = coordinate(b.env, &target)?;
                b.curve_to(pa, pc, r);
            }
        } else if c[i] == '+' || c[i] == '(' {
            let mut update = true;
            let mut rel = false;
            if c[i] == '+' {
                rel = true;
                if c.get(i + 1) == Some(&'+') {
                    i += 2;
                } else {
                    i += 1;
                    update = false;
                }
                skip(&mut i);
            }
            let (inside, end) = block(&c, i).ok_or(Error::Figure("unmatched parenthesis".into()))?;
            i = end;
            let mut r = coordinate(b.env, &inside)?;
            if rel {
                // relative: the offset uses only the linear part of the transform
                let origin = b.env.scope().transform.apply(Point::default());
                let base = relative_base.unwrap_or_else(|| b.point());
                r = Ref { p: base.plus(r.p.minus(origin)), node: None };
                if !update {
                    relative_base = Some(base);
                } else {
                    relative_base = None;
                }
            } else {
                relative_base = None;
            }
            let op_now = operation.take();
            match op_now.as_deref() {
                Some("--") => b.line_to(r),
                Some("-|") => {
                    let a = b.point();
                    let corner = Point::new(r.p.x, a.y);
                    b.line_to(Ref { p: corner, node: None });
                    b.line_to(r);
                }
                Some("|-") => {
                    let a = b.point();
                    let corner = Point::new(a.x, r.p.y);
                    b.line_to(Ref { p: corner, node: None });
                    b.line_to(r);
                }
                Some(op) if op.starts_with("to") => {
                    let mut o = b.to_defaults.clone();
                    o.extend(options::parse(&op[2..]));
                    let a = b.point();
                    let d = r.p.minus(a);
                    let dist = d.norm();
                    let base_angle = d.y.atan2(d.x).to_degrees();
                    let mut out_angle = base_angle;
                    let mut in_angle = base_angle + 180.0;
                    let mut curved = false;
                    let mut looseness = 1.0;
                    for x in &o {
                        match (x.key.as_str(), x.value.as_deref()) {
                            ("out", Some(v)) => {
                                out_angle = b.env.number(v).unwrap_or(out_angle);
                                curved = true;
                            }
                            ("in", Some(v)) => {
                                in_angle = b.env.number(v).unwrap_or(in_angle);
                                curved = true;
                            }
                            ("bend left", v) => {
                                let g = v.and_then(|v| b.env.number(v).ok()).unwrap_or(30.0);
                                out_angle = base_angle + g;
                                in_angle = base_angle + 180.0 - g;
                                curved = true;
                            }
                            ("bend right", v) => {
                                let g = v.and_then(|v| b.env.number(v).ok()).unwrap_or(30.0);
                                out_angle = base_angle - g;
                                in_angle = base_angle + 180.0 + g;
                                curved = true;
                            }
                            ("looseness", Some(v)) => looseness = b.env.number(v).unwrap_or(1.0),
                            _ => {}
                        }
                    }
                    if curved {
                        let k = 0.3915 * dist * looseness;
                        let (so, co) = out_angle.to_radians().sin_cos();
                        let (si, ci) = in_angle.to_radians().sin_cos();
                        b.curve_to(a.plus(Point::new(co * k, so * k)), r.p.plus(Point::new(ci * k, si * k)), r);
                    } else {
                        b.line_to(r);
                    }
                }
                _ => b.move_to(r),
            }
            // as in TikZ: midway on a line or a curve, at the corner of -| and |-
            let default_pos = if matches!(op_now.as_deref(), Some("-|" | "|-")) { "pos=0" } else { "pos=0.5" };
            for (o, text, name) in std::mem::take(&mut b.mid_nodes) {
                b.nodes.push((format!("{default_pos},{o}"), text, name, b.point(), b.last));
            }
        } else if word("to") || word("edge") {
            i += if word("to") { 2 } else { 4 };
            skip(&mut i);
            let mut o = String::new();
            if c.get(i) == Some(&'[') {
                let (x, end) = block(&c, i).unwrap_or_default();
                o = x;
                i = end;
            }
            operation = Some(format!("to{o}"));
        } else if word("cycle") {
            i += 5;
            if let Some(p0) = b.subpath_start {
                if operation.take().is_some() {
                    b.segments.push(Segment::Close);
                    b.last = Some((b.point(), p0));
                    b.current = Some(Ref { p: p0, node: None });
                }
            }
        } else if word("rectangle") {
            i += 9;
            skip(&mut i);
            let (inside, end) = block(&c, i).ok_or(Error::Figure("rectangle without a corner".into()))?;
            i = end;
            let r = coordinate(b.env, &inside)?;
            let a = b.point();
            // corners in the local system, so the rectangle rotates with the scope
            let t = b.env.scope().transform;
            let inv = invert(&t);
            let (la, lb) = (inv.apply(a), inv.apply(r.p));
            let corners = [Point::new(lb.x, la.y), lb, Point::new(la.x, lb.y)];
            b.segments.push(Segment::MoveTo(a));
            for k in corners {
                b.segments.push(Segment::LineTo(t.apply(k)));
            }
            b.segments.push(Segment::Close);
            b.subpath_start = Some(a);
            b.current = Some(Ref { p: r.p, node: None });
            operation = None;
        } else if word("circle") || word("ellipse") {
            let is_ellipse = word("ellipse");
            i += if is_ellipse { 7 } else { 6 };
            skip(&mut i);
            let t = b.env.scope().transform;
            let (mut rx, mut ry) = (0.0, 0.0);
            if c.get(i) == Some(&'[') {
                let (o, end) = block(&c, i).unwrap_or_default();
                i = end;
                for x in options::parse(&o) {
                    match (x.key.as_str(), x.value.as_deref()) {
                        ("radius", Some(v)) => {
                            rx = b.env.length(v)?;
                            ry = rx;
                        }
                        ("x radius", Some(v)) => rx = b.env.length(v)?,
                        ("y radius", Some(v)) => ry = b.env.length(v)?,
                        _ => {}
                    }
                }
                skip(&mut i);
            }
            if c.get(i) == Some(&'(') {
                let (inside, end) = block(&c, i).ok_or(Error::Figure("circle without a radius".into()))?;
                i = end;
                if let Some((a, bb)) = inside.split_once(" and ") {
                    rx = b.env.length(a)?;
                    ry = b.env.length(bb)?;
                } else {
                    rx = b.env.length(&inside)?;
                    ry = rx;
                }
            }
            let center = b.point();
            b.segments.extend(ellipse_segments(center, rx, ry, &t));
            b.subpath_start = Some(center);
            operation = None;
        } else if word("arc") {
            i += 3;
            skip(&mut i);
            let t = b.env.scope().transform;
            let (mut a0, mut a1, mut rx, mut ry) = (0.0, 90.0, 1.0, 1.0);
            let mut delta: Option<f64> = None;
            if c.get(i) == Some(&'(') {
                let (inside, end) = block(&c, i).ok_or(Error::Figure("arc without parameters".into()))?;
                i = end;
                let p: Vec<&str> = inside.split(':').collect();
                if p.len() == 3 {
                    a0 = b.env.number(p[0])?;
                    a1 = b.env.number(p[1])?;
                    let radius = p[2];
                    if let Some((x, y)) = radius.split_once(" and ") {
                        rx = b.env.length(x)?;
                        ry = b.env.length(y)?;
                    } else {
                        rx = b.env.length(radius)?;
                        ry = rx;
                    }
                }
            } else if c.get(i) == Some(&'[') {
                let (o, end) = block(&c, i).unwrap_or_default();
                i = end;
                for x in options::parse(&o) {
                    match (x.key.as_str(), x.value.as_deref()) {
                        ("start angle", Some(v)) => a0 = b.env.number(v)?,
                        ("end angle", Some(v)) => a1 = b.env.number(v)?,
                        ("delta angle", Some(v)) => delta = Some(b.env.number(v)?),
                        ("radius", Some(v)) => {
                            rx = b.env.length(v)?;
                            ry = rx;
                        }
                        ("x radius", Some(v)) => rx = b.env.length(v)?,
                        ("y radius", Some(v)) => ry = b.env.length(v)?,
                        _ => {}
                    }
                }
            }
            if let Some(d) = delta {
                a1 = a0 + d;
            }
            let p = b.point();
            if b.current.is_none() {
                b.segments.push(Segment::MoveTo(p));
            }
            let (s0, c0) = a0.to_radians().sin_cos();
            let center = p.minus(t.linear(Point::new(rx * c0, ry * s0)));
            let segs = arc(center, rx, ry, a0, a1, &t);
            let (s1, c1) = a1.to_radians().sin_cos();
            let end = center.plus(t.linear(Point::new(rx * c1, ry * s1)));
            b.segments.extend(segs);
            b.last = Some((p, end));
            b.current = Some(Ref { p: end, node: None });
            operation = None;
        } else if word("grid") {
            i += 4;
            skip(&mut i);
            let mut step = grid_step;
            if c.get(i) == Some(&'[') {
                let (o, end) = block(&c, i).unwrap_or_default();
                i = end;
                for x in options::parse(&o) {
                    match (x.key.as_str(), x.value.as_deref()) {
                        ("step", Some(v)) => {
                            let s = b.env.length(v)?;
                            step = (s, s);
                        }
                        ("xstep", Some(v)) => step.0 = b.env.length(v)?,
                        ("ystep", Some(v)) => step.1 = b.env.length(v)?,
                        _ => {}
                    }
                }
                skip(&mut i);
            }
            let (inside, end) = block(&c, i).ok_or(Error::Figure("grid without a corner".into()))?;
            i = end;
            let r = coordinate(b.env, &inside)?;
            let t = b.env.scope().transform;
            let inv = invert(&t);
            let (la, lb) = (inv.apply(b.point()), inv.apply(r.p));
            let (x0, x1) = (la.x.min(lb.x), la.x.max(lb.x));
            let (y0, y1) = (la.y.min(lb.y), la.y.max(lb.y));
            match (ticks(x0, x1, step.0), ticks(y0, y1, step.1)) {
                (Some(xs), Some(ys)) => {
                    for x in xs {
                        b.segments.push(Segment::MoveTo(t.apply(Point::new(x, y0))));
                        b.segments.push(Segment::LineTo(t.apply(Point::new(x, y1))));
                    }
                    for y in ys {
                        b.segments.push(Segment::MoveTo(t.apply(Point::new(x0, y))));
                        b.segments.push(Segment::LineTo(t.apply(Point::new(x1, y))));
                    }
                }
                _ => b.env.warn(format!("grid: step {} × {} is invalid or too fine (more than {MAX_GRID_LINES} lines): grid ignored", step.0, step.1)),
            }
            b.current = Some(r);
            operation = None;
        } else if word("plot") {
            i += 4;
            skip(&mut i);
            let mut plot_options = String::new();
            if c.get(i) == Some(&'[') {
                let (o, end) = block(&c, i).unwrap_or_default();
                plot_options = o;
                i = end;
                skip(&mut i);
            }
            let (mut d0, mut d1, mut n, mut var) = b.plot.clone();
            let mut smooth = false;
            for x in options::parse(&plot_options) {
                match (x.key.as_str(), x.value.as_deref()) {
                    ("domain", Some(d)) => {
                        if let Some((a, bb)) = d.split_once(':') {
                            d0 = b.env.number(a)?;
                            d1 = b.env.number(bb)?;
                        }
                    }
                    ("samples", Some(v)) => n = (b.env.number(v)?.max(2.0) as usize).min(2000),
                    ("variable", Some(v)) => var = v.trim().trim_start_matches('\\').to_string(),
                    ("smooth", None) => smooth = true,
                    _ => {}
                }
            }
            let r2: String = c[i..].iter().take(11).collect();
            let mut points = Vec::new();
            if r2.starts_with("coordinates") {
                i += 11;
                skip(&mut i);
                let (list, end) = block(&c, i).ok_or(Error::Figure("plot coordinates without a list".into()))?;
                i = end;
                let lc: Vec<char> = list.chars().collect();
                let mut k = 0;
                while k < lc.len() {
                    if lc[k] == '(' {
                        let (d, f) = block(&lc, k).unwrap_or_default();
                        points.push(coordinate(b.env, &d)?.p);
                        k = f;
                    } else {
                        k += 1;
                    }
                }
            } else {
                let (expr_text, end) = block(&c, i).ok_or(Error::Figure("plot without an expression".into()))?;
                i = end;
                let saved = b.env.vars.values.get(&var).copied();
                let saved_macro = b.env.macros.get(&var).cloned();
                b.env.macros.remove(&var);
                for k in 0..n {
                    let v = d0 + (d1 - d0) * k as f64 / (n - 1) as f64;
                    b.env.vars.insert(var.clone(), v);
                    match coordinate(b.env, &expr_text) {
                        Ok(r) if r.p.x.is_finite() && r.p.y.is_finite() => points.push(r.p),
                        Ok(_) => {}
                        Err(e) => {
                            b.env.warn(format!("plot: {e}"));
                            break;
                        }
                    }
                }
                match saved {
                    Some(v) => b.env.vars.insert(var.clone(), v),
                    None => {
                        b.env.vars.values.remove(&var);
                    }
                }
                if let Some(m) = saved_macro {
                    b.env.macros.insert(var.clone(), m);
                }
            }
            if !points.is_empty() {
                let continues = operation.take().is_some() && b.current.is_some();
                if !continues {
                    b.segments.push(Segment::MoveTo(points[0]));
                    b.subpath_start = Some(points[0]);
                } else {
                    b.segments.push(Segment::LineTo(points[0]));
                }
                if smooth && points.len() > 2 {
                    for k in 0..points.len() - 1 {
                        let p0 = points[k.saturating_sub(1)];
                        let p1 = points[k];
                        let p2 = points[k + 1];
                        let p3 = points[(k + 2).min(points.len() - 1)];
                        let c1 = p1.plus(p2.minus(p0).scale(1.0 / 6.0));
                        let c2 = p2.minus(p3.minus(p1).scale(1.0 / 6.0));
                        b.segments.push(Segment::CurveTo(c1, c2, p2));
                    }
                } else {
                    for p in &points[1..] {
                        b.segments.push(Segment::LineTo(*p));
                    }
                }
                let n = points.len();
                b.last = Some((points[n.saturating_sub(2)], points[n - 1]));
                b.current = Some(Ref { p: points[n - 1], node: None });
            }
        } else if word("node") || word("coordinate") {
            let is_coord = word("coordinate");
            i += if is_coord { 10 } else { 4 };
            let (o, name, at, text, end) = read_node(&c, i)?;
            i = end;
            let at_given = at.is_some();
            let pos = match at {
                Some(a) => coordinate(b.env, &a)?.p,
                None => b.point(),
            };
            let o = if is_coord { format!("coordinate,{o}") } else { o };
            if at_given || operation.is_none() {
                b.nodes.push((o, text, name, pos, b.last));
            } else {
                b.mid_nodes.push((o, text, name));
            }
        } else if c[i] == '[' {
            // options in the middle of a path: they apply to the rest (accepted and ignored)
            let (_, end) = block(&c, i).unwrap_or_default();
            i = end;
        } else if c[i] == '{' {
            let (_, end) = block(&c, i).unwrap_or_default();
            i = end;
        } else {
            let junk: String = c[i..].iter().take(20).collect();
            b.env.warn(format!("unrecognized path fragment: '{junk}'"));
            break;
        }
    }

    // a node left between an operation and a target that never came
    for (o, text, name) in std::mem::take(&mut b.mid_nodes) {
        b.nodes.push((o, text, name, b.point(), b.last));
    }

    // post-processing: decoration, corners, shortening
    let mut segments = std::mem::take(&mut b.segments);
    if let Some((kind, amp, seg)) = b.decoration.take() {
        segments = decorate(&segments, &kind, amp, seg);
    }
    if st.rounded_corners > 0.0 {
        segments = round_corners(&segments, st.rounded_corners);
    }
    if shorten.0 > 0.0 || shorten.1 > 0.0 {
        shorten_ends(&mut segments, shorten.0, shorten.1);
    }
    let nodes = std::mem::take(&mut b.nodes);
    let env = b.env;
    if !segments.iter().all(valid_segment) {
        env.warn(format!("\\{command}: coordinate is not finite or beyond ±{MAX_COORDINATE} cm (division by zero?): path ignored"));
    } else if command == "clip" {
        env.scene.elements.push(Element::Clip(segments));
        if let Some(e) = env.scopes.last_mut() {
            e.clips += 1;
        }
    } else {
        let draws = st.stroke.is_some() || st.fill.is_some() || st.pattern.is_some();
        if draws && segments.iter().any(|s| !matches!(s, Segment::MoveTo(_))) {
            if st.stroke.is_none() {
                st.start_tip = None;
                st.end_tip = None;
            }
            env.scene.elements.push(Element::Path(Path { segments, style: st }));
        }
    }
    // nodes on a path inherit the path's color (a bare color or color=), not draw or fill;
    // with styles already expanded, pointer/.style={color=red} counts too
    let inherited_color = options
        .iter()
        .rev()
        .find(|o| (o.value.is_none() && env.color(&o.key).is_some()) || o.key == "color")
        .map(|o| o.value.clone().unwrap_or_else(|| o.key.clone()));
    for (o, text, name, pos, last) in nodes {
        let o = match &inherited_color {
            Some(c) => format!("text={c},{o}"),
            None => o,
        };
        create_node(env, &o, &text, name, pos, last)?;
    }
    Ok(())
}

/// Reads `node[opts] (name) at (x,y) {text}` from `i`. Returns (options, name, at, text, end).
#[allow(clippy::type_complexity)]
fn read_node(c: &[char], mut i: usize) -> Result<(String, Option<String>, Option<String>, String, usize), Error> {
    let mut opts = String::new();
    let mut name = None;
    let mut at = None;
    loop {
        while i < c.len() && c[i].is_whitespace() {
            i += 1;
        }
        match c.get(i) {
            Some('[') => {
                let (o, end) = block(c, i).ok_or(Error::Figure("unmatched bracket in node".into()))?;
                if !opts.is_empty() {
                    opts.push(',');
                }
                opts.push_str(&o);
                i = end;
            }
            Some('(') => {
                let (n, end) = block(c, i).ok_or(Error::Figure("unmatched parenthesis in node".into()))?;
                name = Some(n.trim().to_string());
                i = end;
            }
            Some('a') if c.get(i + 1) == Some(&'t') && !c.get(i + 2).is_some_and(|x| x.is_ascii_alphabetic()) => {
                i += 2;
                while i < c.len() && c[i].is_whitespace() {
                    i += 1;
                }
                let (p, end) = block(c, i).ok_or(Error::Figure("at without a point".into()))?;
                at = Some(p);
                i = end;
            }
            Some('{') => {
                let (t, end) = block(c, i).ok_or(Error::Figure("unmatched brace in node text".into()))?;
                return Ok((opts, name, at, t, end));
            }
            // a coordinate, or a node without text: it ends here
            _ => return Ok((opts, name, at, String::new(), i)),
        }
    }
}

/// Creates a node: typesets the text, computes the shape and places it by its anchor.
fn create_node(env: &mut Env, own: &str, text: &str, name: Option<String>, pos: Point, last: Option<(Point, Point)>) -> Result<(), Error> {
    let options = effective_options(env, own, "every node");
    let mut shape = Shape::Rectangle;
    let mut draws = false;
    let mut text_color: Option<Color> = None;
    let mut sep = (0.3333 * 0.3515, 0.3333 * 0.3515);
    let (mut min_w, mut min_h) = (0.0f64, 0.0f64);
    let mut text_width: Option<f64> = None;
    let mut align = 'c';
    let mut font = String::new();
    let mut anchor: Option<String> = None;
    let mut shift = Point::default();
    let mut pos_on_path: Option<f64> = None;
    let mut reference: Option<(String, Point, Option<String>)> = None;
    let mut aspect = 1.0;
    let mut rotation = 0.0;
    let mut node_scale = 1.0;
    let mut extra_labels: Vec<(String, String)> = Vec::new();
    let (dv, dh) = env.node_distance;
    let placements = ["above", "below", "left", "right", "above left", "above right", "below left", "below right"];
    for o in &options {
        let v = o.value.as_deref();
        match (o.key.as_str(), v) {
            ("circle", None) => shape = Shape::Circle,
            ("ellipse", None) => shape = Shape::Ellipse,
            ("rectangle", None) => shape = Shape::Rectangle,
            ("diamond", None) => shape = Shape::Diamond,
            ("coordinate", None) => shape = Shape::Coordinate,
            ("shape", Some(s)) => {
                shape = match s.trim() {
                    "circle" => Shape::Circle,
                    "ellipse" => Shape::Ellipse,
                    "diamond" => Shape::Diamond,
                    "coordinate" => Shape::Coordinate,
                    _ => Shape::Rectangle,
                }
            }
            ("aspect", Some(a)) => aspect = env.number(a).unwrap_or(1.0).max(0.05),
            ("draw", _) => draws = !matches!(v, Some("none")),
            ("text", Some(c)) => text_color = env.color(c),
            ("inner sep", Some(s)) => {
                let m = env.length(s).unwrap_or(sep.0);
                sep = (m, m);
            }
            ("inner xsep", Some(s)) => sep.0 = env.length(s).unwrap_or(sep.0),
            ("inner ysep", Some(s)) => sep.1 = env.length(s).unwrap_or(sep.1),
            ("minimum width", Some(s)) => min_w = env.length(s).unwrap_or(0.0),
            ("minimum height", Some(s)) => min_h = env.length(s).unwrap_or(0.0),
            ("minimum size", Some(s)) => {
                let m = env.length(s).unwrap_or(0.0);
                min_w = m;
                min_h = m;
            }
            ("text width", Some(s)) => text_width = env.length(s).ok(),
            ("align", Some(a)) => {
                align = match a.trim() {
                    "left" | "flush left" => 'l',
                    "right" | "flush right" => 'r',
                    _ => 'c',
                }
            }
            ("text centered", None) => align = 'c',
            ("font", Some(f)) => font.push_str(f),
            ("anchor", Some(a)) => anchor = Some(a.trim().to_string()),
            ("xshift", Some(s)) => shift.x += env.length(s).unwrap_or(0.0),
            ("yshift", Some(s)) => shift.y += env.length(s).unwrap_or(0.0),
            ("midway", None) => pos_on_path = Some(0.5),
            ("near start", None) => pos_on_path = Some(0.25),
            ("near end", None) => pos_on_path = Some(0.75),
            ("at start", None) => pos_on_path = Some(0.0),
            ("at end", None) => pos_on_path = Some(1.0),
            ("very near start", None) => pos_on_path = Some(0.125),
            ("very near end", None) => pos_on_path = Some(0.875),
            ("pos", Some(p)) => pos_on_path = env.number(p).ok(),
            ("rotate", Some(r)) => rotation = env.number(r).unwrap_or(0.0),
            ("scale", Some(s)) => node_scale = env.number(s).unwrap_or(1.0),
            ("label", Some(l)) => {
                let (side, t) = l.split_once(':').unwrap_or(("above", l));
                let side = side.trim().trim_start_matches('[').to_string();
                extra_labels.push((side, t.to_string()));
            }
            (p, value) if placements.contains(&p) => {
                anchor = Some(
                    match p {
                        "above" => "south",
                        "below" => "north",
                        "left" => "east",
                        "right" => "west",
                        "above left" => "south east",
                        "above right" => "south west",
                        "below left" => "north east",
                        _ => "north west",
                    }
                    .to_string(),
                );
                let dir = |vert: f64, hor: f64| -> Point {
                    let sy = if p.contains("above") { 1.0 } else if p.contains("below") { -1.0 } else { 0.0 };
                    let sx = if p.contains("left") { -1.0 } else if p.contains("right") { 1.0 } else { 0.0 };
                    Point::new(sx * hor, sy * vert)
                };
                match value {
                    None => {}
                    Some(val) if val.contains(" of ") || val.starts_with("of ") => {
                        // positioning library: below=1cm of A, below left=1cm and 2cm of A, below=of A
                        let (dist, target) = val.split_once("of ").map(|(d, a)| (d.trim(), a.trim())).unwrap_or(("", val));
                        let (v_, h_) = if dist.is_empty() {
                            (dv, dh)
                        } else if let Some((a, bb)) = dist.split_once(" and ") {
                            (env.length(a).unwrap_or(dv), env.length(bb).unwrap_or(dh))
                        } else {
                            let m = env.length(dist).unwrap_or(dv);
                            (m, m)
                        };
                        let target_anchor = match p {
                            "above" => "north",
                            "below" => "south",
                            "left" => "west",
                            "right" => "east",
                            "above left" => "north west",
                            "above right" => "north east",
                            "below left" => "south west",
                            _ => "south east",
                        };
                        let (target_name, explicit_anchor) = match target.split_once('.') {
                            Some((n, a)) => (n.trim().to_string(), Some(a.trim().to_string())),
                            None => (target.to_string(), None),
                        };
                        let n = env.nodes.get(&target_name).ok_or(Error::Figure(format!("unknown node in 'of': {target}")))?;
                        let target_point = n.anchor(explicit_anchor.as_deref().unwrap_or(target_anchor)).unwrap_or(n.center);
                        reference = Some((p.to_string(), target_point.plus(dir(v_, h_)), None));
                    }
                    Some(val) => {
                        let m = env.length(val).unwrap_or(0.0);
                        shift = shift.plus(dir(m, m));
                    }
                }
            }
            _ => {}
        }
    }
    let st = style(env, &options, if draws { "draw" } else { "path" });
    let st = Style { stroke: if draws { st.stroke.or(Some(Color::BLACK)) } else { None }, ..st };
    let default_color = options
        .iter()
        .rev()
        .filter(|o| o.value.is_none())
        .find_map(|o| env.color(&o.key))
        .or_else(|| options.iter().find(|o| o.key == "color").and_then(|o| o.value.as_deref().and_then(|v| env.color(v))))
        .unwrap_or(Color::BLACK);
    let text = text.trim();
    let lbl = if shape == Shape::Coordinate || text.is_empty() {
        None
    } else {
        // `scale` on a node scales its text too, as in TikZ
        let mut l = label::compose_label(text, &font, text_width.map(|w| w / node_scale), align, text_color.unwrap_or(default_color), &mut env.warnings);
        l.scale *= node_scale;
        l.line_height *= node_scale;
        Some(l)
    };
    let (w, h) = match &lbl {
        Some(r) => (text_width.unwrap_or_else(|| r.width()), r.height() + r.depth()),
        None => (0.0, 0.0),
    };
    let (hw, hh) = match shape {
        Shape::Coordinate => (0.0, 0.0),
        Shape::Rectangle => ((w / 2.0 + sep.0).max(min_w / 2.0), (h / 2.0 + sep.1).max(min_h / 2.0)),
        Shape::Circle => {
            let r = ((w / 2.0 + sep.0).hypot(h / 2.0 + sep.1)).max(min_w.max(min_h) / 2.0);
            (r, r)
        }
        Shape::Ellipse => (((w / 2.0 + sep.0) * std::f64::consts::SQRT_2).max(min_w / 2.0), ((h / 2.0 + sep.1) * std::f64::consts::SQRT_2).max(min_h / 2.0)),
        Shape::Diamond => {
            let (a, bb) = (w / 2.0 + sep.0, h / 2.0 + sep.1);
            ((a + bb * aspect).max(min_w / 2.0), (bb + a / aspect).max(min_h / 2.0))
        }
    };
    // reference position: on the path, relative to another node, or the given point
    let mut point = match (pos_on_path, last) {
        (Some(t), Some((a, bb))) => a.lerp(bb, t),
        _ => pos,
    };
    if let Some((_, p, _)) = &reference {
        point = *p;
    }
    let mut node = Node { center: Point::default(), shape, half_width: hw, half_height: hh, label: lbl, style: st, rotation };
    let a = anchor.as_deref().unwrap_or("center");
    let off = node.anchor(a).unwrap_or_default();
    node.center = point.minus(off).plus(shift);
    if !valid_point(node.center) || !(hw.is_finite() && hh.is_finite() && hw.abs() <= MAX_COORDINATE && hh.abs() <= MAX_COORDINATE) {
        env.warn(format!("node '{text}': position or size is not finite or beyond ±{MAX_COORDINATE} cm: ignored"));
        return Ok(());
    }
    if let Some(n) = &name {
        env.nodes.insert(n.clone(), node.clone());
    }
    let pending_labels: Vec<(String, String, Point)> = extra_labels
        .into_iter()
        .map(|(side, t)| {
            let p = match side.as_str() {
                "above" | "north" => node.anchor("north"),
                "below" | "south" => node.anchor("south"),
                "left" | "west" => node.anchor("west"),
                "right" | "east" => node.anchor("east"),
                other => other.parse::<f64>().ok().map(|g| node.border_at_angle(g)).or_else(|| node.anchor(other)),
            }
            .unwrap_or(node.center);
            (side, t, p)
        })
        .collect();
    if shape != Shape::Coordinate || name.is_none() {
        env.scene.elements.push(Element::Node(node));
    }
    for (side, t, p) in pending_labels {
        let placement = match side.as_str() {
            "above" | "north" | "90" => "above",
            "below" | "south" | "270" | "-90" => "below",
            "left" | "west" | "180" => "left",
            "right" | "east" | "0" => "right",
            _ => "above",
        };
        create_node(env, placement, &t, None, p, None)?;
    }
    Ok(())
}

/// An ellipse as four Bézier curves, transformed.
fn ellipse_segments(c: Point, rx: f64, ry: f64, t: &Transform) -> Vec<Segment> {
    let mut v = arc(c, rx, ry, 0.0, 360.0, t);
    v.insert(0, Segment::MoveTo(c.plus(t.linear(Point::new(rx, 0.0)))));
    v.push(Segment::Close);
    v
}

/// An elliptical arc from `a0` to `a1` degrees, in pieces of up to 90°, as cubic curves.
fn arc(c: Point, rx: f64, ry: f64, a0: f64, a1: f64, t: &Transform) -> Vec<Segment> {
    let mut total = a1 - a0;
    if total.abs() > 720.0 {
        // more than two turns draw the same circle: keep one turn plus the rest, with the same end point
        total = total.signum() * (360.0 + total.abs() % 360.0);
    }
    let n = ((total.abs() / 90.0).ceil() as usize).max(1);
    let step = total / n as f64;
    let mut v = Vec::new();
    for k in 0..n {
        let (b0, b1) = ((a0 + step * k as f64).to_radians(), (a0 + step * (k + 1) as f64).to_radians());
        let kk = 4.0 / 3.0 * ((b1 - b0) / 4.0).tan();
        let p = |a: f64| Point::new(rx * a.cos(), ry * a.sin());
        let d = |a: f64| Point::new(-rx * a.sin(), ry * a.cos());
        let (p0, p3) = (p(b0), p(b1));
        let c1 = p0.plus(d(b0).scale(kk));
        let c2 = p3.minus(d(b1).scale(kk));
        v.push(Segment::CurveTo(c.plus(t.linear(c1)), c.plus(t.linear(c2)), c.plus(t.linear(p3))));
    }
    v
}

/// Zigzag or wave over straight segments.
fn decorate(segs: &[Segment], kind: &str, amp: f64, len_step: f64) -> Vec<Segment> {
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
                // a zero or negative segment length becomes a single step; the total number of steps is capped
                let n = if len_step > 0.0 { ((len / len_step).round() as usize).clamp(1, MAX_DECORATION_STEPS) } else { 1 };
                if kind == "random steps" {
                    // steps with a deterministic pseudo-random offset: same input, same drawing
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
                let steps = n * if kind == "zigzag" { 2 } else { 8 };
                for k in 1..=steps {
                    let f = k as f64 / steps as f64;
                    let base = a.plus(d.scale(f));
                    let offset = match kind {
                        "zigzag" => {
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
    // expands Close into a line to the subpath start, so the closing corner is rounded too
    let mut points: Vec<Vec<Point>> = Vec::new();
    let mut closed: Vec<bool> = Vec::new();
    let mut others = false;
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
            Segment::CurveTo(..) => others = true,
        }
    }
    if others {
        return segs.to_vec();
    }
    let mut out = Vec::new();
    for (pts, is_closed) in points.iter().zip(closed) {
        let mut p: Vec<Point> = pts.clone();
        if is_closed && p.len() > 2 && p.first().map(|a| a.minus(*p.last().unwrap()).norm() > 1e-9).unwrap_or(false) {
            // closed without repeating the first point
        } else if is_closed && p.len() > 2 {
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
            let a = here.plus(d1.scale(rr / d1.norm().max(1e-12)));
            let b = here.plus(d2.scale(rr / d2.norm().max(1e-12)));
            (a, here, b)
        };
        let indices: Vec<usize> = if is_closed { (0..n).collect() } else { (1..n - 1).collect() };
        if is_closed {
            let (_, _, b) = corner(0);
            out.push(Segment::MoveTo(b));
        } else {
            out.push(Segment::MoveTo(p[0]));
        }
        for &k in indices.iter().skip(if is_closed { 1 } else { 0 }) {
            let (a, v, b) = corner(k);
            out.push(Segment::LineTo(a));
            out.push(Segment::CurveTo(a.lerp(v, 0.55), b.lerp(v, 0.55), b));
        }
        if is_closed {
            let (a, v, b) = corner(0);
            out.push(Segment::LineTo(a));
            out.push(Segment::CurveTo(a.lerp(v, 0.55), b.lerp(v, 0.55), b));
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
    // a LineTo always follows a point: the builder puts a MoveTo after every Close
    if let (Segment::LineTo(b), Segment::MoveTo(a) | Segment::LineTo(a) | Segment::CurveTo(_, _, a)) = (segs[k], segs[k - 1]) {
        let d = b.minus(a);
        let n = d.norm();
        if n > end && end > 0.0 {
            segs[k] = Segment::LineTo(b.minus(d.scale(end / n)));
        }
    }
}

/// Largest accepted coordinate, in cm (one kilometer): beyond that it is a typo or a bad computation.
pub const MAX_COORDINATE: f64 = 100_000.0;
/// Cap on `grid` lines in each direction.
pub const MAX_GRID_LINES: usize = 10_000;
/// Cap on decoration steps per segment.
pub const MAX_DECORATION_STEPS: usize = 10_000;

fn valid_point(p: Point) -> bool {
    p.x.is_finite() && p.y.is_finite() && p.x.abs() <= MAX_COORDINATE && p.y.abs() <= MAX_COORDINATE
}

fn valid_segment(s: &Segment) -> bool {
    match s {
        Segment::MoveTo(p) | Segment::LineTo(p) => valid_point(*p),
        Segment::CurveTo(a, b, p) => valid_point(*a) && valid_point(*b) && valid_point(*p),
        Segment::Close => true,
    }
}

/// Multiples of `step` between `start` and `end` (by index, without accumulating error). None if the step is unusable.
fn ticks(start: f64, end: f64, step: f64) -> Option<Vec<f64>> {
    if !(step > 0.0 && step.is_finite() && start.is_finite() && end.is_finite()) {
        return None;
    }
    let first = (start / step).ceil();
    let count = ((end + 1e-9) / step).floor() - first + 1.0;
    if count > MAX_GRID_LINES as f64 {
        return None;
    }
    Some((0..count.max(0.0) as usize).map(|k| (first + k as f64) * step).collect())
}
