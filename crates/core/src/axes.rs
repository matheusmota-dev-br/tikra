//! A pgfplots subset: `\begin{axis}[…] \addplot[…] {expr}; \addplot coordinates {…}; \addlegendentry{…}`.
//! Draws the frame or the axes, ticks at round values, the curves clipped to the window, and the legend.

use crate::color::Color;
use crate::options;
use crate::reader::{block, format_number, Env};
use crate::scene::*;
use crate::{path, Error};

struct Series {
    points: Vec<(f64, f64)>,
    options: String,
    marks: bool,
    /// false with `only marks`: the points are not joined
    line: bool,
}

/// A plot mark: a small filled circle in the series' color.
fn mark(center: Point, e: &Style) -> Node {
    Node { center, shape: Shape::Circle, half_width: 0.06, half_height: 0.06, label: None, style: Style { fill: e.stroke, stroke: e.stroke, ..Style::default() }, rotation: 0.0 }
}

fn round_step(range: f64) -> f64 {
    let raw = range / 5.0;
    let p = 10f64.powf(raw.abs().log10().floor());
    let r = raw / p;
    p * if r < 1.5 { 1.0 } else if r < 3.0 { 2.0 } else if r < 7.0 { 5.0 } else { 10.0 }
}

/// Axis ticks: multiples of a round step, by index. In windows where the step vanishes in f64
/// precision (xmin=1e17, xmax=1e17+1), only what can be told apart is kept, capped at 50.
fn round_values(min: f64, max: f64) -> Vec<f64> {
    let step = round_step(max - min);
    if !(step > 0.0 && step.is_finite()) {
        return Vec::new();
    }
    let first = (min / step).ceil();
    let mut out: Vec<f64> = Vec::new();
    for k in 0..50 {
        let v = (first + f64::from(k)) * step;
        if v > max + 1e-9 {
            break;
        }
        if out.last() != Some(&v) {
            out.push(v);
        }
    }
    out
}

pub fn plot_axis(env: &mut Env, options_text: &str, body: &str) -> Result<(), Error> {
    let o = options::parse(&env.expand(options_text));
    let get = |k: &str| o.iter().find(|x| x.key == k).and_then(|x| x.value.clone());
    let width = get("width").and_then(|v| env.length(&v).ok()).unwrap_or(240.0 * crate::PT);
    let height = get("height").and_then(|v| env.length(&v).ok()).unwrap_or(207.0 * crate::PT);
    let samples = get("samples").and_then(|v| env.number(&v).ok()).unwrap_or(25.0) as usize;
    let default_domain = get("domain").and_then(|d| {
        let (a, b) = d.split_once(':')?;
        Some((env.number(a).ok()?, env.number(b).ok()?))
    });
    let axis_lines = get("axis lines").unwrap_or_else(|| "box".into());

    // series
    let c: Vec<char> = env.expand(body).chars().collect();
    let mut series: Vec<Series> = Vec::new();
    let mut legends: Vec<String> = Vec::new();
    let mut i = 0;
    while i < c.len() {
        let rest: String = c[i..].iter().take(16).collect();
        if rest.starts_with("\\addplot") {
            i += 8;
            let plus = c.get(i) == Some(&'+');
            if plus {
                i += 1;
            }
            while i < c.len() && c[i].is_whitespace() {
                i += 1;
            }
            let mut po = String::new();
            if c.get(i) == Some(&'[') {
                let (x, end) = block(&c, i).unwrap_or_default();
                po = x;
                i = end;
            }
            while i < c.len() && c[i].is_whitespace() {
                i += 1;
            }
            let next: String = c[i..].iter().take(11).collect();
            let mut points = Vec::new();
            let mut marks = po.is_empty() || plus;
            if next.starts_with("coordinates") {
                i += 11;
                while i < c.len() && c[i].is_whitespace() {
                    i += 1;
                }
                let (list, end) = block(&c, i).ok_or(Error::Figure("\\addplot coordinates without a list".into()))?;
                i = end;
                let lc: Vec<char> = list.chars().collect();
                let mut k = 0;
                while k < lc.len() {
                    if lc[k] == '(' {
                        let (d, f) = block(&lc, k).unwrap_or_default();
                        let parts = options::split_top_level(&d);
                        if parts.len() >= 2 {
                            if let (Ok(x), Ok(y)) = (env.number(&parts[0]), env.number(&parts[1])) {
                                points.push((x, y));
                            }
                        }
                        k = f;
                    } else {
                        k += 1;
                    }
                }
            } else if c.get(i) == Some(&'{') {
                let (e, end) = block(&c, i).unwrap_or_default();
                i = end;
                marks = false;
                let series_options = options::parse(&po);
                let (d0, d1) = series_options
                    .iter()
                    .find(|x| x.key == "domain")
                    .and_then(|x| x.value.as_ref())
                    .and_then(|d| {
                        let (a, b) = d.split_once(':')?;
                        Some((env.number(a).ok()?, env.number(b).ok()?))
                    })
                    .or(default_domain)
                    .unwrap_or((-5.0, 5.0));
                let n = series_options.iter().find(|x| x.key == "samples").and_then(|x| x.value.as_ref()).and_then(|v| env.number(v).ok()).map_or(samples, |v| v as usize).clamp(2, 2000);
                for k in 0..n {
                    let x = d0 + (d1 - d0) * k as f64 / (n - 1) as f64;
                    env.vars.insert("x".into(), x);
                    if let Ok(y) = env.number(&e) {
                        if y.is_finite() {
                            points.push((x, y));
                        }
                    }
                }
                env.vars.values.remove("x");
            }
            while i < c.len() && c[i] != ';' {
                i += 1;
            }
            let own = options::parse(&po);
            let has = |k: &str| own.iter().any(|o| o.key == k);
            let only_marks = has("only marks");
            if only_marks || own.iter().any(|o| o.key == "mark" && o.value.as_deref().map(str::trim) != Some("none")) {
                marks = true;
            }
            if has("no marks") {
                marks = false;
            }
            series.push(Series { points, options: po, marks, line: !only_marks });
        } else if rest.starts_with("\\addlegendentry") {
            i += 15;
            while i < c.len() && c[i].is_whitespace() {
                i += 1;
            }
            let (t, end) = block(&c, i).unwrap_or_default();
            legends.push(t);
            i = end;
        } else {
            i += 1;
        }
    }

    // window
    let all: Vec<(f64, f64)> = series.iter().flat_map(|s| s.points.iter().copied()).collect();
    let auto = |f: fn(&(f64, f64)) -> f64, min: bool| -> f64 {
        let it = all.iter().map(f);
        if min { it.fold(f64::INFINITY, f64::min) } else { it.fold(f64::NEG_INFINITY, f64::max) }
    };
    let num = |k: &str| get(k).and_then(|v| env.number(&v).ok());
    let xmin = num("xmin").unwrap_or_else(|| auto(|p| p.0, true));
    let xmax = num("xmax").unwrap_or_else(|| auto(|p| p.0, false));
    let ymin = num("ymin").unwrap_or_else(|| auto(|p| p.1, true));
    let ymax = num("ymax").unwrap_or_else(|| auto(|p| p.1, false));
    // without data, pgfplots draws the default window from 0 to 1
    let (xmin, xmax) = if xmin.is_finite() && xmax > xmin { (xmin, xmax) } else { (0.0, 1.0) };
    let (ymin, ymax) = if ymin.is_finite() && ymax > ymin { (ymin, ymax) } else { (0.0, 1.0) };
    if all.is_empty() {
        env.warn("plot without data: drew the default window from 0 to 1");
    }
    let map = |x: f64, y: f64| Point::new((x - xmin) / (xmax - xmin) * width, (y - ymin) / (ymax - ymin) * height);
    let black = Style { stroke: Some(Color::BLACK), ..Style::default() };

    // axes
    let mut axis = |segs: Vec<Segment>, arrows: bool| {
        let mut e = black.clone();
        if arrows {
            e.end_tip = Some(Tip::Latex);
        }
        env.scene.elements.push(Element::Path(Path { segments: segs, style: e }));
    };
    match axis_lines.as_str() {
        "left" => {
            axis(vec![Segment::MoveTo(map(xmin, ymin)), Segment::LineTo(Point::new(width + 0.3, 0.0))], true);
            axis(vec![Segment::MoveTo(map(xmin, ymin)), Segment::LineTo(Point::new(0.0, height + 0.3))], true);
        }
        "middle" | "center" => {
            let y0 = map(0.0, 0.0).y.clamp(0.0, height);
            let x0 = map(0.0, 0.0).x.clamp(0.0, width);
            axis(vec![Segment::MoveTo(Point::new(0.0, y0)), Segment::LineTo(Point::new(width + 0.3, y0))], true);
            axis(vec![Segment::MoveTo(Point::new(x0, 0.0)), Segment::LineTo(Point::new(x0, height + 0.3))], true);
        }
        "none" => {}
        _ => axis(
            vec![Segment::MoveTo(Point::new(0.0, 0.0)), Segment::LineTo(Point::new(width, 0.0)), Segment::LineTo(Point::new(width, height)), Segment::LineTo(Point::new(0.0, height)), Segment::Close],
            false,
        ),
    }

    // ticks and values
    let tick = 0.1;
    for x in round_values(xmin, xmax) {
        let p = map(x, ymin);
        env.scene.elements.push(Element::Path(Path { segments: vec![Segment::MoveTo(p), Segment::LineTo(p.plus(Point::new(0.0, tick)))], style: black.clone() }));
        path::statement(env, "path", &format!("({},{}) node[below] {{${}$}}", p.x, p.y, format_number(x)))?;
    }
    for y in round_values(ymin, ymax) {
        let p = map(xmin, y);
        env.scene.elements.push(Element::Path(Path { segments: vec![Segment::MoveTo(p), Segment::LineTo(p.plus(Point::new(tick, 0.0)))], style: black.clone() }));
        path::statement(env, "path", &format!("({},{}) node[left] {{${}$}}", p.x, p.y, format_number(y)))?;
    }
    if let Some(t) = get("xlabel") {
        path::statement(env, "path", &format!("({},{}) node[below=0.55cm] {{{t}}}", width / 2.0, 0.0))?;
    }
    if let Some(t) = get("ylabel") {
        path::statement(env, "path", &format!("({},{}) node[left=0.75cm, rotate=90, anchor=south] {{{t}}}", 0.0, height / 2.0))?;
    }
    if let Some(t) = get("title") {
        path::statement(env, "path", &format!("({},{}) node[above] {{{t}}}", width / 2.0, height))?;
    }

    // series, clipped to the window
    let cycle = ["blue", "red", "brown!60!black", "black", "teal", "orange", "violet"];
    env.scene.elements.push(Element::Clip(vec![
        Segment::MoveTo(Point::new(0.0, 0.0)),
        Segment::LineTo(Point::new(width, 0.0)),
        Segment::LineTo(Point::new(width, height)),
        Segment::LineTo(Point::new(0.0, height)),
        Segment::Close,
    ]));
    let mut series_styles = Vec::new();
    for (k, s) in series.iter().enumerate() {
        let mut o = if s.options.trim().is_empty() || s.marks { format!("{},", cycle[k % cycle.len()]) } else { String::new() };
        o.push_str(&s.options);
        let series_options = env.styles.expand(options::parse(&o));
        let e = path::style(env, &series_options, "draw");
        series_styles.push(e.clone());
        if s.line && s.points.len() >= 2 {
            let mut segs = vec![Segment::MoveTo(map(s.points[0].0, s.points[0].1))];
            segs.extend(s.points[1..].iter().map(|p| Segment::LineTo(map(p.0, p.1))));
            env.scene.elements.push(Element::Path(Path { segments: segs, style: e.clone() }));
        }
        if s.marks {
            for p in &s.points {
                env.scene.elements.push(Element::Node(mark(map(p.0, p.1), &e)));
            }
        }
    }
    env.scene.elements.push(Element::EndClip);

    // legend
    if !legends.is_empty() {
        // `legend pos` first, then `at` and `anchor` in `legend style` override it
        let (mut at, mut anchor) = match get("legend pos").as_deref().map(str::trim) {
            Some("north west") => ((0.03, 0.97), "north west".to_string()),
            Some("south west") => ((0.03, 0.03), "south west".to_string()),
            Some("south east") => ((0.97, 0.03), "south east".to_string()),
            Some("outer north east") => ((1.03, 1.0), "north west".to_string()),
            _ => ((0.97, 0.97), "north east".to_string()),
        };
        if let Some(ls) = get("legend style") {
            let o = options::parse(&ls);
            let point = o.iter().find(|x| x.key == "at").and_then(|x| x.value.clone()).and_then(|a| {
                let t = options::strip_braces(&a).trim().trim_start_matches('(').trim_end_matches(')').to_string();
                let p = options::split_top_level(&t);
                Some((env.number(&p[0]).ok()?, env.number(p.get(1)?).ok()?))
            });
            if let Some(p) = point {
                at = p;
                // pgfplots keeps its default anchor when only `at` is given
                anchor = "north east".to_string();
            }
            if let Some(a) = o.iter().find(|x| x.key == "anchor").and_then(|x| x.value.clone()) {
                anchor = a.trim().to_string();
            }
        }
        let (ax, ay) = at;
        let mut y = ay * height - 0.3;
        let x_end = ax * width - 0.15;
        let before = env.scene.elements.len();
        for (k, t) in legends.iter().enumerate() {
            let e = series_styles.get(k).cloned().unwrap_or_else(|| black.clone());
            path::statement(env, "path", &format!("({},{}) node[left] {{{t}}}", x_end, y))?;
            let text_start = env.scene.elements.iter().rev().find_map(|el| if let Element::Node(n) = el { Some(n.center.x - n.half_width) } else { None }).unwrap_or(x_end);
            if series.get(k).is_none_or(|s| s.line) {
                env.scene.elements.push(Element::Path(Path {
                    segments: vec![Segment::MoveTo(Point::new(text_start - 0.7, y)), Segment::LineTo(Point::new(text_start - 0.1, y))],
                    style: Style { start_tip: None, end_tip: None, ..e },
                }));
            } else {
                // a series of marks only shows a mark in the legend
                env.scene.elements.push(Element::Node(mark(Point::new(text_start - 0.4, y), &e)));
            }
            y -= 0.5;
        }
        // legend frame, around what was just drawn
        let mut x0 = f64::INFINITY;
        // after `before` there are only the legend texts and its stroke samples
        for el in &env.scene.elements[before..] {
            if let Element::Node(n) = el {
                x0 = x0.min(n.center.x - n.half_width);
            }
            if let Element::Path(t) = el {
                for s in &t.segments {
                    if let Segment::MoveTo(p) | Segment::LineTo(p) = s {
                        x0 = x0.min(p.x);
                    }
                }
            }
        }
        let (top, bottom, right) = (ay * height - 0.05, y + 0.25, x_end + 0.05);
        let x0 = x0 - 0.1;
        env.scene.elements.insert(
            before,
            Element::Path(Path {
                segments: vec![
                    Segment::MoveTo(Point::new(x0, bottom)),
                    Segment::LineTo(Point::new(right, bottom)),
                    Segment::LineTo(Point::new(right, top)),
                    Segment::LineTo(Point::new(x0, top)),
                    Segment::Close,
                ],
                style: Style { stroke: Some(Color::BLACK), fill: Some(Color::WHITE), ..Style::default() },
            }),
        );
        // the legend was laid out with its north east corner at `at`: move it so `anchor` lands there instead
        let (w, h) = (right - x0, top - bottom);
        let (dx, dy) = match anchor.as_str() {
            "north west" => (w, 0.0),
            "north" => (w / 2.0, 0.0),
            "south west" => (w, h),
            "south" => (w / 2.0, h),
            "south east" => (0.0, h),
            "west" => (w, h / 2.0),
            "east" => (0.0, h / 2.0),
            "center" => (w / 2.0, h / 2.0),
            "north east" => (0.0, 0.0),
            other => {
                env.warn(format!("legend anchor {other} not supported: north east used"));
                (0.0, 0.0)
            }
        };
        for el in &mut env.scene.elements[before..] {
            shift(el, dx, dy);
        }
    }
    Ok(())
}

/// Moves an element by `(dx, dy)`, in cm.
fn shift(el: &mut Element, dx: f64, dy: f64) {
    let d = Point::new(dx, dy);
    match el {
        Element::Node(n) => n.center = n.center.plus(d),
        Element::Path(Path { segments, .. }) | Element::Clip(segments) => {
            for s in segments {
                match s {
                    Segment::MoveTo(q) | Segment::LineTo(q) => *q = q.plus(d),
                    Segment::CurveTo(a, b, c) => {
                        for q in [a, b, c] {
                            *q = q.plus(d);
                        }
                    }
                    Segment::Close => {}
                }
            }
        }
        Element::EndClip => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_moves_every_kind_of_element() {
        let o = Point::new(0.0, 0.0);
        let mut els = vec![
            Element::Clip(vec![Segment::MoveTo(o), Segment::CurveTo(o, o, o), Segment::Close]),
            Element::EndClip,
        ];
        for el in &mut els {
            shift(el, 1.0, 2.0);
        }
        let Element::Clip(segs) = &els[0] else { panic!() };
        assert!(matches!(segs[1], Segment::CurveTo(a, b, c) if [a, b, c].iter().all(|p| p.x == 1.0 && p.y == 2.0)));
        assert!(matches!(segs[0], Segment::MoveTo(p) if p.x == 1.0 && p.y == 2.0));
    }
}
