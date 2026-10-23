//! pgf's functional shadings: `\pgfdeclarefunctionalshading[colors]{name}{lower left}{upper right}
//! {init}{function}`, whose color at each point of the box is what a PostScript calculator function
//! leaves from the point's coordinates (bp), and the shadings library's (`color wheel`, `bilinear
//! interpolation`). SVG has no such fill: the box is drawn as a grid of small cells, each in the
//! color at its center.

use std::rc::Rc;

use crate::canvas::{Fill, Item, Paint};
use crate::color::Color;
use crate::scene::{Point, Segment, Transform};
use crate::tex::Token;
use crate::Error;

use super::Engine;

/// Centimeters per bp.
const BP: f64 = 2.54 / 72.0;
/// Cells along each side of a shading's box.
const CELLS: usize = 64;
/// Operations one evaluation of a function may run.
const MAX_OPS: usize = 20_000;

/// A declared functional shading: its box (bp) and the TeX of its setup and function.
#[derive(Debug, Clone)]
pub struct Functional {
    ll: Point,
    ur: Point,
    init: Vec<Token>,
    code: Vec<Token>,
}

/// A shading ready to evaluate: its box and its color at a point of it.
pub struct Prepared {
    ll: Point,
    ur: Point,
    kind: Kind,
}

enum Kind {
    Program(Vec<Op>),
    /// the library's color wheels: hue around the center; saturation (white center) or value
    /// (black center) growing out to 25 bp, or neither
    Wheel {
        white_center: bool,
        black_center: bool,
    },
    /// the library's bilinear interpolation of four corner colors over the middle 50 bp
    Bilinear([Color; 4]),
}

#[derive(Debug, Clone)]
enum Op {
    Num(f64),
    Name(String),
    Proc(Rc<Vec<Op>>),
}

#[derive(Debug, Clone)]
enum Value {
    Num(f64),
    Bool(bool),
    Proc(Rc<Vec<Op>>),
}

/// PostScript code (numbers, operator names, `{…}` procedures) as operations.
fn parse(text: &str) -> Vec<Op> {
    fn block(chars: &[char], i: &mut usize) -> Vec<Op> {
        let mut out = Vec::new();
        while *i < chars.len() {
            let c = chars[*i];
            if c.is_whitespace() {
                *i += 1;
            } else if c == '{' {
                *i += 1;
                out.push(Op::Proc(Rc::new(block(chars, i))));
            } else if c == '}' {
                *i += 1;
                return out;
            } else if c == '%' {
                while *i < chars.len() && chars[*i] != '\n' {
                    *i += 1;
                }
            } else {
                let start = *i;
                while *i < chars.len() && !chars[*i].is_whitespace() && !matches!(chars[*i], '{' | '}' | '%') {
                    *i += 1;
                }
                let word: String = chars[start..*i].iter().collect();
                out.push(match word.parse::<f64>() {
                    Ok(v) => Op::Num(v),
                    Err(_) => Op::Name(word),
                });
            }
        }
        out
    }
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    while i < chars.len() {
        out.extend(block(&chars, &mut i));
    }
    out
}

/// Runs the operations on the stack; `None` when the code does something it cannot.
fn run(ops: &[Op], stack: &mut Vec<Value>, budget: &mut usize) -> Option<()> {
    fn num(stack: &mut Vec<Value>) -> Option<f64> {
        match stack.pop()? {
            Value::Num(v) => Some(v),
            Value::Bool(b) => Some(f64::from(u8::from(b))),
            Value::Proc(_) => None,
        }
    }
    fn boolean(stack: &mut Vec<Value>) -> Option<bool> {
        match stack.pop()? {
            Value::Bool(b) => Some(b),
            Value::Num(v) => Some(v != 0.0),
            Value::Proc(_) => None,
        }
    }
    for op in ops {
        *budget = budget.checked_sub(1)?;
        match op {
            Op::Num(v) => stack.push(Value::Num(*v)),
            Op::Proc(p) => stack.push(Value::Proc(p.clone())),
            Op::Name(name) => {
                let unary = |stack: &mut Vec<Value>, f: fn(f64) -> f64| -> Option<()> {
                    let a = num(stack)?;
                    stack.push(Value::Num(f(a)));
                    Some(())
                };
                let binary = |stack: &mut Vec<Value>, f: fn(f64, f64) -> f64| -> Option<()> {
                    let b = num(stack)?;
                    let a = num(stack)?;
                    stack.push(Value::Num(f(a, b)));
                    Some(())
                };
                let compare = |stack: &mut Vec<Value>, f: fn(f64, f64) -> bool| -> Option<()> {
                    let b = num(stack)?;
                    let a = num(stack)?;
                    stack.push(Value::Bool(f(a, b)));
                    Some(())
                };
                match name.as_str() {
                    "add" => binary(stack, |a, b| a + b)?,
                    "sub" => binary(stack, |a, b| a - b)?,
                    "mul" => binary(stack, |a, b| a * b)?,
                    "div" => binary(stack, |a, b| a / b)?,
                    "idiv" => binary(stack, |a, b| (a.trunc() / b.trunc()).trunc())?,
                    "mod" => binary(stack, |a, b| a.trunc() % b.trunc())?,
                    // base exponent exp
                    "exp" => binary(stack, f64::powf)?,
                    // num den atan: the angle of (den, num), from 0 to 360 degrees
                    "atan" => binary(stack, |n, d| n.atan2(d).to_degrees().rem_euclid(360.0))?,
                    "neg" => unary(stack, |a| -a)?,
                    "abs" => unary(stack, f64::abs)?,
                    "sqrt" => unary(stack, |a| a.max(0.0).sqrt())?,
                    "sin" => unary(stack, |a| a.to_radians().sin())?,
                    "cos" => unary(stack, |a| a.to_radians().cos())?,
                    "ln" => unary(stack, f64::ln)?,
                    "log" => unary(stack, f64::log10)?,
                    "floor" => unary(stack, f64::floor)?,
                    "ceiling" => unary(stack, f64::ceil)?,
                    "round" => unary(stack, |a| (a + 0.5).floor())?,
                    "truncate" | "cvi" => unary(stack, f64::trunc)?,
                    "cvr" => unary(stack, |a| a)?,
                    "eq" => compare(stack, |a, b| a == b)?,
                    "ne" => compare(stack, |a, b| a != b)?,
                    "gt" => compare(stack, |a, b| a > b)?,
                    "ge" => compare(stack, |a, b| a >= b)?,
                    "lt" => compare(stack, |a, b| a < b)?,
                    "le" => compare(stack, |a, b| a <= b)?,
                    "true" => stack.push(Value::Bool(true)),
                    "false" => stack.push(Value::Bool(false)),
                    "and" | "or" | "xor" => {
                        let b = boolean(stack)?;
                        let a = boolean(stack)?;
                        stack.push(Value::Bool(match name.as_str() {
                            "and" => a && b,
                            "or" => a || b,
                            _ => a != b,
                        }));
                    }
                    "not" => {
                        let a = boolean(stack)?;
                        stack.push(Value::Bool(!a));
                    }
                    "dup" => {
                        let a = stack.last()?.clone();
                        stack.push(a);
                    }
                    "exch" => {
                        let n = stack.len();
                        if n < 2 {
                            return None;
                        }
                        stack.swap(n - 1, n - 2);
                    }
                    "pop" => {
                        stack.pop()?;
                    }
                    "copy" => {
                        let k = num(stack)? as usize;
                        let n = stack.len();
                        if k > n {
                            return None;
                        }
                        stack.extend_from_within(n - k..);
                    }
                    "index" => {
                        let k = num(stack)? as usize;
                        let n = stack.len();
                        if k >= n {
                            return None;
                        }
                        stack.push(stack[n - 1 - k].clone());
                    }
                    "roll" => {
                        let j = num(stack)? as i64;
                        let k = num(stack)? as usize;
                        let n = stack.len();
                        if k > n {
                            return None;
                        }
                        if k > 0 {
                            let part = &mut stack[n - k..];
                            let j = j.rem_euclid(k as i64) as usize;
                            part.rotate_right(j);
                        }
                    }
                    "if" => {
                        let Value::Proc(p) = stack.pop()? else { return None };
                        if boolean(stack)? {
                            run(&p, stack, budget)?;
                        }
                    }
                    "ifelse" => {
                        let Value::Proc(no) = stack.pop()? else { return None };
                        let Value::Proc(yes) = stack.pop()? else { return None };
                        let p = if boolean(stack)? { yes } else { no };
                        run(&p, stack, budget)?;
                    }
                    _ => return None,
                }
            }
        }
    }
    Some(())
}

/// A color from hue, saturation and value (each 0…1).
fn hsv(h: f64, s: f64, v: f64) -> Color {
    let h = h.rem_euclid(1.0) * 6.0;
    let i = h.floor();
    let f = h - i;
    let (p, q, t) = (v * (1.0 - s), v * (1.0 - s * f), v * (1.0 - s * (1.0 - f)));
    let (r, g, b) = match i as i64 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    Color { r, g, b }
}

impl Prepared {
    /// The color at a point of the box (bp).
    fn color(&self, p: Point) -> Option<Color> {
        match &self.kind {
            Kind::Program(ops) => {
                let mut stack = vec![Value::Num(p.x), Value::Num(p.y)];
                let mut budget = MAX_OPS;
                run(ops, &mut stack, &mut budget)?;
                let mut c = [0.0; 3];
                for k in (0..3).rev() {
                    c[k] = match stack.pop()? {
                        Value::Num(v) => v.clamp(0.0, 1.0),
                        _ => return None,
                    };
                }
                Some(Color { r: c[0], g: c[1], b: c[2] })
            }
            Kind::Wheel { white_center, black_center } => {
                let hue = if p.x.abs() + p.y.abs() >= 0.0001 { p.x.atan2(p.y).to_degrees().rem_euclid(360.0) / 360.0 } else { p.x };
                let r = ((p.x * p.x + p.y * p.y).sqrt() / 25.0).min(1.0);
                let (s, v) = match (white_center, black_center) {
                    (true, _) => (r, 1.0),
                    (_, true) => (1.0, r),
                    _ => (1.0, 1.0),
                };
                Some(hsv(hue, s, v))
            }
            Kind::Bilinear([ll, lr, ul, ur]) => {
                let (u, v) = (((p.x - 25.0) / 50.0).clamp(0.0, 1.0), ((p.y - 25.0) / 50.0).clamp(0.0, 1.0));
                let mix = |a: f64, b: f64, c: f64, d: f64| ((a * (1.0 - u) + b * u) * (1.0 - v) + (c * (1.0 - u) + d * u) * v).clamp(0.0, 1.0);
                Some(Color { r: mix(ll.r, lr.r, ul.r, ur.r), g: mix(ll.g, lr.g, ul.g, ur.g), b: mix(ll.b, lr.b, ul.b, ur.b) })
            }
        }
    }

    /// The box's center (bp).
    pub fn center(&self) -> Point {
        self.ll.lerp(self.ur, 0.5)
    }

    /// The box as cells of its colors, placed on the canvas by `place` (from the box's bp).
    pub fn cells(&self, place: &Transform) -> Vec<Item> {
        let (w, h) = ((self.ur.x - self.ll.x) / CELLS as f64, (self.ur.y - self.ll.y) / CELLS as f64);
        let mut items = Vec::with_capacity(CELLS * CELLS);
        for j in 0..CELLS {
            for i in 0..CELLS {
                let at = Point::new(self.ll.x + (i as f64 + 0.5) * w, self.ll.y + (j as f64 + 0.5) * h);
                let Some(color) = self.color(at) else { continue };
                // a little over the next cells, so that no seam shows between them
                let (x0, y0) = (self.ll.x + i as f64 * w, self.ll.y + j as f64 * h);
                let (x1, y1) = (x0 + w * if i + 1 < CELLS { 1.3 } else { 1.0 }, y0 + h * if j + 1 < CELLS { 1.3 } else { 1.0 });
                let corners = [Point::new(x0, y0), Point::new(x1, y0), Point::new(x1, y1), Point::new(x0, y1)].map(|p| place.apply(p));
                let mut segments = vec![Segment::MoveTo(corners[0])];
                segments.extend(corners[1..].iter().map(|p| Segment::LineTo(*p)));
                segments.push(Segment::Close);
                items.push(Item::Path { segments, fill: Some(Fill { paint: Paint::Solid(color), opacity: 1.0, even_odd: false }), stroke: None });
            }
        }
        items
    }

    /// The transformation from the box's bp to the canvas that spreads its middle 50 bp over the
    /// box `lo`–`hi`, turned by `angle`, as `\pgfshadepath` fits a shading.
    pub fn fit(&self, lo: Point, hi: Point, angle: f64) -> Transform {
        let (w, h) = (hi.x - lo.x, hi.y - lo.y);
        let (s, co) = angle.to_radians().sin_cos();
        let (s, co) = (s.abs(), co.abs());
        let sx = (w * co + h * s) / (co + s) / 50.0;
        let sy = (w * s + h * co) / (co + s) / 50.0;
        let c = lo.lerp(hi, 0.5);
        let center = self.center();
        Transform::translation(c.x, c.y).compose(&Transform::rotation(angle)).compose(&Transform::scaling(sx, sy)).compose(&Transform::translation(-center.x, -center.y))
    }
}

impl Functional {
    /// The box's lower left and upper right corners (bp).
    pub fn corners(&self) -> (Point, Point) {
        (self.ll, self.ur)
    }
}

impl Engine {
    /// `\pgfdeclarefunctionalshading[colors]{name}{lower left}{upper right}{init}{function}`.
    pub fn declare_functional_shading(&mut self) -> Result<(), Error> {
        let _ = self.optional()?;
        let name = self.argument_string()?;
        let ll = self.point_arg()?;
        let ur = self.point_arg()?;
        let init = self.argument()?;
        let code = self.argument()?;
        let (ll, ur) = (ll.scale(1.0 / BP), ur.scale(1.0 / BP));
        self.functional_shadings.insert(name, Functional { ll, ur, init, code });
        Ok(())
    }

    /// `\pgfshadecolortorgb{color}{\macro}`: `\macro` is the color's `r g b`, and `\macrored`,
    /// `\macrogreen`, `\macroblue` its components, each with a space after it (the space after a
    /// command's name is gone, the one inside it stays).
    pub fn shade_color_to_rgb(&mut self) -> Result<(), Error> {
        let color = self.argument_string()?;
        let target = self.argument()?;
        let c = self.color(&color).unwrap_or(Color::BLACK);
        let Some(Token::Cs(name)) = target.into_iter().find(|t| matches!(t, Token::Cs(_))) else { return Ok(()) };
        let number = |v: f64| format!("{} ", super::pgf_number(v));
        let define = |e: &mut Engine, n: &str, text: String| e.tex.set_meaning(&Token::cs(n), super::macro_tokens(super::tikra_tokens(&text)), false);
        define(self, &name, format!("{}{}{}", number(c.r), number(c.g), number(c.b)));
        define(self, &format!("{name}red"), number(c.r));
        define(self, &format!("{name}green"), number(c.g));
        define(self, &format!("{name}blue"), number(c.b));
        Ok(())
    }

    /// The functional shading `name` (declared, or the library's) as it is now: its setup run, its
    /// function read with the colors and macros of the moment.
    pub fn functional_shading(&mut self, name: &str) -> Option<Prepared> {
        let square = (Point::new(-50.0, -50.0), Point::new(50.0, 50.0));
        let kind = match name {
            "color wheel" | "color wheel white center" | "color wheel black center" => {
                return Some(Prepared {
                    ll: square.0,
                    ur: square.1,
                    kind: Kind::Wheel { white_center: name.ends_with("white center"), black_center: name.ends_with("black center") },
                })
            }
            "bilinear interpolation" if !self.functional_shadings.contains_key(name) => {
                let c = |e: &Engine, n: &str| e.color(n).unwrap_or(Color::WHITE);
                let corners = [c(self, "lower left"), c(self, "lower right"), c(self, "upper left"), c(self, "upper right")];
                return Some(Prepared { ll: Point::default(), ur: Point::new(100.0, 100.0), kind: Kind::Bilinear(corners) });
            }
            _ => {
                let f = self.functional_shadings.get(name)?.clone();
                self.begin_group();
                let text = self.run_tokens(f.init).and_then(|()| self.tex.expand_to_string(f.code));
                self.end_group();
                let text = text.ok()?;
                (Kind::Program(parse(&text)), f.ll, f.ur)
            }
        };
        let (kind, ll, ur) = kind;
        Some(Prepared { ll, ur, kind })
    }
}
