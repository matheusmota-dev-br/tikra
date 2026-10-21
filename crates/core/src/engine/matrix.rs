//! Matrices: `\matrix[options] (m) at (point) { cell & cell \\ cell & cell \\ };`, as pgf's matrix
//! module lays them out. Each cell is drawn once alone to measure how far it reaches left, right,
//! above and below its origin; columns and rows line the origins up with `column sep` and
//! `row sep` between the cells' borders; the matrix is then a node (drawn, named, anchored) and the
//! cells are drawn again in place. `matrix of nodes` makes each cell a node named `m-r-c`, with
//! `|[options]|` for that cell's options.

use crate::canvas::Canvas;
use crate::scene::{Point, Transform};
use crate::tex::{self, Cat, Token};
use crate::Error;

use super::nodes::{NodeSpec, OnPath};
use super::{tikra_tokens, Engine};

/// A cell's extent from its origin: left, right, above, below (all ≥ 0 when it has ink).
#[derive(Debug, Clone, Copy, Default)]
struct Extent {
    left: f64,
    right: f64,
    up: f64,
    down: f64,
    empty: bool,
}

/// The body split into rows of cells; the extra space a `\\[1ex]` asks for after each row.
fn split_body(body: &[Token]) -> (Vec<Vec<Vec<Token>>>, Vec<Vec<Token>>) {
    let mut rows = vec![vec![Vec::new()]];
    let mut extra = vec![Vec::new()];
    let mut depth = 0usize;
    let mut i = 0;
    while i < body.len() {
        let t = &body[i];
        match t {
            Token::Char(_, Cat::Begin) => depth += 1,
            Token::Char(_, Cat::End) => depth = depth.saturating_sub(1),
            Token::Char(_, Cat::Align) if depth == 0 => {
                rows.last_mut().expect("a row").push(Vec::new());
                i += 1;
                continue;
            }
            Token::Cs(n) if depth == 0 && (&**n == "\\" || &**n == "pgfmatrixendrow") => {
                // \\[dist]
                let mut j = i + 1;
                while j < body.len() && body[j].is_space() {
                    j += 1;
                }
                let mut dist = Vec::new();
                if body.get(j).is_some_and(|x| x.is_char('[')) {
                    let mut k = j + 1;
                    while k < body.len() && !body[k].is_char(']') {
                        dist.push(body[k].clone());
                        k += 1;
                    }
                    i = k + 1;
                } else {
                    i += 1;
                }
                *extra.last_mut().expect("a row") = dist;
                rows.push(vec![Vec::new()]);
                extra.push(Vec::new());
                continue;
            }
            _ => {}
        }
        rows.last_mut().expect("a row").last_mut().expect("a cell").push(t.clone());
        i += 1;
    }
    // a last row with only empty cells is the end of the body
    while rows.last().is_some_and(|r| r.iter().all(|c| crate::keys::trim(c).is_empty())) && rows.len() > 1 {
        rows.pop();
        extra.pop();
    }
    (rows, extra)
}

impl Engine {
    /// `\matrix` (the command already read) and `node[matrix]`.
    pub fn matrix(&mut self) -> Result<(), Error> {
        // options, (name), at (point), then the body in braces
        let mut options = Vec::new();
        let mut name = None;
        let mut at = None;
        loop {
            match self.tex.next_nonspace() {
                Some(t) if t.is_char('[') => {
                    if !options.is_empty() {
                        options.push(Token::other(','));
                    }
                    options.extend(self.tex.read_until_char(']', false)?);
                }
                Some(t) if t.is_char('(') => {
                    let n = super::coords::read_coordinate_tokens(self)?;
                    name = Some(self.tex.expand_to_string(n)?.trim().to_string());
                }
                Some(Token::Char('a', Cat::Letter)) => {
                    let _ = self.tex.get_token();
                    match self.tex.next_nonspace() {
                        Some(t) if t.is_char('(') => {
                            let c = super::coords::read_coordinate_tokens(self)?;
                            let text = self.math_string(c)?;
                            at = Some(self.coordinate(&text)?.p);
                        }
                        _ => return Err(Error::Figure("\\matrix: at without a point".into())),
                    }
                }
                Some(Token::Char(_, Cat::Begin)) => break,
                Some(t) => return Err(Error::Figure(format!("\\matrix: unexpected {}", tex::show_token(&t)))),
                None => return Err(Error::Figure("\\matrix without a body".into())),
            }
        }
        let body = self.tex.read_group_body()?;
        // the statement ends with ;
        if let Some(t) = self.tex.next_nonspace() {
            if !t.is_char(';') {
                self.tex.back(t);
            }
        }
        self.begin_group();
        let r = self.matrix_in_group(options, name, at, body);
        self.end_group();
        r
    }

    fn matrix_in_group(&mut self, options: Vec<Token>, name: Option<String>, at: Option<Point>, body: Vec<Token>) -> Result<(), Error> {
        // the matrix's own options are read once to know the cells' settings; the node reads them again
        self.st.matrix = Default::default();
        let saved_mode = self.st.mode.clone();
        self.process_keys(tikra_tokens("every matrix/.try"), Some("/tikz"))?;
        self.process_keys(options.clone(), Some("/tikz"))?;
        self.st.mode = saved_mode;
        let settings = self.st.matrix.clone();
        let name = name.or_else(|| self.st.node.name.clone());
        let at = at.or(self.st.node.at).unwrap_or_else(|| self.st.transform.apply(Point::default()));
        let (rows, extra) = split_body(&body);
        let nrows = rows.len();
        let ncols = rows.iter().map(Vec::len).max().unwrap_or(0);

        // first pass: measure every cell alone
        let mut extents = vec![vec![Extent { empty: true, ..Default::default() }; ncols]; nrows];
        for (r, row) in rows.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                let saved = std::mem::take(&mut self.canvas);
                let frozen = self.canvas_frozen;
                self.canvas_frozen = false;
                let res = self.draw_cell(name.as_deref(), &settings, r + 1, c + 1, cell, Transform::default());
                let drawn = std::mem::replace(&mut self.canvas, saved);
                self.canvas_frozen = frozen;
                res?;
                if let (Some(lo), Some(hi)) = (drawn.min, drawn.max) {
                    extents[r][c] = Extent { left: (-lo.x).max(0.0), right: hi.x.max(0.0), up: hi.y.max(0.0), down: (-lo.y).max(0.0), empty: false };
                }
            }
        }
        let col_sep = settings.column_sep;
        let row_sep = settings.row_sep;
        let left: Vec<f64> = (0..ncols).map(|c| extents.iter().map(|row| if row[c].empty { 0.0 } else { row[c].left }).fold(0.0, f64::max)).collect();
        let right: Vec<f64> = (0..ncols).map(|c| extents.iter().map(|row| if row[c].empty { 0.0 } else { row[c].right }).fold(0.0, f64::max)).collect();
        let up: Vec<f64> = extents.iter().map(|row| row.iter().filter(|e| !e.empty).map(|e| e.up).fold(0.0, f64::max)).collect();
        let down: Vec<f64> = extents.iter().map(|row| row.iter().filter(|e| !e.empty).map(|e| e.down).fold(0.0, f64::max)).collect();
        // origins: columns from the left, rows from the top (the first row's baseline at y = 0)
        let mut xs = Vec::with_capacity(ncols);
        let mut x = 0.0;
        for c in 0..ncols {
            if c > 0 {
                x += right[c - 1] + col_sep + left[c];
            } else {
                x = left[0];
            }
            xs.push(x);
        }
        let mut ys = Vec::with_capacity(nrows);
        let mut y = 0.0;
        for r in 0..nrows {
            if r > 0 {
                let more = if extra[r - 1].is_empty() { 0.0 } else { self.length_of(extra[r - 1].clone()).unwrap_or(0.0) };
                y -= down[r - 1] + row_sep + more + up[r];
            }
            ys.push(y);
        }
        let width = if ncols == 0 { 0.0 } else { xs[ncols - 1] + right[ncols - 1] };
        let height = up.first().copied().unwrap_or(0.0);
        let depth = if nrows == 0 { 0.0 } else { -(ys[nrows - 1] - down[nrows - 1]) };

        // the matrix is a node around the cells
        self.forced_box = Some((width, height, depth));
        let items =
            self.node(NodeSpec { options, name: name.clone(), at: Some(at), text: Vec::new(), place: OnPath::Point(at), point: at, coordinate: false, default_pos: None })?;
        self.forced_box = None;
        self.canvas.items.extend(items);
        let placed = match &name {
            Some(n) => self.nodes.get(n).map(|i| i.transform),
            None => self.last_node.as_ref().map(|i| i.transform),
        }
        .unwrap_or_default();
        // second pass: the cells in place
        for (r, row) in rows.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                let t = placed.compose(&Transform::translation(xs[c], ys[r]));
                self.draw_cell(name.as_deref(), &settings, r + 1, c + 1, cell, t)?;
            }
        }
        Ok(())
    }

    /// One cell, drawn with its origin at `place`.
    fn draw_cell(&mut self, matrix: Option<&str>, settings: &MatrixSettings, r: usize, c: usize, cell: &[Token], place: Transform) -> Result<(), Error> {
        let mut cell = crate::keys::trim(cell).to_vec();
        let empty = cell.is_empty();
        if empty {
            if settings.at_empty_cell.is_empty() {
                return Ok(());
            }
            cell = settings.at_empty_cell.clone();
        }
        self.begin_group();
        self.begin_scope();
        let res = (|| -> Result<(), Error> {
            self.st.transform = place;
            self.st.pending = Transform::default();
            self.tex.define_string("pgfmatrixcurrentrow", &r.to_string(), false);
            self.tex.define_string("pgfmatrixcurrentcolumn", &c.to_string(), false);
            let parity = |n: usize| if n.is_multiple_of(2) { "even" } else { "odd" };
            let styles = format!("every cell/.try,every {} row/.try,every {} column/.try,row {r}/.try,column {c}/.try,row {r} column {c}/.try", parity(r), parity(c));
            self.process_keys(tikra_tokens(&styles), Some("/tikz"))?;
            self.st.apply_pending();
            if !settings.nodes_style.is_empty() {
                let mut v = tikra_tokens("every node/.append style=");
                v.push(Token::Char('{', Cat::Begin));
                v.extend(settings.nodes_style.clone());
                v.push(Token::Char('}', Cat::End));
                self.process_keys(v, Some("/tikz"))?;
            }
            if settings.of_nodes && !empty {
                // |[options]| text
                let (opts, text) = cell_options(&cell);
                let mut statement = tikra_tokens("\\node[anchor=base");
                if let Some(m) = matrix {
                    statement.extend(tikra_tokens(&format!(",name={m}-{r}-{c}")));
                }
                statement.push(Token::other(','));
                statement.extend(opts);
                statement.extend(tikra_tokens("] at (0,0) "));
                statement.push(Token::Char('{', Cat::Begin));
                if settings.math {
                    statement.push(Token::Char('$', Cat::Math));
                }
                statement.extend(text);
                if settings.math {
                    statement.push(Token::Char('$', Cat::Math));
                }
                statement.push(Token::Char('}', Cat::End));
                statement.extend(tikra_tokens(";"));
                self.run_tokens(statement)
            } else if empty {
                self.run_tokens(cell)
            } else {
                let mut v = settings.at_begin_cell.clone();
                v.extend(cell);
                v.extend(settings.at_end_cell.clone());
                self.run_tokens(v)
            }
        })();
        self.end_scope();
        self.end_group();
        res
    }
}

/// `|(name) [red]| text` → (`name=name,red`, `text`).
fn cell_options(cell: &[Token]) -> (Vec<Token>, Vec<Token>) {
    let t = crate::keys::trim(cell);
    if t.first().is_some_and(|x| x.is_char('|')) {
        if let Some(end) = t.iter().skip(1).position(|x| x.is_char('|')) {
            let inner = crate::keys::trim(&t[1..end + 1]);
            let mut options = Vec::new();
            let mut i = 0;
            while i < inner.len() {
                let close = if inner[i].is_char('[') {
                    ']'
                } else if inner[i].is_char('(') {
                    ')'
                } else {
                    i += 1;
                    continue;
                };
                let Some(len) = inner[i + 1..].iter().position(|x| x.is_char(close)) else { break };
                if !options.is_empty() {
                    options.push(Token::other(','));
                }
                if close == ')' {
                    options.extend(tikra_tokens("name="));
                    options.push(Token::Char('{', Cat::Begin));
                    options.extend(inner[i + 1..i + 1 + len].iter().cloned());
                    options.push(Token::Char('}', Cat::End));
                } else {
                    options.extend(inner[i + 1..i + 1 + len].iter().cloned());
                }
                i += len + 2;
            }
            return (options, crate::keys::trim(&t[end + 2..]).to_vec());
        }
    }
    (Vec::new(), t.to_vec())
}

/// What the matrix's options say about its cells.
#[derive(Debug, Clone, Default)]
pub struct MatrixSettings {
    pub column_sep: f64,
    pub row_sep: f64,
    pub of_nodes: bool,
    pub math: bool,
    pub nodes_style: Vec<Token>,
    /// `execute at begin cell`, `execute at end cell`, `execute at empty cell`
    pub at_begin_cell: Vec<Token>,
    pub at_end_cell: Vec<Token>,
    pub at_empty_cell: Vec<Token>,
}

#[allow(dead_code)]
fn unused(_: Canvas) {}
