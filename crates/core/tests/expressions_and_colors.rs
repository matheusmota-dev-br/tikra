//! pgfmath's less common operators and functions, its strings, xcolor's color expressions and
//! models, ColorBrewer's schemes, and the words a figure's text is spoken with.

use std::collections::HashMap;

use tikra::color::{self, Color};
use tikra::expr::{eval, eval_length, eval_string, Variables};

fn value(text: &str) -> f64 {
    eval(text, &Variables::new()).unwrap_or_else(|e| panic!("{text}: {e}"))
}

fn error(text: &str) -> String {
    match eval(text, &Variables::new()) {
        Ok(v) => panic!("{text} = {v}, expected an error"),
        Err(e) => e.to_string(),
    }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-4
}

// ---------------------------------------------------------------- operators

#[test]
fn logical_operators_and_signs() {
    for (text, v) in [("0 || 1", 1.0), ("0 || 0", 0.0), ("1 && 1", 1.0), ("1 && 0", 0.0), ("+3", 3.0), ("-+2", -2.0)] {
        assert_eq!(value(text), v, "{text}");
    }
    assert!(error("1 ? 2").contains("'?' without ':'"));
    assert!(error("{1,2}[0").contains("missing ']'"));
}

#[test]
fn sign_and_scalar() {
    assert_eq!((value("sign(-3)"), value("sign(0)"), value("sign(5)")), (-1.0, 0.0, 1.0));
    // scalar: a ratio of lengths has no unit
    let (v, unit) = eval_length("scalar(2pt/1pt)", &Variables::new()).unwrap();
    assert!(close(v, 2.0) && !unit, "{v} {unit}");
}

#[test]
fn radians_after_a_number() {
    // `r` after a number: radians, as degrees; past 285 the double's own conversion
    assert!(close(value("1r"), 57.29577));
    assert!(close(value("300r"), 300f64.to_degrees()));
}

#[test]
fn trigonometry_in_radians() {
    let mut v = Variables::new();
    v.trig_rad = true;
    let pi = std::f64::consts::PI;
    for (text, want) in [
        ("sin(pi/2)", 1.0),
        ("cos(pi)", -1.0),
        ("sec(0)", 1.0),
        ("cosec(pi/2)", 1.0),
        ("asin(1)", pi / 2.0),
        ("acos(0)", pi / 2.0),
        ("atan(1)", pi / 4.0),
        ("atan2(1,1)", pi / 4.0),
        ("exp(0)", 1.0),
    ] {
        let got = eval(text, &v).unwrap();
        assert!(close(got, want), "{text} = {got}, expected {want}");
    }
}

#[test]
fn a_unit_after_a_whole_expression() {
    // `(1+1) cm`: the unit is the whole expression's
    let (v, unit) = eval_length("(1+1) cm", &Variables::new()).unwrap();
    assert!(close(v, 2.0) && unit, "{v}");
}

// ---------------------------------------------------------------- strings

#[test]
fn strings_picked_by_conditions_in_parentheses() {
    let s = |t: &str| eval_string(t, &Variables::new()).unwrap();
    // parentheses inside quotes do not count, nor around a part only
    assert_eq!(s(r#"(1 ? "(x" : "y")"#), Some("(x".into()));
    assert_eq!(s(r#"((1) ? "a" : "b")"#), Some("a".into()));
    assert_eq!(s(r#"(0) ? "a" : (1 ? "c" : "d")"#), Some("c".into()));
    // an array element followed by more of the expression
    assert_eq!(s(r#"{1,0}[1] ? "yes" : "no""#), Some("no".into()));
}

// ---------------------------------------------------------------- colors

#[test]
fn hues_all_round_the_circle() {
    for (values, rgb) in [("0.25,1,1", (0.5, 1.0, 0.0)), ("0.4,1,1", (0.0, 1.0, 0.4)), ("0.75,1,1", (0.5, 0.0, 1.0)), ("0.9,1,1", (1.0, 0.0, 0.6))] {
        let c = color::define("hsb", values).unwrap();
        assert!(close(c.r, rgb.0) && close(c.g, rgb.1) && close(c.b, rgb.2), "{values}: {c:?}");
    }
}

#[test]
fn grays_complemented_and_mixed() {
    let mut defined = HashMap::new();
    defined.insert("g".to_string(), Color::gray(0.25));
    let c = color::parse("-g", &defined).unwrap();
    assert!(close(c.r, 0.75), "{c:?}");
    // in the gray model of the extended syntax, a gray is its own level
    let c = color::parse("gray,2:g,1;g,3", &defined).unwrap();
    assert!(close(c.r, 0.5), "{c:?}");
}

#[test]
fn brewer_colormaps_of_too_few_or_too_many_classes() {
    assert!(color::brewer::colormap("Accent-2").is_none());
    assert!(color::brewer::colormap("Accent-9").is_none());
    assert_eq!(color::brewer::colormap("Accent-3").unwrap().len(), 3);
}

// ---------------------------------------------------------------- spoken text

#[test]
fn commands_whose_argument_is_not_said() {
    let said = tikra::describe::spoken(r"see \ref x here", tikra::Locale::default());
    assert!(!said.contains("ref"), "{said}");
    assert!(said.contains("see") && said.contains("here"), "{said}");
}

#[test]
fn plus_signs_before_groups() {
    assert_eq!(value("+(3)"), 3.0);
    assert_eq!(value("2*+(3)"), 6.0);
    // an exponent with its sign
    assert_eq!((value("2^+3"), value("2^-1")), (8.0, 0.5));
}

#[test]
fn a_string_after_a_braced_group() {
    let s = |t: &str| eval_string(t, &Variables::new()).unwrap();
    // a group in braces that is not an array: the condition is a number
    assert_eq!(s(r#"{1}+1 > 1 ? "a" : "b""#), Some("a".into()));
}
