use super::*;

/// Runs `text` and returns what TeX would typeset: the characters left after expansion and assignments.
fn run(text: &str) -> String {
    let mut t = Tex::new();
    run_in(&mut t, text)
}

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
                Ok(false) => out.push_str(&show_token(&x)),
                Err(e) => return format!("error: {e}"),
            },
        }
    }
    out
}

#[test]
fn macros_with_parameters() {
    assert_eq!(run(r"\def\a#1#2{(#2,#1)}\a xy"), "(y,x)");
    assert_eq!(run(r"\def\a#1{[#1]}\a{hello world}"), "[hello world]");
    assert_eq!(run(r"\def\pair(#1,#2){#1+#2}\pair(3,4)"), "3+4");
    // braces around a whole delimited argument go; around a part they stay (and are a group when run)
    assert_eq!(run(r"\def\a#1.{<#1>}\a{x}.\a{y}z."), "<x><yz>");
    assert_eq!(run(r"\def\a#1.{\detokenize{#1}}\a{x}.\a{y}z."), "x{y}z");
    // a nested definition with ##
    assert_eq!(run(r"\def\outer#1{\def\inner##1{#1##1}}\outer a\inner b"), "ab");
}

#[test]
fn spaces_lines_and_comments() {
    assert_eq!(run("a  b"), "a b");
    assert_eq!(run("\\def\\x{X}\\x y"), "Xy");
    assert_eq!(run("a% comment\n   b"), "ab");
    assert_eq!(run("a\nb"), "a b");
    assert_eq!(run("a\n\nb"), "ab");
}

#[test]
fn groups_and_global() {
    assert_eq!(run(r"\def\a{1}{\def\a{2}\a}\a"), "21");
    assert_eq!(run(r"\def\a{1}{\gdef\a{2}}\a"), "2");
    assert_eq!(run(r"\def\a{1}{\global\def\a{2}\def\a{3}}\a"), "2");
    assert_eq!(run(r"\def\a{1}\begingroup\def\a{2}\endgroup\a"), "1");
    assert_eq!(run(r"\def\x{X}{\aftergroup\x}y"), "Xy");
}

#[test]
fn edef_let_and_expansion_control() {
    assert_eq!(run(r"\def\a{A}\edef\b{\a\a}\def\a{Z}\b"), "AA");
    assert_eq!(run(r"\def\a{A}\edef\b{\noexpand\a}\def\a{Z}\b"), "Z");
    assert_eq!(run(r"\def\a{A}\edef\b{\unexpanded{\a}}\def\a{Z}\b"), "Z");
    assert_eq!(run(r"\def\a{A}\let\b\a\def\a{Z}\b\a"), "AZ");
    assert_eq!(run(r"\def\a#1{[#1]}\def\b{x}\expandafter\a\b"), "[x]");
    assert_eq!(run(r"\expandafter\def\csname my macro\endcsname{M}\csname my macro\endcsname"), "M");
    assert_eq!(run(r"\string\foo"), "\\foo");
    assert_eq!(run(r"\detokenize{\a b}"), "\\a b");
}

#[test]
fn registers_and_arithmetic() {
    assert_eq!(run(r"\count255=7 \advance\count255 by 3 \the\count255"), "10");
    assert_eq!(run(r"\newcount\n \n=4 \multiply\n by 5 \divide\n 3 \the\n"), "6");
    assert_eq!(run(r"\newdimen\d \d=1.5pt \advance\d by 2pt \the\d"), "3.5pt");
    assert_eq!(run(r"\newdimen\d \d=1cm \the\d"), "28.45274pt");
    assert_eq!(run(r"\newlength\l\setlength{\l}{2pt}\addtolength{\l}{1pt}\the\l"), "3.0pt");
    assert_eq!(run(r"\the\numexpr 2*(3+4)\relax"), "14");
    assert_eq!(run(r"\the\numexpr 7/2\relax"), "4");
    assert_eq!(run(r"\the\dimexpr 1pt*3\relax"), "3.0pt");
    assert_eq!(run(r"\number`a"), "97");
    assert_eq!(run(r"\romannumeral 14"), "xiv");
    assert_eq!(run(r"\newcounter{c}\setcounter{c}{3}\stepcounter{c}\arabic{c}"), "4");
}

#[test]
fn conditionals() {
    assert_eq!(run(r"\ifnum 3>2 yes\else no\fi"), "yes");
    assert_eq!(run(r"\ifnum 3<2 yes\else no\fi"), "no");
    assert_eq!(run(r"\ifdim 1pt<2pt a\fi b"), "ab");
    assert_eq!(run(r"\def\a{x}\def\b{x}\ifx\a\b same\else different\fi"), "same");
    assert_eq!(run(r"\def\a{x}\def\b{y}\ifx\a\b same\else different\fi"), "different");
    assert_eq!(run(r"\ifcase 2 zero\or one\or two\else many\fi"), "two");
    assert_eq!(run(r"\ifcase 5 zero\or one\else many\fi"), "many");
    // the space after the second character is text
    assert_eq!(run(r"\if aa yes\fi\if ab no\fi"), " yes");
    assert_eq!(run(r"\ifodd 3 odd\fi"), "odd");
    assert_eq!(run(r"\iftrue\iffalse x\else y\fi\fi"), "y");
    assert_eq!(run(r"\newif\ifshow \showtrue \ifshow S\fi\showfalse\ifshow T\fi"), "S");
    assert_eq!(run(r"\ifdefined\nothing a\else b\fi\ifdefined\relax c\fi"), "bc");
    assert_eq!(run(r"\ifcsname relax\endcsname a\fi\ifcsname nothing\endcsname b\fi"), "a");
    assert_eq!(run(r"\unless\ifnum 1=2 not\fi"), "not");
    assert_eq!(run(r"\ifthenelse{\equal{a}{a}}{yes}{no}"), "yes");
    assert_eq!(run(r"\ifthenelse{3 > 5 \or 1=1}{yes}{no}"), "yes");
    assert_eq!(run(r"\ifthenelse{\not 3 > 5 \and \isodd{3}}{yes}{no}"), "yes");
}

#[test]
fn latex_commands() {
    assert_eq!(run(r"\newcommand{\pt}[2]{(#1|#2)}\pt{a}{b}"), "(a|b)");
    assert_eq!(run(r"\newcommand\opt[2][d]{#1-#2}\opt{x}\opt[y]{z}"), "d-xy-z");
    assert_eq!(run(r"\newcommand*{\x}{X}\renewcommand{\x}{Y}\providecommand{\x}{Z}\x"), "Y");
    assert_eq!(run(r"\makeatletter\def\a@b{AB}\a@b\makeatother"), "AB");
    assert_eq!(run(r"\makeatletter\@ifnextchar[{opt}{none}[x"), "opt[x");
    assert_eq!(run(r"\makeatletter\@ifnextchar[{opt}{none} x"), "nonex");
    let mut t = Tex::new();
    assert_eq!(run_in(&mut t, r"\newenvironment{box}[1]{<#1|}{|>}"), "");
    assert!(t.environments.contains_key("box"));
}

#[test]
fn loops_end() {
    assert_eq!(run(r"\newcount\i \i=0 \loop\ifnum\i<3 \advance\i 1 \the\i\repeat"), "123");
    assert_eq!(run(r"\newcounter{k}\whiledo{\value{k}<3}{\stepcounter{k}\arabic{k}}"), "123");
    // endless expansion stops with an error instead of hanging
    assert!(run(r"\def\a{\a}\a").starts_with("error"));
    assert!(run(r"\def\a{x\a}\a").starts_with("error"));
}
