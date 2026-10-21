//! Node text, typeset by mathra: `{Exit}`, `{$\vec{v}$}`, `{Check every value twice}`.
//! Lines by `\\`, automatic wrapping at `text width`, sizes `\small`, `\Large`.

use mathra::layout::MathBox;

use crate::color::Color;
use crate::scene::Label;

/// Centimeters per em in a 10pt font.
const EM_CM: f64 = 10.0 * 2.54 / 72.27;

/// Size factor of the LaTeX font commands (10pt = 1).
pub fn font_size(text: &str) -> f64 {
    let mut f = 1.0;
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
        let mut rest = text;
        while let Some(i) = rest.find(cmd) {
            let after = &rest[i + cmd.len()..];
            if !after.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) {
                f = factor;
            }
            rest = after;
        }
    }
    f
}

/// Removes font commands that do not change mathra's drawing (family, series, size).
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
            if COMMANDS.contains(&name.as_str()) {
                i += 1 + name.chars().count();
                while c.get(i) == Some(&' ') {
                    i += 1;
                }
                continue;
            }
        }
        out.push(c[i]);
        i += 1;
    }
    out
}

/// Splits into lines at top-level `\\`.
fn manual_lines(text: &str) -> Vec<String> {
    let c: Vec<char> = text.chars().collect();
    let mut lines = vec![String::new()];
    let mut level = 0;
    let mut math = false;
    let mut i = 0;
    while i < c.len() {
        match c[i] {
            '{' => level += 1,
            '}' => level -= 1,
            '$' => math = !math,
            '\\' if level == 0 && !math && c.get(i + 1) == Some(&'\\') => {
                lines.push(String::new());
                i += 2;
                // \\[2pt]
                if c.get(i) == Some(&'[') {
                    while i < c.len() && c[i] != ']' {
                        i += 1;
                    }
                    i += 1;
                }
                continue;
            }
            _ => {}
        }
        lines.last_mut().unwrap().push(c[i]);
        i += 1;
    }
    lines.into_iter().map(|l| l.trim().to_string()).collect()
}

/// Words separated by spaces outside `$…$` and braces.
fn words(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut level = 0;
    let mut math = false;
    for c in line.chars() {
        match c {
            '{' => level += 1,
            '}' => level -= 1,
            '$' => math = !math,
            ' ' if level == 0 && !math => {
                if !current.is_empty() {
                    out.push(std::mem::take(&mut current));
                }
                continue;
            }
            _ => {}
        }
        current.push(c);
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

fn typeset(text: &str, bold: bool, warnings: &mut Vec<String>) -> MathBox {
    let options = mathra::Options { display: false, ..Default::default() };
    let tex = if bold { format!("\\text{{\\textbf{{{text}}}}}") } else { format!("\\text{{{text}}}") };
    match mathra::typeset(&tex, &options) {
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

/// Typesets a node's text. `text_width` (cm) turns on automatic wrapping.
pub fn compose_label(text: &str, font: &str, text_width: Option<f64>, align: char, color: Color, warnings: &mut Vec<String>) -> Label {
    let size = font_size(&format!("{font} {text}"));
    let bold = font.contains("\\bfseries") || font.contains("\\bf ") || font.trim_end().ends_with("\\bf");
    let scale = EM_CM * size / 1000.0;
    let text = clean(text);
    let space = 333.0f32;
    let mut boxes = Vec::new();
    for line in manual_lines(&text) {
        match text_width {
            None => boxes.push(typeset(&line, bold, warnings)),
            Some(max) => {
                let max = (max / scale) as f32;
                let mut current: Option<MathBox> = None;
                for w in words(&line) {
                    let c = typeset(&w, bold, warnings);
                    current = Some(match current {
                        None => c,
                        Some(mut a) if a.width + space + c.width <= max => {
                            a.append(&MathBox::space(space));
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
    }
    Label { lines: boxes, scale, color, align, line_height: 1.2 * EM_CM * size }
}

#[cfg(test)]
mod tests {
    use super::*;

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
