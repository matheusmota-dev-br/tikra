//! The scene: what is left of the TikZ after reading it. Paths (segments and curves) and nodes (text in a shape).
//! Everything in centimeters, y pointing up, as in TikZ.

use crate::color::Color;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }
    pub fn plus(self, o: Point) -> Point {
        Point::new(self.x + o.x, self.y + o.y)
    }
    pub fn minus(self, o: Point) -> Point {
        Point::new(self.x - o.x, self.y - o.y)
    }
    pub fn scale(self, k: f64) -> Point {
        Point::new(self.x * k, self.y * k)
    }
    pub fn norm(self) -> f64 {
        self.x.hypot(self.y)
    }
    /// The point at fraction `t` of the way to `o`.
    pub fn lerp(self, o: Point, t: f64) -> Point {
        self.plus(o.minus(self).scale(t))
    }
}

/// Affine transform: x' = a·x + c·y + e, y' = b·x + d·y + f.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl Default for Transform {
    fn default() -> Self {
        Transform { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: 0.0, f: 0.0 }
    }
}

impl Transform {
    pub fn apply(&self, p: Point) -> Point {
        Point::new(self.a * p.x + self.c * p.y + self.e, self.b * p.x + self.d * p.y + self.f)
    }
    /// Applies only the linear part (for relative offsets).
    pub fn linear(&self, p: Point) -> Point {
        Point::new(self.a * p.x + self.c * p.y, self.b * p.x + self.d * p.y)
    }
    /// `self` after `o` (`o` is applied first).
    pub fn compose(&self, o: &Transform) -> Transform {
        Transform {
            a: self.a * o.a + self.c * o.b,
            b: self.b * o.a + self.d * o.b,
            c: self.a * o.c + self.c * o.d,
            d: self.b * o.c + self.d * o.d,
            e: self.a * o.e + self.c * o.f + self.e,
            f: self.b * o.e + self.d * o.f + self.f,
        }
    }
    pub fn scaling(sx: f64, sy: f64) -> Transform {
        Transform { a: sx, d: sy, ..Default::default() }
    }
    pub fn translation(dx: f64, dy: f64) -> Transform {
        Transform { e: dx, f: dy, ..Default::default() }
    }
    pub fn rotation(degrees: f64) -> Transform {
        let (s, c) = degrees.to_radians().sin_cos();
        Transform { a: c, b: s, c: -s, d: c, ..Default::default() }
    }
    /// Average scale factor (for circle radii under `scale`).
    pub fn factor(&self) -> f64 {
        (self.a * self.d - self.b * self.c).abs().sqrt()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Segment {
    MoveTo(Point),
    LineTo(Point),
    CurveTo(Point, Point, Point),
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tip {
    /// TikZ's default `>`, `to`.
    Default,
    Latex,
    Stealth,
    Bar,
    Circle,
    /// `->>`: two default tips, one behind the other.
    Double,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    /// Stroke color; None = no outline.
    pub stroke: Option<Color>,
    pub fill: Option<Color>,
    /// Hatching (`pattern=north east lines`) and its color.
    pub pattern: Option<(String, Color)>,
    /// Line width in cm.
    pub line_width: f64,
    /// Dash pattern in cm (on, off, …).
    pub dash: Option<Vec<f64>>,
    pub stroke_opacity: f64,
    pub fill_opacity: f64,
    pub rounded_corners: f64,
    pub start_tip: Option<Tip>,
    pub end_tip: Option<Tip>,
    /// Arrow tip scale (`scale=` inside `{Latex[scale=1.5]}`, `length=`).
    pub tip_scale: f64,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            stroke: None,
            fill: None,
            pattern: None,
            line_width: 0.4 * crate::PT,
            dash: None,
            stroke_opacity: 1.0,
            fill_opacity: 1.0,
            rounded_corners: 0.0,
            start_tip: None,
            end_tip: None,
            tip_scale: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub segments: Vec<Segment>,
    pub style: Style,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    Rectangle,
    Circle,
    Ellipse,
    /// `diamond`
    Diamond,
    /// `coordinate`: a point with no size.
    Coordinate,
}

/// A typeset label: mathra's boxes, one per line, and the font scale.
#[derive(Debug, Clone)]
pub struct Label {
    pub lines: Vec<mathra::layout::MathBox>,
    /// Centimeters per mathra font unit.
    pub scale: f64,
    pub color: Color,
    pub align: char,
    /// Distance between baselines, in cm.
    pub line_height: f64,
}

impl Label {
    pub fn width(&self) -> f64 {
        self.lines.iter().map(|c| c.width).fold(0.0f32, f32::max) as f64 * self.scale
    }
    pub fn height(&self) -> f64 {
        self.lines.first().map_or(0.0, |c| c.height as f64 * self.scale)
    }
    pub fn depth(&self) -> f64 {
        let n = self.lines.len();
        if n == 0 {
            return 0.0;
        }
        self.lines[n - 1].depth as f64 * self.scale + (n - 1) as f64 * self.line_height
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    /// Center of the shape.
    pub center: Point,
    pub shape: Shape,
    /// Half width and half height of the shape (circle: both equal the radius).
    pub half_width: f64,
    pub half_height: f64,
    pub label: Option<Label>,
    pub style: Style,
    pub rotation: f64,
}

impl Node {
    /// The border point at `angle` (degrees), as in `(A.30)`.
    pub fn border_at_angle(&self, angle: f64) -> Point {
        let (s, c) = angle.to_radians().sin_cos();
        self.border_toward(Point::new(c, s))
    }

    /// The border point in direction `d` from the center.
    pub fn border_toward(&self, d: Point) -> Point {
        let n = d.norm();
        if n < 1e-12 {
            return self.center;
        }
        let u = d.scale(1.0 / n);
        let t = match self.shape {
            Shape::Rectangle => {
                let tx = if u.x.abs() > 1e-12 { self.half_width / u.x.abs() } else { f64::INFINITY };
                let ty = if u.y.abs() > 1e-12 { self.half_height / u.y.abs() } else { f64::INFINITY };
                tx.min(ty)
            }
            Shape::Circle | Shape::Ellipse => {
                // zero-radius circle (inner sep=0, no text): the border is the center
                let (a, b) = (self.half_width.max(1e-12), self.half_height.max(1e-12));
                1.0 / ((u.x / a).powi(2) + (u.y / b).powi(2)).sqrt()
            }
            Shape::Diamond => 1.0 / (u.x.abs() / self.half_width.max(1e-12) + u.y.abs() / self.half_height.max(1e-12)),
            Shape::Coordinate => return self.center,
        };
        self.center.plus(u.scale(t))
    }

    /// Named anchors: north, south east, center, base…
    pub fn anchor(&self, name: &str) -> Option<Point> {
        let (w, h) = (self.half_width, self.half_height);
        let c = self.center;
        // baseline of the first text line, where the renderer puts it (render::label_svg)
        let base = self.label.as_ref().map_or(c.y, |r| c.y + (r.height() + r.depth()) / 2.0 - r.height());
        let corner = |sx: f64, sy: f64| -> Point {
            match self.shape {
                Shape::Rectangle | Shape::Coordinate => c.plus(Point::new(sx * w, sy * h)),
                _ => self.border_toward(Point::new(sx * w.max(1e-9), sy * h.max(1e-9))),
            }
        };
        Some(match name.trim() {
            "center" | "" => c,
            "north" => c.plus(Point::new(0.0, h)),
            "south" => c.plus(Point::new(0.0, -h)),
            "east" => c.plus(Point::new(w, 0.0)),
            "west" => c.plus(Point::new(-w, 0.0)),
            "north east" => corner(1.0, 1.0),
            "north west" => corner(-1.0, 1.0),
            "south east" => corner(1.0, -1.0),
            "south west" => corner(-1.0, -1.0),
            "base" | "mid" => Point::new(c.x, base),
            "base west" | "mid west" => Point::new(c.x - w, base),
            "base east" | "mid east" => Point::new(c.x + w, base),
            // where the text baseline starts, without the inner sep
            "text" => Point::new(c.x - self.label.as_ref().map_or(0.0, |r| r.width()) / 2.0, base),
            other => {
                let angle: f64 = other.parse().ok()?;
                return Some(self.border_at_angle(angle));
            }
        })
    }
}

#[derive(Debug, Clone)]
pub enum Element {
    Path(Path),
    Node(Node),
    /// Clip for the following elements of the same scope.
    Clip(Vec<Segment>),
    EndClip,
}

#[derive(Debug, Clone, Default)]
pub struct Scene {
    pub elements: Vec<Element>,
    /// Extra margin around (the `border` of `standalone`), in cm.
    pub margin: f64,
}
