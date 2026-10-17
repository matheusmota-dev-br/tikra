//! The pgfplots subset: windows, axes, series by expression and by coordinates, legend.

mod common;
use common::*;
use tikra::scene::Element;

fn plot(options: &str, body: &str) -> tikra::Output {
    figure(&format!(r"\begin{{tikzpicture}}\begin{{axis}}[{options}] {body} \end{{axis}}\end{{tikzpicture}}"))
}

#[test]
fn axis_styles() {
    for axes in ["axis lines=left", "axis lines=middle", "axis lines=center", "axis lines=none", "axis lines=box", ""] {
        let r = plot(&format!("xmin=-1, xmax=1, ymin=-1, ymax=1, {axes}"), r"\addplot {x};");
        assert!(r.svg.contains("<path"), "{axes}");
        assert!(r.warnings.is_empty(), "{axes}: {:?}", r.warnings);
    }
}

#[test]
fn window_from_the_data() {
    let r = plot("", r"\addplot coordinates {(0,0) (10,100) (zz,1) (3)};");
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    // without data: a window from 0 to 1, with a warning
    let r = plot("", r"\addlegendentry{nothing}");
    assert!(r.warnings.iter().any(|a| a.contains("without data")));
    // xmin = xmax: default window
    plot("xmin=2, xmax=2, ymin=1, ymax=1", r"\addplot coordinates {(2,1)};");
}

#[test]
fn expression_series_with_domain_and_samples() {
    let r = plot("domain=0:2, samples=5, width=8cm, height=5cm, xlabel={$x$}, ylabel={$y$}, title={Title}", r"\addplot[red, thick] {x^2}; \addplot+[domain=-1:1, samples=3] {1/x}; \addplot[samples=zz] {sqrt(x)}; \addplot {\doesnotexist};");
    assert!(r.svg.contains("#ff0000"));
    // an invalid domain on the series or the axis: -5 to 5
    plot("domain=zz:1", r"\addplot[domain=1:zz] {x};");
}

#[test]
fn marks_on_coordinate_series() {
    let c = scene(r"\begin{axis}[xmin=0, xmax=2, ymin=0, ymax=2] \addplot coordinates {(1,1) (2,2)}; \addplot+[red] coordinates {(0,1)}; \addplot[blue] coordinates {(1,0) (0,1)}; \end{axis}");
    // marked points: the first two series (no options, and with +)
    let marks = nodes(&c).iter().filter(|n| n.label.is_none()).count();
    assert_eq!(marks, 3, "{marks}");
    assert!(matches!(c.elements.last(), Some(Element::EndClip)));
}

#[test]
fn positioned_legend_and_frame() {
    let r = plot("xmin=0, xmax=1, ymin=0, ymax=1, legend style={at={(0.5,0.9)}, anchor=north}", r"\addplot {x}; \addlegendentry{line} \addlegendentry{no series}");
    assert!(r.svg.contains("#ffffff"), "the legend's white frame");
    // an invalid at: default corner
    plot("xmin=0, xmax=1, ymin=0, ymax=1, legend style={at={(zz)}}", r"\addplot {x}; \addlegendentry{a}");
    plot(r"xmin=0, xmax=1, ymin=0, ymax=1, legend style={font=\small}", r"\addplot {x}; \addlegendentry{a}");
}

#[test]
fn log_variants_are_read_as_linear() {
    for env in ["semilogyaxis", "semilogxaxis", "loglogaxis"] {
        let r = figure(&format!(r"\begin{{tikzpicture}}\begin{{{env}}}[xmin=1, xmax=10, ymin=1, ymax=10] \addplot coordinates {{(1,1) (10,10)}}; \end{{{env}}}\end{{tikzpicture}}"));
        assert!(r.svg.contains("<path"), "{env}");
    }
}

#[test]
fn axis_without_end_reads_to_the_end() {
    let c = scene(r"\begin{axis}[xmin=0, xmax=1, ymin=0, ymax=1] \addplot coordinates {(0,0) (1,1)};");
    assert!(!paths(&c).is_empty());
}

#[test]
fn addplot_errors() {
    let mut env = tikra::reader::Env::new();
    assert!(tikra::axes::plot_axis(&mut env, "", r"\addplot coordinates").is_err());
    // a series with neither expression nor coordinates: empty
    let mut env = tikra::reader::Env::new();
    tikra::axes::plot_axis(&mut env, "xmin=0, xmax=1, ymin=0, ymax=1", r"\addplot table {data.csv};").unwrap();
}

#[test]
fn huge_or_tiny_window_does_not_hang() {
    // the step vanishes in precision: at most 50 ticks
    plot("xmin=1e17, xmax=1.0000000000000001e17, ymin=0, ymax=1", r"\addplot coordinates {(1e17,0)};");
    plot("xmin=0, xmax=1e-300, ymin=0, ymax=1", "");
    plot("ymin=-1e308, ymax=1e308", "");
}

#[test]
fn legend_with_space_before_the_brace() {
    plot("xmin=0, xmax=1, ymin=0, ymax=1", r"\addplot {x}; \addlegendentry  {with a space}");
}

#[test]
fn incomplete_domains() {
    for o in ["domain=1", "domain=0:zz", "domain=zz:1"] {
        plot(o, &format!(r"\addplot[{o}] {{x}};"));
    }
}

#[test]
fn only_marks_draws_points_without_a_line() {
    let c = scene(r"\begin{axis}[xmin=0, xmax=2, ymin=0, ymax=2] \addplot[only marks] coordinates {(0.5,0.5) (1,1) (1.5,1.5)}; \addlegendentry{points} \end{axis}");
    // three marks plus the one in the legend, and no line joining them
    let marks = nodes(&c).iter().filter(|n| n.label.is_none()).count();
    assert_eq!(marks, 4, "{marks}");
    assert!(paths(&c).iter().all(|p| p.segments.len() != 3), "no polyline through the 3 points");
    // `mark=*` adds marks to a styled line; `no marks` removes them
    let with = scene(r"\begin{axis} \addplot[red, mark=*] coordinates {(0,0) (1,1)}; \end{axis}");
    assert_eq!(nodes(&with).iter().filter(|n| n.label.is_none()).count(), 2);
    let without = scene(r"\begin{axis} \addplot+[no marks] coordinates {(0,0) (1,1)}; \end{axis}");
    assert_eq!(nodes(&without).iter().filter(|n| n.label.is_none()).count(), 0);
}
