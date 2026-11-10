//! The math library (`\tikzmath{…}` and the `evaluate` key) and pgf's functional shadings
//! (`\pgfdeclarefunctionalshading`, whose PostScript function colors each point of its box).

mod common;
use common::*;
use tikra::canvas::{Item, Paint};
use tikra::color::Color;
use tikra::scene::Segment;

/// The end of the first path drawn by a body: where `\draw (0,0) -- (…);` went.
fn end_of(body: &str) -> (f64, f64) {
    let p = points(&first_path(body).segments);
    *p.last().unwrap()
}

/// The warnings of a picture body.
fn warnings_of(body: &str) -> Vec<String> {
    canvas_and_warnings(body).1
}

// --- \tikzmath: assignments and declarations ---

/// A real variable holds the expression's value; an integer one its whole part.
#[test]
fn reals_and_integers() {
    assert!(close_to(end_of(r"\tikzmath{\a = 2*3 - 4.5;} \draw (0,0) -- (\a,0);"), (1.5, 0.0)));
    for decl in ["integer", "int"] {
        let body = format!(r"\tikzmath{{{decl} \n; \n = 7/2;}} \draw (0,0) -- (\n,0);");
        assert!(close_to(end_of(&body), (3.0, 0.0)), "{decl}");
    }
    // `real` is the default; declaring it changes nothing
    assert!(close_to(end_of(r"\tikzmath{real \r; \r = 7/2;} \draw (0,0) -- (\r,0);"), (3.5, 0.0)));
}

/// A length is kept in pt, without its unit.
#[test]
fn lengths_are_kept_in_points() {
    assert!(close_to(end_of(r"\tikzmath{\l = 2cm;} \draw (0,0) -- (\l pt,0);"), (2.0, 0.0)));
}

/// Statements are separated by semicolons and may use the variables set before them.
#[test]
fn statements_in_sequence() {
    assert!(close_to(end_of(r"\tikzmath{\a = 1; \b = \a + 2; \a = \a * \b;} \draw (0,0) -- (\a,\b);"), (3.0, 3.0)));
}

/// A coordinate variable holds a point, and its x and y (in pt) as `\cx` and `\cy`.
#[test]
fn coordinates() {
    // a plain point, and a computation of points
    assert!(close_to(end_of(r"\tikzmath{coordinate \c; \c = (1,2);} \draw (0,0) -- (\c);"), (1.0, 2.0)));
    assert!(close_to(end_of(r"\tikzmath{coordinate \c; \c = (1,2)+(2,-1);} \draw (0,0) -- (\c);"), (3.0, 1.0)));
    assert!(close_to(end_of(r"\tikzmath{coordinate \c; \c = (2,1);} \draw (0,0) -- (\cx,\cy);"), (2.0, 1.0)));
}

/// An index after the variable (braced, or a number right after it) makes an array.
#[test]
fn indexed_variables() {
    assert!(close_to(end_of(r"\tikzmath{\a{1} = 4; \a{2} = 1;} \draw (0,0) -- (\a{1},\a{2});"), (4.0, 1.0)));
    assert!(close_to(end_of(r"\tikzmath{\x1 = 2; \x2 = 5;} \draw (0,0) -- (\x1,\x2);"), (2.0, 5.0)));
    // two indices
    assert!(close_to(end_of(r"\tikzmath{\m{1,2} = 3;} \draw (0,0) -- (\m{1,2},0);"), (3.0, 0.0)));
    // a coordinate array: each point's x and y too
    assert!(close_to(end_of(r"\tikzmath{coordinate \p; \p{a} = (1,3);} \draw (0,0) -- (\p{a});"), (1.0, 3.0)));
    assert!(close_to(end_of(r"\tikzmath{coordinate \p; \p{a} = (1,3);} \draw (0,0) -- (\px{a},\py{a});"), (1.0, 3.0)));
}

/// `let` keeps the text after `=`, expanded, as it is.
#[test]
fn let_keeps_text() {
    let r = no_warnings(r"\begin{tikzpicture}\tikzmath{let \t = ab; let \u = \t cd;}\node {\u};\end{tikzpicture}");
    assert!(r.svg.contains("abcd"), "{}", r.svg);
    // the text is usable as a number too
    assert!(close_to(end_of(r"\tikzmath{let \t = 2.5;} \draw (0,0) -- (\t,0);"), (2.5, 0.0)));
}

// --- \tikzmath: control ---

/// `for` runs its body for each item of the list.
#[test]
fn for_loops() {
    assert!(close_to(end_of(r"\tikzmath{\s = 0; for \i in {1,2,...,4}{\s = \s + \i;};} \draw (0,0) -- (\s,0);"), (10.0, 0.0)));
    // the list is expanded first
    assert!(close_to(end_of(r"\def\l{3,4} \tikzmath{\s = 1; for \i in {\l}{\s = \s * \i;};} \draw (0,0) -- (\s,0);"), (12.0, 0.0)));
}

/// `if … then {…} else {…}` runs one of its two blocks; without `else` nothing when false.
#[test]
fn conditionals() {
    for (cond, x) in [("1 > 0", 1.0), ("1 < 0", 2.0)] {
        let body = format!(r"\tikzmath{{if {cond} then {{\r = 1;}} else {{\r = 2;}};}} \draw (0,0) -- (\r,0);");
        assert!(close_to(end_of(&body), (x, 0.0)), "{cond}");
    }
    assert!(close_to(end_of(r"\tikzmath{\r = 5; if 1 < 0 then {\r = 1;};} \draw (0,0) -- (\r,0);"), (5.0, 0.0)));
    assert!(close_to(end_of(r"\tikzmath{\r = 5; if 2 > 1 then {\r = 1;};} \draw (0,0) -- (\r,0);"), (1.0, 0.0)));
}

/// A braced statement is TikZ code, run as it is: a loop can draw.
#[test]
fn braced_code_draws() {
    let c = canvas(r"\tikzmath{for \i in {1,2,3}{ {\draw (\i,0) -- (\i,1);}; };}");
    let ends: Vec<_> = paths(&c).iter().map(|d| points(&d.segments)[0]).collect();
    assert_eq!(ends.len(), 3);
    assert!(close_to(ends[2], (3.0, 0.0)), "{ends:?}");
}

/// `print` typesets nothing in a picture; `function` is not supported and says so.
#[test]
fn print_and_function() {
    let (c, w) = canvas_and_warnings(r"\tikzmath{print {hello}; \a = 1;} \draw (0,0) -- (\a,0);");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(paths(&c).len(), 1);
    let (_, w) = canvas_and_warnings(r"\tikzmath{function f(\x) {return \x;};}");
    assert!(w.iter().any(|w| w.contains("function not supported")), "{w:?}");
}

/// The `evaluate` key runs math statements as an option.
#[test]
fn evaluate_key() {
    assert!(close_to(end_of(r"\begin{scope}[evaluate={\w = 2+1;}] \draw (0,0) -- (\w,0); \end{scope}"), (3.0, 0.0)));
}

/// A malformed statement is ignored, with a warning that says what is missing.
#[test]
fn malformed_statements() {
    for (statement, message) in [
        (r"let \t 3;", "let without '='"),
        (r"let t = 3;", "an assignment needs a variable"),
        (r"for \i {1,2}{\a = \i;};", "for without 'in'"),
        (r"for i in {1,2}{\a = 1;};", "for without a variable"),
        (r"for \i in 1,2;", "for without a list"),
        (r"for \i in {1,2};", "for without a body"),
        (r"if 1 > 0 {\a = 1;};", "if without 'then'"),
        (r"if 1 > 0 then \a = 1;", "if without a block"),
        (r"nothing here;", "is not a statement"),
        (r"x = 3;", "an assignment needs a variable"),
        // a block with something after it is not code
        (r"{\draw (0,0) -- (1,0);} \a = 2;", "an assignment needs a variable"),
    ] {
        let w = warnings_of(&format!(r"\tikzmath{{{statement}}}"));
        assert!(w.iter().any(|w| w.contains(message) && w.contains("statement ignored")), "{statement}: {w:?}");
    }
}

// --- functional shadings ---

/// The fills of what a body draws: each cell's color.
fn cell_colors(body: &str) -> Vec<Color> {
    let (c, _) = drawn_and_warnings(body);
    c.items
        .iter()
        .filter_map(|i| match i {
            Item::Path { fill: Some(f), .. } => match &f.paint {
                Paint::Solid(c) => Some(*c),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

/// The colors of a functional shading over a 100bp box whose function is `function`.
fn shaded(init: &str, function: &str) -> Vec<Color> {
    cell_colors(&format!(r"\pgfdeclarefunctionalshading{{f}}{{\pgfpointorigin}}{{\pgfpoint{{100bp}}{{100bp}}}}{{{init}}}{{{function}}} \pgfuseshading{{f}}"))
}

/// The one color a function gives everywhere in the box.
fn uniform(function: &str) -> (f64, f64, f64) {
    let c = shaded("", function);
    assert!(!c.is_empty(), "nothing drawn for {function}");
    let first = c[0];
    assert!(c.iter().all(|x| *x == first), "not one color for {function}");
    (first.r, first.g, first.b)
}

fn close3(a: (f64, f64, f64), b: (f64, f64, f64)) -> bool {
    (a.0 - b.0).abs() < 1e-9 && (a.1 - b.1).abs() < 1e-9 && (a.2 - b.2).abs() < 1e-9
}

/// Each arithmetic operator of PostScript, its result in the red component.
#[test]
fn arithmetic_operators() {
    for (code, r) in [
        ("0.25 0.5 add", 0.75),
        ("0.75 0.5 sub", 0.25),
        ("0.5 0.5 mul", 0.25),
        ("1 4 div", 0.25),
        ("7 2 idiv 10 div", 0.3),
        ("7 4 mod 10 div", 0.3),
        ("0.5 2 exp", 0.25),
        ("1 0 atan 360 div", 0.25),
        ("-1 0 atan 360 div", 0.75),
        ("-0.5 neg", 0.5),
        ("-0.5 abs", 0.5),
        ("0.25 sqrt", 0.5),
        ("-4 sqrt", 0.0),
        ("30 sin", 0.5),
        ("60 cos", 0.5),
        ("1 ln", 0.0),
        ("10 log 2 div", 0.5),
        ("0.75 floor", 0.0),
        ("0.25 ceiling", 1.0),
        ("0.4 round", 0.0),
        ("0.5 round", 1.0),
        ("1.7 truncate", 1.0),
        ("0.9 cvi", 0.0),
        ("0.5 cvr", 0.5),
    ] {
        let c = uniform(&format!("pop pop {code} 0 0"));
        assert!(close3(c, (r, 0.0, 0.0)), "{code}: {c:?}");
    }
}

/// Comparisons and the logic operators give booleans, which count as 1 and 0.
#[test]
fn comparisons_and_logic() {
    for (code, r) in [
        ("1 1 eq", 1.0),
        ("1 2 eq", 0.0),
        ("1 2 ne", 1.0),
        ("2 1 gt", 1.0),
        ("1 1 ge", 1.0),
        ("1 2 lt", 1.0),
        ("2 1 le", 0.0),
        ("true false and", 0.0),
        ("true false or", 1.0),
        ("true true xor", 0.0),
        ("1 0 xor", 1.0),
        ("false not", 1.0),
    ] {
        // a boolean is a number to `add`
        let c = uniform(&format!("pop pop {code} 0 add 0 0"));
        assert!(close3(c, (r, 0.0, 0.0)), "{code}: {c:?}");
    }
}

/// The stack operators.
#[test]
fn stack_operators() {
    for (code, rgb) in [
        ("0.5 dup 0", (0.5, 0.5, 0.0)),
        ("0.25 0.75 exch 0", (0.75, 0.25, 0.0)),
        ("0.1 0.2 pop 0.3 0.4", (0.1, 0.3, 0.4)),
        ("0.1 0.2 2 copy pop", (0.1, 0.2, 0.1)),
        ("0.1 0.2 0.3 2 index exch pop", (0.1, 0.2, 0.1)),
        ("0.1 0.2 0.3 3 1 roll", (0.3, 0.1, 0.2)),
        ("0.1 0.2 0.3 3 -1 roll", (0.2, 0.3, 0.1)),
        ("0.1 0.2 0.3 0 1 roll", (0.1, 0.2, 0.3)),
    ] {
        let c = uniform(&format!("pop pop {code}"));
        assert!(close3(c, rgb), "{code}: {c:?}");
    }
}

/// `if` and `ifelse` run a procedure on a condition; the function sees the point (bp).
#[test]
fn conditionals_on_the_point() {
    // red on the right half of the box, blue on the left
    let c = shaded("", "pop 50 gt {1 0 0} {0 0 1} ifelse");
    assert!(c.iter().any(|c| c.r == 1.0) && c.iter().any(|c| c.b == 1.0), "{c:?}");
    // `if`: green above the middle; below, the function leaves nothing and no cell is drawn
    let c = shaded("", "exch pop 50 gt {0 1 0} if");
    assert!(!c.is_empty() && c.iter().all(|c| c.g == 1.0), "{c:?}");
    assert!(close3(uniform("pop pop true {0.5 0.5 0.5} if"), (0.5, 0.5, 0.5)));
    assert!(close3(uniform("pop pop 0 {1 1 1} {0.5 0 0} ifelse"), (0.5, 0.0, 0.0)));
}

/// A PostScript comment (a `%` made an ordinary character) runs to the end of its line.
#[test]
fn comments_are_skipped() {
    let document = r"\catcode`\%=12 \def\fn{pop pop 0 1 0 % then red: 1 0 0}
        \begin{tikzpicture}\pgfdeclarefunctionalshading{f}{\pgfpointorigin}{\pgfpoint{1cm}{1cm}}{}{\fn}\pgfuseshading{f}\end{tikzpicture}";
    let (c, _) = tikra::engine::run_document(document).unwrap();
    let fills: Vec<_> = paths(&c).into_iter().filter_map(|d| d.fill).collect();
    assert!(!fills.is_empty() && fills.iter().all(|f| f.paint == Paint::Solid(Color::rgb(0.0, 1.0, 0.0))), "{fills:?}");
}

/// A function that cannot run (an unknown operator, a stack too short, a procedure as a number,
/// too many steps) colors nothing.
#[test]
fn functions_that_fail_draw_nothing() {
    let long = "1 pop ".repeat(10_001);
    for code in [
        "pop pop 1 0 0 frobnicate".to_string(),
        "pop pop 0 0".to_string(),
        "pop pop 1 0 true".to_string(),
        "pop pop {1} 0 add 0 0".to_string(),
        "pop pop {1} not 0 0".to_string(),
        "pop pop pop dup 0 0".to_string(),
        "pop pop 1 exch 0 0".to_string(),
        "pop pop 1 5 copy".to_string(),
        "pop pop 1 1 index 0".to_string(),
        "pop pop 1 4 1 roll 0".to_string(),
        "pop pop 1 1 if 0 0".to_string(),
        "pop pop 1 1 {0} ifelse 0 0".to_string(),
        "pop pop 1 {0} 1 ifelse 0 0".to_string(),
        format!("pop pop {long} 1 0 0"),
    ] {
        assert!(shaded("", &code).is_empty(), "{}", &code[..code.len().min(40)]);
    }
}

/// The setup code runs first: `\pgfshadecolortorgb` gives a color's components to the function.
#[test]
fn setup_and_color_components() {
    let c = uniform_with(r"\pgfshadecolortorgb{red!50!blue}{\mine}", r"pop pop \minered \minegreen \mineblue");
    assert!(close3(c, (0.5, 0.0, 0.5)), "{c:?}");
    // the whole color at once
    let c = uniform_with(r"\pgfshadecolortorgb{green}{\mine}", r"pop pop \mine");
    assert!(close3(c, (0.0, 1.0, 0.0)), "{c:?}");
    // a target that is not a macro sets nothing; an unknown color is black
    let c = uniform_with(r"\def\mine{0 0 1 }\pgfshadecolortorgb{red}{mine}\pgfshadecolortorgb{nosuchcolor}{\other}", r"pop pop \mine pop \other pop pop");
    assert!(close3(c, (0.0, 0.0, 0.0)), "{c:?}");
}

fn uniform_with(init: &str, function: &str) -> (f64, f64, f64) {
    let c = shaded(init, function);
    assert!(!c.is_empty(), "nothing drawn for {function}");
    (c[0].r, c[0].g, c[0].b)
}

/// A declared functional shading as a `\pgftext` is placed by its anchor, its box its size.
#[test]
fn functional_shading_in_a_text() {
    let body = r"\pgfdeclarefunctionalshading{f}{\pgfpointorigin}{\pgfpoint{2cm}{1cm}}{}{pop pop 1 0 0}
        \pgftext[left,bottom]{\pgfuseshading{f}}";
    let (c, _) = drawn_and_warnings(body);
    let (lo, hi) = (c.min.unwrap(), c.max.unwrap());
    assert!(close(hi.x - lo.x, 2.0) && close(hi.y - lo.y, 1.0), "{lo:?} {hi:?}");
}

/// A path shaded with a declared functional shading is filled with its cells.
#[test]
fn functional_shading_of_a_path() {
    let c = cell_colors(
        r"\pgfdeclarefunctionalshading{half}{\pgfpointorigin}{\pgfpoint{100bp}{100bp}}{}{pop 50 gt {1 0 0} {0 0 1} ifelse}
          \shade[shading=half] (0,0) rectangle (2,1);",
    );
    assert!(c.iter().any(|c| c.r == 1.0) && c.iter().any(|c| c.b == 1.0), "{c:?}");
}

/// The shadings library's Mandelbrot set: black where the iteration stays bounded (around the
/// origin), drawn on white, finer than the other shadings.
#[test]
fn mandelbrot_set() {
    let (c, _) = drawn_and_warnings(r"\pgfuseshading{Mandelbrot set}");
    let fills: Vec<(Color, Vec<Segment>)> = c
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Path { segments, fill: Some(f), .. } => match &f.paint {
                Paint::Solid(c) => Some((*c, segments.clone())),
                _ => None,
            },
            _ => None,
        })
        .collect();
    // the white background first, over the whole 100bp box
    let (bg, segs) = &fills[0];
    assert_eq!(*bg, Color::WHITE);
    let (lo, hi) = bounds(segs);
    assert!(close(hi.x - lo.x, 100.0 * 2.54 / 72.0), "{lo:?} {hi:?}");
    // black cells inside the set, gray ones outside it
    assert!(fills.iter().any(|(c, _)| c.r == 0.0), "no black");
    assert!(fills.iter().any(|(c, _)| c.r > 0.0 && c.r < 1.0), "no gray");
    // finer cells than the 64 per side of the other shadings
    assert!(fills.len() > 64 * 4, "{}", fills.len());
}
