//! Path options, xcolor colors and user styles (`\tikzset`, `\tikzstyle`).

mod common;
use common::*;
use std::collections::HashMap;
use tikra::color::{self, Color};
use tikra::scene::Tip;

#[test]
fn dashes_widths_opacities() {
    for (op, n) in [("dashed", 2), ("densely dashed", 2), ("loosely dashed", 2), ("dotted", 2), ("densely dotted", 2), ("loosely dotted", 2), ("dash dot", 4), ("dash pattern=on 2pt off 1pt on 1pt off 1pt", 4)] {
        let t = first_path(&format!(r"\draw[{op}] (0,0) -- (1,0);"));
        assert_eq!(t.style.dash.as_ref().map(Vec::len), Some(n), "{op}");
    }
    assert!(first_path(r"\draw[dashed, solid] (0,0) -- (1,0);").style.dash.is_none());
    assert!(first_path(r"\draw[dash pattern=on zz] (0,0) -- (1,0);").style.dash.is_none());
    for (op, w) in [("ultra thin", 0.1), ("very thin", 0.2), ("thin", 0.4), ("semithick", 0.6), ("thick", 0.8), ("very thick", 1.2), ("ultra thick", 1.6), ("line width=2pt", 2.0)] {
        let t = first_path(&format!(r"\draw[{op}] (0,0) -- (1,0);"));
        assert!(close(t.style.line_width, w * tikra::PT), "{op}");
    }
    let t = first_path(r"\filldraw[opacity=0.5] (0,0) rectangle (1,1);");
    assert!(close(t.style.stroke_opacity, 0.5) && close(t.style.fill_opacity, 0.5));
    let t = first_path(r"\filldraw[draw opacity=0.2, fill opacity=0.3] (0,0) rectangle (1,1);");
    assert!(close(t.style.stroke_opacity, 0.2) && close(t.style.fill_opacity, 0.3));
    let t = first_path(r"\filldraw[opacity=zz, draw opacity=zz, fill opacity=zz, line width=zz] (0,0) rectangle (1,1);");
    assert!(close(t.style.stroke_opacity, 1.0));
}

#[test]
fn stroke_and_fill_colors() {
    let t = first_path(r"\filldraw[draw=red, fill=blue!20] (0,0) rectangle (1,1);");
    assert_eq!(t.style.stroke.unwrap().hex(), "#ff0000");
    assert_eq!(t.style.fill.unwrap().hex(), "#ccccff");
    let t = first_path(r"\draw[fill] (0,0) rectangle (1,1);");
    assert!(t.style.fill.is_some());
    let t = first_path(r"\filldraw[fill=none, color=green] (0,0) rectangle (1,1);");
    assert!(t.style.fill.is_none());
    assert_eq!(t.style.stroke.unwrap().hex(), "#00ff00");
    let t = first_path(r"\fill[draw] (0,0) rectangle (1,1);");
    assert!(t.style.stroke.is_some());
    // color= with an unknown name changes nothing
    let t = first_path(r"\draw[color=noname] (0,0) -- (1,0);");
    assert_eq!(t.style.stroke.unwrap().hex(), "#000000");
    // shade and shadedraw fill
    assert!(first_path(r"\shade (0,0) rectangle (1,1);").style.fill.is_some());
    assert!(first_path(r"\shadedraw (0,0) rectangle (1,1);").style.stroke.is_some());
}

#[test]
fn hatchings() {
    for p in ["north east lines", "north west lines", "horizontal lines", "vertical lines", "grid", "crosshatch", "dots", "crosshatch dots", "fivepointed stars", "sixpointed stars", "bricks"] {
        let r = no_warnings(&format!(r"\begin{{tikzpicture}} \draw[pattern={p}, pattern color=red] (0,0) rectangle (1,1); \end{{tikzpicture}}"));
        assert!(r.svg.contains("<pattern"), "{p}");
    }
    let t = first_path(r"\path[pattern=dots, pattern color=nothing] (0,0) rectangle (1,1);");
    assert_eq!(t.style.pattern.unwrap().1.hex(), "#000000");
}

#[test]
fn arrow_tips() {
    for (op, start, end) in [
        ("->", None, Some(Tip::Default)),
        ("<-", Some(Tip::Default), None),
        ("<->", Some(Tip::Default), Some(Tip::Default)),
        ("-latex", None, Some(Tip::Latex)),
        ("-Latex", None, Some(Tip::Latex)),
        ("-{Latex[length=3mm]}", None, Some(Tip::Latex)),
        ("-Triangle", None, Some(Tip::Latex)),
        ("-stealth", None, Some(Tip::Stealth)),
        ("-Kite", None, Some(Tip::Stealth)),
        ("|-|", Some(Tip::Bar), Some(Tip::Bar)),
        ("o-*", Some(Tip::Circle), Some(Tip::Circle)),
        ("-to", None, Some(Tip::Default)),
        ("Bar-Circle", Some(Tip::Bar), Some(Tip::Circle)),
    ] {
        let t = first_path(&format!(r"\draw[{op}] (0,0) -- (1,0);"));
        assert_eq!((t.style.start_tip, t.style.end_tip), (start, end), "{op}");
    }
    // >=Latex changes what '>' means
    let c = scene(r"\begin{scope}[>=Stealth] \draw[->] (0,0) -- (1,0); \end{scope}");
    assert_eq!(paths(&c)[0].style.end_tip, Some(Tip::Stealth));
    // an unknown tip and a lone '-': not an arrow
    assert_eq!(first_path(r"\draw[-Nothing] (0,0) -- (1,0);").style.end_tip, None);
    assert_eq!(first_path(r"\draw[-] (0,0) -- (1,0);").style.end_tip, None);
    assert_eq!(first_path(r"\draw[Nothing-] (0,0) -- (1,0);").style.end_tip, None);
    assert!(tikra::path::tip("{Stealth[length=2mm]}").is_some());
    assert!(tikra::path::tip("??").is_none());
    // >=nothing: back to the default
    let c = scene(r"\begin{scope}[>=Nothing] \draw[->] (0,0) -- (1,0); \end{scope}");
    assert_eq!(paths(&c)[0].style.end_tip, Some(Tip::Default));
}

#[test]
fn every_tip_in_the_svg() {
    for op in ["->", "<-", "-latex", "latex-", "-stealth", "stealth-", "|-|", "o-*"] {
        let r = no_warnings(&format!(r"\begin{{tikzpicture}} \draw[{op}] (0,0) -- (1,0); \draw[{op}] (0,1) .. controls (1,2) and (2,2) .. (3,1); \draw[{op}] (0,2) .. controls (0,2) and (0,2) .. (1,3); \end{{tikzpicture}}"));
        assert!(r.svg.contains("<poly") || r.svg.contains("<circle"), "{op}");
    }
    // an arrow on a closed path: no tip
    no_warnings(r"\begin{tikzpicture} \draw[->] (0,0) -- (1,0) -- (1,1) -- cycle; \draw[<-] (0,0) circle (1); \end{tikzpicture}");
    // a curve whose second control point is its end
    no_warnings(r"\begin{tikzpicture} \draw[-latex] (0,0) .. controls (1,1) and (2,0) .. (2,0); \draw[latex-] (0,0) .. controls (0,0) and (1,1) .. (2,0); \end{tikzpicture}");
}

#[test]
fn xcolor_parsing() {
    let d = HashMap::new();
    assert_eq!(color::parse("-red", &d).unwrap().hex(), "#00ffff");
    assert_eq!(color::parse("--red", &d).unwrap().hex(), "#ff0000");
    assert!(color::parse("", &d).is_none());
    assert!(color::parse("-", &d).is_none());
    assert!(color::parse("red!xx", &d).is_none());
    assert!(color::parse("red!50!nothing", &d).is_none());
    for name in ["black", "white", "green", "cyan", "magenta", "yellow", "gray", "darkgray", "lightgray", "brown", "lime", "olive", "orange", "pink", "purple", "teal", "violet", "ForestGreen", "NavyBlue", "RoyalBlue", "Maroon", "Goldenrod", "Gray"] {
        assert!(color::parse(name, &d).is_some(), "{name}");
    }
    assert_eq!(Color { r: 2.0, g: -1.0, b: 0.5 }.hex(), "#ff0080", "values outside 0..1 are clamped");
}

#[test]
fn color_definitions() {
    assert_eq!(color::define("rgb", "1,0.5,0").unwrap().hex(), "#ff8000");
    assert_eq!(color::define("RGB", "255,128,0").unwrap().hex(), "#ff8000");
    assert_eq!(color::define("gray", "0.5").unwrap().hex(), "#808080");
    assert_eq!(color::define("cmyk", "0,1,1,0").unwrap().hex(), "#ff0000");
    assert_eq!(color::define("HTML", "#00FF00").unwrap().hex(), "#00ff00");
    assert!(color::define("HTML", "12").is_none());
    assert!(color::define("HTML", "ZZZZZZ").is_none());
    assert!(color::define("rgb", "1,2").is_none());
    assert!(color::define("hsb", "1,2,3").is_none());
}

#[test]
fn definecolor_and_colorlet_in_the_reader() {
    let c = scene(
        r"\definecolor{a}{HTML}{112233} \providecolor{b}{rgb}{0,0,1} \colorlet{c}{a!50!b} \colorlet{d}{missing}
          \draw[c] (0,0) -- (1,0);",
    );
    assert_eq!(paths(&c)[0].style.stroke.unwrap().hex(), "#091199");
    let (_, w) = scene_and_warnings(r"\definecolor{x}{hsb}{1,1,1}");
    assert!(w.iter().any(|a| a.contains("model not supported")));
}

#[test]
fn user_styles() {
    let c = scene(
        r"\tikzset{base/.style={thick}, base/.append style={red}, fresh/.append style={dashed}, every path/.style={blue}}
          \tikzstyle{box}=[draw, fill=yellow]
          \tikzstyle{loose}[circle]
          \draw[base] (0,0) -- (1,0);
          \draw[fresh] (0,0) -- (1,0);
          \draw (0,0) -- (1,0);
          \node[box] {x};",
    );
    let t = paths(&c);
    assert_eq!(t[0].style.stroke.unwrap().hex(), "#ff0000");
    assert!(close(t[0].style.line_width, 0.8 * tikra::PT));
    assert!(t[1].style.dash.is_some());
    assert_eq!(t[2].style.stroke.unwrap().hex(), "#0000ff", "every path");
    assert!(nodes(&c)[0].style.fill.is_some());
}

#[test]
fn every_node_applies_to_all() {
    let c = scene(r"\tikzset{every node/.style={draw, circle}} \node {a}; \node at (1,0) {b};");
    assert!(nodes(&c).iter().all(|n| n.style.stroke.is_some()));
}

#[test]
fn option_lists_and_braces() {
    use tikra::options::{parse, split_top_level, strip_braces, Styles};
    assert_eq!(split_top_level("a, {b, c}, (d, e), [f, g], ,"), ["a", "{b, c}", "(d, e)", "[f, g]"]);
    assert_eq!(strip_braces("{a}"), "a");
    assert_eq!(strip_braces("{a}{b}"), "{a}{b}", "the opening brace closes before the end");
    assert_eq!(strip_braces("{"), "{");
    assert_eq!(strip_braces("x"), "x");
    assert_eq!(parse("=x")[0].key, "=x");
    let e = Styles::default();
    assert!(e.context("every node").is_empty());
}

#[test]
fn html_too_short() {
    assert!(color::define("HTML", "1122").is_none());
}

#[test]
fn pattern_replaces_the_solid_fill() {
    let t = first_path(r"\fill[pattern=north east lines] (0,0) rectangle (1,1);");
    assert!(t.style.pattern.is_some() && t.style.fill.is_none(), "{:?}", t.style);
    // an explicit fill color stays under the pattern
    let t = first_path(r"\filldraw[fill=yellow, pattern=dots] (0,0) rectangle (1,1);");
    assert!(t.style.pattern.is_some() && t.style.fill.is_some());
}

#[test]
fn double_tip() {
    let t = first_path(r"\draw[->>] (0,0) -- (1,0);");
    assert_eq!(t.style.end_tip, Some(tikra::scene::Tip::Double));
    let t = first_path(r"\draw[<<-] (0,0) -- (1,0);");
    assert_eq!(t.style.start_tip, Some(tikra::scene::Tip::Double));
    let r = no_warnings(r"\begin{tikzpicture}\draw[->>] (0,0) -- (1,0);\end{tikzpicture}");
    assert_eq!(r.svg.matches("<polyline").count(), 2, "two chevrons");
}
