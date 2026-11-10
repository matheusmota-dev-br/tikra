//! `\graph`, the parts the first graph tests leave out: the arrows written backwards, quoted node
//! names, numbered, fresh and anonymous nodes, existing nodes and sets, tries, quick graphs, the
//! rarer operators, the placement with separations, and the keys that hand an operator's work to
//! TeX code (`\tikzgraphforeachcolorednode`, `\tikzgraphpreparecolor`).

mod common;

use common::*;
use tikra::scene::Segment;

/// The texts (their origins, cm) and paths of a figure, with its warnings.
fn draw(body: &str) -> (Vec<(f64, f64)>, Vec<Drawn>, Vec<String>) {
    let (c, w) = canvas_and_warnings(body);
    let t = texts(&c).iter().map(|(_, p)| (p.e, p.f)).collect();
    (t, paths(&c), w)
}

/// The number of texts and paths of a figure that must draw without warnings.
fn counts(body: &str) -> (usize, usize) {
    let (t, p, w) = draw(body);
    assert!(w.is_empty(), "{w:?}\n{body}");
    (t.len(), p.len())
}

/// The names of the nodes a figure names, in the order they were first placed.
fn names(body: &str) -> Vec<String> {
    let out = no_warnings(&format!("\\begin{{tikzpicture}}{body}\\end{{tikzpicture}}"));
    out.points.iter().map(|p| p.name.clone()).collect()
}

/// The first point of a path.
fn start(d: &Drawn) -> (f64, f64) {
    match d.segments[0] {
        Segment::MoveTo(p) => (p.x, p.y),
        ref s => panic!("a path starts with {s:?}"),
    }
}

/// The last point of a path.
const RED: &str = "#ff0000";
const BLUE: &str = "#0000ff";
const BLACK: &str = "#000000";

fn end(d: &Drawn) -> (f64, f64) {
    points(&d.segments).last().copied().expect("an empty path")
}

/// The color of the first text of a figure.
fn text_color(body: &str) -> String {
    let c = canvas(body);
    texts(&c).first().map(|(l, _)| l.color.hex()).expect("no text")
}

// ------------------------------------------------------------------ arrows

#[test]
fn arrows_written_backwards_put_the_tip_at_the_first_node() {
    // a <- b is an edge from a to b with the tip at a; <-> has a tip at both ends
    for (graph, tips) in [(r"a <- b", 1), (r"a <-> b", 2), (r"{a} <- b", 1), (r"{a} <-> b", 2)] {
        let (t, p, w) = draw(&format!(r"\graph {{ {graph} }};"));
        assert!(w.is_empty(), "{w:?}");
        assert_eq!(t.len(), 2, "{graph}");
        assert_eq!(p.len(), 1 + tips, "{graph}: the edge and {tips} tips");
        // the edge goes from a (at 0) to b (at 1 cm), the first tip sits at a
        assert!(start(&p[0]).0 < end(&p[0]).0, "{graph}");
        let (lo, _) = bounds(&p[1].segments);
        assert!(lo.x < 0.5, "{graph}: the tip is at a, not b");
    }
    // a plain - is read as --
    assert_eq!(counts(r"\graph { a - b };"), (2, 1));
}

#[test]
fn a_group_followed_by_anything_but_an_arrow_is_reported() {
    let (t, _, w) = draw(r"\graph { {a} + b };");
    assert!(w.iter().any(|w| w.contains("one of the arrow types <-, --, ->, -!- or <-> expected, not '+'")), "{w:?}");
    // the group is drawn, what follows is dropped
    assert_eq!(t.len(), 1);
}

// ------------------------------------------------------------------ the graph itself

#[test]
fn a_graph_without_braces_is_dropped() {
    // the statement is dropped with a warning
    for code in [r"\tikz \path graph [nodes=red] x;", r"\tikz \path graph"] {
        let out = with_warning(code, "graph without '{'; statement ignored");
        assert_eq!(out.description, "Empty figure.", "{code}");
    }
}

#[test]
fn empty_chains_take_no_room() {
    // {a, , b} places b where {a, b} does: an empty chain adds no depth
    let with = point(r"\graph { a, , b };", "b");
    let without = point(r"\graph { a, b };", "b");
    assert!(close_to((with.x, with.y), (without.x, without.y)), "{with:?} {without:?}");
    assert!(close(with.y, -1.0), "{with:?}");
}

#[test]
fn sublayouts_need_the_graphdrawing_library() {
    let (t, _, w) = draw(r"\graph { a // b };");
    assert!(w.iter().any(|w| w.contains("sublayouts (//) need the graphdrawing library")), "{w:?}");
    assert!(t.is_empty(), "the element is dropped");
}

#[test]
fn a_foreach_without_in_drops_the_graph_even_inside_another() {
    // the whole graph is dropped
    let out = with_warning(r"\tikz \graph { \foreach \x in {1,2} { \foreach \y {a} } };", "graph: \\foreach without 'in'; statement ignored");
    assert_eq!(out.description, "Empty figure.");
}

// ------------------------------------------------------------------ names

#[test]
fn quoted_names_spell_out_the_characters_a_name_cannot_hold() {
    // every special character becomes @ its Unicode name @; the text keeps them
    let n = names(r#"\graph [empty nodes] { "$&^~_|[]()/.-,+*'!:;<=>?`\%\#@{}" -- "a""b" };"#);
    let expected = [
        "DOLLAR SIGN",
        "AMPERSAND",
        "CIRCUMFLEX ACCENT",
        "TILDE",
        "LOW LINE",
        "VERTICAL LINE",
        "LEFT SQUARE BRACKET",
        "RIGHT SQUARE BRACKET",
        "LEFT PARENTHESIS",
        "RIGHT PARENTHESIS",
        "SOLIDUS",
        "FULL STOP",
        "HYPHEN MINUS",
        "COMMA",
        "PLUS SIGN",
        "ASTERISK",
        "APOSTROPHE",
        "EXCLAMATION MARK",
        "COLON",
        "SEMINCOLON",
        "LESS THAN SIGN",
        "EQUALS SIGN",
        "GREATER THAN SIGN",
        "QUESTION MARK",
        "GRAVE ACCENT",
        "REVERSE SOLIDUS",
        "PERCENT SIGN",
        "REVERSE SOLIDUS",
        "NUMBER SIGN",
        "COMMERCIAL AT",
        "LEFT CURLY BRACE",
        "RIGHT CURLY BRACE",
    ]
    .iter()
    .map(|c| format!("@{c}@"))
    .collect::<String>();
    assert_eq!(n, [expected, "a@QUOTATION MARK@b".to_string()], "{n:?}");
}

#[test]
fn a_quoted_name_may_have_a_text_of_its_own() {
    // "p q"/r: the node is named p q and shows r
    let (t, p, w) = draw(r#"\graph { "p q"/r -> s }; \draw (p q) circle (1pt);"#);
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(t.len(), 2);
    assert_eq!(p.len(), 3, "the edge, its tip and the circle");
    // the same quoted name later is the same node
    assert_eq!(counts(r#"\graph { "x y" -> b; "x y" -> c };"#), (3, 4));
}

#[test]
fn an_empty_name_makes_a_new_node_each_time() {
    // /a twice: two anonymous nodes showing a, one edge between them
    let (t, p, w) = draw(r"\graph { /a -> /a };");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!((t.len(), p.len()), (2, 2));
    assert!(close(t[1].0 - t[0].0, 1.0), "{t:?}");
}

#[test]
fn numbered_nodes_are_new_nodes_with_a_number_after_their_name() {
    let n = names(r"\graph [number nodes] { a -> a -> a };");
    assert_eq!(n, ["a 1", "a 2", "a 3"]);
    let n = names(r"\graph [number nodes=5, number nodes sep=-] { x -- y };");
    assert_eq!(n, ["x-5", "y-6"]);
    // in a quick graph too
    let n = names(r#"\graph [quick, number nodes] { "a" -- "a" };"#);
    assert_eq!(n, ["a 1", "a 2"]);
}

#[test]
fn fresh_nodes_get_primes_rather_than_reuse_a_name() {
    let n = names(r"\graph [fresh nodes] { a -> a -> a };");
    assert_eq!(n, ["a", "a'", "a''"]);
}

#[test]
fn a_trie_names_each_node_after_the_path_to_it() {
    // the second chain goes through the same a and a b, then branches to a b d
    let n = names(r"\graph [trie] { a -> b -> c; a -> b -> d };");
    assert_eq!(n, ["a", "a b", "a b c", "a b d"]);
    assert_eq!(counts(r"\graph [trie] { a -> b -> c; a -> b -> d };"), (4, 8));
}

// ------------------------------------------------------------------ nodes made elsewhere

#[test]
fn parentheses_use_nodes_made_outside_the_graph() {
    let body = r"\node (p) at (0,0) {p}; \node (q) at (3,0) {q}; \graph { (p) -> (q) };";
    let (t, p, w) = draw(body);
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(t.len(), 2, "no new node");
    assert_eq!(p.len(), 2, "one edge and its tip");
    assert!(start(&p[0]).0 < 0.5 && end(&p[0]).0 > 2.5, "{:?}", p[0].segments);
}

#[test]
fn use_existing_nodes_draws_only_the_edges() {
    let (t, p, w) = draw(r"\node (a) {a}; \node (b) at (2,1) {b}; \graph [use existing nodes] { a -> b };");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!((t.len(), p.len()), (2, 2));
    assert!(close_to(end(&p[0]), (2.0, 1.0)) || end(&p[0]).0 > 1.5, "{:?}", p[0].segments);
}

#[test]
fn sets_collect_nodes_and_a_graph_uses_them_by_name() {
    let body = r"\tikzset{new set=s} \node[set=s] (a) {a}; \node[set=s] at (0,1) {b}; \graph { (s) -> c };";
    let (t, p, w) = draw(body);
    assert!(w.is_empty(), "{w:?}");
    // a and the unnamed b both go to c
    assert_eq!((t.len(), p.len()), (3, 4));
    // a set must be made first
    let (_, _, w) = draw(r"\node[set=nowhere] {a};");
    assert!(w.iter().any(|w| w.contains("undefined set 'nowhere'")), "{w:?}");
}

// ------------------------------------------------------------------ edge options on nodes

#[test]
fn source_and_target_edge_options_add_up_and_clear() {
    // b [>red, >"t"]: the edge into b is red and has the text t
    let (t, p, w) = draw(r#"\graph { a -> b [>red, >"t"] };"#);
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(t.len(), 3);
    assert_eq!(p[0].stroke.as_ref().map(|s| s.color.hex()).as_deref(), Some(RED));
    // clear > forgets them
    let (t, p, _) = draw(r#"\graph { a -> b [>red, >"t", clear >] };"#);
    assert_eq!(t.len(), 2);
    assert_eq!(p[0].stroke.as_ref().map(|s| s.color.hex()).as_deref(), Some(BLACK));
    // <blue: the edge out of a
    let (_, p, _) = draw(r"\graph { a [<blue] -> b };");
    assert_eq!(p[0].stroke.as_ref().map(|s| s.color.hex()).as_deref(), Some(BLUE));
}

#[test]
fn quotes_on_a_join_take_braced_options_and_more_options_after_them() {
    // "x"{red}: the text, red; then a dashed edge
    let (c, w) = canvas_and_warnings(r#"\graph { a ->["x"{red}, dashed] b };"#);
    assert!(w.is_empty(), "{w:?}");
    let t = texts(&c);
    assert_eq!(t.len(), 3);
    assert!(t.iter().any(|(l, _)| l.color.hex() == RED));
    assert!(paths(&c)[0].stroke.as_ref().is_some_and(|s| s.dash.is_some()));
}

#[test]
fn node_text_can_go_on_the_edges_instead() {
    // every node gives its text to the edges into it: a has none, so its text is gone; b's is
    // on the edge, halfway, and b itself is empty
    let (c, w) = canvas_and_warnings(r"\graph [put node text on incoming edges] { a -> b };");
    assert!(w.is_empty(), "{w:?}");
    let t = texts(&c);
    assert_eq!(t.len(), 1);
    assert!(t[0].1.e > 0.0 && t[0].1.e < 1.0, "{:?}", t[0].1);
    // outgoing, with options: a's text, red, on the edge out of it
    let (c, _) = canvas_and_warnings(r"\graph [put node text on outgoing edges={red}] { a -> b };");
    let t = texts(&c);
    assert_eq!(t.len(), 1);
    assert_eq!(t[0].0.color.hex(), RED);
}

#[test]
fn options_given_to_a_node_already_made_do_not_restyle_it() {
    // the second a only takes part in the edges: it stays black
    assert_eq!(text_color(r"\graph { a -> b; a [red] -> c };"), BLACK);
    // a key neither TikZ nor the graphs library knows is still reported
    let (t, p, w) = draw(r"\graph { a -> b; a [red, unknown key] -> c };");
    assert_eq!(w, ["option 'unknown key' not supported: ignored"]);
    assert_eq!((t.len(), p.len()), (3, 4));
}

#[test]
fn options_with_braces_and_unclosed_brackets() {
    // a label in braces is one option
    let (t, _, w) = draw(r"\graph { a [label={above:L}] };");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(t.len(), 2);
}

// ------------------------------------------------------------------ operators

#[test]
fn the_two_shore_operators() {
    // matching: the i-th with the i-th, nothing for those left over
    assert_eq!(counts(r"\graph { {a, b, c} -- [matching] {d, e} };"), (5, 2));
    assert_eq!(counts(r"\graph { {a} -- [matching] {d, e} };"), (3, 1));
    // matching and star: the left-over ones go to the last of the other side
    assert_eq!(counts(r"\graph { {a, b, c} -- {d} };"), (4, 3));
    // complete bipartite leaves out a node on both sides
    assert_eq!(counts(r"\graph { {a, b} -- [complete bipartite] {b, c} };"), (3, 3));
}

#[test]
fn butterflies_cross_the_shores() {
    let g = |o: &str| counts(&format!(r"\graph {{ {{a, b, c, d}} -- [{o}] {{e, f, g, h}} }};"));
    // the crossing edges and the straight ones; butterfly' only the crossing ones
    assert_eq!(g("butterfly"), (8, 8));
    assert_eq!(g("butterfly'"), (8, 4));
    assert_eq!(g("butterfly={level=2}"), (8, 8));
    // level 0: no crossing, just the straight edges
    assert_eq!(g("butterfly'={level=0}"), (8, 4));
    // which ones: with level 1, e goes to b and a to f
    let (_, p, _) = draw(r"\graph [grow down, branch right] { {a, b} -- [butterfly'] {e, f} };");
    let ends: Vec<_> = p.iter().map(|d| (start(d), end(d))).collect();
    assert_eq!(ends.len(), 2);
    for (s, e) in &ends {
        assert!((s.0 - e.0).abs() > 0.5, "the edges cross: {ends:?}");
    }
}

#[test]
fn an_induced_independent_set_removes_the_edges_of_a_simple_graph() {
    assert_eq!(counts(r"\graph [simple] { a -- b -- c; { [induced independent set] a, b } };"), (3, 1));
    assert_eq!(counts(r"\graph [simple] { a -- b -- c; { [induced cycle] a, b, c } };"), (3, 3));
}

#[test]
fn unknown_operators_are_reported() {
    let (_, _, w) = draw(r"\graph { a -- [operator={\pgfkeys{/tikz/graphs/@op={knot}{}}}] b };");
    assert!(w.iter().any(|w| w.contains("graph: operator 'knot' not supported")), "{w:?}");
}

#[test]
fn operators_written_in_tex() {
    // \tikzgraphinvokeoperator runs another operator now
    assert_eq!(counts(r"\graph { [operator={\tikzgraphinvokeoperator{clique}}] a, b, c };"), (3, 3));
    // \tikzgraphforeachcolorednode gives each node of a color to a macro
    let body = r"\def\dot#1{\fill (#1) circle (2pt);} \graph { [operator={\tikzgraphforeachcolorednode{all}{\dot}}] a, b, c };";
    let (t, p, w) = draw(body);
    assert!(w.is_empty(), "{w:?}");
    assert_eq!((t.len(), p.len()), (3, 3));
    assert!(p.iter().all(|d| d.fill.is_some()));
    // \tikzgraphpreparecolor counts them and names each one
    let body = r"\newcount\many \graph { [operator={\tikzgraphpreparecolor{all}\many{n}\draw (\csname n1\endcsname) -- (\csname n\the\many\endcsname);}] a, b, c };";
    let (_, p, w) = draw(body);
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(p.len(), 1);
    // from a (at the top) to c (two steps down)
    assert!(start(&p[0]).1 > end(&p[0]).1 + 1.0, "{:?}", p[0].segments);
}

#[test]
fn parse_puts_text_before_the_group() {
    assert_eq!(counts(r"\graph { [parse={a -> b,}] c };"), (3, 2));
}

// ------------------------------------------------------------------ placement

#[test]
fn grow_right_sep_leaves_the_separation_between_the_nodes() {
    let body = r"\graph [grow right sep=1cm] { a -> bbb -> c };";
    let (ae, bw) = (point(body, "a.east"), point(body, "bbb.west"));
    assert!(close(bw.x - ae.x, 1.0), "{ae:?} {bw:?}");
    let (be, cw) = (point(body, "bbb.east"), point(body, "c.west"));
    assert!(close(cw.x - be.x, 1.0), "{be:?} {cw:?}");
    // downwards the same, between south and north
    let body = r"\graph [grow down sep=5mm] { a -> b };";
    let (s, n) = (point(body, "a.south"), point(body, "b.north"));
    assert!(close(s.y - n.y, 0.5), "{s:?} {n:?}");
}

// ------------------------------------------------------------------ quick graphs

#[test]
fn quick_graphs_read_quoted_nodes_and_connectors_only() {
    let (t, p, w) = draw(r#"\graph [quick] { "a" -> "b" [red]; "c"/"C" -- "a"; "b" <-> "c" };"#);
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(t.len(), 3);
    // a -> b (edge, tip), c -- a (edge), b <-> c (edge, two tips)
    assert_eq!(p.len(), 6);
    // groups with options, edge options with quotes
    let (c, w) = canvas_and_warnings(r#"\graph [quick] { { [nodes=red] "d", "e" }; "d" ->["x"] "e" };"#);
    assert!(w.is_empty(), "{w:?}");
    let t = texts(&c);
    assert_eq!(t.len(), 3);
    assert_eq!(t[0].0.color.hex(), RED);
    // anything else is reported and skipped
    let (t, _, w) = draw(r#"\graph [quick] { "a", x, "b" };"#);
    assert!(w.iter().any(|w| w.contains("graph [quick]: unexpected 'x'")), "{w:?}");
    assert_eq!(t.len(), 2);
    // a text after / must be quoted too: unquoted, there is none, and a is drawn empty
    let (t, p, _) = draw(r#"\graph [quick] { "a"/b };"#);
    assert!(t.is_empty(), "{t:?}");
    assert!(p.is_empty(), "no edge");
}

#[test]
fn edge_options_with_values_reach_the_edges() {
    let (_, p, w) = draw(r"\graph { a -- [line width=2pt] b };");
    assert!(w.is_empty(), "{w:?}");
    let width = p[0].stroke.as_ref().map(|s| s.width).unwrap();
    assert!(close(width, 2.0 / 72.27 * 2.54), "{width}");
}

// ------------------------------------------------------------------ edge node

#[test]
fn edge_node_puts_nodes_on_a_to_path() {
    // edge label on a plain to: the text above the middle of the line
    let (t, _, w) = draw(r"\draw (0,0) to[edge label=m] (2,0);");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(t.len(), 1);
    assert!(t[0].0 > 0.5 && t[0].0 < 1.5 && t[0].1 > 0.0, "{t:?}");
    // what is not a node is left alone
    assert_eq!(draw(r"\draw (0,0) to[edge node={nothing}] (2,0);").0.len(), 0);
}

// ------------------------------------------------------------------ edge cases

#[test]
fn operators_on_no_nodes_make_no_edges() {
    // a cycle of a color no node has, a matching with an empty side
    assert_eq!(counts(r"\graph { [cycle=nobody] a, b, c };"), (3, 0));
    assert_eq!(counts(r"\graph { {} -- d };"), (1, 0));
    assert_eq!(counts(r"\graph { {a, b} -- {} };"), (2, 0));
}

#[test]
fn graph_keys_outside_a_graph_do_nothing() {
    let (t, p, w) = draw(r"\tikzgraphinvokeoperator{source} \tikzgraphsset{source edge style=red, source edge clear} \draw (0,0) -- (1,0);");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!((t.len(), p.len()), (0, 1));
    assert_eq!(p[0].stroke.as_ref().map(|s| s.color.hex()).as_deref(), Some(BLACK));
}

#[test]
fn a_set_needs_a_name() {
    let (t, _, w) = draw(r"\node[set] {a};");
    assert!(w.iter().any(|w| w.contains("undefined set ''")), "{w:?}");
    assert_eq!(t.len(), 1, "the node is drawn all the same");
}

#[test]
fn a_node_renamed_away_has_no_width_for_the_separation() {
    // a is called z instead: the next node starts one separation after z's west side, as if z had
    // no width
    let body = r"\graph [grow right sep=1cm] { a [/tikz/name=z] -> c };";
    let (z, c) = (point(body, "z.west"), point(body, "c.west"));
    assert!(close(c.x - z.x, 1.0), "{z:?} {c:?}");
}

#[test]
fn an_option_list_left_open_takes_the_rest_of_the_element() {
    // an option list left open takes the rest of the element
    let (c, _) = canvas_and_warnings(r"\graph { a [red };");
    assert_eq!(texts(&c)[0].0.color.hex(), RED);
}
