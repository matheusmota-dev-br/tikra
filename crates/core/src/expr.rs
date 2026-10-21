//! pgfmath expressions inside coordinates and options: `2*\R`, `sin(30)`, `sqrt(2)/2`, `\x+1`.
//! Trigonometry in degrees, as in TikZ. Units (`cm`, `pt`, `mm`) become centimeters.

use std::collections::HashMap;

use crate::Error;

/// Numeric variables (`\R`, `\x`, `x`) and declared functions (`declare function={E(\n) = (\n)^2;}`).
#[derive(Debug, Clone, Default)]
pub struct Variables {
    pub values: HashMap<String, f64>,
    /// name → (parameters without the backslash, body)
    pub functions: HashMap<String, (Vec<String>, String)>,
}

impl Variables {
    pub fn new() -> Variables {
        Variables::default()
    }
    pub fn insert(&mut self, name: String, value: f64) {
        self.values.insert(name, value);
    }
    pub fn get(&self, name: &str) -> Option<&f64> {
        self.values.get(name)
    }
}

/// Centimeters per TeX unit.
pub fn unit_cm(u: &str) -> Option<f64> {
    Some(match u {
        "cm" => 1.0,
        "mm" => 0.1,
        "pt" => 2.54 / 72.27,
        "bp" => 2.54 / 72.0,
        "in" => 2.54,
        "em" => 0.3515, // 10pt
        "ex" => 0.1515,
        "pc" => 12.0 * 2.54 / 72.27,
        _ => return None,
    })
}

/// Nesting cap (parentheses, signs, calls): `f(x) = f(x)` does not overflow the stack.
pub const MAX_DEPTH: usize = 200;

pub fn eval(text: &str, vars: &Variables) -> Result<f64, Error> {
    eval_at(text, vars, 0)
}

fn eval_at(text: &str, vars: &Variables, depth: usize) -> Result<f64, Error> {
    eval_units(text, vars, depth).map(|(v, _)| v)
}

/// The value, and whether a unit appeared in the expression (pgfmath's "units declared").
fn eval_units(text: &str, vars: &Variables, depth: usize) -> Result<(f64, bool), Error> {
    let mut p = Parser { c: text.chars().collect(), pos: 0, vars, depth, units: false };
    p.skip_spaces();
    let v = p.ternary()?;
    p.skip_spaces();
    if p.pos < p.c.len() {
        return Err(Error::Expression(format!("leftover '{}' in '{text}'", p.c[p.pos..].iter().collect::<String>())));
    }
    Ok((v, p.units))
}

/// Like `eval`, but a number without a unit is returned as is (the caller's default unit)
/// and one with a unit is converted to cm. Returns (value, had_unit).
pub fn eval_length(text: &str, vars: &Variables) -> Result<(f64, bool), Error> {
    let t = text.trim();
    // units inside the expression, each number with its own (`1cm+.5pt`)
    if let Ok((v, true)) = eval_units(t, vars, 0) {
        return Ok((v, true));
    }
    for u in ["cm", "mm", "pt", "bp", "in", "em", "ex", "pc"] {
        if let Some(n) = t.strip_suffix(u) {
            if let Ok(v) = eval(n, vars) {
                return Ok((v * unit_cm(u).unwrap(), true));
            }
        }
    }
    // a unit inside the expression (`85pt/3`): the value is a length, already in cm
    eval_units(t, vars, 0)
}

struct Parser<'a> {
    c: Vec<char>,
    pos: usize,
    vars: &'a Variables,
    depth: usize,
    /// a unit was glued to a number
    units: bool,
}

impl Parser<'_> {
    fn skip_spaces(&mut self) {
        while self.c.get(self.pos).is_some_and(|c| c.is_whitespace()) {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.c.get(self.pos).copied()
    }

    /// `c ? a : b`, then `||`, `&&` and the comparisons, which give 1 or 0, as pgfmath has them.
    fn ternary(&mut self) -> Result<f64, Error> {
        let c = self.or()?;
        self.skip_spaces();
        if self.peek() != Some('?') {
            return Ok(c);
        }
        self.pos += 1;
        let a = self.ternary()?;
        self.skip_spaces();
        if self.peek() != Some(':') {
            return Err(Error::Expression("'?' without ':'".into()));
        }
        self.pos += 1;
        let b = self.ternary()?;
        Ok(if c != 0.0 { a } else { b })
    }

    fn starts(&self, s: &str) -> bool {
        s.chars().enumerate().all(|(i, c)| self.c.get(self.pos + i) == Some(&c))
    }

    fn or(&mut self) -> Result<f64, Error> {
        let mut v = self.and()?;
        loop {
            self.skip_spaces();
            if !self.starts("||") {
                return Ok(v);
            }
            self.pos += 2;
            let w = self.and()?;
            v = f64::from(v != 0.0 || w != 0.0);
        }
    }

    fn and(&mut self) -> Result<f64, Error> {
        let mut v = self.comparison()?;
        loop {
            self.skip_spaces();
            if !self.starts("&&") {
                return Ok(v);
            }
            self.pos += 2;
            let w = self.comparison()?;
            v = f64::from(v != 0.0 && w != 0.0);
        }
    }

    fn comparison(&mut self) -> Result<f64, Error> {
        let v = self.sum()?;
        self.skip_spaces();
        for op in ["==", "!=", "<=", ">=", "<", ">"] {
            if self.starts(op) {
                self.pos += op.len();
                let w = self.sum()?;
                return Ok(f64::from(match op {
                    "==" => v == w,
                    "!=" => v != w,
                    "<=" => v <= w,
                    ">=" => v >= w,
                    "<" => v < w,
                    _ => v > w,
                }));
            }
        }
        Ok(v)
    }

    fn sum(&mut self) -> Result<f64, Error> {
        let mut v = self.product()?;
        loop {
            self.skip_spaces();
            match self.peek() {
                Some('+') => {
                    self.pos += 1;
                    v += self.product()?;
                }
                Some('-') => {
                    self.pos += 1;
                    v -= self.product()?;
                }
                _ => return Ok(v),
            }
        }
    }

    fn product(&mut self) -> Result<f64, Error> {
        let mut v = self.power()?;
        loop {
            self.skip_spaces();
            match self.peek() {
                Some('*') => {
                    self.pos += 1;
                    v *= self.power()?;
                }
                Some('/') => {
                    self.pos += 1;
                    v /= self.power()?;
                }
                _ => return Ok(v),
            }
        }
    }

    /// `a^b^c` is right-associative; read in a loop so a long `2^2^…^2` does not deepen the stack.
    fn power(&mut self) -> Result<f64, Error> {
        let mut terms = vec![self.unary()?];
        self.skip_spaces();
        while self.peek() == Some('^') {
            self.pos += 1;
            terms.push(self.unary()?);
            self.skip_spaces();
        }
        let last = terms.pop().unwrap_or_default();
        Ok(terms.into_iter().rev().fold(last, |e, base| base.powf(e)))
    }

    fn unary(&mut self) -> Result<f64, Error> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(Error::Expression(format!("nesting went past {MAX_DEPTH} levels")));
        }
        let r = self.unary_unbounded();
        self.depth -= 1;
        r
    }

    fn unary_unbounded(&mut self) -> Result<f64, Error> {
        self.skip_spaces();
        match self.peek() {
            Some('-') => {
                self.pos += 1;
                Ok(-self.unary()?)
            }
            Some('+') => {
                self.pos += 1;
                self.unary()
            }
            _ => self.atom(),
        }
    }

    fn atom(&mut self) -> Result<f64, Error> {
        self.skip_spaces();
        match self.peek() {
            Some('(') | Some('{') => {
                let close = if self.peek() == Some('(') { ')' } else { '}' };
                self.pos += 1;
                let v = self.ternary()?;
                self.skip_spaces();
                if self.peek() != Some(close) {
                    return Err(Error::Expression(format!("missing '{close}'")));
                }
                self.pos += 1;
                Ok(v)
            }
            Some(c) if c.is_ascii_digit() || c == '.' => {
                let start = self.pos;
                while self.peek().is_some_and(|c| c.is_ascii_digit() || c == '.') {
                    self.pos += 1;
                }
                // pgfmath scientific notation: 1e3, 2.5E-4 (not confused with the em and ex units)
                if matches!(self.peek(), Some('e' | 'E')) {
                    let sign = matches!(self.c.get(self.pos + 1), Some('+' | '-')) as usize;
                    if self.c.get(self.pos + 1 + sign).is_some_and(|c| c.is_ascii_digit()) {
                        self.pos += 1 + sign;
                        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                            self.pos += 1;
                        }
                    }
                }
                let n: String = self.c[start..self.pos].iter().collect();
                let mut v: f64 = n.parse().map_err(|_| Error::Expression(format!("invalid number: {n}")))?;
                // a unit glued to a number inside an expression: 2cm, 3pt
                let rest: String = self.c[self.pos..].iter().take(2).collect();
                if let Some(f) = unit_cm(&rest) {
                    if !self.c.get(self.pos + 2).is_some_and(|c| c.is_ascii_alphabetic()) {
                        self.pos += 2;
                        v *= f;
                        self.units = true;
                    }
                }
                Ok(v)
            }
            Some('\\') => {
                self.pos += 1;
                let start = self.pos;
                while self.peek().is_some_and(|c| c.is_ascii_alphabetic()) {
                    self.pos += 1;
                }
                let name: String = self.c[start..self.pos].iter().collect();
                match name.as_str() {
                    "pi" => Ok(std::f64::consts::PI),
                    _ => self.vars.get(&name).copied().ok_or(Error::Expression(format!("unknown variable: \\{name}"))),
                }
            }
            Some(c) if c.is_ascii_alphabetic() => {
                let start = self.pos;
                // digits after the first letter: atan2, log10
                while self.peek().is_some_and(|c| c.is_ascii_alphanumeric()) {
                    self.pos += 1;
                }
                let name: String = self.c[start..self.pos].iter().collect();
                if name == "pi" {
                    return Ok(std::f64::consts::PI);
                }
                if name == "e" {
                    return Ok(std::f64::consts::E);
                }
                self.skip_spaces();
                if self.peek() != Some('(') {
                    // a variable without a backslash, like the x of pgfplots functions
                    if let Some(v) = self.vars.get(&name) {
                        return Ok(*v);
                    }
                    // a function without arguments: declared (`declare function={C=4;}`) or pgfmath's
                    if let Some(body) = self.vars.functions.get(&name).filter(|f| f.0.is_empty()).map(|f| f.1.clone()) {
                        return eval_at(&body, self.vars, self.depth + 1);
                    }
                    return match name.as_str() {
                        "rand" => Ok(pseudo_random() * 2.0 - 1.0),
                        "rnd" => Ok(pseudo_random()),
                        "true" => Ok(1.0),
                        "false" => Ok(0.0),
                        _ => Err(Error::Expression(format!("function without parentheses: {name}"))),
                    };
                }
                self.pos += 1;
                let mut args = vec![self.ternary()?];
                self.skip_spaces();
                while self.peek() == Some(',') {
                    self.pos += 1;
                    args.push(self.ternary()?);
                    self.skip_spaces();
                }
                if self.peek() != Some(')') {
                    return Err(Error::Expression(format!("missing ')' in {name}(…)")));
                }
                self.pos += 1;
                if let Some((params, body)) = self.vars.functions.get(&name).cloned() {
                    let mut locals = self.vars.clone();
                    for (p, v) in params.iter().zip(&args) {
                        locals.insert(p.clone(), *v);
                    }
                    return eval_at(&body, &locals, self.depth + 1);
                }
                let g = |x: f64| x.to_radians();
                let a = args[0];
                Ok(match name.as_str() {
                    "sin" => sin_degrees(a),
                    "cos" => sin_degrees(a + 90.0),
                    "tan" => g(a).tan(),
                    "asin" => a.asin().to_degrees(),
                    "acos" => a.acos().to_degrees(),
                    "atan" => a.atan().to_degrees(),
                    "atan2" => args.get(1).map_or(0.0, |x| a.atan2(*x).to_degrees()),
                    "sqrt" => a.sqrt(),
                    "abs" => a.abs(),
                    "exp" => a.exp(),
                    "ln" => a.ln(),
                    "log10" => a.log10(),
                    "floor" => a.floor(),
                    "ceil" => a.ceil(),
                    "round" => a.round(),
                    "deg" => a.to_degrees(),
                    "rad" => a.to_radians(),
                    "min" => args.iter().copied().fold(f64::INFINITY, f64::min),
                    "max" => args.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                    "pow" => a.powf(args.get(1).copied().unwrap_or(1.0)),
                    "mod" => a % args.get(1).copied().unwrap_or(1.0),
                    "veclen" => a.hypot(args.get(1).copied().unwrap_or(0.0)),
                    other => return Err(Error::Expression(format!("unknown function: {other}"))),
                })
            }
            Some(c) => Err(Error::Expression(format!("unexpected character: '{c}'"))),
            None => Err(Error::Expression("empty expression".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pgfmath_expressions() {
        let mut v = Variables::new();
        v.insert("R".into(), 0.4);
        assert!((eval("2*\\R", &v).unwrap() - 0.8).abs() < 1e-9);
        assert!((eval("sin(30)", &v).unwrap() - 0.5).abs() < 1e-9);
        assert!((eval("-\\R + 1", &v).unwrap() - 0.6).abs() < 1e-9);
        assert!((eval("2^3", &v).unwrap() - 8.0).abs() < 1e-9);
        assert!((eval_length("10pt", &v).unwrap().0 - 0.3515).abs() < 1e-3);
    }
}

/// The sine of degrees, exact at multiples of 90 as pgfmath's (`sin(360)` is 0, not -2.4e-16).
fn sin_degrees(a: f64) -> f64 {
    let r = a.rem_euclid(360.0);
    if r == 0.0 || r == 180.0 {
        0.0
    } else if r == 90.0 {
        1.0
    } else if r == 270.0 {
        -1.0
    } else {
        a.to_radians().sin()
    }
}

thread_local! {
    static SEED: std::cell::Cell<u64> = const { std::cell::Cell::new(0x2545_F491_4F6C_DD1D) };
}

/// pgfmath's `rnd`: a number in [0, 1). Deterministic, so a figure always draws the same.
fn pseudo_random() -> f64 {
    SEED.with(|s| {
        let mut x = s.get();
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        s.set(x);
        (x >> 11) as f64 / (1u64 << 53) as f64
    })
}

/// Starts `rnd` and `rand` over: each figure draws the same numbers.
pub fn reset_random() {
    SEED.with(|s| s.set(0x2545_F491_4F6C_DD1D));
}
