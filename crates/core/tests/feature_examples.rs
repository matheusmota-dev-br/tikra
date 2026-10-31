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
    assert!(svg.contains(r#"data-src="20:5-"#) && svg.contains(r#"data-src="21:5-"#), "{svg}");
}
