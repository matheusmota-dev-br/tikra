//! Every figure of the benchmark corpus (`corpus/*.tex`) converts, without warnings, to SVG and PNG.

#[test]
fn every_corpus_figure_converts_without_warnings() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus");
    let mut files: Vec<_> = std::fs::read_dir(dir).unwrap().filter_map(Result::ok).map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "tex")).collect();
    files.sort();
    assert!(files.len() >= 20, "{} figures", files.len());
    for f in files {
        let tex = std::fs::read_to_string(&f).unwrap();
        let out = tikra::convert(&tex).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
        assert!(out.warnings.is_empty(), "{}: {:?}", f.display(), out.warnings);
        assert!(!out.svg.contains("NaN") && !out.svg.contains("inf"), "{}", f.display());
        tikra::png(&out.svg, 96.0, true).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
    }
}
