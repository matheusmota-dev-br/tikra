//! Three libraries that run code for each of many things: the calendar (a scope per day, the
//! date tests of `\ifdate` and pgf's date macros), folding (nets of polyhedra) and Lindenmayer
//! systems (a string rewritten, then drawn by a turtle).

mod common;

use common::*;
use tikra::scene::Segment;

/// The texts of a figure, as their origins (cm).
fn text_origins(body: &str) -> Vec<(f64, f64)> {
    texts(&canvas(body)).iter().map(|(_, p)| (p.e, p.f)).collect()
}

/// The warnings of a body.
fn warnings(body: &str) -> Vec<String> {
    canvas_and_warnings(body).1
}

/// The line segments of the first path of a body.
fn lines(body: &str) -> usize {
    first_path(body).segments.iter().filter(|s| matches!(s, Segment::LineTo(_))).count()
}

// ------------------------------------------------------------------ pgf's date macros

#[test]
fn dates_become_julian_days_and_back() {
    // 2000-03-01 is Julian day 2451605, a Wednesday (2); the day before, the 29th of February
    let body = r"\newcount\j \pgfcalendardatetojulian{2000-03-01}{\j}
        \pgfcalendarjuliantoweekday{\the\j}{\w}
        \advance\j by -1 \pgfcalendarjuliantodate{\the\j}{\y}{\m}{\d}
        \node at (0,0) {\the\j}; \node at (0,-1) {\w}; \node at (0,-2) {\y/\m/\d};
        \pgfcalendardatetojulian{2000-01-01+10}{\k} \node at (0,-3) {\k};";
    let out = no_warnings(&format!("\\begin{{tikzpicture}}{body}\\end{{tikzpicture}}"));
    for text in ["2451604", "2", "2000/02/29", "2451555"] {
        assert!(out.description.contains(text), "{text}: {}", out.description);
    }
    // a count given to the weekday, too
    let out = no_warnings(r"\begin{tikzpicture}\newcount\w \pgfcalendarjuliantoweekday{2451545}{\w}\node {\the\w};\end{tikzpicture}");
    assert!(out.description.contains('5'), "{}", out.description);
}

#[test]
fn ifdate_on_a_given_date() {
    // Easter: in April in 1954, 1957 and 1981 (the computus's corrections), in March in 2024
    for (date, easter) in [("1954-04-18", true), ("1957-04-21", true), ("1981-04-19", true), ("2024-03-31", true), ("2024-04-01", false)] {
        let c = canvas(&format!(r"\pgfcalendarifdate{{{date}}}{{Easter}}{{\fill (0,0) circle (1pt);}}{{\draw (0,0) -- (1,0);}}"));
        assert_eq!(paths(&c)[0].fill.is_some(), easter, "{date}");
    }
    // Easter=1 is the Monday after
    let c = canvas(r"\pgfcalendarifdate{2024-04-01}{Easter=1}{\fill (0,0) circle (1pt);}{}");
    assert_eq!(paths(&c).len(), 1);
}

#[test]
fn the_tests_of_ifdate() {
    // one dot per day of January 2001 (it starts on a Monday) that passes the test
    let count = |test: &str| {
        let body = format!(r"\pgfcalendar{{c}}{{2001-01-01}}{{2001-01-31}}{{\ifdate{{{test}}}{{\fill (\pgfcalendarcurrentday,0) circle (1pt);}}{{}}}}");
        let (c, w) = canvas_and_warnings(&body);
        assert!(w.is_empty(), "{test}: {w:?}");
        paths(&c).len()
    };
    assert_eq!(count("day of month=15"), 1);
    assert_eq!(count("end of month"), 1);
    // two days before the end: the 29th only
    assert_eq!(count("end of month=3"), 1);
    assert_eq!(count("at least=01-25"), 7);
    assert_eq!(count("at most=2001-01-03"), 3);
    assert_eq!(count("between=01-10 and 01-12"), 3);
    assert_eq!(count("between=2001-01-30and2001-01-31"), 2);
    assert_eq!(count("Monday, Friday"), 5 + 4);
}

#[test]
fn a_test_nobody_knows_is_reported() {
    let w = warnings(r"\pgfcalendar{c}{2001-01-01}{2001-01-01}{\ifdate{fullmoon}{x}{}}");
    assert!(w.iter().any(|w| w.contains("\\ifdate: unknown test 'fullmoon'")), "{w:?}");
}

#[test]
fn wrong_dates_and_ranges_are_dropped() {
    for (body, message) in [
        (r"\pgfcalendar{c}{2000}{2000-01-02}{}", "not a date: '2000'"),
        (r"\pgfcalendar{c}{2000-01-01}{2001-01-xx}{}", "not a date: '2001-01-xx'"),
        (r"\pgfcalendar{c}{1900-01-01}{2000-01-01}{}", "calendar from 1900-01-01 to 2000-01-01: too many days"),
        (r"\pgfcalendarifdate{yesterday}{all}{}{}", "not a date: 'yesterday'"),
        (r"\pgfcalendardatetojulian{0-0}{\j}", "not a date: '0-0'"),
        (r"\ifdate{all}{}{}", "\\ifdate outside a calendar"),
    ] {
        let w = warnings(body);
        assert!(w.iter().any(|w| w.contains(message)), "{body}: {w:?}");
    }
    // an error on a day drops that statement only: every day draws its dot
    let (c, w) = canvas_and_warnings(r"\pgfcalendar{c}{2001-01-01}{2001-01-09}{\fill (\pgfcalendarcurrentday,0) circle (1pt); \ifdate{between=01-02}{}{}}");
    assert!(w.iter().any(|w| w.contains("between=01-02: no 'and'")), "{w:?}");
    assert_eq!(paths(&c).len(), 9);
    // a limit reached on a day stops the calendar, and the figure
    let e = tikra::convert(r"\tikz \pgfcalendar{c}{2001-01-01}{2001-01-09}{\def\deep{\begin{scope}\deep\end{scope}}\deep};").expect_err("endless nesting");
    assert!(e.to_string().contains("nesting went past"), "{e}");
}

// ------------------------------------------------------------------ \calendar

#[test]
fn a_calendar_at_a_point() {
    // the first day is at (2,1): its text ends there (every day is anchored base east)
    let body = r"\calendar (c) at (2,1) [dates=2001-01-01 to 2001-01-02];";
    let p = point(body, "c-2001-01-01.base east");
    assert!(close_to((p.x, p.y), (2.0, 1.0)), "{p:?}");
    // a point is needed
    let w = warnings(r"\calendar at [dates=2001-01-01 to 2001-01-02];");
    assert!(w.iter().any(|w| w.contains("\\calendar at: a point is missing")), "{w:?}");
}

#[test]
fn a_calendar_expands_macros_and_rejects_the_rest() {
    let body = r"\def\when{[dates=2001-01-01 to 2001-01-03]}\calendar (c) \when;";
    assert_eq!(text_origins(body).len(), 3);
    let w = warnings(r"\calendar [dates=2001-01-01 to 2001-01-03] x;");
    assert!(w.iter().any(|w| w.contains("\\calendar: unexpected x")), "{w:?}");
    // a macro that only gives itself back
    let w = warnings(r"\def\again{\again}\calendar \again;");
    assert!(w.iter().any(|w| w.contains("\\calendar: giving up on this calendar")), "{w:?}");
    // dates must be a range
    let w = warnings(r"\calendar [dates=2001-01-01];");
    assert!(w.iter().any(|w| w.contains("dates=2001-01-01 is not 'start to end'")), "{w:?}");
}

#[test]
fn if_else_choose_per_day() {
    // red on Mondays, else drawn by a code of its own; January 2001 starts on a Monday
    let body = r"\calendar [dates=2001-01-01 to 2001-01-07, day list right] if (Monday) [red] else {\fill (0,-1) circle (1pt);};";
    let c = canvas(body);
    let t = texts(&c);
    assert_eq!(t.len(), 7);
    assert_eq!(t[0].0.color.hex(), "#ff0000");
    assert_eq!(t[1].0.color.hex(), "#000000");
    // the other six days have a dot each
    assert_eq!(paths(&c).iter().filter(|d| d.fill.is_some()).count(), 6);
    // the code of an if given as a group, and an if with no code
    let c = canvas(r"\calendar [dates=2001-01-01 to 2001-01-07, day list right] if (weekend) {\fill (0,-1) circle (1pt);} if (all);");
    assert_eq!(paths(&c).len(), 2);
    let c = canvas(r"\calendar [dates=2001-01-01 to 2001-01-02] if (all) x;");
    assert_eq!(texts(&c).len(), 0, "the calendar is dropped at x");
    // the if key, among the options
    let c = canvas(r"\calendar [dates=2001-01-01 to 2001-01-07, day list right, if={(Sunday) [blue]}];");
    assert_eq!(texts(&c)[6].0.color.hex(), "#0000ff");
    // the tests are needed
    let w = warnings(r"\calendar [dates=2001-01-01 to 2001-01-02] if [red];");
    assert!(w.iter().any(|w| w.contains("\\calendar if: the tests are missing")), "{w:?}");
}

#[test]
fn an_if_at_the_very_end() {
    // the code ends right after the if: no else, nothing more
    let out = no_warnings(r"\tikz \calendar (c) [dates=2001-01-01 to 2001-01-02] if (all)");
    let names: Vec<&str> = out.points.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["c-2001-01-01", "c-2001-01-02"]);
}

#[test]
fn day_lists_go_their_way_and_months_jump() {
    // two months, a day each side of the change; day shifts in em and pt
    let o = |list: &str| {
        let body = format!(r"\calendar (c) [dates=2001-01-30 to 2001-02-02, {list}, day xshift=1em, day yshift=10pt, month xshift=2cm, month yshift=3cm];");
        let p: Vec<_> = ["c-2001-01-30", "c-2001-01-31", "c-2001-02-01", "c-2001-02-02"].iter().map(|n| point(&body, &format!("{n}.base east"))).collect();
        p.windows(2).map(|w| (w[1].x - w[0].x, w[1].y - w[0].y)).collect::<Vec<_>>()
    };
    let pt = 2.54 / 72.27;
    let em = 10.0 * pt;
    for (list, expected) in [
        ("day list downward", [(0.0, -10.0 * pt), (0.0, -10.0 * pt - 3.0), (0.0, -10.0 * pt)]),
        ("day list upward", [(0.0, 10.0 * pt), (0.0, 10.0 * pt + 3.0), (0.0, 10.0 * pt)]),
        ("day list right", [(em, 0.0), (em + 2.0, 0.0), (em, 0.0)]),
        ("day list left", [(-em, 0.0), (-em - 2.0, 0.0), (-em, 0.0)]),
    ] {
        let got = o(list);
        assert!(got.iter().zip(expected).all(|(g, e)| (g.0 - e.0).abs() < 1e-3 && (g.1 - e.1).abs() < 1e-3), "{list}: {got:?}");
    }
}

#[test]
fn a_month_list_puts_each_day_in_its_weekday_column() {
    // February 2001 starts on a Thursday: its 1st is three columns right of where March 2001's
    // 1st (also a Thursday) is, one month down
    let body = r"\calendar (c) [dates=2001-02-01 to 2001-03-01, month list, day xshift=1cm, month yshift=2cm];";
    let (a, b, c) = (point(body, "c-2001-02-01.base east"), point(body, "c-2001-02-02.base east"), point(body, "c-2001-03-01.base east"));
    assert!(close(b.x - a.x, 1.0) && close(b.y, a.y), "{a:?} {b:?}");
    assert!(close(c.x, a.x) && close(a.y - c.y, 2.0), "{a:?} {c:?}");
}

#[test]
fn month_labels_go_where_they_say() {
    // the label's text, from the first day: above or below, by 1.25 day yshifts
    let label = |place: &str| {
        let body = format!(r"\calendar (c) [dates=2001-01-01 to 2001-01-01, day yshift=1cm, month label {place}];");
        let c = canvas(&body);
        let t = texts(&c);
        assert_eq!(t.len(), 2, "{place}");
        // the month's label is drawn before the day's text
        t[0].1.f - t[1].1.f
    };
    for (place, dy) in [("above centered", 1.25), ("above left", 1.25), ("above right", 1.25), ("below centered", -1.25), ("below left", -1.25), ("left", 0.0), ("right", 0.0)] {
        assert!((label(place) - dy).abs() < 0.2, "{place}: {}", label(place));
    }
}

// ------------------------------------------------------------------ folding

#[test]
fn nets_have_their_faces_folds_cuts_and_ears() {
    // a face is a node of numbered faces: one per face of the solid
    for (pic, faces) in [
        ("cube", 6),
        ("dodecahedron", 12),
        ("dodecahedron'", 12),
        ("tetrahedron truncated", 8),
        ("cuboctahedron truncated", 26),
        ("icosidodecahedron", 32),
        ("dodecahedron rhombic", 12),
        ("dodecahedron golden", 12),
    ] {
        let (c, w) = canvas_and_warnings(&format!(r"\pic [numbered faces, folding line length=5mm] {{{pic} folding}};"));
        assert!(w.is_empty(), "{pic}: {w:?}");
        assert_eq!(texts(&c).len(), faces, "{pic}");
    }
    // the command form, with the rhombic triacontahedron that has no pic
    let (c, w) = canvas_and_warnings(r"\tikzfoldingrhombictricontahedron[numbered faces, folding line length=4mm];");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(texts(&c).len(), 14 + 16);
}

#[test]
fn a_cube_net_is_six_squares_of_the_folding_line_length() {
    // every edge is a fold, a cut or an ear's side: all as long as the line length (or the ears' sides)
    let c = canvas(r"\pic [folding line length=1cm, every fold/.style={red}] {cube folding};");
    let p = paths(&c);
    let red: Vec<_> = p.iter().filter(|d| d.stroke.as_ref().is_some_and(|s| s.color.hex() == "#ff0000")).collect();
    // the folds between the six squares (5) and those of the 7 ears
    assert_eq!(red.len(), 5 + 7);
    for d in &red {
        let pts = points(&d.segments);
        let (a, b) = (pts[0], pts[1]);
        assert!(close(((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt(), 1.0), "{pts:?}");
    }
    // a face's code is in the face, turned: face 3@90 of the rhombic dodecahedron
    let c = canvas(r"\pic [face 3={\node[draw] (f) {x};}] {dodecahedron rhombic folding};");
    assert_eq!(texts(&c).len(), 1);
}

// ------------------------------------------------------------------ Lindenmayer systems

#[test]
fn the_basic_layer_draws_a_system_on_the_pgf_path() {
    let body = r"\pgfdeclarelindenmayersystem{square}{\rule{F -> F+F}}
        \pgfkeys{/pgf/lindenmayer system/step=1cm, /pgf/lindenmayer system/angle=90}
        \pgfpathmoveto{\pgfpointorigin}\pgflindenmayersystem{square}{F}{2}\pgfusepath{stroke}";
    let d = first_path(body);
    let pts = points(&d.segments);
    // F+F+F+F: around the unit square
    let expected = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0), (0.0, 0.0)];
    assert_eq!(pts.len(), expected.len(), "{pts:?}");
    assert!(pts.iter().zip(expected).all(|(p, e)| close_to(*p, e)), "{pts:?}");
}

#[test]
fn a_rule_set_declares_a_system_on_the_spot() {
    // f moves without drawing
    let body = r"\draw [l-system={rule set={F -> FfF, f -> ff}, axiom=F, order=1, step=1cm}] (0,0) lindenmayer system;";
    // (the path's (0,0), then the system's own start there)
    let d = first_path(body);
    let pts = points(&d.segments);
    assert_eq!(pts, [(0.0, 0.0), (0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (3.0, 0.0)]);
    assert!(matches!(d.segments[3], Segment::MoveTo(_)));
}

#[test]
fn symbols_say_what_a_character_does() {
    // A and B draw and turn; C does nothing; S and R save and restore
    let body = r"\pgfdeclarelindenmayersystem{mine}{
          \symbol{A}{\pgflsystemdrawforward} \symbol{M}{\pgflsystemmoveforward}
          \symbol{L}{\pgflsystemturnleft} \symbol{R}{\pgflsystemturnright}
          \symbol{S}{\pgflsystemsavestate} \symbol{T}{\pgflsystemrestorestate} \symbol{C}{}}
        \draw [l-system={mine, axiom=SALAT RA C M A, step=1cm, angle=90}] (0,0) lindenmayer system;";
    let d = first_path(body);
    let pts = points(&d.segments);
    // up to (1,0), left turn, to (1,1); back to (0,0) facing x; right turn: to (0,-1); a move, a line
    let expected = [(0.0, 0.0), (0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 0.0), (0.0, -1.0), (0.0, -2.0), (0.0, -3.0)];
    assert!(pts.len() == expected.len() && pts.iter().zip(expected).all(|(p, e)| close_to(*p, e)), "{pts:?}");
    // other code is not drawn, and said so
    let odd = r"\pgfdeclarelindenmayersystem{odd}{\symbol{X}{\relax} \symbol{Y}{\pgflsystemturnleft\pgflsystemturnleft}}";
    let w = warnings(&format!(r"{odd}\draw [l-system={{odd, axiom=FXY}}] lindenmayer system;"));
    assert!(w.iter().any(|w| w.contains("Lindenmayer symbol X: \\relax not supported: ignored")), "{w:?}");
    assert!(w.iter().any(|w| w.contains("Lindenmayer symbol Y: only one of the \\pgflsystem… commands is supported: ignored")), "{w:?}");
}

#[test]
fn random_steps_and_angles_vary_but_stay_close() {
    let body = r"\pgfdeclarelindenmayersystem{line}{\rule{F -> F+F}}
        \draw [l-system={line, axiom=F, order=3, step=1cm, angle=0, randomize step percent=20, randomize angle percent=20}] (0,0) lindenmayer system;";
    let pts = points(&first_path(body).segments);
    // the path's start, the system's, then eight steps
    assert_eq!(pts.len(), 10);
    let (mut longest, mut shortest) = (0.0f64, f64::MAX);
    for w in pts[1..].windows(2) {
        let d = ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt();
        longest = longest.max(d);
        shortest = shortest.min(d);
    }
    // 20% over 20 is 1 pt at most either way
    let pt = 2.54 / 72.27;
    assert!(longest > shortest && longest <= 1.0 + pt + 1e-9 && shortest >= 1.0 - pt - 1e-9, "{shortest} {longest}");
    // the same figure twice is drawn the same: pgf's random numbers start from the same seed
    assert_eq!(pts, points(&first_path(body).segments));
}

#[test]
fn an_anchor_places_the_drawing_like_a_node() {
    // a 2 cm line, F F: placed by its anchor at (5,5)
    let sys = r"\pgfdeclarelindenmayersystem{two}{\rule{A -> FF}}";
    for (anchor, start) in [("west", (5.0, 5.0)), ("east", (3.0, 5.0)), ("center", (4.0, 5.0)), ("south", (4.0, 5.0)), ("north east", (3.0, 5.0)), ("base west", (5.0, 5.0))] {
        let body = format!(r"{sys}\draw [l-system={{two, axiom=A, order=1, step=1cm, anchor={anchor}}}] (5,5) lindenmayer system;");
        // (after the path's own (5,5))
        let pts = points(&first_path(&body).segments);
        assert!(close_to(pts[1], start), "{anchor}: {pts:?}");
        assert!(close_to(*pts.last().unwrap(), (start.0 + 2.0, start.1)), "{anchor}: {pts:?}");
    }
    // a square, F+F+F: north is its top
    let body = r"\pgfdeclarelindenmayersystem{sq}{\rule{A -> F+F+F}}\draw [l-system={sq, axiom=A, order=1, step=1cm, anchor=north west}] (0,0) lindenmayer system;";
    let pts = points(&first_path(body).segments);
    assert!(close_to(pts[1], (0.0, -1.0)), "{pts:?}");
}

#[test]
fn a_system_that_grows_too_long_is_cut() {
    // X becomes ten Xs eight times over; only the F before them draws
    let body = r"\pgfdeclarelindenmayersystem{big}{\rule{X -> XXXXXXXXXX}}\draw [l-system={big, axiom=FX, order=8}] (0,0) lindenmayer system;";
    assert_eq!(lines(body), 1);
}

#[test]
fn systems_and_rules_must_be_complete() {
    for (body, message) in [
        (r"\draw [l-system={axiom=F}] (0,0) lindenmayer system;", "no Lindenmayer system named ''"),
        (r"\draw [l-system={name=nowhere, axiom=F}] (0,0) lindenmayer system;", "no Lindenmayer system named 'nowhere'"),
        (r"\pgfdeclarelindenmayersystem{bad}{\rule{F = FF}}", "Lindenmayer rule 'F = FF' has no '->'"),
    ] {
        let w = warnings(body);
        assert!(w.iter().any(|w| w.contains(message)), "{body}: {w:?}");
    }
}
