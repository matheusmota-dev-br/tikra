//! Coordinates, as `\tikz@scan@one@point` reads them, in canvas centimeters.
//!
//! `(1,2)`, `(1cm,2pt)`, `(30:2)`, `(1,2,3)`, `(a)`, `(a.north east)`, `(a.30)`, `(a |- b)`,
//! `([xshift=1cm]a.east)`, the `cs:` systems (`canvas`, `xyz`, `polar`, `node`), `calc`
//! (`$(a)!0.5!(b)$`, `$(a)+(1,0)$`, `$2*(a)$`, `$(a)!1cm!90:(b)$`) and `(intersection of a--b and c--d)`.

use crate::scene::{Point, Transform};
use crate::tex;
use crate::Error;

use super::Engine;

/// A point read from a coordinate, and the node it names (a line to a node stops at its border).
#[derive(Debug, Clone)]
pub struct Coord {
    pub p: Point,
    pub node: Option<String>,
}

/// Where the `:` of a polar coordinate is, outside braces and parentheses: `({90+45*2}:2)`.
fn top_level(t: &str, target: char) -> Option<usize> {
    let mut depth = 0i32;
    for (i, c) in t.char_indices() {
        match c {
            '{' | '(' | '[' => depth += 1,
            '}' | ')' | ']' => depth -= 1,
            c if c == target && depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// Splits at top-level commas.
pub fn split_commas(t: &str) -> Vec<String> {
    crate::options::split_top_level(t)
}

fn strip_braces(t: &str) -> &str {
    crate::options::strip_braces(t)
}

impl Engine {
    /// The local (untransformed) vector of `(x,y)` or `(x,y,z)` with the current unit vectors.
    fn xyz(&mut self, parts: &[String]) -> Result<Point, Error> {
        let axes = [self.st.x, self.st.y, self.st.z];
        let mut p = Point::default();
        for (k, s) in parts.iter().enumerate().take(3) {
            let (v, unit) = crate::expr::eval_length(strip_braces(s), &self.vars())?;
            let axis = axes[k];
            p = p.plus(if unit {
                // a length follows the axis direction, without its scale (pgf: \pgfpointxy vs \pgfpoint)
                if k == 0 {
                    Point::new(v, 0.0)
                } else if k == 1 {
                    Point::new(0.0, v)
                } else {
                    let n = axis.norm().max(1e-12);
                    axis.scale(v / n)
                }
            } else {
                axis.scale(v)
            });
        }
        Ok(p)
    }

    /// Reads a coordinate's text (macros already expanded) as a canvas point.
    pub fn coordinate(&mut self, inner: &str) -> Result<Coord, Error> {
        let t = inner.trim();
        // ([options] point): the options transform this point only
        if let Some(rest) = t.strip_prefix('[') {
            let Some(end) = top_level(&format!("[{rest}"), ']') else { return Err(Error::Figure(format!("unmatched bracket in ({t})"))) };
            let opts = &rest[..end - 1];
            let point = rest[end..].trim().to_string();
            self.begin_group();
            self.st.pending = Transform::default();
            let r = self.process_keys(super::tikra_tokens(opts), Some("/tikz")).and_then(|_| {
                let pending = self.st.pending;
                let before = self.st.transform;
                // the point in the system with the extra transformations, as TikZ applies them
                self.st.transform = before.compose(&pending);
                self.st.pending = Transform::default();
                self.coordinate(&point)
            });
            self.end_group();
            return r;
        }
        if let Some(calc) = t.strip_prefix('$').and_then(|s| s.strip_suffix('$')) {
            return Ok(Coord { p: self.calc(calc)?, node: None });
        }
        if let Some(rest) = t.strip_prefix("intersection of") {
            return Ok(Coord { p: self.line_intersection(rest)?, node: None });
        }
        // (a |- b), (a -| b)
        for op in ["|-", "-|"] {
            if let Some(i) = find_perpendicular(t, op) {
                let a = self.coordinate(&t[..i])?.p;
                let b = self.coordinate(&t[i + 2..])?.p;
                return Ok(Coord { p: if op == "|-" { Point::new(a.x, b.y) } else { Point::new(b.x, a.y) }, node: None });
            }
        }
        // a coordinate system: (canvas cs:x=1cm,y=2cm)
        if let Some(i) = t.find(" cs:").or_else(|| t.find("cs:").filter(|&k| k > 0)) {
            let system = t[..i].trim();
            let args = t[i..].trim_start().trim_start_matches("cs:").to_string();
            return self.coordinate_system(system, &args);
        }
        let commas = split_commas(t);
        let colon = top_level(t, ':');
        if commas.len() == 1 && colon.is_none() {
            // a node: name, name.anchor, name.angle
            let (name, anchor) = match t.split_once('.') {
                Some((n, a)) if !n.chars().next().is_some_and(|c| c.is_ascii_digit() || c == '-' || c == '+') && self.nodes.contains_key(n.trim()) => (n.trim(), Some(a.trim())),
                _ => (t, None),
            };
            if let Some(node) = self.nodes.get(name) {
                return Ok(match anchor {
                    Some(a) => Coord { p: node.anchor(a).ok_or_else(|| Error::Figure(format!("unknown anchor: {name}.{a}")))?, node: None },
                    None => Coord { p: node.center(), node: Some(name.to_string()) },
                });
            }
            return Err(Error::Figure(format!("unknown coordinate: ({t})")));
        }
        let local = if let (Some(c), 1) = (colon, commas.len()) {
            // polar: (angle:radius), (angle:x radius and y radius)
            let angle = self.number(strip_braces(&t[..c]))?;
            let r = t[c + 1..].trim();
            let (rx, ry, unit) = if let Some((a, b)) = r.split_once(" and ") {
                let (x, ux) = crate::expr::eval_length(strip_braces(a), &self.vars())?;
                let (y, _) = crate::expr::eval_length(strip_braces(b), &self.vars())?;
                (x, y, ux)
            } else {
                let (v, u) = crate::expr::eval_length(strip_braces(r), &self.vars())?;
                (v, v, u)
            };
            let (s, c) = angle.to_radians().sin_cos();
            if unit {
                Point::new(rx * c, ry * s)
            } else {
                self.st.x.scale(rx * c).plus(self.st.y.scale(ry * s))
            }
        } else {
            self.xyz(&commas)?
        };
        Ok(Coord { p: self.st.transform.apply(local), node: None })
    }

    fn coordinate_system(&mut self, system: &str, args: &str) -> Result<Coord, Error> {
        let items: Vec<(String, String)> = split_commas(args)
            .into_iter()
            .map(|p| match p.split_once('=') {
                Some((k, v)) => (k.trim().to_string(), strip_braces(v.trim()).to_string()),
                None => (p.trim().to_string(), String::new()),
            })
            .collect();
        let get = |k: &str| items.iter().find(|(a, _)| a == k).map(|(_, b)| b.clone());
        match system {
            "tikra" => {
                // a point already on the canvas (\p1 of `let`)
                let x = self.length(&items.first().map(|i| i.0.clone()).unwrap_or_default())?;
                let y = self.length(&items.get(1).map(|i| i.0.clone()).unwrap_or_default())?;
                Ok(Coord { p: Point::new(x, y), node: None })
            }
            "canvas" => {
                let x = get("x").map(|v| self.length(&v)).transpose()?.unwrap_or(0.0);
                let y = get("y").map(|v| self.length(&v)).transpose()?.unwrap_or(0.0);
                Ok(Coord { p: self.st.transform.apply(Point::new(x, y)), node: None })
            }
            "xyz" => {
                let parts = vec![get("x").unwrap_or("0".into()), get("y").unwrap_or("0".into()), get("z").unwrap_or("0".into())];
                let p = self.xyz(&parts)?;
                Ok(Coord { p: self.st.transform.apply(p), node: None })
            }
            "canvas polar" | "polar" | "xyz polar" => {
                let angle = get("angle").map(|v| self.number(&v)).transpose()?.unwrap_or(0.0);
                let (rx, ry) = match (get("radius"), get("x radius"), get("y radius")) {
                    (Some(r), _, _) => {
                        let v = if system == "canvas polar" { self.length(&r)? } else { self.number(&r)? };
                        (v, v)
                    }
                    (None, x, y) => (x.map(|v| self.number(&v)).transpose()?.unwrap_or(0.0), y.map(|v| self.number(&v)).transpose()?.unwrap_or(0.0)),
                };
                let (s, c) = angle.to_radians().sin_cos();
                let local = if system == "canvas polar" { Point::new(rx * c, ry * s) } else { self.st.x.scale(rx * c).plus(self.st.y.scale(ry * s)) };
                Ok(Coord { p: self.st.transform.apply(local), node: None })
            }
            "node" => {
                let name = get("name").unwrap_or_default();
                let node = self.nodes.get(&name).ok_or_else(|| Error::Figure(format!("unknown node: {name}")))?;
                let p = match (get("anchor"), get("angle")) {
                    (Some(a), _) => node.anchor(&a).ok_or_else(|| Error::Figure(format!("unknown anchor: {name}.{a}")))?,
                    (None, Some(g)) => node.anchor(&g).unwrap_or(node.center()),
                    _ => return Ok(Coord { p: node.center(), node: Some(name) }),
                };
                Ok(Coord { p, node: None })
            }
            other => Err(Error::Figure(format!("coordinate system '{other}' not supported"))),
        }
    }

    /// `(intersection of A--B and C--D)`: where the two lines (not segments) meet.
    fn line_intersection(&mut self, spec: &str) -> Result<Point, Error> {
        let (l1, l2) = spec.split_once(" and ").ok_or_else(|| Error::Figure("intersection of: missing 'and'".into()))?;
        let ends = |e: &mut Engine, l: &str| -> Result<(Point, Point), Error> {
            let (a, b) = l.split_once("--").ok_or_else(|| Error::Figure("intersection of: a line is A--B".into()))?;
            Ok((e.coordinate(a.trim().trim_start_matches('(').trim_end_matches(')'))?.p, e.coordinate(b.trim().trim_start_matches('(').trim_end_matches(')'))?.p))
        };
        let (a, b) = ends(self, l1)?;
        let (c, d) = ends(self, l2)?;
        let r = b.minus(a);
        let s = d.minus(c);
        let den = r.x * s.y - r.y * s.x;
        if den.abs() < 1e-12 {
            return Err(Error::Figure("intersection of parallel lines".into()));
        }
        let t = ((c.x - a.x) * s.y - (c.y - a.y) * s.x) / den;
        Ok(a.plus(r.scale(t)))
    }

    /// `$(A)!0.5!(B)$`, `$(A)+(1,0)$`, `$0.5*(A)+0.5*(B)$`, `$(A)!0.5!90:(B)$`, `$(A)!(C)!(B)$`.
    pub fn calc(&mut self, text: &str) -> Result<Point, Error> {
        let c: Vec<char> = text.chars().collect();
        let mut i = 0;
        let mut total = Point::default();
        let mut sign = 1.0;
        let mut last: Option<Point> = None;
        let mut factor = 1.0;
        while i < c.len() {
            match c[i] {
                ' ' => i += 1,
                '+' | '-' => {
                    if let Some(p) = last.take() {
                        total = total.plus(p.scale(sign * factor));
                    }
                    sign = if c[i] == '+' { 1.0 } else { -1.0 };
                    factor = 1.0;
                    i += 1;
                }
                '(' => {
                    let (inside, end) = paren(&c, i).ok_or_else(|| Error::Figure("unmatched parenthesis in calc".into()))?;
                    last = Some(self.coordinate(&inside)?.p);
                    i = end;
                }
                '!' => {
                    // (A)!t!(B), (A)!t!angle:(B), (A)!(C)!(B) (projection)
                    let rest: String = c[i + 1..].iter().collect();
                    let a = last.unwrap_or_default();
                    if rest.trim_start().starts_with('(') {
                        let rc: Vec<char> = rest.chars().collect();
                        let k0 = rc.iter().position(|x| *x == '(').unwrap_or(0);
                        let (proj, pend) = paren(&rc, k0).ok_or_else(|| Error::Figure("calc: unmatched parenthesis".into()))?;
                        let after: String = rc[pend..].iter().collect();
                        let after = after.trim_start().trim_start_matches('!');
                        let ac: Vec<char> = after.chars().collect();
                        let k1 = ac.iter().position(|x| *x == '(').ok_or_else(|| Error::Figure("calc: missing the second point".into()))?;
                        let (bt, bend) = paren(&ac, k1).ok_or_else(|| Error::Figure("calc: unmatched parenthesis".into()))?;
                        let p = self.coordinate(&proj)?.p;
                        let b = self.coordinate(&bt)?.p;
                        let d = b.minus(a);
                        let t = if d.norm() > 1e-12 { (p.minus(a).x * d.x + p.minus(a).y * d.y) / (d.x * d.x + d.y * d.y) } else { 0.0 };
                        last = Some(a.lerp(b, t));
                        let consumed = rest.len() - after.len() + ac[..bend].iter().collect::<String>().len();
                        i += 1 + rest[..consumed].chars().count();
                        continue;
                    }
                    let t_end = rest.find('!').ok_or_else(|| Error::Figure("calc: unmatched '!'".into()))?;
                    let t_text = rest[..t_end].to_string();
                    let after = &rest[t_end + 1..];
                    let (angle, target_text) = match after.find('(') {
                        Some(j) if after[..j].contains(':') => (Some(self.number(after[..j].trim().trim_end_matches(':'))?), &after[j..]),
                        Some(j) => (None, &after[j..]),
                        None => return Err(Error::Figure("calc: missing the second point".into())),
                    };
                    let tc: Vec<char> = target_text.chars().collect();
                    let (inside, end) = paren(&tc, 0).ok_or_else(|| Error::Figure("calc: unmatched parenthesis".into()))?;
                    let b = self.coordinate(&inside)?.p;
                    let mut target = b;
                    if let Some(ang) = angle {
                        target = a.plus(Transform::rotation(ang).apply(b.minus(a)));
                    }
                    let (v, unit) = crate::expr::eval_length(&t_text, &self.vars())?;
                    last = Some(if unit { a.plus(target.minus(a).scale(v / target.minus(a).norm().max(1e-12))) } else { a.lerp(target, v) });
                    let consumed: String = tc[..end].iter().collect();
                    i += 1 + t_end + 1 + (after.chars().count() - target_text.chars().count()) + consumed.chars().count();
                }
                _ => {
                    // a factor: 0.5*(A), {\x}*(A)
                    let start = i;
                    let mut depth = 0;
                    while i < c.len() && (depth > 0 || (c[i] != '*' && c[i] != '(')) {
                        match c[i] {
                            '{' => depth += 1,
                            '}' => depth -= 1,
                            _ => {}
                        }
                        i += 1;
                    }
                    let f: String = c[start..i].iter().collect();
                    factor = self.number(strip_braces(f.trim())).unwrap_or(1.0);
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
}

/// `(…)` starting at `start`: the content and the index after the closing parenthesis.
fn paren(c: &[char], start: usize) -> Option<(String, usize)> {
    if c.get(start) != Some(&'(') {
        return None;
    }
    let mut depth = 0i32;
    for (k, ch) in c.iter().enumerate().skip(start) {
        match ch {
            '(' | '{' => depth += 1,
            ')' | '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((c[start + 1..k].iter().collect(), k + 1));
                }
            }
            _ => {}
        }
    }
    None
}

/// The position of ` |- ` / ` -| ` between two coordinates, outside braces.
fn find_perpendicular(t: &str, op: &str) -> Option<usize> {
    let mut depth = 0i32;
    let b = t.as_bytes();
    let o = op.as_bytes();
    for i in 0..b.len() {
        match b[i] {
            b'{' | b'(' | b'[' => depth += 1,
            b'}' | b')' | b']' => depth -= 1,
            _ => {}
        }
        if depth == 0 && i + 1 < b.len() && b[i] == o[0] && b[i + 1] == o[1] && i > 0 {
            return Some(i);
        }
    }
    None
}

/// Reads `(…)` from the token stream (the `(` already read): up to the matching `)`, braces and
/// nested parentheses respected; `($…$)` up to `$)`.
pub fn read_coordinate_tokens(e: &mut Engine) -> Result<Vec<tex::Token>, Error> {
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut braces = 0usize;
    let mut math = false;
    loop {
        let t = e.tex.get_token().ok_or_else(|| Error::Figure("unmatched parenthesis".into()))?;
        match &t {
            tex::Token::Char(_, tex::Cat::Begin) => braces += 1,
            tex::Token::Char(_, tex::Cat::End) => braces = braces.saturating_sub(1),
            tex::Token::Char('$', _) if braces == 0 => math = !math,
            tex::Token::Char('(', _) if braces == 0 => depth += 1,
            tex::Token::Char(')', _) if braces == 0 && !math => {
                if depth == 0 {
                    return Ok(out);
                }
                depth -= 1;
            }
            tex::Token::Char(')', _) if braces == 0 && math => depth = depth.saturating_sub(1),
            _ => {}
        }
        out.push(t);
        if out.len() > tex::MAX_TOKENS {
            return Err(Error::Figure("unmatched parenthesis".into()));
        }
    }
}
