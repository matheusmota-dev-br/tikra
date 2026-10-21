//! Patterns declared in the document and the meta patterns:
//! - the basic layer's `\pgfdeclarepatternformonly[vars]{name}{lower left}{upper right}{tile size}{code}`
//!   (drawn in the color the pattern is used with) and `\pgfdeclarepatterninherentlycolored` (in
//!   its own colors), and TikZ's `\tikzdeclarepattern{name=…, type=…, bottom left=…, …, code=…}`;
//! - the patterns.meta library's `Lines`, `Hatch`, `Dots` and `Stars`, with their `distance`,
//!   `angle`, `xshift`, `yshift`, `line width`, `radius` and `points`.
//!
//! `pattern=name` (or `pattern={Lines[angle=45]}`) and `\pgfsetfillpattern{name}{color}` fill with
//! them, the tile repeated from the page's origin, the tiling turned by its tile transformation.

use crate::canvas::{Canvas, Fill, Item, Paint, Stroke, TileItems};
use crate::color::Color;
use crate::scene::{Point, Segment, Transform};
use crate::tex::Token;
use crate::{Error, PT};

use super::Engine;

/// A declared pattern: one tile's items, its corner and size (canvas cm), its tile transformation,
/// and whether it has its own colors.
#[derive(Debug, Clone)]
pub struct UserPattern {
    items: Vec<Item>,
    origin: Point,
    size: Point,
    transform: Transform,
    colored: bool,
}

impl Engine {
    /// `\pgfdeclarepatternformonly` (`colored` false) and `\pgfdeclarepatterninherentlycolored`.
    pub fn declare_pattern(&mut self, colored: bool) -> Result<(), Error> {
        let _ = self.optional()?;
        let name = self.argument_string()?;
        let origin = self.point_arg()?;
        let _upper_right = self.point_arg()?;
        let size = self.point_arg()?;
        let code = self.argument()?;
        let items = self.draw_tile(code)?;
        self.user_patterns.insert(name.trim().to_string(), UserPattern { items, origin, size, transform: Transform::default(), colored });
        Ok(())
    }

    /// `\tikzdeclarepattern{name=…, type=colored|uncolored, bottom left=…, top right=…, tile size=…,
    /// tile transformation=…, code=…}`.
    pub fn tikz_declare_pattern(&mut self) -> Result<(), Error> {
        let spec = self.argument()?;
        let (mut name, mut colored, mut origin, mut size, mut code) = (String::new(), false, Point::default(), Point::new(PT, PT), Vec::new());
        let mut tile_options = Vec::new();
        let point = |e: &mut Engine, v: Vec<Token>| -> Result<Point, Error> {
            let text = e.math_string(v)?;
            let t = text.trim().trim_start_matches('(').trim_end_matches(')').to_string();
            Ok(e.coordinate(&t)?.p)
        };
        for item in crate::keys::split_list(&spec) {
            let v = item.value.clone().unwrap_or_default();
            match item.key.as_str() {
                "name" => name = self.tex.expand_to_string(v)?.trim().to_string(),
                "type" => colored = self.tex.expand_to_string(v)?.trim() == "colored",
                "bottom left" => origin = point(self, v)?,
                "tile size" => size = point(self, v)?,
                "tile transformation" => tile_options = v,
                "code" => code = v,
                _ => {}
            }
        }
        let items = self.draw_tile(code)?;
        self.begin_group();
        self.st.transform = Transform::default();
        self.st.pending = Transform::default();
        let r = self.process_keys(tile_options, Some("/tikz"));
        let transform = self.st.pending;
        self.end_group();
        r?;
        self.user_patterns.insert(name, UserPattern { items, origin, size, transform, colored });
        Ok(())
    }

    /// What `code` draws, as one tile: in a picture of its own, its coordinates kept.
    fn draw_tile(&mut self, code: Vec<Token>) -> Result<Vec<Item>, Error> {
        let saved_canvas = std::mem::take(&mut self.canvas);
        let saved_depth = std::mem::replace(&mut self.picture_depth, 1);
        let saved_frozen = std::mem::replace(&mut self.canvas_frozen, false);
        self.begin_group();
        self.begin_scope();
        self.st.transform = Transform::default();
        let r = self.run_tokens(code);
        self.end_scope();
        self.end_group();
        let Canvas { items, .. } = std::mem::replace(&mut self.canvas, saved_canvas);
        self.picture_depth = saved_depth;
        self.canvas_frozen = saved_frozen;
        r?;
        Ok(items)
    }

    /// A pattern by name as a paint, form-only ones in `color`: a declared one, or a meta pattern
    /// (`Lines[angle=45]`).
    pub fn pattern_paint(&mut self, name: &str, color: Color) -> Option<Paint> {
        let name = name.trim();
        let p = match self.user_patterns.get(name) {
            Some(p) => p.clone(),
            None => self.meta_pattern(name)?,
        };
        let mut items = p.items;
        if !p.colored {
            for i in items.iter_mut() {
                if let Item::Path { fill, stroke, .. } = i {
                    if let Some(f) = fill {
                        f.paint = Paint::Solid(color);
                    }
                    if let Some(s) = stroke {
                        s.color = color;
                    }
                }
            }
        }
        Some(Paint::Tile { items: TileItems(std::rc::Rc::new(items)), origin: p.origin, size: p.size, transform: p.transform })
    }

    /// The patterns.meta library's `Lines`, `Hatch`, `Dots` and `Stars`, with their options.
    fn meta_pattern(&mut self, spec: &str) -> Option<UserPattern> {
        let (kind, options) = match spec.split_once('[') {
            Some((k, o)) => (k.trim(), o.trim_end().trim_end_matches(']')),
            None => (spec, ""),
        };
        if !matches!(kind, "Lines" | "Hatch" | "Dots" | "Stars") {
            return None;
        }
        let star = kind == "Stars";
        let (mut d, mut angle, mut xs, mut ys) = (if star { 3.0 * 0.1 } else { 3.0 * PT }, 0.0, 0.0, 0.0);
        let (mut lw, mut r, mut points) = (self.st.line_width, if star { 0.1 } else { 0.5 * PT }, 5.0);
        for item in crate::keys::split_list(&super::tikra_tokens(options)) {
            let v = crate::tex::tokens_to_string(&item.value.unwrap_or_default());
            let v = v.trim().trim_start_matches('{').trim_end_matches('}').to_string();
            match item.key.as_str() {
                "distance" => d = self.length(&v).ok()?,
                "angle" => angle = self.number(&v).ok()?,
                "xshift" => xs = self.length(&v).ok()?,
                "yshift" => ys = self.length(&v).ok()?,
                "line width" => lw = self.length(&v).ok()?,
                "radius" => r = self.length(&v).ok()?,
                "points" => points = self.number(&v).ok()?,
                _ => {}
            }
        }
        let black = Color::BLACK;
        let stroke = Stroke { width: lw, color: black, ..self.stroke() };
        let fill = Fill { paint: Paint::Solid(black), opacity: 1.0, even_odd: false };
        let p = Point::new;
        let h = d / 2.0;
        let items = match kind {
            "Lines" | "Hatch" => {
                let mut segs = vec![Segment::MoveTo(p(-h, 0.0)), Segment::LineTo(p(h, 0.0))];
                if kind == "Hatch" {
                    segs.extend([Segment::MoveTo(p(0.0, -h)), Segment::LineTo(p(0.0, h))]);
                }
                vec![Item::Path { segments: segs, fill: None, stroke: Some(stroke) }]
            }
            "Dots" => {
                let mut segs = vec![Segment::MoveTo(p(r, 0.0))];
                segs.extend(super::path::arc_segments(Point::default(), r, r, 0.0, 360.0, &Transform::default()));
                segs.push(Segment::Close);
                vec![Item::Path { segments: segs, fill: Some(fill), stroke: None }]
            }
            _ => {
                let a = 180.0 / points;
                let polar = |deg: f64, len: f64| p(len * deg.to_radians().cos(), len * deg.to_radians().sin());
                let mut segs = vec![Segment::MoveTo(polar(90.0, r))];
                for i in 1..=(points as usize) {
                    let i = i as f64;
                    segs.push(Segment::LineTo(polar(90.0 + 2.0 * i * a - a, 0.5 * r)));
                    segs.push(Segment::LineTo(polar(90.0 + 2.0 * i * a, r)));
                }
                segs.push(Segment::Close);
                vec![Item::Path { segments: segs, fill: Some(fill), stroke: None }]
            }
        };
        let transform = Transform::translation(xs, ys).compose(&Transform::rotation(angle));
        Some(UserPattern { items, origin: p(-h, -h), size: p(d, d), transform, colored: false })
    }

    /// `\pgfsetfillpattern{name}{color}`: the basic layer's fills from here on use the pattern.
    pub fn set_fill_pattern(&mut self) -> Result<(), Error> {
        let name = self.argument_string()?;
        let color = self.argument_string()?;
        let c = self.color(color.trim()).unwrap_or(Color::BLACK);
        self.st.pattern_name = Some(name.trim().to_string());
        self.st.pattern_color = Some(c);
        self.st.fill_pattern = true;
        Ok(())
    }
}
