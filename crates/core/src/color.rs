//! xcolor colors: names, mixes `orange!80!black`, `blue!10` (with white), `rgb`, `HTML`, `-red` (complement).

use std::collections::HashMap;

pub mod brewer;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Color {
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0 };
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0 };

    pub fn hex(&self) -> String {
        let c = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("#{:02x}{:02x}{:02x}", c(self.r), c(self.g), c(self.b))
    }

    /// The color as an xcolor expression that holds it exactly, in xcolor's extended syntax:
    /// `rgb,1:red,1;green,0.5;blue,0`.
    pub fn xcolor(&self) -> String {
        let v = |x: f64| format!("{}", (x.clamp(0.0, 1.0) * 1e4).round() / 1e4);
        format!("rgb,1:red,{};green,{};blue,{}", v(self.r), v(self.g), v(self.b))
    }

    fn mix(a: Color, b: Color, p: f64) -> Color {
        Color { r: a.r * p + b.r * (1.0 - p), g: a.g * p + b.g * (1.0 - p), b: a.b * p + b.b * (1.0 - p) }
    }
}

fn named(name: &str) -> Option<Color> {
    let (r, g, b) = match name {
        "black" => (0.0, 0.0, 0.0),
        "white" => (1.0, 1.0, 1.0),
        "red" => (1.0, 0.0, 0.0),
        "green" => (0.0, 1.0, 0.0),
        "blue" => (0.0, 0.0, 1.0),
        "cyan" => (0.0, 1.0, 1.0),
        "magenta" => (1.0, 0.0, 1.0),
        "yellow" => (1.0, 1.0, 0.0),
        "gray" => (0.5, 0.5, 0.5),
        "darkgray" => (0.25, 0.25, 0.25),
        "lightgray" => (0.75, 0.75, 0.75),
        "brown" => (0.75, 0.5, 0.25),
        "lime" => (0.75, 1.0, 0.0),
        "olive" => (0.5, 0.5, 0.0),
        "orange" => (1.0, 0.5, 0.0),
        "pink" => (1.0, 0.75, 0.75),
        "purple" => (0.75, 0.0, 0.25),
        "teal" => (0.0, 0.5, 0.5),
        "violet" => (0.5, 0.0, 0.5),
        // the most common dvipsnames
        "ForestGreen" => (0.13, 0.55, 0.13),
        "NavyBlue" => (0.0, 0.0, 0.5),
        "RoyalBlue" => (0.25, 0.41, 0.88),
        "Maroon" => (0.5, 0.0, 0.0),
        "Goldenrod" => (0.85, 0.65, 0.13),
        "Gray" => (0.5, 0.5, 0.5),
        _ => return None,
    };
    Some(Color { r, g, b })
}

/// Parses an xcolor expression. `defined` comes from `\definecolor` and `\colorlet`.
pub fn parse(expr: &str, defined: &HashMap<String, Color>) -> Option<Color> {
    // each leading `-` complements; counted in a loop, not by recursion
    let mut e = expr.trim();
    let mut complements = 0usize;
    while let Some(rest) = e.strip_prefix('-') {
        complements += 1;
        e = rest.trim_start();
    }
    if e.is_empty() {
        return None;
    }
    let color = parse_unsigned(e, defined)?;
    Some(if complements % 2 == 1 { Color { r: 1.0 - color.r, g: 1.0 - color.g, b: 1.0 - color.b } } else { color })
}

fn parse_unsigned(e: &str, defined: &HashMap<String, Color>) -> Option<Color> {
    // xcolor's extended syntax: `model,div:color,factor;color,factor…`, the colors weighted by their
    // factors and the sum divided by `div` (the model is the space of the mix: the same in rgb)
    if let Some((head, mix)) = e.split_once(':') {
        let div: f64 = head.split_once(',')?.1.trim().parse().ok().filter(|d: &f64| *d > 0.0)?;
        let mut sum = Color { r: 0.0, g: 0.0, b: 0.0 };
        for part in mix.split(';') {
            let (name, factor) = part.rsplit_once(',')?;
            let c = parse(name, defined)?;
            let f: f64 = factor.trim().parse().ok()?;
            sum = Color { r: sum.r + c.r * f, g: sum.g + c.g * f, b: sum.b + c.b * f };
        }
        return Some(Color { r: sum.r / div, g: sum.g / div, b: sum.b / div });
    }
    let parts: Vec<&str> = e.split('!').map(str::trim).collect();
    let lookup = |n: &str| defined.get(n).copied().or_else(|| named(n)).or_else(|| brewer::color(n));
    let mut color = lookup(parts[0])?;
    let mut i = 1;
    while i < parts.len() {
        let p: f64 = parts[i].parse::<f64>().ok()? / 100.0;
        let other = match parts.get(i + 1) {
            Some(n) => lookup(n)?,
            None => Color::WHITE,
        };
        color = Color::mix(color, other, p);
        i += 2;
    }
    Some(color)
}

/// `\definecolor{name}{model}{values}`.
pub fn define(model: &str, values: &str) -> Option<Color> {
    let nums: Vec<f64> = values.split(',').filter_map(|v| v.trim().parse().ok()).collect();
    match model.trim() {
        "rgb" if nums.len() == 3 => Some(Color { r: nums[0], g: nums[1], b: nums[2] }),
        "RGB" if nums.len() == 3 => Some(Color { r: nums[0] / 255.0, g: nums[1] / 255.0, b: nums[2] / 255.0 }),
        "gray" if nums.len() == 1 => Some(Color { r: nums[0], g: nums[0], b: nums[0] }),
        "cmyk" if nums.len() == 4 => {
            let k = 1.0 - nums[3];
            Some(Color { r: (1.0 - nums[0]) * k, g: (1.0 - nums[1]) * k, b: (1.0 - nums[2]) * k })
        }
        "HTML" => {
            let h = values.trim().trim_start_matches('#');
            let v = |i: usize| u8::from_str_radix(h.get(i..i + 2)?, 16).ok().map(|x| f64::from(x) / 255.0);
            Some(Color { r: v(0)?, g: v(2)?, b: v(4)? })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xcolor_mixes() {
        let d = HashMap::new();
        assert_eq!(parse("blue!10", &d).unwrap().hex(), "#e6e6ff");
        assert_eq!(parse("orange!80!black", &d).unwrap().hex(), "#cc6600");
        assert_eq!(parse("gray!20", &d).unwrap().hex(), "#e6e6e6");
        assert_eq!(define("HTML", "FF8800").unwrap().hex(), "#ff8800");
    }

    #[test]
    fn extended_syntax_holds_a_color() {
        let d = HashMap::new();
        let c = parse("orange!80!black", &d).unwrap();
        assert_eq!(c.xcolor(), "rgb,1:red,0.8;green,0.4;blue,0");
        assert_eq!(parse(&c.xcolor(), &d), Some(c));
        // weights and a divisor, as xcolor mixes them
        assert_eq!(parse("rgb,2:red,1;blue,1", &d).unwrap().hex(), "#800080");
        for bad in ["rgb:red,1", "rgb,0:red,1", "rgb,1:red", "rgb,1:nothing,1", "rgb,1:red,x"] {
            assert_eq!(parse(bad, &d), None, "{bad}");
        }
    }
}
