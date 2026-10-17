//! Helpers shared by the integration tests.
#![allow(dead_code)]

use tikra::scene::{Element, Node, Path, Scene, Segment};

/// Converts and fails with the figure in the error text.
pub fn figure(tikz: &str) -> tikra::Output {
    tikra::convert(tikz).unwrap_or_else(|e| panic!("{e}\n{tikz}"))
}

/// Converts and requires zero warnings.
pub fn no_warnings(tikz: &str) -> tikra::Output {
    let r = figure(tikz);
    assert!(r.warnings.is_empty(), "warnings: {:?}\n{tikz}", r.warnings);
    r
}

/// Converts and requires a warning containing `fragment`.
pub fn with_warning(tikz: &str, fragment: &str) -> tikra::Output {
    let r = figure(tikz);
    assert!(r.warnings.iter().any(|a| a.contains(fragment)), "expected a warning with '{fragment}', got {:?}\n{tikz}", r.warnings);
    r
}

/// The scene of a tikzpicture body (without the environment around it).
pub fn scene(body: &str) -> Scene {
    let mut env = tikra::reader::Env::new();
    tikra::reader::Reader::new(body, &mut env).run(None).unwrap_or_else(|e| panic!("{e}\n{body}"));
    env.scene
}

/// The scene and warnings of a body.
pub fn scene_and_warnings(body: &str) -> (Scene, Vec<String>) {
    let mut env = tikra::reader::Env::new();
    tikra::reader::Reader::new(body, &mut env).run(None).unwrap_or_else(|e| panic!("{e}\n{body}"));
    (env.scene, env.warnings)
}

pub fn paths(c: &Scene) -> Vec<&Path> {
    c.elements.iter().filter_map(|e| if let Element::Path(t) = e { Some(t) } else { None }).collect()
}

pub fn nodes(c: &Scene) -> Vec<&Node> {
    c.elements.iter().filter_map(|e| if let Element::Node(n) = e { Some(n) } else { None }).collect()
}

pub fn first_path(body: &str) -> Path {
    let c = scene(body);
    paths(&c).first().cloned().cloned().unwrap_or_else(|| panic!("no path in {body}"))
}

pub fn first_node(body: &str) -> Node {
    let c = scene(body);
    nodes(&c).first().cloned().cloned().unwrap_or_else(|| panic!("no node in {body}"))
}

/// End points of each segment (Close has none).
pub fn points(segs: &[Segment]) -> Vec<(f64, f64)> {
    segs.iter()
        .filter_map(|s| match s {
            Segment::MoveTo(p) | Segment::LineTo(p) | Segment::CurveTo(_, _, p) => Some((p.x, p.y)),
            Segment::Close => None,
        })
        .collect()
}

pub fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

pub fn close_to(p: (f64, f64), q: (f64, f64)) -> bool {
    close(p.0, q.0) && close(p.1, q.1)
}
