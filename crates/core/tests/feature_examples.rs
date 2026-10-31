//! `examples/`: each one converts without warnings, and does what it is there to show.

use std::path::{Path, PathBuf};

use tikra::{convert_with, Options};

fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

fn example(name: &str) -> String {
    std::fs::read_to_string(dir().join(format!("{name}.tex"))).unwrap()
}

#[test]
fn every_example_converts_without_warnings() {
    for e in std::fs::read_dir(dir()).unwrap().filter_map(Result::ok) {
        let path = e.path();
        if path.extension().is_some_and(|x| x == "tex") {
            let out = tikra::convert(&std::fs::read_to_string(&path).unwrap()).unwrap();
            assert!(out.warnings.is_empty(), "{}: {:?}", path.display(), out.warnings);
        }
    }
}

#[test]
fn the_source_map_example_has_each_plot_on_its_line() {
    let svg = convert_with(&example("source-map"), &Options::default().source()).unwrap().svg;
    assert!(svg.contains(r#"data-src="21:5-"#) && svg.contains(r#"data-src="22:5-"#), "{svg}");
}

#[test]
fn the_walk_example_tells_its_connections() {
    let out = convert_with(&example("walk"), &Options::default().parts()).unwrap();
    assert!(out.parts.iter().any(|p| p.text == "from Idea to Draft"), "{:?}", out.parts);
}

#[test]
fn the_slides_example_has_four() {
    assert_eq!(tikra::slides(&example("slides"), &Options::default()).unwrap().len(), 4);
}
