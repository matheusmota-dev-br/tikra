//! TikZ option lists: `[->, thick, draw=red!50, fill=blue!10, minimum width=1.2cm, pointer]`.
//! Styles from `\tikzset{name/.style={…}}` are expanded recursively.

use std::collections::HashMap;

/// One option: a key and an optional value, in the order they appeared.
#[derive(Debug, Clone, PartialEq)]
pub struct Opt {
    pub key: String,
    pub value: Option<String>,
}

/// Splits at top-level commas, respecting `{}`, `()` and `[]`.
pub fn split_top_level(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut level = 0i32;
    for c in text.chars() {
        match c {
            '{' | '(' | '[' => level += 1,
            '}' | ')' | ']' => level -= 1,
            ',' if level == 0 => {
                parts.push(std::mem::take(&mut current));
                continue;
            }
            _ => {}
        }
        current.push(c);
    }
    parts.push(current);
    parts.into_iter().map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect()
}

/// Strips one pair of outer braces: `{a, b}` → `a, b`.
pub fn strip_braces(v: &str) -> &str {
    let t = v.trim();
    if t.starts_with('{') && t.ends_with('}') && t.len() >= 2 {
        let inner = &t[1..t.len() - 1];
        // only when the opening brace closes at the end
        let mut level = 0;
        for (i, c) in inner.char_indices() {
            match c {
                '{' => level += 1,
                '}' => {
                    level -= 1;
                    if level < 0 && i < inner.len() {
                        return t;
                    }
                }
                _ => {}
            }
        }
        return inner.trim();
    }
    t
}

/// Position of the first `=` outside braces and brackets (`-{Stealth[length=3mm]}` has none).
fn top_level_equals(p: &str) -> Option<usize> {
    let mut level = 0i32;
    for (i, c) in p.char_indices() {
        match c {
            '{' | '[' | '(' => level += 1,
            '}' | ']' | ')' => level -= 1,
            '=' if level == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

pub fn parse(text: &str) -> Vec<Opt> {
    split_top_level(text)
        .into_iter()
        .map(|p| match top_level_equals(&p) {
            // `->` and `<->` are not key=value; `>=Latex` is
            Some(i) if i > 0 => Opt { key: p[..i].trim().to_string(), value: Some(strip_braces(&p[i + 1..]).to_string()) },
            _ => Opt { key: p, value: None },
        })
        .collect()
}

/// Cap on options after expanding styles.
pub const MAX_OPTIONS: usize = 10_000;

/// User-defined styles, `name` → options.
#[derive(Debug, Clone, Default)]
pub struct Styles {
    pub defined: HashMap<String, String>,
}

impl Styles {
    /// Reads the content of `\tikzset{…}` or the options of a `tikzpicture`, keeping the `/.style` entries.
    /// Returns the options that are not style definitions.
    pub fn register(&mut self, text: &str) -> Vec<Opt> {
        let mut rest = Vec::new();
        for o in parse(text) {
            if let Some(name) = o.key.strip_suffix("/.style").or_else(|| o.key.strip_suffix("/.append style")) {
                let name = name.trim().to_string();
                let value = o.value.unwrap_or_default();
                let new = match (o.key.ends_with("append style"), self.defined.get(&name)) {
                    (true, Some(old)) => format!("{old},{value}"),
                    _ => value,
                };
                self.defined.insert(name, new);
            } else {
                rest.push(o);
            }
        }
        rest
    }

    /// Expands styles by name. A style that calls itself back (`a` → `b` → `a`) is not
    /// expanded again, and the total stays within `MAX_OPTIONS` even with styles that multiply.
    pub fn expand(&self, options: Vec<Opt>) -> Vec<Opt> {
        let mut out = Vec::new();
        self.expand_into(options, &mut Vec::new(), &mut out);
        out
    }

    fn expand_into(&self, options: Vec<Opt>, stack: &mut Vec<String>, out: &mut Vec<Opt>) {
        for o in options {
            if out.len() >= MAX_OPTIONS {
                return;
            }
            match (o.value.is_none(), self.defined.get(&o.key)) {
                (true, Some(def)) if !stack.contains(&o.key) => {
                    stack.push(o.key.clone());
                    self.expand_into(parse(def), stack, out);
                    stack.pop();
                }
                _ => out.push(o),
            }
        }
    }

    /// Options of a context style (`every node`, `every path`), if defined.
    pub fn context(&self, name: &str) -> Vec<Opt> {
        self.defined.get(name).map(|d| self.expand(parse(d))).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_and_styles() {
        let mut e = Styles::default();
        e.register("pointer/.style={->, very thick, color=red, >=Latex}, panel/.style={thick, fill=blue!10}");
        let o = e.expand(parse("pointer, dashed"));
        let keys: Vec<&str> = o.iter().map(|o| o.key.as_str()).collect();
        assert_eq!(keys, ["->", "very thick", "color", ">", "dashed"]);
        assert_eq!(o[2].value.as_deref(), Some("red"));
        assert_eq!(parse("minimum width=1.2cm, label={above:$x$}")[1].value.as_deref(), Some("above:$x$"));
        assert_eq!(parse("-{Stealth[length=3mm]}")[0].key, "-{Stealth[length=3mm]}");
    }
}
