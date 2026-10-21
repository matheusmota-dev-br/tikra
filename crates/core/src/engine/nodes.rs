//! Nodes: text in a shape, placed by an anchor, as `\tikz@fig` and pgf's shapes do.
//!
//! A node's own coordinate system has the text's origin (the left end of its first baseline) at
//! (0, 0). The shape's anchors are computed there, from the text box, `inner sep`, the minimum
//! sizes and `outer sep`; the node is then moved so that the chosen anchor lands on its position.
//! The transformations of the picture apply to that position only, unless `transform shape`; those
//! of the node's own options apply to the whole node.

use crate::canvas::{transform_segments, Fill, Item, Paint, Stroke};
use crate::keys;
use crate::label;
use crate::scene::{Label, Point, Segment, Transform};
use crate::tex::{self, Token};
use crate::{Error, PT};

use super::{tikra_tokens, Engine};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    Rectangle,
    Circle,
    Ellipse,
    Diamond,
    Coordinate,
}

pub fn shape_kind(name: &str) -> Option<ShapeKind> {
    Some(match name.trim() {
        "rectangle" => ShapeKind::Rectangle,
        "circle" => ShapeKind::Circle,
        "ellipse" => ShapeKind::Ellipse,
        "diamond" => ShapeKind::Diamond,
        "coordinate" => ShapeKind::Coordinate,
        _ => return None,
    })
}

pub fn is_shape(name: &str) -> bool {
    shape_kind(name).is_some()
}

/// Node settings that last for one node (set by its options).
#[derive(Debug, Clone, Default)]
pub struct NodeSetup {
    pub at: Option<Point>,
    pub pos: Option<f64>,
    pub sloped: bool,
    pub transform_shape: bool,
    pub name: Option<String>,
    /// `label=` and `pin=` values, in order; true for a pin
    pub labels: Vec<(Vec<Token>, bool)>,
    /// `fit=`: the width and height the text box takes
    pub fit: Option<(f64, f64)>,
}

/// A named node: its shape in its own coordinate system, and where that system is on the canvas.
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub shape: ShapeKind,
    /// the shape's center, in node coordinates
    pub c: Point,
    /// half width and half height of the shape as drawn (without outer sep)
    pub half: (f64, f64),
    pub outer: (f64, f64),
    pub mid_y: f64,
    pub text_width: f64,
    pub aspect: f64,
    /// node coordinates → canvas
    pub transform: Transform,
}

impl NodeInfo {
    pub fn center(&self) -> Point {
        self.transform.apply(self.c)
    }

    /// Half sizes of the outer border (anchors lie on it).
    fn outer_half(&self) -> (f64, f64) {
        (self.half.0 + self.outer.0, self.half.1 + self.outer.1)
    }

    /// The border point, in node coordinates, in direction `d` from the center.
    fn border_local(&self, d: Point) -> Point {
        let n = d.norm();
        if n < 1e-12 || self.shape == ShapeKind::Coordinate {
            return self.c;
        }
        let u = d.scale(1.0 / n);
        let (w, h) = self.outer_half();
        let t = match self.shape {
            ShapeKind::Rectangle => {
                let tx = if u.x.abs() > 1e-12 { w / u.x.abs() } else { f64::INFINITY };
                let ty = if u.y.abs() > 1e-12 { h / u.y.abs() } else { f64::INFINITY };
                tx.min(ty)
            }
            ShapeKind::Circle | ShapeKind::Ellipse => {
                let (a, b) = (w.max(1e-12), h.max(1e-12));
                1.0 / ((u.x / a).powi(2) + (u.y / b).powi(2)).sqrt()
            }
            ShapeKind::Diamond => 1.0 / (u.x.abs() / w.max(1e-12) + u.y.abs() / h.max(1e-12)),
            ShapeKind::Coordinate => 0.0,
        };
        self.c.plus(u.scale(t))
    }

    /// The border point toward canvas point `p` (where a line to the node stops).
    pub fn border_toward(&self, p: Point) -> Point {
        let inv = crate::reader::invert(&self.transform);
        let q = inv.apply(p);
        let b = self.border_local(q.minus(self.c));
        self.transform.apply(b)
    }

    /// A named or angle anchor, on the canvas.
    pub fn anchor(&self, name: &str) -> Option<Point> {
        self.anchor_local(name).map(|p| self.transform.apply(p))
    }

    pub fn anchor_local(&self, name: &str) -> Option<Point> {
        let (w, h) = self.outer_half();
        let c = self.c;
        let corner = |sx: f64, sy: f64| -> Point {
            match self.shape {
                ShapeKind::Rectangle | ShapeKind::Coordinate => c.plus(Point::new(sx * w, sy * h)),
                _ => self.border_local(Point::new(sx * w.max(1e-9), sy * h.max(1e-9))),
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
            "base" => Point::new(c.x, 0.0),
            "base west" => Point::new(c.x - w, 0.0),
            "base east" => Point::new(c.x + w, 0.0),
            "mid" => Point::new(c.x, self.mid_y),
            "mid west" => Point::new(c.x - w, self.mid_y),
            "mid east" => Point::new(c.x + w, self.mid_y),
            "text" => Point::new(0.0, 0.0),
            other => {
                let angle: f64 = other.parse().ok()?;
                let (s, co) = angle.to_radians().sin_cos();
                return Some(self.border_local(Point::new(co, s)));
            }
        })
    }

    /// The shape's background path (no outer sep), in node coordinates.
    pub fn background(&self, rounded: f64) -> Vec<Segment> {
        let c = self.c;
        let (w, h) = self.half;
        let p = |x: f64, y: f64| c.plus(Point::new(x, y));
        match self.shape {
            ShapeKind::Rectangle => {
                let r = rounded.min(w.abs()).min(h.abs());
                if r <= 0.0 {
                    vec![Segment::MoveTo(p(-w, -h)), Segment::LineTo(p(w, -h)), Segment::LineTo(p(w, h)), Segment::LineTo(p(-w, h)), Segment::Close]
                } else {
                    // pgf rounds corners with quarter circles made of one curve each
                    let k = r * (1.0 - 0.5523);
                    vec![
                        Segment::MoveTo(p(-w + r, -h)),
                        Segment::LineTo(p(w - r, -h)),
                        Segment::CurveTo(p(w - k, -h), p(w, -h + k), p(w, -h + r)),
                        Segment::LineTo(p(w, h - r)),
                        Segment::CurveTo(p(w, h - k), p(w - k, h), p(w - r, h)),
                        Segment::LineTo(p(-w + r, h)),
                        Segment::CurveTo(p(-w + k, h), p(-w, h - k), p(-w, h - r)),
                        Segment::LineTo(p(-w, -h + r)),
                        Segment::CurveTo(p(-w, -h + k), p(-w + k, -h), p(-w + r, -h)),
                        Segment::Close,
                    ]
                }
            }
            ShapeKind::Diamond => vec![Segment::MoveTo(p(0.0, -h)), Segment::LineTo(p(w, 0.0)), Segment::LineTo(p(0.0, h)), Segment::LineTo(p(-w, 0.0)), Segment::Close],
            ShapeKind::Circle | ShapeKind::Ellipse => {
                let k = 0.5523;
                vec![
                    Segment::MoveTo(p(w, 0.0)),
                    Segment::CurveTo(p(w, h * k), p(w * k, h), p(0.0, h)),
                    Segment::CurveTo(p(-w * k, h), p(-w, h * k), p(-w, 0.0)),
                    Segment::CurveTo(p(-w, -h * k), p(-w * k, -h), p(0.0, -h)),
                    Segment::CurveTo(p(w * k, -h), p(w, -h * k), p(w, 0.0)),
                    Segment::Close,
                ]
            }
            ShapeKind::Coordinate => Vec::new(),
        }
    }
}

/// Where a node on a path goes: the path's current point, or along its last segment.
#[derive(Debug, Clone, Copy)]
pub enum OnPath {
    Point(Point),
    /// a line or a curve: start, controls, end
    Segment(Point, Option<(Point, Point)>, Point),
    /// `-|` and `|-`: two legs through a corner (pos 0.5 is the corner)
    Corner(Point, Point, Point),
}

impl OnPath {
    fn at(&self, t: f64) -> (Point, Point) {
        match *self {
            OnPath::Point(p) => (p, Point::new(1.0, 0.0)),
            OnPath::Segment(a, None, b) => (a.lerp(b, t), b.minus(a)),
            OnPath::Segment(a, Some((c1, c2)), b) => {
                let u = 1.0 - t;
                let p = a.scale(u * u * u).plus(c1.scale(3.0 * u * u * t)).plus(c2.scale(3.0 * u * t * t)).plus(b.scale(t * t * t));
                let d = c1.minus(a).scale(3.0 * u * u).plus(c2.minus(c1).scale(6.0 * u * t)).plus(b.minus(c2).scale(3.0 * t * t));
                (p, d)
            }
            OnPath::Corner(a, c, b) => {
                if t <= 0.5 {
                    (a.lerp(c, 2.0 * t), c.minus(a))
                } else {
                    (c.lerp(b, 2.0 * t - 1.0), b.minus(c))
                }
            }
        }
    }
}

pub struct NodeSpec {
    pub options: Vec<Token>,
    pub name: Option<String>,
    pub at: Option<Point>,
    pub text: Vec<Token>,
    pub place: OnPath,
    /// the path's current point, where a node without `pos` goes
    pub point: Point,
    pub coordinate: bool,
    /// the default `pos` for a node between an operation and its target
    pub default_pos: Option<f64>,
}

impl Engine {
    pub fn st_node(&mut self) -> &mut NodeSetup {
        &mut self.st.node
    }

    fn key_text(&mut self, path: &str) -> String {
        keys::value(&self.tex, path).map(|v| tex::tokens_to_string(&v)).unwrap_or_default()
    }

    fn key_length(&mut self, path: &str, default: f64) -> f64 {
        match keys::value(&self.tex, path) {
            Some(v) if !keys::trim(&v).is_empty() => self.length_of(v).unwrap_or(default),
            _ => default,
        }
    }

    /// Creates a node: returns what it draws (drawn after its path) and registers its name.
    pub fn node(&mut self, spec: NodeSpec) -> Result<Vec<Item>, Error> {
        self.begin_group();
        self.st.node = NodeSetup::default();
        self.st.mode = Default::default();
        self.st.pending = Transform::default();
        self.st.preactions.clear();
        self.st.postactions.clear();
        self.st.on_chain = None;
        self.st.pending_joins.clear();
        self.st.after_node_path.clear();
        self.st.aliases.clear();
        self.st.start_arrow = None;
        self.st.end_arrow = None;
        let r = self.node_in_group(spec);
        self.end_group();
        r
    }

    fn node_in_group(&mut self, spec: NodeSpec) -> Result<Vec<Item>, Error> {
        if spec.coordinate {
            self.tex.define_string(&keys::macro_name("/tikz/shape"), "coordinate", false);
        }
        self.process_keys(tikra_tokens("every node/.try"), Some("/tikz"))?;
        self.process_keys(spec.options.clone(), Some("/tikz"))?;
        // the node begin hook: a node on a chain takes its place and names there
        let (chain_names, joins) = self.chain_node_found()?;
        let mut after = std::mem::take(&mut self.st.after_node_path);
        after.extend(joins);
        self.after_node_path.extend(after);
        let shape_name = self.key_text("/tikz/shape");
        let shape = match shape_kind(&shape_name) {
            Some(s) => s,
            None => {
                self.warn(format!("shape '{}' not supported: drawn as a rectangle", shape_name.trim()));
                ShapeKind::Rectangle
            }
        };
        self.process_keys(tikra_tokens(&format!("every {} node/.try", shape_name.trim())), Some("/tikz"))?;
        let name = spec.name.clone().or_else(|| self.st.node.name.clone());

        // the text box
        let text = self.tex.expand_to_string(spec.text.clone())?;
        let text = text.trim().to_string();
        let text_width = {
            let t = self.key_text("/tikz/text width");
            if t.trim().is_empty() {
                None
            } else {
                Some(self.length(&t)?)
            }
        };
        let align = match self.key_text("/tikz/align").trim() {
            "right" | "flush right" => 'r',
            "center" | "flush center" => 'c',
            "left" | "flush left" | "justify" => 'l',
            _ if text_width.is_some() => 'l',
            _ => 'c',
        };
        let color = self.st.text_paint_color();
        let font = self.st.font.clone();
        let lbl: Option<Label> = if shape == ShapeKind::Coordinate || text.is_empty() {
            None
        } else {
            let mut warnings = Vec::new();
            let l = label::compose_label(&text, &font, text_width, align, color, &mut warnings);
            for w in warnings {
                self.warn(w);
            }
            Some(l)
        };
        let (mut wd, mut ht, mut dp) = match &lbl {
            Some(l) => (text_width.unwrap_or_else(|| l.width()), l.height(), l.depth()),
            None => (0.0, 0.0, 0.0),
        };
        // text height and text depth replace the box's
        for (key, target) in [("/tikz/text height", &mut ht), ("/tikz/text depth", &mut dp)] {
            let v = self.key_text(key);
            if !v.trim().is_empty() {
                *target = self.length(&v)?;
            }
        }
        if let Some(b) = self.forced_box.take() {
            (wd, ht, dp) = b;
        }
        if let Some((w, h)) = self.st.node.fit {
            // fit: the box is the fitted area, centered on the baseline
            (wd, ht, dp) = (w, h / 2.0, h / 2.0);
        }
        let xsep = self.key_length("/pgf/inner xsep", 0.3333 * 0.3515);
        let ysep = self.key_length("/pgf/inner ysep", 0.3333 * 0.3515);
        let min_w = self.key_length("/pgf/minimum width", PT);
        let min_h = self.key_length("/pgf/minimum height", PT);
        let ox = self.key_length("/pgf/outer xsep", 0.5 * self.st.line_width);
        let oy = self.key_length("/pgf/outer ysep", 0.5 * self.st.line_width);
        let aspect = self.key_text("/pgf/aspect").trim().parse::<f64>().ok().filter(|a| *a > 0.0).unwrap_or(1.0);
        let c = Point::new(wd / 2.0, (ht - dp) / 2.0);
        let (xa, ya) = (wd / 2.0 + xsep, (ht + dp) / 2.0 + ysep);
        let half = match shape {
            ShapeKind::Coordinate => (0.0, 0.0),
            ShapeKind::Rectangle => (xa.max(min_w / 2.0), ya.max(min_h / 2.0)),
            ShapeKind::Circle => {
                let r = xa.hypot(ya).max(min_w.max(min_h) / 2.0);
                (r, r)
            }
            ShapeKind::Ellipse => ((xa * std::f64::consts::SQRT_2).max(min_w / 2.0), (ya * std::f64::consts::SQRT_2).max(min_h / 2.0)),
            ShapeKind::Diamond => ((xa + ya * aspect).max(min_w / 2.0), (ya + xa / aspect).max(min_h / 2.0)),
        };
        let outer = if shape == ShapeKind::Coordinate { (0.0, 0.0) } else { (ox, oy) };
        let ex = 0.43 * self.st.font_size * PT;
        let mut info = NodeInfo { shape, c, half, outer, mid_y: 0.5 * ex, text_width: wd, aspect, transform: Transform::default() };

        // where: `at`, positioning, the path's point or a point along its last segment
        let pos = self.st.node.pos.or(spec.default_pos);
        let (target, direction) = match (self.st.node.at.or(spec.at), pos) {
            (Some(p), _) => (p, Point::new(1.0, 0.0)),
            (None, Some(t)) => spec.place.at(t),
            (None, None) => (spec.point, Point::new(1.0, 0.0)),
        };
        let mut anchor = self.key_text("/tikz/anchor");
        if let (Some(side), true, Some(_)) = (self.st.auto, anchor.trim() == "center", pos) {
            // auto: to the left of the segment (right with swap), by its direction
            anchor = auto_anchor(direction, (side == 'r') != self.st.swap).to_string();
        }
        let anchor = anchor.trim();
        let a_local = info.anchor_local(anchor).unwrap_or_else(|| {
            self.warn(format!("unknown anchor '{anchor}': center used"));
            info.c
        });
        // the picture's transformation keeps only its translation, unless transform shape
        let mut linear = Transform::default();
        if self.st.node.transform_shape {
            linear = Transform { e: 0.0, f: 0.0, ..self.st.transform };
        }
        if self.st.node.sloped {
            let angle = direction.y.atan2(direction.x).to_degrees();
            let angle = if angle > 90.0 + 1e-9 || angle < -90.0 - 1e-9 { angle + 180.0 } else { angle };
            linear = linear.compose(&Transform::rotation(angle));
        }
        let t = Transform::translation(target.x, target.y).compose(&linear).compose(&self.st.pending).compose(&Transform::translation(-a_local.x, -a_local.y));
        info.transform = t;
        let center = info.center();
        if !(center.x.is_finite() && center.y.is_finite() && center.x.abs() <= super::path::MAX_COORDINATE && center.y.abs() <= super::path::MAX_COORDINATE)
            || !(half.0.is_finite() && half.1.is_finite() && half.0 <= super::path::MAX_COORDINATE && half.1 <= super::path::MAX_COORDINATE)
        {
            self.warn(format!("node '{text}': position or size is not finite or beyond ±{} cm: ignored", super::path::MAX_COORDINATE));
            return Ok(Vec::new());
        }

        let mut items = Vec::new();
        if shape != ShapeKind::Coordinate {
            // the bounding box: the shape's anchors, as pgf protocols them
            for a in ["north east", "south west", "north west", "south east"] {
                if let Some(p) = info.anchor(a) {
                    self.canvas.include(p);
                }
            }
            let bg = transform_segments(&info.background(self.st.rounded), &t);
            // and the background path, control points included, as \pgfusepath protocols it
            self.canvas.include_path(&bg);
            let fill = if self.st.mode.fill {
                Some(Fill { paint: Paint::Solid(self.st.fill_paint_color()), opacity: self.st.fill_opacity, even_odd: self.st.even_odd })
            } else if self.st.mode.pattern {
                Some(Fill {
                    paint: Paint::Pattern { name: self.st.pattern_name.clone().unwrap_or_default(), color: self.st.pattern_color.unwrap_or(crate::color::Color::BLACK) },
                    opacity: self.st.fill_opacity,
                    even_odd: false,
                })
            } else {
                None
            };
            let stroke = self.st.mode.draw.then(|| self.stroke());
            if fill.is_some() || stroke.is_some() {
                items.push(Item::Path { segments: bg, fill, stroke });
            }
            if let Some(l) = lbl {
                let offset = match (text_width, l.align) {
                    (Some(tw), 'c') => (tw - l.width()) / 2.0,
                    (Some(tw), 'r') => tw - l.width(),
                    _ => 0.0,
                };
                let place = t.compose(&Transform::translation(offset, 0.0));
                items.push(Item::Text { label: l, place, opacity: self.st.text_opacity });
            }
        }
        if let Some(n) = &name {
            // inside a pic, names get its prefix
            let n = format!("{}{n}", self.st.name_prefix);
            self.tex.define_string("tikzlastnode", &n, true);
            self.nodes.insert(n, info.clone());
        }
        for n in chain_names.into_iter().chain(std::mem::take(&mut self.st.aliases)) {
            self.nodes.insert(n, info.clone());
        }
        // the last node, for trees (labels below do not count)
        let registered = name.as_ref().map(|n| format!("{}{n}", self.st.name_prefix));
        if let Some(slot @ None) = self.tree_child_names.last_mut() {
            *slot = Some(registered.clone().unwrap_or_default());
        }
        let last = (Some(info.clone()), registered);
        // labels and pins
        let labels = std::mem::take(&mut self.st.node.labels);
        for (spec_tokens, pin) in labels {
            items.extend(self.label(&info, spec_tokens, pin)?);
        }
        (self.last_node, self.last_node_name) = last;
        Ok(items)
    }

    /// `label=[options]angle:text`, `pin=…`: another node next to this one.
    fn label(&mut self, info: &NodeInfo, spec: Vec<Token>, pin: bool) -> Result<Vec<Item>, Error> {
        let mut v = keys::trim(&spec).to_vec();
        let mut opts = Vec::new();
        if v.first().is_some_and(|t| t.is_char('[')) {
            let mut depth = 0usize;
            let mut end = 0;
            for (i, t) in v.iter().enumerate() {
                if t.is_char('[') {
                    depth += 1;
                } else if t.is_char(']') {
                    depth -= 1;
                    if depth == 0 {
                        end = i;
                        break;
                    }
                }
            }
            opts = v[1..end].to_vec();
            v = v[end + 1..].to_vec();
        }
        let s = tex::tokens_to_string(&v);
        // the angle is up to the first top-level colon
        let (angle_text, text_tokens) = match split_colon(&v) {
            Some((a, b)) => (tex::tokens_to_string(&a).trim().to_string(), b),
            None => (String::new(), v.clone()),
        };
        let _ = s;
        // a direction among the label's options (`"text" above` with quotes) is its position
        let directions = ["above", "below", "left", "right", "above left", "above right", "below left", "below right"];
        let mut given = None;
        let items = keys::split_list(&opts);
        if items.iter().any(|i| i.value.is_none() && directions.contains(&i.key.as_str())) {
            let mut rest = Vec::new();
            for i in items {
                if i.value.is_none() && directions.contains(&i.key.as_str()) {
                    given = Some(i.key.clone());
                } else {
                    if !rest.is_empty() {
                        rest.push(Token::other(','));
                    }
                    rest.extend(tikra_tokens(&i.key));
                    if let Some(v) = i.value {
                        rest.push(Token::other('='));
                        rest.push(Token::Char('{', tex::Cat::Begin));
                        rest.extend(v);
                        rest.push(Token::Char('}', tex::Cat::End));
                    }
                }
            }
            opts = rest;
        }
        let position = if !angle_text.is_empty() {
            angle_text
        } else if let Some(g) = given {
            g
        } else {
            self.key_text("/tikz/label position").trim().to_string()
        };
        let angle = match position.as_str() {
            "above" | "north" => 90.0,
            "below" | "south" => 270.0,
            "left" | "west" => 180.0,
            "right" | "east" => 0.0,
            "above left" | "north west" => 135.0,
            "above right" | "north east" => 45.0,
            "below left" | "south west" => 225.0,
            "below right" | "south east" => 315.0,
            "center" => f64::NAN,
            other => self.number(other).unwrap_or(90.0),
        };
        let distance = if pin { self.key_length("/tikz/pin distance", 3.0 * 0.1515) } else { self.key_length("/tikz/label distance", 0.0) };
        let (border, anchor) = if angle.is_nan() {
            (info.center(), "center".to_string())
        } else {
            let compass = compass_anchor(&position);
            let b = match compass {
                Some(c) => info.anchor(c).unwrap_or(info.center()),
                None => info.anchor(&format!("{angle}")).unwrap_or(info.center()),
            };
            let opposite = match position.as_str() {
                "above" | "north" => "south".to_string(),
                "below" | "south" => "north".to_string(),
                "left" | "west" => "east".to_string(),
                "right" | "east" => "west".to_string(),
                "above left" | "north west" => "south east".to_string(),
                "above right" | "north east" => "south west".to_string(),
                "below left" | "south west" => "north east".to_string(),
                "below right" | "south east" => "north west".to_string(),
                _ => opposite_anchor(angle),
            };
            (b, opposite)
        };
        let (s_, c_) = if angle.is_nan() { (0.0, 0.0) } else { angle.to_radians().sin_cos() };
        let at = border.plus(Point::new(c_ * distance, s_ * distance));
        let mut options = tikra_tokens(&format!("every {}/.try,anchor={anchor},", if pin { "pin" } else { "label" }));
        options.extend(opts);
        let items = self.node(NodeSpec { options, name: None, at: Some(at), text: text_tokens, place: OnPath::Point(at), point: at, coordinate: false, default_pos: None })?;
        if pin {
            // the pin edge, from the label to the node's border
            let edge_from = at;
            let edge_to = border;
            self.begin_group();
            self.st.mode = Default::default();
            self.process_keys(tikra_tokens("every pin edge/.try,draw"), Some("/tikz"))?;
            let stroke = self.stroke();
            self.end_group();
            let mut out = vec![Item::Path { segments: vec![Segment::MoveTo(edge_from), Segment::LineTo(edge_to)], fill: None, stroke: Some(stroke) }];
            out.extend(items);
            return Ok(out);
        }
        Ok(items)
    }

    /// The stroke of the current state.
    pub fn stroke(&self) -> Stroke {
        Stroke {
            color: self.st.stroke_color(),
            width: self.st.line_width,
            dash: self.st.dash.clone().filter(|d| !d.0.is_empty()),
            cap: self.st.cap,
            join: self.st.join,
            miter_limit: self.st.miter_limit,
            opacity: self.st.draw_opacity,
        }
    }
}

/// TikZ's `\tikz@auto@anchor` (left of the direction) and `\tikz@auto@anchor@prime` (right).
fn auto_anchor(d: Point, right: bool) -> &'static str {
    let e = 0.05 * PT;
    let (x, y) = (d.x, d.y);
    let left = if x > e {
        if y > e {
            "south east"
        } else if y < -e {
            "south west"
        } else {
            "south"
        }
    } else if x < -e {
        if y > e {
            "north east"
        } else if y < -e {
            "north west"
        } else {
            "north"
        }
    } else if y > 0.0 {
        "east"
    } else {
        "west"
    };
    if !right {
        return left;
    }
    match left {
        "south east" => "north west",
        "south west" => "north east",
        "south" => "north",
        "north east" => "south west",
        "north west" => "south east",
        "north" => "south",
        "east" => "west",
        _ => "east",
    }
}

fn compass_anchor(p: &str) -> Option<&'static str> {
    Some(match p {
        "above" | "north" => "north",
        "below" | "south" => "south",
        "left" | "west" => "west",
        "right" | "east" => "east",
        "above left" | "north west" => "north west",
        "above right" | "north east" => "north east",
        "below left" | "south west" => "south west",
        "below right" | "south east" => "south east",
        _ => return None,
    })
}

/// The anchor of a label at `angle`: TikZ uses the compass anchor nearest to the opposite side.
fn opposite_anchor(angle: f64) -> String {
    let a = angle.rem_euclid(360.0);
    let names = ["west", "south west", "south", "south east", "east", "north east", "north", "north west"];
    if (a % 45.0).abs() < 1e-9 {
        return names[((a / 45.0).round() as usize) % 8].to_string();
    }
    format!("{}", (a + 180.0).rem_euclid(360.0))
}

/// `30:text` → (`30`, `text`): split at the first colon outside braces.
fn split_colon(v: &[Token]) -> Option<(Vec<Token>, Vec<Token>)> {
    let mut depth = 0usize;
    for (i, t) in v.iter().enumerate() {
        match t {
            Token::Char(_, tex::Cat::Begin) => depth += 1,
            Token::Char(_, tex::Cat::End) => depth = depth.saturating_sub(1),
            Token::Char(':', _) if depth == 0 => return Some((v[..i].to_vec(), tex::strip_outer_braces(keys::trim(&v[i + 1..]).to_vec()))),
            _ => {}
        }
    }
    None
}
