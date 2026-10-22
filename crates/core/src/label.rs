//! Node text, typeset by mathra: `{Exit}`, `{$\vec{v}$}`, `{Check every value twice}`.
//! Lines by `\\`, automatic wrapping at `text width`, sizes `\small`, `\Large`.

use mathra::layout::MathBox;

use crate::color::Color;
use crate::scene::Label;

/// Centimeters per em in a 10pt font.
const EM_CM: f64 = 10.0 * 2.54 / 72.27;

/// The font labels are set in: Latin Modern Math, the Computer Modern that TeX sets the figures in.
pub const FONT: mathra::font::MathFont = mathra::font::MathFont::LatinModern;

/// Size factor of the LaTeX font commands (10pt = 1).
pub fn font_size(text: &str) -> f64 {
    // the last size command in the text is the one in effect
    let mut f = 1.0;
    let mut at = None;
    for (cmd, factor) in [
        ("\\tiny", 0.5),
        ("\\scriptsize", 0.7),
        ("\\footnotesize", 0.8),
        ("\\small", 0.9),
        ("\\normalsize", 1.0),
        ("\\large", 1.2),
        ("\\Large", 1.44),
        ("\\LARGE", 1.728),
        ("\\huge", 2.074),
        ("\\Huge", 2.488),
    ] {
        // \Large contains \large: look for the command followed by something that is not a letter
        let mut from = 0;
        while let Some(i) = text[from..].find(cmd) {
            let i = from + i;
            let after = &text[i + cmd.len()..];
            if !after.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) && at.is_none_or(|a| i > a) {
                f = factor;
                at = Some(i);
            }
            from = i + cmd.len();
        }
    }
    f
}

/// Removes font commands that do not change mathra's drawing (family, series, size), and writes
/// `\verb` as `\texttt`.
fn clean(text: &str) -> String {
    const COMMANDS: &[&str] = &[
        "tiny",
        "scriptsize",
        "footnotesize",
        "small",
        "normalsize",
        "large",
        "Large",
        "LARGE",
        "huge",
        "Huge",
        "sffamily",
        "rmfamily",
        "ttfamily",
        "itshape",
        "mdseries",
        "upshape",
        "centering",
        "raggedright",
        "raggedleft",
        "it",
        "sf",
        "tt",
        "rm",
    ];
    let c: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < c.len() {
        if c[i] == '\\' {
            let name: String = c[i + 1..].iter().take_while(|x| x.is_ascii_alphabetic()).collect();
            // `\verb!…!` (any delimiter): the text as it is, in typewriter
            if name == "verb" {
                if let Some(&d) = c.get(i + 5) {
                    if let Some(len) = c[i + 6..].iter().position(|&x| x == d) {
                        let body: String = c[i + 6..i + 6 + len].iter().collect();
                        out.push_str(&format!("\\texttt{{{body}}}"));
                        i += 7 + len;
                        continue;
                    }
                }
            }
            // `\hskip0pt` (what `execute at begin node` puts first, to allow a break): no space
            if name == "hskip" {
                i += 6;
                while c.get(i).is_some_and(|x| *x == ' ' || x.is_ascii_digit() || *x == '.' || *x == '-') {
                    i += 1;
                }
                if c.get(i..i + 2).is_some_and(|u| ["pt", "em", "ex", "cm", "mm"].contains(&u.iter().collect::<String>().as_str())) {
                    i += 2;
                }
                continue;
            }
            if COMMANDS.contains(&name.as_str()) || name == "bfseries" || name == "bf" {
                i += 1 + name.chars().count();
                while c.get(i) == Some(&' ') {
                    i += 1;
                }
                // a control word kept before keeps its end (`\foo\small x`)
                let tail: String = out.chars().rev().take_while(|x| x.is_ascii_alphabetic()).collect();
                if !tail.is_empty() && out[..out.len() - tail.len()].ends_with('\\') {
                    out.push(' ');
                }
                continue;
            }
        }
        out.push(c[i]);
        i += 1;
    }
    out
}

/// Typewriter text as TeX's `cmtt` sets it, every character half an em and a fortieth wide: letters
/// and digits from the math font's monospace alphabet, the others (which it lacks) centered in that
/// width, spaces too.
fn typewriter(t: &str) -> String {
    const WIDTH: f32 = 0.525;
    let font = FONT.get();
    let mut out = String::from("$");
    let mut run = String::new();
    let flush = |run: &mut String, out: &mut String| {
        if !run.is_empty() {
            out.push_str(&format!("\\mathtt{{{run}}}"));
            run.clear();
        }
    };
    // the text has no backslash: its braces only group
    for c in t.chars().filter(|c| !matches!(c, '{' | '}')) {
        if c.is_ascii_alphanumeric() {
            run.push(c);
            continue;
        }
        flush(&mut run, &mut out);
        // cmtt's parentheses and brackets are shorter than the roman ones, from 1/12 em below the
        // baseline to 25/36 above: the roman glyph shrunk to that height, in the box of cmtt's
        // (bottom, top)
        let short = match c {
            '(' | ')' | '[' | ']' => Some((7.0 / 9.0, -1.0 / 12.0, 25.0 / 36.0)),
            _ => None,
        };
        let advance = if c == ' ' { 0.0 } else { font.glyph(c).map_or(WIDTH, |g| font.advance(g) / font.units_per_em()) };
        let advance = advance * short.map_or(1.0, |s: (f32, f32, f32)| s.0);
        let side = ((WIDTH - advance) / 2.0).max(0.0);
        let glyph = match (c, short) {
            (' ', _) => String::new(),
            ('_' | '#' | '%' | '&' | '$', _) => format!("\\text{{\\{c}}}"),
            (c, Some((scale, bottom, top))) => format!("\\smash{{\\scalebox{{{scale:.4}}}{{{c}}}}}\\rule[{bottom:.4}em]{{0em}}{{{:.4}em}}", top - bottom),
            (c, None) => format!("\\text{{{c}}}"),
        };
        out.push_str(&format!("\\hspace{{{side}em}}{glyph}\\hspace{{{side}em}}"));
    }
    flush(&mut run, &mut out);
    out.push('$');
    if out == "$$" {
        String::new()
    } else {
        out
    }
}

/// Text in another family (`\texttt{…}`, `\textit{…}`, `\emph{…}`, `\textsf{…}`, or all of it
/// under `\ttfamily`, `\itshape`, `\sffamily`) as the math font's letters of that family, the
/// closest it has to TeX's typewriter, italic and sans serif fonts; its spaces kept.
fn families(text: &str, all: Option<&str>) -> String {
    let set = |t: &str, family: &str| -> String {
        if family == "mathtt" {
            return typewriter(t);
        }
        let parts: Vec<String> = t.split(' ').filter(|p| !p.is_empty()).map(|p| format!("\\{family}{{{p}}}")).collect();
        if parts.is_empty() {
            String::new()
        } else {
            format!("${}$", parts.join("\\ "))
        }
    };
    if let Some(family) = all {
        if !text.contains('$') && !text.contains('\\') {
            return set(text, family);
        }
    }
    let mut out = text.to_string();
    for (command, family) in [("texttt", "mathtt"), ("textit", "mathit"), ("emph", "mathit"), ("textsf", "mathsf")] {
        let open = format!("\\{command}{{");
        let mut done = String::new();
        let mut rest = out.as_str();
        while let Some(i) = rest.find(&open) {
            done.push_str(&rest[..i]);
            let body = &rest[i + open.len()..];
            // to the matching brace
            let mut depth = 1;
            let mut end = body.len();
            for (k, c) in body.char_indices() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end = k;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let inner = &body[..end];
            done.push_str(&if inner.contains('$') || inner.contains('\\') { inner.to_string() } else { set(inner, family) });
            rest = body.get(end + 1..).unwrap_or("");
        }
        done.push_str(rest);
        out = done;
    }
    out
}

/// A TeX dimension at the start of `s` (`2cm`, `1 pt`, `.5ex`): its value in points and the rest.
fn dimension(s: &str, size: f64) -> Option<(f64, &str)> {
    let s = s.trim_start();
    let end = s.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == '+')).unwrap_or(s.len());
    let v: f64 = s[..end].parse().ok()?;
    let rest = s[end..].trim_start();
    for (unit, pt) in [("pt", 1.0), ("cm", 72.27 / 2.54), ("mm", 72.27 / 25.4), ("in", 72.27), ("bp", 72.27 / 72.0), ("em", 10.0 * size * quad(10.0 * size)), ("ex", 4.3 * size)] {
        if let Some(r) = rest.strip_prefix(unit) {
            return Some((v * pt, r));
        }
    }
    None
}

/// The ligatures of TeX's text fonts: `--` and `---` as dashes, ``` `` ``` and `''` as quotes; not
/// in math, nor in typewriter text (whose font has none).
fn ligatures(text: &str) -> String {
    if !text.contains("--") && !text.contains("``") && !text.contains("''") {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut math = false;
    let mut rest = text;
    while let Some(c) = rest.chars().next() {
        if !math && rest.starts_with("\\texttt{") {
            // to the matching brace, as it is
            let mut depth = 0;
            let end = rest.char_indices().find(|&(_, ch)| {
                match ch {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ => {}
                }
                ch == '}' && depth == 0
            });
            let end = end.map_or(rest.len(), |(i, _)| i + 1);
            out.push_str(&rest[..end]);
            rest = &rest[end..];
            continue;
        }
        if c == '\\' {
            // a control symbol (`\$`) or word as it is
            let n = rest[1..].chars().next().map_or(0, |d| {
                if d.is_ascii_alphabetic() {
                    rest[1..].find(|x: char| !x.is_ascii_alphabetic()).unwrap_or(rest.len() - 1)
                } else {
                    d.len_utf8()
                }
            });
            out.push_str(&rest[..1 + n]);
            rest = &rest[1 + n..];
            continue;
        }
        if c == '$' {
            math = !math;
        } else if !math {
            let swap = [("---", "\\textemdash{}"), ("--", "\\textendash{}"), ("``", "\u{201c}"), ("''", "\u{201d}")].into_iter().find(|(l, _)| rest.starts_with(l));
            if let Some((l, r)) = swap {
                out.push_str(r);
                rest = &rest[l.len()..];
                continue;
            }
        }
        out.push(c);
        rest = &rest[c.len_utf8()..];
    }
    out
}

/// `\vrule width 1pt height 2cm depth 0pt` as mathra's `$\rule[raise]{width}{height}$`, in ems of
/// the label's font (so that the label's scaling gives the rule its size in points). A height or
/// depth not given runs to the line's, `running` (points); without it the rules are left out.
fn rules(text: &str, size: f64, running: Option<(f64, f64)>) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find("\\vrule") {
        let after = &rest[i + 6..];
        if after.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) {
            out.push_str(&rest[..i + 6]);
            rest = after;
            continue;
        }
        out.push_str(&rest[..i]);
        let (mut w, mut h, mut d) = (0.4, None, None);
        let mut r = after;
        loop {
            let t = r.trim_start();
            let Some((key, more)) = ["width", "height", "depth"].iter().find_map(|k| t.strip_prefix(k).map(|m| (*k, m))) else { break };
            let Some((v, more)) = dimension(more, size) else { break };
            match key {
                "width" => w = v,
                "height" => h = Some(v),
                _ => d = Some(v),
            }
            r = more;
        }
        rest = r;
        let Some((line_h, line_d)) = running else { continue };
        let em = 10.0 * size;
        let (h, d) = (h.unwrap_or(line_h), d.unwrap_or(line_d));
        out.push_str(&format!("$\\rule[{:.4}em]{{{:.4}em}}{{{:.4}em}}$", -d / em, w / em, (h + d) / em));
    }
    out.push_str(rest);
    out
}

/// `\hbox to 2cm{Text\hss}`: the text in a box of that width, to the left of it (`\hss` after),
/// to the right (`\hss` before) or centered (both); `stretch` is how much the label widens its
/// boxes afterwards.
fn hboxes(text: &str, size: f64, stretch: f64) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find("\\hbox") {
        out.push_str(&rest[..i]);
        let after = &rest[i + 5..];
        let parsed = (|| {
            let a = after.trim_start().strip_prefix("to")?;
            let (w, a) = dimension(a, size)?;
            let body = a.trim_start().strip_prefix('{')?;
            let mut depth = 1;
            let end = body.char_indices().find(|&(_, c)| {
                match c {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ => {}
                }
                depth == 0
            })?;
            Some((w, &body[..end.0], &body[end.0 + 1..]))
        })();
        let Some((w, inner, after)) = parsed else {
            out.push_str("\\hbox");
            rest = after;
            continue;
        };
        let inner = inner.trim();
        let (left, inner) = inner.strip_prefix("\\hss").map_or((false, inner), |r| (true, r));
        let (right, inner) = inner.strip_suffix("\\hss").map_or((false, inner), |r| (true, r));
        let inner = inner.trim();
        let em = w / (10.0 * size * stretch);
        if inner.contains('$') || !(left || right) {
            out.push_str(inner);
        } else if left && right {
            out.push_str(&format!("$\\hspace{{{:.4}em}}\\clap{{{inner}}}\\hspace{{{:.4}em}}$", em / 2.0, em / 2.0));
        } else if right {
            out.push_str(&format!("$\\rlap{{{inner}}}\\hspace{{{em:.4}em}}$"));
        } else {
            out.push_str(&format!("$\\hspace{{{em:.4}em}}\\llap{{{inner}}}$"));
        }
        rest = after;
    }
    out.push_str(rest);
    out
}

/// Splits into lines at top-level `\\` and `\par`, each with the space (pt) added after it:
/// `\\[2pt]`, or the `\vskip 1em` after a `\par`.
fn manual_lines(text: &str, size: f64) -> Vec<(String, f64)> {
    let c: Vec<char> = text.chars().collect();
    let mut lines = vec![(String::new(), 0.0)];
    let mut level = 0;
    let mut math = false;
    let mut i = 0;
    // a dimension from `i`: up to the char `end`, or a number and a unit of two letters
    let skip = |i: usize, end: Option<char>| -> (f64, usize) {
        let rest: String = c[i.min(c.len())..].iter().collect();
        let len = match end {
            Some(e) => rest.find(e).unwrap_or(rest.len()),
            None => {
                let t = rest.trim_start();
                let n = t.find(|x: char| !(x.is_ascii_digit() || x == '.' || x == '-')).unwrap_or(t.len());
                rest.len() - t.len() + (n + 2).min(t.len())
            }
        };
        let v = dimension(&rest[..len], size).map_or(0.0, |(v, _)| v);
        (v, i + rest[..len].chars().count())
    };
    while i < c.len() {
        match c[i] {
            '{' => level += 1,
            '}' => level -= 1,
            '$' => math = !math,
            // `\par` ends a paragraph, and a line
            '\\' if level == 0 && !math && c[i + 1..].starts_with(&['p', 'a', 'r']) && !c.get(i + 4).is_some_and(|x| x.is_ascii_alphabetic()) => {
                i += 4;
                let mut j = i;
                while c.get(j).is_some_and(|x| x.is_whitespace()) {
                    j += 1;
                }
                if c[j..].starts_with(&['\\', 'v', 's', 'k', 'i', 'p']) {
                    let (v, next) = skip(j + 6, None);
                    lines.last_mut().unwrap().1 += v;
                    i = next;
                }
                lines.push((String::new(), 0.0));
                continue;
            }
            '\\' if level == 0 && !math && c.get(i + 1) == Some(&'\\') => {
                i += 2;
                if c.get(i) == Some(&'[') {
                    let (v, next) = skip(i + 1, Some(']'));
                    lines.last_mut().unwrap().1 += v;
                    i = next + 1;
                }
                lines.push((String::new(), 0.0));
                continue;
            }
            _ => {}
        }
        lines.last_mut().unwrap().0.push(c[i]);
        i += 1;
    }
    lines.into_iter().map(|(l, v)| (l.trim().to_string(), v)).collect()
}

/// Words separated by spaces outside `$…$` and braces.
/// The pieces a line may break between, each with whether a space comes before it: the words,
/// and the parts of a hyphenated word (`Fixed-` `Parameter`), as TeX may break after a hyphen.
fn words(line: &str) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut level = 0;
    let mut math = false;
    let mut space = false;
    let c: Vec<char> = line.chars().collect();
    for (i, &ch) in c.iter().enumerate() {
        match ch {
            '{' => level += 1,
            '}' => level -= 1,
            '$' => math = !math,
            ' ' if level == 0 && !math => {
                if !current.is_empty() {
                    out.push((std::mem::take(&mut current), space));
                }
                space = true;
                continue;
            }
            '-' if level == 0 && !math && i > 0 && c[i - 1].is_alphabetic() && c.get(i + 1).is_some_and(|x| x.is_alphabetic()) => {
                current.push(ch);
                out.push((std::mem::take(&mut current), space));
                space = false;
                continue;
            }
            _ => {}
        }
        current.push(ch);
    }
    if !current.is_empty() {
        out.push((current, space));
    }
    out
}

fn typeset(text: &str, bold: bool, warnings: &mut Vec<String>) -> MathBox {
    let options = mathra::Options { display: false, ..Default::default() };
    let tex = if bold { format!("\\text{{\\textbf{{{text}}}}}") } else { format!("\\text{{{text}}}") };
    match mathra::typeset_in_font(&tex, &options, FONT) {
        Ok((c, applied)) => {
            warnings.extend(applied.into_iter().map(|a| format!("{} {}", a.rule, a.description)));
            c
        }
        // even in tolerant mode mathra refuses some input (nesting past its limit)
        Err(e) => {
            warnings.push(format!("label '{text}' not typeset: {e}"));
            MathBox::empty()
        }
    }
}

/// How much wider than its 10pt design TeX's font of `pt` points sets text, in the family the text
/// is in: LaTeX picks the Computer Modern of the design size at or below it (`cmr7` for
/// `\scriptsize`, `cmr17` for `\Huge`), whose letters are proportionally wider at small sizes and
/// narrower at large ones, while the label font has only the 10pt design. From the fonts' metrics.
fn design_width(pt: f64, family: char) -> f64 {
    let table: &[(f64, f64)] = match family {
        // cmtt8, cmtt9, cmtt10 and cmtt12
        't' => &[(0.0, 1.0119), (9.0, 1.0), (10.0, 1.0), (12.0, 0.9802)],
        // cmmi5 … cmmi12
        'm' => &[(0.0, 1.4283), (6.0, 1.2526), (7.0, 1.1559), (8.0, 1.0584), (9.0, 1.026), (10.0, 1.0), (12.0, 0.9784)],
        // cmbx5 … cmbx12
        'b' => &[(0.0, 1.3083), (6.0, 1.1996), (7.0, 1.1221), (8.0, 1.064), (9.0, 1.0285), (10.0, 1.0), (12.0, 0.9774)],
        // cmr5 … cmr17
        _ => &[(0.0, 1.3611), (6.0, 1.2222), (7.0, 1.1389), (8.0, 1.0625), (9.0, 1.0278), (10.0, 1.0), (12.0, 0.9791), (17.0, 0.9172)],
    };
    table.iter().rev().find(|(from, _)| pt + 0.01 >= *from).map_or(1.0, |(_, w)| *w)
}

/// The quad (`1em`) of TeX's roman font of `pt` points, over `pt`.
pub fn quad(pt: f64) -> f64 {
    design_width(pt, 'r')
}

/// The family most of the text is set in: typewriter (`t`) when all of it is `\texttt`, math (`m`)
/// when all of it is between dollars, else bold (`b`) or roman (`r`).
fn family(text: &str, font: &str, bold: bool) -> char {
    if font.contains("\\ttfamily") || text.contains("\\ttfamily") {
        return 't';
    }
    // the text outside the groups that `open` starts and `close` ends
    let outside = |t: &str, open: &str, close: char| -> String {
        let mut out = String::new();
        let mut rest = t;
        while let Some(i) = rest.find(open) {
            out.push_str(&rest[..i]);
            rest = &rest[i + open.len()..];
            rest = rest.find(close).map_or("", |j| &rest[j + 1..]);
        }
        out.push_str(rest);
        out
    };
    // letters or digits, not counting the names of control words
    let letters = |t: &str| {
        t.split('\\').enumerate().any(|(k, w)| {
            let w = if k == 0 { w } else { w.trim_start_matches(|c: char| c.is_ascii_alphabetic()) };
            w.chars().any(|c| c.is_alphanumeric())
        })
    };
    if !letters(text) {
        return 'r';
    }
    if !letters(&outside(text, "\\texttt{", '}')) {
        return 't';
    }
    if !letters(&outside(text, "$", '$')) {
        return 'm';
    }
    if bold {
        'b'
    } else {
        'r'
    }
}

/// The box `f` times as wide, its items moved with it (the glyphs are widened when drawn).
fn widen(mut b: MathBox, f: f32) -> MathBox {
    use mathra::layout::Item;
    for item in &mut b.items {
        match item {
            Item::Glyph { x, .. } => *x *= f,
            Item::Rule { x, width, .. } => {
                *x *= f;
                *width *= f;
            }
            Item::Line { x1, x2, .. } => {
                *x1 *= f;
                *x2 *= f;
            }
            Item::Ellipse { cx, rx, .. } => {
                *cx *= f;
                *rx *= f;
            }
        }
    }
    b.width *= f;
    b
}

/// Typesets a node's text. `text_width` (cm) turns on automatic wrapping.
pub fn compose_label(text: &str, font: &str, text_width: Option<f64>, align: char, color: Color, warnings: &mut Vec<String>) -> Label {
    let size = font_size(&format!("{font} {text}"));
    let bold = font.contains("\\bfseries") || font.contains("\\bf ") || font.trim_end().ends_with("\\bf") || text.contains("\\bfseries") || text.contains("\\bf ");
    let scale = EM_CM * size / 1000.0;
    let all = [("\\ttfamily", "mathtt"), ("\\itshape", "mathit"), ("\\sffamily", "mathsf")].into_iter().find(|(c, _)| font.contains(c) || text.contains(c)).map(|(_, f)| f);
    let mut stretch = design_width(10.0 * size, family(text, font, bold));
    if all == Some("mathit") {
        // text italic (cmti) is narrower than the math italic it is drawn with: by 0.943 on the
        // lowercase letters of English text, weighted by how often they come
        stretch *= 0.943;
    }
    let space = 333.0f32;
    let mut boxes = Vec::new();
    // a text that is one group (`{upper line \\ lower line}`): what it groups
    let text = crate::keys::strip_braces(text);
    let text = if all == Some("mathtt") { clean(text) } else { ligatures(&clean(text)) };
    // the space after each line besides the baselines' (cm)
    let mut skips: Vec<f64> = Vec::new();
    for (line, after) in manual_lines(&text, size) {
        let line = if line.contains("\\hbox") { hboxes(&line, size, stretch) } else { line };
        let line = if line.contains("\\vrule") {
            // the rules run to the height and depth of the rest of the line
            let bare = typeset(&families(&rules(&line, size, None), all), bold, &mut Vec::new());
            let pt = 10.0 * size / 1000.0;
            families(&rules(&line, size, Some((bare.height as f64 * pt, bare.depth as f64 * pt))), all)
        } else {
            families(&line, all)
        };
        match text_width {
            None => boxes.push(typeset(&line, bold, warnings)),
            Some(max) => {
                let max = (max / scale / stretch) as f32;
                let mut current: Option<MathBox> = None;
                for (w, spaced) in words(&line) {
                    let c = typeset(&w, bold, warnings);
                    let gap = if spaced { space } else { 0.0 };
                    current = Some(match current {
                        None => c,
                        Some(mut a) if a.width + gap + c.width <= max => {
                            if spaced {
                                a.append(&MathBox::space(space));
                            }
                            a.append(&c);
                            a
                        }
                        Some(a) => {
                            boxes.push(a);
                            c
                        }
                    });
                }
                boxes.push(current.unwrap_or_else(MathBox::empty));
            }
        }
        skips.resize(boxes.len(), 0.0);
        if let Some(s) = skips.last_mut() {
            *s = after * 2.54 / 72.27;
        }
    }
    let lines = boxes.into_iter().map(|b| widen(b, stretch as f32)).collect();
    Label { lines, scale, color, align, line_height: 1.2 * EM_CM * size, stretch, skips }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_and_the_last_size() {
        let mut w = Vec::new();
        let r = compose_label("A\\vrule width 1pt height 2cm", "\\Huge", None, 'c', Color::BLACK, &mut w);
        assert!(w.is_empty(), "{w:?}");
        // 2cm high whatever the size
        assert!((r.height() - 2.0).abs() < 0.02, "{}", r.height());
        assert!((font_size("\\Huge \\scriptsize x") - 0.7).abs() < 1e-9);
    }

    #[test]
    fn wrapping_and_size() {
        let mut w = Vec::new();
        let r = compose_label("Check every value twice", "", Some(2.5), 'c', Color::BLACK, &mut w);
        assert!(r.lines.len() >= 2, "{}", r.lines.len());
        assert!(w.is_empty(), "{w:?}");
        assert!((font_size("\\Large x") - 1.44).abs() < 1e-9);
        assert!((font_size("\\large x") - 1.2).abs() < 1e-9);
        let r = compose_label("$m_1$ \\\\ second", "", None, 'c', Color::BLACK, &mut w);
        assert_eq!(r.lines.len(), 2);
    }
}
