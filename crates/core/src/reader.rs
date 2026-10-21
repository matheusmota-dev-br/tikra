//! Interpreter for the supported TikZ subset (spec/subset.md):
//! `\draw`, `\fill`, `\filldraw`, `\path`, `\node`, `\coordinate`, `\foreach`, `scope`, `\def`,
//! `\pgfmathsetmacro`, `\tikzset`, `\definecolor`, positioning (`below=of A`), `calc`, `plot`.

use std::collections::HashMap;

use crate::color::{self, Color};
use crate::expr::{self, Variables};
use crate::options::{self, Opt, Styles};
use crate::scene::*;
use crate::{path, Error, PT};

/// State that spans the whole figure.
pub struct Env {
    pub vars: Variables,
    pub macros: HashMap<String, String>,
    pub styles: Styles,
    pub colors: HashMap<String, Color>,
    pub nodes: HashMap<String, Node>,
    pub scopes: Vec<Scope>,
    pub scene: Scene,
    pub warnings: Vec<String>,
    /// What `>` means in `->` (`>=Latex`).
    pub default_tip: Tip,
    /// `node distance`: vertical and horizontal, in cm.
    pub node_distance: (f64, f64),
    /// Statements run so far (every `\foreach` iteration counts).
    pub steps: usize,
    /// Current nesting of `scope`, `\foreach` and environments.
    pub depth: usize,
}

/// Cap on statements per figure: nested `\foreach` loops do not hang the caller.
pub const MAX_STEPS: usize = 100_000;
/// Cap on nesting of `scope`, `\foreach` and environments.
pub const MAX_DEPTH: usize = 64;
/// Cap on text length after macro expansion (`\def\a{\a\a}` would double on each pass).
pub const MAX_EXPANSION: usize = 100_000;

/// One `scope` level (or the `tikzpicture` itself).
#[derive(Clone)]
pub struct Scope {
    pub transform: Transform,
    /// Options inherited by paths and nodes.
    pub options: Vec<Opt>,
    /// Unit vectors (`x=4cm`, `y=0.7cm`).
    pub ux: Point,
    pub uy: Point,
    pub clips: usize,
}

impl Default for Scope {
    fn default() -> Self {
        Scope { transform: Transform::default(), options: Vec::new(), ux: Point::new(1.0, 0.0), uy: Point::new(0.0, 1.0), clips: 0 }
    }
}

impl Env {
    pub fn new() -> Env {
        Env {
            vars: Variables::new(),
            macros: HashMap::new(),
            styles: Styles::default(),
            colors: HashMap::new(),
            nodes: HashMap::new(),
            scopes: vec![Scope::default()],
            scene: Scene::default(),
            warnings: Vec::new(),
            default_tip: Tip::Default,
            node_distance: (1.0, 1.0),
            steps: 0,
            depth: 0,
        }
    }

    pub fn scope(&self) -> &Scope {
        self.scopes.last().expect("there is always a scope")
    }

    pub fn warn(&mut self, s: impl Into<String>) {
        let s = s.into();
        if !self.warnings.contains(&s) {
            self.warnings.push(s);
        }
    }

    /// Replaces user macros (`\R`, `\angle`, `\foreach` variables) with their text.
    pub fn expand(&self, text: &str) -> String {
        let mut s = text.to_string();
        for _ in 0..20 {
            let mut changed = false;
            let c: Vec<char> = s.chars().collect();
            let mut out = String::with_capacity(s.len());
            let mut i = 0;
            while i < c.len() {
                if c[i] == '\\' {
                    let name: String = c[i + 1..].iter().take_while(|x| x.is_ascii_alphabetic()).collect();
                    if let Some(v) = self.macros.get(&name) {
                        out.push_str(v);
                        i += 1 + name.chars().count();
                        // \R cm: the space that ends the macro name goes away
                        changed = true;
                        continue;
                    }
                }
                out.push(c[i]);
                i += 1;
            }
            s = out;
            if !changed || s.len() > MAX_EXPANSION {
                break;
            }
        }
        s
    }

    pub fn length(&self, text: &str) -> Result<f64, Error> {
        Ok(expr::eval_length(&self.expand(text), &self.vars)?.0)
    }

    pub fn number(&self, text: &str) -> Result<f64, Error> {
        expr::eval(&self.expand(text), &self.vars)
    }

    pub fn color(&self, text: &str) -> Option<Color> {
        color::parse(&self.expand(text), &self.colors)
    }
}

impl Default for Env {
    fn default() -> Self {
        Env::new()
    }
}

/// Reads text at statement level.
pub struct Reader<'a> {
    c: Vec<char>,
    pos: usize,
    pub env: &'a mut Env,
}

/// Content of `{…}`, `[…]` or `(…)` starting at the opening delimiter.
pub fn block(c: &[char], start: usize) -> Option<(String, usize)> {
    let open = *c.get(start)?;
    let close = match open {
        '{' => '}',
        '[' => ']',
        '(' => ')',
        _ => return None,
    };
    let mut level = 0;
    let mut i = start;
    while i < c.len() {
        let ch = c[i];
        if ch == '\\' {
            i += 2;
            continue;
        }
        if ch == open {
            level += 1;
        } else if ch == close {
            level -= 1;
            if level == 0 {
                return Some((c[start + 1..i].iter().collect(), i + 1));
            }
        } else if open != '{' && ch == '{' {
            // inside () and [] braces nest too: (\x, {sin(\x)})
            let (_, end) = block(c, i)?;
            i = end;
            continue;
        }
        i += 1;
    }
    None
}

/// Strips `%` comments to the end of the line (keeps `\%`).
pub fn strip_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        let c: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < c.len() {
            if c[i] == '\\' && i + 1 < c.len() {
                out.push(c[i]);
                out.push(c[i + 1]);
                i += 2;
                continue;
            }
            if c[i] == '%' {
                break;
            }
            out.push(c[i]);
            i += 1;
        }
        out.push('\n');
    }
    out
}

impl<'a> Reader<'a> {
    pub fn new(text: &str, env: &'a mut Env) -> Reader<'a> {
        Reader { c: text.chars().collect(), pos: 0, env }
    }

    fn skip_spaces(&mut self) {
        while self.c.get(self.pos).is_some_and(|c| c.is_whitespace()) {
            self.pos += 1;
        }
    }

    fn at(&self, s: &str) -> bool {
        s.chars().enumerate().all(|(k, ch)| self.c.get(self.pos + k) == Some(&ch))
    }

    fn command_name(&self) -> Option<String> {
        if self.c.get(self.pos) != Some(&'\\') {
            return None;
        }
        Some(self.c[self.pos + 1..].iter().take_while(|x| x.is_ascii_alphabetic()).collect())
    }

    /// Reads up to the top-level `;` (consumed) and returns the text.
    fn statement(&mut self) -> String {
        let mut level = 0i32;
        let start = self.pos;
        while self.pos < self.c.len() {
            let ch = self.c[self.pos];
            match ch {
                '\\' => {
                    self.pos += 2;
                    continue;
                }
                '{' | '[' | '(' => level += 1,
                '}' | ']' | ')' => level -= 1,
                ';' if level <= 0 => {
                    let s: String = self.c[start..self.pos].iter().collect();
                    self.pos += 1;
                    return s;
                }
                _ => {}
            }
            self.pos += 1;
        }
        self.c[start..].iter().collect()
    }

    fn argument(&mut self) -> Option<String> {
        self.skip_spaces();
        let (s, end) = block(&self.c, self.pos)?;
        self.pos = end;
        Some(s)
    }

    fn optional(&mut self) -> Option<String> {
        self.skip_spaces();
        if self.c.get(self.pos) != Some(&'[') {
            return None;
        }
        self.argument()
    }

    /// Runs statements until the end of the text or until `\end{end}`.
    pub fn run(&mut self, end: Option<&str>) -> Result<(), Error> {
        if self.env.depth >= MAX_DEPTH {
            return Err(Error::Figure(format!("scope/\\foreach nesting went past {MAX_DEPTH} levels")));
        }
        self.env.depth += 1;
        let r = self.run_level(end);
        self.env.depth -= 1;
        r
    }

    fn run_level(&mut self, end: Option<&str>) -> Result<(), Error> {
        loop {
            self.skip_spaces();
            if self.pos >= self.c.len() {
                return Ok(());
            }
            self.env.steps += 1;
            if self.env.steps > MAX_STEPS {
                return Err(Error::Figure(format!("figure too large: more than {MAX_STEPS} statements")));
            }
            if self.at("\\end{") {
                let start = self.pos;
                self.pos += 4;
                let Some(name) = self.argument() else {
                    // unclosed \end{: skip the "\end{" and go on
                    self.pos = start + 5;
                    self.env.warn("\\end{ without '}': ignored");
                    continue;
                };
                if Some(name.trim()) == end || end.is_none() {
                    return Ok(());
                }
                self.env.warn(format!("\\end{{{}}} without a matching \\begin: ignored", name.trim()));
                continue;
            }
            let Some(cmd) = self.command_name() else {
                // loose text between statements: skip to the end of the line
                let rest: String = self.c[self.pos..].iter().take_while(|c| **c != '\n').collect();
                if !rest.trim().is_empty() && rest.trim() != ";" {
                    self.env.warn(format!("text outside a statement ignored: {}", rest.trim().chars().take(40).collect::<String>()));
                }
                while self.pos < self.c.len() && self.c[self.pos] != '\n' {
                    self.pos += 1;
                }
                continue;
            };
            self.pos += 1 + cmd.chars().count();
            match cmd.as_str() {
                "begin" => {
                    let name = self.argument().unwrap_or_default();
                    match name.trim() {
                        "scope" => {
                            let o = self.optional().unwrap_or_default();
                            self.push_scope(&o)?;
                            self.run(Some("scope"))?;
                            self.pop_scope();
                        }
                        "axis" | "semilogyaxis" | "semilogxaxis" | "loglogaxis" => {
                            let o = self.optional().unwrap_or_default();
                            let body = self.until_end(name.trim());
                            crate::axes::plot_axis(self.env, &o, &body)?;
                        }
                        other => {
                            self.env.warn(format!("environment {other} not supported: content read as statements"));
                            self.run(Some(other))?;
                        }
                    }
                }
                "foreach" => self.foreach()?,
                "def" | "gdef" | "edef" | "xdef" | "newcommand" | "renewcommand" | "providecommand" => {
                    self.skip_spaces();
                    let name = if self.c.get(self.pos) == Some(&'{') {
                        self.argument().unwrap_or_default().trim().trim_start_matches('\\').to_string()
                    } else {
                        // \def at the end of the text does not go past the end
                        self.pos = (self.pos + 1).min(self.c.len());
                        let n: String = self.c[self.pos..].iter().take_while(|x| x.is_ascii_alphabetic()).collect();
                        self.pos += n.chars().count();
                        n
                    };
                    // \newcommand{\a}[1]{…}: arguments are not supported in figures; the body is used without them
                    while self.optional().is_some() {}
                    let value = self.argument().unwrap_or_default();
                    self.env.macros.insert(name, value);
                }
                "pgfmathsetmacro" | "pgfmathsetlengthmacro" | "pgfmathparse" => {
                    let name = self.argument().unwrap_or_default().trim().trim_start_matches('\\').to_string();
                    if cmd == "pgfmathparse" {
                        continue;
                    }
                    let e = self.argument().unwrap_or_default();
                    match self.env.length(&e) {
                        Ok(v) if v.is_finite() => {
                            self.env.macros.insert(name, format_number(v));
                        }
                        Ok(v) => self.env.warn(format!("\\pgfmathsetmacro{{\\{name}}}: non-finite result ({v}), macro not defined")),
                        Err(err) => self.env.warn(format!("\\pgfmathsetmacro{{\\{name}}}: {err}")),
                    }
                }
                "tikzset" => {
                    let o = self.argument().unwrap_or_default();
                    let rest = self.env.styles.register(&o);
                    self.apply_globals(&rest);
                }
                "tikzstyle" => {
                    let name = self.argument().unwrap_or_default();
                    self.skip_spaces();
                    if self.c.get(self.pos) == Some(&'=') {
                        self.pos += 1;
                    }
                    let o = self.optional().unwrap_or_default();
                    self.env.styles.defined.insert(name.trim().to_string(), o);
                }
                "definecolor" | "providecolor" => {
                    let name = self.argument().unwrap_or_default();
                    let model = self.argument().unwrap_or_default();
                    let values = self.argument().unwrap_or_default();
                    match color::define(&model, &values) {
                        Some(c) => {
                            self.env.colors.insert(name.trim().to_string(), c);
                        }
                        None => self.env.warn(format!("\\definecolor{{{name}}}{{{model}}}: model not supported")),
                    }
                }
                "colorlet" => {
                    let name = self.argument().unwrap_or_default();
                    let e = self.argument().unwrap_or_default();
                    if let Some(c) = self.env.color(&e) {
                        self.env.colors.insert(name.trim().to_string(), c);
                    }
                }
                "usetikzlibrary" | "usepackage" | "pgfplotsset" | "usepgfplotslibrary" => {
                    self.optional();
                    self.argument();
                }
                "draw" | "fill" | "filldraw" | "path" | "clip" | "shade" | "shadedraw" | "pattern" | "node" | "coordinate" | "matrix" | "pic" => {
                    let body = self.statement();
                    let body = self.env.expand(&body);
                    let text = match cmd.as_str() {
                        "node" => format!("node{body}"),
                        "coordinate" => format!("coordinate{body}"),
                        _ => body,
                    };
                    if matches!(cmd.as_str(), "matrix" | "pic") {
                        self.env.warn(format!("\\{cmd} is not supported: ignored"));
                        continue;
                    }
                    let base = if matches!(cmd.as_str(), "node" | "coordinate") { "path" } else { cmd.as_str() };
                    if let Err(e) = path::statement(self.env, base, &text) {
                        self.env.warn(format!("\\{cmd}: {e}; statement ignored"));
                    }
                }
                "small" | "footnotesize" | "large" | "Large" | "scriptsize" | "normalsize" | "centering" | "sffamily" | "bfseries" => {}
                other => {
                    let rest = self.statement();
                    self.env.warn(format!("\\{other} not supported: '{}' ignored", rest.chars().take(40).collect::<String>().trim()));
                }
            }
        }
    }

    fn until_end(&mut self, name: &str) -> String {
        let end = format!("\\end{{{name}}}");
        let rest: String = self.c[self.pos..].iter().collect();
        match rest.find(&end) {
            Some(i) => {
                self.pos += rest[..i + end.len()].chars().count();
                rest[..i].to_string()
            }
            None => {
                self.pos = self.c.len();
                rest
            }
        }
    }

    /// Figure options that apply everywhere (`>=Latex`, `node distance`, an already registered `every node/.style`).
    pub fn apply_globals(&mut self, o: &[Opt]) {
        for op in o {
            match (op.key.as_str(), op.value.as_deref()) {
                (">", Some(v)) => self.env.default_tip = path::tip(v).unwrap_or(Tip::Default),
                ("node distance", Some(v)) => {
                    let parts: Vec<&str> = v.split(" and ").collect();
                    let a = self.env.length(parts[0]).unwrap_or(1.0);
                    let b = parts.get(1).and_then(|p| self.env.length(p).ok()).unwrap_or(a);
                    self.env.node_distance = (a, b);
                }
                ("declare function", Some(v)) => {
                    for def in v.split(';') {
                        let Some((head, body)) = def.split_once('=') else { continue };
                        let head = head.trim();
                        let Some((name, params)) = head.split_once('(') else { continue };
                        let params = params.trim_end_matches(')').split(',').map(|p| p.trim().trim_start_matches('\\').to_string()).collect();
                        self.env.vars.functions.insert(name.trim().to_string(), (params, body.trim().to_string()));
                    }
                }
                _ => {}
            }
        }
    }

    /// `\begin{scope}[rotate=30, shift={(A)}, thick]` or the `tikzpicture` options.
    pub fn push_scope(&mut self, text: &str) -> Result<(), Error> {
        let rest = self.env.styles.register(text);
        let rest = self.env.styles.expand(rest);
        self.apply_globals(&rest);
        let mut new = self.env.scope().clone();
        new.clips = 0;
        let mut local = Transform::default();
        for o in &rest {
            let v = o.value.as_deref();
            match (o.key.as_str(), v) {
                ("scale", Some(v)) => {
                    let s = self.env.number(v)?;
                    local = local.compose(&Transform::scaling(s, s));
                }
                ("xscale", Some(v)) => local = local.compose(&Transform::scaling(self.env.number(v)?, 1.0)),
                ("yscale", Some(v)) => local = local.compose(&Transform::scaling(1.0, self.env.number(v)?)),
                ("rotate", Some(v)) => local = local.compose(&Transform::rotation(self.env.number(v)?)),
                ("xshift", Some(v)) => local = local.compose(&Transform::translation(self.env.length(v)?, 0.0)),
                ("yshift", Some(v)) => local = local.compose(&Transform::translation(0.0, self.env.length(v)?)),
                ("shift", Some(v)) => {
                    let t = options::strip_braces(v).trim().trim_start_matches('(').trim_end_matches(')').to_string();
                    let p = path::absolute_coordinate(self.env, &t)?;
                    // shift={(A)} moves to point A (already in canvas coordinates)
                    let target = self.env.scope().transform.compose(&local);
                    let inv = invert(&target);
                    let q = inv.apply(p);
                    local = local.compose(&Transform::translation(q.x, q.y));
                }
                ("x", Some(v)) => new.ux = unit_vector(self.env, v, true)?,
                ("y", Some(v)) => new.uy = unit_vector(self.env, v, false)?,
                _ => new.options.push(o.clone()),
            }
        }
        new.transform = new.transform.compose(&local);
        self.env.scopes.push(new);
        Ok(())
    }

    pub fn pop_scope(&mut self) {
        // the stack is never left empty (the figure scope is put back below)
        let e = self.env.scopes.pop().unwrap_or_default();
        for _ in 0..e.clips {
            self.env.scene.elements.push(Element::EndClip);
        }
        if self.env.scopes.is_empty() {
            self.env.scopes.push(Scope::default());
        }
    }

    /// `\foreach \x/\y in {1/a, 2/b} {…}`, `\foreach \x in {0,0.5,...,2}`, `[count=\i]`.
    fn foreach(&mut self) -> Result<(), Error> {
        self.skip_spaces();
        let mut vars_text = String::new();
        while self.pos < self.c.len() && !self.at(" in ") && !self.at("in{") && !self.at(" in{") && self.c[self.pos] != '[' {
            vars_text.push(self.c[self.pos]);
            self.pos += 1;
        }
        let foreach_options = self.optional().unwrap_or_default();
        self.skip_spaces();
        if self.at("in") {
            self.pos += 2;
        }
        let list = self.argument().unwrap_or_default();
        self.skip_spaces();
        let body = if self.c.get(self.pos) == Some(&'{') {
            self.argument().unwrap_or_default()
        } else {
            let s = self.statement();
            format!("{s};")
        };
        let names: Vec<String> = vars_text.split('/').map(|v| v.trim().trim_start_matches('\\').to_string()).filter(|v| !v.is_empty()).collect();
        let counter = options::parse(&foreach_options)
            .into_iter()
            .find(|o| o.key == "count")
            .and_then(|o| o.value)
            .map(|v| v.trim().trim_start_matches('\\').split_whitespace().next().unwrap_or("").to_string());
        let values = expand_list(&self.env.expand(&list));
        let saved = self.env.macros.clone();
        let saved_numbers = self.env.vars.values.clone();
        for (i, item) in values.iter().enumerate() {
            let parts: Vec<&str> = item.split('/').collect();
            for (k, name) in names.iter().enumerate() {
                let v = parts.get(k).or(parts.last()).copied().unwrap_or("").trim();
                self.env.macros.insert(name.clone(), v.to_string());
                if let Ok(n) = expr::eval(v, &self.env.vars) {
                    self.env.vars.insert(name.clone(), n);
                }
            }
            if let Some(c) = &counter {
                self.env.macros.insert(c.clone(), (i + 1).to_string());
            }
            let mut sub = Reader::new(&body, self.env);
            sub.run(None)?;
        }
        // loop variables do not leak
        self.env.macros = saved;
        self.env.vars.values = saved_numbers;
        Ok(())
    }
}

/// `{0,1,...,5}`, `{0,0.5,...,2}`, `{a,b,c}`, `{1/a, 2/b}`.
pub fn expand_list(list: &str) -> Vec<String> {
    let items: Vec<String> = options::split_top_level(list);
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < items.len() {
        if items[i] == "..." && !out.is_empty() && i + 1 < items.len() {
            let end: f64 = items[i + 1].parse().unwrap_or(0.0);
            let n = out.len();
            let last: f64 = out[n - 1].parse().unwrap_or(0.0);
            let step = if n >= 2 { last - out[n - 2].parse::<f64>().unwrap_or(last - 1.0) } else { 1.0 };
            if step.abs() > 1e-12 {
                let mut v = last + step;
                let mut guard = 0;
                while (step > 0.0 && v <= end + 1e-9) || (step < 0.0 && v >= end - 1e-9) {
                    out.push(format_number(v));
                    v += step;
                    guard += 1;
                    if guard > 10_000 {
                        break;
                    }
                }
            }
            i += 2;
            continue;
        }
        out.push(items[i].clone());
        i += 1;
    }
    out
}

pub fn format_number(v: f64) -> String {
    let r = (v * 1e6).round() / 1e6;
    // integer only while it fits exactly in an i64; above that f64's Display already has no exponent
    if r.fract() == 0.0 && r.abs() < 1e15 {
        format!("{}", r as i64)
    } else {
        format!("{r}")
    }
}

fn unit_vector(env: &Env, v: &str, x_axis: bool) -> Result<Point, Error> {
    let t = v.trim();
    if t.starts_with('(') || t.starts_with('{') {
        let inner = options::strip_braces(t).trim().trim_start_matches('(').trim_end_matches(')').to_string();
        return path::absolute_coordinate(env, &inner);
    }
    let m = env.length(t)?;
    Ok(if x_axis { Point::new(m, 0.0) } else { Point::new(0.0, m) })
}

pub fn invert(t: &Transform) -> Transform {
    let det = t.a * t.d - t.b * t.c;
    if det.abs() < 1e-15 {
        return Transform::default();
    }
    let (a, b, c, d) = (t.d / det, -t.b / det, -t.c / det, t.a / det);
    Transform { a, b, c, d, e: -(a * t.e + c * t.f), f: -(b * t.e + d * t.f) }
}

/// TikZ line widths, in cm.
pub fn line_width(name: &str) -> Option<f64> {
    Some(
        match name {
            "ultra thin" => 0.1,
            "very thin" => 0.2,
            "thin" => 0.4,
            "semithick" => 0.6,
            "thick" => 0.8,
            "very thick" => 1.2,
            "ultra thick" => 1.6,
            _ => return None,
        } * PT,
    )
}
