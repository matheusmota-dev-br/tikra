//! Every figure of the benchmark corpus (`corpus/<topic>/*.tex`) converts, without warnings, to SVG and PNG.

use std::path::{Path, PathBuf};

/// The `.tex` files under `dir`, in its folders too, sorted.
fn figures(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).unwrap().filter_map(Result::ok) {
        let p = e.path();
        if p.is_dir() {
            out.extend(figures(&p));
        } else if p.extension().is_some_and(|x| x == "tex") {
            out.push(p);
        }
    }
    out.sort();
    out
}

#[test]
fn every_corpus_figure_converts_without_warnings() {
    let files = figures(Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus")));
    assert!(files.len() >= 20, "{} figures", files.len());
    for f in files {
        let tex = std::fs::read_to_string(&f).unwrap();
        let out = tikra::convert(&tex).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
        assert!(out.warnings.is_empty(), "{}: {:?}", f.display(), out.warnings);
        // the drawing, not its words (the description says "rainfall") nor its metadata (which
        // holds the source: `info=` has an "inf")
        let drawing = match (out.svg.find("<title"), out.svg.find("</metadata>")) {
            (Some(a), Some(b)) => format!("{}{}", &out.svg[..a], &out.svg[b..]),
            _ => out.svg.clone(),
        };
        assert!(!drawing.contains("NaN") && !drawing.contains("inf"), "{}", f.display());
        tikra::png(&out.svg, 96.0, true).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
    }
}

/// The source map adds an attribute to each element and changes nothing else, whatever the figure
/// does with what it drew: layers, spies, fadings, pictures in nodes, plots.
#[test]
fn the_source_map_only_adds_its_attribute() {
    let files = figures(Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus")));
    for f in files {
        let tex = std::fs::read_to_string(&f).unwrap();
        let steps = tikra::convert_with(&tex, &tikra::Options::default().steps()).unwrap().svg;
        let mapped = tikra::convert_with(&tex, &tikra::Options::default().source()).unwrap().svg;
        let mut bare = mapped.clone();
        while let Some(i) = bare.find(r#" data-src=""#) {
            let end = i + 11 + bare[i + 11..].find('"').unwrap() + 1;
            bare.replace_range(i..end, "");
        }
        assert!(bare == steps, "{}: differs beyond data-src", f.display());
        // every step says where it came from
        assert_eq!(mapped.matches(r#" data-step=""#).count(), mapped.matches(r#" data-src=""#).count(), "{}", f.display());
    }
}

/// The parts are groups around what is drawn: the picture is the same, and every group is closed.
#[test]
fn the_parts_change_no_pixel() {
    let files = figures(Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus")));
    for f in files {
        let tex = std::fs::read_to_string(&f).unwrap();
        let plain = tikra::convert(&tex).unwrap().svg;
        let parts = tikra::convert_with(&tex, &tikra::Options::default().parts()).unwrap().svg;
        assert_eq!(parts.matches("<g").count(), parts.matches("</g>").count(), "{}", f.display());
        assert!(tikra::png(&plain, 40.0, true).unwrap() == tikra::png(&parts, 40.0, true).unwrap(), "{}", f.display());
    }
}
