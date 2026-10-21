//! Colormaps, as `\pgfplotscreatecolormap` reads them: `color(0cm)=(blue); rgb255(1cm)=(0,0,255)
//! gray(2cm)=(0.5) rgb=(0.2,0.3,0.4) [1cm]…`, positions in cm (each entry 1cm, or the step in
//! brackets, after the previous one when not given), colors interpolated linearly in RGB.

use std::collections::HashMap;

use crate::color::{self, Color};

#[derive(Debug, Clone, PartialEq)]
pub struct Colormap {
    /// Positions (increasing) and colors.
    pub stops: Vec<(f64, Color)>,
}

impl Colormap {
    /// Reads a colormap specification; None when it has no color.
    pub fn parse(spec: &str, colors: &HashMap<String, Color>) -> Option<Colormap> {
        let s: Vec<char> = spec.chars().collect();
        let mut i = 0;
        let mut step = 1.0;
        let mut pos: Option<f64> = None;
        let mut stops = Vec::new();
        let skip = |i: &mut usize| {
            while *i < s.len() && (s[*i].is_whitespace() || s[*i] == ';') {
                *i += 1;
            }
        };
        let group = |i: &mut usize, open: char, close: char| -> Option<String> {
            if s.get(*i) != Some(&open) {
                return None;
            }
            let mut depth = 0;
            let start = *i + 1;
            while *i < s.len() {
                if s[*i] == open {
                    depth += 1;
                } else if s[*i] == close {
                    depth -= 1;
                    if depth == 0 {
                        let v: String = s[start..*i].iter().collect();
                        *i += 1;
                        return Some(v);
                    }
                }
                *i += 1;
            }
            None
        };
        loop {
            skip(&mut i);
            if i >= s.len() {
                break;
            }
            if s[i] == '[' {
                let inner = group(&mut i, '[', ']')?;
                step = length_cm(&inner).unwrap_or(1.0);
                continue;
            }
            let start = i;
            while i < s.len() && (s[i].is_alphanumeric() || s[i] == ' ') && s[i] != '(' && s[i] != '=' {
                i += 1;
            }
            let model: String = s[start..i].iter().collect::<String>().trim().to_string();
            if model.is_empty() {
                i += 1;
                continue;
            }
            let at = if s.get(i) == Some(&'(') { group(&mut i, '(', ')').and_then(|p| length_cm(&p)) } else { None };
            skip(&mut i);
            if s.get(i) != Some(&'=') {
                break;
            }
            i += 1;
            skip(&mut i);
            let value = group(&mut i, '(', ')')?;
            let p = at.unwrap_or_else(|| pos.map_or(0.0, |q| q + step));
            pos = Some(p);
            let c = match model.as_str() {
                "color" => color::parse(value.trim(), colors),
                "rgb" => rgb(&value, 1.0),
                "rgb255" => rgb(&value, 255.0),
                "gray" => value.trim().parse::<f64>().ok().map(|g| Color { r: g, g, b: g }),
                "cmyk" => {
                    let v: Vec<f64> = value.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                    (v.len() == 4).then(|| Color { r: (1.0 - v[0]) * (1.0 - v[3]), g: (1.0 - v[1]) * (1.0 - v[3]), b: (1.0 - v[2]) * (1.0 - v[3]) })
                }
                _ => None,
            };
            if let Some(c) = c {
                stops.push((p, c));
            }
        }
        if stops.is_empty() {
            return None;
        }
        Some(Colormap { stops })
    }

    /// The color at `t` from 0 to 1.
    pub fn at(&self, t: f64) -> Color {
        let first = self.stops[0];
        let last = self.stops[self.stops.len() - 1];
        let t = if t.is_finite() { t.clamp(0.0, 1.0) } else { 0.0 };
        let p = first.0 + t * (last.0 - first.0);
        for w in self.stops.windows(2) {
            let ((a, ca), (b, cb)) = (w[0], w[1]);
            if p <= b + 1e-12 {
                let f = if b > a { (p - a) / (b - a) } else { 0.0 };
                return mix(ca, cb, f);
            }
        }
        last.1
    }

    /// Piecewise constant access (`colormap access=const`): the color of the interval `t` is in.
    pub fn at_const(&self, t: f64) -> Color {
        let n = self.stops.len();
        let k = ((t.clamp(0.0, 1.0) * n as f64).floor() as usize).min(n - 1);
        self.stops[k].1
    }

    /// Color stops in [0, 1], for shadings.
    pub fn normalized(&self) -> Vec<(f64, Color)> {
        let a = self.stops[0].0;
        let b = self.stops[self.stops.len() - 1].0;
        self.stops.iter().map(|(p, c)| (if b > a { (p - a) / (b - a) } else { 0.0 }, *c)).collect()
    }
}

pub fn mix(a: Color, b: Color, f: f64) -> Color {
    Color { r: a.r + (b.r - a.r) * f, g: a.g + (b.g - a.g) * f, b: a.b + (b.b - a.b) * f }
}

fn rgb(v: &str, scale: f64) -> Option<Color> {
    let v: Vec<f64> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
    (v.len() == 3).then(|| Color { r: v[0] / scale, g: v[1] / scale, b: v[2] / scale })
}

fn length_cm(s: &str) -> Option<f64> {
    crate::expr::eval_length(s.trim(), &crate::expr::Variables::new()).ok().map(|(v, unit)| if unit { v } else { v * crate::PT })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hot_and_steps() {
        let m = Colormap::parse("color(0cm)=(blue); color(1cm)=(yellow); color(2cm)=(orange); color(3cm)=(red)", &HashMap::new()).unwrap();
        assert_eq!(m.stops.len(), 4);
        assert_eq!(m.at(0.0), Color { r: 0.0, g: 0.0, b: 1.0 });
        assert_eq!(m.at(1.0), Color { r: 1.0, g: 0.0, b: 0.0 });
        let m = Colormap::parse("rgb=(0,0,0) rgb=(1,1,1)", &HashMap::new()).unwrap();
        assert!((m.at(0.5).r - 0.5).abs() < 1e-9);
        let m = Colormap::parse("[1cm]rgb255(0cm)=(0,0,0) rgb255(3cm)=(255,0,0) rgb255(6cm)=(255,255,0)", &HashMap::new()).unwrap();
        assert!((m.at(0.5).r - 1.0).abs() < 1e-9);
    }
}
