//! Arrow tips: reading specifications (`->`, `<->`, `-{Latex[length=3mm]}`, `|<->|`, `->>`) and
//! drawing the tips at the ends of a path.

use crate::canvas::{Cap, Fill, Item, Join, Paint, Stroke};
use crate::scene::{Point, Segment};
use crate::PT;

/// `<start>-<end>`: the tips at each end (None for none). None if the text is not an arrow specification.
pub fn parse_arrows(spec: &str) -> Option<(Option<String>, Option<String>)> {
    let c: Vec<char> = spec.trim().chars().collect();
    let mut level = 0;
    let mut cut = None;
    for (i, ch) in c.iter().enumerate() {
        match ch {
            '{' | '[' => level += 1,
            '}' | ']' => level -= 1,
            '-' if level == 0 => {
                cut = Some(i);
                break;
            }
            _ => {}
        }
    }
    let i = cut?;
    let a: String = c[..i].iter().collect();
    let b: String = c[i + 1..].iter().collect();
    let side = |s: &str| -> Option<Option<String>> {
        let s = s.trim();
        if s.is_empty() {
            return Some(None);
        }
        if s.split(['[', '{', '}']).next().is_some_and(|name| tip_known(name.trim()) || name.trim().is_empty()) || s.chars().all(|ch| "<>|o*()[]".contains(ch)) {
            return Some(Some(s.to_string()));
        }
        None
    };
    let start = side(&a)?;
    let end = side(&b)?;
    if start.is_none() && end.is_none() && !spec.trim().starts_with('-') {
        return None;
    }
    Some((start, end))
}

fn tip_known(name: &str) -> bool {
    matches!(
        name,
        ">" | "<"
            | ">>"
            | "<<"
            | "to"
            | "to reversed"
            | "latex"
            | "Latex"
            | "latex'"
            | "Triangle"
            | "triangle 45"
            | "triangle 60"
            | "triangle 90"
            | "stealth"
            | "Stealth"
            | "stealth'"
            | "Straight Barb"
            | "Kite"
            | "|"
            | "Bar"
            | "o"
            | "*"
            | "Circle"
            | "Classical TikZ Rightarrow"
            | "Computer Modern Rightarrow"
            | "Rays"
            | "Square"
            | "Diamond"
            | "Arc Barb"
            | "Hooks"
            | "Tee Barb"
            | "Bracket"
            | "Parenthesis"
            | "Implies"
            | "Turned Square"
            | "Ellipse"
            | "Rectangle"
            | "|<"
            | ">|"
            | "<|"
            | "|>"
            | "spaced"
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tip {
    To,
    Latex,
    Stealth,
    Bar,
    Circle,
    Double,
}

fn resolve(spec: &str, default: &str) -> Option<Tip> {
    let s = spec.trim().trim_start_matches('{').trim_end_matches('}');
    let name = s.split('[').next().unwrap_or("").trim();
    let name = if matches!(name, ">" | "<") { default.trim() } else { name };
    Some(match name {
        "to" | "" | "Computer Modern Rightarrow" | "Classical TikZ Rightarrow" => Tip::To,
        "latex" | "Latex" | "latex'" | "Triangle" | "triangle 45" | "triangle 60" | "triangle 90" => Tip::Latex,
        "stealth" | "Stealth" | "stealth'" | "Straight Barb" | "Kite" => Tip::Stealth,
        "|" | "Bar" | "Tee Barb" => Tip::Bar,
        ">>" | "<<" => Tip::Double,
        "o" | "Circle" | "*" => Tip::Circle,
        _ => Tip::To,
    })
}

fn tip_length(tip: Tip, lw: f64) -> f64 {
    match tip {
        Tip::Latex => 3.0 * PT + 2.5 * lw,
        Tip::Stealth => 3.0 * PT + 3.0 * lw,
        Tip::To | Tip::Double => 1.8 * PT + 2.0 * lw,
        Tip::Bar | Tip::Circle => 1.5 * PT + 1.5 * lw,
    }
}

/// The tip's drawing with its point at `p`, pointing along `d`, and how much to pull the line back.
pub fn tip_items(spec: &str, default: &str, p: Point, d: Point, stroke: &Stroke, items: &mut Vec<Item>) -> f64 {
    let Some(tip) = resolve(spec, default) else { return 0.0 };
    let lw = stroke.width;
    let len = tip_length(tip, lw);
    let u = d.scale(1.0 / d.norm().max(1e-12));
    let nrm = Point::new(-u.y, u.x);
    let solid = |segments: Vec<Segment>| Item::Path { segments, fill: Some(Fill { paint: Paint::Solid(stroke.color), opacity: stroke.opacity, even_odd: false }), stroke: None };
    let line = |segments: Vec<Segment>| Item::Path { segments, fill: None, stroke: Some(Stroke { dash: None, cap: Cap::Round, join: Join::Round, ..stroke.clone() }) };
    match tip {
        Tip::Latex => {
            let base = p.minus(u.scale(len));
            let w = len * 0.42;
            items.push(solid(vec![Segment::MoveTo(p), Segment::LineTo(base.plus(nrm.scale(w))), Segment::LineTo(base.minus(nrm.scale(w))), Segment::Close]));
            len * 0.85
        }
        Tip::Stealth => {
            let base = p.minus(u.scale(len));
            let w = len * 0.45;
            let mid = p.minus(u.scale(len * 0.7));
            items.push(solid(vec![Segment::MoveTo(p), Segment::LineTo(base.plus(nrm.scale(w))), Segment::LineTo(mid), Segment::LineTo(base.minus(nrm.scale(w))), Segment::Close]));
            len * 0.85
        }
        Tip::To => {
            let base = p.minus(u.scale(len));
            let w = len * 0.75;
            items.push(line(vec![Segment::MoveTo(base.plus(nrm.scale(w))), Segment::LineTo(p), Segment::LineTo(base.minus(nrm.scale(w)))]));
            0.0
        }
        Tip::Double => {
            for q in [p, p.minus(u.scale(len * 0.8))] {
                let base = q.minus(u.scale(len));
                let w = len * 0.75;
                items.push(line(vec![Segment::MoveTo(base.plus(nrm.scale(w))), Segment::LineTo(q), Segment::LineTo(base.minus(nrm.scale(w)))]));
            }
            0.0
        }
        Tip::Bar => {
            items.push(Item::Path {
                segments: vec![Segment::MoveTo(p.plus(nrm.scale(len))), Segment::LineTo(p.minus(nrm.scale(len)))],
                fill: None,
                stroke: Some(Stroke { dash: None, ..stroke.clone() }),
            });
            0.0
        }
        Tip::Circle => {
            let c = p.minus(u.scale(len / 2.0));
            let r = len / 2.0;
            let k = 0.5523 * r;
            let q = |x: f64, y: f64| c.plus(Point::new(x, y));
            items.push(solid(vec![
                Segment::MoveTo(q(r, 0.0)),
                Segment::CurveTo(q(r, k), q(k, r), q(0.0, r)),
                Segment::CurveTo(q(-k, r), q(-r, k), q(-r, 0.0)),
                Segment::CurveTo(q(-r, -k), q(-k, -r), q(0.0, -r)),
                Segment::CurveTo(q(k, -r), q(r, -k), q(r, 0.0)),
                Segment::Close,
            ]));
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specifications() {
        assert_eq!(parse_arrows("->"), Some((None, Some(">".into()))));
        assert_eq!(parse_arrows("<->"), Some((Some("<".into()), Some(">".into()))));
        assert_eq!(parse_arrows("-{Latex[length=3mm]}"), Some((None, Some("{Latex[length=3mm]}".into()))));
        assert_eq!(parse_arrows("|-|"), Some((Some("|".into()), Some("|".into()))));
        assert_eq!(parse_arrows("-"), Some((None, None)));
        assert_eq!(parse_arrows("red"), None);
        assert_eq!(parse_arrows("north east"), None);
        assert_eq!(parse_arrows("x-y"), None);
    }
}
