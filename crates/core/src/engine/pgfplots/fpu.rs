//! Enough of pgf's floating point unit to step through a domain the way pgfplots does.
//!
//! A number is a decimal `n * 10^p`. pgf adds two of them by scaling both mantissas to [100, 1000),
//! reading them as TeX dimensions (sp), shifting the one of smaller exponent (`\divide` by ten,
//! cutting), adding and printing the sum as TeX prints a dimension; it divides with the mantissas
//! scaled to [10, 100). So sums keep about eight digits and quotients five or six, and a sampled
//! domain can end a little short of its end (`-5:5` with 25 samples ends at 4.9998401).

const UNITY: i128 = 65536;

/// A decimal number `n * 10^p`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Num {
    n: i128,
    p: i32,
}

impl Num {
    pub const ZERO: Num = Num { n: 0, p: 0 };

    /// The number written as `v` prints (its shortest decimal form).
    pub fn from_f64(v: f64) -> Option<Num> {
        if !v.is_finite() {
            return None;
        }
        let s = format!("{:e}", v);
        let (m, e) = s.split_once('e')?;
        let e: i32 = e.parse().ok()?;
        let neg = m.starts_with('-');
        let m = m.trim_start_matches('-');
        let (int, frac) = m.split_once('.').unwrap_or((m, ""));
        let digits = format!("{int}{frac}");
        if digits.len() > 30 {
            return None;
        }
        let n: i128 = digits.parse().ok()?;
        Some(Num { n: if neg { -n } else { n }, p: e - frac.len() as i32 }.trimmed())
    }

    pub fn value(self) -> f64 {
        self.n as f64 * 10f64.powi(self.p)
    }

    pub fn negated(self) -> Num {
        Num { n: -self.n, p: self.p }
    }

    pub fn is_zero(self) -> bool {
        self.n == 0
    }

    fn trimmed(mut self) -> Num {
        if self.n == 0 {
            return Num::ZERO;
        }
        while self.n % 10 == 0 {
            self.n /= 10;
            self.p += 1;
        }
        self
    }

    /// The power of ten of the leading digit.
    fn exponent(self) -> i32 {
        let mut d = 0;
        let mut m = self.n.abs();
        while m >= 10 {
            m /= 10;
            d += 1;
        }
        d + self.p
    }

    /// The mantissa scaled to `10^shift` times [1, 10), read as a dimension (sp), and the power of
    /// ten that goes with it.
    fn scaled(self, shift: i32) -> (i128, i32) {
        if self.n == 0 {
            return (0, 0);
        }
        let e = self.exponent() - shift;
        // the mantissa is n * 10^(p - e): a decimal with k = e - p digits after the point
        let k = e - self.p;
        let sp = if k <= 0 {
            self.n.abs() * 10i128.pow((-k) as u32) * UNITY
        } else if k > 30 {
            0
        } else {
            // TeX rounds the decimal digits to the nearest sp
            let d = 10i128.pow(k as u32);
            (2 * self.n.abs() * UNITY + d) / (2 * d)
        };
        (self.n.signum() * sp, e)
    }

    /// A dimension (sp) as TeX prints it, times `10^e`.
    fn printed(sp: i128, e: i32) -> Num {
        let neg = sp < 0;
        let mut s = sp.abs();
        let int = s / UNITY;
        // TeX's \the of a dimension: the fewest decimals that read back as the same sp
        let mut frac: i128 = 0;
        let mut k = 0;
        s = 10 * (s % UNITY) + 5;
        let mut delta = 10;
        loop {
            if delta > UNITY {
                s += 0x8000 - 50000;
            }
            frac = 10 * frac + s / UNITY;
            k += 1;
            s = 10 * (s % UNITY);
            delta *= 10;
            if s <= delta {
                break;
            }
        }
        let n = int * 10i128.pow(k) + frac;
        Num { n: if neg { -n } else { n }, p: e - k as i32 }.trimmed()
    }

    /// `\pgfmathfloatadd@`.
    pub fn plus(self, o: Num) -> Num {
        if self.n == 0 {
            return o;
        }
        if o.n == 0 {
            return self;
        }
        let ((mut a, ea), (mut b, eb)) = (self.scaled(2), o.scaled(2));
        let e = ea.max(eb);
        for (m, x) in [(&mut a, ea), (&mut b, eb)] {
            let d = e - x;
            // `\divide` cuts toward zero
            *m = if d > 8 { 0 } else { *m / 10i128.pow(d as u32) };
        }
        Num::printed(a + b, e)
    }

    /// `\pgfmathfloatdivide@`: the mantissas scaled to [10, 100) and divided as pgfmath divides
    /// dimensions (by a whole number: `\divide`, cutting).
    pub fn over(self, o: Num) -> Num {
        if o.n == 0 || self.n == 0 {
            return Num::ZERO;
        }
        let ((a, ea), (b, eb)) = (self.scaled(1), o.scaled(1));
        let q = if b % UNITY == 0 { a / (b / UNITY) } else { a * UNITY / b };
        Num::printed(q, ea - eb)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(v: f64) -> Num {
        Num::from_f64(v).unwrap()
    }

    fn assert_near(a: Num, b: f64) {
        assert!((a.value() - b).abs() < 1e-12, "{} != {b}", a.value());
    }

    #[test]
    fn sums_and_quotients_as_pgf() {
        // the values pgf's fpu gives (\pgfmathfloatadd@ and \pgfmathfloatdivide@)
        assert_near(n(-5.0).plus(n(0.41666)), -4.58334);
        assert_near(n(0.208333).plus(n(0.208333)), 0.41666599);
        assert_near(n(4.7916585).plus(n(0.208333)), 4.9999915);
        assert_near(n(10.0).over(n(24.0)), 0.41666);
        assert_near(n(5.0).over(n(24.0)), 0.208333);
        assert_near(n(-4.58334).plus(n(5.0)), 0.41666);
    }
}
