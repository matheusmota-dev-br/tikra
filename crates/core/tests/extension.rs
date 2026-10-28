//! Libraries from outside tikra: their keys and definitions, used through the public API only.

use tikra::extension::{Context, Error, Library, Tokens};
use tikra::Options;

/// `twice={options}`: the options, then `line width` doubled.
fn twice(cx: &mut Context, value: Option<Tokens>) -> Result<(), Error> {
    cx.keys(value.unwrap_or_default())?;
    let w = cx.line_width();
    cx.keys(Tokens::text(&format!("line width={}cm", 2.0 * w)))
}

/// `midpoint={A}{B}{name}`: a coordinate halfway between two others.
fn midpoint(cx: &mut Context, value: Option<Tokens>) -> Result<(), Error> {
    let a = value.unwrap_or_default().arguments(3);
    let (p, q, name) = (cx.expand(a[0].clone())?, cx.expand(a[1].clone())?, cx.expand(a[2].clone())?);
    let at = cx.point(&p)?.plus(cx.point(&q)?).scale(0.5);
    cx.run(Tokens::text(&format!("\\coordinate ({name}) at ({}cm,{}cm);", at.x, at.y)))
}

/// `complain`: a warning, and nothing drawn.
fn complain(cx: &mut Context, _value: Option<Tokens>) -> Result<(), Error> {
    cx.warn("complain: nothing to do");
    Ok(())
}

fn definitions() -> String {
    r"\tikzset{road/.style={twice={draw=gray}}}\def\tikramid{midpoint}".into()
}

static ROADS: Library = Library { name: "roads", definitions, keys: &[("/tikz/twice", twice), ("/tikz/midpoint", midpoint), ("/tikz/complain", complain)] };

fn convert(body: &str, libraries: &[&'static Library]) -> tikra::Output {
    tikra::convert_with_libraries(&format!("\\begin{{tikzpicture}}{body}\\end{{tikzpicture}}"), &Options::default(), libraries).unwrap()
}

#[test]
fn a_library_key_runs_and_its_styles_are_defined() {
    let with = convert(r"\draw[road] (0,0) -- (2,0);", &[&ROADS]);
    assert!(with.warnings.is_empty(), "{:?}", with.warnings);
    assert!(with.svg.contains("stroke-width=\"0.797\""), "{}", with.svg);
}

#[test]
fn without_the_library_its_keys_are_unknown() {
    let without = convert(r"\draw[road] (0,0) -- (2,0);", &[]);
    assert!(!without.warnings.is_empty());
    // and the library used before does not stay in the engine
    convert(r"\draw[road] (0,0) -- (2,0);", &[&ROADS]);
    assert!(!convert(r"\draw[twice] (0,0) -- (1,0);", &[]).warnings.is_empty());
}

#[test]
fn a_library_key_reads_points_and_runs_code() {
    let r = convert(r"\path[midpoint={0,0}{4,2}{M}]; \fill (M) circle (2pt);", &[&ROADS]);
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
}

#[test]
fn a_library_key_warns() {
    let r = convert(r"\draw[complain] (0,0) -- (1,0);", &[&ROADS]);
    assert!(r.warnings.iter().any(|w| w.contains("complain: nothing to do")), "{:?}", r.warnings);
}

#[test]
fn tokens_split_as_tex_does() {
    let a = Tokens::text("{a}{b c}").arguments(3);
    assert_eq!(a.iter().map(Tokens::to_text).collect::<Vec<_>>(), ["a", "b c", ""]);
    let (opts, rest) = Tokens::text(" [red, thick] $x$").leading_options();
    assert_eq!((opts.to_text().as_str(), rest.to_text().trim()), ("red, thick", "$x$"));
    let keys: Vec<String> = Tokens::text("pos=.3, near end").key_list().into_iter().map(|(k, _)| k).collect();
    assert_eq!(keys, ["pos", "near end"]);
}

use tikra::extension::{NodeShape, PathSegment, Point, ShapeBorder, ShapeInput, ShapePaint, ShapePath};

/// `flag`: a pole from the node's point up 1 cm with a triangle at its top, filled with the node's
/// fill and outlined twice as wide as the lines; a solid dot at the foot. Its anchors are `foot`
/// and `top`, its border an outline, the text right of the pole.
fn flag(cx: &mut Context, input: &ShapeInput) -> Result<NodeShape, Error> {
    let w = 2.0 * input.line_width;
    let p = Point::new;
    let triangle = vec![PathSegment::MoveTo(p(0.0, 1.0)), PathSegment::LineTo(p(0.6, 0.85)), PathSegment::LineTo(p(0.0, 0.7)), PathSegment::Close];
    // a key of the library, read while the shape is made
    let size = cx.number(Tokens::text("\\flagdot"))?;
    let k = 0.55 * size;
    let dot = vec![
        PathSegment::MoveTo(p(size, 0.0)),
        PathSegment::CurveTo(p(size, k), p(k, size), p(0.0, size)),
        PathSegment::CurveTo(p(-k, size), p(-size, k), p(-size, 0.0)),
        PathSegment::CurveTo(p(-size, -k), p(-k, -size), p(0.0, -size)),
        PathSegment::CurveTo(p(k, -size), p(size, -k), p(size, 0.0)),
        PathSegment::Close,
    ];
    Ok(NodeShape {
        center: p(0.0, 0.5),
        anchors: vec![("foot".into(), p(0.0, 0.0)), ("top".into(), p(0.0, 1.0))],
        border: ShapeBorder::Outline(vec![p(-0.1, 0.0), p(0.6, 0.0), p(0.6, 1.0), p(-0.1, 1.0)]),
        text: p(0.7, 0.5 - input.height / 2.0),
        paths: vec![
            ShapePath { segments: triangle.clone(), paint: ShapePaint::Fill },
            ShapePath { segments: vec![PathSegment::MoveTo(p(0.0, 0.0)), PathSegment::LineTo(p(0.0, 1.0))], paint: ShapePaint::Stroke(input.line_width) },
            ShapePath { segments: triangle, paint: ShapePaint::Stroke(w) },
            ShapePath { segments: dot, paint: ShapePaint::Solid },
        ],
        extent: Vec::new(),
    })
}

/// `post` and `mark`: the other borders, a rectangle and a point.
fn post(_cx: &mut Context, _input: &ShapeInput) -> Result<NodeShape, Error> {
    let p = Point::new;
    let square = vec![PathSegment::MoveTo(p(-0.25, -0.25)), PathSegment::LineTo(p(0.25, -0.25)), PathSegment::LineTo(p(0.25, 0.25)), PathSegment::Close];
    Ok(NodeShape {
        border: ShapeBorder::Rectangle(p(0.5, 0.25)),
        paths: vec![
            ShapePath { segments: square, paint: ShapePaint::Background },
            ShapePath { segments: vec![PathSegment::MoveTo(p(-0.5, -0.25)), PathSegment::LineTo(p(0.5, 0.25))], paint: ShapePaint::Stroke(0.05) },
        ],
        ..NodeShape::default()
    })
}

fn mark(_cx: &mut Context, input: &ShapeInput) -> Result<NodeShape, Error> {
    // one function may draw several shapes, told apart by name
    assert!(input.name == "mark" || input.name == "other mark", "{}", input.name);
    // nothing drawn, but room made around it
    Ok(NodeShape { extent: vec![Point::new(-1.0, -1.0), Point::new(1.0, 1.0)], ..NodeShape::default() })
}

/// `declare flags`: the shapes, declared from the library's definitions.
fn declare(cx: &mut Context, _value: Option<Tokens>) -> Result<(), Error> {
    cx.declare_shape("flag", flag);
    cx.declare_shape(" post ", post);
    cx.declare_shape("mark", mark);
    cx.declare_shape("other mark", mark);
    Ok(())
}

fn flag_definitions() -> String {
    r"\def\flagdot{0.05}\pgfkeys{/tikz/declare flags}\tikzset{flag/.style={shape=flag}}".into()
}

static FLAGS: Library = Library { name: "flags", definitions: flag_definitions, keys: &[("/tikz/declare flags", declare)] };

/// The numbers of the `d` of the paths of an SVG, one list per path.
fn path_numbers(svg: &str) -> Vec<Vec<f64>> {
    svg.split("<path d=\"").skip(1).map(|p| p[..p.find('"').unwrap()].split([' ', 'M', 'L', 'C', 'Z']).filter_map(|t| t.parse().ok()).collect()).collect()
}

#[test]
fn a_library_shape_draws_its_paths_as_it_paints_them() {
    let plain = convert(r"\node[flag] {};", &[&FLAGS]);
    assert!(plain.warnings.is_empty(), "{:?}", plain.warnings);
    // the pole, the outline twice as wide, the dot in the line color; no fill without `fill`
    assert!(plain.svg.contains("stroke-width=\"0.399\"") && plain.svg.contains("stroke-width=\"0.797\""), "{}", plain.svg);
    assert!(plain.svg.contains("fill=\"#000000\""), "{}", plain.svg);
    assert_eq!(plain.svg.matches("<path").count(), 3, "{}", plain.svg);
    let filled = convert(r"\node[flag, fill=red, draw=blue] {};", &[&FLAGS]);
    assert!(filled.svg.contains("fill=\"#ff0000\"") && filled.svg.contains("stroke=\"#0000ff\""), "{}", filled.svg);
    assert_eq!(filled.svg.matches("<path").count(), 4, "{}", filled.svg);
}

#[test]
fn a_library_shape_has_its_anchors_and_is_placed_by_them() {
    // the top of a flag placed by its foot at (1,1) is at (1,2): a line from there to (3,2) is flat
    let r = convert(r"\node[flag, anchor=foot] (F) at (1,1) {A}; \draw[red] (F.top) -- (3,2) (F.foot) -- (F.center) (F.30) -- (F.text);", &[&FLAGS]);
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let red = r.svg.split("<path d=\"").find(|p| p.contains("#ff0000")).unwrap();
    let n: Vec<f64> = red[..red.find('"').unwrap()].split([' ', 'M', 'L']).filter_map(|t| t.parse().ok()).collect();
    assert!((n[1] - n[3]).abs() < 1e-6 && (n[2] - n[0] - 56.693).abs() < 0.01, "{n:?}");
    // the text is drawn
    assert!(r.svg.contains("<g transform="), "{}", r.svg);
}

#[test]
fn a_library_shape_turns_with_its_node_and_stops_lines_at_its_border() {
    let straight = path_numbers(&convert(r"\node[flag] {};", &[&FLAGS]).svg);
    let turned = path_numbers(&convert(r"\node[flag, rotate=90] {};", &[&FLAGS]).svg);
    assert_ne!(straight, turned);
    // a line to the flag stops at its outline, one to a post at its rectangle, one to a mark at it
    for (shape, x) in [("flag", 3.0 - 0.1), ("post", 3.0 - 0.5), ("mark", 3.0), ("other mark", 3.0)] {
        let r = convert(&format!(r"\node[shape={shape}] (N) at (3,0.5) {{}}; \draw[red] (0,0.5) -- (N);"), &[&FLAGS]);
        assert!(r.warnings.is_empty(), "{shape}: {:?}", r.warnings);
        let red = r.svg.split("<path d=\"").find(|p| p.contains("#ff0000")).unwrap();
        let n: Vec<f64> = red[..red.find('"').unwrap()].split([' ', 'M', 'L']).filter_map(|t| t.parse().ok()).collect();
        assert!((n[2] - n[0] - x * 28.3465).abs() < 0.05, "{shape}: {n:?}");
    }
}

#[test]
fn a_library_shape_has_a_background_white_or_its_fill() {
    assert!(convert(r"\node[shape=post] {};", &[&FLAGS]).svg.contains("fill=\"#ffffff\""));
    let filled = convert(r"\node[shape=post, fill=red] {};", &[&FLAGS]).svg;
    assert!(filled.contains("fill=\"#ff0000\"") && !filled.contains("fill=\"#ffffff\""), "{filled}");
}

#[test]
fn a_library_shape_makes_room_for_its_extent() {
    let r = convert(r"\node[shape=mark] {};", &[&FLAGS]);
    // 2 cm square, plus 2 bp of margin on each side
    let viewbox: Vec<f64> = r.svg.split("viewBox=\"").nth(1).unwrap().split('"').next().unwrap().split(' ').map(|v| v.parse().unwrap()).collect();
    assert!((viewbox[2] - 2.0 * 28.3465 - 3.986).abs() < 0.01 && (viewbox[3] - 2.0 * 28.3465 - 3.986).abs() < 0.01, "{viewbox:?}");
}

#[test]
fn a_library_shape_is_known_only_with_its_library() {
    assert!(!convert(r"\node[shape=flag] {};", &[]).warnings.is_empty());
}
