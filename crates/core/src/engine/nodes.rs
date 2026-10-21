//! Nodes: text in a shape, placed by an anchor, as `\tikz@fig` and pgf's shapes do.
//!
//! A node's own coordinate system has the text's origin (the left end of its first baseline) at
//! (0, 0). The shape (see [`super::shapes`]) is computed there, from the text boxes of the node's
//! parts, `inner sep`, the minimum sizes and `outer sep`; the node is then moved so that the chosen
//! anchor lands on its position. The transformations of the picture apply to that position only,
//! unless `transform shape`; those of the node's own options apply to the whole node.

use std::rc::Rc;

use crate::canvas::{transform_segments, Fill, Item, Paint, Stroke};
use crate::keys;
use crate::label;
use crate::scene::{Label, Point, Segment, Transform};
use crate::tex::{self, Token};
use crate::{Error, PT};

use super::shapes::{self, BoxSize, Geometry, Layer, Usage};
use super::{tikra_tokens, Engine};

pub fn is_shape(name: &str) -> bool {
    shapes::is_shape(name)
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
    /// `fit`: the width and height of the fitted area, which become the text box
    pub fit: Option<(f64, f64)>,
    /// `circle through`: the point on the circle (canvas)
    pub through: Option<Point>,
}

/// A named node: its shape in its own coordinate system, and where that system is on the canvas.
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub shape: String,
    pub geometry: Rc<Geometry>,
    /// node coordinates → canvas
    pub transform: Transform,
}

impl NodeInfo {
    pub fn center(&self) -> Point {
        self.transform.apply(self.geometry.center)
    }

    /// The border point toward canvas point `p` (where a line to the node stops).
    pub fn border_toward(&self, p: Point) -> Point {
        let inv = crate::reader::invert(&self.transform);
        let q = inv.apply(p);
        let b = self.geometry.border_point(q.minus(self.geometry.center));
        self.transform.apply(b)
    }

    /// A named or angle anchor, on the canvas.
    pub fn anchor(&self, name: &str) -> Option<Point> {
        self.anchor_local(name).map(|p| self.transform.apply(p))
    }

    pub fn anchor_local(&self, name: &str) -> Option<Point> {
        self.geometry.anchor(name)
    }
}

/// The node's text split at `\nodepart[options]{name}`: the parts in order, `text` first.
pub fn split_parts(tokens: &[Token]) -> Vec<(String, Vec<Token>)> {
    let mut parts = vec![("text".to_string(), Vec::new())];
    let mut depth = 0usize;
    let mut i = 0;
    let skip_spaces = |j: &mut usize| {
        while tokens.get(*j).is_some_and(|t| t.is_space()) {
            *j += 1;
        }
    };
    while i < tokens.len() {
        let t = &tokens[i];
        match t {
            Token::Char(_, tex::Cat::Begin) => depth += 1,
            Token::Char(_, tex::Cat::End) => depth = depth.saturating_sub(1),
            _ => {}
        }
        if depth == 0 && t.is_cs("nodepart") {
            let mut j = i + 1;
            skip_spaces(&mut j);
            // the part's options are not used
            if tokens.get(j).is_some_and(|t| t.is_char('[')) {
                while j < tokens.len() && !tokens[j].is_char(']') {
                    j += 1;
                }
                j += 1;
                skip_spaces(&mut j);
            }
            let mut name = Vec::new();
            if let Some(Token::Char(_, tex::Cat::Begin)) = tokens.get(j) {
                let mut d = 0usize;
                while j < tokens.len() {
                    match &tokens[j] {
                        Token::Char(_, tex::Cat::Begin) => d += 1,
                        Token::Char(_, tex::Cat::End) => d -= 1,
                        _ => {}
                    }
                    if d == 0 {
                        break;
                    }
                    if !(d == 1 && matches!(tokens[j], Token::Char(_, tex::Cat::Begin))) {
                        name.push(tokens[j].clone());
                    }
                    j += 1;
                }
                j += 1;
            } else if let Some(t) = tokens.get(j) {
                name.push(t.clone());
                j += 1;
            }
            let name = tex::tokens_to_string(&name).split_whitespace().collect::<Vec<_>>().join(" ");
            // pgf's other names for the boxes of the rectangle split
            let name = match name.as_str() {
                "one" => "text".to_string(),
                "second" => "two".to_string(),
                "third" => "three".to_string(),
                "fourth" => "four".to_string(),
                _ => name,
            };
            parts.push((name, Vec::new()));
            i = j;
            continue;
        }
        parts.last_mut().expect("parts start with text").1.push(t.clone());
        i += 1;
    }
    parts
}

/// The shape keys, read from the engine's pgfkeys.
struct EngineKeys<'a>(&'a mut Engine);

impl shapes::Keys for EngineKeys<'_> {
    fn length(&mut self, key: &str, default: f64) -> f64 {
        self.0.key_length(key, default)
    }

    fn length_of(&mut self, text: &str, default: f64) -> f64 {
        self.0.length(text).unwrap_or(default)
    }

    fn number(&mut self, key: &str, default: f64) -> f64 {
        match keys::value(&self.0.tex, key) {
            Some(v) if !keys::trim(&v).is_empty() => self.0.number_of(v).unwrap_or(default),
            _ => default,
        }
    }

    fn text(&mut self, key: &str) -> String {
        match keys::value(&self.0.tex, key) {
            Some(v) => self.0.tex.expand_to_string(v).unwrap_or_default(),
            None => String::new(),
        }
    }

    fn flag(&mut self, name: &str) -> bool {
        let m = self.0.tex.meaning_of(&format!("if{name}"));
        m == self.0.tex.meaning_of("iftrue")
    }

    fn point(&mut self, key: &str) -> Option<Point> {
        let text = self.text(key);
        let text = crate::options::strip_braces(text.trim()).trim().to_string();
        if text.is_empty() {
            return None;
        }
        // `(a)+(b)`: the second relative to the first
        let mut total: Option<Point> = None;
        for part in text.split(")+(").map(|s| s.trim_start_matches('+')) {
            let inner = part.trim().trim_start_matches('(').trim_end_matches(')').to_string();
            let p = self.0.coordinate(&inner).ok()?.p;
            total = Some(match total {
                None => p,
                Some(t) => t.plus(p.minus(self.0.coordinate("0,0").ok()?.p)),
            });
        }
        total
    }

    fn text_box(&mut self, tex: &str) -> BoxSize {
        let (text, _) = self.0.shape_text(tex);
        if text.trim().is_empty() {
            return BoxSize::default();
        }
        let mut warnings = Vec::new();
        let l = label::compose_label(text.trim(), &self.0.st.font, None, 'c', self.0.st.text_paint_color(), &mut warnings);
        BoxSize { wd: l.width(), ht: l.height(), dp: l.depth() }
    }

    fn code_paths(&mut self, key: &str, t: Transform) -> Vec<Vec<Segment>> {
        let e = &mut *self.0;
        let Some(code) = keys::value(&e.tex, key).filter(|v| !keys::trim(v).is_empty()) else { return Vec::new() };
        // run where nothing it draws stays or counts for the picture's size
        let n = e.canvas.items.len();
        let frozen = e.canvas_frozen;
        let path = std::mem::take(&mut e.pgf_path);
        e.begin_group();
        e.canvas_frozen = true;
        e.st.transform = t;
        e.st.mode = Default::default();
        e.st.mode.draw = true;
        let r = e.run_tokens(code);
        e.end_group();
        e.canvas_frozen = frozen;
        e.pgf_path = path;
        if let Err(err) = r {
            e.warn(format!("{key}: {err}"));
        }
        e.canvas.items.split_off(n).into_iter().filter_map(|i| if let Item::Path { segments, stroke: Some(_), .. } = i { Some(segments) } else { None }).collect()
    }
}

/// A node's labels to make: the node, their specifications (true for a pin), the last node.
/// and the values of [`LABEL_KEYS`] in the node.
type NodeLabels = (NodeInfo, Vec<(Vec<Token>, bool)>, (Option<NodeInfo>, Option<String>), Vec<(&'static str, tex::Meaning)>);

/// The keys of a node's options that its labels read.
const LABEL_KEYS: [&str; 3] = ["/tikz/label distance", "/tikz/label position", "/tikz/pin distance"];

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

    pub fn key_text(&mut self, path: &str) -> String {
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
        // `transform shape` given to a scope or a path holds for its nodes
        self.st.node = NodeSetup { transform_shape: self.st.node.transform_shape, ..NodeSetup::default() };
        self.st.mode = Default::default();
        self.st.pending = Transform::default();
        self.st.on_chain = None;
        self.st.pending_joins.clear();
        self.st.after_node_path.clear();
        self.st.aliases.clear();
        self.st.start_arrow = None;
        self.st.end_arrow = None;
        // a node's background is decorated, and has actions, by its own options only
        let outer = self.st.clone();
        self.st.decorate = false;
        self.st.preactions = Default::default();
        self.st.postactions = Default::default();
        let r = self.node_in_group(spec, &outer);
        self.end_group();
        let (mut items, labels) = r?;
        // labels and pins: made after the node, out of its options (TikZ's `append after command`)
        if let Some((info, labels, last, kept)) = labels {
            // the label distances and position that the node's options set still hold
            self.begin_group();
            for (key, value) in kept {
                self.tex.set_meaning(&Token::cs(&keys::macro_name(key)), value, false);
            }
            let r = (|| {
                for (spec_tokens, pin) in labels {
                    items.extend(self.label(&info, spec_tokens, pin)?);
                }
                Ok::<(), Error>(())
            })();
            self.end_group();
            r?;
            (self.last_node, self.last_node_name) = last;
        }
        Ok(items)
    }

    /// The node's drawing, and what its labels need: the node, their specifications, and the last
    /// node to leave after them.
    fn node_in_group(&mut self, spec: NodeSpec, scope: &super::GState) -> Result<(Vec<Item>, Option<NodeLabels>), Error> {
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
        let shape_name = self.key_text("/tikz/shape").split_whitespace().collect::<Vec<_>>().join(" ");
        let shape = if shapes::is_shape(&shape_name) {
            shape_name.clone()
        } else {
            self.warn(format!("shape '{shape_name}' not supported: drawn as a rectangle"));
            "rectangle".to_string()
        };
        self.process_keys(tikra_tokens(&format!("every {shape_name} node/.try")), Some("/tikz"))?;
        let name = spec.name.clone().or_else(|| self.st.node.name.clone()).or_else(|| self.set_member_name());

        // the text boxes, one per node part
        let mut texts = Vec::new();
        // expanded first: a macro may hold the `\nodepart`s
        // with what `execute at begin node` and `execute at end node` put around it
        let body = |e: &Engine, name: &str| match e.tex.meaning_of(name) {
            tex::Meaning::Macro(m) => tex::substitute(&m.body, &[]),
            _ => Vec::new(),
        };
        let mut text_tokens = body(self, "tikz@atbegin@node");
        text_tokens.extend(spec.text.clone());
        text_tokens.extend(body(self, "tikz@atend@node"));
        let expanded = self.tex.expand_fully(text_tokens)?;
        for (part, tokens) in split_parts(&expanded) {
            texts.push((part, tex::tokens_to_string(&tokens).trim().to_string()));
        }
        let text = texts[0].1.clone();
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
        let mut labels: Vec<(String, Option<Label>)> = Vec::new();
        let mut boxes: Vec<(String, BoxSize)> = Vec::new();
        for (k, (part, t)) in texts.iter().enumerate() {
            let lbl = if shape == "coordinate" || t.is_empty() {
                None
            } else {
                let mut warnings = Vec::new();
                // a text that is all `\textcolor{color}{…}` is in that color
                let (t, color) = match whole_textcolor(t).and_then(|(c, inner)| self.color(&c).map(|c| (inner, c))) {
                    Some((inner, c)) => (inner, c),
                    None => (t.to_string(), color),
                };
                let t = t.as_str();
                let l = label::compose_label(t, &font, text_width, align, color, &mut warnings);
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
            if k == 0 {
                if let Some(b) = self.forced_box.take() {
                    (wd, ht, dp) = b;
                }
                if let Some((w, h)) = self.st.node.fit {
                    // fit: the box is the fitted area, centered on the baseline
                    (wd, ht, dp) = (w, h / 2.0, h / 2.0);
                }
            }
            boxes.push((part.clone(), BoxSize { wd, ht, dp }));
            labels.push((part.clone(), lbl));
        }
        if let Some(p) = self.st.node.through {
            // the through library: as wide as twice the distance from the node's position
            let at = self.st.node.at.or(spec.at).unwrap_or(spec.point);
            let size = format!("{}pt", 2.0 * p.minus(at).norm() / PT);
            for key in ["/pgf/minimum width", "/pgf/minimum height"] {
                self.tex.define_string(&keys::macro_name(key), &size, false);
            }
        }
        let xsep = self.key_length("/pgf/inner xsep", 0.3333 * 0.3515);
        let ysep = self.key_length("/pgf/inner ysep", 0.3333 * 0.3515);
        let min_w = self.key_length("/pgf/minimum width", PT);
        let min_h = self.key_length("/pgf/minimum height", PT);
        let ox = self.key_length("/pgf/outer xsep", 0.5 * self.st.line_width);
        let oy = self.key_length("/pgf/outer ysep", 0.5 * self.st.line_width);
        let aspect = self.key_text("/pgf/aspect").trim().parse::<f64>().ok().filter(|a| *a > 0.0).unwrap_or(1.0);
        let ex = 0.43 * self.st.font_size * PT;
        let line_width = self.st.line_width;
        let compute = |e: &mut Engine, pointer: Option<Point>| {
            let mut keys = EngineKeys(e);
            let mut input = shapes::Input {
                parts: &boxes,
                inner_xsep: xsep,
                inner_ysep: ysep,
                minimum_width: min_w,
                minimum_height: min_h,
                outer_xsep: ox,
                outer_ysep: oy,
                aspect,
                ex,
                line_width,
                pointer,
                keys: &mut keys,
            };
            shapes::compute(&shape, &mut input).expect("a known shape")
        };
        let geometry = compute(self, None);
        let mut info = NodeInfo { shape: shape.clone(), geometry: Rc::new(geometry), transform: Transform::default() };

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
        // pgfplots' `near ticklabel` anchors face the axis
        let resolved = self.frame.as_ref().and_then(|f| f.near_anchor(anchor, &self.st.pending));
        let anchor = resolved.as_deref().unwrap_or(anchor);
        let a_local = info.anchor_local(anchor).unwrap_or_else(|| {
            self.warn(format!("unknown anchor '{anchor}': center used"));
            info.geometry.center
        });
        // the picture's transformation keeps only its translation, unless transform shape
        let mut linear = Transform::default();
        if self.st.node.transform_shape {
            linear = Transform { e: 0.0, f: 0.0, ..self.st.transform };
        }
        if self.st.node.sloped {
            let angle = direction.y.atan2(direction.x).to_degrees();
            let angle = if !self.st.allow_upside_down && (angle > 90.0 + 1e-9 || angle < -90.0 - 1e-9) { angle + 180.0 } else { angle };
            linear = linear.compose(&Transform::rotation(angle));
        }
        let t = Transform::translation(target.x, target.y).compose(&linear).compose(&self.st.pending).compose(&Transform::translation(-a_local.x, -a_local.y));
        info.transform = t;
        // a callout's absolute pointer is in the picture: once the node is placed, the shape is
        // computed again with it in node coordinates (its other anchors do not depend on it)
        if shape.ends_with("callout") {
            if let Some(p) = shapes::Keys::point(&mut EngineKeys(self), "/pgf/callout pointer@absolute") {
                let local = crate::reader::invert(&t).apply(p);
                info.geometry = Rc::new(compute(self, Some(local)));
            }
        }
        let limit = super::path::MAX_COORDINATE;
        let fine = |p: Point| p.x.is_finite() && p.y.is_finite() && p.x.abs() <= limit && p.y.abs() <= limit;
        if !fine(info.center()) || !info.geometry.anchors.iter().all(|(_, p)| fine(t.apply(*p))) {
            self.warn(format!("node '{text}': position or size is not finite or beyond ±{limit} cm: ignored"));
            return Ok((Vec::new(), None));
        }
        let items = self.draw_shape(&info, labels, text_width, scope)?;
        if let Some(n) = &name {
            // inside a pic, names get its prefix
            let n = format!("{}{n}", self.st.name_prefix);
            self.tex.define_string("tikzlastnode", &n, true);
            self.join_sets(&n);
            self.nodes.insert(n, info.clone());
        } else {
            // an unnamed node is `\tikzlastnode` by an internal name, as TikZ's `tikz@f@…`
            let n = format!("tikz@f@{}", self.nodes.len());
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
        (self.last_node, self.last_node_name) = last.clone();
        let labels = std::mem::take(&mut self.st.node.labels);
        let kept = LABEL_KEYS.iter().map(|k| (*k, self.tex.meaning_of(&keys::macro_name(k)))).collect();
        Ok((items, Some((info, labels, last, kept))))
    }

    /// What a node draws, in pgf's order: behind the background, the background (as the node's
    /// options say), before it, the texts of the parts, behind the foreground, the foreground and
    /// before it. Every path counts in the bounding box, as pgf protocols paths when they are made.
    fn draw_shape(&mut self, info: &NodeInfo, labels: Vec<(String, Option<Label>)>, text_width: Option<f64>, scope: &super::GState) -> Result<Vec<Item>, Error> {
        let t = info.transform;
        let g = info.geometry.clone();
        let mut items = Vec::new();
        // a shaded node: its shading fitted to the background path, as \pgfshadepath does
        let background: Vec<Segment> =
            g.paths.iter().filter(|p| p.layer == Layer::Background && matches!(p.usage, Usage::Node)).flat_map(|p| transform_segments(&p.segments, &t)).collect();
        let node_fill = if self.st.mode.shade && !background.is_empty() {
            Some(Fill { paint: Paint::Shading(self.shading_of(&background)), opacity: self.st.fill_opacity, even_odd: self.st.even_odd })
        } else if self.st.mode.fill {
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
        let node_stroke = self.st.mode.draw.then(|| self.stroke());
        let layer_items = |e: &mut Engine, layers: &[Layer], items: &mut Vec<Item>| -> Result<(), Error> {
            for p in g.paths.iter().filter(|p| layers.contains(&p.layer)) {
                let mut segs = p.segments.clone();
                if p.layer == Layer::Background && g.roundable && e.st.rounded > 0.0 {
                    segs = super::output::round_corners(&segs, e.st.rounded);
                }
                let segs = transform_segments(&segs, &t);
                if matches!(p.usage, Usage::Node)
                    && p.layer == Layer::Background
                    && (e.st.decorate || e.st.mode.picture.is_some() || !e.st.preactions.is_empty() || !e.st.postactions.is_empty() || (e.st.double.is_some() && e.st.mode.draw))
                {
                    // the background as a path of its own, decorated, with its actions, or double
                    let saved = std::mem::take(&mut e.canvas.items);
                    let r = e.finish_path(segs, scope, "node");
                    items.extend(std::mem::replace(&mut e.canvas.items, saved));
                    r?;
                    continue;
                }
                let (fill, stroke) = match &p.usage {
                    Usage::Node => (node_fill.clone(), node_stroke.clone()),
                    Usage::Fill(c) => {
                        let color = e.color(c).unwrap_or(crate::color::Color::WHITE);
                        (Some(Fill { paint: Paint::Solid(color), opacity: e.st.fill_opacity, even_odd: false }), None)
                    }
                    Usage::Stroke => (None, Some(e.stroke())),
                };
                if e.counts_for_size() {
                    e.canvas.include_path(&segs);
                    if let Some(s) = &stroke {
                        // \pgfusepath adds half the line width around a stroked path
                        e.canvas.include_path_margin(&segs, s.width / 2.0);
                    }
                }
                if fill.is_some() || stroke.is_some() {
                    items.push(Item::Path { segments: segs, fill, stroke });
                }
            }
            Ok(())
        };
        layer_items(self, &[Layer::BehindBackground, Layer::Background, Layer::BeforeBackground], &mut items)?;
        for (part, lbl) in labels {
            let (Some(l), Some(origin)) = (lbl, g.parts.iter().find(|(n, _)| *n == part).map(|(_, p)| *p)) else { continue };
            let offset = match (text_width, l.align) {
                (Some(tw), 'c') => (tw - l.width()) / 2.0,
                (Some(tw), 'r') => tw - l.width(),
                _ => 0.0,
            };
            let place = t.compose(&Transform::translation(origin.x + offset, origin.y));
            items.push(Item::Text { label: l, place, opacity: self.st.text_opacity });
        }
        for st in &g.texts {
            let (text, own) = self.shape_text(&st.tex);
            let color = own.or(st.color.clone()).and_then(|c| self.color(&c)).unwrap_or_else(|| self.st.text_paint_color());
            if text.trim().is_empty() {
                continue;
            }
            let mut warnings = Vec::new();
            let l = label::compose_label(text.trim(), &self.st.font, None, 'c', color, &mut warnings);
            for w in warnings {
                self.warn(w);
            }
            items.push(Item::Text { label: l, place: t.compose(&Transform::translation(st.at.x, st.at.y)), opacity: self.st.text_opacity });
        }
        layer_items(self, &[Layer::BehindForeground, Layer::Foreground, Layer::BeforeForeground], &mut items)?;
        Ok(items)
    }

    /// A shape's own text, expanded, with `\char`\&` as its character and a leading `\color{…}` taken
    /// out as its color.
    fn shape_text(&mut self, tex: &str) -> (String, Option<String>) {
        let mut text = self.tex.expand_to_string(tikra_tokens(tex)).unwrap_or_else(|_| tex.to_string());
        while let Some(i) = text.find("\\char") {
            let rest = text[i + 5..].trim_start();
            let rest = rest.strip_prefix('`').unwrap_or(rest);
            let rest = rest.strip_prefix('\\').unwrap_or(rest);
            let Some(ch) = rest.chars().next() else { break };
            let after = rest[ch.len_utf8()..].to_string();
            let ch = if "&%$#_{}".contains(ch) { format!("\\{ch}") } else { ch.to_string() };
            text = format!("{}{ch}{after}", &text[..i]);
        }
        let mut color = None;
        if let Some(rest) = text.trim_start().strip_prefix("\\color") {
            let rest = rest.trim_start();
            if let Some(r) = rest.strip_prefix('{') {
                if let Some(j) = r.find('}') {
                    color = Some(r[..j].to_string());
                    text = r[j + 1..].to_string();
                }
            }
        }
        (text, color)
    }

    /// `label=[options]angle:text`, `pin=…`: another node next to this one.
    pub(super) fn label(&mut self, info: &NodeInfo, spec: Vec<Token>, pin: bool) -> Result<Vec<Item>, Error> {
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
        // `label position` among the label's options is its default position (`\tikz@label@angle`
        // expands it after them)
        for i in keys::split_list(&opts) {
            if i.key == "label position" {
                if let Some(v) = &i.value {
                    given = Some(tex::tokens_to_string(v).trim().to_string());
                }
            }
        }
        // `pin edge={…}`: options of the pin's edge, not of its label
        let mut edge_options = Vec::new();
        let items = keys::split_list(&opts);
        let taken = |i: &keys::Item| (i.value.is_none() && directions.contains(&i.key.as_str())) || i.key == "pin edge";
        if items.iter().any(taken) {
            let mut rest = Vec::new();
            for i in items {
                if i.key == "pin edge" {
                    edge_options.push(Token::other(','));
                    edge_options.extend(i.value.unwrap_or_default());
                } else if i.value.is_none() && directions.contains(&i.key.as_str()) {
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
        // as `tikz@label@post`: at the node's anchor of that (whole) angle, moved away from the
        // center by the distance, with the anchor on the side away from the node
        let (at, anchor) = if angle.is_nan() {
            (info.center(), "center".to_string())
        } else {
            let border = info.anchor(&format!("{}", angle.trunc())).unwrap_or(info.center());
            let d = border.minus(info.center());
            if d.norm() < 1e-9 {
                // a node without size (a coordinate): straight out at the angle
                (border.plus(Point::new(angle.to_radians().cos(), angle.to_radians().sin()).scale(distance)), direction_anchor(angle).to_string())
            } else {
                let u = d.scale(1.0 / d.norm());
                (border.plus(u.scale(distance)), auto_anchor(Point::new(u.y, -u.x).scale(PT), false).to_string())
            }
        };
        let mut options = tikra_tokens(&format!("every {}/.try,anchor={anchor},", if pin { "pin" } else { "label" }));
        options.extend(opts);
        let items = self.node(NodeSpec { options, name: None, at: Some(at), text: text_tokens, place: OnPath::Point(at), point: at, coordinate: false, default_pos: None })?;
        if pin {
            // the pin edge: an edge from the node to the pin's label, between their borders
            let label = self.last_node.clone();
            let (from, to) = match &label {
                Some(l) => (info.border_toward(l.center()), l.border_toward(info.center())),
                None => (info.border_toward(at), at),
            };
            self.begin_group();
            self.st.mode = Default::default();
            let mut edge = tikra_tokens("every pin edge/.try,draw");
            edge.extend(edge_options);
            self.process_keys(edge, Some("/tikz"))?;
            let stroke = self.stroke();
            self.end_group();
            let mut out = vec![Item::Path { segments: vec![Segment::MoveTo(from), Segment::LineTo(to)], fill: None, stroke: Some(stroke) }];
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

/// `\tikz@compute@direction`: the anchor of a label at `angle` (whole degrees) away from a point.
fn direction_anchor(angle: f64) -> &'static str {
    let a = (angle.trunc() as i64).rem_euclid(360);
    match a {
        0..=3 => "west",
        4..=86 => "south west",
        87..=93 => "south",
        94..=176 => "south east",
        177..=183 => "east",
        184..=266 => "north east",
        267..=273 => "north",
        274..=356 => "north west",
        _ => "west",
    }
}

/// `\textcolor{color}{text}` as the whole text: (color, text).
fn whole_textcolor(t: &str) -> Option<(String, String)> {
    let rest = t.trim().strip_prefix("\\textcolor")?.trim_start();
    let group = |s: &str| -> Option<(String, usize)> {
        let s2 = s.strip_prefix('{')?;
        let mut depth = 1;
        for (i, ch) in s2.char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some((s2[..i].to_string(), i + 2));
                    }
                }
                _ => {}
            }
        }
        None
    };
    let (c, n) = group(rest)?;
    let rest = rest[n..].trim_start();
    let (inner, m) = group(rest)?;
    rest[m..].trim().is_empty().then_some((c, inner))
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
