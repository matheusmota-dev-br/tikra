//! pgf's basic layer: `\pgfpathmoveto`, `\pgfusepath`, `\pgfpoint…`, `\pgfset…`, `\pgftransform…`,
//! `pgfscope`, `pgfpicture`, `\pgftext`, `\pgfnode`. Points are in the current coordinate system,
//! as pgf keeps them; the current transformation applies when a point goes into a path.

use crate::scene::{Point, Segment, Transform};
use crate::tex::{self, Meaning, Token};
use crate::{Error, PT};

use super::nodes::{NodeSpec, OnPath};
use super::state::Mode;
use super::{tikra_tokens, Engine};

/// Ids of the basic layer's commands (`Meaning::Native`), after the drawing layer's.
pub const BASE: u16 = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum B {
    MoveTo,
    LineTo,
    CurveTo,
    QuadraticCurveTo,
    Close,
    Arc,
    ArcAxes,
    Ellipse,
    Circle,
    Rectangle,
    RectangleCorners,
    Grid,
    Parabola,
    Sine,
    Cosine,
    UsePath,
    UsePathQStroke,
    UsePathQFill,
    UsePathQFillStroke,
    UsePathQClip,
    LineWidth,
    Dash,
    StrokeColor,
    FillColor,
    SetColor,
    StrokeOpacity,
    FillOpacity,
    ButtCap,
    RoundCap,
    RectCap,
    MiterJoin,
    RoundJoin,
    BevelJoin,
    MiterLimit,
    EoRule,
    NonzeroRule,
    Arrows,
    ArrowsStart,
    ArrowsEnd,
    ShortenStart,
    ShortenEnd,
    CornersArced,
    Shift,
    XShift,
    YShift,
    Scale,
    XScale,
    YScale,
    XSlant,
    YSlant,
    Rotate,
    Cm,
    Reset,
    ResetNonTranslations,
    Scope,
    EndScope,
    Picture,
    EndPicture,
    Text,
    Node,
    Coordinate,
    XVec,
    YVec,
    ZVec,
}

const COMMANDS: &[(&str, B)] = &[
    ("pgfpathmoveto", B::MoveTo),
    ("pgfpathlineto", B::LineTo),
    ("pgfpathcurveto", B::CurveTo),
    ("pgfpathquadraticcurveto", B::QuadraticCurveTo),
    ("pgfpathclose", B::Close),
    ("pgfpatharc", B::Arc),
    ("pgfpatharcaxes", B::ArcAxes),
    ("pgfpathellipse", B::Ellipse),
    ("pgfpathcircle", B::Circle),
    ("pgfpathrectangle", B::Rectangle),
    ("pgfpathrectanglecorners", B::RectangleCorners),
    ("pgfpathgrid", B::Grid),
    ("pgfpathparabola", B::Parabola),
    ("pgfpathsine", B::Sine),
    ("pgfpathcosine", B::Cosine),
    ("pgfusepath", B::UsePath),
    ("pgfusepathqstroke", B::UsePathQStroke),
    ("pgfusepathqfill", B::UsePathQFill),
    ("pgfusepathqfillstroke", B::UsePathQFillStroke),
    ("pgfusepathqclip", B::UsePathQClip),
    ("pgfsetlinewidth", B::LineWidth),
    ("pgfsetdash", B::Dash),
    ("pgfsetstrokecolor", B::StrokeColor),
    ("pgfsetfillcolor", B::FillColor),
    ("pgfsetcolor", B::SetColor),
    ("pgfsetstrokeopacity", B::StrokeOpacity),
    ("pgfsetfillopacity", B::FillOpacity),
    ("pgfsetbuttcap", B::ButtCap),
    ("pgfsetroundcap", B::RoundCap),
    ("pgfsetrectcap", B::RectCap),
    ("pgfsetmiterjoin", B::MiterJoin),
    ("pgfsetroundjoin", B::RoundJoin),
    ("pgfsetbeveljoin", B::BevelJoin),
    ("pgfsetmiterlimit", B::MiterLimit),
    ("pgfseteorule", B::EoRule),
    ("pgfsetnonzerorule", B::NonzeroRule),
    ("pgfsetarrows", B::Arrows),
    ("pgfsetarrowsstart", B::ArrowsStart),
    ("pgfsetarrowsend", B::ArrowsEnd),
    ("pgfsetshortenstart", B::ShortenStart),
    ("pgfsetshortenend", B::ShortenEnd),
    ("pgfsetcornersarced", B::CornersArced),
    ("pgftransformshift", B::Shift),
    ("pgftransformxshift", B::XShift),
    ("pgftransformyshift", B::YShift),
    ("pgftransformscale", B::Scale),
    ("pgftransformxscale", B::XScale),
    ("pgftransformyscale", B::YScale),
    ("pgftransformxslant", B::XSlant),
    ("pgftransformyslant", B::YSlant),
    ("pgftransformrotate", B::Rotate),
    ("pgftransformcm", B::Cm),
    ("pgftransformreset", B::Reset),
    ("pgftransformresetnontranslations", B::ResetNonTranslations),
    ("pgfscope", B::Scope),
    ("endpgfscope", B::EndScope),
    ("pgfpicture", B::Picture),
    ("endpgfpicture", B::EndPicture),
    ("pgftext", B::Text),
    ("pgfnode", B::Node),
    ("pgfcoordinate", B::Coordinate),
    ("pgfsetxvec", B::XVec),
    ("pgfsetyvec", B::YVec),
    ("pgfsetzvec", B::ZVec),
];

/// The names a point argument can start with.
const POINTS: &[&str] = &[
    "pgfpoint",
    "pgfqpoint",
    "pgfpointorigin",
    "pgfpointpolar",
    "pgfqpointpolar",
    "pgfpointxy",
    "pgfqpointxy",
    "pgfpointxyz",
    "pgfqpointxyz",
    "pgfpointpolarxy",
    "pgfpointadd",
    "pgfpointdiff",
    "pgfpointscale",
    "pgfqpointscale",
    "pgfpointlineattime",
    "pgfpointlineatdistance",
    "pgfpointcurveattime",
    "pgfpointnormalised",
    "pgfpointanchor",
    "pgfpointshapeborder",
    "pgfpointintersectionoflines",
    "pgfpointtransformed",
];

pub fn define(e: &mut Engine) {
    for (i, (name, _)) in COMMANDS.iter().enumerate() {
        e.tex.define(name, Meaning::Native(BASE + i as u16));
    }
    // points are read by the commands that take them; on their own they do nothing
    for name in POINTS {
        e.tex.define(name, Meaning::Prim(tex::Prim::Relax));
    }
}

pub fn is_basic(id: u16) -> bool {
    (BASE..BASE + COMMANDS.len() as u16).contains(&id)
}

/// The basic layer's path being built, in canvas coordinates.
#[derive(Debug, Clone, Default)]
pub struct PgfPath {
    pub segments: Vec<Segment>,
    pub last: Option<Point>,
    pub start: Option<Point>,
}

impl Engine {
    /// A point argument (`\pgfpoint{1cm}{2cm}`, `\pgfpointpolar{30}{1cm}`, a macro that holds one), in
    /// the current coordinate system (untransformed).
    pub fn point_arg(&mut self) -> Result<Point, Error> {
        let tokens = self.argument()?;
        self.point_of(tokens)
    }

    pub fn point_of(&mut self, tokens: Vec<Token>) -> Result<Point, Error> {
        let depth = self.tex.depth();
        self.tex.push_frame(tokens);
        let r = self.point_here(depth);
        self.tex.drain_to(depth);
        r
    }

    fn point_here(&mut self, depth: usize) -> Result<Point, Error> {
        // the first token, expanding macros until a point command appears
        let mut guard = 0;
        let name = loop {
            let Some(t) = self.tex.get_token_until(depth) else { return Err(Error::Figure("a point is missing".into())) };
            if t.is_space() {
                continue;
            }
            match &t {
                Token::Cs(n) if POINTS.contains(&&**n) => break n.to_string(),
                _ => {
                    let m = self.tex.meaning(&t);
                    if m.is_expandable() && guard < 1000 {
                        guard += 1;
                        self.tex.expand(&t, &m)?;
                        continue;
                    }
                    return Err(Error::Figure(format!("not a point: {}", tex::show_token(&t))));
                }
            }
        };
        let arg = |e: &mut Engine| e.tex.read_undelimited();
        let length = |e: &mut Engine, v: Vec<Token>| e.length_of(v);
        Ok(match name.as_str() {
            "pgfpointorigin" => Point::default(),
            "pgfpoint" | "pgfqpoint" => {
                let x = arg(self)?;
                let y = arg(self)?;
                Point::new(length(self, x)?, length(self, y)?)
            }
            "pgfpointpolar" | "pgfqpointpolar" => {
                let a = arg(self)?;
                let r = arg(self)?;
                let angle = self.number_of(a)?;
                let text = tex::tokens_to_string(&r);
                let (rx, ry) = match text.split_once(" and ") {
                    Some((x, y)) => (self.length(x)?, self.length(y)?),
                    None => {
                        let v = length(self, r)?;
                        (v, v)
                    }
                };
                let (s, c) = angle.to_radians().sin_cos();
                Point::new(rx * c, ry * s)
            }
            "pgfpointxy" | "pgfqpointxy" => {
                let x = arg(self)?;
                let y = arg(self)?;
                let (x, y) = (self.number_of(x)?, self.number_of(y)?);
                self.st.x.scale(x).plus(self.st.y.scale(y))
            }
            "pgfpointxyz" | "pgfqpointxyz" => {
                let x = arg(self)?;
                let y = arg(self)?;
                let z = arg(self)?;
                let (x, y, z) = (self.number_of(x)?, self.number_of(y)?, self.number_of(z)?);
                self.st.x.scale(x).plus(self.st.y.scale(y)).plus(self.st.z.scale(z))
            }
            "pgfpointpolarxy" => {
                let a = arg(self)?;
                let r = arg(self)?;
                let (angle, radius) = (self.number_of(a)?, self.number_of(r)?);
                let (s, c) = angle.to_radians().sin_cos();
                self.st.x.scale(radius * c).plus(self.st.y.scale(radius * s))
            }
            "pgfpointadd" => {
                let p = arg(self)?;
                let q = arg(self)?;
                self.point_of(p)?.plus(self.point_of(q)?)
            }
            "pgfpointdiff" => {
                // the second minus the first
                let p = arg(self)?;
                let q = arg(self)?;
                self.point_of(q)?.minus(self.point_of(p)?)
            }
            "pgfpointscale" | "pgfqpointscale" => {
                let f = arg(self)?;
                let p = arg(self)?;
                let f = self.number_of(f)?;
                self.point_of(p)?.scale(f)
            }
            "pgfpointlineattime" => {
                let t = arg(self)?;
                let p = arg(self)?;
                let q = arg(self)?;
                let t = self.number_of(t)?;
                self.point_of(p)?.lerp(self.point_of(q)?, t)
            }
            "pgfpointlineatdistance" => {
                let d = arg(self)?;
                let p = arg(self)?;
                let q = arg(self)?;
                let d = length(self, d)?;
                let (p, q) = (self.point_of(p)?, self.point_of(q)?);
                let v = q.minus(p);
                p.plus(v.scale(d / v.norm().max(1e-12)))
            }
            "pgfpointcurveattime" => {
                let t = arg(self)?;
                let t = self.number_of(t)?;
                let mut pts = Vec::new();
                for _ in 0..4 {
                    let a = arg(self)?;
                    pts.push(self.point_of(a)?);
                }
                let u = 1.0 - t;
                pts[0].scale(u * u * u).plus(pts[1].scale(3.0 * u * u * t)).plus(pts[2].scale(3.0 * u * t * t)).plus(pts[3].scale(t * t * t))
            }
            "pgfpointnormalised" => {
                let p = arg(self)?;
                let p = self.point_of(p)?;
                // pgf normalises to a length of 1pt
                p.scale(PT / p.norm().max(1e-12))
            }
            "pgfpointanchor" | "pgfpointshapeborder" => {
                let name_cmd = name.clone();
                let name = arg(self)?;
                let second = arg(self)?;
                let name = self.tex.expand_to_string(name)?.trim().to_string();
                let canvas = {
                    let node = self.nodes.get(&name).cloned().ok_or_else(|| Error::Figure(format!("unknown node: {name}")))?;
                    if name.is_empty() {
                        Point::default()
                    } else if name_cmd == "pgfpointshapeborder" {
                        let p = self.point_of(second)?;
                        node.border_toward(self.st.transform.apply(p))
                    } else {
                        let a = self.tex.expand_to_string(second)?;
                        node.anchor(a.trim()).ok_or_else(|| Error::Figure(format!("unknown anchor: {name}.{}", a.trim())))?
                    }
                };
                crate::reader::invert(&self.st.transform).apply(canvas)
            }
            "pgfpointintersectionoflines" => {
                let mut p = Vec::new();
                for _ in 0..4 {
                    let a = arg(self)?;
                    p.push(self.point_of(a)?);
                }
                let (r, s) = (p[1].minus(p[0]), p[3].minus(p[2]));
                let den = r.x * s.y - r.y * s.x;
                if den.abs() < 1e-12 {
                    return Err(Error::Figure("intersection of parallel lines".into()));
                }
                let t = ((p[2].x - p[0].x) * s.y - (p[2].y - p[0].y) * s.x) / den;
                p[0].plus(r.scale(t))
            }
            "pgfpointtransformed" => {
                let p = arg(self)?;
                let p = self.point_of(p)?;
                self.st.transform.apply(p)
            }
            _ => Point::default(),
        })
    }

    fn pgf_add(&mut self, s: Segment) {
        let p = match s {
            Segment::MoveTo(p) | Segment::LineTo(p) | Segment::CurveTo(_, _, p) => Some(p),
            Segment::Close => self.pgf_path.start,
        };
        if let Segment::MoveTo(p) = s {
            self.pgf_path.start = Some(p);
        }
        if !matches!(s, Segment::MoveTo(_)) && self.pgf_path.segments.is_empty() {
            let at = self.pgf_path.last.unwrap_or_default();
            self.pgf_path.segments.push(Segment::MoveTo(at));
            self.pgf_path.start = Some(at);
        }
        self.pgf_path.segments.push(s);
        self.pgf_path.last = p;
    }

    fn last_local(&self) -> Point {
        crate::reader::invert(&self.st.transform).apply(self.pgf_path.last.unwrap_or_default())
    }

    /// Uses the basic layer's path with `actions` (`fill`, `stroke`, `draw`, `clip`, `discard`…).
    fn pgf_use(&mut self, actions: &str) -> Result<(), Error> {
        let segs = std::mem::take(&mut self.pgf_path.segments);
        self.pgf_path = PgfPath::default();
        self.begin_group();
        self.st.mode = Mode::default();
        for a in actions.split(',').map(str::trim) {
            match a {
                "fill" => self.st.mode.fill = true,
                "stroke" | "draw" => self.st.mode.draw = true,
                "clip" => self.st.mode.clip = true,
                "use as bounding box" => self.st.mode.bounding_box = true,
                "discard" | "" => {}
                other => self.warn(format!("\\pgfusepath{{{other}}} not supported: ignored")),
            }
        }
        // the clip outlives this group: it belongs to the pgf scope around
        let r = self.use_path(segs, "\\pgfusepath");
        self.end_group();
        r
    }

    /// `\pgfpicture` … `\endpgfpicture`, and the `pgfpicture` environment.
    pub fn basic_picture(&mut self, begin: bool) -> Result<(), Error> {
        if begin {
            self.begin_picture(None)
        } else {
            self.end_picture();
            Ok(())
        }
    }

    pub fn basic_command(&mut self, id: u16) -> Result<(), Error> {
        let Some(&(_, cmd)) = COMMANDS.get((id - BASE) as usize) else { return Ok(()) };
        let t = self.st.transform;
        match cmd {
            B::MoveTo => {
                let p = self.point_arg()?;
                self.pgf_add(Segment::MoveTo(t.apply(p)));
            }
            B::LineTo => {
                let p = self.point_arg()?;
                self.pgf_add(Segment::LineTo(t.apply(p)));
            }
            B::CurveTo => {
                let a = self.point_arg()?;
                let b = self.point_arg()?;
                let c = self.point_arg()?;
                self.pgf_add(Segment::CurveTo(t.apply(a), t.apply(b), t.apply(c)));
            }
            B::QuadraticCurveTo => {
                let q = self.point_arg()?;
                let p = self.point_arg()?;
                let a = self.last_local();
                self.pgf_add(Segment::CurveTo(t.apply(a.lerp(q, 2.0 / 3.0)), t.apply(p.lerp(q, 2.0 / 3.0)), t.apply(p)));
            }
            B::Close => self.pgf_add(Segment::Close),
            B::Arc | B::ArcAxes => {
                let s = self.argument()?;
                let en = self.argument()?;
                let (a0, a1) = (self.number_of(s)?, self.number_of(en)?);
                let center_from = self.last_local();
                let (segments, end) = if cmd == B::Arc {
                    let r = self.argument()?;
                    let text = tex::tokens_to_string(&r);
                    let (rx, ry) = match text.split_once(" and ") {
                        Some((x, y)) => (self.length(x)?, self.length(y)?),
                        None => {
                            let v = self.length_of(r)?;
                            (v, v)
                        }
                    };
                    let (s0, c0) = a0.to_radians().sin_cos();
                    let center = center_from.minus(Point::new(rx * c0, ry * s0));
                    let segs = super::path::arc_segments(t.apply(center), rx, ry, a0, a1, &t);
                    let (s1, c1) = a1.to_radians().sin_cos();
                    (segs, t.apply(center.plus(Point::new(rx * c1, ry * s1))))
                } else {
                    // the arc of the ellipse with these two axes
                    let ax = self.point_arg()?;
                    let ay = self.point_arg()?;
                    let axes = Transform { a: ax.x, b: ax.y, c: ay.x, d: ay.y, e: 0.0, f: 0.0 };
                    let (s0, c0) = a0.to_radians().sin_cos();
                    let center = center_from.minus(axes.apply(Point::new(c0, s0)));
                    let full = t.compose(&Transform::translation(center.x, center.y)).compose(&axes);
                    let segs = super::path::arc_segments(full.apply(Point::default()), 1.0, 1.0, a0, a1, &Transform { e: 0.0, f: 0.0, ..full });
                    let (s1, c1) = a1.to_radians().sin_cos();
                    (segs, full.apply(Point::new(c1, s1)))
                };
                for s in segments {
                    self.pgf_add(s);
                }
                self.pgf_path.last = Some(end);
            }
            B::Ellipse | B::Circle => {
                let c = self.point_arg()?;
                let (a, b) = if cmd == B::Circle {
                    let r = self.argument()?;
                    let r = self.length_of(r)?;
                    (Point::new(r, 0.0), Point::new(0.0, r))
                } else {
                    (self.point_arg()?, self.point_arg()?)
                };
                let axes = Transform { a: a.x, b: a.y, c: b.x, d: b.y, e: 0.0, f: 0.0 };
                let full = t.compose(&Transform::translation(c.x, c.y)).compose(&axes);
                self.pgf_add(Segment::MoveTo(full.apply(Point::new(1.0, 0.0))));
                for s in super::path::arc_segments(full.apply(Point::default()), 1.0, 1.0, 0.0, 360.0, &Transform { e: 0.0, f: 0.0, ..full }) {
                    self.pgf_add(s);
                }
                self.pgf_add(Segment::Close);
                self.pgf_path.last = Some(t.apply(c));
            }
            B::Rectangle | B::RectangleCorners => {
                let a = self.point_arg()?;
                let b = self.point_arg()?;
                let b = if cmd == B::Rectangle { a.plus(b) } else { b };
                self.pgf_add(Segment::MoveTo(t.apply(a)));
                self.pgf_add(Segment::LineTo(t.apply(Point::new(a.x, b.y))));
                self.pgf_add(Segment::LineTo(t.apply(b)));
                self.pgf_add(Segment::LineTo(t.apply(Point::new(b.x, a.y))));
                self.pgf_add(Segment::Close);
                self.pgf_path.last = Some(t.apply(a));
            }
            B::Grid => {
                // /pgf/stepx and /pgf/stepy are 1cm
                let mut step = (1.0, 1.0);
                if let Some(o) = self.optional()? {
                    for item in crate::keys::split_list(&o) {
                        let v = item.value.map(|v| self.length_of(v)).transpose()?;
                        match (item.key.as_str(), v) {
                            ("step", Some(_)) => {}
                            ("stepx", Some(v)) => step.0 = v,
                            ("stepy", Some(v)) => step.1 = v,
                            _ => {}
                        }
                    }
                }
                let ll = self.point_arg()?;
                let ur = self.point_arg()?;
                let (x0, x1) = (ll.x.min(ur.x), ll.x.max(ur.x));
                let (y0, y1) = (ll.y.min(ur.y), ll.y.max(ur.y));
                let lines = |a: f64, b: f64, s: f64| -> Vec<f64> {
                    if s <= 0.0 {
                        return Vec::new();
                    }
                    let first = (a / s).ceil();
                    let n = ((b / s).floor() - first + 1.0).clamp(0.0, 10_000.0) as usize;
                    (0..n).map(|k| (first + k as f64) * s).collect()
                };
                for y in lines(y0, y1, step.1) {
                    self.pgf_add(Segment::MoveTo(t.apply(Point::new(x0, y))));
                    self.pgf_add(Segment::LineTo(t.apply(Point::new(x1, y))));
                }
                for x in lines(x0, x1, step.0) {
                    self.pgf_add(Segment::MoveTo(t.apply(Point::new(x, y0))));
                    self.pgf_add(Segment::LineTo(t.apply(Point::new(x, y1))));
                }
            }
            B::Parabola => {
                let bend = self.point_arg()?;
                let end = self.point_arg()?;
                let mut from = self.last_local();
                if bend.norm() > 1e-12 {
                    let c = bend;
                    self.pgf_add(Segment::CurveTo(
                        t.apply(from.plus(Point::new(0.1125 * c.x, 0.225 * c.y))),
                        t.apply(from.plus(Point::new(0.5 * c.x, c.y))),
                        t.apply(from.plus(c)),
                    ));
                    from = from.plus(c);
                }
                if end.norm() > 1e-12 {
                    let c = end;
                    self.pgf_add(Segment::CurveTo(
                        t.apply(from.plus(Point::new(0.5 * c.x, 0.0))),
                        t.apply(from.plus(Point::new(0.8875 * c.x, 0.775 * c.y))),
                        t.apply(from.plus(c)),
                    ));
                }
            }
            B::Sine | B::Cosine => {
                let d = self.point_arg()?;
                let a = self.last_local();
                let (c1, c2) = if cmd == B::Sine {
                    (Point::new(0.3260 * d.x, 0.5120 * d.y), Point::new(0.6380 * d.x, d.y))
                } else {
                    (Point::new(0.3620 * d.x, 0.0), Point::new(0.6740 * d.x, 0.4880 * d.y))
                };
                self.pgf_add(Segment::CurveTo(t.apply(a.plus(c1)), t.apply(a.plus(c2)), t.apply(a.plus(d))));
            }
            B::UsePath => {
                let a = self.argument_string()?;
                self.pgf_use(&a)?;
            }
            B::UsePathQStroke => self.pgf_use("stroke")?,
            B::UsePathQFill => self.pgf_use("fill")?,
            B::UsePathQFillStroke => self.pgf_use("fill,stroke")?,
            B::UsePathQClip => self.pgf_use("clip")?,
            B::LineWidth => {
                let v = self.argument()?;
                let w = self.length_of(v)?;
                self.process_keys(tikra_tokens(&format!("line width={}pt", w / PT)), Some("/tikz"))?;
            }
            B::Dash => {
                // \pgfsetdash{{3pt}{2pt}}{0pt}
                let pattern = self.argument()?;
                let phase = self.argument()?;
                let parts = super::options::split_arguments(&pattern, 16);
                let mut v = Vec::new();
                for p in parts.into_iter().filter(|p| !crate::keys::trim(p).is_empty()) {
                    v.push(self.length_of(p)?);
                }
                let phase = self.length_of(phase).unwrap_or(0.0);
                self.st.dash = if v.is_empty() { None } else { Some((v, phase)) };
            }
            B::StrokeColor | B::FillColor | B::SetColor => {
                let c = self.argument_string()?;
                match self.color(&c) {
                    Some(c) => match cmd {
                        B::StrokeColor => self.st.draw_color = Some(c),
                        B::FillColor => self.st.fill_color = Some(c),
                        _ => {
                            self.st.color = c;
                            self.st.draw_color = None;
                            self.st.fill_color = None;
                            self.st.text_color = None;
                        }
                    },
                    None => self.warn(format!("unknown color '{c}'")),
                }
            }
            B::StrokeOpacity | B::FillOpacity => {
                let v = self.argument()?;
                let o = self.number_of(v)?;
                if cmd == B::StrokeOpacity {
                    self.st.draw_opacity = o;
                } else {
                    self.st.fill_opacity = o;
                }
            }
            B::ButtCap => self.st.cap = crate::canvas::Cap::Butt,
            B::RoundCap => self.st.cap = crate::canvas::Cap::Round,
            B::RectCap => self.st.cap = crate::canvas::Cap::Square,
            B::MiterJoin => self.st.join = crate::canvas::Join::Miter,
            B::RoundJoin => self.st.join = crate::canvas::Join::Round,
            B::BevelJoin => self.st.join = crate::canvas::Join::Bevel,
            B::MiterLimit => {
                let v = self.argument()?;
                self.st.miter_limit = self.number_of(v)?;
            }
            B::EoRule => self.st.even_odd = true,
            B::NonzeroRule => self.st.even_odd = false,
            B::Arrows | B::ArrowsStart | B::ArrowsEnd => {
                let s = self.argument_string()?;
                match cmd {
                    B::Arrows => {
                        if let Some((a, b)) = super::arrows::parse_arrows(&s) {
                            self.st.start_arrow = a;
                            self.st.end_arrow = b;
                        }
                    }
                    B::ArrowsStart => self.st.start_arrow = Some(s).filter(|x| !x.is_empty()),
                    _ => self.st.end_arrow = Some(s).filter(|x| !x.is_empty()),
                }
            }
            B::ShortenStart | B::ShortenEnd => {
                let v = self.argument()?;
                let d = self.length_of(v)?;
                if cmd == B::ShortenStart {
                    self.st.shorten_start = d;
                } else {
                    self.st.shorten_end = d;
                }
            }
            B::CornersArced => {
                let p = self.point_arg()?;
                self.st.rounded = p.x.abs().max(p.y.abs());
            }
            B::Shift => {
                let p = self.point_arg()?;
                self.st.transform = t.compose(&Transform::translation(p.x, p.y));
            }
            B::XShift | B::YShift => {
                let v = self.argument()?;
                let d = self.length_of(v)?;
                let s = if cmd == B::XShift { Transform::translation(d, 0.0) } else { Transform::translation(0.0, d) };
                self.st.transform = t.compose(&s);
            }
            B::Scale | B::XScale | B::YScale => {
                let v = self.argument()?;
                let f = self.number_of(v)?;
                let s = match cmd {
                    B::Scale => Transform::scaling(f, f),
                    B::XScale => Transform::scaling(f, 1.0),
                    _ => Transform::scaling(1.0, f),
                };
                self.st.transform = t.compose(&s);
            }
            B::XSlant | B::YSlant => {
                let v = self.argument()?;
                let f = self.number_of(v)?;
                let s = if cmd == B::XSlant { Transform { c: f, ..Transform::default() } } else { Transform { b: f, ..Transform::default() } };
                self.st.transform = t.compose(&s);
            }
            B::Rotate => {
                let v = self.argument()?;
                let a = self.number_of(v)?;
                self.st.transform = t.compose(&Transform::rotation(a));
            }
            B::Cm => {
                let mut n = [0.0; 4];
                for k in &mut n {
                    let v = self.argument()?;
                    *k = self.number_of(v)?;
                }
                let p = self.point_arg()?;
                self.st.transform = t.compose(&Transform { a: n[0], b: n[1], c: n[2], d: n[3], e: p.x, f: p.y });
            }
            B::Reset => self.st.transform = Transform::default(),
            B::ResetNonTranslations => self.st.transform = Transform { a: 1.0, b: 0.0, c: 0.0, d: 1.0, ..t },
            B::Scope => {
                self.begin_group();
                self.begin_scope();
            }
            B::EndScope => {
                self.end_scope();
                self.end_group();
            }
            B::Picture => self.basic_picture(true)?,
            B::EndPicture => self.basic_picture(false)?,
            B::Text => {
                // \pgftext[left,base,at=…,rotate=…]{text}
                let o = self.optional()?.unwrap_or_default();
                let text = self.argument()?;
                let mut anchor_x = "";
                let mut anchor_y = "";
                let mut at = Point::default();
                let mut rotate = 0.0;
                for item in crate::keys::split_list(&o) {
                    match (item.key.as_str(), item.value) {
                        ("left", None) => anchor_x = "west",
                        ("right", None) => anchor_x = "east",
                        ("top", None) => anchor_y = "north",
                        ("bottom", None) => anchor_y = "south",
                        ("base", None) => anchor_y = "base",
                        ("at", Some(v)) => at = self.point_of(v)?,
                        ("x", Some(v)) => at.x += self.length_of(v)?,
                        ("y", Some(v)) => at.y += self.length_of(v)?,
                        ("rotate", Some(v)) => rotate = self.number_of(v)?,
                        _ => {}
                    }
                }
                let anchor = match (anchor_y, anchor_x) {
                    ("", "") => "center".to_string(),
                    ("", x) => x.to_string(),
                    (y, "") => y.to_string(),
                    (y, x) => format!("{y} {x}"),
                };
                let place = t.apply(at);
                let options = tikra_tokens(&format!("shape=rectangle,inner sep=0pt,outer sep=0pt,anchor={anchor},rotate={rotate}"));
                let items = self.node(NodeSpec { options, name: None, at: Some(place), text, place: OnPath::Point(place), point: place, coordinate: false, default_pos: None })?;
                self.canvas.items.extend(items);
            }
            B::Node => {
                // \pgfnode{shape}{anchor}{text}{name}{path usage}
                let shape = self.argument_string()?;
                let anchor = self.argument_string()?;
                let text = self.argument()?;
                let name = self.argument_string()?;
                let usage = self.argument()?;
                let usage = tex::tokens_to_string(&usage);
                let mut opts = format!("shape={shape},anchor={anchor}");
                if usage.contains("stroke") || usage.contains("draw") {
                    opts.push_str(",draw");
                }
                if usage.contains("fill") {
                    opts.push_str(",fill");
                }
                let place = t.apply(Point::default());
                let name = (!name.is_empty()).then_some(name);
                let items = self.node(NodeSpec {
                    options: tikra_tokens(&opts),
                    name,
                    at: Some(place),
                    text,
                    place: OnPath::Point(place),
                    point: place,
                    coordinate: false,
                    default_pos: None,
                })?;
                self.canvas.items.extend(items);
            }
            B::Coordinate => {
                let name = self.argument_string()?;
                let p = self.point_arg()?;
                let place = t.apply(p);
                let items = self.node(NodeSpec {
                    options: Vec::new(),
                    name: Some(name),
                    at: Some(place),
                    text: Vec::new(),
                    place: OnPath::Point(place),
                    point: place,
                    coordinate: true,
                    default_pos: None,
                })?;
                self.canvas.items.extend(items);
            }
            B::XVec | B::YVec | B::ZVec => {
                let p = self.point_arg()?;
                match cmd {
                    B::XVec => self.st.x = p,
                    B::YVec => self.st.y = p,
                    _ => self.st.z = p,
                }
            }
        }
        Ok(())
    }
}
