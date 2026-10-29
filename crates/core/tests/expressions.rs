//! pgfmath expressions through the public API: functions, constants, units and errors.

use tikra::expr::{eval, eval_length, eval_string, unit_cm, Variables};

fn close(text: &str, expected: f64) {
    let mut v = Variables::new();
    v.insert("x".into(), 2.0);
    let r = eval(text, &v).unwrap_or_else(|e| panic!("{text}: {e}"));
    assert!((r - expected).abs() < 1e-9, "{text} = {r}, expected {expected}");
}

#[test]
fn functions_constants_and_units() {
    for (t, e) in [
        ("cos(60)", 0.5),
        ("tan(45)", 1.0),
        ("asin(1)", 90.0),
        ("acos(0)", 90.0),
        ("atan(1)", 45.0),
        ("atan2(1,1)", 45.0),
        ("atan2(1)", 0.0),
        ("sqrt(9)", 3.0),
        ("abs(-2)", 2.0),
        ("exp(0)", 1.0),
        ("ln(e)", 1.0),
        ("log10(100)", 2.0),
        ("floor(1.7)", 1.0),
        ("ceil(1.2)", 2.0),
        ("round(1.5)", 2.0),
        ("deg(pi)", 180.0),
        ("rad(180)", std::f64::consts::PI),
        ("min(3, 1, 2)", 1.0),
        ("max(3, 1, 2)", 3.0),
        ("pow(2, 10)", 1024.0),
        ("pow(5)", 5.0),
        ("mod(7, 3)", 1.0),
        ("mod(7)", 0.0),
        // as TeX computes them: never negative
        ("Mod(-7, 3)", 2.0),
        ("mod(-7, 3)", -1.0),
        ("Mod(7, -3)", 1.0),
        ("Mod(-7.5, 2)", 0.5),
        ("veclen(3, 4)", 5.0),
        ("veclen(3)", 3.0),
        ("\\pi", std::f64::consts::PI),
        ("+2", 2.0),
        ("{1+1}*2", 4.0),
        ("x^2", 4.0),
        ("2cm", 2.0),
        ("2e3", 2000.0),
        ("2.5E-1", 0.25),
        ("1e+2", 100.0),
        ("2^3^2", 512.0),
    ] {
        close(t, e);
    }
}

#[test]
fn expression_errors() {
    let v = Variables::new();
    for t in ["", "1 2", "(1", "{1", "max(1, 2", "foo(1)", "@", "\\nothing", "doesnotexist", "1.2.3", "2e", "2cmx", "3 +"] {
        assert!(eval(t, &v).is_err(), "{t}");
    }
    // a unit suffix with an invalid rest: falls back to evaluating without a unit and fails there
    assert!(eval_length("xcm", &v).is_err());
    assert_eq!(eval_length("2", &v).unwrap(), (2.0, false));
    assert_eq!(unit_cm("km"), None);
    for u in ["bp", "in", "em", "ex", "pc", "mm"] {
        assert!(eval_length(&format!("1{u}"), &v).unwrap().1, "{u}");
    }
}

#[test]
fn errors_inside_subexpressions() {
    let v = Variables::new();
    for t in ["(1+)", "2*", "2/", "2^", "-", "max(1,)", "sin(1+)", "{1*}"] {
        assert!(eval(t, &v).is_err(), "{t}");
    }
}

#[test]
fn an_array_of_strings_is_indexed_from_zero() {
    let mut v = Variables::new();
    v.insert("x".into(), 2.0);
    let pick = |t: &str| eval_string(t, &v);
    assert_eq!(pick(r#"{"red","green","blue"}[1]"#).unwrap().as_deref(), Some("green"));
    // the index is an expression
    assert_eq!(pick(r#"{"red","green","blue"}[x-1]"#).unwrap().as_deref(), Some("green"));
    assert!(pick(r#"{"red","green"}[2]"#).is_err());
}
