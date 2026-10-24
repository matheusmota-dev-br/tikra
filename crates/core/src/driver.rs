//! The SVG driver: writes a [`Canvas`] as SVG, the way pgf's drivers write PDF. SVG user units are
//! big points (1 cm = 28.3465 bp), the unit of PDF, so the picture has the size TeX gives it.

use std::collections::HashSet;
use std::fmt::Write;

use mathra::font::GlyphId;
use mathra::layout::Item as MathItem;

use crate::canvas::{Canvas, Cap, Fill, Item, Join, Paint, Shading, Stroke};
use crate::scene::{Label, Point, Segment, Transform};

/// Big points per centimeter.
pub const BP: f64 = 72.0 / 2.54;

fn n(v: f64) -> String {
    num(v, 3)
}

fn num(v: f64, places: i32) -> String {
    let k = 10f64.powi(places);
    let r = (v * k).round() / k;
    if r == 0.0 {
        return "0".into();
    }
    if r.fract() == 0.0 && r.abs() < 1e15 {
        format!("{}", r as i64)
    } else {
        format!("{r}")
    }
}

/// A canvas point (cm, y up) in SVG units (bp, y down).
fn at(p: Point) -> (f64, f64) {
    (p.x * BP, -p.y * BP)
}

pub fn path_data(segs: &[Segment]) -> String {
    let mut d = String::new();
    for s in segs {
        match s {
            Segment::MoveTo(p) => {
                let (x, y) = at(*p);
                let _ = write!(d, "M{} {} ", n(x), n(y));
            }
            Segment::LineTo(p) => {
                let (x, y) = at(*p);
                let _ = write!(d, "L{} {} ", n(x), n(y));
            }
            Segment::CurveTo(a, b, p) => {
                let (ax, ay) = at(*a);
                let (bx, by) = at(*b);
                let (x, y) = at(*p);
                let _ = write!(d, "C{} {} {} {} {} {} ", n(ax), n(ay), n(bx), n(by), n(x), n(y));
            }
            Segment::Close => d.push_str("Z "),
        }
    }
    d.trim_end().to_string()
}

fn escape(t: &str) -> String {
    t.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn stroke_attributes(s: &Stroke) -> String {
    let mut a = format!(r#" stroke="{}" stroke-width="{}""#, s.color.hex(), n(s.width * BP));
    if let Some((dash, phase)) = &s.dash {
        if !dash.is_empty() && dash.iter().any(|d| *d > 0.0) {
            let _ = write!(a, r#" stroke-dasharray="{}""#, dash.iter().map(|x| n(x * BP)).collect::<Vec<_>>().join(" "));
            if *phase != 0.0 {
                let _ = write!(a, r#" stroke-dashoffset="{}""#, n(phase * BP));
            }
        }
    }
    match s.cap {
        Cap::Butt => {}
        Cap::Round => a.push_str(r#" stroke-linecap="round""#),
        Cap::Square => a.push_str(r#" stroke-linecap="square""#),
    }
    match s.join {
        Join::Miter => {
            // pgf's default miter limit is 10, SVG's is 4
            if s.miter_limit != 4.0 {
                let _ = write!(a, r#" stroke-miterlimit="{}""#, n(s.miter_limit.max(1.0)));
            }
        }
        Join::Round => a.push_str(r#" stroke-linejoin="round""#),
        Join::Bevel => a.push_str(r#" stroke-linejoin="bevel""#),
    }
    if s.opacity < 1.0 {
        let _ = write!(a, r#" stroke-opacity="{}""#, n(s.opacity.max(0.0)));
    }
    a
}

struct Defs {
    text: String,
    count: usize,
    /// glyphs drawn inside definitions (a pattern's tile)
    glyphs: Vec<GlyphId>,
    /// the page's lower left corner (SVG units): patterns tile from there, as in PDF
    corner: (f64, f64),
}

impl Defs {
    fn id(&mut self, prefix: &str) -> String {
        self.count += 1;
        format!("tr-{prefix}{}", self.count)
    }
}

fn paint_value(p: &Paint, defs: &mut Defs) -> String {
    match p {
        Paint::Solid(c) => c.hex(),
        Paint::Pattern { name, color } => {
            let id = defs.id("p");
            defs.text.push_str(&pattern(name, &id, &color.hex(), defs.corner));
            format!("url(#{id})")
        }
        Paint::Tile { items, origin, size, transform: t } => {
            // the tile's items drawn first (their own definitions come before the pattern)
            let (mut inner, mut glyphs) = (String::new(), Vec::new());
            items_svg(&items.0, &mut inner, defs, &mut glyphs);
            defs.glyphs.append(&mut glyphs);
            let id = defs.id("p");
            let (w, h) = (size.x * BP, size.y * BP);
            // the tile's corner in SVG space (y down): its content is placed from there
            let (x0, y0) = (origin.x * BP, -(origin.y + size.y) * BP);
            // from the page's corner
            let (cx, cy) = defs.corner;
            let m = format!("matrix({} {} {} {} {} {})", n(t.a), n(-t.b), n(-t.c), n(t.d), n(t.e * BP + cx), n(-t.f * BP + cy));
            let _ = write!(
                defs.text,
                r#"<pattern id="{id}" patternUnits="userSpaceOnUse" x="{}" y="{}" width="{}" height="{}" patternTransform="{m}"><g transform="translate({} {})">{inner}</g></pattern>"#,
                n(x0),
                n(y0),
                n(w),
                n(h),
                n(-x0),
                n(-y0)
            );
            format!("url(#{id})")
        }
        Paint::Shading(sh) => {
            let id = defs.id("s");
            let stops = |stops: &[(f64, crate::color::Color)]| {
                stops.iter().map(|(o, c)| format!(r#"<stop offset="{}" stop-color="{}"/>"#, num(o.clamp(0.0, 1.0), 4), c.hex())).collect::<String>()
            };
            // the gradient in pgf's shading box, taken to SVG space by its transformation
            let (t, body, kind) = match sh {
                Shading::Axial { transform, stops: s } => (transform, format!(r#"x1="0" y1="-50" x2="0" y2="50">{}"#, stops(s)), "linearGradient"),
                Shading::Radial { transform, focus, stops: s } => {
                    (transform, format!(r#"cx="0" cy="0" r="50" fx="{}" fy="{}">{}"#, n(focus.x), n(focus.y), stops(s)), "radialGradient")
                }
            };
            let m = format!("matrix({} {} {} {} {} {})", num(t.a * BP, 5), num(-t.b * BP, 5), num(t.c * BP, 5), num(-t.d * BP, 5), n(t.e * BP), n(-t.f * BP));
            let _ = write!(defs.text, r#"<{kind} id="{id}" gradientUnits="userSpaceOnUse" gradientTransform="{m}" {body}</{kind}>"#);
            format!("url(#{id})")
        }
    }
}

fn fill_attributes(f: &Fill, defs: &mut Defs) -> String {
    let mut a = format!(r#" fill="{}""#, paint_value(&f.paint, defs));
    if f.opacity < 1.0 {
        let _ = write!(a, r#" fill-opacity="{}""#, n(f.opacity.max(0.0)));
    }
    if f.even_odd {
        a.push_str(r#" fill-rule="evenodd""#);
    }
    a
}

/// `color!f` (as xcolor mixes it with white): its hex.
fn tint(c: (f64, f64, f64), f: f64) -> String {
    let m = |v: f64| ((v * f + (1.0 - f)) * 255.0).round().clamp(0.0, 255.0) as u8;
    format!("#{:02x}{:02x}{:02x}", m(c.0), m(c.1), m(c.2))
}

/// pgf's patterns (the `patterns` library), at their size in TeX: 3 pt cells, 0.4 pt lines; tiled
/// from the page's lower left corner `corner`.
fn pattern(name: &str, id: &str, c: &str, corner: (f64, f64)) -> String {
    // the cell size in bp, from 3 pt
    let t = 3.0 * 72.0 / 72.27;
    let w = 0.4 * 72.0 / 72.27;
    let line = |x1: f64, y1: f64, x2: f64, y2: f64| format!(r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{c}" stroke-width="{}"/>"#, n(x1), n(y1), n(x2), n(y2), n(w));
    let half = 0.5 * 72.0 / 72.27;
    let dot = |x: f64, y: f64| format!(r#"<circle cx="{}" cy="{}" r="{}" fill="{c}"/>"#, n(x), n(y), n(half));
    let (size, body) = match name {
        "north east lines" => (t, line(-t * 0.1, t * 1.1, t * 1.1, -t * 0.1)),
        "north west lines" => (t, line(-t * 0.1, -t * 0.1, t * 1.1, t * 1.1)),
        // (pgf's y up is y down here, a tile further: pgf's y is the tile's size less y; what pgf
        // draws on a tile's edge is drawn on both edges)
        "horizontal lines" => (t, line(0.0, t / 2.0, t, t / 2.0)),
        "vertical lines" => (t, line(t / 2.0, 0.0, t / 2.0, t)),
        "grid" => (t, line(0.0, t / 2.0, t, t / 2.0) + &line(t / 2.0, 0.0, t / 2.0, t)),
        "crosshatch" => (t, line(0.0, t, t, 0.0) + &line(0.0, 0.0, t, t)),
        "dots" => (t, [(0.0, 0.0), (t, 0.0), (0.0, t), (t, t)].iter().map(|&(x, y)| dot(x, y)).collect()),
        "crosshatch dots" => (t, [(0.0, 0.0), (t, 0.0), (0.0, t), (t, t), (t / 2.0, t / 2.0)].iter().map(|&(x, y)| dot(x, y)).collect()),
        // 3 mm tiles, a star of 1 mm around (1 mm, 1 mm)
        "fivepointed stars" | "sixpointed stars" => {
            let mm = 72.0 / 25.4;
            let at = |a: f64| {
                let (s, co) = a.to_radians().sin_cos();
                format!("{},{}", n((1.0 + co) * mm), n((3.0 - 1.0 - s) * mm))
            };
            let d = if name == "fivepointed stars" {
                format!("M{} L{} L{} L{} L{} Z", at(18.0), at(162.0), at(306.0), at(90.0), at(234.0))
            } else {
                format!("M{} L{} L{} Z M{} L{} L{} Z", at(30.0), at(150.0), at(270.0), at(-30.0), at(-270.0), at(-150.0))
            };
            (3.0 * mm, format!(r#"<path d="{d}" fill="{c}"/>"#))
        }
        // 4 mm tiles (pgf's y up is y down here, a tile further: pgf's y is the tile's size less y)
        "bricks" => {
            let mm = 72.0 / 25.4;
            let w = 0.8 * 72.0 / 72.27;
            let line = |x1: f64, y1: f64, x2: f64, y2: f64| {
                format!(r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{c}" stroke-width="{}"/>"#, n(x1 * mm), n(y1 * mm), n(x2 * mm), n(y2 * mm), n(w))
            };
            (4.0 * mm, line(0.0, 3.0, 4.0, 3.0) + &line(0.0, 1.0, 4.0, 1.0) + &line(1.0, 4.0, 1.0, 3.0) + &line(3.0, 3.0, 3.0, 1.0) + &line(1.0, 1.0, 1.0, 0.0))
        }
        "checkerboard" => {
            let s = 2.0 * 72.0 / 25.4;
            (s * 2.0, format!(r#"<rect y="{}" width="{}" height="{}" fill="{c}"/><rect x="{}" width="{}" height="{}" fill="{c}"/>"#, n(s), n(s), n(s), n(s), n(s), n(s)))
        }
        // the inherently colored ones, in their own colors
        "checkerboard light gray" => {
            let s = 2.0 * 72.0 / 25.4;
            let gray = tint((0.0, 0.0, 0.0), 0.2);
            (
                s * 2.0,
                format!(
                    r##"<rect width="{}" height="{}" fill="#ffffff"/><rect y="{}" width="{}" height="{}" fill="{gray}"/><rect x="{}" width="{}" height="{}" fill="{gray}"/>"##,
                    n(2.0 * s),
                    n(2.0 * s),
                    n(s),
                    n(s),
                    n(s),
                    n(s),
                    n(s),
                    n(s)
                ),
            )
        }
        // 4 pt tiles: the first color below 2 pt, the second above
        "horizontal lines light gray" | "horizontal lines gray" | "horizontal lines dark gray" | "horizontal lines light blue" | "horizontal lines dark blue" => {
            let (base, a, b) = match name {
                "horizontal lines light gray" => ((0.0, 0.0, 0.0), 0.1, 0.15),
                "horizontal lines gray" => ((0.0, 0.0, 0.0), 0.3, 0.35),
                "horizontal lines dark gray" => ((0.0, 0.0, 0.0), 0.9, 0.85),
                "horizontal lines light blue" => ((0.0, 0.0, 1.0), 0.1, 0.15),
                _ => ((0.0, 0.0, 1.0), 0.9, 0.85),
            };
            let pt = 72.0 / 72.27;
            let s = 4.0 * pt;
            (
                s,
                format!(
                    r#"<rect y="{}" width="{}" height="{}" fill="{}"/><rect width="{}" height="{}" fill="{}"/>"#,
                    n(2.0 * pt),
                    n(s),
                    n(2.0 * pt),
                    tint(base, a),
                    n(s),
                    n(2.0 * pt),
                    tint(base, b)
                ),
            )
        }
        // 8 pt tiles: a light dot just below a dark one, twice, on the background
        "crosshatch dots gray" | "crosshatch dots light steel blue" => {
            let (back, dark) = if name == "crosshatch dots gray" {
                (tint((0.0, 0.0, 0.0), 0.2), (0.0, 0.0, 0.0))
            } else {
                let steel = (0.6875, 0.765625, 0.8671875);
                (tint(steel, 1.0), (steel.0 / 2.0, steel.1 / 2.0, steel.2 / 2.0))
            };
            let pt = 72.0 / 72.27;
            let s = 8.0 * pt;
            let dot = |x: f64, y: f64, c: &str| format!(r#"<circle cx="{}" cy="{}" r="{}" fill="{c}"/>"#, n(x * pt), n((8.0 - y) * pt), n(pt));
            let (light, darker) = (tint(dark, 0.1), tint(dark, 0.7));
            let body = format!(r#"<rect width="{}" height="{}" fill="{back}"/>"#, n(s), n(s))
                + &dot(2.0, 1.75, &light)
                + &dot(6.0, 5.75, &light)
                + &dot(2.0, 2.25, &darker)
                + &dot(6.0, 6.25, &darker);
            (s, body)
        }
        _ => (t, line(0.0, t, t, 0.0)),
    };
    format!(r#"<pattern id="{id}" patternUnits="userSpaceOnUse" x="{}" y="{}" width="{}" height="{}">{body}</pattern>"#, n(corner.0), n(corner.1), n(size), n(size))
}

/// The SVG matrix for a label: inner coordinates in bp with y down, mapped by `place` (cm, y up).
fn label_matrix(t: &Transform) -> String {
    if t.a == 1.0 && t.b == 0.0 && t.c == 0.0 && t.d == 1.0 {
        return format!("translate({} {})", n(t.e * BP), n(-t.f * BP));
    }
    format!("matrix({} {} {} {} {} {})", num(t.a, 5), num(-t.b, 5), num(-t.c, 5), num(t.d, 5), n(t.e * BP), n(-t.f * BP))
}

/// The items of a mathra box with its baseline at `origin` (label cm, y up), `scale` cm per unit,
/// glyphs `stretch` times as wide.
fn math_box(c: &mathra::layout::MathBox, origin: Point, scale: f64, stretch: f64, glyphs: &mut Vec<GlyphId>) -> String {
    let mut s = String::new();
    let k = scale * BP;
    let inner = |x: f64, y: f64| -> (f64, f64) { ((origin.x + x * scale) * BP, -(origin.y + y * scale) * BP) };
    for item in &c.items {
        match item {
            MathItem::Glyph { id, x, y, scale: gs, color } => {
                glyphs.push(*id);
                let (px, py) = inner(*x as f64, *y as f64);
                let f = *gs as f64 * k;
                let fill = color.as_ref().map(|c| format!(r#" fill="{c}""#)).unwrap_or_default();
                let _ = write!(s, r##"<use href="#tr-g{}" transform="translate({} {}) scale({} {})"{fill}/>"##, id.0, n(px), n(py), num(f * stretch, 5), num(-f, 5));
            }
            MathItem::Rule { x, y, width, height, color } => {
                let (px, py) = inner(*x as f64, (*y + *height) as f64);
                let fill = color.as_ref().map(|c| format!(r#" fill="{c}""#)).unwrap_or_default();
                let _ = write!(s, r#"<rect x="{}" y="{}" width="{}" height="{}"{fill}/>"#, n(px), n(py), n(*width as f64 * k), n(*height as f64 * k));
            }
            MathItem::Line { x1, y1, x2, y2, thickness, color } => {
                let (a, b) = (inner(*x1 as f64, *y1 as f64), inner(*x2 as f64, *y2 as f64));
                let _ = write!(
                    s,
                    r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}"/>"#,
                    n(a.0),
                    n(a.1),
                    n(b.0),
                    n(b.1),
                    color.as_deref().unwrap_or("currentColor"),
                    n(*thickness as f64 * k)
                );
            }
            MathItem::Ellipse { cx, cy, rx, ry, thickness, color } => {
                let (px, py) = inner(*cx as f64, *cy as f64);
                let _ = write!(
                    s,
                    r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="none" stroke="{}" stroke-width="{}"/>"#,
                    n(px),
                    n(py),
                    n(*rx as f64 * k),
                    n(*ry as f64 * k),
                    color.as_deref().unwrap_or("currentColor"),
                    n(*thickness as f64 * k)
                );
            }
        }
    }
    s
}

/// A label's lines: the first baseline at y = 0, the others below, aligned within the label's width.
fn label(l: &Label, glyphs: &mut Vec<GlyphId>) -> String {
    let width = l.width();
    let mut s = String::new();
    for (k, line) in l.lines.iter().enumerate() {
        let w = line.width as f64 * l.scale;
        let x = match l.align {
            'l' => 0.0,
            'r' => width - w,
            _ => (width - w) / 2.0,
        };
        s.push_str(&math_box(line, Point::new(x, l.baseline(k)), l.scale, l.stretch, glyphs));
    }
    s
}

/// The items as SVG elements, into `body`; their definitions (gradients, clips, masks) into `defs`.
fn items_svg(items: &[Item], body: &mut String, defs: &mut Defs, glyphs: &mut Vec<GlyphId>) {
    for item in items {
        match item {
            Item::Path { segments, fill, stroke } => {
                if segments.is_empty() || (fill.is_none() && stroke.is_none()) {
                    continue;
                }
                let d = path_data(segments);
                let mut a = String::new();
                match fill {
                    Some(f) => a.push_str(&fill_attributes(f, defs)),
                    None => a.push_str(r#" fill="none""#),
                }
                if let Some(s) = stroke {
                    a.push_str(&stroke_attributes(s));
                }
                let _ = write!(body, r#"<path d="{d}"{a}/>"#);
            }
            Item::Text { label: l, place, opacity } => {
                let op = if *opacity < 1.0 { format!(r#" opacity="{}""#, n(*opacity)) } else { String::new() };
                let _ = write!(body, r#"<g transform="{}" fill="{}"{op}>"#, label_matrix(place), l.color.hex());
                body.push_str(&label(l, glyphs));
                body.push_str("</g>");
            }
            Item::Clip { segments, even_odd } => {
                let id = defs.id("c");
                let rule = if *even_odd { r#" clip-rule="evenodd""# } else { "" };
                let _ = write!(defs.text, r#"<clipPath id="{id}"><path d="{}"{rule}/></clipPath>"#, path_data(segments));
                let _ = write!(body, r#"<g clip-path="url(#{id})">"#);
            }
            Item::Group { opacity } => {
                if *opacity < 1.0 {
                    let _ = write!(body, r#"<g opacity="{}">"#, n(opacity.max(0.0)));
                } else {
                    body.push_str(r#"<g style="isolation:isolate">"#);
                }
            }
            Item::Blend { mode } => {
                let _ = write!(body, r#"<g style="mix-blend-mode:{mode}">"#);
            }
            Item::Transform { transform: t } => {
                // the canvas transformation in the SVG's coordinates: y down, in bp
                let _ = write!(body, r#"<g transform="matrix({} {} {} {} {} {})">"#, n(t.a), n(-t.b), n(-t.c), n(t.d), n(t.e * BP), n(-t.f * BP));
            }
            Item::Fade { mask } => {
                let gradient = paint_value(&Paint::Shading(mask.clone()), defs);
                let id = defs.id("m");
                let _ = write!(
                    defs.text,
                    r#"<mask id="{id}" maskUnits="userSpaceOnUse" x="-100000" y="-100000" width="200000" height="200000"><rect x="-100000" y="-100000" width="200000" height="200000" fill="{gradient}"/></mask>"#
                );
                let _ = write!(body, r#"<g mask="url(#{id})">"#);
            }
            Item::FadePicture { items } => {
                // the picture as a luminance mask: white opaque, black and nothing transparent
                let mut inner = String::new();
                items_svg(items, &mut inner, defs, glyphs);
                let id = defs.id("m");
                let _ = write!(defs.text, r#"<mask id="{id}" maskUnits="userSpaceOnUse" x="-100000" y="-100000" width="200000" height="200000">{inner}</mask>"#);
                let _ = write!(body, r#"<g mask="url(#{id})">"#);
            }
            Item::End => body.push_str("</g>"),
        }
    }
}

/// FNV-1a: a hash that is the same on every platform, for ids that depend only on the input.
fn fnv(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, b| (h ^ u64::from(*b)).wrapping_mul(0x100_0000_01b3))
}

pub fn svg(canvas: &Canvas, source: &str, words: &crate::describe::Description) -> String {
    let m = canvas.margin;
    let (min, max) = match (canvas.min, canvas.max) {
        (Some(a), Some(b)) => (a, b),
        _ => (Point::default(), Point::default()),
    };
    let (x0, y0, x1, y1) = (min.x - m, min.y - m, max.x + m, max.y + m);
    let (width, height) = (((x1 - x0) * BP).max(0.0), ((y1 - y0) * BP).max(0.0));

    let mut body = String::new();
    let mut defs = Defs { text: String::new(), count: 0, glyphs: Vec::new(), corner: (x0 * BP, -y0 * BP) };
    let mut glyphs = Vec::new();
    items_svg(&canvas.items, &mut body, &mut defs, &mut glyphs);
    glyphs.append(&mut defs.glyphs);
    let font = crate::label::FONT.get();
    let mut seen = HashSet::new();
    for g in glyphs {
        if seen.insert(g.0) {
            let _ = write!(defs.text, r#"<path id="tr-g{}" d="{}"/>"#, g.0, font.outline(g));
        }
    }
    // the words' ids are the figure's own: a page with many figures labels each by its own words
    let id = fnv(source.as_bytes()) & 0xffff_ffff;
    let mut s = String::new();
    let _ = write!(
        s,
        r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:tikra="urn:tikra:1" viewBox="{} {} {} {}" width="{}pt" height="{}pt" role="img" aria-labelledby="tr-title-{id:08x}"{}>"#,
        n(x0 * BP),
        n(-y1 * BP),
        n(width),
        n(height),
        n(width),
        n(height),
        if words.by_author { String::new() } else { format!(r#" aria-describedby="tr-desc-{id:08x}""#) }
    );
    let _ = write!(s, r#"<title id="tr-title-{id:08x}">{}</title>"#, escape(&words.alt));
    // the author's text is the whole of it; tikra's own has a longer description
    if !words.by_author {
        let _ = write!(s, r#"<desc id="tr-desc-{id:08x}">{}</desc>"#, escape(&words.text));
    }
    let _ = write!(s, r#"<metadata><tikra:source type="tikz">{}</tikra:source></metadata>"#, escape(source));
    let _ = write!(s, "<defs>{}</defs>{body}</svg>", defs.text);
    s
}
