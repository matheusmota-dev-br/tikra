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
    /// The sampling variable's value (expressions), and `quiver`'s (u, v, w).
    pub param: [f64; 2],
    pub uvw: [f64; 3],
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
        Coord { x, y, z, meta: f64::NAN, err: [f64::NAN; 4], label: NO_LABEL, param: [f64::NAN; 2], uvw: [f64::NAN; 3] }
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
    /// `bar shift` (or `bar shift auto`) given to the axis or the plot: stacked bars keep it.
    pub bar_shift_given: bool,
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
    /// `mesh input=image` (`matrix plot`): the points are the centers of the cells.
    pub image: bool,
    /// `late options`: axis options the plot asks for (`matrix plot`: `y dir=reverse`).
    pub late: Vec<Token>,
    /// `boxplot`, `boxplot prepared`: the box's numbers (the points are the outliers).
    pub boxplot: Option<Box<super::boxplot::BoxStats>>,
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
    let k = (k as i64 + shift).max(0) as usize;
    // `cycle multi list={a\nextlist b}`: one entry of each, the last list fastest
    if list.iter().any(|t| t.is_cs("nextlist")) {
        let lists: Vec<Vec<Vec<Token>>> = list.split(|t| t.is_cs("nextlist")).map(|part| multi_part(e, part)).filter(|l| !l.is_empty()).collect();
        let mut out = Vec::new();
        let mut rest = k;
        let mut chosen = Vec::new();
        if name.starts_with("@multiindex") {
            // `cycle multiindex list`: the same index into every list, the shorter ones padded
            // with nothing (restarted with `cycle multiindex* list`)
            let longest = lists.iter().map(Vec::len).max().unwrap_or(1);
            for l in &lists {
                let i = if name == "@multiindex*@" { k % l.len() } else { k % longest };
                chosen.push(l.get(i).cloned().unwrap_or_default());
            }
            chosen.reverse();
        } else {
            for l in lists.iter().rev() {
                chosen.push(l[rest % l.len()].clone());
                rest /= l.len();
            }
        }
        for entry in chosen.into_iter().rev().filter(|c| !c.is_empty()) {
            if !out.is_empty() {
                out.push(Token::other(','));
            }
            out.extend(entry);
        }
        return Ok(out);
    }
    let entries = split_cycle_list(&list);
    if entries.is_empty() {
        return Ok(Vec::new());
    }
    Ok(entries[k % entries.len()].clone())
}

/// A list of `cycle multi list`: a cycle list's name (`[2 of]mark list`: its first two entries)
/// or the entries themselves.
fn multi_part(e: &mut Engine, part: &[Token]) -> Vec<Vec<Token>> {
    let text = tex::tokens_to_string(crate::keys::trim(part)).trim().to_string();
    let (limit, name) = match text.strip_prefix('[').and_then(|r| r.split_once(" of]")) {
        Some((n, rest)) => (n.trim().parse::<usize>().ok(), rest.trim().to_string()),
        None => (None, text.clone()),
    };
    if let Some(list) = crate::keys::value(&e.tex, &format!("/pgfplots/@cycle lists/{name}")) {
        let mut entries = split_cycle_list(&list);
        if let Some(n) = limit {
            entries.truncate(n.max(1));
        }
        return entries;
    }
    split_cycle_list(part)
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
    plot.image = plot.mesh.is_some() && keys::value(e, "/pgfplots/mesh input") == "image";
    plot.late = keys::value_tokens(e, "/pgfplots/late options").unwrap_or_default();
    plot.scatter = keys::flag(e, "/pgfplots/scatter");
    plot.handler = keys::value(e, "/tikz/@plot handler");
    let axis_options = e.axis.as_ref().map(|a| tex::tokens_to_string(&a.options)).unwrap_or_default();
    plot.bar_shift_given = axis_options.contains("bar shift");
    // a prepared contour: its level is z in 3d, the third column (point meta=explicit) in 2d
    if plot.handler == "contour prepared" && keys::value(e, "/pgfplots/point meta").is_empty() {
        if three_d {
            keys::set(e, "/pgfplots/point meta", "z");
        } else if keys::value(e, "/pgfplots/@contour format") != "matlab" {
            keys::set(e, "/pgfplots/table/meta index", "2");
            keys::set(e, "/pgfplots/point meta", "explicit");
        }
    }
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
                    (plot.coords, plot.scanlines) = coordinates(e, list)?;
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
    data_cs(e, plot);
    // `is smithchart cs`: the input is in the unit circle's plane
    if e.axis.as_ref().is_some_and(|a| a.kind == "smithchart") && keys::flag(e, "/pgfplots/is smithchart cs") {
        for c in plot.coords.iter_mut() {
            (c.x, c.y) = super::smith::impedance(c.x, c.y);
        }
    }
    if plot.handler == "contour prepared" && keys::value(e, "/pgfplots/@contour format") == "matlab" {
        super::contour::from_matlab(plot);
        keys::set(e, "/pgfplots/point meta", "explicit");
    }
    if keys::flag(e, "/pgfplots/@quiver") {
        quiver_vectors(e, plot);
    }
    if keys::flag(e, "/pgfplots/@hist") {
        histogram(e, plot)?;
    }
    match keys::value(e, "/pgfplots/@boxplot").as_str() {
        "raw" => super::boxplot::survey(e, plot, false),
        "prepared" => super::boxplot::survey(e, plot, true),
        _ => {}
    }
    point_meta(e, plot)?;
    if plot.patch {
        patch_table(e, plot);
    }
    lattice_from_scanlines(e, plot);
    filter(e, plot)?;
    if let Some(code) = keys::value_tokens(e, "/pgfplots/execute at end plot") {
        e.run_tokens(code)?;
    }
    Ok(())
}

/// `patch table={0 1 2\\ 1 2 3\\}` (or a file name): the patches are rows of indices into the
/// points read; `patch table with point meta` adds a color to each row, `… with individual point
/// meta` one to each of its corners. The points become the patches' corners, one after another,
/// each with its color.
fn patch_table(e: &mut Engine, plot: &mut Plot) {
    let mut found = None;
    for (key, kind) in [("patch table", 0), ("patch table with point meta", 1), ("patch table with individual point meta", 2)] {
        let v = keys::value(e, &format!("/pgfplots/{key}"));
        if !v.trim().is_empty() {
            found = Some((v, kind));
        }
    }
    let Some((text, kind)) = found else { return };
    let text = text.replace("\\\\", "\n");
    let trimmed = text.trim();
    let text = if !trimmed.contains('\n') && !trimmed.contains(' ') && trimmed.chars().any(|c| c.is_alphabetic()) {
        match table::read_file(trimmed, &search_path(e)) {
            Ok(t) => t,
            Err(m) => {
                e.warn(format!("patch table: {m}"));
                return;
            }
        }
    } else {
        text
    };
    let t = Table::parse(&text, table::ColSep::Space, "false", 0, "");
    let mut coords = Vec::new();
    for row in &t.rows {
        let v: Vec<f64> = row.iter().filter_map(|c| table::parse_number(c)).collect();
        let n = match kind {
            0 => v.len(),
            1 => v.len().saturating_sub(1),
            _ => v.len() / 2,
        };
        for (i, idx) in v[..n].iter().enumerate() {
            let Some(c) = plot.coords.get(*idx as usize) else { continue };
            let mut c = *c;
            match kind {
                1 => c.meta = v[n],
                2 => c.meta = v.get(n + i).copied().unwrap_or(f64::NAN),
                _ => {}
            }
            coords.push(c);
        }
    }
    plot.coords = coords;
    plot.scanlines.clear();
    if kind > 0 {
        plot.has_meta = true;
    }
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

/// Where the points of one patch of a type lie in the unit interval or square (`sample in unit
/// cube` of the patch classes), in their order; None for a type without them.
fn patch_unit_points(kind: &str) -> Option<Vec<(f64, f64)>> {
    let (t1, t2) = (1.0 / 3.0, 2.0 / 3.0);
    Some(match kind.trim() {
        "line" => vec![(0.0, 0.0), (1.0, 0.0)],
        "quadratic spline" => vec![(0.0, 0.0), (1.0, 0.0), (0.5, 0.0)],
        "cubic spline" => vec![(0.0, 0.0), (1.0, 0.0), (t1, 0.0), (t2, 0.0)],
        "rectangle" | "bilinear" => vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
        "triangle quadr" => vec![(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (0.5, 0.0), (0.5, 0.5), (0.0, 0.5), (0.0, 1.0), (1.0, 0.0), (1.0, 1.0), (0.5, 0.5), (1.0, 0.5), (0.5, 1.0)],
        "biquadratic" => vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0), (0.5, 0.0), (1.0, 0.5), (0.5, 1.0), (0.0, 0.5), (0.5, 0.5)],
        "bicubic" => (0..16).map(|k| ([0.0, t1, t2, 1.0][k % 4], [0.0, t1, t2, 1.0][k / 4])).collect(),
        _ => return None,
    })
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
    // `patch type sampling`: each interval (or cell) gives the points of one patch, at the places
    // of the patch type's points in the unit interval (square)
    let unit = if keys::flag(e, "/pgfplots/patch type sampling") { patch_unit_points(&keys::value(e, "/pgfplots/patch type")) } else { None };
    if let Some(unit) = unit {
        let at = |v: &[f64], i: usize, t: f64| v[i] + t * (v[i + 1] - v[i]);
        let two = plot.three_d && texts.len() >= 3 && unit.iter().any(|(_, v)| *v != 0.0);
        let ys = if two { sample_points(e, "/pgfplots/y domain", "/pgfplots/samples y", false)? } else { vec![0.0, 0.0] };
        for j in 0..ys.len().saturating_sub(1) {
            for i in 0..xs.len().saturating_sub(1) {
                for &(u, v) in &unit {
                    let (x, y) = (at(&xs, i, u), at(&ys, j, v));
                    vars.insert(var.clone(), x);
                    vars.insert(var_y.clone(), y);
                    let mut c = Coord::new(eval(&texts[0], &vars), texts.get(1).map_or(f64::NAN, |t| eval(t, &vars)), texts.get(2).map_or(0.0, |t| eval(t, &vars)));
                    c.param = [x, y];
                    out.push(c);
                }
            }
        }
        plot.patch = plot.mesh.is_some();
        return Ok(out);
    }
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
                    let mut c = Coord::new(eval(&texts[0], &vars), eval(&texts[1], &vars), eval(&texts[2], &vars));
                    c.param = [x, y];
                    out.push(c);
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
            let mut c = Coord::new(cx, cy, cz);
            c.param = [x, f64::NAN];
            out.push(c);
        }
    }
    Ok(out)
}

/// A value of `coordinates`: a number, or an expression.
/// A coordinate of axis `c`: a name of `symbolic c coords`, or a number.
fn axis_value(e: &mut Engine, c: char, tokens: &[Token]) -> f64 {
    if let Some(i) = super::symbolic::index(e, c, &tex::tokens_to_string(tokens)) {
        return i;
    }
    value(e, tokens)
}

fn value(e: &mut Engine, tokens: &[Token]) -> f64 {
    let s = tex::tokens_to_string(tokens);
    if let Some(v) = table::parse_number(&s) {
        return v;
    }
    let tokens = tokens.to_vec();
    e.math_string(tokens).and_then(|t| crate::expr::eval(&t, &e.vars())).unwrap_or(f64::NAN)
}

/// `coordinates {(x,y) (x,y) +- (ex,ey) [meta] …}`.
/// The points of `coordinates {…}`, and where its scanlines (ended by empty lines) end.
fn coordinates(e: &mut Engine, list: Vec<Token>) -> Result<(Vec<Coord>, Vec<usize>), Error> {
    let explicit_symbolic = keys::value(e, "/pgfplots/point meta").trim() == "explicit symbolic";
    let list = e.tex.expand_fully(list)?;
    let mut out = Vec::new();
    let mut scanlines: Vec<usize> = Vec::new();
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
        if t.is_cs("par") {
            if !out.is_empty() && scanlines.last() != Some(&out.len()) {
                scanlines.push(out.len());
            }
            i += 1;
            continue;
        }
        if t.is_char('(') {
            let Some(inside) = read_paren(&mut i) else { break };
            let parts = split_commas(&inside);
            if parts.iter().all(|p| p.is_empty()) {
                // () is a jump
                out.push(Coord::new(f64::NAN, f64::NAN, f64::NAN));
                continue;
            }
            let x = parts.first().map_or(f64::NAN, |p| axis_value(e, 'x', p));
            let y = parts.get(1).map_or(f64::NAN, |p| axis_value(e, 'y', p));
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
                // `explicit symbolic`: the text as written, never a number
                if !explicit_symbolic {
                    c.meta = value(e, written);
                }
                c.label = add_label(crate::keys::trim(written).to_vec());
                j = k + 1;
            }
            i = j;
            out.push(c);
            continue;
        }
        i += 1;
    }
    Ok((out, scanlines))
}

fn search_path(e: &mut Engine) -> Vec<String> {
    let s = keys::value(e, "/pgfplots/table/@search path");
    s.split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect()
}

thread_local! {
    /// Tables `\pgfplotstableread` keeps, by macro name: their text and their options.
    static READ_TABLES: std::cell::RefCell<std::collections::HashMap<String, (String, Vec<Token>)>> = std::cell::RefCell::new(std::collections::HashMap::new());
}

/// `[options]{file or data}` with line ends as characters (rows), and the data as text.
fn table_source(e: &mut Engine) -> Result<(Option<Vec<Token>>, String), Error> {
    let saved = (e.tex.catcode('\n'), e.tex.catcode('\r'));
    e.tex.set_catcode('\n', Cat::Other);
    e.tex.set_catcode('\r', Cat::Other);
    let r = (|| -> Result<(Option<Vec<Token>>, Vec<Token>), Error> {
        // line ends before the options or the data are spaces still
        let next = |e: &mut Engine| loop {
            match e.tex.next_nonspace() {
                Some(Token::Char('\n' | '\r', Cat::Other)) => continue,
                t => break t,
            }
        };
        if let Some(t) = next(e) {
            e.tex.back(t);
        }
        let opts = e.optional()?;
        let data = match next(e) {
            Some(Token::Char(_, Cat::Begin)) => e.tex.read_group_body()?,
            Some(t) => vec![t],
            None => Vec::new(),
        };
        Ok((opts, data))
    })();
    e.tex.set_catcode('\n', saved.0);
    e.tex.set_catcode('\r', saved.1);
    let (opts, data) = r?;
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
    Ok((opts, text))
}

/// `\pgfplotstableread[options]{file or data}\macro`: the table, kept for `table {\macro}`.
pub fn table_read(e: &mut Engine) -> Result<(), Error> {
    let (opts, text) = table_source(e)?;
    let name = match e.tex.next_nonspace() {
        Some(Token::Cs(n)) => n.to_string(),
        Some(Token::Char(_, Cat::Begin)) => {
            let body = e.tex.read_group_body()?;
            match body.iter().find(|t| matches!(t, Token::Cs(_))) {
                Some(Token::Cs(n)) => n.to_string(),
                _ => String::new(),
            }
        }
        _ => String::new(),
    };
    if name.is_empty() {
        e.warn("\\pgfplotstableread without a macro to keep the table: ignored");
        return Ok(());
    }
    READ_TABLES.with(|t| t.borrow_mut().insert(name, (text, opts.unwrap_or_default())));
    Ok(())
}

/// `table[options]{file or data}`.
fn read_table(e: &mut Engine, three_d: bool) -> Result<(Vec<Coord>, Vec<usize>), Error> {
    let (opts, mut text) = table_source(e)?;
    // a table `\pgfplotstableread` kept: its options, then the plot's
    let kept = text.trim().strip_prefix('\\').map(|n| n.trim().to_string()).and_then(|n| READ_TABLES.with(|t| t.borrow().get(&n).cloned()));
    let mut all_opts = Vec::new();
    if let Some((t, o)) = kept {
        text = t;
        all_opts.extend(o);
        all_opts.extend(super::toks(","));
    }
    all_opts.extend(opts.unwrap_or_default());
    if !all_opts.is_empty() {
        let saved = e.st.key_path.clone();
        e.process_keys(all_opts, Some("/pgfplots/table"))?;
        e.st.key_path = saved;
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
    // a file the document wrote (`\pgfplotscolormaptodatafile`)
    let written = format!("tikra@file@{trimmed}");
    let table = if is_file && e.tex.is_defined(&written) {
        let content = e.tex.expand_to_string(vec![Token::cs(&written)])?;
        Table::parse(&content, sep, &header, skip, &comment)
    } else if is_file {
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
    let symbolic = symbolic_meta(e);
    let quiver = keys::flag(e, "/pgfplots/@quiver");
    let t_table = t;
    let smeta = match crate::keys::value(&e.tex, "/pgfplots/point meta") {
        _ if symbolic.is_some() => Src::None,
        Some(v) if tex::tokens_to_string(&v).contains("\\thisrow") => Src::Expr(v),
        _ => pick(e, "meta", None),
    };
    // a tieline plot reads the tie's other end of each row too: `tie end x` (or its index),
    // else the column three after x's
    let tie: Option<[Src; 3]> = (keys::value(e, "/tikz/@plot handler") == "tieline").then(|| {
        let mut ends = [Src::None, Src::None, Src::None];
        for (k, c) in ["x", "y", "z"].iter().enumerate() {
            let own = [&sx, &sy, &sz][k];
            let name = keys::value(e, &format!("/pgfplots/table/tie end {c}"));
            ends[k] = if !name.is_empty() {
                Src::Col(t.column(&name))
            } else if let Some(i) = keys::number(e, &format!("/pgfplots/table/tie end {c} index")) {
                Src::Col(Some(i as usize))
            } else {
                match own {
                    Src::Col(Some(i)) if i + 3 < t.columns() => Src::Col(Some(i + 3)),
                    _ => Src::None,
                }
            };
        }
        ends
    });
    if tie.as_ref().is_some_and(|t| t.iter().any(|s| matches!(s, Src::None | Src::Col(None)))) {
        e.warn("tieline: the tie ends' columns are not found (give 'tie end x' and its friends): ties not drawn");
    }
    let sey = pick(e, "y error", None);
    let sex = pick(e, "x error", None);
    let seyp = pick(e, "y error plus", None);
    let seym = pick(e, "y error minus", None);
    let box_filter = match e.tex.meaning_of(&crate::keys::macro_name("/pgfplots/boxplot/data filter/.@cmd")) {
        tex::Meaning::Macro(m) if !m.body.is_empty() && !keys::value(e, "/pgfplots/@boxplot").is_empty() => Some(m.body.clone()),
        _ => None,
    };
    let mut out = Vec::with_capacity(t.rows.len());
    for r in 0..t.rows.len() {
        // an expression may use the coordinates read so far: `z expr=x^2-y^2`
        let known: std::cell::RefCell<Vec<(&str, f64)>> = Default::default();
        let get = |e: &mut Engine, s: &Src| -> f64 {
            match s {
                Src::Col(Some(c)) => t.cell(r, *c).and_then(table::parse_number).unwrap_or(f64::NAN),
                Src::Col(None) | Src::None => f64::NAN,
                Src::Expr(tokens) => {
                    let replaced = replace_thisrow(tokens, t, r);
                    let mut vars = e.vars();
                    for (n, v) in known.borrow().iter() {
                        vars.insert((*n).into(), *v);
                    }
                    e.math_string(replaced).and_then(|s| crate::expr::eval(&s, &vars)).unwrap_or(f64::NAN)
                }
            }
        };
        // a cell of a symbolic axis is a name
        let sym = |e: &mut Engine, s: &Src, c: char, v: f64| match s {
            Src::Col(Some(col)) if !v.is_finite() => t.cell(r, *col).and_then(|cell| super::symbolic::index(e, c, cell)).unwrap_or(v),
            _ => v,
        };
        let x = get(e, &sx);
        let x = sym(e, &sx, 'x', x);
        known.borrow_mut().push(("x", x));
        let y = get(e, &sy);
        let y = sym(e, &sy, 'y', y);
        known.borrow_mut().push(("y", y));
        let z = if matches!(sz, Src::None) { 0.0 } else { get(e, &sz) };
        let mut c = Coord::new(x, y, z);
        c.meta = get(e, &smeta);
        if let Src::Col(Some(col)) = &smeta {
            if let Some(cell) = t.cell(r, *col) {
                c.label = add_label(tex::string_tokens(cell));
            }
        }
        if quiver {
            for (k, name) in ["u", "v", "w"].iter().enumerate() {
                if let Some(t) = keys::value_tokens(e, &format!("/pgfplots/quiver/{name}")) {
                    let replaced = replace_thisrow(&t, t_table, r);
                    let mut vars = e.vars();
                    vars.insert("x".into(), x);
                    vars.insert("y".into(), y);
                    vars.insert("z".into(), z);
                    c.uvw[k] = e.math_string(replaced).and_then(|s| crate::expr::eval(&s, &vars)).unwrap_or(f64::NAN);
                }
            }
        }
        if let Some((tokens, mathparse)) = &symbolic {
            let replaced = replace_thisrow(tokens, t, r);
            let text = e.tex.expand_to_string(replaced).unwrap_or_default();
            let l = symbolic_label(e, &text, *mathparse, [x, y, z]);
            c.label = add_label(tex::string_tokens(&l));
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
        let other = tie.as_ref().map(|ends| {
            let mut b = c;
            b.x = get(e, &ends[0]);
            b.y = get(e, &ends[1]);
            b.z = get(e, &ends[2]);
            b
        });
        // `boxplot/data filter/.code`: the row's value in `\pgfmathresult` (empty: left out), with
        // `\thisrow` the row's cells
        if let Some(body) = &box_filter {
            let data = keys::value(e, "/pgfplots/boxplot/data");
            let v = match data.as_str() {
                "x" => &mut c.x,
                "z" => &mut c.z,
                _ => &mut c.y,
            };
            if v.is_finite() {
                let text = super::super::pgf_number(*v);
                let code = replace_thisrow_as(&tex::substitute(body, &[tex::string_tokens(&text)]), t, r, true);
                e.begin_group();
                e.tex.define_string("pgfmathresult", &text, false);
                let res = e.run_tokens(code).and_then(|_| e.tex.expand_to_string(vec![Token::cs("pgfmathresult")]));
                e.end_group();
                *v = table::parse_number(&res?).unwrap_or(f64::NAN);
            }
        }
        out.push(c);
        out.extend(other);
    }
    Ok(out)
}

/// `\thisrow{name}`, `\thisrowno{i}`, `\coordindex`, `\lineno` replaced by the row's values.
fn replace_thisrow(tokens: &[Token], t: &Table, row: usize) -> Vec<Token> {
    replace_thisrow_as(tokens, t, row, false)
}

/// The same, the cells as their text (`raw`) or as numbers in parentheses.
fn replace_thisrow_as(tokens: &[Token], t: &Table, row: usize, raw: bool) -> Vec<Token> {
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
            if raw {
                let cell = col.and_then(|c| t.cell(row, c)).unwrap_or("");
                out.extend(cell.chars().map(|c| Token::Char(c, if c.is_alphabetic() { Cat::Letter } else { Cat::Other })));
                i = end;
                continue;
            }
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
    // `x filter/.code={…}`: the code changes `\pgfmathresult`, the coordinate (empty: none)
    for a in 0..3 {
        let key = format!("/pgfplots/{} filter", ["x", "y", "z"][a]);
        match e.tex.meaning_of(&crate::keys::macro_name(&format!("{key}/.@cmd"))) {
            tex::Meaning::Macro(m) if !m.body.is_empty() => {}
            _ => continue,
        }
        for (i, c) in plot.coords.iter_mut().enumerate() {
            let v = match a {
                0 => &mut c.x,
                1 => &mut c.y,
                _ => &mut c.z,
            };
            if !v.is_finite() {
                continue;
            }
            e.begin_group();
            let text = super::super::pgf_number(*v);
            e.tex.define_string("pgfmathresult", &text, false);
            e.tex.define_string("coordindex", &i.to_string(), false);
            let r = e.run_key(&key, Some(tex::string_tokens(&text))).and_then(|_| e.tex.expand_to_string(vec![Token::cs("pgfmathresult")]));
            e.end_group();
            *v = table::parse_number(&r?).unwrap_or(f64::NAN);
        }
    }
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

/// An expression with the parentheses still open at its end closed: pgfmath reads
/// `(sqrt((1)^2 + (2*x)^2)` (the manual's quiver example) as if they were.
fn balanced(text: &str) -> String {
    let open = text.chars().filter(|c| *c == '(').count();
    let close = text.chars().filter(|c| *c == ')').count();
    format!("{text}{}", ")".repeat(open.saturating_sub(close)))
}

/// `quiver`: each point's (u, v, w), from `quiver/u` and its friends (expressions of x, y, z and
/// the sampling variable), times `scale arrows`.
fn quiver_vectors(e: &mut Engine, plot: &mut Plot) {
    let var = e.st.variable.clone();
    let var_y = keys::value(e, "/pgfplots/variable y").trim_start_matches('\\').to_string();
    let texts: Vec<Option<String>> = ["u", "v", "w"].iter().map(|n| keys::value_tokens(e, &format!("/pgfplots/quiver/{n}")).and_then(|t| e.math_string(t).ok())).collect();
    let scale = keys::number(e, "/pgfplots/quiver/scale arrows").unwrap_or(1.0);
    let mut vars = e.vars();
    for c in plot.coords.iter_mut() {
        vars.insert("x".into(), c.x);
        vars.insert("y".into(), c.y);
        vars.insert("z".into(), c.z);
        if c.param[0].is_finite() {
            vars.insert(var.clone(), c.param[0]);
        }
        if c.param[1].is_finite() && !var_y.is_empty() {
            vars.insert(var_y.clone(), c.param[1]);
        }
        for (u, text) in c.uvw.iter_mut().zip(&texts) {
            if !u.is_finite() {
                *u = match text {
                    Some(t) if !t.trim().is_empty() => crate::expr::eval(&balanced(t), &vars).unwrap_or(f64::NAN),
                    _ => 0.0,
                };
            }
            *u *= scale;
        }
    }
}

/// `data cs`: the input's coordinate system (`cart`, `polar` in degrees, `polarrad`) turned into
/// the axis' (`polar` for a polar axis).
fn data_cs(e: &mut Engine, plot: &mut Plot) {
    let source = keys::value(e, "/pgfplots/data cs");
    let target = keys::value(e, "/pgfplots/@axis type");
    if source == target || source.is_empty() {
        return;
    }
    for c in plot.coords.iter_mut() {
        if source == "polarrad" {
            c.x = c.x.to_degrees();
        }
        let (x, y) = (c.x, c.y);
        match (source.as_str(), target.as_str()) {
            ("polar" | "polarrad", "cart") => {
                let (s, co) = super::polar::sin_cos(x);
                (c.x, c.y) = (y * co, y * s);
            }
            ("cart", "polar") => {
                (c.x, c.y) = (y.atan2(x).to_degrees().rem_euclid(360.0), x.hypot(y));
            }
            _ => {}
        }
    }
}

/// `hist`: the points become the bins' counts, `(start of bin k, count k)`, and a last point at
/// the end of the data range (`\pgfplotsplothandlersurveyend@hist@`).
fn histogram(e: &mut Engine, plot: &mut Plot) -> Result<(), Error> {
    let data = keys::value(e, "/pgfplots/hist/data");
    let value_expr = keys::value(e, "/pgfplots/hist/data value");
    let mut vars = e.vars();
    let mut values = Vec::new();
    for c in &plot.coords {
        let v = if !value_expr.is_empty() {
            vars.insert("x".into(), c.x);
            vars.insert("y".into(), c.y);
            vars.insert("z".into(), c.z);
            crate::expr::eval(&value_expr, &vars).unwrap_or(f64::NAN)
        } else {
            match data.as_str() {
                "x" => c.x,
                "z" => c.z,
                "y" | "" => c.y,
                other => {
                    vars.insert("x".into(), c.x);
                    vars.insert("y".into(), c.y);
                    vars.insert("z".into(), c.z);
                    crate::expr::eval(other, &vars).unwrap_or(f64::NAN)
                }
            }
        };
        if v.is_finite() {
            values.push(v);
        }
    }
    let bins = keys::number(e, "/pgfplots/hist/bins").unwrap_or(10.0).max(1.0) as usize;
    // `data min` defaults to `xmin`
    let limit = |e: &mut Engine, k: &str, axis: &str| keys::number(e, &format!("/pgfplots/hist/{k}")).or_else(|| keys::number(e, &format!("/pgfplots/{axis}")));
    let lo = limit(e, "data min", "xmin").unwrap_or_else(|| values.iter().copied().fold(f64::INFINITY, f64::min));
    let hi = limit(e, "data max", "xmax").unwrap_or_else(|| values.iter().copied().fold(f64::NEG_INFINITY, f64::max));
    if values.is_empty() || !(lo.is_finite() && hi.is_finite()) {
        plot.coords.clear();
        return Ok(());
    }
    let h = (hi - lo) / bins as f64;
    let mut counts = vec![0.0f64; bins];
    for v in &values {
        let k = if h != 0.0 { ((v - lo) / h + 1e-4).trunc() } else { 0.0 };
        let k = if k < 0.0 { 0 } else { (k as usize).min(bins - 1) };
        counts[k] += 1.0;
    }
    let total: f64 = counts.iter().sum();
    let cumulative = keys::flag(e, "/pgfplots/hist/cumulative");
    let density = keys::flag(e, "/pgfplots/hist/density");
    if cumulative {
        let mut acc = 0.0;
        for c in counts.iter_mut() {
            acc += *c;
            *c = if density { acc / total } else { acc };
        }
    } else if density && total > 0.0 && h != 0.0 {
        for c in counts.iter_mut() {
            *c /= total * h;
        }
    }
    let mut coords: Vec<Coord> = counts.iter().enumerate().map(|(k, c)| Coord::new(lo + h * k as f64, *c, 0.0)).collect();
    if keys::flag(e, "/pgfplots/hist/intervals") {
        coords.push(Coord::new(hi, counts[bins - 1], 0.0));
    }
    plot.coords = coords;
    plot.three_d = false;
    plot.scanlines.clear();
    plot.handler = keys::value(e, "/tikz/@plot handler");
    Ok(())
}

/// `point meta={symbolic={R,G,B}}`: the text of each point's meta, and whether
/// `mesh/color input=explicit mathparse` evaluates it.
fn symbolic_meta(e: &mut Engine) -> Option<(Vec<Token>, bool)> {
    let raw = crate::keys::value(&e.tex, "/pgfplots/point meta")?;
    let items = crate::keys::split_list(crate::keys::trim(&raw));
    let item = items.first().filter(|i| i.key == "symbolic" && items.len() == 1)?;
    let mathparse = keys::value(e, "/pgfplots/mesh/color input") == "explicit mathparse";
    Some((item.value.clone().unwrap_or_default(), mathparse))
}

/// A symbolic point meta for a point: with `mathparse`, each part (`R,G,B`, `wave=…`) evaluated.
fn symbolic_label(e: &mut Engine, text: &str, mathparse: bool, xyz: [f64; 3]) -> String {
    if !mathparse {
        return text.trim().to_string();
    }
    let mut vars = e.vars();
    vars.insert("x".into(), xyz[0]);
    vars.insert("y".into(), xyz[1]);
    vars.insert("z".into(), xyz[2]);
    let (model, rest) = match text.split_once('=') {
        Some((m, r)) if m.trim().chars().all(|c| c.is_ascii_alphanumeric()) => (format!("{}=", m.trim()), r),
        _ => (String::new(), text),
    };
    let parts: Vec<String> =
        crate::keys::split_top_level(rest).iter().map(|p| crate::expr::eval(p.trim(), &vars).map(|v| format!("{v}")).unwrap_or_else(|_| p.trim().to_string())).collect();
    format!("{model}{}", parts.join(","))
}

/// `point meta`: the value each point maps to the colormap with.
fn point_meta(e: &mut Engine, plot: &mut Plot) -> Result<(), Error> {
    if let Some((tokens, mathparse)) = symbolic_meta(e) {
        // tables have set them row by row
        if plot.coords.iter().all(|c| c.label == NO_LABEL) {
            let text = e.tex.expand_to_string(tokens).unwrap_or_default();
            for i in 0..plot.coords.len() {
                let c = plot.coords[i];
                let l = symbolic_label(e, &text, mathparse, [c.x, c.y, c.z]);
                plot.coords[i].label = add_label(tex::string_tokens(&l));
            }
        }
        plot.has_meta = false;
        return Ok(());
    }
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
                // `\coordindex` is the point's number
                let per_point = source.contains("\\coordindex");
                let mut text = if per_point { String::new() } else { e.math_string(raw.clone())? };
                let mut vars = e.vars();
                for (i, c) in plot.coords.iter_mut().enumerate() {
                    if per_point {
                        e.tex.define_string("coordindex", &i.to_string(), false);
                        text = e.math_string(raw.clone())?;
                    }
                    vars.insert("x".into(), c.x);
                    vars.insert("y".into(), c.y);
                    vars.insert("z".into(), c.z);
                    c.meta = crate::expr::eval(&balanced(&text), &vars).unwrap_or(f64::NAN);
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
        // patches are their own (`mesh input=patches`)
        _ if plot.patch => None,
        _ => plot.scanlines.first().filter(|c| **c > 0 && n.is_multiple_of(**c)).map(|c| (n / c, *c)).or_else(|| {
            // without scanlines a matrix is square (`matrix delinearize uses sqrt`)
            let s = (n as f64).sqrt().round() as usize;
            (plot.image && s * s == n && s > 0).then_some((s, s))
        }),
    };
    if let Some((r, c)) = lattice.filter(|(r, c)| r * c == n && *r > 1 && *c > 1) {
        plot.lattice = Some((r, c));
    }
}
