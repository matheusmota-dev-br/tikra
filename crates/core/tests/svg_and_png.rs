//! The generated SVG (bounds, labels, clips, opacities) and PNG rasterization.

mod common;
use common::*;
use tikra::canvas::{Canvas, Item};
use tikra::color::Color;
use tikra::scene::{Label, Transform};

fn png_width(png: &[u8]) -> u32 {
    u32::from_be_bytes([png[16], png[17], png[18], png[19]])
}

#[test]
fn svg_with_opacities_dashes_and_drawn_nodes() {
    let r = no_warnings(
        r"\begin{tikzpicture}
            \filldraw[fill opacity=0.4, draw opacity=0.6, dashed] (0,0) rectangle (1,1);
            \node[draw, fill=yellow, dashed, rounded corners] at (3,0) {box};
            \node[draw, circle, fill=red] at (5,0) {c};
            \node[draw, diamond] at (7,0) {d};
            \node[draw, rounded corners=1mm, minimum size=1cm] at (9,0) {};
          \end{tikzpicture}",
    );
    for fragment in ["fill-opacity=\"0.4\"", "stroke-opacity=\"0.6\"", "stroke-dasharray", "#fff200", "<use href=\"#tr-g"] {
        assert!(r.svg.contains(fragment), "{fragment}");
    }
}

#[test]
fn multiline_label_and_alignments() {
    for a in ["left", "right", "center"] {
        let r = no_warnings(&format!(r"\begin{{tikzpicture}} \node[align={a}] {{line one\\ two}}; \end{{tikzpicture}}"));
        assert!(r.svg.contains("<g transform="), "{a}");
    }
    // math with a fraction and a root: mathra's rules and lines
    let r = no_warnings(r"\begin{tikzpicture} \node {$\frac{a}{b} + \sqrt{x} + \overline{y}$}; \end{tikzpicture}");
    assert!(r.svg.contains("<rect"));
}

#[test]
fn mathra_items_with_color_and_ellipse() {
    let r = figure(
        r"\begin{tikzpicture} \node {$\textcolor{red}{x} + \cancel{y} + \boxed{z} + \fcolorbox{red}{yellow}{w}$}; \node at (0,1) {$\bigcirc$}; \node at (0,2) {$\textcolor{red}{\frac{a}{b}}$}; \end{tikzpicture}",
    );
    assert!(r.svg.contains("<svg"));
}

#[test]
fn empty_canvas_has_minimum_size() {
    let svg = tikra::driver::svg(&Canvas::default(), "", &Default::default());
    assert!(svg.contains("viewBox"));
    // a text whose label has no lines
    let mut c = Canvas::default();
    let label = Label { lines: Vec::new(), scale: 1.0, color: Color::BLACK, align: 'c', line_height: 0.0, stretch: 1.0, skips: Vec::new(), font: tikra::label::FONT, lift: 0.0 };
    assert_eq!(label.depth(), 0.0);
    assert_eq!(label.height(), 0.0);
    c.push(Item::Text { label, place: Transform::default(), opacity: 1.0 });
    assert!(tikra::driver::svg(&c, "", &Default::default()).contains("viewBox"));
}

#[test]
fn nested_clips() {
    let r = no_warnings(r"\begin{tikzpicture} \begin{scope} \clip (0,0) circle (1); \clip (0,0) rectangle (2,2); \fill (0,0) rectangle (3,3); \end{scope} \end{tikzpicture}");
    assert!(r.svg.contains("tr-c2"));
    assert_eq!(r.svg.matches("<g clip-path").count(), 2);
}

#[test]
fn png_with_and_without_background_and_dpi() {
    let r = no_warnings(r"\begin{tikzpicture} \draw (0,0) -- (2.54,0); \end{tikzpicture}");
    let a = tikra::png(&r.svg, 96.0, true).unwrap();
    let b = tikra::png(&r.svg, 192.0, false).unwrap();
    assert!(png_width(&b) > png_width(&a) * 19 / 10);
    assert_ne!(a, b);
}

#[test]
fn png_rejects_invalid_svg_and_bad_dpi() {
    assert!(matches!(tikra::png("this is not svg", 300.0, true), Err(tikra::Error::Png(_))));
    let r = no_warnings(r"\begin{tikzpicture} \draw (0,0) -- (1,0); \end{tikzpicture}");
    for dpi in [0.0, -10.0, f32::NAN, f32::INFINITY] {
        let e = tikra::png(&r.svg, dpi, true).unwrap_err();
        assert!(e.to_string().contains("invalid dpi"), "{dpi}: {e}");
    }
    // a tiny dpi: the image vanishes
    assert!(tikra::png(&r.svg, 0.001, true).is_err());
    // a huge dpi: goes past the pixel cap instead of allocating gigabytes
    let e = tikra::png(&r.svg, 1e6, true).unwrap_err();
    assert!(e.to_string().contains("too large"), "{e}");
}

#[test]
fn a_striped_pattern_has_no_seams_between_its_tiles() {
    // horizontal stripes: along a row every pixel has the stripe's color, none lighter where tiles meet
    let r = no_warnings(r"\tikz \fill[pattern=horizontal lines light blue] (0,0) rectangle (2,1);");
    let png = tikra::png(&r.svg, 100.0, true).unwrap();
    let image = resvg::tiny_skia::Pixmap::decode_png(&png).unwrap();
    let row: Vec<u8> = (10..60).map(|x| image.pixel(x, 20).unwrap().red()).collect();
    assert!(row.iter().all(|&v| v == row[0]), "{row:?}");
}

#[test]
fn a_filled_and_stroked_path_at_one_opacity_is_one_group() {
    // as PDF paints fill and stroke: the stroke does not darken the fill under it
    let svg = no_warnings(r"\tikz \filldraw[fill=red, opacity=0.5, line width=2mm] (0,0) rectangle (1,1);").svg;
    assert!(svg.contains(r#" opacity="0.5""#) && !svg.contains("fill-opacity") && !svg.contains("stroke-opacity"), "{svg}");
    // different opacities stay apart
    let svg = no_warnings(r"\tikz \filldraw[fill=red, fill opacity=0.5, draw opacity=0.3] (0,0) rectangle (1,1);").svg;
    assert!(svg.contains("fill-opacity") && svg.contains("stroke-opacity"), "{svg}");
}

#[test]
fn a_stroke_that_lets_through_shows_what_is_under_the_path_not_its_fill() {
    // PDF paints fill and stroke as a knockout group: under a translucent stroke, the fill is not there
    let out = no_warnings(r"\tikz \filldraw[fill=yellow, draw opacity=0.5, line width=4mm] (0,0) rectangle (2,2);");
    assert!(out.svg.contains("<mask") && out.svg.contains("stroke-opacity"), "{}", out.svg);
    let image = resvg::tiny_skia::Pixmap::decode_png(&tikra::png(&out.svg, 100.0, true).unwrap()).unwrap();
    // the stroke's inner half (over the fill) is the same gray as its outer half (over the white)
    let (inner, outer) = (image.pixel(10, 40).unwrap(), image.pixel(5, 40).unwrap());
    assert_eq!((inner.red(), inner.green(), inner.blue()), (outer.red(), outer.green(), outer.blue()));
    assert_eq!(inner.blue(), inner.red());
}

#[test]
fn the_lines_of_a_label_take_its_color() {
    // a molecule's bonds are lines in currentColor: the label sets color, not only fill
    let svg = no_warnings(r"\tikz \node[text=red] {$\chemfig{H_3C-OH}$};").svg;
    assert!(svg.contains(r##"fill="#ff0000" color="#ff0000""##) && svg.contains(r#"stroke="currentColor""#), "{svg}");
}

#[test]
fn construction_animation() {
    let code = r"\begin{tikzpicture} \draw[->] (0,0) -- (1,1); \draw[dashed] (0,1) -- (1,0); \fill (2,0) circle (2pt); \node at (3,0) {$x$}; \end{tikzpicture}";
    let still = tikra::convert(code).unwrap().svg;
    assert!(!still.contains("<style>") && !still.contains("class=\"tr-"));
    let svg = tikra::convert_with(code, &tikra::Options::default().animated()).unwrap().svg;
    assert!(svg.contains("<style>@keyframes tr-d"));
    // five steps (the arrow tip is one); the solid line is drawn along its length, the dashed one,
    // the dot and the label fade in
    assert!(svg.contains(r#"data-steps="5""#) && svg.contains(r#"data-step="0" class="tr-d" pathLength="1" style="--i:0""#));
    assert!(!svg.split("stroke-dasharray=").nth(1).unwrap().split("/>").next().unwrap().contains("tr-d"));
    assert!(svg.contains(r#"<g transform="#) && svg.matches(r#"class="tr-f""#).count() >= 3);
    // the drawing is the same: only the attributes of the steps and the style are added
    let svg_only = svg.replace(&svg[svg.find("<style>").unwrap()..svg.find("</style>").unwrap() + 8], "");
    let plain = without_steps(&svg_only);
    assert_eq!(plain, still);
    // a PNG of the animated SVG is the still figure
    assert_eq!(tikra::png(&svg, 72.0, true).unwrap(), tikra::png(&still, 72.0, true).unwrap());
}

#[test]
fn steps_without_the_animation() {
    let code = r"\begin{tikzpicture} \draw (0,0) -- (1,1); \fill (2,0) circle (2pt); \node at (3,0) {$x$}; \end{tikzpicture}";
    let still = tikra::convert(code).unwrap().svg;
    let svg = tikra::convert_with(code, &tikra::Options::default().steps()).unwrap().svg;
    // numbered in the order drawn, counted on the <svg>, and no CSS
    assert!(svg.contains(r#"data-steps="3""#) && svg.contains(r#"data-step="0""#) && svg.contains(r#"data-step="2""#), "{svg}");
    assert!(!svg.contains("<style>") && !svg.contains("class=\"tr-"));
    assert_eq!(without_steps(&svg), still);
}

#[test]
fn each_element_knows_the_code_that_drew_it() {
    let code = "\\newcommand{\\both}{\\draw (0,0) -- (1,0); \\draw (0,1) -- (1,1);}\n\\begin{tikzpicture}\n  \\draw (0,0) -- (1,1); \\fill (2,0) circle (2pt);\n  \\both\n  \\foreach \\i in {1,2} {\n    \\draw (\\i,2) -- (\\i,3);\n  }\n  \\draw (0,0)\n    -- (2,2);\n\\end{tikzpicture}";
    let plain = tikra::convert_with(code, &tikra::Options::default().steps()).unwrap().svg;
    let svg = tikra::convert_with(code, &tikra::Options::default().source()).unwrap().svg;
    assert_eq!(sources(&svg), ["3:3-3:23", "3:25-3:49", "4:3-4:7", "4:3-4:7", "5:3-7:3", "5:3-7:3", "8:3-9:13"], "{svg}");
    assert_eq!(without_sources(&svg), plain);
    // the animation carries them too
    let animated = tikra::convert_with(code, &tikra::Options::default().source().animated()).unwrap().svg;
    assert_eq!(sources(&animated), sources(&svg));
}

#[test]
fn an_addplot_keeps_its_own_line_though_the_axis_draws_at_its_end() {
    let code = "\\begin{tikzpicture}\n\\begin{axis}\n\\addplot coordinates {(0,0) (1,1)};\n\\draw (axis cs:0,0) -- (axis cs:1,0);\n\\addplot coordinates {(0,1) (1,0)};\n\\end{axis}\n\\end{tikzpicture}";
    let svg = tikra::convert_with(code, &tikra::Options::default().source()).unwrap().svg;
    let lines: Vec<_> = sources(&svg).into_iter().map(|s| s.split(':').next().unwrap().to_string()).collect();
    for line in ["2", "3", "4", "5"] {
        assert!(lines.iter().any(|l| l == line), "line {line} in {lines:?}");
    }
    assert!(!lines.iter().any(|l| l == "6"), "{lines:?}");
}

#[test]
fn the_lines_are_the_ones_of_the_code_given() {
    // loose statements: tikra puts a picture around them
    let svg = tikra::convert_with("\\draw (0,0) -- (1,1);\n\\draw (1,0) -- (0,1);", &tikra::Options::default().source()).unwrap().svg;
    assert_eq!(sources(&svg), ["1:1-1:21", "2:1-2:21"]);
}

#[test]
fn layers_and_pictures_in_commands_keep_their_lines() {
    let code = "\\pgfdeclarelayer{back}\\pgfsetlayers{back,main}\n\\begin{tikzpicture}\n\\draw (0,0) -- (1,0);\n\\begin{pgfonlayer}{back}\n\\fill[red] (0,0) rectangle (1,1);\n\\end{pgfonlayer}\n\\end{tikzpicture}\n\\tikz{\\draw (0,0) -- (1,1);}";
    let svg = tikra::convert_with(code, &tikra::Options::default().source()).unwrap().svg;
    assert_eq!(sources(&svg), ["5:1-5:33", "3:1-3:21", "8:1-8:28"], "{svg}");
}

/// The places of the elements of an SVG, in order.
fn sources(svg: &str) -> Vec<String> {
    svg.split(r#" data-src=""#).skip(1).map(|s| s.split('"').next().unwrap().to_string()).collect()
}

/// The SVG without the places of its elements.
fn without_sources(svg: &str) -> String {
    let mut s = svg.to_string();
    while let Some(i) = s.find(r#" data-src=""#) {
        let end = i + 11 + s[i + 11..].find('"').unwrap() + 1;
        s.replace_range(i..end, "");
    }
    s
}

/// The SVG without the steps' attributes.
fn without_steps(svg: &str) -> String {
    let mut s = svg.replace(r#" class="tr-d" pathLength="1""#, "").replace(r#" class="tr-f""#, "");
    for attr in [r#" style="--i:"#, r#" data-step=""#, r#" data-steps=""#] {
        while let Some(i) = s.find(attr) {
            let end = i + attr.len() + s[i + attr.len()..].find('"').unwrap() + 1;
            s.replace_range(i..end, "");
        }
    }
    s
}

#[test]
fn fills_side_by_side_leave_no_seam_between_them() {
    // two squares meeting inside a pixel: the white under them does not show along their edge
    let out = no_warnings(r"\tikz[x=0.25cm] { \fill[red] (0,0) rectangle (1,1); \fill[red] (1,0) rectangle (2,1); }");
    let image = resvg::tiny_skia::Pixmap::decode_png(&tikra::png(&out.svg, 100.0, true).unwrap()).unwrap();
    let row: Vec<u8> = (3..image.width() - 3).map(|x| image.pixel(x, image.height() / 2).unwrap().green()).collect();
    assert!(row.iter().all(|&g| g < 10), "{row:?}");
    // a fill folded back over another one gets no stroke: it would stick out of both
    let svg = no_warnings(r"\tikz { \fill[red] (0,0) -- (1,0) -- (1,1) -- cycle; \fill[blue] (0,0) -- (1,0) -- (0.5,0.5) -- cycle; }").svg;
    assert!(!svg.contains("<g><path"), "{svg}");
}

#[test]
fn each_thing_drawn_is_a_part_with_its_words() {
    let code = "\\begin{tikzpicture}\n\\node (a) at (0,0) {Start};\n\\node (b) at (3,0) {End};\n\\draw[->] (a) -- (b);\n\\draw (0,1) -- (1,1);\n\\end{tikzpicture}";
    let plain = tikra::convert_with(code, &tikra::Options::default().source()).unwrap();
    let r = tikra::convert_with(code, &tikra::Options::default().parts()).unwrap();
    let texts: Vec<&str> = r.parts.iter().map(|p| p.text.as_str()).collect();
    assert_eq!(texts, ["“Start”", "“End”", "from Start to End", "one shape"]);
    assert_eq!(r.parts[0].source, "2:1-2:27");
    assert!(r.svg.contains(r#"role="group" aria-labelledby"#) && r.svg.contains(r#"role="img" tabindex="0" aria-label="from Start to End""#), "{}", r.svg);
    // the words come with the source map too, and the SVG only with the parts
    assert_eq!(plain.parts, r.parts);
    assert!(!plain.svg.contains("data-part") && plain.svg.contains(r#"role="img""#));
    assert!(!tikra::convert(code).unwrap().svg.contains("data-part"));
}

#[test]
fn a_step_has_a_sentence_or_nothing() {
    let code = "\\begin{tikzpicture}\n\\node (a) at (0,0) {Start};\n\\node (b) at (3,0) {End};\n\\draw[->] (a) -- (b);\n\\draw (0,1) -- (1,1);\n\\end{tikzpicture}";
    let r = tikra::convert_with(code, &tikra::Options::default().source()).unwrap();
    let said: Vec<&str> = r.parts.iter().map(|p| p.sentence.as_str()).collect();
    assert_eq!(said, ["Text in the figure: Start.", "Text in the figure: End.", "Connections: from Start to End.", ""]);
    let pt = tikra::convert_with(code, &tikra::Options::locale("pt-BR".parse().unwrap()).source()).unwrap();
    assert!(pt.parts[2].sentence.contains("Start") && pt.parts[2].sentence != r.parts[2].sentence, "{}", pt.parts[2].sentence);
}

#[test]
fn a_reader_stops_once_at_each_part() {
    let code = "\\begin{tikzpicture}\\node[draw] (a) {A};\\draw[->] (0,1) -- (1,1);\\clip (0,0) rectangle (1,1);\\end{tikzpicture}";
    let svg = tikra::convert_with(code, &tikra::Options::default().parts()).unwrap().svg;
    // every drawn element is inside a stop or hidden from the reader
    let body = svg.split("</defs>").nth(1).unwrap();
    assert!(body.matches(r#"role="img" tabindex="0""#).count() >= 2, "{body}");
    assert!(!body.contains(r#"role="group""#), "{body}");
    assert_eq!(body.matches("data-part=").count(), body.matches(r#"role="img""#).count() + body.matches(r#"aria-hidden="true""#).count(), "{body}");
}

#[test]
fn the_parts_are_told_in_every_language() {
    let code = "\\begin{tikzpicture}\\node (a) {A};\\node (b) at (2,0) {B};\\draw[->] (a) -- (b);\\draw (0,1) -- (1,1);\\end{tikzpicture}";
    for code_of in ["en", "pt-BR", "es", "fr", "de", "it", "ru", "zh", "ja", "ko", "hi", "ar", "id", "tr", "nl", "pl"] {
        let locale = code_of.parse().unwrap();
        let r = tikra::convert_with(code, &tikra::Options::locale(locale).parts()).unwrap();
        assert_eq!(r.parts.len(), 4, "{code_of}");
        assert!(r.parts.iter().all(|p| !p.text.trim().is_empty() && !p.text.contains('{') && !p.sentence.contains('{')), "{code_of}: {:?}", r.parts);
        assert!(r.parts[..3].iter().all(|p| !p.sentence.is_empty()), "{code_of}: {:?}", r.parts);
    }
}

#[test]
fn a_part_after_a_clip_keeps_its_group_apart() {
    let code = r"\begin{tikzpicture}\begin{scope}[opacity=.5,transparency group]\draw (0,0) -- (1,1);\draw (1,0) -- (0,1);\end{scope}\draw (2,0) -- (3,0);\end{tikzpicture}";
    let svg = tikra::convert_with(code, &tikra::Options::default().parts()).unwrap().svg;
    assert_eq!(svg.matches("<g").count(), svg.matches("</g>").count(), "{svg}");
}

#[test]
fn an_addplot_is_a_part_of_its_own() {
    let code = "\\begin{tikzpicture}\n\\begin{axis}[xlabel=time]\n\\addplot coordinates {(0,0) (1,1)};\n\\end{axis}\n\\end{tikzpicture}";
    let r = tikra::convert_with(code, &tikra::Options::default().parts()).unwrap();
    assert!(r.parts.iter().any(|p| p.source.starts_with("2:")), "{:?}", r.parts);
    assert!(r.parts.iter().any(|p| p.source.starts_with("3:") && p.text.contains("Line")), "{:?}", r.parts);
}

mod overlays {
    use tikra::{click_through, convert, slides, Options};

    fn lines(svg: &str) -> usize {
        svg.split("</metadata>").nth(1).unwrap().matches("<path").count()
    }

    fn drawn(code: &str) -> Vec<usize> {
        slides(code, &Options::default()).unwrap().iter().map(|s| lines(&s.svg)).collect()
    }

    fn picture(body: &str) -> String {
        format!(r"\begin{{tikzpicture}}\draw (0,0) -- (1,0);{body}\end{{tikzpicture}}")
    }

    #[test]
    fn only_and_its_slides() {
        assert_eq!(drawn(&picture(r"\only<2->{\draw (0,1) -- (1,1);}\only<3>{\draw (0,2) -- (1,2);}")), [1, 2, 3]);
    }

    #[test]
    fn a_draw_can_take_them_itself() {
        assert_eq!(drawn(&picture(r"\draw<2> (0,1) -- (1,1); \node<3-> at (0,2) {};\fill<3> (0,0) circle (1pt);")), [1, 2, 2]);
    }

    #[test]
    fn uncover_keeps_its_room_and_only_does_not() {
        let size = |o: &tikra::Output| o.svg.split("viewBox=\"").nth(1).unwrap().split('"').next().unwrap().to_string();
        let kept = slides(&picture(r"\uncover<2>{\draw (0,3) -- (1,3);}"), &Options::default()).unwrap();
        assert_eq!(size(&kept[0]), size(&kept[1]));
        assert!(kept[0].svg.contains(r#"<g opacity="0">"#) && !kept[1].svg.contains(r#"<g opacity="0">"#));
        let gone = slides(&picture(r"\only<2>{\draw (0,3) -- (1,3);}"), &Options::default()).unwrap();
        assert_ne!(size(&gone[0]), size(&gone[1]));
    }

    #[test]
    fn pause_hides_what_follows_until_its_slide() {
        let s = slides(&picture(r"\pause\draw (0,1) -- (1,1);\pause\draw (0,2) -- (1,2);"), &Options::default()).unwrap();
        assert_eq!(s.len(), 3);
        assert_eq!(s[0].svg.matches(r#"<g opacity="0">"#).count(), 2);
        assert_eq!(s[1].svg.matches(r#"<g opacity="0">"#).count(), 1);
        assert_eq!(s[2].svg.matches(r#"<g opacity="0">"#).count(), 0);
    }

    #[test]
    fn a_loop_gives_each_slide_its_own() {
        let code = picture(r"\foreach \i in {1,2,3} { \only<\i>{\draw (0,\i) -- (1,\i);} }");
        assert_eq!(drawn(&code), [2, 2, 2]);
    }

    #[test]
    fn the_counter_goes_on_with_plus() {
        assert_eq!(drawn(&picture(r"\only<+->{\draw (0,1) -- (1,1);}\only<+->{\draw (0,2) -- (1,2);}")), [2, 3]);
    }

    #[test]
    fn alt_visible_invisible_and_onslide() {
        assert_eq!(drawn(&picture(r"\alt<2>{\draw (0,1) -- (1,1); \draw (0,2) -- (1,2);}{\draw (0,3) -- (1,3);}")), [2, 3]);
        let s = slides(&picture(r"\invisible<1>{\draw (0,1) -- (1,1);}\visible<3>{\draw (0,2) -- (1,2);}"), &Options::default()).unwrap();
        assert_eq!(s.iter().map(|o| o.svg.matches(r#"<g opacity="0">"#).count()).collect::<Vec<_>>(), [2, 1, 0]);
        let bare = slides(&picture(r"\onslide<2->\draw (0,1) -- (1,1);\onslide\draw (0,2) -- (1,2);"), &Options::default()).unwrap();
        assert_eq!(bare[0].svg.matches(r#"<g opacity="0">"#).count(), 1);
        assert_eq!(bare[1].svg.matches(r#"<g opacity="0">"#).count(), 0);
    }

    #[test]
    fn their_names_are_free_for_the_code() {
        // `\alt` is not defined as LaTeX sees it, so `\providecommand` defines it
        let code = r"\providecommand{\alt}{2}\begin{tikzpicture}\draw (0,0) -- (\alt,1);\end{tikzpicture}";
        let r = convert(code).unwrap();
        assert!(r.warnings.is_empty() && lines(&r.svg) == 1, "{:?}", r.warnings);
        let code = r"\ifdefined\pause\def\w{1}\else\def\w{2}\fi\begin{tikzpicture}\draw (0,0) -- (\w,1);\end{tikzpicture}";
        assert!(convert(code).unwrap().svg.contains("56.892"));
        assert_eq!(drawn(&picture(r"\newcommand{\only}[1]{#1}\only{\draw (0,1) -- (1,1);}")), [2]);
    }

    #[test]
    fn a_figure_without_overlays_is_one_slide_as_ever() {
        let code = picture("");
        assert_eq!(slides(&code, &Options::default()).unwrap()[0].svg, convert(&code).unwrap().svg);
    }

    #[test]
    fn a_plain_conversion_is_the_last_slide() {
        let code = picture(r"\only<2>{\draw (0,1) -- (1,1);}");
        assert_eq!(convert(&code).unwrap().svg, slides(&code, &Options::default()).unwrap()[1].svg);
    }

    #[test]
    fn one_svg_that_advances_on_a_click() {
        let s = slides(&picture(r"\only<2>{\draw[fill=red] (-2,1) rectangle (1,1.5);}"), &Options::default()).unwrap();
        let svg = click_through(&s).unwrap();
        assert!(svg.contains(r#"data-slides="2""#) && svg.contains(r#"begin="trNext1.click""#) && svg.contains(r#"id="tr-slide-2""#));
        // a PNG shows the first slide
        assert!(tikra::png(&svg, 72.0, true).is_ok());
    }
}

mod played {
    use tikra::{convert, convert_with, Options};

    const FIGURE: &str = "\\usetikzlibrary{animations}\\begin{tikzpicture}\\fill[red] :fill = {0s=\"red\", 2s=\"blue\", freeze} :xshift = {1s=\"0cm\", 3s=\"2cm\", repeats=2} (0,0) circle (1pt);\\draw (0,0) -- (1,0) :fill = {0s=\"red\", 2s=\"blue\"} :rotate = {0s=\"0\", 1s=\"90\", begin on=click};\\end{tikzpicture}";

    #[test]
    fn the_keyframes_reach_the_svg() {
        let svg = convert_with(FIGURE, &Options::default().play()).unwrap().svg;
        assert!(svg.contains(r##"<animate attributeName="fill" values="#ff0000;#0000ff" keyTimes="0;1" dur="2s" begin="0s" fill="freeze"/>"##), "{svg}");
        assert!(svg.contains(r#"type="translate" values="0 0;56.69291338582677 0" keyTimes="0;1" dur="2s" begin="1s" repeatCount="2" fill="remove""#), "{svg}");
        assert!(svg.contains(r#"type="rotate" values="-0;-90" keyTimes="0;1" dur="1s" begin="trAnim2.click+0s""#), "{svg}");
        assert_eq!(svg.matches("<g").count(), svg.matches("</g>").count());
    }

    #[test]
    fn a_fill_is_not_animated_on_what_has_none() {
        let svg = convert_with(FIGURE, &Options::default().play()).unwrap().svg;
        assert_eq!(svg.matches(r#"attributeName="fill""#).count(), 1, "{svg}");
    }

    #[test]
    fn without_the_option_the_figure_is_still() {
        assert!(!convert(FIGURE).unwrap().svg.contains("animate"));
    }

    #[test]
    fn a_png_of_a_played_figure_is_the_still_one() {
        let code = "\\usetikzlibrary{animations}\\begin{tikzpicture}\\fill[red] :fill = {0s=\"red\", 2s=\"blue\"} (0,0) circle (1pt);\\draw (0,0) -- (1,0) :opacity = {0s=\"1\", 1s=\"0\"};\\end{tikzpicture}";
        let (played, still) = (convert_with(code, &Options::default().play()).unwrap().svg, convert(code).unwrap().svg);
        assert_eq!(tikra::png(&played, 72.0, true).unwrap(), tikra::png(&still, 72.0, true).unwrap());
    }

    #[test]
    fn a_turn_is_around_the_object_wherever_the_figure_lands() {
        let code = "\\usetikzlibrary{animations}\\begin{tikzpicture}\\draw (0,0) -- (1,1) :rotate = {0s=\"0\", 1s=\"90\"};\\end{tikzpicture}";
        let svg = convert_with(code, &Options::default().play()).unwrap().svg;
        assert!(!svg.contains("translate(--") && !svg.contains("translate(- "), "{svg}");
        let inner = svg.split(r#"<g transform="translate("#).nth(2).unwrap().split(')').next().unwrap();
        assert!(inner.split(' ').all(|v| v.parse::<f64>().is_ok()), "{inner}");
    }

    #[test]
    fn a_scope_moves_all_it_draws() {
        let code = "\\usetikzlibrary{animations}\\begin{tikzpicture}\\begin{scope}[animate={myself:opacity={0s=\"1\",1s=\"0\"}}]\\draw (0,0) -- (1,0);\\draw (0,1) -- (1,1);\\end{scope}\\draw (0,2) -- (1,2);\\end{tikzpicture}";
        let r = convert_with(code, &Options::default().play()).unwrap();
        assert_eq!(r.svg.matches(r#"attributeName="opacity""#).count(), 2, "{}", r.svg);
    }

    #[test]
    fn another_objects_animation_says_it_is_not_played() {
        let code = "\\usetikzlibrary{animations}\\begin{tikzpicture}\\draw (0,0) -- (1,0) [animate={other:fill={0s=\"red\",1s=\"blue\"}}];\\end{tikzpicture}";
        let r = convert_with(code, &Options::default().play()).unwrap();
        assert!(r.warnings.iter().any(|w| w.contains("other")), "{:?}", r.warnings);
    }
}

mod points {
    use tikra::{convert, convert_with, Options};

    const BARS: &str = "\\begin{tikzpicture}\\begin{axis}[ybar]\\addplot coordinates {(1,2) (2,5) (3,-3)};\\end{axis}\\end{tikzpicture}";
    const MARKS: &str = "\\begin{tikzpicture}\\begin{axis}\\addplot[mark=*] coordinates {(1,2.5) (2,5)};\\end{axis}\\end{tikzpicture}";

    #[test]
    fn each_bar_is_its_own_with_its_values() {
        let svg = convert_with(BARS, &Options::default().points()).unwrap().svg;
        assert_eq!(svg.matches(r#"class="tr-point""#).count(), 3, "{svg}");
        assert!(svg.contains(r#"tabindex="0" aria-label="2, 5" data-x="2" data-y="5""#), "{svg}");
        assert!(svg.contains(r#"data-x="3" data-y="-3""#));
        assert!(svg.contains(r#"<text class="tr-tip""#) && svg.contains(".tr-point:focus .tr-tip"));
        assert_eq!(svg.matches("<g").count(), svg.matches("</g>").count());
    }

    #[test]
    fn each_mark_is_its_own_with_its_values() {
        let svg = convert_with(MARKS, &Options::default().points()).unwrap().svg;
        assert!(svg.contains(r#"data-x="1" data-y="2.5""#) && svg.contains(r#"data-x="2" data-y="5""#), "{svg}");
        assert_eq!(svg.matches("<g").count(), svg.matches("</g>").count());
    }

    #[test]
    fn with_the_animation_bars_grow_from_their_base() {
        let svg = convert_with(BARS, &Options::default().points().animated()).unwrap().svg;
        assert!(svg.contains(r#"class="tr-up""#) && svg.contains(r#"class="tr-down""#) && svg.contains("@keyframes tr-gy"), "{svg}");
        assert_eq!(svg.matches(r#"class="tr-up""#).count(), 2);
    }

    #[test]
    fn the_bars_are_drawn_the_same() {
        let (points, plain) = (convert_with(BARS, &Options::default().points()).unwrap().svg, convert(BARS).unwrap().svg);
        assert!(!plain.contains("tr-point"));
        assert_eq!(tikra::png(&points, 96.0, true).unwrap(), tikra::png(&plain, 96.0, true).unwrap());
    }
}

#[cfg(feature = "motion")]
mod motion {
    use tikra::motion::{apng, gif, Motion};

    const FIGURE: &str = r"\begin{tikzpicture} \draw[->] (0,0) -- (1,1); \draw[dashed] (0,1) -- (1,0); \fill[red] (2,0) circle (2pt); \node at (3,0) {$x$}; \end{tikzpicture}";

    fn animated() -> String {
        tikra::convert_with(FIGURE, &tikra::Options::default().animated()).unwrap().svg
    }

    #[test]
    fn a_gif_is_a_looping_gif_of_the_size_asked_for() {
        let bytes = gif(&animated(), Motion { width: 120, fps: 10 }).unwrap();
        assert!(bytes.starts_with(b"GIF89a") && bytes.ends_with(&[0x3b]));
        assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), 120);
        assert!(bytes.windows(11).any(|w| w == b"NETSCAPE2.0"));
    }

    #[test]
    fn an_apng_has_its_frames_and_their_control_chunks() {
        let bytes = apng(&animated(), Motion { width: 120, fps: 10 }).unwrap();
        assert!(bytes.starts_with(b"\x89PNG"));
        let at = bytes.windows(4).position(|w| w == b"acTL").unwrap();
        let frames = u32::from_be_bytes(bytes[at + 4..at + 8].try_into().unwrap());
        assert!(frames > 2);
        assert_eq!(bytes.windows(4).filter(|w| *w == b"fcTL").count(), frames as usize);
    }

    #[test]
    fn the_same_figure_gives_the_same_bytes() {
        let m = Motion { width: 120, fps: 10 };
        assert_eq!(gif(&animated(), m).unwrap(), gif(&animated(), m).unwrap());
        assert_eq!(apng(&animated(), m).unwrap(), apng(&animated(), m).unwrap());
    }

    #[test]
    fn a_still_svg_is_refused_with_a_reason() {
        let still = tikra::convert(FIGURE).unwrap().svg;
        assert!(gif(&still, Motion::default()).unwrap_err().to_string().contains("animated"));
    }
}
