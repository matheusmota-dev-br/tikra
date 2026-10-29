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
fn preview_package_setup_is_neither_text_nor_a_warning() {
    // the preamble of many TeXample figures: nothing of it is drawn
    let plain = no_warnings(r"\begin{tikzpicture}\draw (0,0)--(1,0);\end{tikzpicture}");
    let r = no_warnings(
        r"\documentclass{article}
          \usepackage{tikz}
          \usepackage[active,tightpage]{preview}
          \PreviewEnvironment{tikzpicture}
          \setlength\PreviewBorder{5pt}%
          \begin{document}
          \begin{tikzpicture}\draw (0,0)--(1,0);\end{tikzpicture}
          \end{document}",
    );
    let drawing = |svg: &str| (svg[..svg.find(" role=").unwrap()].to_string(), svg[svg.find("</metadata>").unwrap()..].to_string());
    assert_eq!(drawing(&r.svg), drawing(&plain.svg));
}

#[test]
fn tikzpicture_with_options_and_spaces() {
    let r = no_warnings("\\begin{tikzpicture}  \n [scale=2]\n \\draw (0,0) -- (1,0);\n\\end{tikzpicture}");
    assert!(r.svg.contains("<path"));
}

#[test]
fn more_than_one_tikzpicture() {
    // as TeX sets them: side by side, a space between them; a blank line starts a new line
    let one = no_warnings(r"\begin{tikzpicture}\draw (0,0)--(1,0);\end{tikzpicture}");
    let two = no_warnings(r"\begin{tikzpicture}\draw (0,0)--(1,0);\end{tikzpicture} \begin{tikzpicture}\draw (0,0)--(2,0);\end{tikzpicture}");
    assert_eq!(two.svg.matches("<path").count(), 2);
    let width = |svg: &str| -> f64 { svg.split("width=\"").nth(1).unwrap().split("pt").next().unwrap().parse().unwrap() };
    // 1 cm + 3.33 pt + 2 cm (each picture half a line width wider at both ends), and the margins
    assert!((width(&two.svg) - width(&one.svg) - (2.0 * 28.3465 + (3.333 + 0.4) * 72.0 / 72.27)).abs() < 0.1, "{}", width(&two.svg));
    let lines = no_warnings("\\tikz \\draw (0,0)--(1,0);\n\n\\tikz \\draw (0,0)--(1,0);");
    assert!(width(&lines.svg) < width(&one.svg) + 0.1);
}

#[test]
fn scalebox_scales_the_pictures_in_it() {
    // the drawing, its lines and its size, by each factor (the second one up)
    let width = |svg: &str| -> f64 { svg.split("width=\"").nth(1).unwrap().split("pt").next().unwrap().parse().unwrap() };
    let one = no_warnings(r"\tikz \draw[line width=2pt] (0,0) -- (4,0);");
    let half = no_warnings(r"\scalebox{0.5}{\tikz \draw[line width=2pt] (0,0) -- (4,0);}");
    assert!((width(&one.svg) - width(&half.svg) - 2.0 * 28.3465 - 1.0).abs() < 0.1, "{}", width(&half.svg));
    assert!(half.svg.contains("stroke-width=\"1\"") || half.svg.contains("stroke-width=\"0.996\""), "{}", half.svg);
    let wide = no_warnings(r"\scalebox{2}[1]{\tikz \draw (0,0) circle (1);}");
    assert!(width(&wide.svg) > 4.0 * 28.3465, "{}", wide.svg);
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
fn center_lines_and_the_space_around_it() {
    // the height of the document's drawing, in pt
    let height = |doc: &str| -> f64 {
        let (c, _) = tikra::engine::run_document(doc).unwrap();
        (c.max.unwrap().y - c.min.unwrap().y) / tikra::PT
    };
    let pic = r"\begin{tikzpicture}\draw (0,0) rectangle (3,1);\end{tikzpicture}";
    // `\\` in `center` ends the paragraph: the blank line after it adds no empty line, so the text's
    // baseline is a \baselineskip (12pt) below the picture (28.85pt high), as TeX has it
    let h = height(&format!("\\begin{{center}}\n{pic}\n\\\\\n\nxx\n\\end{{center}}"));
    assert!((h - 40.85).abs() < 0.3, "{h}");
    // \topsep above and below the list (the space below it comes before the next line)
    let h = height(&format!("{pic}\n\\begin{{center}}\n{pic}\n\\end{{center}}\nxx"));
    assert!((h - 86.7).abs() < 0.3, "{h}");
    // in a \vbox the space below the list stays at its end, \partopsep more as it began a paragraph
    // (TeX's \vbox is 50.85pt high)
    let h = height(&format!("\\vbox\\bgroup\\hsize=12cm\\relax\n\\begin{{center}}\n{pic}\n\\\\\n\nxx\n\\end{{center}}\n\\egroup"));
    assert!((h - 50.85).abs() < 0.3, "{h}");
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
    // the ids of the words end in the figure's own suffix
    let svg = r.svg.replace(&r.svg[r.svg.find("tr-title-").unwrap() + 8..r.svg.find("tr-title-").unwrap() + 17], "");
    assert!(svg.contains(r#"aria-labelledby="tr-title" aria-describedby="tr-desc""#));
    assert!(svg.contains("<title id=\"tr-title\">Drawing of one shape with the text “&quot;a&quot; &amp; &lt;b&gt;”</title>"));
    assert!(svg.contains("<desc id=\"tr-desc\">Text in the figure: &quot;a&quot; &amp; &lt;b&gt;.</desc>"));
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

#[test]
fn pagecolor_paints_the_page_behind_the_figure() {
    // the whole page, margin included, under the figure; its argument is not text
    let doc = |pre: &str| {
        format!(r"\documentclass[border=5pt]{{standalone}}{pre}\begin{{document}}\begin{{tikzpicture}}\fill[blue] (0,0) circle (1);\end{{tikzpicture}}\end{{document}}")
    };
    let r = no_warnings(&doc(r"\pagecolor{olive!50!yellow!50!white}"));
    let view = &r.svg[r.svg.find("viewBox=\"").unwrap() + 9..];
    let view = &view[..view.find('"').unwrap()];
    let [x, y, w, h]: [&str; 4] = view.split(' ').collect::<Vec<_>>().try_into().unwrap();
    let body = &r.svg[r.svg.find("</defs>").unwrap()..];
    assert!(body.starts_with(&format!(r##"</defs><rect x="{x}" y="{y}" width="{w}" height="{h}" fill="#dfdf80"/>"##)), "{body}");
    assert!(!r.svg.contains("<use"), "no text: {}", r.svg);
    // a color model, and \nopagecolor taking it away
    assert!(no_warnings(&doc(r"\pagecolor[RGB]{200,30,30}")).svg.contains(r##"fill="#c81e1e"/>"##));
    assert!(!no_warnings(&doc(r"\pagecolor{red}\nopagecolor")).svg.contains("<rect"));
    let r = with_warning(&doc(r"\pagecolor{nocolor}"), "\\pagecolor: unknown color 'nocolor'");
    assert!(!r.svg.contains("<rect"));
}
