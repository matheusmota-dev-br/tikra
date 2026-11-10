//! Matrices, trees, chains and mind maps, the parts the other tests leave out: the separations a
//! cell or a row asks for, the basic layer's `\pgfmatrix`, the cell codes, `|[options]|`, a matrix
//! anchored on one of its cells; children without bodies, missing children and the other growth
//! functions; branches and the other chain keys; the bars of a mind map that cannot be drawn.

mod common;

use common::*;

/// The x distance from one point to another.
fn dx(body: &str, from: &str, to: &str) -> f64 {
    point(body, to).x - point(body, from).x
}

/// The y distance from one point to another.
fn dy(body: &str, from: &str, to: &str) -> f64 {
    point(body, to).y - point(body, from).y
}

/// The warnings of a body.
fn warnings(body: &str) -> Vec<String> {
    canvas_and_warnings(body).1
}

/// The colors of the texts of a body, in drawing order.
fn text_colors(body: &str) -> Vec<String> {
    texts(&canvas(body)).iter().map(|(l, _)| l.color.hex()).collect()
}

// ------------------------------------------------------------------ matrices

#[test]
fn a_cell_or_a_row_may_ask_for_more_separation() {
    let plain = r"\matrix (m) [matrix of nodes] { a & b \\ c & d \\ };";
    let wide = r"\matrix (m) [matrix of nodes] { a &[1cm] b \\[5mm] c & d \\ };";
    let more = dx(wide, "m-1-1", "m-1-2") - dx(plain, "m-1-1", "m-1-2");
    assert!(close(more, 1.0), "{more}");
    let more = dy(plain, "m-1-1", "m-2-1") - dy(wide, "m-1-1", "m-2-1");
    assert!(close(more, 0.5), "{more}");
    // between origins: the distance is the origins' own
    let origins = r"\matrix (m) [matrix of nodes] { a &[2cm,between origins] bbbbb \\ };";
    assert!(close(dx(origins, "m-1-1.base", "m-1-2.base"), 2.0));
}

#[test]
fn a_separation_that_is_not_a_length_is_reported() {
    let w = warnings(r"\matrix [column sep=wide] { \node {a}; & \node {b}; \\ };");
    assert!(w.iter().any(|w| w.contains("matrix separation 'wide' not understood: ignored")), "{w:?}");
}

#[test]
fn the_basic_layer_matrix_lays_out_its_cells() {
    // two cells side by side, a stroked frame named mm around them
    let body = r"\pgfmatrix{rectangle}{center}{mm}{\pgfusepath{stroke}}{\pgfpointorigin}{}{\node (x) {a}; \pgfmatrixnextcell \node (y) {b}; \pgfmatrixendrow}";
    let (c, w) = canvas_and_warnings(body);
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(texts(&c).len(), 2);
    let frame = paths(&c).into_iter().filter(|d| d.stroke.is_some()).count();
    assert_eq!(frame, 1, "the frame only");
    assert!(dx(body, "x", "y") > 0.0);
    // the frame is centered on the origin
    let center = point(body, "mm.center");
    assert!(close_to((center.x, center.y), (0.0, 0.0)) || center.x.abs() < 1e-6, "{center:?}");
    // filled: the fill usage too
    let (c, _) = canvas_and_warnings(r"\pgfmatrix{rectangle}{center}{}{\pgfusepath{fill,draw}}{\pgfpointorigin}{}{\node {a}; \pgfmatrixendrow}");
    assert!(paths(&c).iter().any(|d| d.fill.is_some() && d.stroke.is_some()));
}

#[test]
fn matrix_options_may_come_in_several_brackets() {
    let (c, w) = canvas_and_warnings(r"\matrix [draw] [matrix of nodes, nodes={red}] { a & b \\ };");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(paths(&c).iter().filter(|d| d.stroke.is_some()).count(), 1, "the matrix is drawn");
    assert_eq!(text_colors(r"\matrix [draw] [matrix of nodes, nodes={red}] { a & b \\ };"), ["#ff0000", "#ff0000"]);
}

#[test]
fn a_matrix_without_its_parts_is_dropped() {
    for (body, message) in [(r"\matrix at {a\\};", "\\matrix: at without a point"), (r"\matrix x {a\\};", "\\matrix: unexpected x")] {
        let (c, w) = canvas_and_warnings(body);
        assert!(w.iter().any(|w| w.contains(message)), "{body}: {w:?}");
        assert!(texts(&c).is_empty());
    }
}

#[test]
fn a_matrix_statement_may_end_without_a_semicolon() {
    // what follows the body is the next statement
    let (c, w) = canvas_and_warnings(r"\matrix { \node {a}; \\ } \draw (0,0) -- (1,0);");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!((texts(&c).len(), paths(&c).len()), (1, 1));
}

#[test]
fn the_basic_layer_cell_codes_run_around_the_cells() {
    // \pgfmatrixbegincode before each cell, \pgfmatrixemptycode in empty ones
    let body = r"\def\pgfmatrixbegincode{\color{blue}} \def\pgfmatrixemptycode{\node {e};} \matrix { \node {a}; & \\ };";
    let (c, w) = canvas_and_warnings(body);
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(text_colors(body), ["#0000ff", "#000000"]);
    assert_eq!(texts(&c).len(), 2);
    // the TikZ keys win over them
    let body = r"\def\pgfmatrixemptycode{\node {e};} \matrix [execute at empty cell={\node {f};}] { \node {a}; & \\ };";
    assert_eq!(texts(&canvas(body)).len(), 2);
}

#[test]
fn empty_cells_run_the_empty_cell_code() {
    // a matrix of nodes: the empty cell gets the code's node
    let body = r"\matrix (m) [matrix of nodes, execute at empty cell={\node[red] {-};}] { a & \\ & d \\ };";
    let (c, w) = canvas_and_warnings(body);
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(texts(&c).len(), 4);
    assert_eq!(text_colors(body).iter().filter(|c| *c == "#ff0000").count(), 2);
}

#[test]
fn cells_given_options_and_names_between_bars() {
    let body = r"\matrix (m) [matrix of nodes] { |[red]| a & |(nb)| b & |(nc) [blue]| c & |[green| d \\ };";
    let (c, w) = canvas_and_warnings(body);
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(texts(&c).len(), 4);
    let colors = text_colors(body);
    assert_eq!(colors[0], "#ff0000");
    assert_eq!(colors[2], "#0000ff");
    // the names are the cells' own, not m-1-2
    assert!(dx(body, "nb", "nc") > 0.0);
}

#[test]
fn a_matrix_anchored_on_one_of_its_cells() {
    // the cell's center is where the matrix goes; a node of that name made before is replaced
    let body = r"\node (m-2-1) at (9,9) {z}; \matrix (m) [matrix of nodes, anchor=m-2-1.center] at (5,5) { a & b \\ c & d \\ };";
    let p = point(body, "m-2-1.center");
    assert!(close_to((p.x, p.y), (5.0, 5.0)), "{p:?}");
    // with matrix anchor too
    let body = r"\matrix (m) [matrix of nodes, matrix anchor=m-1-2.center] at (1,2) { a & b \\ };";
    let p = point(body, "m-1-2.center");
    assert!(close_to((p.x, p.y), (1.0, 2.0)), "{p:?}");
}

#[test]
fn macros_in_the_body_are_read_for_their_separators() {
    // the row in a macro: its & and \\ count
    let body = r"\def\firstrow{a & b \\} \matrix (m) [matrix of nodes] { \firstrow c & d \\ };";
    let (c, w) = canvas_and_warnings(body);
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(texts(&c).len(), 4);
    assert!(dy(body, "m-1-2", "m-2-2") < 0.0);
}

#[test]
fn figures_cut_short_draw_what_they_have() {
    // the document ends in the middle of a statement: what was read is drawn, with a warning
    let run = |code: &str| tikra::engine::run_document(code).unwrap_or_else(|e| panic!("{e}\n{code}"));
    let has = |w: &[String], m: &str| w.iter().any(|w| w.contains(m));
    let (c, w) = run(r"\begin{tikzpicture}\matrix");
    assert!(has(&w, "\\matrix without a body; statement ignored"), "{w:?}");
    assert!(c.items.is_empty());
    // a matrix needs no ; at the very end
    let (c, w) = run(r"\begin{tikzpicture}\matrix {\node {a};\\}");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(texts(&c).len(), 1);
    // a child, children, a foreach: the root and an edge to each child
    for (code, edges) in
        [(r"\begin{tikzpicture}\node {r} child", 1), (r"\begin{tikzpicture}\node {r} child {node {a}} child", 2), (r"\begin{tikzpicture}\node {r} child foreach \x in {1,2}", 2)]
    {
        let (c, w) = run(code);
        assert!(has(&w, "path without ';' at the end"), "{code}: {w:?}");
        assert_eq!(paths(&c).len(), edges, "{code}");
    }
    // without its `in`, even when the document ends after an `i`, or without a variable
    for (code, message) in [
        (r"\begin{tikzpicture}\node {r} child foreach \x", "child foreach without 'in'; statement ignored"),
        (r"\begin{tikzpicture}\node {r} child foreach \x i", "child foreach without 'in'; statement ignored"),
        (r"\begin{tikzpicture}\node {r} child foreach in {1,2} {node {c}};", "child foreach without a variable; statement ignored"),
    ] {
        let (c, w) = run(code);
        assert!(has(&w, message), "{code}: {w:?}");
        assert!(paths(&c).is_empty(), "{code}: no edge");
    }
    let (c, w) = run(r"\begin{tikzpicture}\node (x) {x}; \begin{scope}[start chain] \chainin (x)");
    assert!(has(&w, "path without ';' at the end"), "{w:?}");
    assert_eq!(texts(&c).len(), 1);
}

#[test]
fn plain_empty_cells_and_bars_left_open() {
    let (c, w) = canvas_and_warnings(r"\matrix { \node {a}; & & \node {c}; \\ };");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(texts(&c).len(), 2);
    // a bar with no closing one is part of the text
    let (c, w) = canvas_and_warnings(r"\matrix [matrix of nodes] { |x \\ };");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(texts(&c).len(), 1);
}

// ------------------------------------------------------------------ trees

#[test]
fn a_child_needs_a_node_before_it() {
    let w = warnings(r"\path child {node {a}};");
    assert!(w.iter().any(|w| w.contains("child without a node before it")), "{w:?}");
}

#[test]
fn an_unnamed_parent_is_joined_to_its_children() {
    // the root has no name: the edges start at it all the same
    let (c, w) = canvas_and_warnings(r"\node {r} child {node {a}} child {node {b}};");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(texts(&c).len(), 3);
    let edges = paths(&c);
    assert_eq!(edges.len(), 2);
    for e in &edges {
        let (lo, hi) = bounds(&e.segments);
        assert!(hi.y < 0.0 && lo.y > -1.5, "from the root down to a child: {:?}", e.segments);
    }
}

#[test]
fn a_node_halfway_along_a_line_can_have_children() {
    // m is at (1,0), unnamed; its child one level distance below it
    let (c, w) = canvas_and_warnings(r"\draw (0,0) -- node {m} (2,0) child {node {c}};");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(texts(&c).len(), 2);
    let p = paths(&c);
    assert_eq!(p.len(), 2, "the line and the edge");
    let edge = p.iter().find(|d| bounds(&d.segments).1.y < 0.0).expect("the edge");
    let (lo, hi) = bounds(&edge.segments);
    assert!(close(lo.x, 1.0) && close(hi.x, 1.0) && lo.y > -1.5, "{:?}", edge.segments);
}

#[test]
fn children_without_bodies_are_coordinates() {
    // child child: two children with nothing in them; the edges still go to where they are
    let (c, w) = canvas_and_warnings(r"\node (r) {r} child child;");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(texts(&c).len(), 1);
    assert_eq!(paths(&c).len(), 2);
    // an empty body is the same
    let a = point(r"\node (r) {r} child {} child {};", "r-2");
    let b = point(r"\node (r) {r} child child;", "r-2");
    assert!(close_to((a.x, a.y), (b.x, b.y)), "{a:?} {b:?}");
    // a foreach without a body: one coordinate per item
    let (c, _) = canvas_and_warnings(r"\node (r) {r} child foreach \x in {1,2,3};");
    assert_eq!(paths(&c).len(), 3);
}

#[test]
fn a_missing_child_takes_its_place_but_draws_nothing() {
    let body = r"\node (r) {r} child {node {a}} child[missing] child {node {c}};";
    let (c, w) = canvas_and_warnings(body);
    assert!(w.is_empty(), "{w:?}");
    assert_eq!((texts(&c).len(), paths(&c).len()), (3, 2));
    // the third is two sibling distances from the first
    assert!(close(dx(body, "r-1", "r-3"), 3.0));
}

#[test]
fn options_at_the_start_of_a_child_hold_for_its_path() {
    // the child itself is placed before them, at the level distance in force
    let body = r"\node (r) {r} child {[level distance=5mm] node (a) {a} child {node (b) {b}}};";
    assert!(close(dy(body, "r", "a"), -1.5), "{}", dy(body, "r", "a"));
    assert!(close(dy(body, "a", "b"), -0.5), "{}", dy(body, "a", "b"));
}

#[test]
fn growing_clockwise_and_counterclockwise_from_an_angle() {
    // three children, 20 degrees apart, at the level distance
    let ccw = r"\node (r) {r} [counterclockwise from=0, sibling angle=90] child {node {a}} child {node {b}};";
    let a = point(ccw, "r-2");
    assert!(close_to((a.x, a.y), (0.0, 1.5)), "{a:?}");
    let cw = r"\node (r) {r} [clockwise from=0, sibling angle=90] child {node {a}} child {node {b}};";
    let a = point(cw, "r-2");
    assert!(close_to((a.x, a.y), (0.0, -1.5)), "{a:?}");
}

#[test]
fn growing_via_three_points() {
    let body = r"\node (r) {r} [grow via three points={one child at (0,-1) and two children at (-1,-1) and (1,-1)}] child {node {a}} child {node {b}};";
    let a = point(body, "r-1");
    let b = point(body, "r-2");
    assert!(close_to((a.x, a.y), (-1.0, -1.0)), "{a:?}");
    assert!(close_to((b.x, b.y), (1.0, -1.0)), "{b:?}");
    let one = point(r"\node (r) {r} [grow via three points={one child at (0,-2) and two children at (-1,-1) and (1,-1)}] child {node {a}};", "r-1");
    assert!(close_to((one.x, one.y), (0.0, -2.0)), "{one:?}");
}

// ------------------------------------------------------------------ chains

#[test]
fn branches_start_at_the_last_node_and_come_back() {
    let body = r"\begin{scope}[start chain=main going right, node distance=1cm]
        \node[on chain] (a) {a}; \node[on chain] (b) {b};
        { [start branch=side going below] \node[on chain] (c) {c}; }
        \node[on chain] (d) {d};
        { [continue branch=side] \node[on chain] (e) {e}; }
      \end{scope}";
    let (_, w) = canvas_and_warnings(body);
    assert!(w.is_empty(), "{w:?}");
    // c below b, d right of b, e below c
    assert!(close(point(body, "c").x, point(body, "b").x));
    assert!(point(body, "c").y < point(body, "b").y);
    assert!(close(point(body, "d").y, point(body, "b").y));
    assert!(close(point(body, "e").x, point(body, "c").x) && point(body, "e").y < point(body, "c").y);
    // the branch's nodes are named after it
    assert!(close(point(body, "main/side-1").y, point(body, "c").y));
}

#[test]
fn chain_keys_with_unknown_chains_are_reported() {
    for (body, message) in [
        (r"\node[start branch=b] {x};", "start branch: no chain to branch from ('chain')"),
        (r"\node[continue chain=nowhere] {x};", "continue chain: unknown chain 'nowhere'"),
        (r"\begin{scope}[start chain=c] \node[on chain] {x}; { [continue branch=b] } \end{scope}", "continue branch: unknown chain 'c/b'"),
        (r"\node[on chain=nowhere] {x};", "on chain: unknown chain 'nowhere'"),
    ] {
        let w = warnings(body);
        assert!(w.iter().any(|w| w.contains(message)), "{body}: {w:?}");
    }
}

#[test]
fn continue_chain_turns_it_another_way() {
    let body = r"\begin{scope}[start chain=c going right, node distance=1cm]
        \node[on chain] (a) {a}; \node[on chain] (b) {b};
        \node[continue chain=going below, on chain] (d) {d};
        \node[continue chain=c going left, on chain] (e) {e};
      \end{scope}";
    let (_, w) = canvas_and_warnings(body);
    assert!(w.is_empty(), "{w:?}");
    assert!(close(point(body, "d").x, point(body, "b").x) && point(body, "d").y < point(body, "b").y);
    assert!(close(point(body, "e").y, point(body, "d").y) && point(body, "e").x < point(body, "d").x);
}

#[test]
fn a_default_direction_and_a_placed_chain() {
    // the default direction holds for the chains started after it
    let body = r"\begin{scope}[chain default direction=going above, start chain, node distance=1cm]
        \node[on chain] (a) {a}; \node[on chain] (b) {b}; \end{scope}";
    assert!(close(point(body, "b").x, point(body, "a").x) && point(body, "b").y > point(body, "a").y);
    // placed: every node where the options say, even the first
    let body = r"\begin{scope}[start chain=p placed {xshift=\tikzchaincount cm}]
        \node[on chain] (a) {a}; \node[on chain] (b) {b}; \end{scope}";
    let (_, w) = canvas_and_warnings(body);
    assert!(w.is_empty(), "{w:?}");
    let (a, b) = (point(body, "a"), point(body, "b"));
    assert!(close_to((a.x, a.y), (1.0, 0.0)), "{a:?}");
    assert!(close_to((b.x, b.y), (2.0, 0.0)), "{b:?}");
}

#[test]
fn joins_with_a_named_node_or_before_on_chain() {
    // join=with a: an edge from a, whatever the previous node
    let body = r"\begin{scope}[start chain=going right] \node[on chain] (a) {a}; \node[on chain] (b) {b}; \node[on chain, join=with a] (c) {c}; \end{scope}";
    let (c, w) = canvas_and_warnings(body);
    assert!(w.is_empty(), "{w:?}");
    let edges = paths(&c);
    assert_eq!(edges.len(), 1);
    let (lo, hi) = bounds(&edges[0].segments);
    assert!(lo.x < point(body, "a.east").x + 1e-6 && hi.x > point(body, "c.west").x - 1e-6, "{lo:?} {hi:?}");
    // join given before on chain waits for it; the first node has nothing to join
    let body = r"\begin{scope}[start chain=going right] \node[join, on chain] (a) {a}; \node[join=by red, on chain] (b) {b}; \end{scope}";
    let (c, w) = canvas_and_warnings(body);
    assert!(w.is_empty(), "{w:?}");
    let edges = paths(&c);
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].stroke.as_ref().map(|s| s.color.hex()).as_deref(), Some("#ff0000"));
}

#[test]
fn chainin_needs_an_existing_node() {
    for (body, message) in [(r"\chainin;", "\\chainin without (node)"), (r"\chainin (nobody);", "\\chainin: unknown node (nobody)")] {
        let w = warnings(&format!(r"\begin{{scope}}[start chain] {body} \end{{scope}}"));
        assert!(w.iter().any(|w| w.contains(message)), "{body}: {w:?}");
    }
    // without options, the node simply joins
    let body = r"\node (x) at (3,0) {x}; \begin{scope}[start chain=going right] \node[on chain] (a) {a}; \chainin (x) [join]; \end{scope}";
    let (c, w) = canvas_and_warnings(body);
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(paths(&c).len(), 1);
}

// ------------------------------------------------------------------ mind maps

#[test]
fn a_circle_connection_bar_needs_two_circles_apart() {
    let lib = r"\usetikzlibrary{mindmap}";
    for (body, message) in [
        (r"\node[circle] (a) {a}; \path (a) to[circle connection bar] (3,0);", "a circle connection bar connects two nodes"),
        (r"\node[circle] (a) {a}; \coordinate (b) at (3,0); \path (a) to[circle connection bar] (b);", "cannot connect a node of shape 'coordinate'"),
    ] {
        let w = warnings(&format!("{lib}{body}"));
        assert!(w.iter().any(|w| w.contains(message)), "{body}: {w:?}");
    }
    // two circles at the same place: no bar between them
    let (c, w) = canvas_and_warnings(&format!(r"{lib}\node[circle, draw] (a) {{a}}; \node[circle] (b) {{b}}; \path[fill] (a) to[circle connection bar] (b);"));
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(paths(&c).len(), 1, "the circle only");
}
