//! The TikZ libraries made of styles and small additions: automata, petri, er, shadows, through,
//! spy, 3d, perspective and the matrix library's delimiters.

use tikra::canvas::{Item, Paint};
use tikra::scene::{Point, Segment};
use tikra::PT;

/// What the engine draws for a picture body, without warnings.
fn items(body: &str) -> Vec<Item> {
    let (canvas, warnings) = tikra::engine::run_document(&format!("\\begin{{tikzpicture}}{body}\\end{{tikzpicture}}")).unwrap();
    assert!(warnings.is_empty(), "{body}: {warnings:?}");
    canvas.items
}

fn paths(body: &str) -> Vec<Vec<Segment>> {
    items(body).into_iter().filter_map(|i| if let Item::Path { segments, .. } = i { Some(segments) } else { None }).collect()
}

fn near(p: Point, x: f64, y: f64) -> bool {
    (p.x - x).abs() < 1e-3 && (p.y - y).abs() < 1e-3
}

fn point(s: &Segment) -> Point {
    match *s {
        Segment::MoveTo(p) | Segment::LineTo(p) | Segment::CurveTo(_, _, p) => p,
        Segment::Close => panic!("close"),
    }
}

fn first_point(s: &[Segment]) -> Point {
    point(&s[0])
}

#[test]
fn an_initial_state_has_an_arrow_from_the_left() {
    // the arrow starts 3ex left of the state's west side, and ends at its border
    let p = paths(r"\node[state,initial] (q) {$q$};");
    let arrow = p.iter().find(|s| matches!(s.last(), Some(Segment::LineTo(_)))).expect("the initial arrow");
    let (a, b) = (point(&arrow[arrow.len() - 2]), point(&arrow[arrow.len() - 1]));
    assert!(a.x < b.x && (a.y - b.y).abs() < 1e-9, "{arrow:?}");
}

#[test]
fn an_accepting_state_is_drawn_double() {
    let strokes: Vec<f64> =
        items(r"\node[state,accepting] {$q$};").into_iter().filter_map(|i| if let Item::Path { stroke: Some(s), .. } = i { Some(s.width / PT) } else { None }).collect();
    // the outer line, 2 × 0.4pt + 0.6pt, then the white core
    assert!(strokes.iter().any(|w| (w - 1.4).abs() < 1e-6) && strokes.iter().any(|w| (w - 0.6).abs() < 1e-6), "{strokes:?}");
}

#[test]
fn on_grid_places_centers_at_the_node_distance() {
    let p = paths(r"\node[draw] (a) {A}; \node[draw,on grid,right=2cm of a] (b) {Longer}; \draw (a.center) -- (b.center);");
    let line = p.last().unwrap();
    assert!(near(point(&line[1]).minus(point(&line[0])), 2.0, 0.0), "{line:?}");
}

#[test]
fn tokens_are_placed_around_the_center_of_a_place() {
    // the place, then two tokens at ±.5 × 1.5ex (by their rightmost points)
    let p = paths(r"\node[place,tokens=2] at (0,0) {};");
    let ex = 4.3 * PT;
    let centers: Vec<f64> = p[1..].iter().map(|s| first_point(s).x - first_point(&p[0]).x).collect();
    assert_eq!(centers.len(), 2, "{p:?}");
    assert!(centers[0] < 0.0 && (centers[1] - centers[0] - 1.5 * ex).abs() < 1e-3, "{centers:?}");
}

#[test]
fn er_shapes_are_styles() {
    assert_eq!(paths(r"\node[entity] {E}; \node[relationship] at (2,0) {R}; \node[attribute] at (4,0) {A};").len(), 3);
}

#[test]
fn a_drop_shadow_is_a_shifted_copy_behind() {
    let it = items(r"\node[fill=white,drop shadow] {A};");
    let fills: Vec<(Point, f64)> = it
        .iter()
        .filter_map(|i| match i {
            Item::Path { segments, fill: Some(f), .. } => Some((first_point(segments), f.opacity)),
            _ => None,
        })
        .collect();
    // first the shadow, .5ex right and down, half transparent
    let (shadow, node) = (fills[0], fills[1]);
    let d = shadow.0.minus(node.0);
    assert!(near(d, 0.5 * 4.3 * PT, -0.5 * 4.3 * PT) && shadow.1 == 0.5, "{fills:?}");
}

#[test]
fn a_copy_shadow_has_the_colors_of_the_node() {
    let it = items(r"\node[copy shadow,fill=red,draw=blue] {A};");
    match &it[0] {
        Item::Path { fill: Some(f), stroke: Some(s), .. } => {
            assert_eq!(f.paint, Paint::Solid(tikra::color::Color { r: 1.0, g: 0.0, b: 0.0 }));
            assert_eq!(s.color, tikra::color::Color { r: 0.0, g: 0.0, b: 1.0 });
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn options_of_a_coordinate_move_a_node_anchor() {
    let p = paths(r"\node (a) at (1,1) {}; \draw (a.center) -- ([shift={(1,0)}]a.center) -- ([xshift=1cm,yshift=1cm]a.center);");
    let (a, b, c) = (point(&p[0][0]), point(&p[0][1]), point(&p[0][2]));
    assert!(near(b.minus(a), 1.0, 0.0) && near(c.minus(a), 1.0, 1.0), "{p:?}");
}
