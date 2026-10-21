//! tikra: TikZ figures to SVG and PNG, with no TeX install. Math labels are drawn by mathra.
//!
//! ```
//! let r = tikra::convert(r"\begin{tikzpicture} \draw[->] (0,0) -- (1,1) node[right] {$x$}; \end{tikzpicture}").unwrap();
//! assert!(r.svg.starts_with("<svg"));
//! ```

pub mod canvas;
pub mod color;
pub mod driver;
pub mod engine;
pub mod expr;
pub mod keys;
pub mod label;
pub mod options;
pub mod path;
pub mod reader;
pub mod render;
pub mod scene;
pub mod tex;

use std::fmt;

/// Centimeters per TeX point.
pub const PT: f64 = 2.54 / 72.27;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Expression(String),
    Figure(String),
    Png(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Expression(m) => write!(f, "expression: {m}"),
            Error::Figure(m) => write!(f, "figure: {m}"),
            Error::Png(m) => write!(f, "png: {m}"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone)]
pub struct Output {
    pub svg: String,
    /// What was ignored or approximated (unsupported commands, labels with mathra warnings).
    pub warnings: Vec<String>,
}

/// Accepts a full `standalone` document, a bare `tikzpicture`, an inline `\tikz` or an `axis`.
pub fn convert(code: &str) -> Result<Output, Error> {
    let mut early = Vec::new();
    // TikZ escaped twice (\\draw, \\coordinate): the same repair mathra applies to equations
    let repaired = mathra::input::latex::undo_doubled_backslashes(code);
    if repaired.is_some() {
        early.push("doubled backslashes (code escaped twice): undid one level".to_string());
    }
    let mut base = repaired.unwrap_or_else(|| code.to_string());
    if let Some(r) = repair_markdown(&base) {
        early.push("TikZ escaped for Markdown (```, \\\\draw, \\[ \\]): undone".to_string());
        base = r;
    }
    let text = reader::strip_comments(&base);
    let input = if text.contains("\\begin{tikzpicture}") || tikz_command(&text).is_some() {
        base
    } else if text.contains("\\begin{axis}") || ["\\draw", "\\fill", "\\node", "\\path", "\\filldraw", "\\coordinate"].iter().any(|c| text.contains(c)) {
        // an axis or loose statements, with no tikzpicture around them
        format!("\\begin{{tikzpicture}}\\relax\n{base}\n\\end{{tikzpicture}}")
    } else {
        return Err(Error::Figure("no tikzpicture, \\tikz or axis found".into()));
    };
    let (canvas, warnings) = engine::run_document(&input)?;
    let svg = driver::svg(&canvas, code, "Figure");
    let mut all = early;
    all.extend(warnings);
    Ok(Output { svg, warnings: all })
}

/// The earlier interpreter, kept while the engine replaces it.
pub fn convert_legacy(code: &str) -> Result<Output, Error> {
    let mut env = reader::Env::new();
    // TikZ escaped twice (\\draw, \\coordinate): the same repair mathra applies to equations
    let repaired = mathra::input::latex::undo_doubled_backslashes(code);
    if repaired.is_some() {
        env.warn("doubled backslashes (code escaped twice): undid one level");
    }
    let mut base = repaired.unwrap_or_else(|| code.to_string());
    if let Some(r) = repair_markdown(&base) {
        env.warn("TikZ escaped for Markdown (```, \\\\draw, \\[ \\]): undone");
        base = r;
    }
    let text = reader::strip_comments(&base);
    // margin: the standalone border, or 2pt
    env.scene.margin = standalone_border(&text).unwrap_or(2.0 * PT);

    let (preamble, options, body, end) = if let Some(i) = text.find("\\begin{tikzpicture}") {
        let after: Vec<char> = text[i + "\\begin{tikzpicture}".len()..].chars().collect();
        let mut k = 0;
        while k < after.len() && after[k].is_whitespace() {
            k += 1;
        }
        let (options, start) = if after.get(k) == Some(&'[') { reader::block(&after, k).unwrap_or_default() } else { (String::new(), 0) };
        let rest: String = after[start..].iter().collect();
        if text.matches("\\begin{tikzpicture}").count() > 1 {
            env.warn("more than one tikzpicture: only the first was drawn");
        }
        (text[..i].to_string(), options, rest, Some("tikzpicture"))
    } else if let Some(i) = tikz_command(&text) {
        let after: Vec<char> = text[i + 5..].chars().collect();
        let mut k = 0;
        while k < after.len() && after[k].is_whitespace() {
            k += 1;
        }
        let (options, start) = if after.get(k) == Some(&'[') { reader::block(&after, k).unwrap_or_default() } else { (String::new(), k) };
        let mut rest: String = after[start..].iter().collect();
        let t = rest.trim_start();
        if t.starts_with('{') {
            let c: Vec<char> = t.chars().collect();
            rest = reader::block(&c, 0).map(|b| b.0).unwrap_or_default();
        }
        (text[..i].to_string(), options, rest, None)
    } else if text.contains("\\begin{axis}") {
        (String::new(), String::new(), text.clone(), None)
    } else if ["\\draw", "\\fill", "\\node", "\\path", "\\filldraw", "\\coordinate"].iter().any(|c| text.contains(c)) {
        // loose statements, with no tikzpicture around them
        (String::new(), String::new(), text.clone(), None)
    } else {
        return Err(Error::Figure("no tikzpicture, \\tikz or axis found".into()));
    };

    // the preamble only defines styles, colors and macros
    let preamble = clean_preamble(&preamble);
    reader::Reader::new(&preamble, &mut env).run(None)?;
    let mut r = reader::Reader::new(&body, &mut env);
    r.push_scope(&options)?;
    r.run(end)?;
    r.pop_scope();

    let svg = render::svg(&env.scene, code, "Figure");
    Ok(Output { svg, warnings: env.warnings })
}

/// TikZ that went through a Markdown generator: code fences, `\\draw` and `\[red\]`.
fn repair_markdown(t: &str) -> Option<String> {
    const COMMANDS: &[&str] = &["draw", "fill", "filldraw", "path", "node", "coordinate", "foreach", "begin", "end", "def", "clip", "tikzset", "shade", "pgfmathsetmacro"];
    let mut changed = false;
    let mut s: String = t
        .lines()
        .filter(|l| {
            let fence = l.trim_start().starts_with("```");
            changed |= fence;
            !fence
        })
        .collect::<Vec<_>>()
        .join("\n");
    for c in COMMANDS {
        let doubled = format!("\\\\{c}");
        if s.contains(&doubled) {
            s = s.replace(&doubled, &format!("\\{c}"));
            changed = true;
        }
    }
    if s.contains("\\[") || s.contains("\\]") {
        // a Markdown \[ inside \draw\[…\]; \\[2pt] (a line break with spacing) is left alone
        let c: Vec<char> = s.chars().collect();
        let mut out = String::with_capacity(s.len());
        let mut i = 0;
        while i < c.len() {
            if c[i] == '\\' && matches!(c.get(i + 1), Some('[') | Some(']')) && (i == 0 || c[i - 1] != '\\') {
                out.push(c[i + 1]);
                i += 2;
                changed = true;
                continue;
            }
            out.push(c[i]);
            i += 1;
        }
        s = out;
    }
    changed.then_some(s)
}

/// Position of `\\tikz` as a command (not `\\tikzset` or `\\tikzstyle`).
fn tikz_command(text: &str) -> Option<usize> {
    text.match_indices("\\tikz").map(|(i, _)| i).find(|&i| !text[i + 5..].starts_with(|c: char| c.is_ascii_alphabetic()))
}

fn standalone_border(text: &str) -> Option<f64> {
    let i = text.find("\\documentclass[")?;
    let rest = &text[i + 15..];
    let end = rest.find(']')?;
    let o = options::parse(&rest[..end]);
    let b = o.iter().find(|x| x.key == "border")?.value.clone()?;
    expr::eval_length(options::strip_braces(&b).split_whitespace().next()?, &expr::Variables::new()).ok().map(|v| v.0)
}

/// Drops from the preamble what is not a definition (documentclass, usepackage, begin{document}).
fn clean_preamble(p: &str) -> String {
    p.lines()
        .filter(|l| {
            let t = l.trim_start();
            !(t.starts_with("\\documentclass")
                || t.starts_with("\\usepackage")
                || t.starts_with("\\begin{document}")
                || t.starts_with("\\usetikzlibrary")
                || t.starts_with("\\pgfplotsset"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// PNG pixel cap (100 megapixels, ~400 MB of memory).
pub const MAX_PIXELS: u64 = 100_000_000;

/// Rasterizes the SVG to PNG. `dpi` 300 is the print default; `background` paints it white.
pub fn png(svg: &str, dpi: f32, background: bool) -> Result<Vec<u8>, Error> {
    use resvg::{tiny_skia, usvg};
    if !(dpi > 0.0 && dpi.is_finite()) {
        return Err(Error::Png(format!("invalid dpi: {dpi}")));
    }
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).map_err(png_error)?;
    // usvg converts the `pt` of the width attribute at 96 px per inch: the scale is relative to that
    let scale = dpi / 96.0;
    let size = tree.size().to_int_size().scale_by(scale).ok_or(Error::Png("invalid size".into()))?;
    if u64::from(size.width()) * u64::from(size.height()) > MAX_PIXELS {
        return Err(Error::Png(format!("image too large: {} × {} px at {dpi} dpi (cap of {MAX_PIXELS} pixels)", size.width(), size.height())));
    }
    let mut image = tiny_skia::Pixmap::new(size.width(), size.height()).ok_or(Error::Png("image too large".into()))?;
    if background {
        image.fill(tiny_skia::Color::WHITE);
    }
    resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut image.as_mut());
    image.encode_png().map_err(png_error)
}

/// Errors from the rasterization libraries (invalid SVG, PNG encoding failure).
fn png_error(e: impl fmt::Display) -> Error {
    Error::Png(e.to_string())
}
