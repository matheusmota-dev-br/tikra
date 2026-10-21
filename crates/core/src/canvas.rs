//! What the drawing layer produces, at the level of pgf's system layer: paths with their stroke and
//! fill, text placed by a transformation, clips and transparency groups, and the picture's bounding
//! box as pgf computes it. All geometry (arrow tips, node shapes, decorations) is done before; the
//! SVG driver only writes these primitives.

use crate::color::Color;
use crate::scene::{Label, Point, Segment, Transform};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cap {
    Butt,
    Round,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Join {
    Miter,
    Round,
    Bevel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    pub color: Color,
    /// in cm
    pub width: f64,
    /// on, off, … in cm, and the phase
    pub dash: Option<(Vec<f64>, f64)>,
    pub cap: Cap,
    pub join: Join,
    pub miter_limit: f64,
    pub opacity: f64,
}

/// A color stop of a shading: position from 0 to 1, and the color.
pub type Stop = (f64, Color);

/// A shading as pgf declares it, in its own 100 bp box (−50 to 50), and the transformation that
/// takes that box to the canvas (`\pgfshadepath` fits its middle half to the path's bounding box).
#[derive(Debug, Clone, PartialEq)]
pub enum Shading {
    /// Colors along y, from y = −50 (offset 0) to y = 50 (offset 1).
    Axial { transform: Transform, stops: Vec<Stop> },
    /// Circles from `focus` (radius 0) out to the circle of radius 50 around the origin.
    Radial { transform: Transform, focus: Point, stops: Vec<Stop> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Paint {
    Solid(Color),
    /// A pattern by name (`north east lines`), in its color, at the picture's scale.
    Pattern {
        name: String,
        color: Color,
    },
    Shading(Shading),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fill {
    pub paint: Paint,
    pub opacity: f64,
    pub even_odd: bool,
}

#[derive(Debug, Clone)]
pub enum Item {
    Path {
        segments: Vec<Segment>,
        fill: Option<Fill>,
        stroke: Option<Stroke>,
    },
    /// Text: the label's first baseline starts at the origin of `place`, which maps label coordinates
    /// (cm, y up) to the canvas.
    Text {
        label: Label,
        place: Transform,
        opacity: f64,
    },
    /// Clips what follows, up to the matching `End`.
    Clip {
        segments: Vec<Segment>,
        even_odd: bool,
    },
    /// A group: what follows, up to the matching `End`, is drawn with this opacity as a whole.
    Group {
        opacity: f64,
    },
    /// `transform canvas`: what follows, up to the matching `End`, is drawn transformed, line widths
    /// and text included (the picture's size does not follow, as in pgf).
    Transform {
        transform: Transform,
    },
    /// A fading: what follows, up to the matching `End`, is as opaque as the shading is white.
    Fade {
        mask: Shading,
    },
    /// A fading drawn as a picture (`\pgfdeclarefading`): what follows, up to the matching `End`, is
    /// as opaque as the picture is white there, and transparent where it draws nothing.
    FadePicture {
        items: Vec<Item>,
    },
    End,
}

#[derive(Debug, Clone, Default)]
pub struct Canvas {
    pub items: Vec<Item>,
    /// The bounding box, as pgf computes it (points and control points of paths, node shapes, text), in cm.
    pub min: Option<Point>,
    pub max: Option<Point>,
    /// The margin around (the standalone `border`), in cm.
    pub margin: f64,
}

impl Canvas {
    /// Grows the bounding box to include `p`.
    pub fn include(&mut self, p: Point) {
        if !(p.x.is_finite() && p.y.is_finite()) {
            return;
        }
        self.min = Some(match self.min {
            Some(m) => Point::new(m.x.min(p.x), m.y.min(p.y)),
            None => p,
        });
        self.max = Some(match self.max {
            Some(m) => Point::new(m.x.max(p.x), m.y.max(p.y)),
            None => p,
        });
    }

    /// Grows the bounding box by every point of a path, control points included, as pgf does.
    pub fn include_path(&mut self, segments: &[Segment]) {
        for s in segments {
            match s {
                Segment::MoveTo(p) | Segment::LineTo(p) => self.include(*p),
                Segment::CurveTo(a, b, p) => {
                    self.include(*a);
                    self.include(*b);
                    self.include(*p);
                }
                Segment::Close => {}
            }
        }
    }

    /// Grows the bounding box by the box of a path's points enlarged by `margin` on every side, as
    /// `\pgfusepath` does by half the line width for a stroked path.
    pub fn include_path_margin(&mut self, segments: &[Segment], margin: f64) {
        let mut b = Canvas::default();
        b.include_path(segments);
        if let (Some(min), Some(max)) = (b.min, b.max) {
            self.include(Point::new(min.x - margin, min.y - margin));
            self.include(Point::new(max.x + margin, max.y + margin));
        }
    }

    pub fn push(&mut self, item: Item) {
        self.items.push(item);
    }
}

impl Item {
    /// Moves the item by `d` (cm).
    pub fn translate(&mut self, d: Point) {
        let t = Transform::translation(d.x, d.y);
        match self {
            Item::Path { segments, fill, .. } => {
                *segments = transform_segments(segments, &t);
                if let Some(Fill { paint: Paint::Shading(Shading::Axial { transform, .. } | Shading::Radial { transform, .. }), .. }) = fill {
                    *transform = t.compose(transform);
                }
            }
            Item::Clip { segments, .. } => *segments = transform_segments(segments, &t),
            Item::Text { place, .. } => *place = t.compose(place),
            Item::Fade { mask: Shading::Axial { transform, .. } | Shading::Radial { transform, .. } } => *transform = t.compose(transform),
            // the items inside move too: the transformation moves with them
            Item::Transform { transform } => *transform = t.compose(transform).compose(&Transform::translation(-d.x, -d.y)),
            Item::FadePicture { items } => items.iter_mut().for_each(|i| i.translate(d)),
            Item::Group { .. } | Item::End => {}
        }
    }

    /// The item as drawn on a transformed canvas (pgf's low-level transformation): its geometry,
    /// its text and its line widths and dashes, which scale by the transformation's mean scale.
    pub fn transform(&mut self, t: &Transform) {
        let scale = (t.a * t.d - t.b * t.c).abs().sqrt();
        match self {
            Item::Path { segments, fill, stroke } => {
                *segments = transform_segments(segments, t);
                if let Some(Fill { paint: Paint::Shading(Shading::Axial { transform, .. } | Shading::Radial { transform, .. }), .. }) = fill {
                    *transform = t.compose(transform);
                }
                if let Some(s) = stroke {
                    s.width *= scale;
                    if let Some((dash, phase)) = &mut s.dash {
                        dash.iter_mut().for_each(|d| *d *= scale);
                        *phase *= scale;
                    }
                }
            }
            Item::Clip { segments, .. } => *segments = transform_segments(segments, t),
            Item::Text { place, .. } => *place = t.compose(place),
            Item::Fade { mask: Shading::Axial { transform, .. } | Shading::Radial { transform, .. } } => *transform = t.compose(transform),
            // a transformed group inside: the outer transformation around it
            Item::Transform { transform } => *transform = t.compose(transform).compose(&t.inverse()),
            Item::FadePicture { items } => items.iter_mut().for_each(|i| i.transform(t)),
            Item::Group { .. } | Item::End => {}
        }
    }
}

/// The segments with a transformation applied.
pub fn transform_segments(segments: &[Segment], t: &Transform) -> Vec<Segment> {
    segments
        .iter()
        .map(|s| match s {
            Segment::MoveTo(p) => Segment::MoveTo(t.apply(*p)),
            Segment::LineTo(p) => Segment::LineTo(t.apply(*p)),
            Segment::CurveTo(a, b, p) => Segment::CurveTo(t.apply(*a), t.apply(*b), t.apply(*p)),
            Segment::Close => Segment::Close,
        })
        .collect()
}
