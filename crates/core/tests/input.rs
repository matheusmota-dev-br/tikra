//! The input forms `convert` accepts and the repairs made before reading.

mod common;
use common::*;

#[test]
fn standalone_document_with_border() {
    let r = no_warnings(
        r"\documentclass[tikz,border=1cm]{standalone}
          \usepackage{amsmath}
          \usetikzlibrary{arrows.meta}
          \pgfplotsset{compat=1.18}
          \begin{document}
          \begin{tikzpicture} \draw (0,0) -- (1,0); \end{tikzpicture}
          \end{document}",
    );
    // 1 cm of line + 2 cm of margin = 3 cm ≈ 85 pt
    assert!(r.svg.contains(r#"width="85."#), "{}", &r.svg[..200]);
}

#[test]
fn standalone_without_border_uses_default_margin() {
    // documentclass with options but no border: a 2pt margin
    let r = no_warnings(r"\documentclass[tikz]{standalone}\begin{document}\begin{tikzpicture}\draw (0,0)--(1,0);\end{tikzpicture}\end{document}");
    assert!(r.svg.contains(r#"width="32."#), "{}", &r.svg[..200]);
    // a border whose value is not a length: default margin
    no_warnings(r"\documentclass[border={abc}]{standalone}\begin{document}\begin{tikzpicture}\draw (0,0)--(1,0);\end{tikzpicture}\end{document}");
    // unclosed documentclass[
    no_warnings(r"\documentclass[border=3pt \begin{tikzpicture}\draw (0,0)--(1,0);\end{tikzpicture}");
}

#[test]
fn tikzpicture_with_options_and_spaces() {
    let r = no_warnings("\\begin{tikzpicture}  \n [scale=2]\n \\draw (0,0) -- (1,0);\n\\end{tikzpicture}");
    assert!(r.svg.contains("<path"));
}

#[test]
fn more_than_one_tikzpicture() {
    with_warning(r"\begin{tikzpicture}\draw (0,0)--(1,0);\end{tikzpicture} \begin{tikzpicture}\draw (0,0)--(2,0);\end{tikzpicture}", "more than one");
}

#[test]
fn inline_tikz_with_and_without_braces() {
    no_warnings(r"\tikz \draw (0,0) -- (1,1);");
    no_warnings(r"\tikz[thick] {\draw (0,0) -- (1,1); \fill (0,0) circle (2pt);}");
    no_warnings(r"\tikz  [thick]  \draw (0,0) -- (1,1);");
    // \tikzset in the preamble does not count as \tikz
    no_warnings(r"\tikzset{a/.style={red}} \tikz \draw[a] (0,0) -- (1,1);");
}

#[test]
fn bare_axis_and_loose_statements() {
    no_warnings(r"\begin{axis}[xmin=0,xmax=1,ymin=0,ymax=1] \addplot coordinates {(0,0) (1,1)}; \end{axis}");
    no_warnings(r"\draw (0,0) -- (1,1);");
    no_warnings(r"\coordinate (A) at (1,1); \fill (A) circle (1pt);");
}

#[test]
fn no_figure_is_an_error() {
    let e = tikra::convert("just some text, no figure").unwrap_err();
    assert!(e.to_string().contains("no tikzpicture"), "{e}");
    assert!(tikra::convert("").is_err());
}

#[test]
fn errors_have_a_message_per_kind() {
    assert_eq!(tikra::Error::Expression("x".into()).to_string(), "expression: x");
    assert_eq!(tikra::Error::Figure("y".into()).to_string(), "figure: y");
    assert_eq!(tikra::Error::Png("z".into()).to_string(), "png: z");
    let e: Box<dyn std::error::Error> = Box::new(tikra::Error::Figure("w".into()));
    assert!(e.to_string().contains('w'));
}

#[test]
fn doubled_backslashes_are_undone() {
    let r = with_warning(r"\\begin{tikzpicture} \\draw (0,0) -- (1,1); \\end{tikzpicture}", "doubled backslashes");
    assert!(r.svg.contains("<path"));
}

#[test]
fn markdown_fences_commands_and_brackets() {
    // a code fence with escaped \[ \]
    with_warning("```latex\n\\draw\\[red\\] (0,0) -- (1,1);\n```", "Markdown");
    // one command with a doubled backslash among normal ones
    let r = with_warning("\\begin{tikzpicture} \\\\draw (0,0) -- (1,1); \\node at (0,0) {a}; \\end{tikzpicture}", "Markdown");
    assert!(r.svg.contains("<path"));
    // \\[2pt] (a line break with spacing) is left alone
    no_warnings(r"\begin{tikzpicture} \node[align=center] at (0,0) {a \\[2pt] b}; \end{tikzpicture}");
    // \[ at the start of the text
    with_warning("\\[\\draw (0,0) -- (1,1);", "Markdown");
}

#[test]
fn preamble_defines_styles_colors_and_macros() {
    let r = no_warnings(
        r"\documentclass{standalone}
          \usepackage{tikz}
          \definecolor{brand}{RGB}{200,30,30}
          \tikzset{arrow/.style={->, thick, brand}}
          \def\L{3}
          \begin{document}
          \begin{tikzpicture} \draw[arrow] (0,0) -- (\L,0); \end{tikzpicture}
          \end{document}",
    );
    assert!(r.svg.contains("#c81e1e"), "color from the preamble");
}

#[test]
fn comments_are_removed_but_escaped_percent_stays() {
    let r = no_warnings("\\begin{tikzpicture} % a comment with \\draw (9,9)\n \\draw (0,0) -- (1,0); \\node at (0,1) {50\\%};\n\\end{tikzpicture}");
    assert!(r.svg.contains("<use"), "the 50% label is drawn");
}

#[test]
fn svg_has_title_and_escaped_source() {
    let r = no_warnings(r#"\begin{tikzpicture} \draw (0,0) -- (1,0) node[right] {"a" & <b>}; \end{tikzpicture}"#);
    assert!(r.svg.contains("<title id=\"tr-title\">Figure</title>"));
    assert!(r.svg.contains("<tikra:source type=\"tikz\">"));
    assert!(r.svg.contains("&quot;a&quot; &amp; &lt;b&gt;"));
}

#[test]
fn version_and_unit() {
    assert!(!tikra::VERSION.is_empty());
    assert!((tikra::PT * 72.27 - 2.54).abs() < 1e-12);
}

#[test]
fn errors_that_bring_the_figure_down() {
    // tikzpicture and preamble options with an invalid computation
    assert!(tikra::convert(r"\begin{tikzpicture}[scale=zz] \draw (0,0) -- (1,0); \end{tikzpicture}").is_err());
    assert!(tikra::convert(r"\begin{scope}[scale=zz]\end{scope} \begin{tikzpicture} \draw (0,0) -- (1,0); \end{tikzpicture}").is_err());
    // an empty border: default margin
    no_warnings(r"\documentclass[border={}]{standalone}\begin{document}\begin{tikzpicture}\draw (0,0)--(1,0);\end{tikzpicture}\end{document}");
}
