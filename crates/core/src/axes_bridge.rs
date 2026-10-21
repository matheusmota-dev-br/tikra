//! A bridge while pgfplots is rewritten on the new engine: an `axis` environment is drawn by the
//! earlier interpreter (`axes.rs` over `reader::Env`), and its scene is moved to the canvas.

use crate::canvas::{Fill, Item, Paint, Stroke};
use crate::engine::Engine;
use crate::scene::{Element, Point, Segment, Shape, Tip, Transform};
use crate::Error;

pub fn axis(e: &mut Engine, options: &str, body: &str) -> Result<(), Error> {
    let mut env = crate::reader::Env::new();
    env.colors = e.colors.clone();
    crate::axes::plot_axis(&mut env, options, body)?;
    for w in env.warnings {
        e.warn(w);
    }
    let t = e.st.transform;
    for el in env.scene.elements {
        match el {
            Element::Path(p) => {
                let segs = crate::canvas::transform_segments(&p.segments, &t);
                e.canvas.include_path(&segs);
                let s = &p.style;
                let stroke = s.stroke.map(|c| Stroke {
                    color: c,
                    width: s.line_width,
                    dash: s.dash.clone().map(|d| (d, 0.0)),
                    cap: crate::canvas::Cap::Butt,
                    join: crate::canvas::Join::Miter,
                    miter_limit: 10.0,
                    opacity: s.stroke_opacity,
                });
                let fill = s.fill.map(|c| Fill { paint: Paint::Solid(c), opacity: s.fill_opacity, even_odd: false });
                let mut segs = segs;
                let mut tips = Vec::new();
                if let Some(st) = &stroke {
                    if let Some(tip) = s.end_tip {
                        if let Some((pt, d)) = last_direction(&segs) {
                            let back = crate::engine::arrows::tip_items(tip_name(tip), "to", pt, d, st, &mut tips);
                            if back > 0.0 {
                                if let Some(Segment::LineTo(q)) = segs.last_mut() {
                                    let u = d.scale(1.0 / d.norm().max(1e-12));
                                    *q = q.minus(u.scale(back));
                                }
                            }
                        }
                    }
                }
                e.canvas.push(Item::Path { segments: segs, fill, stroke });
                e.canvas.items.extend(tips);
            }
            Element::Node(n) => {
                let center = t.apply(n.center);
                let (w, h) = (n.half_width, n.half_height);
                e.canvas.include(center.plus(Point::new(-w, -h)));
                e.canvas.include(center.plus(Point::new(w, h)));
                let rot = Transform::translation(center.x, center.y).compose(&Transform::rotation(n.rotation));
                if n.shape != Shape::Coordinate && (n.style.fill.is_some() || n.style.stroke.is_some()) {
                    let segs = vec![
                        Segment::MoveTo(Point::new(-w, -h)),
                        Segment::LineTo(Point::new(w, -h)),
                        Segment::LineTo(Point::new(w, h)),
                        Segment::LineTo(Point::new(-w, h)),
                        Segment::Close,
                    ];
                    let stroke = n.style.stroke.map(|c| Stroke {
                        color: c,
                        width: n.style.line_width,
                        dash: None,
                        cap: crate::canvas::Cap::Butt,
                        join: crate::canvas::Join::Miter,
                        miter_limit: 10.0,
                        opacity: 1.0,
                    });
                    let fill = n.style.fill.map(|c| Fill { paint: Paint::Solid(c), opacity: 1.0, even_odd: false });
                    e.canvas.push(Item::Path { segments: crate::canvas::transform_segments(&segs, &rot), fill, stroke });
                }
                if let Some(l) = n.label {
                    let total = l.height() + l.depth();
                    let place = rot.compose(&Transform::translation(-l.width() / 2.0, total / 2.0 - l.height()));
                    e.canvas.push(Item::Text { label: l, place, opacity: 1.0 });
                }
            }
            Element::Clip(segs) => {
                let segs = crate::canvas::transform_segments(&segs, &t);
                e.canvas.push(Item::Clip { segments: segs, even_odd: false });
            }
            Element::EndClip => e.canvas.push(Item::End),
        }
    }
    Ok(())
}

fn tip_name(t: Tip) -> &'static str {
    match t {
        Tip::Latex => "Latex",
        Tip::Stealth => "Stealth",
        Tip::Bar => "|",
        Tip::Circle => "o",
        Tip::Double => ">>",
        Tip::Default => "to",
    }
}

fn last_direction(segs: &[Segment]) -> Option<(Point, Point)> {
    let n = segs.len();
    if n < 2 {
        return None;
    }
    match (segs[n - 2], segs[n - 1]) {
        (Segment::MoveTo(a) | Segment::LineTo(a), Segment::LineTo(b)) => Some((b, b.minus(a))),
        _ => None,
    }
}
