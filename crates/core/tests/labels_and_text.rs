//! The text of nodes as `tikra::label` sets it: font commands, typewriter text, rules and boxes of
//! a width, paragraphs, lists and tabulars, and LaTeX's `picture` environment. Most tests compare
//! two ways of writing the same thing, which TeX sets alike.

use tikra::color::Color;
use tikra::label::{compose_label, font_size};
use tikra::scene::Label;

/// The label of `text`, on one line or in a `text width` (cm), and its warnings.
fn set(text: &str, font: &str, width: Option<f64>, align: char) -> (Label, Vec<String>) {
    let mut w = Vec::new();
    let l = compose_label(text, font, width, align, Color::BLACK, &mut w);
    (l, w)
}

/// The label of `text` alone, which must give no warning.
fn label(text: &str) -> Label {
    let (l, w) = set(text, "", None, 'c');
    assert!(w.is_empty(), "{text}: {w:?}");
    l
}

/// The width of `text`, in cm.
fn width(text: &str) -> f64 {
    label(text).width()
}

fn same_width(a: &str, b: &str) {
    let (x, y) = (width(a), width(b));
    assert!((x - y).abs() < 1e-4, "{a}: {x}, {b}: {y}");
}

const PT: f64 = 2.54 / 72.27;

// ---------------------------------------------------------------- font commands

#[test]
fn font_sizes_by_command() {
    assert_eq!(font_size(r"\fontsize{20}{24}\selectfont"), 2.0);
    // a size that is not a number changes nothing
    assert_eq!(font_size(r"\fontsize{big}{24}\selectfont"), 1.0);
    // \fontsize's arguments are not text; Computer Modern at 20pt is a narrower design
    let (a, b) = (width(r"\fontsize{20}{24}\selectfont A"), width("A"));
    assert!(a > 1.7 * b && a < 2.0 * b, "{a} {b}");
    // at the end of the text, short of its arguments: nothing of it is set
    same_width(r"A\fontsize", "A");
}

#[test]
fn commands_that_set_nothing() {
    // \relax, and the spaces after it, are nothing
    same_width(r"a\relax b", "ab");
    // a size command after a control word keeps that word apart from what follows
    same_width(r"\S\small x", r"\small\S x");
}

#[test]
fn verb_is_typewriter_text() {
    same_width(r"\verb!ab!", r"\texttt{ab}");
    // each character as wide as any other: underscores, signs, parentheses
    same_width(r"\verb!a_b!", r"\verb!abc!");
    same_width(r"\verb!f(x)!", r"\verb!fxyz!");
    // typewriter text has no ligatures: two hyphens are two characters
    same_width(r"\texttt{a--b}", r"\texttt{abcd}");
}

#[test]
fn empty_family_commands_set_nothing() {
    same_width(r"a\texttt{}b", "ab");
    same_width(r"a\textit{}b", "ab");
}

#[test]
fn groups_inside_a_family_command() {
    same_width(r"\textit{a{b}c}", r"\textit{abc}");
}

#[test]
fn long_arrows_of_amsmath() {
    same_width(r"$a \implies b$", r"$a \;\Longrightarrow\; b$");
    same_width(r"$a \iff b$", r"$a \;\Longleftrightarrow\; b$");
}

// ---------------------------------------------------------------- rules and boxes

#[test]
fn rules_of_a_depth_and_without_a_unit() {
    let deep = label(r"a\vrule depth 5pt");
    assert!((deep.lines[0].depth as f64 * deep.scale - 5.0 * PT).abs() < 1e-3, "{}", deep.lines[0].depth);
    // a dimension without a unit ends the rule's specification: the default width
    same_width(r"a\vrule width 2 b", r"a\vrule{} width 2 b");
}

#[test]
fn boxes_of_a_width() {
    for text in [r"\hbox to 2cm{\hss a}", r"\hbox to 2cm{a\hss}", r"\hbox to 2cm{\hss a\hss}"] {
        assert!((width(text) - 2.0).abs() < 1e-3, "{text}: {}", width(text));
    }
    // without \hss, or with math, the box is its content
    same_width(r"\hbox to 2cm{a}", "a");
    same_width(r"\hbox to 2cm{\hss $a$}", "$a$");
}

// ---------------------------------------------------------------- lines and paragraphs

#[test]
fn par_and_vskip_end_a_line() {
    let a = label(r"a\par\vskip 10pt b");
    let b = label(r"a\\[10pt] b");
    assert_eq!(a.lines.len(), 2);
    assert!((a.skips[0] - b.skips[0]).abs() < 1e-9, "{:?} {:?}", a.skips, b.skips);
}

#[test]
fn paragraphs_break_after_hyphens_and_punctuation() {
    let (l, _) = set("Fixed-Parameter Tractability", "", Some(2.0), 'l');
    assert!(l.lines.len() >= 2);
    // justified text: wider spaces after a sentence, less after a semicolon
    let (j, _) = set("One. Two; three, four: five six seven eight nine.", "", Some(3.0), 'j');
    assert!(j.lines.len() >= 2);
    for b in &j.lines[..j.lines.len() - 1] {
        assert!((b.width as f64 * j.scale - 3.0).abs() < 0.05, "{}", b.width as f64 * j.scale);
    }
}

#[test]
fn an_empty_paragraph_is_one_empty_line() {
    let (l, _) = set("", "", Some(2.0), 'l');
    assert_eq!(l.lines.len(), 1);
    assert_eq!(l.lines[0].width, 0.0);
}

#[test]
fn a_paragraph_exactly_as_wide_as_its_text() {
    let w = width("abc def");
    let (l, _) = set("abc def", "", Some(w), 'j');
    assert_eq!(l.lines.len(), 1);
}

#[test]
fn lists_in_a_paragraph() {
    let list = r"\begin{itemize}\item one\item two\end{itemize}";
    // \topsep before, \itemsep and \parsep between items: smaller in \small and \footnotesize
    for (font, between) in [("", 8.0), (r"\small", 4.0), (r"\footnotesize", 4.0)] {
        let (l, _) = set(&format!("Before {list} after"), font, Some(4.0), 'l');
        assert_eq!(l.lines.len(), 4, "{font}");
        assert!((l.skips[1] - between * PT).abs() < 1e-6, "{font}: {:?}", l.skips);
    }
    // an item's own label, and an item's lines after its first indented as far
    let (l, _) = set(r"\begin{itemize}\item[--] a long item with words enough to wrap\end{itemize}", "", Some(2.5), 'l');
    assert!(l.lines.len() >= 2);
    let (n, _) = set(r"\begin{enumerate}\item a\item b\end{enumerate}", "", Some(3.0), 'l');
    assert_eq!(n.lines.len(), 2);
}

#[test]
fn tabular_columns() {
    // right and left columns, an `@{…}` between them, cells by `&`
    let l = label(r"\begin{tabular}{r@{.}l} 12&5 \\ 1&25 \end{tabular}");
    assert_eq!(l.lines.len(), 2);
    assert!((l.lines[0].width - l.lines[1].width).abs() < 1e-3);
    let l = label(r"\begin{tabular}{l|c} a&b \\ ccc&d \end{tabular}");
    assert_eq!(l.lines.len(), 2);
}

// ---------------------------------------------------------------- pictures

#[test]
fn pictures_of_lines_and_circles() {
    // a picture is a box of its size; slanted lines and rings are drawn in it
    let l = label(r"\begin{picture}(20,10)\put(0,0){\line(2,1){20}}\put(10,5){\circle{6}}\end{picture}");
    assert!((l.width() - 20.0 * PT).abs() < 1e-3, "{}", l.width());
    use mathra::layout::Item;
    let items = &l.lines[0].items;
    assert!(items.iter().any(|i| matches!(i, Item::Line { .. })));
    assert!(items.iter().any(|i| matches!(i, Item::Ellipse { .. })));
}

#[test]
fn a_picture_in_a_scaled_box() {
    let l = label(r"\scalebox{2}{\begin{picture}(10,10)\put(0,0){\circle*{4}}\end{picture}}");
    assert!((l.width() - 20.0 * PT).abs() < 1e-3, "{}", l.width());
}

#[test]
fn lines_of_their_own_size() {
    // a smaller line with a rule in it: the rule is smaller too
    let l = label(r"a\vrule width 10pt\\\tiny b\vrule width 10pt");
    assert!(l.lines[1].width < l.lines[0].width);
}

#[test]
fn picture_settings_and_what_is_not_supported() {
    let warned = |text: &str| set(text, "", None, 'c').1;
    // \unitlength set without a picture after it, or not to a length, or without braces
    assert!(warned(r"\begin{picture}(1,1)\end{picture}\setlength{\unitlength}{2pt}").is_empty());
    assert_eq!(warned(r"\setlength{\unitlength}{wide}\begin{picture}(1,1)\end{picture}"), ["\\unitlength: 'wide' is not a length; kept 1pt"]);
    for (unit, w) in [("1in", 72.27), ("72bp", 72.27), ("65536sp", 1.0)] {
        let l = label(&format!(r"\setlength{{\unitlength}}{{{unit}}}\begin{{picture}}(1,1)\end{{picture}}"));
        assert!((l.width() - w * PT).abs() < 1e-3, "{unit}: {}", l.width());
    }
    // (what follows `\unitlength` without braces stays text: the unit is still 1pt)
    let (l, _) = set(r"\setlength\unitlength 2pt\begin{picture}(10,1)\end{picture}", "", None, 'c');
    assert!(l.width() > 10.0 * PT && l.width() < 30.0 * PT, "{}", l.width());
    let (l, _) = set(r"\setlength{\unitlength}{2pt\begin{picture}(10,1)\end{picture}", "", None, 'c');
    assert!(l.width() > 0.0);
    // a picture without its end is text
    assert!(set(r"x\begin{picture}(1,1)", "", None, 'c').0.width() > width("x"));
    // what the picture environment has and tikra does not
    let w = warned(
        r"\begin{picture}(10,10)\thicklines\thinlines\linethickness{1pt}\qbezier(0,0)(1,1)(2,0)% a comment
\put(0,0){\vector(1,0){5}}\put(0,0){\oval(4,4)}\put(0,0){text}\put(0,0){\,}\put(0,0){\thicklines\line(1,1){3}}\end{picture}",
    );
    for expected in [
        "picture: \\qbezier not supported: ignored",
        "picture: \\vector drawn as a line",
        "picture: \\oval not supported: ignored",
        "picture: 'text' not supported: ignored",
        "picture: '\\,' not supported: ignored",
    ] {
        assert!(w.iter().any(|w| w == expected), "{expected}: {w:?}");
    }
}

#[test]
fn a_verb_without_its_closing_delimiter_is_not_verbatim() {
    for text in [r"x\verb!ab", r"x\verb"] {
        let (_, w) = set(text, "", None, 'c');
        assert!(w.iter().any(|w| w.contains("verb")), "{text}: {w:?}");
    }
}

#[test]
fn a_command_that_begins_like_vrule_is_not_a_rule() {
    let (_, w) = set(r"a\vrulex b", "", None, 'c');
    assert!(w.iter().any(|w| w.contains("vrulex")), "{w:?}");
}

#[test]
fn groups_in_boxes_and_cells() {
    assert!((width(r"\hbox to 2cm{\hss {a}}") - 2.0).abs() < 1e-3);
    let l = label(r"\begin{tabular}{ll} {a}&b \\ c&{d} \end{tabular}");
    assert_eq!(l.lines.len(), 2);
    same_width(r"\begin{tabular}{l} {ab} \end{tabular}", r"\begin{tabular}{l} ab \end{tabular}");
}

#[test]
fn par_with_spaces_before_its_vskip() {
    let a = label("a\\par   \\vskip 10pt b");
    let b = label(r"a\\[10pt] b");
    assert!((a.skips[0] - b.skips[0]).abs() < 1e-9, "{:?} {:?}", a.skips, b.skips);
}

#[test]
fn drawn_marks_in_a_line_of_another_size() {
    use mathra::layout::Item;
    // a line in \tiny: its strokes and circles half as large
    let small = label(r"a\\\tiny $\cancel{x}$ $\textcircled{b}$");
    let large = label(r"$\cancel{x}$ $\textcircled{b}$");
    let thick = |l: &Label, k: usize| l.lines[k].items.iter().find_map(|i| if let Item::Line { thickness, .. } = i { Some(*thickness) } else { None }).unwrap();
    let ring = |l: &Label, k: usize| l.lines[k].items.iter().find_map(|i| if let Item::Ellipse { rx, .. } = i { Some(*rx) } else { None }).unwrap();
    assert!((thick(&small, 1) / thick(&large, 0) - 0.5).abs() < 0.01);
    assert!(ring(&small, 1) < 0.75 * ring(&large, 0));
}

#[test]
fn a_picture_in_a_turned_box() {
    use mathra::layout::Item;
    let l = label(r"\rotatebox{90}{\begin{picture}(10,4)\put(0,0){\circle*{4}}\end{picture}}");
    let inner = l.lines[0].items.iter().find_map(|i| if let Item::Transformed { items, .. } = i { Some(items) } else { None }).unwrap();
    assert!(inner.iter().any(|i| matches!(i, Item::Ellipse { .. })));
}

#[test]
fn a_command_that_begins_like_implies_is_not_an_arrow() {
    let (_, w) = set(r"$a \impliesx b$", "", None, 'c');
    assert!(w.iter().any(|w| w.contains("impliesx")), "{w:?}");
}
