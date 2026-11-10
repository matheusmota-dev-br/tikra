//! The animations library, still (a snapshot) and played, and the spy library: what the other
//! tests leave out. Snapshots between, before and after the times, the units of time, the
//! attributes played and the events they wait for, the room a moving node needs, the system
//! layer's timelines; spies without their parts, over clips, and with nodes neither drawn nor
//! filled.

mod common;

use common::*;
use tikra::canvas::{Animation, Canvas, Item, Moving};
use tikra::engine::{run_document_mapped, Run};

const ANIM: &str = r"\usetikzlibrary{animations}";

/// The fill color of the first filled path of a body.
fn fill(body: &str) -> String {
    let c = canvas(&format!("{ANIM}{body}"));
    paths(&c)
        .iter()
        .find_map(|d| match &d.fill.as_ref()?.paint {
            tikra::canvas::Paint::Solid(c) => Some(c.hex()),
            _ => None,
        })
        .expect("a filled path")
}

/// The canvas and warnings of a body, the animations played.
fn played(body: &str) -> (Canvas, Vec<String>) {
    let code = format!("{ANIM}\\begin{{tikzpicture}}{body}\\end{{tikzpicture}}");
    let (c, w, _) = run_document_mapped(&code, &[], Run { play: true, ..Run::default() }).unwrap_or_else(|e| panic!("{e}\n{body}"));
    (c, w)
}

/// The animations played in a body.
fn animations(body: &str) -> Vec<Animation> {
    let (c, w) = played(body);
    assert!(w.is_empty(), "{w:?}\n{body}");
    c.items
        .into_iter()
        .flat_map(|i| match i {
            Item::Animated { animations, .. } => animations,
            _ => Vec::new(),
        })
        .collect()
}

/// The first number of each value.
fn firsts(a: &Animation) -> Vec<f64> {
    a.values.iter().map(|v| v.split_whitespace().next().unwrap().parse().unwrap()).collect()
}

// ------------------------------------------------------------------ snapshots

#[test]
fn a_snapshot_takes_the_value_at_its_time() {
    let at = |t: &str, timeline: &str| fill(&format!(r"\tikzset{{make snapshot of={t}}} \fill[green] :fill = {{{timeline}}} (0,0) rectangle (1,1);"));
    let line = r#"0s="red", 1s="blue""#;
    // halfway in milliseconds: the colors mixed half and half
    assert_eq!(at("500ms", line), "#800080");
    // at the first time exactly, the first value
    assert_eq!(at("0s", line), "#ff0000");
    // after the last one (0.05 min is 3 s) the animation is over: the base color is back
    assert_eq!(at("0.05min", line), "#00ff00");
    // unless it freezes
    assert_eq!(at("0.05min", &format!("{line}, freeze")), "#0000ff");
    // before the first time, the first value holds
    assert_eq!(at("0.5s", r#"1s="red", 2s="blue""#), "#ff0000");
    // no times at all: nothing changes
    assert_eq!(at("1s", "begin=1s"), "#00ff00");
    // a value that is not braced is no timeline
    assert_eq!(fill(r"\tikzset{make snapshot of=1s} \fill[green] :fill = x (0,0) rectangle (1,1);"), "#00ff00");
}

#[test]
fn animate_gives_the_snapshot_values_to_the_options() {
    let snap = r"\tikzset{make snapshot of=1s}";
    // whom:attribute
    assert_eq!(fill(&format!(r#"{snap}\fill[green, animate={{myself:fill={{0s="red", 2s="red"}}}}] (0,0) rectangle (1,1);"#)), "#ff0000");
    // whom: = {{:attribute = …}}
    assert_eq!(fill(&format!(r#"{snap}\fill[green, animate={{myself: = {{:fill = {{0s="blue", 2s="blue"}}}}}}] (0,0) rectangle (1,1);"#)), "#0000ff");
    // a style of /tikz/animate, with and without a value
    let styles = r##"\tikzset{animate/redden/.style={myself:fill={0s="red", 2s="red"}}, animate/tint/.style={myself:fill={0s="#1", 2s="#1"}}}"##;
    assert_eq!(fill(&format!(r"{snap}{styles}\fill[green, animate={{redden}}] (0,0) rectangle (1,1);")), "#ff0000");
    assert_eq!(fill(&format!(r"{snap}{styles}\fill[green, animate={{tint=blue}}] (0,0) rectangle (1,1);")), "#0000ff");
}

/// In `whom: = {…}`, an attribute whose timeline is over at the snapshot keeps its base value,
/// while the others take theirs.
#[test]
fn animate_with_several_attributes_some_over() {
    let snap = r"\tikzset{make snapshot of=3s}";
    let c = canvas(&format!(
        r#"{ANIM}{snap}\draw[green, line width=1pt, animate={{myself: = {{:fill = {{0s="red", 1s="blue"}}, :line width = {{0s="2pt", 4s="2pt"}}}}}}] (0,0) rectangle (1,1);"#
    ));
    let p = &paths(&c)[0];
    assert!(p.fill.is_none(), "the fill is over: none, as without it");
    assert!(close(p.stroke.as_ref().unwrap().width, 2.0 * tikra::PT));
}

#[test]
fn an_animation_cut_short_by_the_end_of_the_document() {
    for code in [r"\begin{tikzpicture}\fill (0,0) circle (1) :fill", r"\begin{tikzpicture}\fill (0,0) circle (1) :fill ="] {
        let (c, w) = tikra::engine::run_document(&format!("{ANIM}{code}")).unwrap_or_else(|e| panic!("{e}\n{code}"));
        assert!(w.iter().any(|w| w.contains("path without ';' at the end")), "{code}: {w:?}");
        assert_eq!(paths(&c).len(), 1, "{code}: the circle is drawn");
    }
}

// ------------------------------------------------------------------ played

#[test]
fn an_animation_may_wait_for_an_event_and_a_delay() {
    let a = animations(r#"\fill :fill = {0s="red", 1s="blue", begin on={mouse over, delay=2s, of next=x}} (0,0) circle (1);"#);
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].on.as_deref(), Some("mouseover"));
    assert_eq!(a[0].begin, 2.0);
    assert_eq!(a[0].values, ["#ff0000", "#0000ff"]);
}

#[test]
fn a_played_animation_needs_times_and_values_it_understands() {
    assert!(animations(r"\fill :fill = {begin=1s} (0,0) circle (1);").is_empty());
    let (_, w) = played(r#"\fill :fill = {0s="nocolor", 1s="red"} (0,0) circle (1);"#);
    assert!(w.iter().any(|w| w.contains("animation of fill: a value is not understood; not played")), "{w:?}");
    // an attribute that is not played, even spelled with a macro
    for attribute in ["visible", r"\vis"] {
        let (_, w) = played(&format!(r#"\fill :{attribute} = {{0s="false", 1s="true"}} (0,0) circle (1);"#));
        assert!(w.iter().any(|w| w.contains(&format!("animation of {attribute}: not played"))), "{w:?}");
    }
}

#[test]
fn played_shifts_colors_and_scalings() {
    // shift: in bp, y down
    let a = animations(r#"\fill :shift = {0s="(0:0)", 1s="(90:2)"} (0,0) circle (1);"#);
    assert!(matches!(a[0].moving, Moving::Shift));
    let ends: Vec<Vec<f64>> = a[0].values.iter().map(|v| v.split_whitespace().map(|x| x.parse().unwrap()).collect()).collect();
    assert!(ends[1][0].abs() < 1e-6 && close(ends[1][1], -144.0 / 2.54), "{ends:?}");
    // color: the fill and the stroke
    let a = animations(r#"\filldraw :color = {0s="red", 1s="blue"} (0,0) circle (1);"#);
    let names: Vec<_> = a
        .iter()
        .map(|a| match a.moving {
            Moving::Attribute(n, _) => n,
            _ => "",
        })
        .collect();
    assert_eq!(names, ["fill", "stroke"]);
    // scalings: both ways, or one
    for (attribute, values) in [("scale", ["1", "2"]), ("xscale", ["1 1", "2 1"]), ("yscale", ["1 1", "1 2"])] {
        let a = animations(&format!(r#"\fill :{attribute} = {{0s="1", 1s="2"}} (0,0) circle (1);"#));
        assert!(matches!(a[0].moving, Moving::Scaling), "{attribute}");
        assert_eq!(a[0].values, values, "{attribute}");
    }
    assert_eq!(firsts(&animations(r#"\fill :rotate = {0s="0", 1s="90"} (0,0) circle (1);"#)[0]), [0.0, -90.0]);
}

// ------------------------------------------------------------------ room

#[test]
fn a_moving_node_gets_room_at_each_place() {
    let plain = size(r"\node {x};");
    // up 2 cm, right 3 cm: the picture reaches there
    let up = size(&format!(r#"{ANIM}\node[animate={{myself:yshift={{0s="0cm", 1s="2cm"}}}}] {{x}};"#));
    // (the node reaches by its anchors, with their outer sep)
    assert!((up.1 - plain.1 - 2.0).abs() < 0.02, "{up:?} {plain:?}");
    let right = size(&format!(r#"{ANIM}\node[animate={{myself:shift={{0s="(0:0)", 1s="(0:3)"}}}}] {{x}};"#));
    assert!((right.0 - plain.0 - 3.0).abs() < 0.02, "{right:?} {plain:?}");
    // a value that is no length gives no room
    let none = size(&format!(r#"{ANIM}\node[animate={{myself:xshift={{0s="far", 1s="away"}}}}] {{x}};"#));
    assert!(close(none.0, plain.0) && close(none.1, plain.1), "{none:?} {plain:?}");
}

// ------------------------------------------------------------------ the system layer

#[test]
fn system_layer_timelines_of_several_attributes() {
    // the opacity and the stroke opacity of one node, both at the snapshot (half way: 0.5); a fill timeline keeps the base
    let body = r"\pgfsysanimsnapshot{1}\tikz {
        \pgfidrefnextuse{\objid}{n}\pgfsysanimkeywhom{\objid}{}
        \pgfsysanimkeytime{0}{1}{1}{0}{0}\pgfsysanimvalscalar{1}
        \pgfsysanimkeytime{2}{1}{1}{0}{0}\pgfsysanimvalscalar{0}
        \pgfsysanimate{opacity}
        \pgfsysanimate{strokeopacity}
        \pgfsysanimate{fill}
        \node (n) [fill=blue, draw] {N};
      }";
    let (c, w) = tikra::engine::run_document(body).unwrap();
    assert!(w.is_empty(), "{w:?}");
    let f = c.items.iter().find_map(|i| if let Item::Path { fill: Some(f), .. } = i { Some(f.clone()) } else { None }).unwrap();
    assert!(close(f.opacity, 0.5), "{}", f.opacity);
    let s = c.items.iter().find_map(|i| if let Item::Path { stroke: Some(s), .. } = i { Some(s.opacity) } else { None }).unwrap();
    assert!(close(s, 0.5), "{s}");
    assert!(matches!(f.paint, tikra::canvas::Paint::Solid(c) if c.hex() == "#0000ff"));
}

/// Two attributes of one object, each by its own timeline: both are at their snapshot values.
#[test]
fn system_layer_two_timelines_for_one_object() {
    let body = r"\pgfsysanimsnapshot{1}\tikz {
        \pgfidrefnextuse{\objid}{n}\pgfsysanimkeywhom{\objid}{}
        \pgfsysanimkeytime{0}{1}{1}{0}{0}\pgfsysanimvalscalar{1}
        \pgfsysanimkeytime{2}{1}{1}{0}{0}\pgfsysanimvalscalar{0}
        \pgfsysanimate{fillopacity}
        \pgfsysanimkeytime{0}{1}{1}{0}{0}\pgfsysanimvaldimension{0pt}
        \pgfsysanimkeytime{2}{1}{1}{0}{0}\pgfsysanimvaldimension{4pt}
        \pgfsysanimate{linewidth}
        \node (n) [fill=blue, draw] {N};
      }";
    let (c, w) = tikra::engine::run_document(body).unwrap();
    assert!(w.is_empty(), "{w:?}");
    let f = c.items.iter().find_map(|i| if let Item::Path { fill: Some(f), .. } = i { Some(f.opacity) } else { None }).unwrap();
    assert!(close(f, 0.5), "{f}");
    let s = c.items.iter().find_map(|i| if let Item::Path { stroke: Some(s), .. } = i { Some(s.width) } else { None }).unwrap();
    assert!(close(s, 2.0 * tikra::PT), "{s}");
}

// ------------------------------------------------------------------ spy

const SPY: &str = r"\usetikzlibrary{spy}";

/// The clips of a canvas.
fn clips(c: &Canvas) -> usize {
    c.items.iter().filter(|i| matches!(i, Item::Clip { .. })).count()
}

/// Whether every group of a canvas is closed.
fn balanced(c: &Canvas) -> bool {
    let mut depth = 0i64;
    for i in &c.items {
        match i {
            Item::Clip { .. }
            | Item::Group { .. }
            | Item::Fade { .. }
            | Item::Blend { .. }
            | Item::Crisp
            | Item::Transform { .. }
            | Item::FadePicture { .. }
            | Item::Animated { .. }
            | Item::Point(_) => depth += 1,
            Item::End => depth -= 1,
            _ => {}
        }
        if depth < 0 {
            return false;
        }
    }
    depth == 0
}

#[test]
fn a_spy_needs_a_point_and_a_node() {
    let (c, w) = canvas_and_warnings(&format!(r"{SPY}\begin{{scope}}[spy using outlines] \fill (0,0) circle (1); \spy on (0,0); \end{{scope}}"));
    assert!(w.iter().any(|w| w.contains("\\spy without 'on (point) in node': 'on (0,0)' ignored")), "{w:?}");
    assert_eq!(clips(&c), 0);
}

#[test]
fn the_magnified_copy_closes_what_the_scope_left_open() {
    // a scope with its own clip, and a clip left open in the spy scope: the copy is balanced
    let body = format!(
        r"{SPY}\begin{{scope}}[spy using outlines={{circle, size=1cm}}]
          \begin{{scope}} \clip (0,0) rectangle (1,1); \fill (0,0) circle (1); \end{{scope}}
          \clip (-2,-2) rectangle (2,2); \draw (0,0) -- (1,1);
          \spy on (0.5,0.5) in node at (4,0);
        \end{{scope}}"
    );
    let (c, w) = canvas_and_warnings(&body);
    assert!(w.is_empty(), "{w:?}");
    assert!(balanced(&c));
    // the scope's two clips, the copies of the two, and the in node's own
    assert_eq!(clips(&c), 5);
}

#[test]
fn a_spy_node_out_of_reach_shows_nothing() {
    let (c, w) = canvas_and_warnings(&format!(r"{SPY}\begin{{scope}}[spy scope] \fill (0,0) circle (1); \spy on (0,0) in node at (200000,0); \end{{scope}}"));
    assert!(w.iter().any(|w| w.contains("not finite or beyond")), "{w:?}");
    assert_eq!(clips(&c), 0);
}

#[test]
fn a_spy_node_neither_drawn_nor_filled_still_shows_the_copy() {
    // spy scope: no outline, no fill; the copy goes after the node
    let (c, w) = canvas_and_warnings(&format!(r"{SPY}\begin{{scope}}[spy scope={{size=1cm}}] \fill (0,0) circle (1); \spy on (0,0) in node at (3,0); \end{{scope}}"));
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(clips(&c), 1);
    assert!(matches!(c.items.last(), Some(Item::End)));
    assert!(balanced(&c));
}

#[test]
fn an_overlay_spy_is_filled_under_its_copy() {
    let (c, w) = canvas_and_warnings(&format!(r"{SPY}\begin{{scope}}[spy using overlays={{size=1cm}}] \fill (0,0) circle (1); \spy on (0,0) in node at (3,0); \end{{scope}}"));
    assert!(w.is_empty(), "{w:?}");
    // the in node's fill, then its clip with the copy
    let k = c.items.iter().position(|i| matches!(i, Item::Clip { .. })).expect("the copy");
    assert!(matches!(&c.items[k - 1], Item::Path { fill: Some(_), stroke: None, .. }), "{:?}", c.items[k - 1]);
}

#[test]
fn a_connection_path_that_is_no_macro_draws_nothing() {
    let body = |pre: &str| format!(r"{SPY}{pre}\begin{{scope}}[spy using outlines={{size=1cm}}] \fill (0,0) circle (1); \spy on (0,0) in node at (3,0); \end{{scope}}");
    let with = canvas(&body(""));
    let without = canvas(&body(r"\makeatletter\let\tikra@spy@path\relax\makeatother"));
    assert_eq!(paths(&with).len(), paths(&without).len());
    assert_eq!(clips(&without), 1);
}

/// Played, an image has no attribute of its own to animate: it moves with its node, as it is.
#[test]
fn a_played_image_is_left_as_it_is() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="4pt" height="3pt" viewBox="0 0 4 3"><rect width="4" height="3" fill="blue"/></svg>"#;
    let dir = std::env::temp_dir().join(format!("tikra-played-images-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("dot.png"), tikra::png(svg, 72.0, false).unwrap()).unwrap();
    std::env::set_var("TIKRA_INPUT_DIR", dir.display().to_string());
    for body in [
        r#"\node[:fill = {0s="red", 2s="blue"}] {\includegraphics{dot.png}};"#,
        r#"\begin{scope}[:fill = {0s="red", 2s="blue"}] \clip (0,0) rectangle (1,1); \node {\includegraphics{dot.png}}; \end{scope}"#,
        r#"\begin{scope}[:fill = {0s="red", 2s="blue"}] \begin{axis}[hide axis] \addplot graphics[xmin=0,xmax=1,ymin=0,ymax=1] {dot.png}; \end{axis} \end{scope}"#,
    ] {
        let r = tikra::convert_with(&format!(r"\begin{{tikzpicture}}{body}\end{{tikzpicture}}"), &tikra::Options::default().play()).unwrap();
        assert!(r.svg.contains(r#"<g id="trAnim1">"#) && r.svg.contains("<image "), "{body}: {}", r.svg);
        assert!(!r.svg.contains("<animate"), "{body}: {}", r.svg);
    }
}
