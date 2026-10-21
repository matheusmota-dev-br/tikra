//! Node shapes of the shape libraries: drawn without warnings, with their named anchors usable in
//! coordinates.

fn no_warnings(body: &str) -> String {
    let r = tikra::convert(&format!("\\begin{{tikzpicture}}{body}\\end{{tikzpicture}}")).unwrap();
    assert!(r.warnings.is_empty(), "{body}: {:?}", r.warnings);
    r.svg
}

#[test]
fn geometric_shapes_and_their_anchors() {
    for (shape, anchors) in [
        ("regular polygon, regular polygon sides=6", "corner 1,side 6"),
        ("star, star points=7, star point ratio=2", "outer point 7,inner point 1"),
        ("trapezium, trapezium left angle=70", "bottom left corner,top side"),
        ("semicircle", "apex,arc start,chord center"),
        ("isosceles triangle, shape border rotate=90", "apex,left corner,lower side"),
        ("kite, kite vertex angles=100 and 50", "upper vertex,lower left side"),
        ("dart", "tip,tail center,left tail"),
        ("circular sector, circular sector angle=90", "arc start,sector center,arc center"),
        ("cylinder, aspect=0.5", "top,before top,after bottom,shape center"),
    ] {
        let mut body = format!("\\node[draw, {shape}] (s) {{text}};");
        for a in anchors.split(',') {
            body.push_str(&format!("\\draw (0,2) -- (s.{a}) -- (s.30) -- (s.mid);"));
        }
        let svg = no_warnings(&body);
        assert!(svg.matches("<path").count() >= 2, "{shape}");
    }
}

#[test]
fn shape_names_alone_are_shapes() {
    no_warnings("\\node[trapezium, draw] {a}; \\node[regular polygon, draw] at (2,0) {b};");
}
