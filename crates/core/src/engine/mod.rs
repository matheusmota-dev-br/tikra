//! The drawing layer: TikZ on top of [`crate::tex`]. Commands such as `\draw` read their arguments
//! from the token stream, the way `tikz.code.tex` does; options are pgfkeys; the graphic state is
//! saved with TeX groups; what is drawn goes to a [`Canvas`].

pub mod arrows;
pub mod basic;
pub mod coords;
pub mod intersections;
pub mod nodes;
pub mod options;
pub mod output;
pub mod path;
pub mod shapes;
pub mod state;

use std::collections::HashMap;
use std::rc::Rc;

use crate::canvas::{Canvas, Item};
use crate::color::{self, Color};
use crate::scene::{Point, Transform};
use crate::tex::{self, Cat, Meaning, Prim, Tex, Token};
use crate::Error;

pub use state::GState;

/// Commands of the drawing layer (the number is the `Meaning::Native` id).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Cmd {
    Path,
    Draw,
    Fill,
    Filldraw,
    Clip,
    Shade,
    Shadedraw,
    Pattern,
    Node,
    Coordinate,
    Useasboundingbox,
    Matrix,
    Pic,
    Graph,
    Tikz,
    Begin,
    End,
    Scoped,
    Tikzset,
    Pgfkeys,
    Pgfqkeys,
    Pgfkeysalso,
    Pgfset,
    Pgfplotsset,
    Tikzstyle,
    Key,
    SetPath,
    Foreach,
    Definecolor,
    Colorlet,
    Color,
    Textcolor,
    Pgfmathsetmacro,
    Pgfmathparse,
    Pgfmathtruncatemacro,
    Pgfmathsetlengthmacro,
    Pgfmathsetcount,
    Pgfmathsetlength,
    Pgfmathdeclarefunction,
    Pgfmathprintnumber,
    Usetikzlibrary,
    Usepackage,
    Documentclass,
    Ignore1,
    Ignore2,
    FontSize,
    FontSeries,
    Unsupported,
}

const COMMANDS: &[(&str, Cmd)] = &[
    ("path", Cmd::Path),
    ("draw", Cmd::Draw),
    ("fill", Cmd::Fill),
    ("filldraw", Cmd::Filldraw),
    ("clip", Cmd::Clip),
    ("shade", Cmd::Shade),
    ("shadedraw", Cmd::Shadedraw),
    ("pattern", Cmd::Pattern),
    ("node", Cmd::Node),
    ("coordinate", Cmd::Coordinate),
    ("useasboundingbox", Cmd::Useasboundingbox),
    ("matrix", Cmd::Matrix),
    ("pic", Cmd::Pic),
    ("graph", Cmd::Graph),
    ("tikz", Cmd::Tikz),
    ("begin", Cmd::Begin),
    ("end", Cmd::End),
    ("scoped", Cmd::Scoped),
    ("tikzset", Cmd::Tikzset),
    ("pgfkeys", Cmd::Pgfkeys),
    ("pgfqkeys", Cmd::Pgfqkeys),
    ("pgfkeysalso", Cmd::Pgfkeysalso),
    ("pgfset", Cmd::Pgfset),
    ("pgfplotsset", Cmd::Pgfplotsset),
    ("tikzstyle", Cmd::Tikzstyle),
    ("tikra@key", Cmd::Key),
    ("tikra@setpath", Cmd::SetPath),
    ("foreach", Cmd::Foreach),
    ("definecolor", Cmd::Definecolor),
    ("providecolor", Cmd::Definecolor),
    ("colorlet", Cmd::Colorlet),
    ("color", Cmd::Color),
    ("textcolor", Cmd::Textcolor),
    ("pgfmathsetmacro", Cmd::Pgfmathsetmacro),
    ("pgfmathparse", Cmd::Pgfmathparse),
    ("pgfmathtruncatemacro", Cmd::Pgfmathtruncatemacro),
    ("pgfmathsetlengthmacro", Cmd::Pgfmathsetlengthmacro),
    ("pgfmathsetcount", Cmd::Pgfmathsetcount),
    ("pgfmathsetlength", Cmd::Pgfmathsetlength),
    ("pgfmathdeclarefunction", Cmd::Pgfmathdeclarefunction),
    ("pgfmathprintnumber", Cmd::Pgfmathprintnumber),
    ("usetikzlibrary", Cmd::Usetikzlibrary),
    ("usepgflibrary", Cmd::Usetikzlibrary),
    ("usepgfplotslibrary", Cmd::Usetikzlibrary),
    ("usepgfmodule", Cmd::Usetikzlibrary),
    ("usepackage", Cmd::Usepackage),
    ("RequirePackage", Cmd::Usepackage),
    ("documentclass", Cmd::Documentclass),
    ("pgfplotsset", Cmd::Pgfplotsset),
    ("label", Cmd::Ignore1),
    ("hspace", Cmd::Ignore1),
    ("vspace", Cmd::Ignore1),
    ("pgfdeclarelayer", Cmd::Ignore1),
    ("pgfsetlayers", Cmd::Ignore1),
    ("tikzexternalize", Cmd::Ignore1),
    ("pgfrealjobname", Cmd::Ignore1),
    ("pagestyle", Cmd::Ignore1),
    ("thispagestyle", Cmd::Ignore1),
    ("caption", Cmd::Ignore1),
    ("tikzsetnextfilename", Cmd::Ignore1),
    ("pgfplotstableset", Cmd::Ignore1),
    ("tiny", Cmd::FontSize),
    ("scriptsize", Cmd::FontSize),
    ("footnotesize", Cmd::FontSize),
    ("small", Cmd::FontSize),
    ("normalsize", Cmd::FontSize),
    ("large", Cmd::FontSize),
    ("Large", Cmd::FontSize),
    ("LARGE", Cmd::FontSize),
    ("huge", Cmd::FontSize),
    ("Huge", Cmd::FontSize),
    ("bfseries", Cmd::FontSeries),
    ("mdseries", Cmd::FontSeries),
    ("itshape", Cmd::FontSeries),
    ("upshape", Cmd::FontSeries),
    ("slshape", Cmd::FontSeries),
    ("scshape", Cmd::FontSeries),
    ("ttfamily", Cmd::FontSeries),
    ("sffamily", Cmd::FontSeries),
    ("rmfamily", Cmd::FontSeries),
    ("centering", Cmd::FontSeries),
    ("raggedright", Cmd::FontSeries),
    ("raggedleft", Cmd::FontSeries),
    ("noindent", Cmd::FontSeries),
    ("maketitle", Cmd::FontSeries),
    ("selectfont", Cmd::FontSeries),
    ("datavisualization", Cmd::Unsupported),
    ("calendar", Cmd::Unsupported),
    ("spy", Cmd::Unsupported),
];

/// Commands that the drawing layer turns into a path statement, with their default options.
fn path_command(cmd: Cmd) -> Option<&'static str> {
    Some(match cmd {
        Cmd::Path => "",
        Cmd::Draw => "draw",
        Cmd::Fill => "fill",
        Cmd::Filldraw => "fill,draw",
        Cmd::Clip => "clip",
        Cmd::Shade => "shade",
        Cmd::Shadedraw => "shade,draw",
        Cmd::Pattern => "pattern",
        Cmd::Useasboundingbox => "use as bounding box",
        _ => return None,
    })
}

/// A pgf scope (picture, `scope`, node): what must be closed in the canvas when it ends.
#[derive(Debug, Clone, Default)]
struct PgfScope {
    ends: usize,
}

/// Cap on statements per figure: nested loops do not hang the caller.
pub const MAX_STEPS: usize = 100_000;
/// Cap on nesting of scopes, loops and environments.
pub const MAX_DEPTH: usize = 64;

pub struct Engine {
    pub tex: Tex,
    pub st: GState,
    saved: Vec<GState>,
    pub canvas: Canvas,
    pub nodes: HashMap<String, nodes::NodeInfo>,
    pub colors: HashMap<String, Color>,
    scopes: Vec<PgfScope>,
    /// how many pictures are open (`tikzpicture` inside `tikzpicture` is a scope)
    pub picture_depth: usize,
    pub pictures: usize,
    pub steps: usize,
    pub depth: usize,
    /// text outside nodes was seen and reported
    text_reported: bool,
    /// the native keys, by id − NATIVE_KEY_BASE
    key_fns: Vec<options::KeyFn>,
    /// `use as bounding box` was used: the bounding box no longer grows
    pub canvas_frozen: bool,
    /// nested runs of token lists (styles, loop bodies)
    nesting: usize,
    /// the basic layer's path being built
    pub pgf_path: basic::PgfPath,
    /// paths kept by `name path`
    pub named_paths: HashMap<String, Vec<crate::scene::Segment>>,
    /// pictures already drawn, and how the document lays them out
    boxes: Vec<PictureBox>,
    layout: Vec<Layout>,
    /// keys run so far
    pub keys_run: usize,
}

/// A finished picture: its canvas and the height of its baseline in it (cm).
#[derive(Debug, Clone)]
struct PictureBox {
    canvas: Canvas,
    baseline: f64,
}

/// What a document puts between pictures: TeX typesets them in lines, like words.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Layout {
    Picture(usize),
    Space,
    Par,
}

/// Cap on nested runs (a style calling itself).
pub const MAX_NESTING: usize = 150;
/// Cap on keys per figure (styles that multiply: `s/.style={t,t,t,t}`…).
pub const MAX_KEYS: usize = 200_000;

/// Ids at and above this are native keys (`\pgfk@…/.@cmd`), below are commands.
pub const NATIVE_KEY_BASE: u16 = 1000;

thread_local! {
    /// The engine after defining every command and key: cloned for each figure.
    static PROTOTYPE: std::cell::RefCell<Option<Engine>> = const { std::cell::RefCell::new(None) };
}

impl Clone for Engine {
    fn clone(&self) -> Self {
        Engine {
            tex: self.tex.clone(),
            st: self.st.clone(),
            saved: self.saved.clone(),
            canvas: self.canvas.clone(),
            nodes: self.nodes.clone(),
            colors: self.colors.clone(),
            scopes: self.scopes.clone(),
            picture_depth: self.picture_depth,
            pictures: self.pictures,
            steps: self.steps,
            depth: self.depth,
            text_reported: self.text_reported,
            key_fns: self.key_fns.clone(),
            canvas_frozen: self.canvas_frozen,
            nesting: self.nesting,
            keys_run: self.keys_run,
            pgf_path: self.pgf_path.clone(),
            named_paths: self.named_paths.clone(),
            boxes: self.boxes.clone(),
            layout: self.layout.clone(),
        }
    }
}

impl Engine {
    /// A fresh engine with TikZ defined (built once per thread, then cloned).
    pub fn new() -> Engine {
        PROTOTYPE.with(|p| {
            let mut p = p.borrow_mut();
            if p.is_none() {
                *p = Some(Engine::build());
            }
            p.as_ref().expect("built").clone()
        })
    }

    fn build() -> Engine {
        let mut e = Engine {
            tex: Tex::new(),
            st: GState::default(),
            saved: Vec::new(),
            canvas: Canvas::default(),
            nodes: HashMap::new(),
            colors: HashMap::new(),
            scopes: Vec::new(),
            picture_depth: 0,
            pictures: 0,
            steps: 0,
            depth: 0,
            text_reported: false,
            key_fns: Vec::new(),
            canvas_frozen: false,
            nesting: 0,
            keys_run: 0,
            pgf_path: basic::PgfPath::default(),
            named_paths: HashMap::new(),
            boxes: Vec::new(),
            layout: Vec::new(),
        };
        for (name, cmd) in COMMANDS {
            e.tex.define(name, Meaning::Native(*cmd as u16));
        }
        e.tex.define_text(r"\let\bgroup={\let\egroup=}\def\pgfkeysnovalue{\pgfkeysnovalue}\def\pgfkeysvalueof#1{\csname pgfk@#1\endcsname}\def\pgfkeysgetvalue#1#2{\expandafter\let\expandafter#2\csname pgfk@#1\endcsname}\def\pgfkeyssetvalue#1#2{\expandafter\def\csname pgfk@#1\endcsname{#2}}\def\pgfkeysifdefined#1{\ifcsname pgfk@#1\endcsname\expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi}\def\pgfmathresult{}\def\tikzlastnode{}");
        basic::define(&mut e);
        options::define_keys(&mut e);
        e
    }

    // ---------------------------------------------------------------- running

    pub fn warn(&mut self, s: impl Into<String>) {
        self.tex.warn(s);
    }

    /// Runs the input until it ends.
    pub fn run(&mut self) -> Result<(), Error> {
        while let Some(t) = self.tex.get_x_token()? {
            self.execute(t)?;
        }
        Ok(())
    }

    /// Runs `tokens` to completion (a loop body, a style's code), then returns.
    pub fn run_tokens(&mut self, tokens: Vec<Token>) -> Result<(), Error> {
        if self.nesting >= MAX_NESTING {
            // a style or macro that calls itself: TeX would run out of memory
            self.warn(format!("styles or macros nested more than {MAX_NESTING} levels (one calls itself?): the rest ignored"));
            return Ok(());
        }
        self.nesting += 1;
        let depth = self.tex.depth();
        self.tex.push_frame(tokens);
        let r = (|| {
            while let Some(t) = self.tex.get_x_token_until(depth)? {
                self.execute(t)?;
            }
            Ok(())
        })();
        if r.is_err() {
            self.tex.drain_to(depth);
        }
        self.nesting -= 1;
        r
    }

    fn count_step(&mut self) -> Result<(), Error> {
        self.steps += 1;
        if self.steps > MAX_STEPS {
            return Err(Error::Figure(format!("figure too large: more than {MAX_STEPS} statements")));
        }
        Ok(())
    }

    pub fn execute(&mut self, t: Token) -> Result<(), Error> {
        let m = self.tex.meaning(&t);
        match m {
            Meaning::Char(_, Cat::Begin) => self.begin_group(),
            Meaning::Char(_, Cat::End) => self.end_group(),
            Meaning::Prim(Prim::Par) if self.picture_depth == 0 => {
                if !self.layout.is_empty() && self.layout.last() != Some(&Layout::Par) {
                    self.layout.push(Layout::Par);
                }
            }
            Meaning::Prim(Prim::Begingroup) => self.begin_group(),
            Meaning::Prim(Prim::Endgroup) => self.end_group(),
            Meaning::Native(id) if id >= NATIVE_KEY_BASE => {
                // a key's code used as a command: nothing to do outside key processing
            }
            Meaning::Native(id) => {
                // keys run by a key list are not statements, and their errors go to whoever runs the list
                if id == Cmd::Key as u16 || id == Cmd::SetPath as u16 {
                    return self.command(id, &t);
                }
                self.count_step()?;
                self.tex.new_statement();
                let r = self.command(id, &t);
                self.recover(r, &t)?;
            }
            Meaning::Char(c, _) if matches!(t, Token::Char(..)) => self.text(c),
            Meaning::Undefined => {
                if let Token::Cs(name) = &t {
                    self.unknown_command(name)?;
                }
            }
            _ => {
                if !self.tex.execute_basic(t.clone())? {
                    if let Meaning::Char(c, _) = self.tex.meaning(&t) {
                        self.text(c);
                    }
                }
            }
        }
        Ok(())
    }

    /// A character outside any statement: TeX would typeset it; in a picture TikZ ignores it.
    fn text(&mut self, c: char) {
        if self.picture_depth == 0 && c.is_whitespace() {
            // a space between two pictures is TeX's interword space
            if matches!(self.layout.last(), Some(Layout::Picture(_))) {
                self.layout.push(Layout::Space);
            }
            return;
        }
        if c.is_whitespace() || c == ';' || self.text_reported {
            return;
        }
        if self.picture_depth > 0 {
            self.text_reported = true;
            self.warn("text outside a statement ignored");
        }
    }

    /// An undefined command: in a picture, the statement it starts is skipped to its `;`.
    fn unknown_command(&mut self, name: &str) -> Result<(), Error> {
        if self.picture_depth > 0 {
            let rest = self.skip_statement()?;
            self.warn(format!("\\{name} not supported: '{}' ignored", rest.chars().take(40).collect::<String>().trim()));
        }
        Ok(())
    }

    /// Reads up to the `;` that ends a statement (outside braces), and returns it as text.
    pub fn skip_statement(&mut self) -> Result<String, Error> {
        let mut out = Vec::new();
        let mut depth = 0usize;
        while let Some(t) = self.tex.get_token() {
            match t {
                Token::Char(_, Cat::Begin) => depth += 1,
                Token::Char(_, Cat::End) => {
                    if depth == 0 {
                        self.tex.back(t);
                        break;
                    }
                    depth -= 1;
                }
                Token::Char(';', _) if depth == 0 => break,
                _ => {}
            }
            out.push(t);
            if out.len() > tex::MAX_TOKENS {
                break;
            }
        }
        Ok(tex::tokens_to_string(&out))
    }

    // ---------------------------------------------------------------- groups and scopes

    pub fn begin_group(&mut self) {
        self.tex.begin_group();
        self.saved.push(self.st.clone());
    }

    pub fn end_group(&mut self) {
        if self.tex.end_group() {
            if let Some(s) = self.saved.pop() {
                self.st = s;
            }
        } else {
            self.warn("extra '}'");
        }
    }

    pub fn begin_scope(&mut self) {
        self.scopes.push(PgfScope::default());
    }

    pub fn end_scope(&mut self) {
        if let Some(s) = self.scopes.pop() {
            for _ in 0..s.ends {
                self.canvas.push(Item::End);
            }
        }
    }

    /// Opens a canvas item that the current pgf scope closes (a clip, a transparency group).
    pub fn open_in_scope(&mut self, item: Item) {
        self.canvas.push(item);
        match self.scopes.last_mut() {
            Some(s) => s.ends += 1,
            None => self.canvas.push(Item::End),
        }
    }

    // ---------------------------------------------------------------- reading arguments

    pub fn argument(&mut self) -> Result<Vec<Token>, Error> {
        self.tex.read_undelimited()
    }

    pub fn argument_string(&mut self) -> Result<String, Error> {
        let a = self.argument()?;
        Ok(self.tex.expand_to_string(a)?.trim().to_string())
    }

    /// `[…]` if it comes next (after spaces), without the brackets.
    pub fn optional(&mut self) -> Result<Option<Vec<Token>>, Error> {
        match self.tex.next_nonspace() {
            Some(t) if t.is_char('[') => Ok(Some(self.tex.read_until_char(']', false)?)),
            Some(t) => {
                self.tex.back(t);
                Ok(None)
            }
            None => Ok(None),
        }
    }

    /// Text for pgfmath: macros expanded, registers replaced by their values.
    pub fn math_string(&mut self, tokens: Vec<Token>) -> Result<String, Error> {
        let t = self.tex.expand_fully(tokens)?;
        let mut s = String::new();
        for (i, x) in t.iter().enumerate() {
            match self.tex.meaning(x) {
                Meaning::Count(r) if matches!(x, Token::Cs(_)) => s.push_str(&self.tex.counts.get(&r).copied().unwrap_or(0).to_string()),
                Meaning::CharDef(v) if matches!(x, Token::Cs(_)) => s.push_str(&v.to_string()),
                Meaning::Dimen(r) | Meaning::Skip(r) if matches!(x, Token::Cs(_)) => {
                    let sp = self.tex.dimens.get(&r).copied().unwrap_or(0);
                    s.push_str(&format!("{}pt", sp as f64 / 65536.0));
                }
                _ => s.push_str(&tex::tokens_to_string(&t[i..i + 1])),
            }
            // keep `\x y` apart when a control word is followed by a letter
            if let (Token::Cs(_), Some(Token::Char(c, _))) = (x, t.get(i + 1)) {
                if c.is_alphabetic() && !s.ends_with(' ') {
                    s.push(' ');
                }
            }
        }
        Ok(s)
    }

    pub fn vars(&self) -> crate::expr::Variables {
        crate::expr::Variables { values: HashMap::new(), functions: (*self.st.functions).clone() }
    }

    /// A pgfmath number from text.
    pub fn number(&mut self, text: &str) -> Result<f64, Error> {
        let t = tex::string_tokens(text);
        let s = self.math_string(t)?;
        crate::expr::eval(&s, &self.vars())
    }

    /// A pgfmath length in cm (a number without a unit is in pt, as pgf reads lengths).
    pub fn length(&mut self, text: &str) -> Result<f64, Error> {
        let s = self.math_string(tikra_tokens(text))?;
        let (v, unit) = crate::expr::eval_length(&s, &self.vars())?;
        Ok(if unit { v } else { v * crate::PT })
    }

    pub fn number_of(&mut self, tokens: Vec<Token>) -> Result<f64, Error> {
        let s = self.math_string(tokens)?;
        crate::expr::eval(&s, &self.vars())
    }

    pub fn length_of(&mut self, tokens: Vec<Token>) -> Result<f64, Error> {
        let s = self.math_string(tokens)?;
        let (v, unit) = crate::expr::eval_length(&s, &self.vars())?;
        Ok(if unit { v } else { v * crate::PT })
    }

    pub fn color(&self, text: &str) -> Option<Color> {
        color::parse(text.trim(), &self.colors)
    }

    // ---------------------------------------------------------------- commands

    fn command(&mut self, id: u16, t: &Token) -> Result<(), Error> {
        if basic::is_basic(id) {
            return self.basic_command(id);
        }
        let Some(&(_, cmd)) = COMMANDS.iter().find(|(_, c)| *c as u16 == id) else { return Ok(()) };
        if let Some(defaults) = path_command(cmd) {
            let r = path::statement(self, defaults);
            return self.recover(r, t);
        }
        match cmd {
            Cmd::Node => {
                self.tex.push_tokens(tikra_tokens("node "));
                let r = path::statement(self, "");
                self.recover(r, t)
            }
            Cmd::Coordinate => {
                self.tex.push_tokens(tikra_tokens("coordinate "));
                let r = path::statement(self, "");
                self.recover(r, t)
            }
            Cmd::Matrix | Cmd::Pic | Cmd::Graph => {
                let rest = self.skip_statement()?;
                let _ = rest;
                let name = tex::show_token(t);
                self.warn(format!("{name} is not supported: ignored"));
                Ok(())
            }
            Cmd::Unsupported => {
                let name = tex::show_token(t);
                let rest = if self.picture_depth > 0 { self.skip_statement()? } else { String::new() };
                self.warn(format!("{name} not supported: '{}' ignored", rest.chars().take(40).collect::<String>().trim()));
                Ok(())
            }
            Cmd::Tikz => self.tikz_command(),
            Cmd::Begin => self.begin_environment(),
            Cmd::End => self.end_environment(),
            Cmd::Scoped => {
                let o = self.optional()?;
                self.begin_group();
                self.begin_scope();
                self.run_tokens(tikra_tokens("\\tikz@scope@options"))?;
                if let Some(o) = o {
                    self.process_keys(o, Some("/tikz"))?;
                }
                self.st.apply_pending();
                // the next statement or group is the content
                let body = self.next_statement()?;
                self.run_tokens(body)?;
                self.end_scope();
                self.end_group();
                Ok(())
            }
            Cmd::Tikzset => {
                let a = self.argument()?;
                self.keys_command(a, Some("/tikz"), true)
            }
            Cmd::Pgfset => {
                let a = self.argument()?;
                self.keys_command(a, Some("/pgf"), true)
            }
            Cmd::Pgfplotsset => {
                let a = self.argument()?;
                self.keys_command(a, Some("/pgfplots"), true)
            }
            Cmd::Pgfkeys => {
                let a = self.argument()?;
                self.keys_command(a, Some("/"), true)
            }
            Cmd::Pgfqkeys => {
                let p = self.argument_string()?;
                let a = self.argument()?;
                self.keys_command(a, Some(&p), true)
            }
            Cmd::Pgfkeysalso => {
                let a = self.argument()?;
                self.keys_command(a, None, false)
            }
            Cmd::Tikzstyle => {
                // \tikzstyle{name}=[options], \tikzstyle{name}+=[options]
                let name = self.argument_string()?;
                let mut append = false;
                if let Some(x) = self.tex.next_nonspace() {
                    if x.is_char('+') {
                        append = true;
                    } else if !x.is_char('=') {
                        self.tex.back(x);
                    }
                }
                if append {
                    if let Some(x) = self.tex.next_nonspace() {
                        if !x.is_char('=') {
                            self.tex.back(x);
                        }
                    }
                }
                let o = self.optional()?.unwrap_or_default();
                let mut v = tikra_tokens(&format!("{name}/.{}=", if append { "append style" } else { "style" }));
                v.push(Token::Char('{', Cat::Begin));
                v.extend(o);
                v.push(Token::Char('}', Cat::End));
                self.keys_command(v, Some("/tikz"), true)
            }
            Cmd::Key => self.invoke_key_command(),
            Cmd::SetPath => {
                let p = self.argument_string()?;
                self.st.key_path = p;
                Ok(())
            }
            Cmd::Foreach => self.foreach(),
            Cmd::Definecolor => {
                // \definecolor{name}{model}{values}, with an optional [type] first
                let _ = self.optional()?;
                let name = self.argument_string()?;
                let model = self.argument_string()?;
                let values = self.argument_string()?;
                match color::define(&model, &values) {
                    Some(c) => {
                        self.colors.insert(name, c);
                    }
                    None => self.warn(format!("\\definecolor{{{name}}}{{{model}}}: model not supported")),
                }
                Ok(())
            }
            Cmd::Colorlet => {
                let _ = self.optional()?;
                let name = self.argument_string()?;
                let e = self.argument_string()?;
                match self.color(&e) {
                    Some(c) => {
                        self.colors.insert(name, c);
                    }
                    None => self.warn(format!("\\colorlet{{{name}}}: unknown color '{e}'")),
                }
                Ok(())
            }
            Cmd::Color => {
                let _ = self.optional()?;
                let e = self.argument_string()?;
                if let Some(c) = self.color(&e) {
                    self.st.color = c;
                    self.st.draw_color = None;
                    self.st.fill_color = None;
                    self.st.text_color = None;
                }
                Ok(())
            }
            Cmd::Textcolor => {
                // text, typeset by TeX: outside nodes it is ignored
                let _ = self.argument()?;
                let _ = self.argument()?;
                Ok(())
            }
            Cmd::Pgfmathsetmacro | Cmd::Pgfmathtruncatemacro | Cmd::Pgfmathsetlengthmacro => {
                let name = self.argument()?;
                let expr = self.argument()?;
                let Some(name) = name.into_iter().find(|x| !x.is_space()) else { return Ok(()) };
                let text = self.math_string(expr)?;
                let v = match cmd {
                    Cmd::Pgfmathsetlengthmacro => crate::expr::eval_length(&text, &self.vars()).map(|(v, u)| if u { v / crate::PT } else { v }),
                    _ => crate::expr::eval(&text, &self.vars()),
                };
                match v {
                    Ok(v) if v.is_finite() => {
                        let s = match cmd {
                            Cmd::Pgfmathtruncatemacro => format!("{}", v.trunc() as i64),
                            Cmd::Pgfmathsetlengthmacro => format!("{}pt", pgf_number(v)),
                            _ => pgf_number(v),
                        };
                        self.tex.set_meaning(&name, macro_from(&s), false);
                        self.tex.define_string("pgfmathresult", &s, false);
                    }
                    Ok(v) => self.warn(format!("\\pgfmathsetmacro{{{}}}: non-finite result ({v}), macro not defined", tex::show_token(&name))),
                    Err(e) => self.warn(format!("\\pgfmathsetmacro{{{}}}: {e}", tex::show_token(&name))),
                }
                Ok(())
            }
            Cmd::Pgfmathparse => {
                let expr = self.argument()?;
                let text = self.math_string(expr)?;
                match crate::expr::eval(&text, &self.vars()) {
                    Ok(v) => self.tex.define_string("pgfmathresult", &pgf_number(v), false),
                    Err(e) => self.warn(format!("\\pgfmathparse: {e}")),
                }
                Ok(())
            }
            Cmd::Pgfmathsetcount | Cmd::Pgfmathsetlength => {
                let target = self.argument()?;
                let expr = self.argument()?;
                let text = self.math_string(expr)?;
                let Some(name) = target.into_iter().find(|x| !x.is_space()) else { return Ok(()) };
                match self.tex.meaning(&name) {
                    Meaning::Count(r) => {
                        let v = crate::expr::eval(&text, &self.vars())?;
                        self.tex.set_count(r, v.trunc() as i64, false);
                    }
                    Meaning::Dimen(r) | Meaning::Skip(r) => {
                        let (v, u) = crate::expr::eval_length(&text, &self.vars())?;
                        let pt = if u { v / crate::PT } else { v };
                        self.tex.set_dimen(r, (pt * 65536.0).round() as i64, false);
                    }
                    _ => {}
                }
                Ok(())
            }
            Cmd::Pgfmathdeclarefunction => {
                // \pgfmathdeclarefunction{name}{n}{code}: code sets \pgfmathresult; only `\pgfmathparse{expr}` bodies are understood
                let name = self.argument_string()?;
                let n: usize = self.argument_string()?.parse().unwrap_or(0);
                let body = self.argument()?;
                let text = tex::tokens_to_string(&body);
                let expr = text.split("\\pgfmathparse").nth(1).map(|r| r.trim().trim_start_matches('{').trim_end_matches('}').to_string());
                match expr {
                    Some(e) => {
                        let params = (1..=n).map(|k| format!("#{k}")).collect();
                        Rc::make_mut(&mut self.st.functions).insert(name, (params, e));
                    }
                    None => self.warn(format!("\\pgfmathdeclarefunction{{{name}}}: only a \\pgfmathparse body is supported")),
                }
                Ok(())
            }
            Cmd::Pgfmathprintnumber => {
                let _ = self.optional()?;
                let a = self.argument()?;
                let v = self.number_of(a)?;
                self.tex.push_tokens(tex::string_tokens(&pgf_number(v)));
                Ok(())
            }
            Cmd::Usetikzlibrary | Cmd::Usepackage | Cmd::Documentclass => {
                let o = if cmd == Cmd::Documentclass { self.documentclass_options() } else { self.optional()? };
                // the name in braces (a broken \documentclass[ may have none)
                let a = match self.tex.next_nonspace() {
                    Some(Token::Char(_, Cat::Begin)) => {
                        let body = self.tex.read_group_body()?;
                        self.tex.expand_to_string(body)?
                    }
                    Some(t) => {
                        self.tex.back(t);
                        String::new()
                    }
                    None => String::new(),
                };
                if cmd == Cmd::Documentclass && a.contains("standalone") {
                    if let Some(o) = o {
                        self.standalone_options(o)?;
                    }
                }
                Ok(())
            }
            Cmd::Ignore1 => {
                let _ = self.optional()?;
                let _ = self.argument()?;
                Ok(())
            }
            Cmd::Ignore2 => {
                let _ = self.argument()?;
                let _ = self.argument()?;
                Ok(())
            }
            Cmd::FontSize => {
                if let Token::Cs(n) = t {
                    self.st.font_size = font_size(n);
                    self.st.font.push_str(&format!("\\{n}"));
                }
                Ok(())
            }
            Cmd::FontSeries => {
                if let Token::Cs(n) = t {
                    self.st.font.push_str(&format!("\\{n}"));
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// An error in a statement: reported, and the rest of the statement skipped, as TeX goes on after an
    /// error. Limits (endless loops, a figure too large) stop the figure.
    fn recover(&mut self, r: Result<(), Error>, t: &Token) -> Result<(), Error> {
        match r {
            Ok(()) => Ok(()),
            Err(e) if is_limit(&e) => Err(e),
            Err(e) => {
                self.warn(format!("{}: {e}; statement ignored", tex::show_token(t)));
                let _ = self.skip_statement();
                Ok(())
            }
        }
    }

    /// `\documentclass[…]`, read with tolerance: a bracket left open ends before the next `\begin`.
    fn documentclass_options(&mut self) -> Option<Vec<Token>> {
        let t = self.tex.next_nonspace()?;
        if !t.is_char('[') {
            self.tex.back(t);
            return None;
        }
        let mut out = Vec::new();
        let mut depth = 0usize;
        while let Some(x) = self.tex.get_token() {
            match &x {
                Token::Char(_, Cat::Begin) => depth += 1,
                Token::Char(_, Cat::End) => depth = depth.saturating_sub(1),
                x if depth == 0 && x.is_char(']') => return Some(out),
                x if x.is_cs("begin") => {
                    self.tex.back(x.clone());
                    return Some(out);
                }
                _ => {}
            }
            out.push(x);
        }
        Some(out)
    }

    /// The `border` option of `\documentclass[border=…]{standalone}`.
    fn standalone_options(&mut self, o: Vec<Token>) -> Result<(), Error> {
        for item in crate::keys::split_list(&o) {
            if item.key == "border" {
                if let Some(v) = item.value {
                    let s = self.tex.expand_to_string(v)?;
                    let first = s.split_whitespace().next().unwrap_or("0pt").to_string();
                    if let Ok(m) = self.length(&first) {
                        self.canvas.margin = m;
                    }
                }
            }
        }
        Ok(())
    }

    /// The next statement in a picture: a `{…}` group, or tokens up to and including `;`.
    pub fn next_statement(&mut self) -> Result<Vec<Token>, Error> {
        match self.tex.next_nonspace() {
            Some(Token::Char(_, Cat::Begin)) => self.tex.read_group_body(),
            Some(t) => {
                self.tex.back(t);
                let mut out = Vec::new();
                let mut depth = 0usize;
                while let Some(x) = self.tex.get_token() {
                    match x {
                        Token::Char(_, Cat::Begin) => depth += 1,
                        Token::Char(_, Cat::End) => depth = depth.saturating_sub(1),
                        _ => {}
                    }
                    let end = depth == 0 && x.is_char(';');
                    out.push(x);
                    if end || out.len() > tex::MAX_TOKENS {
                        break;
                    }
                }
                Ok(out)
            }
            None => Ok(Vec::new()),
        }
    }

    // ---------------------------------------------------------------- pictures and environments

    fn begin_picture(&mut self, o: Option<Vec<Token>>) -> Result<(), Error> {
        if self.depth >= MAX_DEPTH {
            return Err(Error::Figure(format!("scope/\\foreach nesting went past {MAX_DEPTH} levels")));
        }
        self.depth += 1;
        if self.picture_depth == 0 {
            self.pictures += 1;
            // each picture is drawn on its own canvas, laid out with the others at the end
            let margin = self.canvas.margin;
            self.canvas = Canvas { margin, ..Canvas::default() };
            self.canvas_frozen = false;
        }
        self.picture_depth += 1;
        self.begin_group();
        self.begin_scope();
        self.run_tokens(tikra_tokens("\\tikz@picture@start"))?;
        if let Some(o) = o {
            // the picture's own options failing bring the figure down, as in TeX
            self.process_keys(o, Some("/tikz")).map_err(|e| Error::Figure(format!("picture options: {e}")))?;
        }
        self.st.apply_pending();
        Ok(())
    }

    fn end_picture(&mut self) {
        self.end_scope();
        let outermost = self.picture_depth == 1;
        let baseline = if outermost { self.picture_baseline() } else { 0.0 };
        self.end_group();
        self.picture_depth = self.picture_depth.saturating_sub(1);
        self.depth = self.depth.saturating_sub(1);
        if outermost {
            let margin = self.canvas.margin;
            let canvas = std::mem::replace(&mut self.canvas, Canvas { margin, ..Canvas::default() });
            self.layout.push(Layout::Picture(self.boxes.len()));
            self.boxes.push(PictureBox { canvas, baseline });
        }
    }

    /// Where the picture's baseline is (`baseline=`): by default its lower edge, as TikZ sets it.
    fn picture_baseline(&mut self) -> f64 {
        let lower = self.canvas.min.map_or(0.0, |m| m.y);
        let Some(spec) = self.st.baseline.clone() else { return lower };
        let s = spec.trim();
        if s.is_empty() {
            return 0.0;
        }
        if let Some(inner) = s.strip_prefix('(').and_then(|r| r.strip_suffix(')')) {
            return self.coordinate(inner).map(|c| c.p.y).unwrap_or(lower);
        }
        self.length(s).unwrap_or(lower)
    }

    /// `\tikz[options]{…}` or `\tikz[options] statement;`
    fn tikz_command(&mut self) -> Result<(), Error> {
        let o = self.optional()?;
        let body = self.next_statement()?;
        self.begin_picture(o)?;
        self.run_tokens(body)?;
        self.end_picture();
        Ok(())
    }

    fn begin_environment(&mut self) -> Result<(), Error> {
        let name = self.argument_string()?;
        self.tex.env_stack.push(name.clone());
        match name.as_str() {
            "tikzpicture" => {
                let o = self.optional()?;
                self.begin_picture(o)
            }
            "pgfpicture" => self.begin_picture(None),
            "pgfscope" => {
                self.begin_group();
                self.begin_scope();
                Ok(())
            }
            "scope" => {
                if self.depth >= MAX_DEPTH {
                    return Err(Error::Figure(format!("scope/\\foreach nesting went past {MAX_DEPTH} levels")));
                }
                self.depth += 1;
                let o = self.optional()?;
                self.begin_group();
                self.begin_scope();
                self.run_tokens(tikra_tokens("\\tikz@scope@options"))?;
                if let Some(o) = o {
                    self.process_keys(o, Some("/tikz")).map_err(|e| Error::Figure(format!("scope options: {e}")))?;
                }
                self.st.apply_pending();
                Ok(())
            }
            "axis" | "semilogxaxis" | "semilogyaxis" | "loglogaxis" => {
                let o = self.optional()?.map(|o| tex::tokens_to_string(&o)).unwrap_or_default();
                let body = self.environment_body(&name)?;
                self.tex.env_stack.pop();
                let opened = self.picture_depth == 0;
                if opened {
                    self.begin_picture(None)?;
                }
                let r = crate::axes_bridge::axis(self, &o, &body);
                if opened {
                    self.end_picture();
                }
                r
            }
            "document" | "center" | "figure" | "flushleft" | "flushright" | "minipage" | "pgfonlayer" | "tikzfadingfrompicture" => {
                if name == "minipage" {
                    let _ = self.optional()?;
                    let _ = self.argument()?;
                } else if name == "pgfonlayer" {
                    let _ = self.argument()?;
                } else if name == "figure" {
                    let _ = self.optional()?;
                }
                self.begin_group();
                Ok(())
            }
            _ => {
                if let Some(code) = self.tex.environment_begin(&name)? {
                    self.begin_group();
                    self.tex.push_tokens(code);
                    return Ok(());
                }
                self.warn(format!("environment {name} not supported: content read as statements"));
                self.begin_group();
                Ok(())
            }
        }
    }

    /// The raw text of an environment, up to its `\end{name}` (the end consumed).
    fn environment_body(&mut self, name: &str) -> Result<String, Error> {
        let mut out = Vec::new();
        let mut depth = 0usize;
        while let Some(t) = self.tex.get_token() {
            if t.is_cs("begin") || t.is_cs("end") {
                let is_end = t.is_cs("end");
                let arg = self.tex.read_undelimited()?;
                let arg_name = tex::tokens_to_string(&arg);
                if arg_name.trim() == name {
                    if is_end {
                        if depth == 0 {
                            break;
                        }
                        depth -= 1;
                    } else {
                        depth += 1;
                    }
                }
                out.push(t);
                out.push(Token::Char('{', Cat::Begin));
                out.extend(arg);
                out.push(Token::Char('}', Cat::End));
                continue;
            }
            out.push(t);
            if out.len() > tex::MAX_TOKENS {
                break;
            }
        }
        Ok(tex::tokens_to_string(&out))
    }

    fn end_environment(&mut self) -> Result<(), Error> {
        let name = self.argument_string()?;
        let Some(pos) = self.tex.env_stack.iter().rposition(|n| *n == name) else {
            self.warn(format!("\\end{{{name}}} without a matching \\begin: ignored"));
            return Ok(());
        };
        // environments left open inside are closed with it
        while self.tex.env_stack.len() > pos + 1 {
            let inner = self.tex.env_stack.pop().unwrap_or_default();
            self.warn(format!("\\begin{{{inner}}} without \\end: closed at \\end{{{name}}}"));
            self.close_environment(&inner)?;
        }
        self.tex.env_stack.pop();
        self.close_environment(&name)
    }

    fn close_environment(&mut self, name: &str) -> Result<(), Error> {
        match name {
            "tikzpicture" | "pgfpicture" => self.end_picture(),
            "pgfscope" => {
                self.end_scope();
                self.end_group();
            }
            "scope" => {
                self.end_scope();
                self.end_group();
                self.depth = self.depth.saturating_sub(1);
            }
            _ => {
                if let Some((_, end)) = self.tex.environments.get(name).cloned() {
                    self.run_tokens(end)?;
                }
                self.end_group();
            }
        }
        Ok(())
    }

    // ---------------------------------------------------------------- \foreach

    /// `\foreach \x/\y [count=\i, evaluate=\x as \v using …, remember=\x as \lastx (initially 0)] in {list} body`
    fn foreach(&mut self) -> Result<(), Error> {
        // variables up to `[` or `in`
        let mut vars: Vec<Token> = Vec::new();
        let mut opts: Vec<Token> = Vec::new();
        loop {
            let Some(t) = self.tex.get_token() else { return Err(Error::Figure("\\foreach without 'in'".into())) };
            if t.is_char('[') {
                opts.extend(self.tex.read_until_char(']', false)?);
                continue;
            }
            if t.is_char('i') {
                if let Some(n) = self.tex.get_token() {
                    if n.is_char('n') {
                        break;
                    }
                    self.tex.back(n);
                }
            }
            vars.push(t);
            if vars.len() > 1000 {
                return Err(Error::Figure("\\foreach without 'in'".into()));
            }
        }
        let names: Vec<Token> = vars.iter().filter(|t| matches!(t, Token::Cs(_))).cloned().collect();
        // the list: a group, or a macro that holds it
        let list_tokens = match self.tex.next_nonspace() {
            Some(Token::Char(_, Cat::Begin)) => self.tex.read_group_body()?,
            Some(t) => vec![t],
            None => Vec::new(),
        };
        let list = self.tex.expand_fully(list_tokens)?;
        let items = foreach_items(&list);
        // options after the list too: \foreach \x in {…} [count=\i]
        if let Some(t) = self.tex.next_nonspace() {
            if t.is_char('[') {
                opts.extend(tex::string_tokens(","));
                opts.extend(self.tex.read_until_char(']', false)?);
            } else {
                self.tex.back(t);
            }
        }
        let body = self.next_statement()?;
        let options = crate::keys::split_list(&opts);
        let mut count: Option<(Token, i64)> = None;
        let mut remember: Vec<(Token, Token, Option<Vec<Token>>)> = Vec::new();
        let mut evaluate: Vec<(Token, Option<Token>, Option<Vec<Token>>)> = Vec::new();
        for o in &options {
            let v = o.value.clone().unwrap_or_default();
            let parts: Vec<Token> = v.iter().filter(|t| !t.is_space()).cloned().collect();
            match o.key.as_str() {
                "count" => {
                    // count=\i from 0
                    let name = parts.first().cloned();
                    let s = tex::tokens_to_string(&v);
                    let start = s.split("from").nth(1).and_then(|x| x.trim().parse().ok()).unwrap_or(1);
                    if let Some(n) = name {
                        count = Some((n, start));
                    }
                }
                "remember" => {
                    let s = tex::tokens_to_string(&v);
                    let initially = s.split("initially").nth(1).map(|x| tex::string_tokens(x.trim().trim_start_matches('(').trim_end_matches(')')));
                    let cs: Vec<Token> = v.iter().filter(|t| matches!(t, Token::Cs(_))).cloned().collect();
                    if cs.len() >= 2 {
                        remember.push((cs[0].clone(), cs[1].clone(), initially));
                    }
                }
                "evaluate" => {
                    let s = tex::tokens_to_string(&v);
                    let cs: Vec<Token> = v.iter().filter(|t| matches!(t, Token::Cs(_))).cloned().collect();
                    let using = s.split("using").nth(1).map(|x| tex::string_tokens(x.trim()));
                    let target = if s.contains(" as ") { cs.get(1).cloned() } else { None };
                    if let Some(var) = cs.first() {
                        evaluate.push((var.clone(), target, using));
                    }
                }
                "parse" => {}
                _ => {}
            }
        }
        if self.depth >= MAX_DEPTH {
            return Err(Error::Figure(format!("scope/\\foreach nesting went past {MAX_DEPTH} levels")));
        }
        self.depth += 1;
        // remembered values live across iterations
        let mut memory: Vec<Option<Vec<Token>>> = remember.iter().map(|r| r.2.clone()).collect();
        for (k, item) in items.iter().enumerate() {
            self.count_step()?;
            self.begin_group();
            let parts = split_slash(item);
            for (i, name) in names.iter().enumerate() {
                let v = parts.get(i).or(parts.last()).cloned().unwrap_or_default();
                self.tex.set_meaning(name, macro_tokens(v), false);
            }
            if let Some((n, start)) = &count {
                self.tex.set_meaning(n, macro_from(&(start + k as i64).to_string()), false);
            }
            for ((_, as_name, _), m) in remember.iter().zip(&memory) {
                if let Some(v) = m {
                    self.tex.set_meaning(as_name, macro_tokens(v.clone()), false);
                }
            }
            for (var, target, using) in &evaluate {
                let expr = match using {
                    Some(u) => u.clone(),
                    None => vec![var.clone()],
                };
                let v = self.number_of(expr)?;
                let dest = target.clone().unwrap_or_else(|| var.clone());
                self.tex.set_meaning(&dest, macro_from(&pgf_number(v)), false);
            }
            let r = self.run_tokens(body.clone());
            for (i, (var, _, _)) in remember.iter().enumerate() {
                if let Meaning::Macro(m) = self.tex.meaning(var) {
                    memory[i] = Some(tex::substitute(&m.body, &[]));
                }
            }
            self.end_group();
            r?;
        }
        self.depth -= 1;
        Ok(())
    }
}

impl Default for Engine {
    fn default() -> Self {
        Engine::new()
    }
}

/// Errors that stop the whole figure: they guard against hostile input.
pub fn is_limit(e: &Error) -> bool {
    let s = e.to_string();
    s.contains("figure too large") || s.contains("scope/\\foreach nesting went past") || s.contains("picture options:") || s.contains("scope options:")
}

/// A macro without parameters holding text.
pub fn macro_from(s: &str) -> Meaning {
    macro_tokens(tex::string_tokens(s))
}

pub fn macro_tokens(v: Vec<Token>) -> Meaning {
    Meaning::Macro(Rc::new(tex::Macro { params: Vec::new(), body: v.into_iter().map(tex::Body::Tok).collect(), optional: None, protected: false }))
}

/// Text read as TeX reads it (control sequences, braces), for internal use.
pub fn tikra_tokens(text: &str) -> Vec<Token> {
    let mut t = Tex::new_lexer();
    t.push_text(text);
    let mut v = Vec::new();
    while let Some(x) = t.get_token() {
        v.push(x);
    }
    v
}

/// A number as pgfmath writes one: `2`, `0.5`, `-3.14159`.
pub fn pgf_number(v: f64) -> String {
    let r = (v * 100000.0).round() / 100000.0;
    if r.fract() == 0.0 && r.abs() < 1e15 {
        format!("{}", r as i64)
    } else {
        format!("{r}")
    }
}

fn font_size(name: &str) -> f64 {
    match name {
        "tiny" => 5.0,
        "scriptsize" => 7.0,
        "footnotesize" => 8.0,
        "small" => 9.0,
        "large" => 12.0,
        "Large" => 14.4,
        "LARGE" => 17.28,
        "huge" => 20.74,
        "Huge" => 24.88,
        _ => 10.0,
    }
}

/// `1/a` → [`1`, `a`]: an item split at its top-level slashes.
fn split_slash(item: &[Token]) -> Vec<Vec<Token>> {
    let mut parts = vec![Vec::new()];
    let mut depth = 0usize;
    for t in item {
        match t {
            Token::Char(_, Cat::Begin) => depth += 1,
            Token::Char(_, Cat::End) => depth = depth.saturating_sub(1),
            Token::Char('/', _) if depth == 0 => {
                parts.push(Vec::new());
                continue;
            }
            _ => {}
        }
        if let Some(p) = parts.last_mut() {
            p.push(t.clone());
        }
    }
    parts.into_iter().map(|p| tex::strip_outer_braces(crate::keys::trim(&p).to_vec())).collect()
}

/// The items of a `\foreach` list, with `...` ranges expanded as pgffor does (numbers and letters).
pub fn foreach_items(list: &[Token]) -> Vec<Vec<Token>> {
    let mut raw: Vec<Vec<Token>> = Vec::new();
    let mut current = Vec::new();
    let mut depth = 0usize;
    for t in list {
        match t {
            Token::Char(_, Cat::Begin) => depth += 1,
            Token::Char(_, Cat::End) => depth = depth.saturating_sub(1),
            Token::Char(',', _) if depth == 0 => {
                raw.push(std::mem::take(&mut current));
                continue;
            }
            _ => {}
        }
        current.push(t.clone());
    }
    raw.push(current);
    let raw: Vec<Vec<Token>> = raw.into_iter().map(|p| tex::strip_outer_braces(crate::keys::trim(&p).to_vec())).filter(|p| !p.is_empty()).collect();
    let text = |v: &[Token]| tex::tokens_to_string(v).trim().to_string();
    let mut out: Vec<Vec<Token>> = Vec::new();
    let mut i = 0;
    while i < raw.len() {
        if text(&raw[i]) == "..." && !out.is_empty() && i + 1 < raw.len() {
            let end_s = text(&raw[i + 1]);
            let last_s = text(&out[out.len() - 1]);
            let before = if out.len() >= 2 { Some(text(&out[out.len() - 2])) } else { None };
            // letters: a,...,e
            let letter = |s: &str| if s.chars().count() == 1 && s.chars().all(|c| c.is_ascii_alphabetic()) { s.chars().next() } else { None };
            if let (Some(a), Some(b)) = (letter(&last_s), letter(&end_s)) {
                let step: i32 = before.as_deref().and_then(letter).map_or(1, |p| a as i32 - p as i32).max(1);
                let mut c = a as i32 + step;
                while c <= b as i32 && out.len() < 100_000 {
                    out.push(tex::string_tokens(&char::from_u32(c as u32).unwrap_or('?').to_string()));
                    c += step;
                }
                i += 2;
                continue;
            }
            let (Ok(last), Ok(end)) = (last_s.parse::<f64>(), end_s.parse::<f64>()) else {
                i += 1;
                continue;
            };
            let step = before.and_then(|b| b.parse::<f64>().ok()).map_or(if end >= last { 1.0 } else { -1.0 }, |b| last - b);
            if step.abs() > 1e-12 {
                let mut k = 1.0;
                loop {
                    let v = last + step * k;
                    if (step > 0.0 && v > end + 1e-9) || (step < 0.0 && v < end - 1e-9) || out.len() > 100_000 {
                        break;
                    }
                    out.push(tex::string_tokens(&pgf_number(v)));
                    k += 1.0;
                }
            }
            i += 2;
            continue;
        }
        out.push(raw[i].clone());
        i += 1;
    }
    out
}

/// TeX's interword space and line spacing for Computer Modern at 10 pt, in cm.
const SPACE: f64 = 3.33333 * crate::PT;
const BASELINE_SKIP: f64 = 12.0 * crate::PT;
const LINE_SKIP: f64 = 1.0 * crate::PT;

/// Lays the pictures out as TeX sets boxes in a paragraph: side by side on a baseline, a space
/// where the source had one, a new line at each paragraph; then one canvas with all of them.
fn typeset(boxes: &[PictureBox], layout: &[Layout], margin: f64) -> Canvas {
    let mut out = Canvas { margin, ..Canvas::default() };
    let mut lines: Vec<Vec<Layout>> = vec![Vec::new()];
    for l in layout {
        match l {
            Layout::Par => lines.push(Vec::new()),
            other => lines.last_mut().expect("a line").push(*other),
        }
    }
    let mut baseline_y = 0.0;
    let mut previous_depth: Option<f64> = None;
    for line in lines.iter().filter(|l| l.iter().any(|x| matches!(x, Layout::Picture(_)))) {
        let (mut height, mut depth) = (0.0f64, 0.0f64);
        for l in line {
            if let Layout::Picture(i) = l {
                let b = &boxes[*i];
                let (lo, hi) = (b.canvas.min.map_or(0.0, |m| m.y), b.canvas.max.map_or(0.0, |m| m.y));
                height = height.max(hi - b.baseline);
                depth = depth.max(b.baseline - lo);
            }
        }
        if let Some(d) = previous_depth {
            let gap = BASELINE_SKIP - d - height;
            baseline_y -= if gap >= 0.0 { BASELINE_SKIP } else { d + LINE_SKIP + height };
        }
        let mut x = 0.0;
        let mut started = false;
        for l in line {
            match l {
                Layout::Space if started => x += SPACE,
                Layout::Picture(i) => {
                    let b = &boxes[*i];
                    let (Some(min), Some(max)) = (b.canvas.min, b.canvas.max) else {
                        started = true;
                        continue;
                    };
                    let d = Point::new(x - min.x, baseline_y - b.baseline);
                    for item in &b.canvas.items {
                        let mut it = item.clone();
                        it.translate(d);
                        out.items.push(it);
                    }
                    out.include(min.plus(d));
                    out.include(max.plus(d));
                    x += max.x - min.x;
                    started = true;
                }
                _ => {}
            }
        }
        previous_depth = Some(depth);
    }
    out
}

/// Runs a whole document (or figure) and returns the canvas and the warnings.
pub fn run_document(code: &str) -> Result<(Canvas, Vec<String>), Error> {
    let mut e = Engine::new();
    // tikra's margin around a figure, unless a standalone border says otherwise
    e.canvas.margin = 2.0 * crate::PT;
    e.tex.push_text(code);
    e.run()?;
    // anything left open is closed
    while e.picture_depth > 0 {
        e.end_picture();
    }
    while !e.scopes.is_empty() {
        e.end_scope();
    }
    let margin = e.canvas.margin;
    e.canvas = typeset(&e.boxes, &e.layout, margin);
    let mut warnings = std::mem::take(&mut e.tex.warnings);
    warnings.dedup();
    let _ = Transform::default();
    Ok((e.canvas, warnings))
}
