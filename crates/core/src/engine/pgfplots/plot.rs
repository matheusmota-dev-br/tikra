//! `\addplot`, as `pgfplotscoordprocessing.code.tex` reads it:
//! `\addplot[+][options] [plot[behavior]] <input> <trailing path>;` with the inputs `coordinates`,
//! `table`, `expression` (`{…}`, `(x(t),y(t))`), `file`, `fill between`, and `\addplot3`.
//! The survey computes the coordinates; [`super::draw`] draws them.

use crate::tex::{self, Cat, Token};
use crate::Error;

use super::keys;
use super::table::{self, Table};
use super::{Engine, Entry};

/// A data point. Unbounded or missing values are NaN.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coord {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub meta: f64,
    /// error bars: x minus, x plus, y minus, y plus (NaN: none)
    pub err: [f64; 4],
    /// The point meta as written (`[label]`, a table cell): an index into the plot's labels.
    pub label: u32,
}

/// No label.
pub const NO_LABEL: u32 = u32::MAX;

thread_local! {
    /// The labels of the plot being read.
    static LABELS: std::cell::RefCell<Vec<Vec<Token>>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn add_label(tokens: Vec<Token>) -> u32 {
    LABELS.with(|l| {
        let mut l = l.borrow_mut();
        l.push(tokens);
        (l.len() - 1) as u32
    })
}

impl Coord {
    pub fn new(x: f64, y: f64, z: f64) -> Coord {
        Coord { x, y, z, meta: f64::NAN, err: [f64::NAN; 4], label: NO_LABEL }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Plot {
    /// The full option list it is drawn with.
    pub style: Vec<Token>,
    pub coords: Vec<Coord>,
    pub three_d: bool,
    /// What follows the input up to the `;` (`\closedcycle`, `-- cycle`, `node {…}`).
    pub post: Vec<Token>,
    pub forget: bool,
    /// `/tikz/@plot handler` (empty: lines or `smooth`)
    pub handler: String,
    /// The plot's number (`\plotnum`) and its number among plots of the same handler.
    pub index: usize,
    /// `stack plots=x|y`, with `stack dir=minus`
    pub stack: Option<char>,
    pub stack_minus: bool,
    /// For a lattice (`\addplot3` of an expression, `mesh/rows`): (rows, cols).
    pub lattice: Option<(usize, usize)>,
    /// `fill between[of=a and b]`
    pub fill_between: Option<Vec<Token>>,
    /// `x tick=data`: the first plot's x coordinates are kept.
    pub symbolic: bool,
    /// `mesh` (`'m'`) or `surf`/`patch` (`'s'`) plots, colored by the colormap.
    pub mesh: Option<char>,
    pub patch: bool,
    /// `scatter`: marks colored by their point meta.
    pub scatter: bool,
    /// The plot has numeric point meta (`point meta=y`, `explicit`…).
    pub has_meta: bool,
    /// Where the input's scanlines end (empty lines in tables and coordinates).
    pub scanlines: Vec<usize>,
    /// The point metas as written, for `point meta=explicit symbolic` (see `Coord::label`).
    pub labels: Vec<Vec<Token>>,
}

/// Letters following in the input: a word like `coordinates`.
fn word(e: &mut Engine) -> String {
    let mut w = String::new();
    loop {
        match e.tex.get_token() {
            Some(Token::Char(c, Cat::Letter)) => w.push(c),
            Some(t) => {
                e.tex.back(t);
                break;
            }
            None => break,
        }
        if w.len() > 40 {
            break;
        }
    }
    w
}

fn peek(e: &mut Engine) -> Option<Token> {
    let t = e.tex.next_nonspace();
    if let Some(t) = &t {
        e.tex.back(t.clone());
    }
    t
}

/// `\addplot…;`
pub fn addplot(e: &mut Engine) -> Result<(), Error> {
    let mut three_d = false;
    let mut plus = false;
    if let Some(t) = e.tex.get_token() {
        if t.is_char('3') {
            three_d = true;
        } else {
            e.tex.back(t);
        }
    }
    if let Some(t) = e.tex.next_nonspace() {
        if t.is_char('+') {
            plus = true;
        } else {
            e.tex.back(t);
        }
    }
    let user = e.optional()?;
    // `plot[behavior options]`
    let mut behavior = Vec::new();
    if let Some(Token::Char('p', Cat::Letter)) = peek(e) {
        let _ = e.tex.next_nonspace();
        let w = word(e);
        if w == "plot" {
            if let Some(o) = e.optional()? {
                behavior = o;
            }
        } else {
            e.tex.push_tokens(tex::string_tokens(&w));
        }
    }
    let (index, cycle) = {
        let a = e.axis.as_ref().expect("in an axis");
        (a.numplots, a.numplots)
    };
    let user_options = user.clone().unwrap_or_default();
    let mut spec = Vec::new();
    if user.is_none() || plus {
        spec.extend(cycle_entry(e, cycle)?);
        if plus && !user_options.is_empty() {
            spec.push(Token::other(','));
        }
    }
    spec.extend(user_options.clone());
    if !behavior.is_empty() {
        spec.push(Token::other(','));
        spec.extend(behavior);
    }
    let mut style = super::toks(&format!("/pgfplots/every axis plot,/pgfplots/every axis plot except legend,/pgfplots/every axis plot no {index}/.try,"));
    style.extend(spec.clone());
    style.extend(super::toks(",/pgfplots/.cd,/pgfplots/every axis plot post"));

    // the survey: the plot's keys, then its input
    e.begin_group();
    let r = survey(e, &style, three_d);
    e.end_group();
    let mut plot = r?;
    plot.style = style;
    plot.index = index;
    if !plot.forget {
        let a = e.axis.as_mut().expect("in an axis");
        a.numplots += 1;
        a.images.push(spec);
    }
    e.axis.as_mut().expect("in an axis").entries.push(Entry::Plot(Box::new(plot)));
    Ok(())
}

/// The entry of the cycle list for plot `k` (`\pgfplots@getautoplotspec`).
pub fn cycle_entry(e: &mut Engine, k: usize) -> Result<Vec<Token>, Error> {
    let name = keys::value(e, "/pgfplots/cycle list name");
    let shift = keys::number(e, "/pgfplots/cycle list shift").unwrap_or(0.0) as i64;
    let Some(list) = crate::keys::value(&e.tex, &format!("/pgfplots/@cycle lists/{name}")) else {
        e.warn(format!("cycle list '{name}' not defined"));
        return Ok(Vec::new());
    };
    let entries = split_cycle_list(&list);
    if entries.is_empty() {
        return Ok(Vec::new());
    }
    let n = entries.len() as i64;
    let i = ((k as i64 + shift) % n + n) % n;
    Ok(entries[i as usize].clone())
}

/// A cycle list: entries separated by `\\`, or by commas when there is no `\\`.
pub fn split_cycle_list(list: &[Token]) -> Vec<Vec<Token>> {
    let backslashes = list.iter().any(|t| t.is_cs("\\"));
    let mut out = Vec::new();
    let mut cur = Vec::new();
    let mut depth = 0usize;
    for t in list {
        match t {
            Token::Char(_, Cat::Begin) => depth += 1,
            Token::Char(_, Cat::End) => depth = depth.saturating_sub(1),
            _ => {}
        }
        let sep = depth == 0 && if backslashes { t.is_cs("\\") } else { t.is_char(',') };
        if sep {
            out.push(std::mem::take(&mut cur));
            continue;
        }
        cur.push(t.clone());
    }
    out.push(cur);
    out.into_iter().map(|v| tex::strip_outer_braces(crate::keys::trim(&v).to_vec())).filter(|v| !v.is_empty()).collect()
}

/// Reads the plot's input and trailing path, with its keys set (inside the plot's group).
fn survey(e: &mut Engine, style: &[Token], three_d: bool) -> Result<Plot, Error> {
    e.st.mode = Default::default();
    e.process_keys(style.to_vec(), Some("/pgfplots"))?;
    if let Some(code) = keys::value_tokens(e, "/pgfplots/execute at begin plot") {
        e.run_tokens(code)?;
    }
    let mut plot = Plot { three_d, ..Default::default() };
    LABELS.with(|l| l.borrow_mut().clear());
    let r = survey_input(e, &mut plot, three_d);
    plot.labels = LABELS.with(|l| std::mem::take(&mut *l.borrow_mut()));
    r?;
    Ok(plot)
}

fn survey_input(e: &mut Engine, plot: &mut Plot, three_d: bool) -> Result<(), Error> {
    plot.forget = keys::flag(e, "/pgfplots/forget plot");
    plot.mesh = match keys::value(e, "/pgfplots/@mesh").as_str() {
        "m" => Some('m'),
        "s" => Some('s'),
        _ => None,
    };
    plot.patch = keys::flag(e, "/pgfplots/@patch");
    plot.scatter = keys::flag(e, "/pgfplots/scatter");
    plot.handler = keys::value(e, "/tikz/@plot handler");
    plot.stack = match keys::value(e, "/pgfplots/stack plots").as_str() {
        "x" => Some('x'),
        "y" => Some('y'),
        _ => None,
    };
    plot.stack_minus = keys::value(e, "/pgfplots/stack dir") == "minus";
    let t = e.tex.next_nonspace().ok_or_else(|| Error::Figure("\\addplot without input".into()))?;
    match &t {
        Token::Char('(', _) => {
            let tokens = super::super::coords::read_coordinate_tokens(e)?;
            let parts = split_commas(&tokens);
            plot.coords = sample(e, &parts, plot)?;
        }
        Token::Char(_, Cat::Begin) => {
            let body = e.tex.read_group_body()?;
            let mut parts = vec![super::toks("x"), body];
            if three_d {
                parts = vec![super::toks("x"), super::toks("y"), parts.pop().unwrap_or_default()];
            }
            plot.coords = sample(e, &parts, plot)?;
        }
        Token::Char(c, Cat::Letter) => {
            e.tex.back(t.clone());
            let w = word(e);
            let _ = c;
            match w.as_str() {
                "coordinates" => {
                    let list = match e.tex.next_nonspace() {
                        Some(Token::Char(_, Cat::Begin)) => e.tex.read_group_body()?,
                        _ => return Err(Error::Figure("coordinates without a list".into())),
                    };
                    plot.coords = coordinates(e, list)?;
                }
                "table" => {
                    (plot.coords, plot.scanlines) = read_table(e, three_d)?;
                }
                "expression" => {
                    // `\addplot expression[domain=0:3] {…}`: options for the sampling
                    if let Some(o) = e.optional()? {
                        e.process_keys(o, Some("/pgfplots"))?;
                    }
                    let body = match e.tex.next_nonspace() {
                        Some(Token::Char(_, Cat::Begin)) => e.tex.read_group_body()?,
                        _ => return Err(Error::Figure("expression without a formula".into())),
                    };
                    let parts = if three_d { vec![super::toks("x"), super::toks("y"), body] } else { vec![super::toks("x"), body] };
                    plot.coords = sample(e, &parts, plot)?;
                }
                "file" => {
                    let _ = e.optional()?;
                    let name = e.argument_string()?;
                    let search = search_path(e);
                    match table::read_file(&name, &search) {
                        Ok(text) => {
                            let t = Table::parse(&text, table::ColSep::Space, "false", 0, "");
                            plot.coords = table_coords(e, &t, three_d)?;
                            plot.scanlines = t.scanlines.clone();
                        }
                        Err(m) => e.warn(format!("\\addplot file: {m}")),
                    }
                }
                "fill" => {
                    let between = word_after_space(e);
                    if between != "between" {
                        return Err(Error::Figure(format!("\\addplot fill {between} not supported")));
                    }
                    plot.fill_between = Some(e.optional()?.unwrap_or_default());
                }
                "gnuplot" | "shell" | "graphics" | "function" => {
                    let _ = e.optional()?;
                    let _ = e.argument()?;
                    e.warn(format!("\\addplot {w} not supported: it needs an external program or file"));
                }
                other => return Err(Error::Figure(format!("\\addplot input '{other}' not supported"))),
            }
        }
        other => return Err(Error::Figure(format!("\\addplot input '{}' not supported", tex::show_token(other)))),
    }
    // the trailing path, up to the `;`
    let mut depth = 0usize;
    while let Some(x) = e.tex.get_token() {
        match &x {
            Token::Char(_, Cat::Begin) => depth += 1,
            Token::Char(_, Cat::End) => {
                if depth == 0 {
                    e.tex.back(x);
                    break;
                }
                depth -= 1;
            }
            _ => {}
        }
        if depth == 0 && x.is_char(';') {
            break;
        }
        plot.post.push(x);
        if plot.post.len() > tex::MAX_TOKENS {
            break;
        }
    }
    point_meta(e, plot)?;
    lattice_from_scanlines(e, plot);
    filter(e, plot)?;
    if let Some(code) = keys::value_tokens(e, "/pgfplots/execute at end plot") {
        e.run_tokens(code)?;
    }
    Ok(())
}

fn word_after_space(e: &mut Engine) -> String {
    if let Some(t) = e.tex.next_nonspace() {
        e.tex.back(t);
    }
    word(e)
}

/// Splits tokens at top-level commas.
pub fn split_commas(tokens: &[Token]) -> Vec<Vec<Token>> {
    let mut out = vec![Vec::new()];
    let mut depth = 0i32;
    for t in tokens {
        match t {
            Token::Char(_, Cat::Begin) => depth += 1,
            Token::Char(_, Cat::End) => depth -= 1,
            Token::Char('(', _) => depth += 1,
            Token::Char(')', _) => depth -= 1,
            Token::Char(',', _) if depth == 0 => {
                out.push(Vec::new());
                continue;
            }
            _ => {}
        }
        if let Some(l) = out.last_mut() {
            l.push(t.clone());
        }
    }
    out.into_iter().map(|p| tex::strip_outer_braces(crate::keys::trim(&p).to_vec())).collect()
}

/// The domain and samples of an expression plot.
fn sample_points(e: &mut Engine, domain_key: &str, samples_key: &str, log: bool) -> Result<Vec<f64>, Error> {
    let at = crate::keys::value(&e.tex, "/pgfplots/samples at").filter(|v| !crate::keys::trim(v).is_empty());
    if let Some(list) = at.filter(|_| domain_key == "/pgfplots/domain") {
        let list = e.tex.expand_fully(list)?;
        let mut v = Vec::new();
        for item in super::super::foreach_items(&list) {
            v.push(e.number_of(item)?);
        }
        return Ok(v);
    }
    let d = keys::value(e, domain_key);
    let d = if d.is_empty() { keys::value(e, "/pgfplots/domain") } else { d };
    let (a, b) = if d.is_empty() {
        e.st.domain
    } else {
        let (a, b) = d.split_once(':').ok_or_else(|| Error::Figure(format!("domain '{d}' is not a:b")))?;
        (e.number(a)?, e.number(b)?)
    };
    let n = keys::number(e, samples_key).or_else(|| keys::number(e, "/pgfplots/samples")).map_or(e.st.samples, |s| s.max(1.0) as usize).clamp(1, 10_000);
    if n == 1 {
        return Ok(vec![a]);
    }
    let mut v = Vec::with_capacity(n);
    for k in 0..n {
        let t = k as f64 / (n - 1) as f64;
        v.push(if log && a > 0.0 && b > 0.0 { (a.ln() + t * (b.ln() - a.ln())).exp() } else { a + t * (b - a) });
    }
    Ok(v)
}

/// Samples `(x(var), y(var)[, z(var, y)])`.
fn sample(e: &mut Engine, parts: &[Vec<Token>], plot: &mut Plot) -> Result<Vec<Coord>, Error> {
    let var = e.st.variable.clone();
    let var_y = keys::value(e, "/pgfplots/variable y").trim_start_matches('\\').to_string();
    // the variables read as names in the expressions
    for (name, value) in [(var.as_str(), var.clone()), (var_y.as_str(), var_y.clone())] {
        e.tex.set_meaning(&Token::cs(name), super::super::macro_from(&value), false);
    }
    let texts: Vec<String> = parts.iter().map(|p| e.math_string(p.clone())).collect::<Result<_, _>>()?;
    let xlog = keys::value(e, "/pgfplots/xmode") == "log";
    let uniform_x = texts.first().is_some_and(|t| t.trim() == var);
    let xs = sample_points(e, "/pgfplots/domain", "/pgfplots/samples", xlog && uniform_x)?;
    let mut vars = e.vars();
    let mut out = Vec::new();
    let eval = |text: &str, vars: &crate::expr::Variables| crate::expr::eval(text, vars).unwrap_or(f64::NAN);
    if plot.three_d && texts.len() >= 3 {
        let ys = sample_points(e, "/pgfplots/y domain", "/pgfplots/samples y", false)?;
        let line = ys.len() < 2;
        if line {
            for &x in &xs {
                vars.insert(var.clone(), x);
                vars.insert(var_y.clone(), 0.0);
                out.push(Coord::new(eval(&texts[0], &vars), eval(&texts[1], &vars), eval(&texts[2], &vars)));
            }
        } else {
            for &y in &ys {
                for &x in &xs {
                    vars.insert(var.clone(), x);
                    vars.insert(var_y.clone(), y);
                    out.push(Coord::new(eval(&texts[0], &vars), eval(&texts[1], &vars), eval(&texts[2], &vars)));
                }
            }
            plot.lattice = Some((ys.len(), xs.len()));
        }
    } else {
        for &x in &xs {
            vars.insert(var.clone(), x);
            let cx = eval(&texts[0], &vars);
            let cy = texts.get(1).map_or(f64::NAN, |t| eval(t, &vars));
            let cz = texts.get(2).map_or(0.0, |t| eval(t, &vars));
            out.push(Coord::new(cx, cy, cz));
        }
    }
    Ok(out)
}

/// A value of `coordinates`: a number, or an expression.
fn value(e: &mut Engine, tokens: &[Token]) -> f64 {
    let s = tex::tokens_to_string(tokens);
    if let Some(v) = table::parse_number(&s) {
        return v;
    }
    let tokens = tokens.to_vec();
    e.math_string(tokens).and_then(|t| crate::expr::eval(&t, &e.vars())).unwrap_or(f64::NAN)
}

/// `coordinates {(x,y) (x,y) +- (ex,ey) [meta] …}`.
fn coordinates(e: &mut Engine, list: Vec<Token>) -> Result<Vec<Coord>, Error> {
    let list = e.tex.expand_fully(list)?;
    let mut out = Vec::new();
    let mut i = 0;
    let read_paren = |i: &mut usize| -> Option<Vec<Token>> {
        let mut depth = 0i32;
        let start = *i + 1;
        while *i < list.len() {
            match &list[*i] {
                Token::Char('(', _) => depth += 1,
                Token::Char(')', _) => {
                    depth -= 1;
                    if depth == 0 {
                        let v = list[start..*i].to_vec();
                        *i += 1;
                        return Some(v);
                    }
                }
                _ => {}
            }
            *i += 1;
        }
        None
    };
    while i < list.len() {
        let t = &list[i];
        if t.is_char('(') {
            let Some(inside) = read_paren(&mut i) else { break };
            let parts = split_commas(&inside);
            if parts.iter().all(|p| p.is_empty()) {
                // () is a jump
                out.push(Coord::new(f64::NAN, f64::NAN, f64::NAN));
                continue;
            }
            let x = parts.first().map_or(f64::NAN, |p| value(e, p));
            let y = parts.get(1).map_or(f64::NAN, |p| value(e, p));
            let z = parts.get(2).map_or(0.0, |p| value(e, p));
            let mut c = Coord::new(x, y, z);
            // error bars, then [meta]
            let mut j = i;
            while j < list.len() && list[j].is_space() {
                j += 1;
            }
            if j + 1 < list.len() && (list[j].is_char('+') || list[j].is_char('-')) && (list[j + 1].is_char('-') || list[j + 1].is_char('=')) {
                let kind = (list[j].is_char('+'), list[j + 1].is_char('-'));
                let mut k = j + 2;
                while k < list.len() && list[k].is_space() {
                    k += 1;
                }
                if k < list.len() && list[k].is_char('(') {
                    let mut kk = k;
                    if let Some(inside) = read_paren(&mut kk) {
                        let p = split_commas(&inside);
                        let ex = p.first().map_or(f64::NAN, |v| value(e, v));
                        let ey = p.get(1).map_or(f64::NAN, |v| value(e, v));
                        match kind {
                            (true, true) => c.err = [ex, ex, ey, ey],
                            (true, false) => {
                                c.err[1] = ex;
                                c.err[3] = ey;
                            }
                            _ => {
                                c.err[0] = ex;
                                c.err[2] = ey;
                            }
                        }
                        j = kk;
                    }
                }
            }
            while j < list.len() && list[j].is_space() {
                j += 1;
            }
            if j < list.len() && list[j].is_char('[') {
                let mut k = j + 1;
                let start = k;
                while k < list.len() && !list[k].is_char(']') {
                    k += 1;
                }
                let written = &list[start..k.min(list.len())];
                c.meta = value(e, written);
                c.label = add_label(crate::keys::trim(written).to_vec());
                j = k + 1;
            }
            i = j;
            out.push(c);
            continue;
        }
        i += 1;
    }
    Ok(out)
}

fn search_path(e: &mut Engine) -> Vec<String> {
    let s = keys::value(e, "/pgfplots/table/@search path");
    s.split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect()
}

/// `table[options]{file or data}`.
fn read_table(e: &mut Engine, three_d: bool) -> Result<(Vec<Coord>, Vec<usize>), Error> {
    // newlines separate rows: read the data with end of line as a character, as pgfplots does
    let saved = (e.tex.catcode('\n'), e.tex.catcode('\r'));
    e.tex.set_catcode('\n', Cat::Other);
    e.tex.set_catcode('\r', Cat::Other);
    let r = (|| -> Result<(Option<Vec<Token>>, Vec<Token>), Error> {
        let opts = e.optional()?;
        let data = match e.tex.next_nonspace() {
            Some(Token::Char(_, Cat::Begin)) => e.tex.read_group_body()?,
            Some(t) => vec![t],
            None => Vec::new(),
        };
        Ok((opts, data))
    })();
    e.tex.set_catcode('\n', saved.0);
    e.tex.set_catcode('\r', saved.1);
    let (opts, data) = r?;
    if let Some(o) = opts {
        let saved = e.st.key_path.clone();
        e.process_keys(o, Some("/pgfplots/table"))?;
        e.st.key_path = saved;
    }
    let mut text = String::new();
    for t in &data {
        match t {
            Token::Char(c, _) => text.push(*c),
            Token::Cs(n) if &**n == "\\" => text.push('\n'),
            Token::Cs(n) => {
                text.push('\\');
                text.push_str(n);
                text.push(' ');
            }
        }
    }
    let row_sep = keys::value(e, "/pgfplots/table/row sep");
    if row_sep.contains("\\\\") {
        text = text.replace("\\\\", "\n");
    }
    let sep = table::col_sep(&keys::value(e, "/pgfplots/table/col sep"));
    let header = keys::value(e, "/pgfplots/table/header");
    let skip = keys::number(e, "/pgfplots/table/skip first n").unwrap_or(0.0) as usize;
    let comment = keys::value(e, "/pgfplots/table/comment chars");
    let trimmed = text.trim();
    let is_file = !trimmed.contains('\n') && !trimmed.contains(' ') && !trimmed.is_empty() && trimmed.chars().any(|c| c.is_alphabetic());
    let table = if is_file {
        match table::read_file(trimmed, &search_path(e)) {
            Ok(content) => Table::parse(&content, sep, &header, skip, &comment),
            Err(m) => {
                e.warn(format!("\\addplot table: {m}"));
                return Ok((Vec::new(), Vec::new()));
            }
        }
    } else if trimmed.starts_with('\\') {
        e.warn(format!("\\addplot table {trimmed}: tables read with \\pgfplotstableread are not supported"));
        return Ok((Vec::new(), Vec::new()));
    } else {
        Table::parse(&text, sep, &header, skip, &comment)
    };
    Ok((table_coords(e, &table, three_d)?, table.scanlines.clone()))
}

/// The coordinates of a table: the `x`, `y`, `z` columns (by name, index or expression).
fn table_coords(e: &mut Engine, t: &Table, three_d: bool) -> Result<Vec<Coord>, Error> {
    #[derive(Clone)]
    enum Src {
        Col(Option<usize>),
        Expr(Vec<Token>),
        None,
    }
    let pick = |e: &mut Engine, axis: &str, default: Option<usize>| -> Src {
        if let Some(expr) = keys::value_tokens(e, &format!("/pgfplots/table/{axis} expr")) {
            return Src::Expr(expr);
        }
        let name = keys::value(e, &format!("/pgfplots/table/{axis}"));
        if !name.is_empty() {
            return Src::Col(t.column(&name));
        }
        if let Some(i) = keys::number(e, &format!("/pgfplots/table/{axis} index")) {
            return Src::Col(Some(i as usize));
        }
        match default {
            Some(d) => Src::Col(Some(d)),
            None => Src::None,
        }
    };
    let sx = pick(e, "x", Some(0));
    let sy = pick(e, "y", Some(1));
    let sz = pick(e, "z", if three_d { Some(2) } else { None });
    let smeta = match crate::keys::value(&e.tex, "/pgfplots/point meta") {
        Some(v) if tex::tokens_to_string(&v).contains("\\thisrow") => Src::Expr(v),
        _ => pick(e, "meta", None),
    };
    let sey = pick(e, "y error", None);
    let sex = pick(e, "x error", None);
    let seyp = pick(e, "y error plus", None);
    let seym = pick(e, "y error minus", None);
    let mut out = Vec::with_capacity(t.rows.len());
    for r in 0..t.rows.len() {
        let get = |e: &mut Engine, s: &Src| -> f64 {
            match s {
                Src::Col(Some(c)) => t.cell(r, *c).and_then(table::parse_number).unwrap_or(f64::NAN),
                Src::Col(None) | Src::None => f64::NAN,
                Src::Expr(tokens) => {
                    let replaced = replace_thisrow(tokens, t, r);
                    e.math_string(replaced).and_then(|s| crate::expr::eval(&s, &e.vars())).unwrap_or(f64::NAN)
                }
            }
        };
        let x = get(e, &sx);
        let y = get(e, &sy);
        let z = if matches!(sz, Src::None) { 0.0 } else { get(e, &sz) };
        let mut c = Coord::new(x, y, z);
        c.meta = get(e, &smeta);
        if let Src::Col(Some(col)) = &smeta {
            if let Some(cell) = t.cell(r, *col) {
                c.label = add_label(tex::string_tokens(cell));
            }
        }
        let ey = get(e, &sey);
        let ex = get(e, &sex);
        c.err = [ex, ex, ey, ey];
        let (p, m) = (get(e, &seyp), get(e, &seym));
        if p.is_finite() {
            c.err[3] = p;
        }
        if m.is_finite() {
            c.err[2] = m;
        }
        out.push(c);
    }
    Ok(out)
}

/// `\thisrow{name}`, `\thisrowno{i}`, `\coordindex`, `\lineno` replaced by the row's values.
fn replace_thisrow(tokens: &[Token], t: &Table, row: usize) -> Vec<Token> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let tk = &tokens[i];
        let (is_row, is_no) = (tk.is_cs("thisrow"), tk.is_cs("thisrowno"));
        if is_row || is_no {
            let mut j = i + 1;
            while j < tokens.len() && tokens[j].is_space() {
                j += 1;
            }
            let (arg, end) = if j < tokens.len() && matches!(tokens[j], Token::Char(_, Cat::Begin)) {
                let mut depth = 0;
                let mut k = j;
                loop {
                    match &tokens[k] {
                        Token::Char(_, Cat::Begin) => depth += 1,
                        Token::Char(_, Cat::End) => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    k += 1;
                    if k >= tokens.len() {
                        break;
                    }
                }
                (tex::tokens_to_string(&tokens[j + 1..k.min(tokens.len())]), k + 1)
            } else {
                (tokens.get(j).map(|t| tex::tokens_to_string(std::slice::from_ref(t))).unwrap_or_default(), j + 1)
            };
            let col = if is_no { arg.trim().parse().ok() } else { t.column(arg.trim()) };
            let v = col.and_then(|c| t.cell(row, c)).and_then(table::parse_number).unwrap_or(f64::NAN);
            out.extend(tex::string_tokens(&format!("({})", if v.is_nan() { "nan".to_string() } else { format!("{v}") })));
            i = end;
            continue;
        }
        if tk.is_cs("coordindex") || tk.is_cs("lineno") {
            out.extend(tex::string_tokens(&row.to_string()));
            i += 1;
            continue;
        }
        out.push(tk.clone());
        i += 1;
    }
    out
}

/// `restrict x to domain`, `restrict y to domain`, `unbounded coords`.
fn filter(e: &mut Engine, plot: &mut Plot) -> Result<(), Error> {
    for (axis, key) in [('x', "/pgfplots/restrict x to domain"), ('y', "/pgfplots/restrict y to domain")] {
        let d = keys::value(e, key);
        if let Some((a, b)) = d.split_once(':') {
            let (a, b) = (e.number(a)?, e.number(b)?);
            for c in &mut plot.coords {
                let v = if axis == 'x' { &mut c.x } else { &mut c.y };
                if *v < a || *v > b {
                    *v = f64::NAN;
                }
            }
        }
    }
    let jump = keys::value(e, "/pgfplots/unbounded coords") == "jump";
    let log = (keys::value(e, "/pgfplots/xmode") == "log", keys::value(e, "/pgfplots/ymode") == "log");
    for c in &mut plot.coords {
        if (log.0 && c.x <= 0.0) || (log.1 && c.y <= 0.0) {
            c.x = f64::NAN;
        }
        if !(c.x.is_finite() && c.y.is_finite() && (!plot.three_d || c.z.is_finite())) {
            c.x = f64::NAN;
        }
    }
    if !jump && plot.lattice.is_none() {
        plot.coords.retain(|c| c.x.is_finite());
    }
    Ok(())
}

/// `point meta`: the value each point maps to the colormap with.
fn point_meta(e: &mut Engine, plot: &mut Plot) -> Result<(), Error> {
    let raw = crate::keys::value(&e.tex, "/pgfplots/point meta").unwrap_or_default();
    let source = tex::tokens_to_string(crate::keys::trim(&raw)).trim().to_string();
    match source.as_str() {
        "" | "none" => {}
        "x" => plot.coords.iter_mut().for_each(|c| c.meta = c.x),
        "y" => plot.coords.iter_mut().for_each(|c| c.meta = c.y),
        "z" => plot.coords.iter_mut().for_each(|c| c.meta = c.z),
        "f(x)" => {
            let three = plot.three_d;
            plot.coords.iter_mut().for_each(|c| c.meta = if three { c.z } else { c.y })
        }
        "explicit" | "explicit symbolic" => {}
        _ => {
            // \thisrow{…} is evaluated with the table's rows
            if !source.contains("\\thisrow") {
                let text = e.math_string(raw.clone())?;
                let mut vars = e.vars();
                for c in plot.coords.iter_mut() {
                    vars.insert("x".into(), c.x);
                    vars.insert("y".into(), c.y);
                    vars.insert("z".into(), c.z);
                    c.meta = crate::expr::eval(&text, &vars).unwrap_or(f64::NAN);
                }
            }
        }
    }
    plot.has_meta = !matches!(source.as_str(), "" | "none" | "explicit symbolic");
    Ok(())
}

/// A lattice from `mesh/rows`, `mesh/cols` or the input's scanlines.
fn lattice_from_scanlines(e: &mut Engine, plot: &mut Plot) {
    if plot.lattice.is_some() || plot.mesh.is_none() {
        return;
    }
    let n = plot.coords.len();
    let rows = keys::number(e, "/pgfplots/mesh/rows").map(|v| v as usize);
    let cols = keys::number(e, "/pgfplots/mesh/cols").map(|v| v as usize);
    let lattice = match (rows, cols) {
        (_, Some(c)) if c > 0 => Some((n / c, c)),
        (Some(r), _) if r > 0 => Some((r, n / r)),
        _ => plot.scanlines.first().filter(|c| **c > 0 && n.is_multiple_of(**c)).map(|c| (n / c, *c)),
    };
    if let Some((r, c)) = lattice.filter(|(r, c)| r * c == n && *r > 1 && *c > 1) {
        plot.lattice = Some((r, c));
    }
}
