//! pgf's node shapes, ported from `pgfmoduleshapes.code.tex` and the `shapes.*` libraries.
//!
//! A shape is computed once, when its node is made: from the sizes of the node's text boxes (one
//! per node part), `inner sep`, the minimum sizes, `outer sep` and its own keys, it gives a
//! [`Geometry`] in the node's coordinate system (the origin of the `text` part's box at (0, 0)): its
//! named anchors, the border on which lines to the node stop, where each part's text goes, and the
//! paths it draws. As in pgf, anchors and the border include `outer sep`; the paths do not.

mod basic;
mod geometric;

use crate::scene::{Point, Segment};

/// Width, height and depth of a node part's text box, in cm.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BoxSize {
    pub wd: f64,
    pub ht: f64,
    pub dp: f64,
}

/// The values a shape reads from pgf keys: lengths in cm.
pub trait Keys {
    /// A length key (`/pgf/star point height`), or `default` if unset or unreadable.
    fn length(&mut self, key: &str, default: f64) -> f64;
    /// A number key (`/pgf/star points`), evaluated.
    fn number(&mut self, key: &str, default: f64) -> f64;
    /// The text of a key, expanded (`/pgf/cylinder end fill`).
    fn text(&mut self, key: &str) -> String;
    /// A TeX conditional set by an `.is if` key: `pgfshapeborderusesincircle` for `\ifpgf…`.
    fn flag(&mut self, name: &str) -> bool;
}

/// What a shape is computed from.
pub struct Input<'a> {
    /// the node parts' boxes, `text` first; a part not given has an empty box
    pub parts: &'a [(String, BoxSize)],
    pub inner_xsep: f64,
    pub inner_ysep: f64,
    pub minimum_width: f64,
    pub minimum_height: f64,
    pub outer_xsep: f64,
    pub outer_ysep: f64,
    /// `/pgf/aspect` (`\pgfshapeaspect`)
    pub aspect: f64,
    /// the node font's ex, for the `mid` anchors (at .5ex)
    pub ex: f64,
    pub line_width: f64,
    pub keys: &'a mut dyn Keys,
}

impl Input<'_> {
    /// The box of a node part (empty if the text does not have it).
    pub fn part(&self, name: &str) -> BoxSize {
        self.parts.iter().find(|(n, _)| n == name).map(|(_, b)| *b).unwrap_or_default()
    }

    /// The `text` part's box (`\pgfnodeparttextbox`).
    pub fn text(&self) -> BoxSize {
        self.part("text")
    }

    /// The larger of the two outer seps, which most shapes of the libraries use.
    pub fn outer_sep(&self) -> f64 {
        self.outer_xsep.max(self.outer_ysep)
    }

    /// pgf's `\centerpoint` of most shapes: the middle of the text box.
    pub fn center_point(&self) -> Point {
        let t = self.text();
        Point::new(t.wd / 2.0, (t.ht - t.dp) / 2.0)
    }

    /// `\basepoint`: the middle of the text's baseline.
    pub fn base_point(&self) -> Point {
        Point::new(self.text().wd / 2.0, 0.0)
    }

    /// `\midpoint`: half an ex above the base point.
    pub fn mid_point(&self) -> Point {
        Point::new(self.text().wd / 2.0, 0.5 * self.ex)
    }

    /// Half the text box plus inner sep: the `x` and `y` most shapes start from.
    pub fn half_with_inner(&self) -> (f64, f64) {
        let t = self.text();
        (t.wd / 2.0 + self.inner_xsep, (t.ht + t.dp) / 2.0 + self.inner_ysep)
    }

    pub fn number(&mut self, key: &str, default: f64) -> f64 {
        self.keys.number(key, default)
    }

    pub fn length(&mut self, key: &str, default: f64) -> f64 {
        self.keys.length(key, default)
    }

    /// `/pgf/shape border rotate`, rounded as the shapes without `shape border uses incircle` do:
    /// to a multiple of 90 in [0, 360).
    pub fn border_rotation(&mut self) -> (f64, bool) {
        let incircle = self.keys.flag("pgfshapeborderusesincircle");
        let r = self.number("/pgf/shape border rotate", 0.0);
        if incircle {
            (r, true)
        } else {
            // \pgfmathmod then integer arithmetic: (r mod 360 + 45) / 90 * 90, truncated
            let m = r % 360.0;
            let mut k = ((m.trunc() as i64 + 45) / 90) * 90;
            if k < 0 {
                k += 360;
            }
            (k as f64, false)
        }
    }
}

/// Where lines to the node stop: the border, with `outer sep`.
#[derive(Debug, Clone)]
pub enum Border {
    /// the center itself (coordinate)
    Point,
    /// a rectangle around the center, by its half sizes (`\pgfpointborderrectangle`)
    Rectangle(Point),
    /// an ellipse around the center, by its radii (`\pgfpointborderellipse`)
    Ellipse(Point),
    /// a closed outline: the border point toward a direction is where the ray from the center
    /// leaves it
    Outline(Vec<Point>),
}

/// The layer a shape's path is drawn in, as pgf's shape declarations name them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    BehindBackground,
    Background,
    BeforeBackground,
    BehindForeground,
    Foreground,
    BeforeForeground,
}

/// How a shape's path is used.
#[derive(Debug, Clone, PartialEq)]
pub enum Usage {
    /// as the node says (`draw`, `fill`, a pattern): the background and foreground paths
    Node,
    /// filled with a color (a color expression, as `cylinder end fill=red!20`)
    Fill(String),
}

#[derive(Debug, Clone)]
pub struct ShapePath {
    pub layer: Layer,
    pub segments: Vec<Segment>,
    pub usage: Usage,
}

/// A shape, computed for one node, in the node's coordinate system.
#[derive(Debug, Clone)]
pub struct Geometry {
    /// the `center` anchor: borders are found from it
    pub center: Point,
    /// the named anchors
    pub anchors: Vec<(String, Point)>,
    pub border: Border,
    /// where the origin of each node part's box goes
    pub parts: Vec<(String, Point)>,
    pub paths: Vec<ShapePath>,
    /// the background is made of straight lines only, so `rounded corners` apply to it
    pub roundable: bool,
}

impl Geometry {
    /// A geometry with its center and border; anchors and paths are added by the shape.
    pub fn new(center: Point, border: Border) -> Geometry {
        Geometry { center, anchors: Vec::new(), border, parts: vec![("text".into(), Point::new(0.0, 0.0))], paths: Vec::new(), roundable: false }
    }

    pub fn anchor_at(&mut self, name: &str, p: Point) {
        self.anchors.push((name.to_string(), p));
    }

    /// Adds a path, in a layer.
    pub fn path(&mut self, layer: Layer, segments: Vec<Segment>, usage: Usage) {
        self.paths.push(ShapePath { layer, segments, usage });
    }

    pub fn background(&mut self, segments: Vec<Segment>) {
        self.path(Layer::Background, segments, Usage::Node);
    }

    /// The border point in direction `d` from the center.
    pub fn border_point(&self, d: Point) -> Point {
        let c = self.center;
        let n = d.norm();
        if n < 1e-12 {
            return c;
        }
        match &self.border {
            Border::Point => c,
            Border::Rectangle(h) => c.plus(border_rectangle(d, *h)),
            Border::Ellipse(r) => c.plus(border_ellipse(d, *r)),
            Border::Outline(o) => ray_exit(c, d, o).unwrap_or(c),
        }
    }

    /// A named anchor, or a number: the border point at that angle.
    pub fn anchor(&self, name: &str) -> Option<Point> {
        let name = normalize(name);
        if name.is_empty() {
            return Some(self.center);
        }
        if let Some((_, p)) = self.anchors.iter().find(|(n, _)| *n == name) {
            return Some(*p);
        }
        if name == "center" {
            return Some(self.center);
        }
        if let Some((_, p)) = self.parts.iter().find(|(n, _)| *n == name) {
            return Some(*p);
        }
        let angle: f64 = name.parse().ok()?;
        Some(self.border_point(polar(angle, 1.0)))
    }

    /// Whether the shape has an anchor of this name (not counting angles).
    pub fn has_anchor(&self, name: &str) -> bool {
        let name = normalize(name);
        self.anchors.iter().any(|(n, _)| *n == name) || self.parts.iter().any(|(n, _)| *n == name)
    }
}

/// Spaces collapsed, as TeX reads anchor names.
fn normalize(name: &str) -> String {
    name.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The keys of the shapes, as the shape libraries define them (run after TikZ's own definitions).
pub const DEFINITIONS: &str = r"
\newif\ifpgfshapeborderusesincircle
\newif\ifpgf@lib@shapes@starouterradiususesratio
\pgf@lib@shapes@starouterradiususesratiotrue
\newif\ifpgf@lib@sh@trapeziumstretches
\newif\ifpgf@lib@sh@trapeziumstretchesbody
\newif\ifpgf@lib@sh@isosceslestrianglestretches
\newif\ifpgfcylinderusescustomfill
\def\tikra@kiteangles#1and#2and#3\@@{\pgfkeys{/pgf/kite upper vertex angle={#1},/pgf/kite lower vertex angle={#2}}}
\pgfkeys{/pgf/.cd,
  shape border uses incircle/.is if=pgfshapeborderusesincircle,
  shape border rotate/.initial=0,
  star points/.initial=5,
  star point height/.initial=.5cm,
  star point height/.code={\pgfkeyssetvalue{/pgf/star point height}{#1}\pgf@lib@shapes@starouterradiususesratiofalse},
  star point ratio/.initial=1.5,
  star point ratio/.code={\pgfkeyssetvalue{/pgf/star point ratio}{#1}\pgf@lib@shapes@starouterradiususesratiotrue},
  star rotate/.style={/pgf/shape border rotate={#1}},
  regular polygon sides/.initial=5,
  regular polygon rotate/.style={/pgf/shape border rotate={#1}},
  trapezium left angle/.initial=60,
  trapezium right angle/.initial=60,
  trapezium angle/.style={/pgf/trapezium left angle={#1},/pgf/trapezium right angle={#1}},
  trapezium stretches/.is if=pgf@lib@sh@trapeziumstretches,
  trapezium stretches body/.is if=pgf@lib@sh@trapeziumstretchesbody,
  isosceles triangle apex angle/.initial=45,
  isosceles triangle stretches/.is if=pgf@lib@sh@isosceslestrianglestretches,
  kite upper vertex angle/.initial=120,
  kite lower vertex angle/.initial=60,
  kite vertex angles/.code={\tikra@kiteangles#1 and #1 and\@@},
  dart tip angle/.initial=45,
  dart tail angle/.initial=135,
  circular sector angle/.initial=60,
  cylinder uses custom fill/.is if=pgfcylinderusescustomfill,
  cylinder end fill/.initial=white,
  cylinder body fill/.initial=white,
}
";

/// Every shape tikra knows, by pgf's name.
pub fn is_shape(name: &str) -> bool {
    NAMES.contains(&normalize(name).as_str())
}

const NAMES: &[&str] = &[
    "rectangle",
    "circle",
    "coordinate",
    "ellipse",
    "diamond",
    "star",
    "regular polygon",
    "trapezium",
    "semicircle",
    "isosceles triangle",
    "kite",
    "dart",
    "circular sector",
    "cylinder",
];

/// Computes the shape `name` for a node; None if the shape is not known.
pub fn compute(name: &str, input: &mut Input) -> Option<Geometry> {
    Some(match normalize(name).as_str() {
        "rectangle" => basic::rectangle(input),
        "circle" => basic::circle(input),
        "coordinate" => basic::coordinate(input),
        "ellipse" => basic::ellipse(input),
        "diamond" => basic::diamond(input),
        "star" => geometric::star(input),
        "regular polygon" => geometric::regular_polygon(input),
        "trapezium" => geometric::trapezium(input),
        "semicircle" => geometric::semicircle(input),
        "isosceles triangle" => geometric::isosceles_triangle(input),
        "kite" => geometric::kite(input),
        "dart" => geometric::dart(input),
        "circular sector" => geometric::circular_sector(input),
        "cylinder" => geometric::cylinder(input),
        _ => return None,
    })
}

// ------------------------------------------------------------------ geometry helpers

/// `\pgfqpointpolar`: the point at `angle` degrees and distance `r`.
pub fn polar(angle: f64, r: f64) -> Point {
    let (s, c) = angle.to_radians().sin_cos();
    Point::new(r * c, r * s)
}

/// `\pgfmathrotatepointaround`: `p` rotated by `angle` degrees around `c`.
pub fn rotate_around(p: Point, c: Point, angle: f64) -> Point {
    if angle == 0.0 {
        return p;
    }
    let (s, co) = angle.to_radians().sin_cos();
    let d = p.minus(c);
    c.plus(Point::new(d.x * co - d.y * s, d.x * s + d.y * co))
}

/// `\pgfmathanglebetweenpoints`: the angle of `q` seen from `p`, in [0, 360).
pub fn angle_between(p: Point, q: Point) -> f64 {
    let a = (q.y - p.y).atan2(q.x - p.x).to_degrees();
    if a < 0.0 {
        a + 360.0
    } else {
        a
    }
}

/// `\pgfpointintersectionoflines`: where the line through `a` and `b` meets the line through `c`
/// and `d`.
pub fn intersect_lines(a: Point, b: Point, c: Point, d: Point) -> Option<Point> {
    let r = b.minus(a);
    let s = d.minus(c);
    let den = r.x * s.y - r.y * s.x;
    if den.abs() < 1e-15 {
        return None;
    }
    let t = ((c.x - a.x) * s.y - (c.y - a.y) * s.x) / den;
    Some(a.plus(r.scale(t)))
}

/// `\pgfpointborderrectangle`: the point where the ray in direction `d` leaves the rectangle of
/// half sizes `h` centered at the origin.
pub fn border_rectangle(d: Point, h: Point) -> Point {
    let tx = if d.x.abs() > 1e-12 { h.x / d.x.abs() } else { f64::INFINITY };
    let ty = if d.y.abs() > 1e-12 { h.y / d.y.abs() } else { f64::INFINITY };
    d.scale(tx.min(ty))
}

/// `\pgfpointborderellipse`: the point where the ray in direction `d` meets the ellipse of radii
/// `r` centered at the origin.
pub fn border_ellipse(d: Point, r: Point) -> Point {
    let (a, b) = (r.x.max(1e-12), r.y.max(1e-12));
    let k = 1.0 / ((d.x / a).powi(2) + (d.y / b).powi(2)).sqrt();
    d.scale(k)
}

/// Where the ray from `o` in direction `d` leaves the closed outline (its farthest crossing).
pub fn ray_exit(o: Point, d: Point, outline: &[Point]) -> Option<Point> {
    let n = outline.len();
    let mut best: Option<f64> = None;
    for k in 0..n {
        let a = outline[k];
        let b = outline[(k + 1) % n];
        let s = b.minus(a);
        let den = d.x * s.y - d.y * s.x;
        if den.abs() < 1e-15 {
            continue;
        }
        // o + t d = a + u s
        let w = a.minus(o);
        let t = (w.x * s.y - w.y * s.x) / den;
        let u = (w.x * d.y - w.y * d.x) / den;
        if t > 1e-12 && (-1e-9..=1.0 + 1e-9).contains(&u) && best.is_none_or(|bt| t > bt) {
            best = Some(t);
        }
    }
    best.map(|t| o.plus(d.scale(t)))
}

/// The border anchor from a reference point: where the ray from `o` toward `d` leaves the outline,
/// or `o` if it does not.
pub fn ray(o: Point, d: Point, outline: &[Point]) -> Point {
    ray_exit(o, d, outline).unwrap_or(o)
}

/// The polygon with each side moved out by `d` (the corners by the miter length), as the shapes
/// compute their anchor borders from `outer sep`.
pub fn miter_offset(poly: &[Point], d: f64) -> Vec<Point> {
    let n = poly.len();
    if n < 3 || d == 0.0 {
        return poly.to_vec();
    }
    // the outward normal side depends on the orientation
    let area: f64 = (0..n).map(|k| poly[k].x * poly[(k + 1) % n].y - poly[(k + 1) % n].x * poly[k].y).sum();
    let sign = if area >= 0.0 { 1.0 } else { -1.0 };
    let shifted = |a: Point, b: Point| -> (Point, Point) {
        let e = b.minus(a);
        let l = e.norm().max(1e-15);
        // right normal of a counterclockwise polygon points outside
        let nrm = Point::new(e.y / l, -e.x / l).scale(sign * d);
        (a.plus(nrm), b.plus(nrm))
    };
    (0..n)
        .map(|k| {
            let (p0, p1) = shifted(poly[(k + n - 1) % n], poly[k]);
            let (q0, q1) = shifted(poly[k], poly[(k + 1) % n]);
            intersect_lines(p0, p1, q0, q1).unwrap_or(p1)
        })
        .collect()
}

/// A closed polygon as a path.
pub fn polygon_path(points: &[Point]) -> Vec<Segment> {
    let mut v = Vec::with_capacity(points.len() + 1);
    for (k, p) in points.iter().enumerate() {
        v.push(if k == 0 { Segment::MoveTo(*p) } else { Segment::LineTo(*p) });
    }
    v.push(Segment::Close);
    v
}

/// `\pgfpatharc` from the current point: an elliptical arc of radii `rx`, `ry` from `a0` to `a1`
/// degrees, whose start is `from`. Returns the end point.
pub fn arc_to(v: &mut Vec<Segment>, from: Point, a0: f64, a1: f64, rx: f64, ry: f64) -> Point {
    let c = from.minus(Point::new(rx * a0.to_radians().cos(), ry * a0.to_radians().sin()));
    v.extend(super::path::arc_segments(c, rx, ry, a0, a1, &crate::scene::Transform::default()));
    c.plus(Point::new(rx * a1.to_radians().cos(), ry * a1.to_radians().sin()))
}

/// Like [`arc_to`], rotated by `rotate` degrees around the arc's start (as `\pgftransformrotate`
/// before `\pgfpatharc`).
pub fn arc_to_rotated(v: &mut Vec<Segment>, from: Point, a0: f64, a1: f64, rx: f64, ry: f64, rotate: f64) -> Point {
    let mut local = Vec::new();
    let end = arc_to(&mut local, Point::new(0.0, 0.0), a0, a1, rx, ry);
    let t = crate::scene::Transform::translation(from.x, from.y).compose(&crate::scene::Transform::rotation(rotate));
    v.extend(crate::canvas::transform_segments(&local, &t));
    t.apply(end)
}

/// Points along an elliptical arc around `c` (for outlines), every degree or so.
pub fn arc_points(c: Point, rx: f64, ry: f64, a0: f64, a1: f64) -> Vec<Point> {
    let n = ((a1 - a0).abs().ceil() as usize).clamp(2, 3600);
    (0..=n)
        .map(|k| {
            let a = (a0 + (a1 - a0) * k as f64 / n as f64).to_radians();
            c.plus(Point::new(rx * a.cos(), ry * a.sin()))
        })
        .collect()
}

/// Points along a path's segments (curves flattened), for outlines; closes are dropped.
pub fn flatten(segs: &[Segment]) -> Vec<Point> {
    let mut out: Vec<Point> = Vec::new();
    let mut cur = Point::new(0.0, 0.0);
    for s in segs {
        match *s {
            Segment::MoveTo(p) | Segment::LineTo(p) => {
                out.push(p);
                cur = p;
            }
            Segment::CurveTo(a, b, p) => {
                for k in 1..=24 {
                    let t = k as f64 / 24.0;
                    let u = 1.0 - t;
                    out.push(cur.scale(u * u * u).plus(a.scale(3.0 * u * u * t)).plus(b.scale(3.0 * u * t * t)).plus(p.scale(t * t * t)));
                }
                cur = p;
            }
            Segment::Close => {}
        }
    }
    out
}

/// pgf's trigonometry in degrees.
pub fn sin(a: f64) -> f64 {
    a.to_radians().sin()
}

pub fn cos(a: f64) -> f64 {
    a.to_radians().cos()
}

pub fn tan(a: f64) -> f64 {
    a.to_radians().tan()
}

pub fn cot(a: f64) -> f64 {
    1.0 / tan(a)
}

/// The angle between two lines (`\pgfmathanglebetweenlines`): from direction `p1 → p2` to
/// direction `q1 → q2`, in [0, 360).
pub fn angle_between_lines(p1: Point, p2: Point, q1: Point, q2: Point) -> f64 {
    let a = angle_between(p1, p2);
    let b = angle_between(q1, q2);
    let d = b - a;
    if d < 0.0 {
        d + 360.0
    } else {
        d
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PT;
    use std::collections::HashMap;
    use std::f64::consts::FRAC_1_SQRT_2;

    /// Keys from a table of numbers and lengths (cm), flags by name.
    #[derive(Default)]
    pub struct TableKeys {
        pub values: HashMap<&'static str, f64>,
        pub flags: Vec<&'static str>,
    }

    impl Keys for TableKeys {
        fn length(&mut self, key: &str, default: f64) -> f64 {
            self.values.get(key).copied().unwrap_or(default)
        }
        fn number(&mut self, key: &str, default: f64) -> f64 {
            self.values.get(key).copied().unwrap_or(default)
        }
        fn text(&mut self, _key: &str) -> String {
            String::new()
        }
        fn flag(&mut self, name: &str) -> bool {
            self.flags.contains(&name)
        }
    }

    /// The shape for a text box of `wd`×`ht`+`dp` (pt), with TikZ's default seps.
    pub fn shape(name: &str, wd: f64, ht: f64, dp: f64, keys: &mut TableKeys) -> Geometry {
        let parts = [("text".to_string(), BoxSize { wd: wd * PT, ht: ht * PT, dp: dp * PT })];
        let mut input = Input {
            parts: &parts,
            inner_xsep: 3.333 * PT,
            inner_ysep: 3.333 * PT,
            minimum_width: PT,
            minimum_height: PT,
            outer_xsep: 0.2 * PT,
            outer_ysep: 0.2 * PT,
            aspect: 1.0,
            ex: 4.3 * PT,
            line_width: 0.4 * PT,
            keys,
        };
        compute(name, &mut input).unwrap()
    }

    fn close(p: Point, x: f64, y: f64) -> bool {
        (p.x - x * PT).abs() < 1e-3 * PT && (p.y - y * PT).abs() < 1e-3 * PT
    }

    #[test]
    fn rectangle_and_circle_anchors() {
        let mut k = TableKeys::default();
        let g = shape("rectangle", 20.0, 7.0, 2.0, &mut k);
        // half width 10 + 3.333 + .2 around x = 10, half height 4.5 + 3.333 + .2 around y = 2.5
        assert!(close(g.anchor("north east").unwrap(), 23.533, 10.533), "{:?}", g.anchor("north east"));
        assert!(close(g.anchor("base west").unwrap(), -3.533, 0.0));
        assert!(close(g.anchor("mid").unwrap(), 10.0, 2.15));
        assert!(close(g.anchor("text").unwrap(), 0.0, 0.0));
        let b = g.anchor("0").unwrap();
        assert!(close(b, 23.533, 2.5), "{b:?}");
        let c = shape("circle", 0.0, 0.0, 0.0, &mut k);
        let r = (2.0f64).sqrt() * 3.333 + 0.2;
        assert!(close(c.anchor("north").unwrap(), 0.0, r));
        assert!(close(c.anchor("north east").unwrap(), FRAC_1_SQRT_2 * r, FRAC_1_SQRT_2 * r));
    }

    #[test]
    fn regular_polygon_and_star() {
        let mut k = TableKeys::default();
        k.values.insert("/pgf/regular polygon sides", 4.0);
        let g = shape("regular polygon", 0.0, 0.0, 0.0, &mut k);
        // a square around the incircle of radius √2·3.333: corners at 45°, 135°…
        let r = 2.0f64.sqrt() * 3.333 / (45.0f64).to_radians().cos();
        let a = r + 0.2 / (45.0f64).to_radians().sin();
        let c1 = g.anchor("corner 1").unwrap();
        assert!(close(c1, a * FRAC_1_SQRT_2, a * FRAC_1_SQRT_2), "{c1:?}");
        assert!(g.anchor("side 4").is_some() && g.anchor("corner 5").is_none());
        let mut k = TableKeys { flags: vec!["pgf@lib@shapes@starouterradiususesratio"], ..Default::default() };
        let s = shape("star", 0.0, 0.0, 0.0, &mut k);
        let outer = s.anchor("outer point 1").unwrap().norm();
        let inner = s.anchor("inner point 1").unwrap().norm();
        assert!(outer > 1.4 * inner && outer < 1.8 * inner, "{outer} {inner}");
        assert!((s.anchor("outer point 1").unwrap().x).abs() < 1e-9);
    }

    #[test]
    fn geometric_named_anchors() {
        let mut k = TableKeys::default();
        let t = shape("trapezium", 20.0, 7.0, 0.0, &mut k);
        assert!(t.anchor("bottom left corner").unwrap().x < t.anchor("top left corner").unwrap().x);
        let i = shape("isosceles triangle", 10.0, 7.0, 0.0, &mut k);
        let apex = i.anchor("apex").unwrap();
        assert!(apex.x > i.anchor("east").unwrap().x - 1e-9 && (apex.y - i.center.y).abs() < 1e-9);
        let s = shape("semicircle", 10.0, 7.0, 0.0, &mut k);
        assert!(s.anchor("apex").unwrap().y > s.anchor("chord center").unwrap().y);
        let d = shape("diamond", 10.0, 7.0, 0.0, &mut k);
        let (n, e, ne) = (d.anchor("north").unwrap(), d.anchor("east").unwrap(), d.anchor("north east").unwrap());
        assert!((ne.x - (e.x + d.center.x) / 2.0).abs() < 1e-9 && (ne.y - (n.y + d.center.y) / 2.0).abs() < 1e-9);
        for name in ["kite", "dart", "circular sector", "cylinder"] {
            let g = shape(name, 10.0, 7.0, 2.0, &mut k);
            for a in ["north", "south", "east", "west", "base east", "mid west"] {
                if let Some(p) = g.anchor(a) {
                    assert!(p.x.is_finite() && p.y.is_finite(), "{name} {a}");
                }
            }
            assert!(!g.paths.is_empty(), "{name}");
        }
        let c = shape("cylinder", 10.0, 7.0, 2.0, &mut k);
        assert!(c.anchor("top").unwrap().x > c.anchor("before top").unwrap().x);
    }

    #[test]
    fn miter_offset_of_a_square() {
        let sq = [Point::new(0.0, 0.0), Point::new(1.0, 0.0), Point::new(1.0, 1.0), Point::new(0.0, 1.0)];
        let o = miter_offset(&sq, 0.1);
        assert!((o[0].x + 0.1).abs() < 1e-12 && (o[0].y + 0.1).abs() < 1e-12, "{o:?}");
        assert!((o[2].x - 1.1).abs() < 1e-12 && (o[2].y - 1.1).abs() < 1e-12, "{o:?}");
        // clockwise gives the same
        let mut cw = sq.to_vec();
        cw.reverse();
        let o = miter_offset(&cw, 0.1);
        assert!((o[0].x + 0.1).abs() < 1e-12 && (o[0].y - 1.1).abs() < 1e-12, "{o:?}");
    }

    #[test]
    fn rays_leave_outlines() {
        let sq = [Point::new(-1.0, -1.0), Point::new(1.0, -1.0), Point::new(1.0, 1.0), Point::new(-1.0, 1.0)];
        let p = ray(Point::new(0.0, 0.0), Point::new(2.0, 1.0), &sq);
        assert!((p.x - 1.0).abs() < 1e-12 && (p.y - 0.5).abs() < 1e-12, "{p:?}");
        let e = border_ellipse(Point::new(1.0, 0.0), Point::new(2.0, 1.0));
        assert!((e.x - 2.0).abs() < 1e-12);
    }
}
