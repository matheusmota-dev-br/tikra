//! Nodes, the parts the first node tests leave out: node parts with options, labels at every
//! direction and `absolute`, `auto` on the right of each direction, late options, nodes placed
//! later, texts that are shadings, images or colored, the nodes of nonlinear transformations, and
//! the circuit symbols at the ends of their line.

mod common;

use common::*;
use tikra::canvas::Paint;
use tikra::scene::{Label, Point, Transform};

/// The center of a text, in the picture (cm).
fn center(l: &Label, place: &Transform) -> Point {
    place.apply(Point::new(l.width() / 2.0, (l.height() - l.depth()) / 2.0))
}

/// The centers of the texts of a body.
fn centers(body: &str) -> Vec<Point> {
    texts(&canvas(body)).iter().map(|(l, p)| center(l, p)).collect()
}

/// The texts of a body with their colors, which must draw without warnings.
fn colors(body: &str) -> Vec<String> {
    let (c, w) = canvas_and_warnings(body);
    assert!(w.is_empty(), "{w:?}\n{body}");
    texts(&c).iter().map(|(l, _)| l.color.hex()).collect()
}

// ------------------------------------------------------------------ node parts

#[test]
fn a_node_part_may_have_options_and_an_unbraced_name() {
    // the options of \nodepart are read and set aside: two texts, one in each box
    let c = centers(r"\node[rectangle split, rectangle split parts=2] {a \nodepart[red] {two} b};");
    assert_eq!(c.len(), 2);
    assert!(c[0].y > c[1].y, "{c:?}");
    // \nodepart l is \nodepart{l}: a part the shape does not have, whose text is not shown
    for body in [r"\node[circle split] {a \nodepart l b};", r"\node[circle split] {a \nodepart{l} b};"] {
        assert_eq!(centers(body).len(), 1, "{body}");
    }
}

// ------------------------------------------------------------------ placement

#[test]
fn a_node_past_the_corner_of_a_vertical_horizontal_path_is_on_its_second_leg() {
    // -| goes right, then up: pos .75 is halfway up the second leg
    let c = centers(r"\draw (0,0) -| (2,2) node[pos=.75] {x};");
    assert!(close_to((c[0].x, c[0].y), (2.0, 1.0)), "{c:?}");
}

/// Options between a corner path (`|-`) or an arc and a node move the node as they move points:
/// the node is where `pos` puts it on that part of the path, then shifted.
#[test]
fn options_before_a_node_after_a_corner_or_an_arc_move_it() {
    // `|-` up, then right: pos .25 is halfway up the first leg, at (0, .5)
    let q = point(r"\draw (0,0) |- (2,1) [xshift=1cm] node[pos=.25] (n) {};", "n.center");
    assert!(close_to((q.x, q.y), (1.0, 0.5)), "{q:?}");
    // a quarter circle around (-1, 0): pos .5 is at 45 degrees
    let q = point(r"\draw (0,0) arc (0:90:1) [xshift=1cm] node[pos=.5] (n) {};", "n.center");
    let h = std::f64::consts::FRAC_1_SQRT_2;
    assert!(close_to((q.x, q.y), (h, h)), "{q:?}");
}

#[test]
fn an_unknown_anchor_is_the_center() {
    let (c, w) = canvas_and_warnings(r"\node[anchor=nowhere] at (1,1) {x};");
    assert!(w.iter().any(|w| w.contains("unknown anchor 'nowhere': center used")), "{w:?}");
    let t = texts(&c);
    let m = center(&t[0].0, &t[0].1);
    assert!(close_to((m.x, m.y), (1.0, 1.0)), "{m:?}");
}

#[test]
fn a_node_on_a_nonlinear_transformation_goes_where_its_point_goes() {
    // the transformation doubles x: the point (1,0) goes to (2,0)
    let c = centers(r"\pgftransformnonlinear{\csname pgf@x\endcsname=2\csname pgf@x\endcsname} \node at (1,0) {x};");
    assert!(close_to((c[0].x, c[0].y), (2.0, 0.0)), "{c:?}");
    // with transform shape nonlinear, a node is also stretched as the transformation stretches the
    // picture: twice as wide where x doubles
    let wide = size(r"\pgftransformnonlinear{\csname pgf@x\endcsname=2\csname pgf@x\endcsname} \node[draw, transform shape nonlinear, minimum size=1cm] at (1,0) {};");
    let plain = size(r"\pgftransformnonlinear{\csname pgf@x\endcsname=2\csname pgf@x\endcsname} \node[draw, minimum size=1cm] at (1,0) {};");
    assert!((wide.0 - 2.0 - (plain.0 - 1.0)).abs() < 0.01, "the line width is not doubled: {wide:?} {plain:?}");
    assert!((wide.1 - plain.1).abs() < 0.001, "{wide:?} {plain:?}");
}

#[test]
fn behind_path_draws_the_node_before_its_path() {
    let c = canvas(r"\draw[fill=red] (0,0) rectangle (2,2) node[behind path, fill=blue] at (1,1) {x};");
    let fills: Vec<String> = paths(&c)
        .iter()
        .filter_map(|d| match d.fill.as_ref().map(|f| &f.paint) {
            Some(Paint::Solid(color)) => Some(color.hex()),
            _ => None,
        })
        .collect();
    assert_eq!(fills, ["#0000ff", "#ff0000"], "the node first");
}

#[test]
fn a_shifting_animation_makes_room_for_the_node_where_it_goes() {
    let still = size(r"\node[draw] {x};");
    let moved = size(r#"\node[draw, animate={myself:xshift={0s="0cm", 2s="3cm"}}] {x};"#);
    assert!(close(moved.0 - still.0, 3.0), "{still:?} {moved:?}");
}

#[test]
fn a_node_named_by_name_path_has_its_border_as_that_path() {
    // the line crosses the border at x = ±1: a dot of 1pt at each crossing
    let body = r"\node[draw, minimum size=2cm, name path=box] at (0,0) {}; \path[name path=line] (-3,0.5) -- (3,0.5);
        \fill[name intersections={of=box and line}] (intersection-1) circle (1pt) (intersection-2) circle (1pt);";
    let p = paths(&canvas(body));
    let (lo, hi) = bounds(&p[1].segments);
    let r = 2.54 / 72.27;
    assert!(close_to((lo.x, lo.y), (-1.0 - r, 0.5 - r)) && close_to((hi.x, hi.y), (1.0 + r, 0.5 + r)), "{lo:?} {hi:?}");
}

// ------------------------------------------------------------------ late options, later nodes

#[test]
fn late_options_add_labels_to_a_node_already_there() {
    let c = centers(r"\node (a) at (1,1) {a}; \path node also [label=above:L] (a);");
    assert_eq!(c.len(), 2);
    assert!(close(c[1].x, c[0].x) && c[1].y > c[0].y, "{c:?}");
    let (c, w) = canvas_and_warnings(r"\path node also [label=above:L] (nobody);");
    assert!(w.iter().any(|w| w.contains("late options: no node named 'nobody'")), "{w:?}");
    assert!(texts(&c).is_empty());
}

#[test]
fn a_node_positioned_later_goes_where_it_is_positioned() {
    let body = r"{\pgfpositionnodelater{\relax}\node[draw] (k) {k};}\pgfpositionnodenow{\pgfpoint{2cm}{1cm}}";
    let c = centers(body);
    assert!(close_to((c[0].x, c[0].y), (2.0, 1.0)), "{c:?}");
    // it is known by its name there
    let k = point(body, "k");
    assert!(close_to((k.x, k.y), (2.0, 1.0)), "{k:?}");
    // and counts for the picture's size there, as the node would
    let (kept, plain) = (size(body), size(r"\node[draw] {k};"));
    assert!((kept.0 - plain.0).abs() < 0.02 && (kept.1 - plain.1).abs() < 0.02, "{kept:?} {plain:?}");
    // with no node kept there is nothing to place
    let (_, w) = canvas_and_warnings(r"\pgfpositionnodenow{\pgfpoint{1cm}{0cm}}");
    assert!(w.iter().any(|w| w.contains("no node kept by \\pgfpositionnodelater")), "{w:?}");
}

// ------------------------------------------------------------------ texts

#[test]
fn a_text_that_is_a_shading_is_drawn_as_the_shading() {
    let body = r"\pgfdeclarehorizontalshading{fade}{1cm}{color(0cm)=(red); color(2cm)=(blue)} \node[draw] {\pgfuseshading{fade}};";
    let (c, w) = canvas_and_warnings(body);
    assert!(w.is_empty(), "{w:?}");
    assert!(texts(&c).is_empty());
    // the frame is around the shading's box, 2 cm by 1 cm, plus the inner sep
    let (w, h) = size(body);
    let pt = 2.54 / 72.27;
    let around = 2.0 * 3.333 * pt + 0.4 * pt;
    assert!((w - 2.0 - around).abs() < 0.01, "{w}");
    assert!((h - 1.0 - around).abs() < 0.01, "{h}");
}

#[test]
fn colors_around_the_whole_text() {
    const RED: &str = "#ff0000";
    assert_eq!(colors(r"\node {\textcolor{red}{x}};"), [RED]);
    assert_eq!(colors(r"\node {\textcolor{red}{a {b} c}};"), [RED]);
    // a \color switch first colors the rest, groups and all
    assert_eq!(colors(r"\node {\color{red} a {b} c};"), [RED]);
    // a color no one defined leaves the text as it is
    assert_eq!(colors(r"\node {a \textcolor{nocolor}{x} b};"), ["#000000"]);
}

#[test]
fn colors_inside_a_text_keep_words_and_math_apart() {
    // the colored part has words and math: still one text, of the node's color around it
    let (c, w) = canvas_and_warnings(r"\node {see \textcolor{blue}{the angle $\alpha$ here} now};");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(texts(&c).len(), 1);
    let plain = size(r"\node {see the angle $\alpha$ here now};");
    let colored = size(r"\node {see \textcolor{blue}{the angle $\alpha$ here} now};");
    assert!((plain.0 - colored.0).abs() < 0.05, "{plain:?} {colored:?}");
    // a \color switch inside a group stops at its end, braces inside included
    let a = size(r"\node {x {\color{red} y {z}} w};");
    let b = size(r"\node {x y z w};");
    assert!((a.0 - b.0).abs() < 0.05, "{a:?} {b:?}");
}

#[test]
fn a_command_that_fails_in_a_text_drops_the_node() {
    let (c, w) = canvas_and_warnings(r"\node {\advance x};");
    assert_eq!(w, ["\\node: figure: you can't use x after \\advance; statement ignored"]);
    assert!(texts(&c).is_empty());
}

#[test]
fn a_begin_node_hook_set_to_relax_adds_nothing() {
    let c = centers(r"\expandafter\let\csname tikz@atbegin@node\endcsname\relax \node {x};");
    assert_eq!(c.len(), 1);
}

// ------------------------------------------------------------------ images

#[test]
fn the_example_image_is_drawn_at_the_size_asked() {
    let bp = 2.54 / 72.0;
    for (options, w, h) in [
        ("", 320.0 * bp, 240.0 * bp),
        ("[scale=0.5]", 160.0 * bp, 120.0 * bp),
        ("[width=4cm]", 4.0, 3.0),
        ("[height=3cm]", 4.0, 3.0),
        ("[width=2cm, height=5cm, keepaspectratio]", 2.0, 5.0),
    ] {
        let body = format!(r"\node[inner sep=0pt, outer sep=0pt] {{\includegraphics{options}{{example-image-a}}}};");
        let (c, warnings) = drawn_and_warnings(&body);
        assert!(warnings.is_empty(), "{warnings:?}");
        let (lo, hi) = (c.min.unwrap(), c.max.unwrap());
        assert!((hi.x - lo.x - w).abs() < 0.02 && (hi.y - lo.y - h).abs() < 0.02, "{options}: {lo:?} {hi:?}");
        // the gray box, its lines, its frame and its letter A
        assert_eq!(paths(&c).len(), 3, "{options}");
        assert_eq!(texts(&c).len(), 1, "{options}");
    }
}

#[test]
fn image_options_tikra_does_not_know_are_reported() {
    let (_, w) = canvas_and_warnings(r"\node {\includegraphics[angle=90, interpolate]{example-image-a}};");
    assert_eq!(w, ["image option 'angle' ignored"]);
}

#[test]
fn an_image_not_found_keeps_the_size_asked() {
    let body = r"\node[inner sep=0pt, outer sep=0pt, draw] {\includegraphics[width=2cm, height=1cm]{no-such-file.png}};";
    let (_, w) = canvas_and_warnings(body);
    assert!(w.iter().any(|w| w.contains("image not drawn")), "{w:?}");
    let (sw, sh) = size(body);
    assert!(sw > 2.0 && sw < 2.05 && sh > 1.0 && sh < 1.05, "{sw} {sh}");
}

#[test]
fn an_image_used_before_it_is_declared_is_reported() {
    let (_, w) = canvas_and_warnings(r"\node {\pgfuseimage{nowhere}};");
    assert!(w.iter().any(|w| w.contains("\\pgfuseimage: no image 'nowhere' declared")), "{w:?}");
}

// ------------------------------------------------------------------ labels

#[test]
fn labels_of_a_coordinate_go_straight_out_with_the_anchor_away() {
    // the label's center is off the coordinate in the direction of the angle
    for angle in [0, 2, 30, 90, 120, 180, 200, 270, 300, 358] {
        let c = centers(&format!(r"\coordinate[label={angle}:x] (c) at (0,0);"));
        let (s, co) = (f64::from(angle).to_radians().sin(), f64::from(angle).to_radians().cos());
        let (dx, dy) = (c[0].x, c[0].y);
        assert!(dx * co + dy * s > 0.05, "{angle}: {c:?}");
        // sideways, only as much as the text's own half size allows
        assert!((dx * s - dy * co).abs() < 0.25, "{angle}: {c:?}");
    }
}

#[test]
fn an_absolute_label_ignores_the_node_turn() {
    // the node is turned a quarter: its own 0° points up, the page's to the right
    let turned = centers(r"\node[draw, rotate=90, label=0:x] at (0,0) {a};");
    let absolute = centers(r"\begin{scope}[absolute]\node[draw, rotate=90, label=0:x] at (0,0) {a};\end{scope}");
    assert!(turned[1].y > 0.2 && turned[1].x.abs() < 0.05, "{turned:?}");
    assert!(absolute[1].x > 0.2 && absolute[1].y.abs() < 0.05, "{absolute:?}");
}

#[test]
fn auto_swap_puts_the_text_on_the_right_of_the_line() {
    // eight directions: the text's center is to the right of the direction of the line
    for (dx, dy) in [(1, 1), (1, -1), (1, 0), (-1, 1), (-1, -1), (-1, 0), (0, 1), (0, -1)] {
        let c = centers(&format!(r"\draw (0,0) -- ({dx},{dy}) node[midway, auto, swap] {{x}};"));
        let m = (f64::from(dx) / 2.0, f64::from(dy) / 2.0);
        let (ox, oy) = (c[0].x - m.0, c[0].y - m.1);
        // right of (dx, dy) is (dy, -dx)
        assert!(ox * f64::from(dy) - oy * f64::from(dx) > 0.05, "({dx},{dy}): {c:?}");
    }
}

// ------------------------------------------------------------------ shapes' own keys and texts

#[test]
fn a_callout_pointer_may_be_a_point_plus_a_shift() {
    // (a)+(0,1) is the point (2,1), as if it were written so
    let sum = point(r"\node (a) at (2,0) {}; \node[rectangle callout, callout absolute pointer={(a)+(0,1)}] (c) {x};", "c.pointer");
    let plain = point(r"\node[rectangle callout, callout absolute pointer={(2,1)}] (c) {x};", "c.pointer");
    assert!(close_to((sum.x, sum.y), (plain.x, plain.y)), "{sum:?} {plain:?}");
    assert!((sum.x - 2.0).abs() < 0.2 && (sum.y - 1.0).abs() < 0.2, "{sum:?}");
}

#[test]
fn an_iec_gate_shows_its_symbol_with_its_color() {
    // no symbol: no text
    assert_eq!(texts(&canvas(r"\node[and gate IEC, and gate IEC symbol={}] {};")).len(), 0);
    // a \color at its start is the symbol's color
    assert_eq!(colors(r"\node[and gate IEC, and gate IEC symbol={\color{red}x}] {};"), ["#ff0000"]);
    // what the typesetter cannot read is reported
    let (_, w) = canvas_and_warnings(r"\node[and gate IEC, and gate IEC symbol={$\nosuchcommand$}] {};");
    assert!(!w.is_empty());
}

#[test]
fn a_circle_iec_runs_its_before_background_code() {
    let body = r"\node[generic circle IEC, draw, minimum size=1cm, /pgf/generic circle IEC/before background={\draw (-2pt,0) -- (2pt,0);}] {};";
    let plain = r"\node[generic circle IEC, draw, minimum size=1cm] {};";
    assert_eq!(paths(&canvas(body)).len(), paths(&canvas(plain)).len() + 1);
}

#[test]
fn patterns_fill_a_node() {
    let c = canvas(r"\node[pattern=north east lines, minimum size=1cm] {};");
    let p = paths(&c);
    assert_eq!(p.len(), 1);
    assert!(p[0].fill.as_ref().is_some_and(|f| matches!(f.paint, Paint::Pattern { .. } | Paint::Tile { .. })), "{:?}", p[0].fill);
}

// ------------------------------------------------------------------ circuits

/// A body in a picture with the circuits.ee.IEC options.
fn circuit(body: &str) -> String {
    format!(r"\begin{{scope}}[circuit ee IEC]{body}\end{{scope}}")
}

#[test]
fn a_symbol_at_either_end_of_its_line_is_centered_there() {
    for (key, x) in [("at start", 0.0), ("pos=0", 0.0), ("at end", 3.0), ("pos=1", 3.0), ("near end", 2.25), ("pos=", 1.5)] {
        let body = circuit(&format!(r"\draw (0,0) to [resistor={{{key}, name=r}}] (3,0);"));
        let r = point(&body, "r.center");
        assert!(close_to((r.x, r.y), (x, 0.0)), "{key}: {r:?}");
    }
    // on a vertical line, the symbol turns with it
    let body = circuit(r"\draw (0,0) to [resistor={at end, name=r}] (0,3);");
    let (n, e) = (point(&body, "r.north"), point(&body, "r.east"));
    assert!(n.x < -0.05 && close(e.y, 3.0 + (e.y - 3.0)) && e.y > 3.0, "{n:?} {e:?}");
}

#[test]
fn a_sloped_info_turns_with_the_line() {
    let c = canvas(&circuit(r"\draw (0,0) to [resistor={info sloped=R}] (2,2);"));
    let t = texts(&c);
    assert_eq!(t.len(), 1);
    let turn = t[0].1.b.atan2(t[0].1.a).to_degrees();
    assert!((turn - 45.0).abs() < 1e-6, "{turn}");
}

#[test]
fn the_symbol_unit_scales_the_symbols() {
    let small = size(&circuit(r"\node[resistor] {};"));
    let large = size(r"\begin{scope}[circuit ee IEC, circuit symbol unit=14pt]\node[resistor] {};\end{scope}");
    assert!(close(large.0 / small.0, 2.0) || (large.0 - 2.0 * small.0).abs() < 0.03, "{small:?} {large:?}");
}

#[test]
fn current_direction_draws_its_tip_on_the_line() {
    let with = paths(&canvas(&circuit(r"\draw (0,0) to [current direction] (2,0);")));
    let without = paths(&canvas(&circuit(r"\draw (0,0) to (2,0);")));
    assert!(with.len() > without.len(), "{} {}", with.len(), without.len());
    assert!(with.iter().any(|d| d.fill.is_some()), "the filled tip");
}

#[test]
fn arrow_options_set_the_length_of_direction_ee() {
    let tip = |len: &str| {
        let c = canvas(&circuit(&format!(r"\pgfsetarrowoptions{{direction ee}}{{{len}}} \draw[-direction ee] (0,0) -- (2,0);")));
        let p = paths(&c);
        let (lo, hi) = bounds(&p.last().unwrap().segments);
        hi.x - lo.x
    };
    let (short, long) = (tip("2pt"), tip("6pt"));
    assert!(long > 2.0 * short, "{short} {long}");
    // another arrow's options are not ours
    let c = canvas(&circuit(r"\pgfsetarrowoptions{latex}{6pt} \draw (0,0) -- (2,0);"));
    assert_eq!(paths(&c).len(), 1);
}

#[test]
fn an_annotation_takes_the_labels_among_its_options() {
    let c = canvas(&circuit(r"\draw (0,0) to [diode={light emitting={info=L}}] (3,0);"));
    assert_eq!(texts(&c).len(), 1);
}

// ------------------------------------------------------------------ more of the shapes' keys

#[test]
fn an_empty_shape_key_keeps_its_default() {
    // star points and star point height left empty are 5 and .5cm
    let empty = size(r"\node[draw, star, star points={}, star point height={}] {};");
    let given = size(r"\node[draw, star, star points=5, star point height=.5cm] {};");
    assert!(close_to(empty, given), "{empty:?} {given:?}");
    // a key with no value at all reads as empty: an arrow box with no arrows
    let unset = paths(&canvas(r"\expandafter\let\csname pgfk@/pgf/arrow box arrow list\endcsname\relax \node[draw, arrow box] {};"));
    let plain = paths(&canvas(r"\node[draw, arrow box, arrow box arrows={}] {};"));
    assert_eq!(unset.len(), 1);
    assert_eq!(unset[0].segments, plain[0].segments);
}

#[test]
fn a_failing_before_background_code_is_reported() {
    let (c, w) = canvas_and_warnings(r"\node[generic circle IEC, draw, /pgf/generic circle IEC/before background={\advance x}] {};");
    assert!(w.iter().any(|w| w.starts_with("/pgf/generic circle IEC/before background:")), "{w:?}");
    // the circle is drawn all the same
    assert_eq!(paths(&c).len(), 1);
}

#[test]
fn another_tip_for_direction_ee_is_filled_and_stroked_as_tips_are() {
    let p = paths(&canvas(r"\node[shape=direction ee, /pgf/direction ee arrow=Stealth, draw] {};"));
    // the line, the tip's fill (in the line's color) and its outline
    assert_eq!(p.len(), 3);
    assert!(p[1].fill.is_some() && p[1].stroke.is_none(), "{:?}", p[1]);
    assert!(p[2].fill.is_none() && p[2].stroke.is_some(), "{:?}", p[2]);
}

#[test]
fn a_shading_text_needs_a_text_part_to_go_in() {
    // the tip of direction ee has no text part: the shading is not drawn
    let body = r"\pgfdeclarehorizontalshading{fade}{1cm}{color(0cm)=(red); color(2cm)=(blue)} \node[shape=direction ee, draw] {\pgfuseshading{fade}};";
    let shaded = |body: &str| paths(&canvas(body)).iter().any(|d| d.fill.as_ref().is_some_and(|f| matches!(f.paint, Paint::Shading(_))));
    assert!(!shaded(body));
    // a rectangle has one
    assert!(shaded(&body.replace("shape=direction ee, ", "")));
}

#[test]
fn a_shape_text_that_is_not_a_color_switch_keeps_the_node_color() {
    // \colorbox colors a box behind the text, not the text
    let (c, _) = canvas_and_warnings(r"\node[and gate IEC, and gate IEC symbol={\colorbox{red}{x}}] {};");
    assert_eq!(texts(&c).iter().map(|(l, _)| l.color.hex()).collect::<Vec<_>>(), ["#000000"]);
}

#[test]
fn an_animated_node_after_a_clip_makes_no_room() {
    // after a clip nothing counts for the picture's size, the node's shifts neither
    let base = r"\draw (0,0) -- (1,0);";
    let moved = size(&format!(r#"{base} \begin{{scope}}\clip (0,0) rectangle (1,1); \node[animate={{myself:xshift={{0s="0cm", 2s="3cm"}}}}] {{x}};\end{{scope}}"#));
    let still = size(&format!(r"{base} \begin{{scope}}\clip (0,0) rectangle (1,1); \node {{x}};\end{{scope}}"));
    assert!(close_to(moved, still), "{moved:?} {still:?}");
    // a node given overlay counts for nothing either
    assert!(close_to(size(r"\draw (0,0) -- (1,0); \node[overlay] at (3,0) {x};"), size(base)));
}

#[test]
fn a_tikz_command_in_a_text_draws_its_picture_in_the_node() {
    // the inner picture is 1 cm square: the node is that plus its inner sep and line width
    let (w, h) = size(r"\node[draw] {\tikz \draw (0,0) rectangle (1,1);};");
    let pt = 2.54 / 72.27;
    let around = 2.0 * 3.333 * pt + 0.4 * pt;
    assert!((w - 1.0 - around - 0.4 * pt).abs() < 0.01 && (h - 1.0 - around - 0.4 * pt).abs() < 0.01, "{w} {h}");
}

#[test]
fn begin_in_a_text_without_an_environment_is_written_as_it_is() {
    for body in [r"\node {x \begin y};", r"\node {x \begin};"] {
        let (c, w) = canvas_and_warnings(body);
        assert_eq!(texts(&c).len(), 1);
        assert!(w.iter().any(|w| w.contains("unsupported text command: \\begin; written as is")), "{w:?}");
    }
}
