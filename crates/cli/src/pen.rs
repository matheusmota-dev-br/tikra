//! A figure for a pen plotter: strokes only, in G-code or HPGL. Curves become short lines, fills
//! and glyphs their outlines, and the strokes are put in the order that lifts the pen least.

use resvg::usvg::{self, tiny_skia_path::PathSegment};

/// Millimeters per CSS pixel, the unit `usvg` works in.
const MM: f64 = 25.4 / 96.0;
/// How long a curve's short lines are, at most, in mm.
const STEP: f64 = 0.25;

/// A pen-down line, in mm, from the paper's lower left corner.
pub type Stroke = Vec<(f64, f64)>;

/// What to do with the strokes.
#[derive(Debug, Clone)]
pub struct Pen {
    /// Keep the order TikZ drew them in.
    pub keep_order: bool,
    /// The paper, in mm: the figure is centered on it, and made smaller if it does not fit.
    pub paper: Option<(f64, f64)>,
    pub up: String,
    pub down: String,
    /// Drawing speed, in mm a minute.
    pub feed: f64,
}

impl Default for Pen {
    fn default() -> Pen {
        Pen { keep_order: false, paper: None, up: "G0 Z5".into(), down: "G0 Z0".into(), feed: 1500.0 }
    }
}

/// The strokes of what a group draws.
fn walk(group: &usvg::Group, height: f64, out: &mut Vec<Stroke>) {
    if group.opacity().get() == 0.0 {
        return;
    }
    for node in group.children() {
        match node {
            usvg::Node::Group(g) => walk(g, height, out),
            usvg::Node::Path(p) if p.is_visible() && (p.fill().is_some() || p.stroke().is_some()) => {
                let t = p.abs_transform();
                let at = |q: usvg::tiny_skia_path::Point| {
                    let (x, y) = (f64::from(q.x), f64::from(q.y));
                    let (sx, kx, ky, sy, tx, ty) = (f64::from(t.sx), f64::from(t.kx), f64::from(t.ky), f64::from(t.sy), f64::from(t.tx), f64::from(t.ty));
                    ((sx * x + kx * y + tx) * MM, height - (ky * x + sy * y + ty) * MM)
                };
                let mut line: Stroke = Vec::new();
                let mut first = (0.0, 0.0);
                for s in p.data().segments() {
                    match s {
                        PathSegment::MoveTo(q) => {
                            if line.len() > 1 {
                                out.push(std::mem::take(&mut line));
                            }
                            first = at(q);
                            line = vec![first];
                        }
                        PathSegment::LineTo(q) => line.push(at(q)),
                        PathSegment::QuadTo(a, q) => curve(&mut line, [at(a), at(a), at(q)], true),
                        PathSegment::CubicTo(a, b, q) => curve(&mut line, [at(a), at(b), at(q)], false),
                        PathSegment::Close => line.push(first),
                    }
                }
                if line.len() > 1 {
                    out.push(line);
                }
            }
            _ => {}
        }
    }
}

/// A curve from the end of `line` as short lines. A quadratic's control point comes twice.
fn curve(line: &mut Stroke, [a, b, q]: [(f64, f64); 3], quadratic: bool) {
    let p = line.last().copied().unwrap_or(a);
    let dist = |u: (f64, f64), v: (f64, f64)| (u.0 - v.0).hypot(u.1 - v.1);
    let length = dist(p, a) + dist(a, b) + dist(b, q);
    let n = ((length / STEP).ceil() as usize).clamp(1, 256);
    for k in 1..=n {
        let t = k as f64 / n as f64;
        let u = 1.0 - t;
        let of = |p: f64, a: f64, b: f64, q: f64| {
            if quadratic {
                u * u * p + 2.0 * u * t * a + t * t * q
            } else {
                u * u * u * p + 3.0 * u * u * t * a + 3.0 * u * t * t * b + t * t * t * q
            }
        };
        line.push((of(p.0, a.0, b.0, q.0), of(p.1, a.1, b.1, q.1)));
    }
}

/// The strokes of an SVG, and the size of the figure, in mm.
pub fn strokes(svg: &str) -> Result<(Vec<Stroke>, (f64, f64)), String> {
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).map_err(|e| e.to_string())?;
    let (w, h) = (f64::from(tree.size().width()) * MM, f64::from(tree.size().height()) * MM);
    let mut out = Vec::new();
    walk(tree.root(), h, &mut out);
    Ok((out, (w, h)))
}

/// The strokes centered on the paper, smaller if they do not fit it.
fn on_paper(strokes: &mut [Stroke], size: (f64, f64), paper: (f64, f64)) {
    let k = (paper.0 / size.0).min(paper.1 / size.1).min(1.0);
    let (dx, dy) = ((paper.0 - size.0 * k) / 2.0, (paper.1 - size.1 * k) / 2.0);
    for p in strokes.iter_mut().flatten() {
        *p = (p.0 * k + dx, p.1 * k + dy);
    }
}

/// Each next stroke the nearest one to where the pen is, from either of its ends.
fn nearest_first(mut left: Vec<Stroke>) -> Vec<Stroke> {
    let mut out = Vec::with_capacity(left.len());
    let mut at = (0.0, 0.0);
    while !left.is_empty() {
        let dist = |p: (f64, f64)| (p.0 - at.0).hypot(p.1 - at.1);
        let mut best = (0, false, f64::MAX);
        for (i, s) in left.iter().enumerate() {
            for (reversed, end) in [(false, s[0]), (true, s[s.len() - 1])] {
                if dist(end) < best.2 {
                    best = (i, reversed, dist(end));
                }
            }
        }
        let mut s = left.swap_remove(best.0);
        if best.1 {
            s.reverse();
        }
        at = s[s.len() - 1];
        out.push(s);
    }
    out
}

fn prepared(svg: &str, pen: &Pen) -> Result<Vec<Stroke>, String> {
    let (mut all, size) = strokes(svg)?;
    if let Some(paper) = pen.paper {
        on_paper(&mut all, size, paper);
    }
    Ok(if pen.keep_order { all } else { nearest_first(all) })
}

/// The figure as G-code, in mm.
pub fn gcode(svg: &str, pen: &Pen) -> Result<String, String> {
    let mut s = format!("G21\nG90\n{}\n", pen.up);
    for stroke in prepared(svg, pen)? {
        s.push_str(&format!("G0 X{:.3} Y{:.3}\n{}\n", stroke[0].0, stroke[0].1, pen.down));
        for p in &stroke[1..] {
            s.push_str(&format!("G1 X{:.3} Y{:.3} F{}\n", p.0, p.1, pen.feed));
        }
        s.push_str(&pen.up);
        s.push('\n');
    }
    s.push_str("G0 X0 Y0\n");
    Ok(s)
}

/// The figure as HPGL, in its units of 0.025 mm.
pub fn hpgl(svg: &str, pen: &Pen) -> Result<String, String> {
    let unit = |v: f64| (v * 40.0).round() as i64;
    let mut s = String::from("IN;SP1;\n");
    for stroke in prepared(svg, pen)? {
        s.push_str(&format!("PU{},{};PD", unit(stroke[0].0), unit(stroke[0].1)));
        let points: Vec<String> = stroke[1..].iter().map(|p| format!("{},{}", unit(p.0), unit(p.1))).collect();
        s.push_str(&points.join(","));
        s.push_str(";\n");
    }
    s.push_str("PU0,0;SP0;\n");
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO_LINES: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" width="100" height="100"><path d="M0 0 L10 0" stroke="black" fill="none"/><path d="M90 90 L90 80" stroke="black" fill="none"/><path d="M12 0 L20 0" stroke="black" fill="none"/></svg>"#;

    /// The pen's moves in a G-code: how far it draws and how far it travels up, in mm.
    fn lengths(code: &str) -> (f64, f64) {
        let (mut at, mut down, mut drawn, mut travel) = ((0.0f64, 0.0f64), false, 0.0, 0.0);
        for line in code.lines() {
            if line == "G0 Z0" {
                down = true;
            } else if line == "G0 Z5" {
                down = false;
            }
            let coordinate = |key: char| line.split(' ').find_map(|w| w.strip_prefix(key)).and_then(|v| v.parse::<f64>().ok());
            if let (Some(x), Some(y)) = (coordinate('X'), coordinate('Y')) {
                let d = (x - at.0).hypot(y - at.1);
                if line.starts_with("G1") {
                    assert!(down, "drawing with the pen up: {line}");
                    drawn += d;
                } else {
                    assert!(!down, "traveling with the pen down: {line}");
                    travel += d;
                }
                at = (x, y);
            }
        }
        (drawn, travel)
    }

    #[test]
    fn a_line_has_its_length_in_mm() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 96 96" width="96" height="96"><path d="M0 0 L96 0" stroke="black" fill="none"/></svg>"#;
        let (drawn, _) = lengths(&gcode(svg, &Pen::default()).unwrap());
        assert!((drawn - 25.4).abs() < 0.01, "{drawn}");
    }

    #[test]
    fn y_goes_up_from_the_lower_left_corner() {
        let (all, (_, h)) = strokes(TWO_LINES).unwrap();
        assert!((all[0][0].1 - h).abs() < 1e-3);
    }

    #[test]
    fn the_nearest_order_lifts_the_pen_less() {
        let kept = lengths(&gcode(TWO_LINES, &Pen { keep_order: true, ..Pen::default() }).unwrap());
        let near = lengths(&gcode(TWO_LINES, &Pen::default()).unwrap());
        assert!((kept.0 - near.0).abs() < 1e-3);
        assert!(near.1 < kept.1, "{near:?} {kept:?}");
    }

    #[test]
    fn a_fill_is_its_outline_and_a_hidden_group_is_nothing() {
        let square = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 96 96" width="96" height="96"><path d="M0 0 L96 0 L96 96 L0 96 Z" fill="red"/><g opacity="0"><path d="M0 0 L96 96" stroke="black"/></g></svg>"#;
        let (drawn, _) = lengths(&gcode(square, &Pen::default()).unwrap());
        assert!((drawn - 4.0 * 25.4).abs() < 0.01, "{drawn}");
    }

    #[test]
    fn hpgl_counts_in_its_units() {
        let code = hpgl(TWO_LINES, &Pen { keep_order: true, ..Pen::default() }).unwrap();
        assert!(code.starts_with("IN;SP1;\n") && code.ends_with("PU0,0;SP0;\n"));
        let first = code.lines().nth(1).unwrap();
        // 10 px are 2.646 mm, 106 units
        assert!(first.starts_with("PU0,") && first.contains(";PD106,"), "{first}");
    }

    #[test]
    fn the_figure_fits_the_paper() {
        let (all, _) = strokes(TWO_LINES).unwrap();
        let mut a = all;
        on_paper(&mut a, (26.458, 26.458), (13.229, 100.0));
        assert!(a.iter().flatten().all(|p| p.0 >= -0.01 && p.0 <= 13.3 && p.1 >= 0.0 && p.1 <= 100.0));
    }
}
