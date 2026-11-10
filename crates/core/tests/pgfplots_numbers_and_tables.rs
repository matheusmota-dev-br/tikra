//! pgfplots' numbers and tables: `\pgfmathprintnumber`'s styles, the floating point unit that
//! samples domains, the tick algorithm's arithmetic, and the tables `\addplot table` reads
//! (files, `\pgfplotstableread`, `\pgfplotstabletypeset`, `\thisrow`, linear regressions,
//! histograms, error bars and tie lines).

mod common;
use common::*;
use tikra::engine::pgfplots::fpu::{self, Num};
use tikra::engine::pgfplots::number::{NumberFormat, SciStyle, Style};
use tikra::engine::pgfplots::table;
use tikra::engine::pgfplots::ticks::{self, TickSettings};

/// The figure's description and warnings: an axis with these options around a body.
fn describe(options: &str, body: &str) -> (String, Vec<String>) {
    let r = figure(&format!("\\begin{{tikzpicture}}\\begin{{axis}}[{options}]\n{body}\n\\end{{axis}}\\end{{tikzpicture}}"));
    (r.description, r.warnings)
}

/// The description of a figure that must have no warnings.
fn described(options: &str, body: &str) -> String {
    let (d, w) = describe(options, body);
    assert!(w.is_empty(), "warnings: {w:?}\n{body}");
    d
}

/// The texts of a picture, as its description reads them.
fn words(body: &str) -> String {
    no_warnings(&format!("\\begin{{tikzpicture}}{body}\\end{{tikzpicture}}")).description
}

// ---------------------------------------------------------------------------------------------
// \pgfmathprintnumber

#[test]
fn the_styles_print_numbers_as_pgf_does() {
    let f = |style: Style| NumberFormat { style, ..Default::default() };
    // `frac`: the smallest denominator, the whole part apart, the sign before
    for (v, s) in [(2.0, "2"), (0.75, "\\frac{3}{4}"), (1.5, "1\\frac{1}{2}"), (-0.5, "-\\frac{1}{2}"), (-2.25, "-2\\frac{1}{4}")] {
        assert_eq!(f(Style::Frac).print(v), s, "{v}");
    }
    // `int detect`: whole numbers as integers, the others in scientific notation
    assert_eq!(f(Style::IntDetect).print(4.0), "4");
    assert_eq!(f(Style::IntDetect).print(2.5), "2.5\\cdot 10^{0}");
    // `relative=0`: rounded to `precision` digits after 10^0, then printed as `std` would
    assert_eq!(f(Style::Relative(0)).print(1.23456), "1.23");
    assert_eq!(f(Style::Relative(0)).print(0.001), "0");
    assert_eq!(f(Style::Relative(0)).print(123456.0), "1.23456\\cdot 10^{5}");
    // `relative=-2` keeps two digits after the hundredths: 0.012345 is 0.0123
    assert_eq!(f(Style::Relative(-2)).print(0.012345), "0.0123");
    // fixed relative: `precision` significant digits, no zeros filled in
    let fr = NumberFormat { style: Style::FixedRelative, precision: 3, ..Default::default() };
    assert_eq!(fr.print(0.0123456), "0.0123");
    assert_eq!(fr.print(98765.0), "98{,}800");
}

#[test]
fn infinities_signs_and_tiny_numbers() {
    let d = NumberFormat::default();
    assert_eq!(d.print(f64::INFINITY), "\\infty");
    assert_eq!(d.print(f64::NEG_INFINITY), "-\\infty");
    let pos = NumberFormat { showpos: true, ..Default::default() };
    assert_eq!(pos.print(f64::INFINITY), "+\\infty");
    assert_eq!(pos.print(2.0), "+2");
    assert_eq!(pos.print(-2.0), "-2");
    // a number below the last digit kept is rounded away entirely
    let fixed = NumberFormat { style: Style::Fixed, ..Default::default() };
    assert_eq!(fixed.print(0.0001), "0");
    assert_eq!(fixed.print(0.004), "0");
    assert_eq!(fixed.print(0.005), "0.01");
}

#[test]
fn scientific_notations() {
    let sci = |sci_style: SciStyle| NumberFormat { style: Style::Sci, sci_style, ..Default::default() };
    for (style, big, small) in [
        (SciStyle::LowerE, "1.23e{+}4", "1.2e{-}3"),
        (SciStyle::UpperE, "1.23E{+}4", "1.2E{-}3"),
        (SciStyle::Subscript, "1.23_{4}", "1.2_{-3}"),
        (SciStyle::Superscript, "1.23^{4}", "1.2^{-3}"),
        (SciStyle::TenE, "1.23\\cdot 10^{4}", "1.2\\cdot 10^{-3}"),
    ] {
        assert_eq!(sci(style).print(12345.0), big, "{style:?}");
        assert_eq!(sci(style).print(0.0012), small, "{style:?}");
    }
    // without `retain unit mantissa`, a mantissa of one is left out (of 10^e only)
    let bare = NumberFormat { style: Style::Sci, unit_mantissa: false, ..Default::default() };
    assert_eq!(bare.print(1000.0), "10^{3}");
    assert_eq!(bare.print(-0.01), "-10^{-2}");
    assert_eq!(bare.print(2000.0), "2\\cdot 10^{3}");
    let e = NumberFormat { style: Style::Sci, unit_mantissa: false, sci_style: SciStyle::LowerE, ..Default::default() };
    assert_eq!(e.print(1000.0), "1e{+}3");
}

#[test]
fn number_format_keys_in_a_node() {
    // the keys of /pgf/number format, given to \pgfmathprintnumber
    let d = words(
        r"\node {\pgfmathprintnumber[fixed, precision=3, zerofill]{2.5}};
          \node at (0,1) {\pgfmathprintnumber[fixed, precision=3, zerofill=false]{2.5}};
          \node at (0,2) {\pgfmathprintnumber[showpos]{7}};
          \node at (0,3) {\pgfmathprintnumber[int detect]{4.0}};
          \node at (0,4) {$\pgfmathprintnumber[frac]{0.75}$};",
    );
    // printed in math, as pgf does, so the sign is read as math
    assert_eq!(d, "Text in the figure: 2.500; 2.5; plus 7; 4; 3 over 4.");
}

// ---------------------------------------------------------------------------------------------
// the floating point unit

#[test]
fn the_fpu_at_its_edges() {
    // what is not a number has no decimal form
    assert!(Num::from_f64(f64::NAN).is_none());
    assert!(Num::from_f64(f64::INFINITY).is_none());
    // zero divided, or divided by zero, is zero (pgf's fpu does not fail)
    let one = Num::from_f64(1.0).unwrap();
    assert!(Num::ZERO.over(one).is_zero());
    assert!(one.over(Num::ZERO).is_zero());
    assert_eq!(Num::from_f64(3.0).unwrap().over(Num::from_f64(4.0).unwrap()).value(), 0.75);
    // square roots and logarithms of what has none
    assert!(fpu::sqrt(-1.0).is_nan());
    assert!(fpu::sqrt(f64::NAN).is_nan());
    assert_eq!(fpu::sqrt(0.25), 0.5);
    assert_eq!(fpu::ln(0.0), f64::NEG_INFINITY);
    assert!(fpu::ln(-1.0).is_nan());
    assert!((fpu::ln(1e5) - 11.51291).abs() < 1e-9);
}

// ---------------------------------------------------------------------------------------------
// the tick algorithm's arithmetic

fn settings(distance: Option<f64>) -> TickSettings {
    TickSettings { length_pt: 200.0, max_space: 35.0, try_min: 4, distance, scale: None }
}

#[test]
fn tick_distances_snap_to_one_two_and_five() {
    // every power of ten (whose logarithm may be inexact) is its own distance
    for k in -300..=300 {
        let h = 10f64.powi(k);
        let n = ticks::normalize_distance(h);
        assert!((n / h - 1.0).abs() < 1e-9, "{h}: {n}");
        let n = ticks::normalize_distance(3.0 * h);
        assert!((n / h - 2.0).abs() < 1e-9, "3·{h}: {n}");
    }
    // no distance: nothing to snap
    assert_eq!(ticks::normalize_distance(-1.0), -1.0);
    assert!(ticks::normalize_distance(f64::NAN).is_nan());
    // an empty range has no ticks
    assert_eq!(ticks::linear(1.0, 1.0, settings(None)), (Vec::new(), 0.0));
}

#[test]
fn the_data_scale_of_negative_data() {
    // data up to 0: the order of the smallest value chooses 10^E (a 200pt axis: 10^(3-3))
    assert_eq!(ticks::data_scale(-500.0, 0.0, 200.0), 1.0);
    assert_eq!(ticks::data_scale(0.0, 500.0, 200.0), 1.0);
}

#[test]
fn logarithmic_ticks() {
    let ln = |v: f64| v.ln();
    // `xtick distance=100` on a log axis: every second power of ten
    let (t, h, uniform) = ticks::logarithmic(0.0, ln(1e6), settings(Some(100.0)));
    assert!(!uniform);
    assert!((h - ln(100.0)).abs() < 1e-9);
    let decades: Vec<f64> = t.iter().map(|v| (v / std::f64::consts::LN_10 * 1e6).round() / 1e6).collect();
    // pgfplots starts one decade up when the first power is closer than half a step to the
    // axis' start (the last candidate is past the end: it is not drawn)
    assert_eq!(decades, vec![1.0, 3.0, 5.0, 7.0]);
    // a factor of one is no distance
    assert_eq!(ticks::logarithmic(0.0, ln(1e6), settings(Some(1.0))), (Vec::new(), 0.0, false));
    // inside one decade there is no second power of ten: ticks at fractional powers instead
    let (t, h, uniform) = ticks::logarithmic(ln(2.0), ln(8.0), settings(None));
    assert!(!uniform && t.len() >= 3, "{t:?}");
    assert!(h < std::f64::consts::LN_10, "{h}");
    assert!(t.windows(2).all(|w| (w[1] - w[0] - h).abs() < 1e-9));
    // and below one, the start is one step lower
    let (t, _, _) = ticks::logarithmic(ln(0.2), ln(0.8), settings(None));
    assert!(t[0] <= ln(0.2) && t.len() >= 3, "{t:?}");
}

#[test]
fn scale_exponents_of_zero_limits() {
    // limits of zero have no order of magnitude: the other one decides
    assert_eq!(ticks::scale_exponent(0.0, 0.0, 3, -1), 0);
    assert_eq!(ticks::scale_exponent(12345.0, 0.0, 3, -1), -4);
    assert_eq!(ticks::scale_exponent(0.0, 12345.0, 3, -1), -4);
}

// ---------------------------------------------------------------------------------------------
// tables

#[test]
fn table_cells_and_columns() {
    // a `{…}` group is one cell, spaces and all; `[index]1` is the second column
    let t = table::Table::parse("name value\n{Palo Alto} 3\n{New {York}} 4 5", table::ColSep::Space, "true", 0, "");
    assert_eq!(t.cell(0, 0), Some("Palo Alto"));
    assert_eq!(t.cell(1, 0), Some("New {York}"));
    assert_eq!(t.column("[index]1"), Some(1));
    assert_eq!(t.column("[index]x"), None);
    // the widest row counts
    assert_eq!(t.columns(), 3);
}

/// A folder of its own that tikra reads files from (with `.`, which other tests read).
fn input_dir() -> std::path::PathBuf {
    static DIR: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("tikra-pgfplots-tables-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("points.dat"), "x y\n0 1\n1 4\n2 9\n").unwrap();
        std::fs::write(dir.join("sub").join("deep.dat"), "a b\n5 6\n").unwrap();
        std::fs::write(dir.join("bad.dat"), [b'x', b' ', 0xff, 0xfe, b'\n']).unwrap();
        std::fs::write(dir.join("faces.dat"), "0 1 2\n").unwrap();
        std::env::set_var("TIKRA_INPUT_DIR", format!("{}:.", dir.display()));
        dir
    })
    .clone()
}

#[test]
fn files_are_found_in_the_input_folders() {
    // without input folders, the current one (the only test of this file that reads files
    // before they are set)
    std::env::set_var("TIKRA_INPUT_DIR", "");
    assert_eq!(table::find_file("Cargo.toml", &[]).unwrap(), std::path::Path::new(".").join("Cargo.toml"));
    let dir = input_dir();
    // `table/search path`: its folders in each input folder; `..` in one is skipped
    let found = table::find_file("deep.dat", &["../elsewhere".into(), "sub".into()]).unwrap();
    assert_eq!(found, dir.join("sub").join("deep.dat"));
    assert_eq!(table::find_file(" ", &[]).unwrap_err(), "no file name");
    // a file that is not text
    let err = table::read_file("bad.dat", &[]).unwrap_err();
    assert!(err.starts_with("file 'bad.dat': invalid utf-8"), "{err}");
    // larger than allowed
    let err = table::read_bytes("points.dat", &[], 4).unwrap_err();
    assert_eq!(err, "file 'points.dat' is larger than 4 bytes");
    // in a figure
    let d = described("", r"\addplot file {points.dat};");
    assert!(d.contains("(0, 1), (1, 4), (2, 9)"), "{d}");
    let (_, w) = describe("", r"\addplot table {bad.dat};");
    assert!(w.iter().any(|w| w.contains("invalid utf-8")), "{w:?}");
    let (_, w) = describe("", r"\addplot file {nothing.dat};");
    assert!(w.iter().any(|w| w == "\\addplot file: file 'nothing.dat' not found"), "{w:?}");
}

#[test]
fn tables_read_with_pgfplotstableread() {
    // kept in a macro, outside the axis or in it, with its own options
    let d = described(
        "",
        r"\pgfplotstableread[col sep=comma]{u,v
          1,2
          3,5}\kept
          \addplot table[x=u, y=v] {\kept};",
    );
    assert!(d.contains("(1, 2), (3, 5)"), "{d}");
    // the macro in braces
    let d = described("", "\\pgfplotstableread{p q\n1 7\n2 8}{\\braced}\n\\addplot table {\\braced};");
    assert!(d.contains("(1, 7), (2, 8)"), "{d}");
    // `table from \macro`
    let d = described("", "\\pgfplotstableread{p q\n1 6\n2 5}\\fromit\n\\addplot table [x=p, y=q] from \\fromit;");
    assert!(d.contains("(1, 6), (2, 5)"), "{d}");
    // no macro to keep it in
    let (_, w) = describe("", "\\pgfplotstableread{p q\n1 6}{}\\addplot coordinates {(0,0) (1,1)};");
    assert!(w.iter().any(|w| w.contains("without a macro")), "{w:?}");
}

/// The box of a document's page, in pt.
fn page(doc: &str) -> [f64; 4] {
    let (c, w) = tikra::engine::run_document(doc).unwrap();
    assert!(w.is_empty(), "{w:?}");
    let (lo, hi) = (c.min.unwrap(), c.max.unwrap());
    [lo.x, lo.y, hi.x, hi.y].map(|v| v / tikra::PT)
}

#[test]
fn a_table_typeset_as_text() {
    // the columns asked for, their names over them, the numbers printed by \pgfmathprintnumber:
    // the `tabular` one would write
    let picture = r"\tikz \draw (0,0) -- (1,0);";
    let typeset = page(&format!("{picture}\\pgfplotstabletypeset[columns={{b,a}}]{{a b c\n1 2.5 x\n3 10000 y\n}}"));
    let written = page(&format!(r"{picture}\begin{{tabular}}{{cc}}b & a\\$2.5$ & $1$\\$10{{,}}000$ & $3$\end{{tabular}}"));
    assert!(typeset.iter().zip(written).all(|(a, b)| (a - b).abs() < 1e-6), "{typeset:?} {written:?}");
    // (a table of other numbers is another size)
    let other = page(&format!(r"{picture}\begin{{tabular}}{{cc}}b & a\\$2.5$ & $1$\\$1$ & $3$\end{{tabular}}"));
    assert!((other[2] - written[2]).abs() > 1.0, "{other:?} {written:?}");
    // all the columns, of a table read with the options; the cells that are no numbers as written
    let typeset = page(&format!("{picture}\\pgfplotstabletypeset[col sep=comma]{{a,b\n7,w\n}}"));
    let written = page(&format!(r"{picture}\begin{{tabular}}{{cc}}a & b\\$7$ & w\end{{tabular}}"));
    assert_eq!(typeset, written);
    // a file that is not there: nothing
    let (_, w) = tikra::engine::run_document(&format!("\\pgfplotstabletypeset{{nothing.dat}} {picture}")).unwrap();
    assert!(w.iter().any(|w| w.contains("nothing.dat' not found")), "{w:?}");
}

#[test]
fn linear_regressions() {
    // no options: x and y are the first two columns; a row without y is left out
    let d = described("", r"\addplot table[y=create col/linear regression, row sep=\\] {x y\\ 0 1\\ 1 3\\ 2 5\\ 3 nan\\};");
    assert!(d.contains("(0, 1), (1, 3), (2, 5)"), "{d}");
    // weighted by `variance list`: the middle point counts a quarter
    let d = described("", r"\addplot table[x index=0, y={create col/linear regression={y={y}, variance list={1,2,1}}}, row sep=\\] {a y\\ 0 1\\ 1 3\\ 2 6\\};");
    assert!(d.contains("(0, 0.944), (1, 3.444), (2, 5.944)"), "{d}");
    // weighted by a column of variances: a point of huge variance does not count
    let d = described("", r"\addplot table[y={create col/linear regression={y=y, variance=s}}, row sep=\\] {x y s\\ 0 0 1\\ 1 1 1\\ 2 9 1000\\};");
    assert!(d.contains("(0, 0), (1, 1), (2, 2)"), "{d}");
    // in logarithms on a log axis: y = 10^x is a line there
    let d = described("ymode=log, log basis y=10", r"\addplot table[y={create col/linear regression={y=y}}, row sep=\\] {x y\\ 0 1\\ 1 10\\ 2 100\\};");
    assert!(d.contains("(0, 1), (1, 10), (2, 100)"), "{d}");
    // a variance given for y itself, on a log axis
    let d = described(
        "",
        r"\addplot table[y={create col/linear regression={y=y, ymode=log, variance format=linear, variance list={1,1,1}}}, row sep=\\] {x y\\ 0 1\\ 1 10\\ 2 100\\};",
    );
    assert!(d.contains("(0, 1), (1, 10), (2, 100)"), "{d}");
    // a column that is not there
    let (_, w) = describe("", r"\addplot table[y={create col/linear regression={y=nope}}, row sep=\\] {x y\\ 0 1\\ 1 2\\};");
    assert!(w.iter().any(|w| w == "create col/linear regression: column not found"), "{w:?}");
}

#[test]
fn this_row_in_expressions() {
    // `\thisrow {name}` with a space, `\thisrowno` of a bare digit, `\coordindex`
    let d = described("", r"\addplot table[x expr=\coordindex, y expr=\thisrow {b}+\thisrowno 0, row sep=\\] {a b\\ 10 1\\ 20 2\\};");
    assert!(d.contains("(0, 11), (1, 22)"), "{d}");
    // `\lineno` is the row's number too
    let d = described("", r"\addplot table[x expr=\lineno, y=b, row sep=\\] {a b\\ 10 1\\ 20 2\\};");
    assert!(d.contains("(0, 1), (1, 2)"), "{d}");
}

#[test]
fn error_bars_of_their_own_columns() {
    // `y error plus` and `y error minus`, `x error plus` and `x error minus`: each side apart
    let d = described(
        "",
        r"\addplot+[error bars/.cd, y dir=both, y explicit, x dir=both, x explicit] table[x=x, y=y, y error plus=p, y error minus=m, x error plus=q, x error minus=n, row sep=\\] {x y p m q n\\ 0 1 0.5 0.25 0.1 0.2\\ 1 2 0.5 0.25 0.1 0.2\\};",
    );
    assert!(d.contains("(0, 1), (1, 2)"), "{d}");
}

#[test]
fn histograms() {
    // the names of `hist/symbolic coords` are the data
    let d = described("", r"\addplot[hist={symbolic coords={a,b,c}, bins=3}] table[y index=0, header=false, row sep=\\] {a\\b\\b\\c\\};");
    assert!(d.contains("3 bins — 0 to 0.667: 1; 0.667 to 1.333: 2; 1.333 to 2: 1"), "{d}");
    // `data value`: an expression of the point
    let d = described("", r"\addplot[hist={bins=2, data value=2*y}] coordinates {(0,0) (0,1) (0,2) (0,2)};");
    assert!(d.contains("2 bins — 0 to 2: 1; 2 to 4: 3"), "{d}");
    // `data=x+y`: another expression
    let d = described("", r"\addplot[hist={bins=2, data=x+y}] coordinates {(0,0) (1,0) (1,1) (2,2)};");
    assert!(d.contains("2 bins — 0 to 2: 2; 2 to 4: 2"), "{d}");
    // cumulative counts, and their share with `density`
    let d = described("", r"\addplot[hist={bins=2, cumulative}] coordinates {(0,0) (0,1) (0,3) (0,4)};");
    assert!(d.contains("2 bins — 0 to 2: 2; 2 to 4: 4"), "{d}");
    let d = described("", r"\addplot[hist={bins=2, cumulative, density}] coordinates {(0,0) (0,1) (0,3) (0,4)};");
    assert!(d.contains("2 bins — 0 to 2: 0.5; 2 to 4: 1"), "{d}");
    // a density: the counts over the total and the bins' width
    let d = described("", r"\addplot[hist={bins=2, density}] coordinates {(0,0) (0,1) (0,3) (0,4)};");
    assert!(d.contains("2 bins — 0 to 2: 0.25; 2 to 4: 0.25"), "{d}");
    // no data: no bins
    let (d, _) = describe("", r"\addplot[hist] coordinates {(0,nan)};");
    assert!(!d.contains("bins"), "{d}");
}
