//! Libraries from outside tikra: their keys and definitions, used through the public API only.

use tikra::extension::{Context, Error, Library, Tokens};
use tikra::Options;

/// `twice={options}`: the options, then `line width` doubled.
fn twice(cx: &mut Context, value: Option<Tokens>) -> Result<(), Error> {
    cx.keys(value.unwrap_or_default())?;
    let w = cx.line_width();
    cx.keys(Tokens::text(&format!("line width={}cm", 2.0 * w)))
}

/// `midpoint={A}{B}{name}`: a coordinate halfway between two others.
fn midpoint(cx: &mut Context, value: Option<Tokens>) -> Result<(), Error> {
    let a = value.unwrap_or_default().arguments(3);
    let (p, q, name) = (cx.expand(a[0].clone())?, cx.expand(a[1].clone())?, cx.expand(a[2].clone())?);
    let at = cx.point(&p)?.plus(cx.point(&q)?).scale(0.5);
    cx.run(Tokens::text(&format!("\\coordinate ({name}) at ({}cm,{}cm);", at.x, at.y)))
}

/// `complain`: a warning, and nothing drawn.
fn complain(cx: &mut Context, _value: Option<Tokens>) -> Result<(), Error> {
    cx.warn("complain: nothing to do");
    Ok(())
}

fn definitions() -> String {
    r"\tikzset{road/.style={twice={draw=gray}}}\def\tikramid{midpoint}".into()
}

static ROADS: Library = Library { name: "roads", definitions, keys: &[("/tikz/twice", twice), ("/tikz/midpoint", midpoint), ("/tikz/complain", complain)] };

fn convert(body: &str, libraries: &[&'static Library]) -> tikra::Output {
    tikra::convert_with_libraries(&format!("\\begin{{tikzpicture}}{body}\\end{{tikzpicture}}"), &Options::default(), libraries).unwrap()
}

#[test]
fn a_library_key_runs_and_its_styles_are_defined() {
    let with = convert(r"\draw[road] (0,0) -- (2,0);", &[&ROADS]);
    assert!(with.warnings.is_empty(), "{:?}", with.warnings);
    assert!(with.svg.contains("stroke-width=\"0.797\""), "{}", with.svg);
}

#[test]
fn without_the_library_its_keys_are_unknown() {
    let without = convert(r"\draw[road] (0,0) -- (2,0);", &[]);
    assert!(!without.warnings.is_empty());
    // and the library used before does not stay in the engine
    convert(r"\draw[road] (0,0) -- (2,0);", &[&ROADS]);
    assert!(!convert(r"\draw[twice] (0,0) -- (1,0);", &[]).warnings.is_empty());
}

#[test]
fn a_library_key_reads_points_and_runs_code() {
    let r = convert(r"\path[midpoint={0,0}{4,2}{M}]; \fill (M) circle (2pt);", &[&ROADS]);
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
}

#[test]
fn a_library_key_warns() {
    let r = convert(r"\draw[complain] (0,0) -- (1,0);", &[&ROADS]);
    assert!(r.warnings.iter().any(|w| w.contains("complain: nothing to do")), "{:?}", r.warnings);
}

#[test]
fn tokens_split_as_tex_does() {
    let a = Tokens::text("{a}{b c}").arguments(3);
    assert_eq!(a.iter().map(Tokens::to_text).collect::<Vec<_>>(), ["a", "b c", ""]);
    let (opts, rest) = Tokens::text(" [red, thick] $x$").leading_options();
    assert_eq!((opts.to_text().as_str(), rest.to_text().trim()), ("red, thick", "$x$"));
    let keys: Vec<String> = Tokens::text("pos=.3, near end").key_list().into_iter().map(|(k, _)| k).collect();
    assert_eq!(keys, ["pos", "near end"]);
}
