//! `symbolic x coords={a,b,c}` (and y, z): names that stand for 0, 1, 2… wherever a coordinate of
//! that axis is read (points, table cells, `xtick`, `xmin`), and are printed back as its tick
//! labels.

use super::keys;
use super::Engine;

/// The names of axis `c`'s symbolic coordinates (none: not symbolic).
pub fn names(e: &Engine, c: char) -> Vec<String> {
    let Some(v) = crate::keys::value(&e.tex, &format!("/pgfplots/symbolic {c} coords")) else { return Vec::new() };
    let text = crate::tex::tokens_to_string(&v);
    crate::keys::split_top_level(&text).iter().map(|s| crate::keys::strip_braces(s.trim()).trim().to_string()).filter(|s| !s.is_empty()).collect()
}

/// The number a name stands for on axis `c`.
pub fn index(e: &Engine, c: char, text: &str) -> Option<f64> {
    let t = crate::keys::strip_braces(text.trim()).trim();
    names(e, c).iter().position(|n| n == t).map(|i| i as f64)
}

/// A key's number (`xmin=tool1` on a symbolic axis is its index).
pub fn number(e: &mut Engine, c: char, key: &str) -> Option<f64> {
    let v = keys::value(e, key);
    if !v.is_empty() {
        if let Some(i) = index(e, c, &v) {
            return Some(i);
        }
    }
    keys::number(e, key)
}

/// The tick label of tick `t` on a symbolic axis: the name at that index.
pub fn label(e: &Engine, c: char, t: f64) -> Option<String> {
    let names = names(e, c);
    if names.is_empty() {
        return None;
    }
    let k = t.round();
    if (t - k).abs() > 1e-6 || k < 0.0 {
        return Some(String::new());
    }
    Some(names.get(k as usize).cloned().unwrap_or_default())
}
