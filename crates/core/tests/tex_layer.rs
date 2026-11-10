//! The TeX layer on its own (`tikra::tex`): category codes, registers, conditionals, the e-TeX and
//! LaTeX commands it knows, and what it does with input it cannot read. Each test runs a few lines
//! of TeX and checks what TeX would typeset from them.

use std::hash::Hasher;

use tikra::tex::{dimen, Cat, FxHasher, Meaning, Prim, Tex, Token, MAX_EXPANSIONS};

/// Runs `text` in `t` and returns what TeX would typeset: the characters left after expansion and
/// assignments (a command that does nothing here is shown by name), or the error.
fn run_in(t: &mut Tex, text: &str) -> String {
    t.push_text(text);
    let mut out = String::new();
    loop {
        let x = match t.get_x_token() {
            Ok(Some(x)) => x,
            Ok(None) => break,
            Err(e) => return format!("error: {e}"),
        };
        match &x {
            Token::Char(c, Cat::Letter | Cat::Other | Cat::Space | Cat::Math | Cat::Sup | Cat::Sub | Cat::Align) => out.push(*c),
            _ => match t.execute_basic(x.clone()) {
                Ok(true) => {}
                Ok(false) => out.push_str(&tikra::tex::show_token(&x)),
                Err(e) => return format!("error: {e}"),
            },
        }
    }
    out
}

fn run(text: &str) -> String {
    run_in(&mut Tex::new(), text)
}

/// What `text` typesets, and the warnings it gave.
fn run_warned(text: &str) -> (String, Vec<String>) {
    let mut t = Tex::new();
    let out = run_in(&mut t, text);
    (out, t.warnings)
}

// ---------------------------------------------------------------- category codes and reading

#[test]
fn every_category_code_can_be_set_and_read_back() {
    for n in 0..=15 {
        assert_eq!(run(&format!(r"\catcode 200={n} \the\catcode 200")), n.to_string(), "category {n}");
    }
    // a category that does not exist leaves the character as it was
    assert_eq!(run(r"\catcode`\!=16 \the\catcode`\!"), "12");
    // as an integer, \catcode is a number to compute with
    assert_eq!(run(r"\count1=\catcode`\a \advance\count1 by 1 \the\count1"), "12");
}

#[test]
fn the_default_category_codes_of_special_characters() {
    // the null character is ignored, ~ is active, delete is invalid
    assert_eq!(run(r"\the\catcode 0/\the\catcode`\~/\the\catcode 127"), "9/13/15");
    // an ignored or invalid character is dropped from the text
    assert_eq!(run("a\u{0}b\u{7f}c"), "abc");
}

#[test]
fn a_changed_category_code_changes_how_the_text_reads() {
    // ! as a letter is part of a control word
    assert_eq!(run(r"\catcode`\!=11 \def\a!{X}\a!"), "X");
    // a category code set in a group is restored after it; a global one stays
    assert_eq!(run(r"{\catcode`\!=11 }\the\catcode`\!"), "12");
    assert_eq!(run(r"{\global\catcode`\!=11 }\the\catcode`\!"), "11");
}

#[test]
fn active_characters_are_defined_like_macros() {
    assert_eq!(run(r"\catcode`\!=13 \def!{X}a!b"), "aXb");
    // locally in a group, globally with \gdef
    assert_eq!(run(r"\catcode`\!=13 \def!{X}{\def!{Y}!}!"), "YX");
    assert_eq!(run(r"\catcode`\!=13 \def!{X}{\gdef!{Z}}!"), "Z");
    // an active character with no meaning is undefined
    let mut t = Tex::new();
    assert_eq!(t.meaning(&Token::Char('!', Cat::Active)), Meaning::Undefined);
    run_in(&mut t, r"\catcode`\!=13 \let!\relax");
    assert_eq!(t.meaning(&Token::Char('!', Cat::Active)), Meaning::Prim(Prim::Relax));
}

#[test]
fn superscript_pairs_name_a_character() {
    // ^^ and a character: the one 64 positions away
    assert_eq!(run("a^^:b"), "azb");
    assert_eq!(run(r"\the\catcode`^^:"), "11");
}

#[test]
fn line_ends_of_every_kind() {
    // a carriage return and line feed is one end of line, read as a space
    assert_eq!(run("a\r\nb"), "a b");
    assert_eq!(run("a\r\n\r\nb"), "ab");
}

#[test]
fn an_escape_character_at_the_end_is_the_empty_control_sequence() {
    // the control sequence with an empty name, the one \csname\endcsname makes
    assert_eq!(run("\\expandafter\\def\\csname\\endcsname{E}x\\"), "xE");
}

#[test]
fn macros_defined_from_source_keep_the_category_codes_of_the_text() {
    // `define_text` reads with @ a letter, and gives @ back the category it had
    let mut t = Tex::new();
    assert_eq!(run_in(&mut t, r"\makeatother"), "");
    t.define_text(r"\def\a@b{X}");
    assert_eq!(run_in(&mut t, r"\the\catcode`\@"), "12");
    assert_eq!(run_in(&mut t, r"\makeatletter\a@b"), "X");
}

#[test]
fn input_left_to_read() {
    let mut t = Tex::default();
    t.push_text("a");
    assert!(t.has_input());
    assert_eq!(t.get_token(), Some(Token::Char('a', Cat::Letter)));
    assert!(!t.has_input());
}

#[test]
fn a_token_put_back_is_not_read_yet() {
    // the source map: a statement that reads a token and puts it back has read nothing
    let mut t = Tex::new();
    t.note_reading();
    t.push_text("ab");
    let a = t.get_token().unwrap();
    assert_eq!(t.statement_span(), Some((0, 1)));
    t.back(a);
    assert_eq!(t.statement_span(), Some((0, 0)));
    assert_eq!(t.get_token(), Some(Token::Char('a', Cat::Letter)));
    assert_eq!(t.statement_span(), Some((0, 1)));
}

#[test]
fn the_hasher_reads_a_size_as_a_64_bit_number() {
    let (mut a, mut b) = (FxHasher::default(), FxHasher::default());
    a.write_usize(12345);
    b.write_u64(12345);
    assert_eq!(a.finish(), b.finish());
    assert_ne!(a.finish(), FxHasher::default().finish());
}

// ---------------------------------------------------------------- registers

#[test]
fn token_registers() {
    assert_eq!(run(r"\toks0={a b}\the\toks0"), "a b");
    assert_eq!(run(r"\toks3={x}{\toks3={y}\the\toks3}\the\toks3"), "yx");
    assert_eq!(run(r"\toks3={x}{\global\toks3={g}}\the\toks3"), "g");
    // by name: \toksdef, \newtoks
    assert_eq!(run(r"\toksdef\t=5 \t={five}\the\t\the\toks5"), "fivefive");
    assert_eq!(run(r"\newtoks\t \t{new}\the\t"), "new");
    assert_eq!(run(r"\newtoks\t [\the\t]"), "[]");
    // \the of a register is not expanded again inside \edef
    assert_eq!(run(r"\def\m{M}\toks0={\m}\edef\e{\the\toks0 }\def\m{N}\e"), "N");
}

#[test]
fn dimension_and_skip_registers() {
    assert_eq!(run(r"\dimen3=2pt \the\dimen3"), "2.0pt");
    // glue keeps its natural size (stretch and shrink are read and dropped)
    assert_eq!(run(r"\skip3=2pt plus1fil minus1pt\dimen1=\skip3 \the\dimen1"), "2.0pt");
    assert_eq!(run(r"\skip3=1pt plus2fill\relax\the\skip3"), "1.0pt");
    assert_eq!(run(r"\dimendef\d=7 \d=3pt \the\d/\the\dimen7"), "3.0pt/3.0pt");
    assert_eq!(run(r"\skipdef\s=8 \s=4pt plus1pt\relax\the\s"), "4.0pt");
    assert_eq!(run(r"\countdef\c=9 \c=12 \the\c/\the\count9"), "12/12");
    assert_eq!(run(r"\newskip\s \s=5pt\relax\the\s"), "5.0pt");
    // a dimension read as a number is in scaled points
    assert_eq!(run(r"\dimen3=1pt \count1=\dimen3 \the\count1/\number\dimen3"), "65536/65536");
    assert_eq!(run(r"\skipdef\s=8 \s=2sp\relax\number\s"), "2");
    // registers named by number inside a number or a length
    assert_eq!(run(r"\count4=6 \dimen4=1sp \count1=\count4 \dimen5=\dimen4 \the\count1/\number\dimen5"), "6/1");
    assert_eq!(run(r"\number\dimexpr 1sp*3\relax/\number\numexpr 2*3\relax"), "3/6");
    assert_eq!(run(r"\dimen3=\dimexpr 2pt+1pt\relax\the\dimen3"), "3.0pt");
    assert_eq!(run(r"\chardef\c=7 \the\c"), "7");
}

#[test]
fn signs_before_numbers_and_lengths() {
    assert_eq!(run(r"\count1=+-+5 \the\count1"), "-5");
    assert_eq!(run(r"\dimen1=+-+2pt \the\dimen1"), "-2.0pt");
}

#[test]
fn numbers_in_other_bases_and_characters() {
    assert_eq!(run(r#"\number"1F/\number'17/\number`\a/\number`\\"#), "31/15/97/92");
    // a number made by a macro is read on
    assert_eq!(run(r"\def\two{2}\number1\two3"), "123");
}

#[test]
fn advance_multiply_and_divide_by_register_number() {
    assert_eq!(run(r"\dimen5=2pt \advance\dimen5 by 1pt \the\dimen5"), "3.0pt");
    assert_eq!(run(r"\skip5=2pt \multiply\skip5 by 3 \the\skip5"), "6.0pt");
    assert_eq!(run(r"\count5=7 \divide\count5 by 0"), "error: figure: arithmetic overflow (division by 0)");
    assert!(run(r"\advance\relax by 1").starts_with("error: figure: you can't use \\relax after \\advance"));
}

#[test]
fn the_of_everything_that_has_a_value() {
    assert_eq!(run(r"\the\catcode`\{"), "1");
    assert_eq!(run(r"\setbox0=\hbox{}\wd0=2pt \ht0=3pt \dp0=1pt \the\wd0/\the\ht0/\the\dp0"), "2.0pt/3.0pt/1.0pt");
    assert_eq!(run(r"\setbox0=\hbox{}\dp0=1pt \count1=\dp0 \the\count1/\number\ht0"), "65536/0");
    // what has no value: nothing, and a warning
    let (out, w) = run_warned(r"[\the\relax]");
    assert_eq!(out, "[]");
    assert!(w.iter().any(|w| w == r"\the\relax is not supported"), "{w:?}");
}

#[test]
fn register_numbers_out_of_range() {
    assert_eq!(run(r"\count70000=1"), "error: figure: bad register number 70000");
}

#[test]
fn a_length_without_a_value() {
    // a length where a value is (`.6\linewidth` in options tikra does not know): nothing assigned
    let (out, w) = run_warned(r"\newdimen\d \d=2pt \d]x\the\d");
    assert_eq!(out, "]x2.0pt");
    assert!(w.iter().any(|w| w == r"\d: no length to assign: ignored"), "{w:?}");
}

// ---------------------------------------------------------------- expansion

#[test]
fn expandafter_and_noexpand_at_the_edges() {
    // \expandafter before a token that does not expand: both are read as they are
    assert_eq!(run(r"\def\a{A}\expandafter\a\relax x"), "Ax");
    // with nothing after it
    assert_eq!(run(r"\def\a{A}\expandafter\a"), "A");
    assert_eq!(run(r"x\noexpand"), "x");
}

#[test]
fn csname_builds_a_name_from_characters() {
    // a name too long to be one stops with an error
    let long = "a".repeat(10_001);
    assert_eq!(run(&format!(r"\csname {long}\endcsname")), "error: figure: \\csname too long");
}

#[test]
fn meaning_shows_what_a_command_is() {
    for (text, shown) in [
        (r"\def\a#1.#2{x#1y#2}\meaning\a", "macro:#1.#2->x#1y#2"),
        (r"\meaning 1", "the character 1"),
        (r"\meaning\nothing", "undefined"),
        (r"\meaning\relax", "\\relax"),
        (r"\meaning\expandafter", "\\expandafter"),
        (r"\countdef\c=5 \meaning\c", "\\count5"),
        (r"\dimendef\c=5 \meaning\c", "\\dimen5"),
        (r"\skipdef\c=5 \meaning\c", "\\skip5"),
        (r"\toksdef\c=5 \meaning\c", "\\toks5"),
        (r"\chardef\c=255 \meaning\c", "\\char\"FF"),
    ] {
        assert_eq!(run(text), shown, "{text}");
    }
}

#[test]
fn jobname_and_string_comparison() {
    assert_eq!(run(r"\jobname"), "figure");
    // \pdfstrcmp expands both texts and compares them: -1, 0 or 1
    assert_eq!(run(r"\def\b{b}\pdfstrcmp{a}{\b}/\pdfstrcmp{\b}{b}/\strcmp{c}{\b}"), "-1/0/1");
    assert_eq!(run(r"\pdfstrcmp a{b}"), "error: figure: missing '{'");
    assert_eq!(run(r"\pdfstrcmp\relax{a}{a}"), "0");
}

#[test]
fn expansion_limits() {
    // over the cap of all expansions of a figure
    let mut t = Tex::new();
    t.expansions = 10 * MAX_EXPANSIONS;
    assert!(run_in(&mut t, r"\def\a{}\a").starts_with("error: figure: figure too large"));
    // a token that does not expand is left alone by `expand`
    let mut t = Tex::new();
    t.push_text("b");
    t.expand(&Token::cs("relax"), &Meaning::Prim(Prim::Relax)).unwrap();
    t.expand(&Token::cs("count"), &Meaning::Count(3)).unwrap();
    assert_eq!(t.get_token(), Some(Token::Char('b', Cat::Letter)));
    assert_eq!(t.get_token(), None);
}

#[test]
fn arguments_that_do_not_match_or_do_not_end() {
    // a delimiter that is not there
    assert_eq!(run(r"\def\a.#1{}\a x"), "error: figure: use of \\a does not match its definition");
    // an optional argument with nothing after the command: its default
    assert_eq!(run(r"\newcommand\o[1][d]{<#1>}\o"), "<d>");
    // arguments too long to be read
    let long = "a".repeat(1_000_001);
    assert_eq!(run(&format!(r"\def\a#1{{}}\a{{{long}}}")), "error: figure: argument too long");
    assert_eq!(run(&format!(r"\def\a#1.{{}}\a {long}.")), "error: figure: \\a ends before its delimiter");
    assert_eq!(run(&format!(r"\newcommand\o[1][{long}]{{}}")), "error: figure: unmatched bracket: ']' missing");
    assert_eq!(run(&format!(r"\NewDocumentCommand\f{{r()}}{{}}\f({long})")), "error: figure: ')' missing");
    // an expansion too long: a macro that doubles its text
    assert!(run(r"\def\a#1{\a{#1#1}}\edef\b{\a x}").starts_with("error"));
}

#[test]
fn parameter_texts() {
    // #{: the parameter text ends at the brace, which begins the body
    assert_eq!(run(r"\def\a#1#{[#1]}x"), "x");
    // # before anything else in a body is a parameter character
    assert_eq!(run(r"\def\a{\def\b##1{(##1)}}\a\b q"), "(q)");
}

// ---------------------------------------------------------------- conditionals

#[test]
fn modes() {
    // in a figure's text, TeX is in horizontal mode, not in math, not inner
    assert_eq!(run(r"\ifvmode V\fi\ifhmode H\fi\ifmmode M\fi\ifinner I\fi"), "H");
}

#[test]
fn if_and_ifcat_with_control_sequences() {
    // a control sequence that is not a character counts as the same character and category
    assert_eq!(run(r"\if\relax\relax T\else F\fi"), "T");
    assert_eq!(run(r"\ifcat\relax\relax T\else F\fi"), "T");
    assert_eq!(run(r"\ifcat abT\else F\fi\ifcat a1T\else F\fi"), "TF");
    // \let to a character compares as that character
    assert_eq!(run(r"\let\x=a\if\x aT\else F\fi"), "T");
    // \noexpand: the control sequence itself, not its expansion
    assert_eq!(run(r"\def\a{b}\if\noexpand\a b T\else F\fi\if\a bT\else F\fi"), "FT");
}

#[test]
fn ifx_compares_meanings() {
    assert_eq!(run(r"\ifx\relax\relax T\else F\fi\ifx aaT\else F\fi\ifx\relax a T\else F\fi"), "TTF");
    assert_eq!(run(r"\let\r\relax\ifx\r\relax T\else F\fi"), "T");
}

#[test]
fn comparisons_need_a_relation() {
    assert_eq!(run(r"\ifnum 1!2 T\fi"), "error: figure: missing = < or > in a comparison");
}

#[test]
fn conditionals_nested_in_a_skipped_branch() {
    assert_eq!(run(r"\iftrue a\else \ifnum1=1 b\fi c\fi d"), "ad");
    // an \ifcase whose case is past the last \or, without \else
    assert_eq!(run(r"\ifcase 3 a\or b\fi c"), "c");
    assert_eq!(run(r"\iffalse a\or b\fi"), "error: figure: \\or outside \\ifcase");
    assert_eq!(run(r"\unless\ifcase 1 a\fi"), "error: figure: \\unless must be followed by a conditional");
}

#[test]
fn an_extra_else_or_fi_is_a_warning() {
    let (out, w) = run_warned(r"a\else b\fi c\or d");
    assert_eq!(out, "abcd");
    assert_eq!(w, ["extra \\else", "extra \\fi", "extra \\or"]);
}

#[test]
fn ifcsname_builds_its_name_as_csname_does() {
    assert_eq!(run(r"\def\n{relax}\ifcsname\n\endcsname T\else F\fi"), "T");
}

// ---------------------------------------------------------------- numbers and lengths

#[test]
fn missing_numbers_and_units() {
    assert_eq!(run(r"\count1=x"), "error: figure: missing number");
    assert_eq!(run(r"\count1=\relax"), "error: figure: missing number");
    assert_eq!(run(r"\dimen1=3xx"), "error: figure: illegal unit of measure (pt inserted)");
    assert_eq!(run(r"\dimen1=3"), "error: figure: illegal unit of measure (pt inserted)");
    assert_eq!(run(r"\dimen1=3t"), "error: figure: illegal unit of measure (pt inserted)");
}

#[test]
fn numexpr_and_dimexpr() {
    assert_eq!(run(r"\the\numexpr 7-2*3\relax/\the\numexpr -(2+3)*2\relax/\the\numexpr(1+2\relax"), "1/-10/3");
    assert_eq!(run(r"\the\numexpr 1/0\relax"), "error: figure: division by 0 in \\numexpr");
    // without \relax, the expression ends where it cannot go on
    assert_eq!(run(r"\the\numexpr 2+3 x/\the\dimexpr 1pt+1pt x"), "5x/2.0ptx");
    assert_eq!(run(r"\count1=\numexpr 4\relax\the\count1"), "4");
    // on its own, an expression is read and dropped
    assert_eq!(run(r"a\numexpr 1+2\relax b\dimexpr 1pt\relax c\glueexpr 1pt\relax d"), "abcd");
}

// ---------------------------------------------------------------- assignments and other commands

#[test]
fn after_assignment_and_after_group() {
    assert_eq!(run(r"\def\x{X}\afterassignment\x\count1=5 y"), "Xy");
    // \aftergroup at the end of the input
    assert_eq!(run(r"a\aftergroup"), "a");
}

#[test]
fn ignorespaces_skips_the_spaces_that_follow() {
    assert_eq!(run(r"a\ignorespaces   b"), "ab");
    assert_eq!(run(r"a\def\s{ }\ignorespaces\s\s b"), "ab");
}

#[test]
fn protected_definitions() {
    // a protected macro is not expanded inside \edef
    assert_eq!(run(r"\protected\def\p{P}\edef\e{\p}\def\p{Q}\e"), "Q");
    assert_eq!(run(r"\protected\long\def\p{P}\edef\e{\p}\def\p{Q}\e"), "Q");
    assert_eq!(run(r"{\protected\global\def\p{P}}\p"), "P");
    assert_eq!(run(r"\protected\edef\p{P}\p"), "P");
    // before anything else it does nothing
    assert_eq!(run(r"\protected x"), "x");
}

#[test]
fn let_with_spaces_and_equals() {
    assert_eq!(run(r"\def\a{A}\let\b  \a\b"), "A");
    assert_eq!(run(r"\def\a{A}\let\b= \a\b"), "A");
}

#[test]
fn futurelet_looks_ahead() {
    assert_eq!(run(r"\def\t{\ifx\n x(x)\fi}\futurelet\n\t x"), "(x)x");
}

#[test]
fn an_extra_group_end_is_a_warning() {
    let (out, w) = run_warned(r"a}b\endgroup c");
    assert_eq!(out, "abc");
    assert_eq!(w, ["extra '}'", "extra \\endgroup"]);
}

#[test]
fn messages_are_read_and_dropped() {
    assert_eq!(run(r"a\message{m}\immediate\write16{w}\typeout{t}\errmessage{e}\showtokens{s}\show\x\showthe\count1 b"), "a1 b");
}

#[test]
fn case_changes() {
    assert_eq!(run(r"\uppercase{aB1\relax}\lowercase{Cd}"), "AB1cd");
}

#[test]
fn ifthen_booleans_and_lengths() {
    assert_eq!(run(r"\newboolean{b}\setboolean{b}{true}\ifthenelse{\boolean{b}}{T}{F}\setboolean{b}{False}\ifthenelse{\boolean{b}}{T}{F}"), "TF");
    // \boolean outside \ifthenelse is the boolean's \if
    assert_eq!(run(r"\newboolean{b}\boolean{b}T\else F\fi"), "F");
    assert_eq!(run(r"\ifthenelse{\lengthtest{1pt<2pt}}{T}{F}\ifthenelse{\lengthtest{1cm>2cm}}{T}{F}\ifthenelse{\lengthtest{10mm=1cm}}{T}{F}"), "TFT");
    assert_eq!(run(r"\ifthenelse{\lengthtest{1pt}}{T}{F}"), "F");
    // \or and \and that are all false and not all true
    assert_eq!(run(r"\ifthenelse{1=2 \or 2=3}{T}{F}\ifthenelse{1=1 \and 1=2}{T}{F}"), "FF");
    // \isodd of a single token, \equal of groups in groups
    assert_eq!(run(r"\ifthenelse{\isodd 7}{T}{F}\ifthenelse{\equal{{a}b}{{a}b}}{T}{F}"), "TT");
}

#[test]
fn etoolbox_string_comparison() {
    assert_eq!(run(r"\ifstrequal{abc}{abc}{T}{F}\ifstrequal{abc}{abd}{T}{F}"), "TF");
}

#[test]
fn latex_utilities() {
    assert_eq!(run(r"\makeatletter\@gobble{x}y\@gobbletwo abc\@firstofone{d}"), "ycd");
    // an undefined counter reads as 0; setting it does nothing
    assert_eq!(run(r"\setcounter{none}{3}\arabic{none}/\number\value{none}"), "0/0");
    // \newcounter with a parent counter, which is read and not used
    assert_eq!(run(r"\newcounter{p}\newcounter{c}[p]\setcounter{c}{2}\thec"), "2");
    assert_eq!(run(r"\newcounter{c}x\thec"), "x0");
}

#[test]
fn setlength_of_something_that_is_not_a_length() {
    let (out, w) = run_warned(r"\setlength{\relax}{2pt}x");
    assert_eq!(out, "x");
    assert_eq!(w, ["\\setlength: \\relax is not a length"]);
}

#[test]
fn boxes_are_measured_as_empty() {
    // \setbox with a number, from another box, or from something else
    assert_eq!(run(r"\setbox2=\hbox{ab}\wd2=1pt \the\wd2"), "1.0pt");
    assert_eq!(run(r"\setbox2=\box3 \setbox2\copy\x \the\wd2"), "0.0pt");
    assert_eq!(run(r"\setbox2=x"), "x");
    assert_eq!(run(r"\setbox"), "");
    assert_eq!(run(r"\newbox\b\setbox\b=\box"), "");
    // a box named by a control sequence keeps what was put in it
    let mut t = Tex::new();
    run_in(&mut t, r"\setbox\partbox=\hbox{low}");
    assert_eq!(t.named_boxes.len(), 1);
    assert_eq!(t.named_boxes[0].0, "partbox");
    // \hbox to a width: its content
    assert_eq!(run(r"\hbox to 2pt{a}\hbox{b}"), "ab");
}

#[test]
fn newcommand_and_newenvironment_forms() {
    // a number of arguments not followed by a default
    assert_eq!(run(r"\newcommand\c[1]{<#1>}\c x"), "<x>");
    assert_eq!(run(r"\newcommand\c{C}"), "");
    let mut t = Tex::new();
    run_in(&mut t, r"\newenvironment*{e}[2][d]{<#1|#2>}{.}\newenvironment{f}[1]{(#1)}{.}\newenvironment{g}{G}{.}");
    t.push_text("{x}");
    assert_eq!(t.environment_begin("e").unwrap().map(|v| tikra::tex::tokens_to_string(&v)), Some("<d|x>".into()));
    t.push_text("{y}");
    assert_eq!(t.environment_begin("f").unwrap().map(|v| tikra::tex::tokens_to_string(&v)), Some("(y)".into()));
    assert_eq!(t.environment_begin("g").unwrap().map(|v| tikra::tex::tokens_to_string(&v)), Some("G".into()));
    assert_eq!(t.environment_begin("h").unwrap(), None);
}

#[test]
fn document_commands_with_every_kind_of_argument() {
    // `>{…}` processors change nothing; a required argument that is missing is its default, with
    // a warning
    let (out, w) = run_warned(r"\NewDocumentCommand\f{>{\foo} m R(){z}}{#1/#2}\f{a}(b)\f{c}d");
    assert_eq!(out, "a/bc/zd");
    assert_eq!(w, ["\\f is missing its argument in '()'"]);
    // an argument type tikra does not know is read as `m`
    let (out, w) = run_warned(r"\NewDocumentCommand\g{v}{[#1]}\g{x}");
    assert_eq!(out, "[x]");
    assert_eq!(w, ["document command argument type 'v' not supported: read as 'm'"]);
    // \IfNoValueF and \IfValueF
    assert_eq!(run(r"\NewDocumentCommand\h{o}{\IfNoValueF{#1}{<#1>}\IfValueF{#1}{-}}\h\h[q]"), "-<q>");
    // a default in groups in groups
    assert_eq!(run(r"\NewDocumentCommand\k{O{{a}b}}{#1}\k"), "ab");
}

#[test]
fn document_commands_defined_through_an_active_character() {
    assert_eq!(run(r"\catcode`\!=13 \let!\NewDocumentCommand !\f{m}{<#1>}\f{x}"), "<x>");
}

#[test]
fn text_for_reading_again_keeps_spaces_after_control_words() {
    let letter = |c| Token::Char(c, Cat::Letter);
    let v = [Token::cs("o"), Token::Char(' ', Cat::Space), letter('x'), Token::cs("o"), letter('y'), Token::cs("o"), Token::other('1')];
    assert_eq!(tikra::tex::tokens_to_text(&v), r"\o{} x\o y\o1");
}

#[test]
fn roman_numerals_of_nothing() {
    assert_eq!(run(r"[\romannumeral 0][\romannumeral -5]"), "[][]");
}

/// The tokens of `text`, read without expanding anything.
fn tokens(text: &str) -> Vec<Token> {
    let mut t = Tex::new();
    t.push_text(text);
    std::iter::from_fn(|| t.get_token()).collect()
}

#[test]
fn expressions_cut_short_by_the_end_of_the_input() {
    for (text, value) in [(r"\count1=\numexpr 2+3", "5"), (r"\count1=\numexpr(2+3", "5"), (r"\dimen1=\dimexpr 2pt", "2.0pt"), (r"\skip1=1pt plus1", "1.0pt")] {
        let mut t = Tex::new();
        assert_eq!(run_in(&mut t, text), "", "{text}");
        let reg = if text.contains("count") {
            r"\the\count1"
        } else if text.contains("skip") {
            r"\the\skip1"
        } else {
            r"\the\dimen1"
        };
        assert_eq!(run_in(&mut t, reg), value, "{text}");
    }
}

#[test]
fn an_expansion_too_long() {
    // a macro that doubles its argument, 21 times over: two million characters
    let mut text = String::from(r"\def\a#1{#1#1}\edef\b{");
    text.push_str(&r"\a{".repeat(21));
    text.push('x');
    text.push_str(&"}".repeat(22));
    assert_eq!(run(&text), "error: figure: expansion too long");
}

#[test]
fn expanding_piece_by_piece_drops_what_is_left_after_an_error() {
    let mut t = Tex::new();
    t.push_text("after");
    let depth = t.expand_start(tokens(r"\number x rest"));
    assert!(t.expand_more(depth, &[]).is_err());
    assert_eq!(t.depth(), depth);
    assert_eq!(t.get_token(), Some(Token::Char('a', Cat::Letter)));
}

#[test]
fn execute_basic_expands_what_expands_and_leaves_the_rest() {
    let mut t = Tex::new();
    run_in(&mut t, r"\def\m{M}");
    assert!(t.execute_basic(Token::cs("m")).unwrap());
    assert_eq!(t.get_token(), Some(Token::Char('M', Cat::Letter)));
    // an undefined command is not TeX's: it is left to the drawing layer
    assert!(!t.execute_basic(Token::cs("draw")).unwrap());
    assert_eq!(run(r"\nothing x"), r"\nothingx");
}

#[test]
fn let_reads_spaces_before_the_value() {
    assert_eq!(run(r"\def\a{A}\def\sp{ }\expandafter\let\expandafter\b\sp\a\b"), "A");
}

#[test]
fn names_with_control_sequences_in_them() {
    // TeX stops with "Missing \endcsname" here; tikra keeps the control sequence's name in the name
    assert_eq!(run(r"\expandafter\def\csname a\relax\endcsname{Q}\csname a\relax\endcsname"), "Q");
    assert_eq!(run(r"\expandafter\def\csname a\relax\endcsname{Q}\ifcsname a\relax\endcsname T\else F\fi"), "T");
}

#[test]
fn malformed_parameters_and_specifications() {
    // TeX stops with an error on these; tikra reads on: `#` and what follows it is part of the
    // parameter text (without the `#`), or of the body
    assert_eq!(run(r"\def\a#x{A}\a x"), "A");
    assert_eq!(run(r"\def\a{#x}\a"), "#x");
    // a specification with a control sequence in it, an `O` without its default, a `>` without
    // its processor
    assert_eq!(run(r"\NewDocumentCommand\f{m\relax O >m}{[#1|#2|#3]}\f{a}{b}"), "[a||b]");
}

#[test]
fn definitions_cut_short_by_the_end_of_the_input() {
    assert_eq!(run(r"\newcommand\c[1]"), "error: figure: a command is missing its argument");
    assert_eq!(run(r"\newcommand\c"), "error: figure: a command is missing its argument");
    assert_eq!(run(r"\newenvironment{e}[1]"), "error: figure: a command is missing its argument");
    assert_eq!(run(r"\newenvironment{e}"), "error: figure: a command is missing its argument");
    assert_eq!(run(r"\newcounter{c}"), "");
}

#[test]
fn ifthen_tests_with_nothing_to_compare() {
    // an empty test, and an \equal with one text, are false
    assert_eq!(run(r"\ifthenelse{}{T}{F}\ifthenelse{\equal{a}}{T}{F}"), "FF");
}

#[test]
fn registers_of_every_kind_read_as_numbers() {
    assert_eq!(run(r"\newdimen\d \d=1sp \count1=\d \the\count1"), "1");
    assert_eq!(run(r"\count7=3 \dimen7=2sp \count1=\count7 \advance\count1 by \dimen7 \the\count1"), "5");
    assert_eq!(run(r"\count1=\dimexpr 3sp\relax \the\count1"), "3");
    assert_eq!(run(r"\dimen1=\dimen7 \dimen7=1pt \dimen1=\dimen7 \the\dimen1"), "1.0pt");
}

#[test]
fn a_command_of_the_drawing_layer_means_relax() {
    // as `\meaning` shows a primitive TeX does not know of, unexpandable
    let doc = r"\begin{tikzpicture}\ifnum\pdfstrcmp{\meaning\draw}{\string\relax}=0 \draw (0,0) -- (1,0);\else\draw (0,0) -- (0,1);\fi\end{tikzpicture}";
    let (c, w) = tikra::engine::run_document(doc).unwrap();
    assert!(w.is_empty(), "{w:?}");
    let (min, max) = (c.min.unwrap(), c.max.unwrap());
    assert!(max.x - min.x > 0.9 && max.y - min.y < 0.1, "{min:?} {max:?}");
}

// ---------------------------------------------------------------- units and xstring

#[test]
fn units_of_every_kind() {
    for (text, shown) in [
        (r"\dimen1=7200bp \the\dimen1", "7227.0pt"),
        (r"\dimen1=72bp \the\dimen1", "72.26999pt"),
        (r"\dimen1=7200px \the\dimen1", "7227.0pt"),
        (r"\dimen1=1157dd \the\dimen1", "1238.0pt"),
        (r"\dimen1=1157cc \the\dimen1", "14856.0pt"),
        (r"\dimen1=65536sp \the\dimen1", "1.0pt"),
    ] {
        assert_eq!(run(text), shown, "{text}");
    }
    // a unit TeX does not know has no size
    assert_eq!(dimen::exact(1, &[], "furlong"), None);
    // scaled points and centimeters
    assert!((dimen::to_cm(1_864_680) - 1.0).abs() < 1e-6);
    assert_eq!(dimen::from_cm(1.0), 1_864_680);
    assert_eq!(dimen::from_cm(1e9), dimen::MAX_DIMEN);
}

#[test]
fn xstring_counts_groups_in_groups_as_one_character() {
    assert_eq!(run(r"\StrLen{a{b{c}}d}"), "3");
}
