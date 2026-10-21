//! Running pgfkeys: key lists, handlers (`.style`, `.code`, `.default`, `.initial`, `.store in`,
//! `.is choice`…), TikZ's `.unknown` (a color, an arrow specification, a shape name), and the keys
//! that TikZ defines with code, done natively. Keys that TikZ defines as styles are defined here the
//! same way, from TikZ's own definitions.

use std::rc::Rc;

use crate::canvas::{Cap, Join};
use crate::keys::{self, macro_name};
use crate::scene::{Point, Transform};
use crate::tex::{self, Body, Cat, Macro, Meaning, Param, Token};
use crate::{Error, PT};

use super::{macro_from, macro_tokens, tikra_tokens, Engine, NATIVE_KEY_BASE};

pub type KeyFn = fn(&mut Engine, Option<Vec<Token>>) -> Result<(), Error>;

fn group(v: Vec<Token>) -> Vec<Token> {
    let mut out = Vec::with_capacity(v.len() + 2);
    out.push(Token::Char('{', Cat::Begin));
    out.extend(v);
    out.push(Token::Char('}', Cat::End));
    out
}

fn is_novalue(v: &[Token]) -> bool {
    let v: Vec<&Token> = v.iter().filter(|t| !t.is_space()).collect();
    v.len() == 1 && v[0].is_cs("pgfkeysnovalue")
}

impl Engine {
    /// Runs a key list now, with `path` as the default path (or the current one).
    pub fn process_keys(&mut self, list: Vec<Token>, path: Option<&str>) -> Result<(), Error> {
        let tokens = self.key_invocations(list, path, true);
        self.run_tokens(tokens)
    }

    /// `\tikzset`, `\pgfkeys`, `\pgfkeysalso`: the list's keys, one after the other.
    pub fn keys_command(&mut self, list: Vec<Token>, path: Option<&str>, restore: bool) -> Result<(), Error> {
        let tokens = self.key_invocations(list, path, restore);
        self.run_tokens(tokens)
    }

    fn key_invocations(&mut self, list: Vec<Token>, path: Option<&str>, restore: bool) -> Vec<Token> {
        let old = self.st.key_path.clone();
        let mut out = Vec::new();
        if let Some(p) = path {
            out.push(Token::cs("tikra@setpath"));
            out.extend(group(tex::string_tokens(p)));
        }
        for item in keys::split_list(&list) {
            out.push(Token::cs("tikra@key"));
            out.extend(group(tikra_tokens(&item.key)));
            out.extend(group(item.value.unwrap_or_else(|| vec![Token::cs("pgfkeysnovalue")])));
        }
        if restore && path.is_some() {
            out.push(Token::cs("tikra@setpath"));
            out.extend(group(tex::string_tokens(&old)));
        }
        out
    }

    /// `\tikra@key{key}{value}`: one key.
    pub fn invoke_key_command(&mut self) -> Result<(), Error> {
        let key = self.argument_string()?;
        let value = self.argument()?;
        let value = if is_novalue(&value) { None } else { Some(value) };
        self.invoke(&key, value)
    }

    pub fn invoke(&mut self, key: &str, value: Option<Vec<Token>>) -> Result<(), Error> {
        self.keys_run += 1;
        if self.keys_run > super::MAX_KEYS {
            if self.keys_run == super::MAX_KEYS + 1 {
                self.warn(format!("more than {} options (styles that multiply?): the rest ignored", super::MAX_KEYS));
            }
            return Ok(());
        }
        let path = keys::full_path(&self.st.key_path, key);
        // a key named like a handler (`characters/.append`) is a key: pgfkeys looks for keys first
        if let Some((base, h)) = keys::handler(&path).filter(|_| !self.tex.is_defined(&macro_name(&format!("{path}/.@cmd")))) {
            let (mut base, h) = (base.to_string(), h.to_string());
            // `/handler config=full or existing`: a handler that changes a key that does not exist
            // goes to the key of the same name on the search path (pgfplots' `every mark/.append style`)
            if matches!(h.as_str(), ".append style" | ".prefix style" | ".append code" | ".prefix code" | ".append" | ".prefix" | ".add") && !self.key_exists(&base) {
                if let Some(list) = keys::value(&self.tex, &format!("{}/.@searchalso", keys::parent(&base))) {
                    let name = keys::name(&base).to_string();
                    let found = tex::tokens_to_string(&list).split(',').map(|p| format!("{}/{name}", p.trim().trim_end_matches('/'))).find(|other| self.key_exists(other));
                    if let Some(other) = found {
                        base = other;
                    }
                }
            }
            return self.handler(&base, &h, value);
        }
        if self.run_key(&path, value.clone())? {
            return Ok(());
        }
        self.unknown(&path, value)
    }

    /// Whether a key has code or a value.
    pub fn key_exists(&self, path: &str) -> bool {
        self.tex.is_defined(&macro_name(path)) || self.tex.is_defined(&macro_name(&format!("{path}/.@cmd")))
    }

    /// Runs the key at `path` if it is defined; false if it is not.
    pub fn run_key(&mut self, path: &str, value: Option<Vec<Token>>) -> Result<bool, Error> {
        let cmd = self.tex.meaning_of(&macro_name(&format!("{path}/.@cmd")));
        let value = value.or_else(|| keys::value(&self.tex, &format!("{path}/.@def")));
        match cmd {
            Meaning::Native(id) if id >= NATIVE_KEY_BASE => {
                let f = self.key_fns[(id - NATIVE_KEY_BASE) as usize];
                self.current_key.clear();
                self.current_key.push_str(path);
                let v = value.filter(|v| !is_novalue(v));
                f(self, v)?;
                Ok(true)
            }
            Meaning::Macro(m) => {
                let n = m.params.iter().filter(|p| matches!(p, Param::Arg(_))).count();
                let v = value.unwrap_or_else(|| vec![Token::cs("pgfkeysnovalue")]);
                let args = if n <= 1 { vec![v] } else { split_arguments(&v, n) };
                let code = tex::substitute(&m.body, &args);
                self.run_tokens(code)?;
                Ok(true)
            }
            _ => {
                if self.tex.is_defined(&macro_name(path)) {
                    match value {
                        Some(v) => self.tex.set_meaning(&Token::cs(&macro_name(path)), macro_tokens(v), false),
                        None => {
                            // a stored value used as a key: its value is "typeset" (inserted)
                            if let Some(v) = keys::value(&self.tex, path) {
                                self.tex.push_tokens(v);
                            }
                        }
                    }
                    return Ok(true);
                }
                Ok(false)
            }
        }
    }

    fn unknown(&mut self, path: &str, value: Option<Vec<Token>>) -> Result<(), Error> {
        let parent = keys::parent(path).to_string();
        let name = keys::name(path).to_string();
        // the parent's .unknown handler (TikZ's: a color, an arrow specification, a shape)
        let handler = macro_name(&format!("{parent}/.unknown/.@cmd"));
        if self.tex.is_defined(&handler) {
            self.tex.define_string("pgfkeyscurrentname", &name, false);
            self.tex.define_string("pgfkeyscurrentkey", path, false);
            self.tex.set_meaning(&Token::cs("pgfkeyscurrentvalue"), macro_tokens(value.clone().unwrap_or_else(|| vec![Token::cs("pgfkeysnovalue")])), false);
            if self.run_key(&format!("{parent}/.unknown"), value.clone())? {
                return Ok(());
            }
        }
        // .search also
        if let Some(list) = keys::value(&self.tex, &format!("{parent}/.@searchalso")) {
            let text = tex::tokens_to_string(&list);
            for p in text.split(',').map(str::trim).filter(|p| !p.is_empty()) {
                let other = format!("{}/{name}", p.trim_end_matches('/'));
                if self.run_key(&other, value.clone())? {
                    return Ok(());
                }
            }
        }
        self.unknown_warning(&name, path);
        Ok(())
    }

    fn unknown_warning(&mut self, name: &str, path: &str) {
        let _ = path;
        self.warn(format!("option '{name}' not supported: ignored"));
    }

    /// `/tikz/.unknown`: a color, an arrow specification or a shape name; otherwise a `/pgf/` key.
    fn tikz_unknown(&mut self, value: Option<Vec<Token>>) -> Result<(), Error> {
        let name = keys::value(&self.tex, "pgfkeyscurrentname").or_else(|| match self.tex.meaning_of("pgfkeyscurrentname") {
            Meaning::Macro(m) => Some(tex::substitute(&m.body, &[])),
            _ => None,
        });
        let name = name.map(|v| tex::tokens_to_string(&v)).unwrap_or_default();
        if name.starts_with('"') {
            return self.quotes(&name, value);
        }
        if value.is_none() {
            if let Some(c) = self.color(&name) {
                self.st.color = c;
                self.st.draw_color = None;
                self.st.fill_color = None;
                self.st.text_color = None;
                return Ok(());
            }
            if let Some((a, b)) = super::arrows::parse_arrows(&name, &self.st.tips) {
                self.st.start_arrow = a;
                self.st.end_arrow = b;
                return Ok(());
            }
            if super::nodes::is_shape(&name) {
                self.tex.define_string(&macro_name("/tikz/shape"), &name, false);
                return Ok(());
            }
        }
        let pgf = format!("/pgf/{name}");
        if self.run_key(&pgf, value)? {
            return Ok(());
        }
        self.unknown_warning(&name, &format!("/tikz/{name}"));
        Ok(())
    }

    /// The quotes library: `"text" options` makes a label in node options, and a node on the path in
    /// `to`/`edge` options; `'` after the quotes means `swap`.
    fn quotes(&mut self, key: &str, value: Option<Vec<Token>>) -> Result<(), Error> {
        // pgfkeys split `"x=1"` at its `=`: put it back together
        let mut whole = key.to_string();
        if let Some(v) = value {
            whole.push('=');
            whole.push_str(&tex::tokens_to_string(&v));
        }
        let Some(end) = whole[1..].find('"').map(|i| i + 1) else {
            self.warn(format!("quotes: closing '\"' missing in {whole}"));
            return Ok(());
        };
        let text = whole[1..end].to_string();
        let mut options = whole[end + 1..].trim().to_string();
        if let Some(rest) = options.strip_prefix('\'') {
            options = format!("swap,{}", rest.trim());
        }
        // `"text" above, red` is one key in a list only up to the comma: the rest are options
        let options = options.split_whitespace().collect::<Vec<_>>().join(" ");
        if self.st.quotes_in_pic {
            self.st.pic_text = Some((tikra_tokens(&text), tikra_tokens(&options)));
        } else if self.st.quotes_on_path {
            let mut o = tikra_tokens("every edge quotes/.try,");
            o.extend(tikra_tokens(&options));
            self.st.quoted.push((tikra_tokens(&text), o));
        } else {
            let spec = format!("[every label quotes/.try,{options}]{text}");
            self.st.node.labels.push((tikra_tokens(&spec), false));
        }
        Ok(())
    }

    // ---------------------------------------------------------------- handlers

    fn define_code(&mut self, path: &str, n: usize, body: Vec<Token>) {
        let params = (1..=n as u8).map(Param::Arg).collect();
        let m = Macro { params, body: tex::parse_body(body), optional: None, protected: false };
        self.tex.set_meaning(&Token::cs(&macro_name(&format!("{path}/.@cmd"))), Meaning::Macro(Rc::new(m)), false);
    }

    /// The code of a key, as tokens with `#1`… in place, if it is a macro.
    fn code_tokens(&self, path: &str) -> Option<(usize, Vec<Token>)> {
        match self.tex.meaning_of(&macro_name(&format!("{path}/.@cmd"))) {
            Meaning::Macro(m) => {
                let n = m.params.len();
                let mut v = Vec::new();
                for b in &m.body {
                    match b {
                        Body::Tok(t) => {
                            if matches!(t, Token::Char(_, Cat::Param)) {
                                // a ## inside: keep it doubled so the code stays the same once defined again
                                v.push(t.clone());
                            }
                            v.push(t.clone());
                        }
                        Body::Arg(k) => {
                            v.push(Token::Char('#', Cat::Param));
                            v.push(Token::other(char::from(b'0' + k)));
                        }
                    }
                }
                // single # were doubled above; undo for plain tokens
                let mut out = Vec::with_capacity(v.len());
                let mut i = 0;
                while i < v.len() {
                    if matches!(v[i], Token::Char(_, Cat::Param))
                        && matches!(v.get(i + 1), Some(Token::Char(_, Cat::Param)))
                        && !matches!(v.get(i + 2), Some(Token::Char(c, _)) if c.is_ascii_digit())
                    {
                        out.push(v[i].clone());
                        i += 2;
                        continue;
                    }
                    out.push(v[i].clone());
                    i += 1;
                }
                Some((n, out))
            }
            _ => None,
        }
    }

    fn style_body(value: Vec<Token>) -> Vec<Token> {
        let mut b = vec![Token::cs("pgfkeysalso")];
        b.extend(group(value));
        b
    }

    fn handler(&mut self, base: &str, h: &str, value: Option<Vec<Token>>) -> Result<(), Error> {
        let v = value.clone().unwrap_or_default();
        match h {
            ".cd" => self.st.key_path = base.to_string(),
            ".style" | ".estyle" => {
                let v = if h == ".estyle" { self.tex.expand_fully(v)? } else { v };
                self.define_code(base, 1, Self::style_body(v));
            }
            ".append style" | ".prefix style" => {
                let (n, old) = self.code_tokens(base).unwrap_or((1, Vec::new()));
                let new = Self::style_body(v);
                let body = if h == ".append style" { [old, new].concat() } else { [new, old].concat() };
                self.define_code(base, n.max(1), body);
            }
            ".code" | ".ecode" => {
                let v = if h == ".ecode" { self.tex.expand_fully(v)? } else { v };
                self.define_code(base, 1, v);
            }
            ".append code" | ".prefix code" => {
                let (n, old) = self.code_tokens(base).unwrap_or((1, Vec::new()));
                let body = if h == ".append code" { [old, v].concat() } else { [v, old].concat() };
                self.define_code(base, n.max(1), body);
            }
            ".code 2 args" | ".style 2 args" | ".code n args" | ".style n args" | ".ecode 2 args" | ".estyle 2 args" => {
                let (n, body) = if h.contains("n args") {
                    let args = split_arguments(&v, 2);
                    let n: usize = tex::tokens_to_string(&args[0]).trim().parse().unwrap_or(1);
                    (n, args[1].clone())
                } else {
                    (2, v)
                };
                let body = if h.contains("style") { Self::style_body(body) } else { body };
                self.define_code(base, n, body);
            }
            ".tip" => {
                // `name/.tip=spec`: \pgfdeclarearrow{name=name, means=spec}, the name being the last part of the path
                let name = base.rsplit('/').next().unwrap_or(base).trim().to_string();
                let means = tex::tokens_to_string(&v);
                std::rc::Rc::make_mut(&mut self.st.tips).declare(&name, &means);
            }
            ".default" => self.tex.set_meaning(&Token::cs(&macro_name(&format!("{base}/.@def"))), macro_tokens(v), false),
            ".initial" => self.tex.set_meaning(&Token::cs(&macro_name(base)), macro_tokens(v), false),
            ".value required" | ".value forbidden" | ".show code" | ".show value" | ".is family" | ".activate family" | ".deactivate family" | ".belongs to family" => {}
            ".store in" | ".estore in" => {
                let target: Vec<Token> = v.into_iter().filter(|t| !t.is_space()).collect();
                let mut body = vec![Token::cs(if h == ".store in" { "def" } else { "edef" })];
                body.extend(target);
                body.extend(group(vec![Token::Char('#', Cat::Param), Token::other('1')]));
                self.define_code(base, 1, body);
            }
            ".is if" => {
                let name = tex::tokens_to_string(&v).trim().to_string();
                // key=true runs \nametrue
                let body = vec![Token::cs("csname"), Token::other(' ')].into_iter().take(1).collect::<Vec<_>>();
                let mut body = body;
                body.extend(tex::string_tokens(&name));
                body.push(Token::Char('#', Cat::Param));
                body.push(Token::other('1'));
                body.push(Token::cs("endcsname"));
                self.define_code(base, 1, body);
                self.tex.set_meaning(&Token::cs(&macro_name(&format!("{base}/.@def"))), macro_from("true"), false);
            }
            ".is choice" => {
                // base=choice runs base/choice
                let mut body = vec![Token::cs("pgfkeysalso")];
                let mut inner = tex::string_tokens(&format!("{base}/"));
                inner.push(Token::Char('#', Cat::Param));
                inner.push(Token::other('1'));
                body.extend(group(inner));
                self.define_code(base, 1, body);
            }
            ".search also" => {
                let s = tex::tokens_to_string(&v);
                self.tex.define_string(&macro_name(&format!("{base}/.@searchalso")), &s, false);
            }
            ".try" | ".retry" | ".lastretry" => {
                let _ = self.run_key(base, value)?;
            }
            ".expand once" | ".expand twice" | ".expanded" => {
                let v = self.tex.expand_fully(v)?;
                if !self.run_key(base, Some(v.clone()))? {
                    self.unknown(base, Some(v))?;
                }
            }
            ".get" => {
                if let Some(target) = v.into_iter().find(|t| !t.is_space()) {
                    let val = keys::value(&self.tex, base).unwrap_or_default();
                    self.tex.set_meaning(&target, macro_tokens(val), false);
                }
            }
            ".add" => {
                let args = split_arguments(&v, 2);
                let old = keys::value(&self.tex, base).unwrap_or_default();
                let new = [args[0].clone(), old, args[1].clone()].concat();
                self.tex.set_meaning(&Token::cs(&macro_name(base)), macro_tokens(new), false);
            }
            ".append" | ".prefix" => {
                let old = keys::value(&self.tex, base).unwrap_or_default();
                let new = if h == ".append" { [old, v].concat() } else { [v, old].concat() };
                self.tex.set_meaning(&Token::cs(&macro_name(base)), macro_tokens(new), false);
            }
            ".forward to" => {
                let target = tex::tokens_to_string(&v).trim().to_string();
                let mut inner = tex::string_tokens(&format!("{target}="));
                inner.extend(group(vec![Token::Char('#', Cat::Param), Token::other('1')]));
                self.define_code(base, 1, Self::style_body(inner));
            }
            ".list" => {
                for item in super::foreach_items(&v) {
                    if !self.run_key(base, Some(item.clone()))? {
                        self.unknown(base, Some(item))?;
                    }
                }
            }
            ".pic" => {
                // name/.pic={code}: the pic type `name`, under /tikz/pics
                let name = base.trim_start_matches("/tikz/").trim_start_matches('/');
                let mut body = tikra_tokens("code=");
                body.extend(group(v));
                self.define_code(&format!("/tikz/pics/{name}"), 1, Self::style_body(body));
            }
            ".unknown" => {
                // `/foo/.unknown/.code={…}` reaches here as a handler of the path /foo/.unknown
                self.define_code(&format!("{base}/.unknown"), 1, v);
            }
            other => {
                // a user handler: /handlers/<name>/.code
                let user = format!("/handlers/{}", other);
                self.tex.define_string("pgfkeyscurrentpath", base, false);
                if !self.run_key(&user, value)? {
                    self.warn(format!("key handler '{other}' not supported (in '{base}/{other}'): ignored"));
                }
            }
        }
        Ok(())
    }
}

/// `{a}{b}` (or `ab`) as `n` arguments, TeX-style.
pub fn split_arguments(v: &[Token], n: usize) -> Vec<Vec<Token>> {
    let mut out = Vec::new();
    let mut i = 0;
    while out.len() < n {
        while i < v.len() && v[i].is_space() {
            i += 1;
        }
        if i >= v.len() {
            out.push(Vec::new());
            continue;
        }
        if matches!(v[i], Token::Char(_, Cat::Begin)) {
            let mut depth = 0usize;
            let start = i;
            while i < v.len() {
                match v[i] {
                    Token::Char(_, Cat::Begin) => depth += 1,
                    Token::Char(_, Cat::End) => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                i += 1;
            }
            out.push(v[start + 1..i.min(v.len())].to_vec());
            i += 1;
        } else {
            out.push(vec![v[i].clone()]);
            i += 1;
        }
    }
    out
}

// -------------------------------------------------------------------- the native keys

fn text(e: &mut Engine, v: &Option<Vec<Token>>) -> Result<String, Error> {
    Ok(match v {
        Some(v) => e.tex.expand_to_string(v.clone())?.trim().to_string(),
        None => String::new(),
    })
}

fn length(e: &mut Engine, v: &Option<Vec<Token>>) -> Result<Option<f64>, Error> {
    match v {
        Some(v) => Ok(Some(e.length_of(v.clone())?)),
        None => Ok(None),
    }
}

fn number(e: &mut Engine, v: &Option<Vec<Token>>) -> Result<Option<f64>, Error> {
    match v {
        Some(v) => Ok(Some(e.number_of(v.clone())?)),
        None => Ok(None),
    }
}

fn set_value(e: &mut Engine, path: &str, s: &str) {
    e.tex.define_string(&macro_name(path), s, false);
}

/// A transformation added to those of the options (`\tikz@addtransform`).
fn add_transform(e: &mut Engine, t: Transform) {
    e.st.pending = e.st.pending.compose(&t);
}

/// A point for `shift=`: in the coordinate system the shift happens in.
fn shift_vector(e: &mut Engine, v: &Option<Vec<Token>>) -> Result<Point, Error> {
    let s = text(e, v)?;
    let inner = s.trim().trim_start_matches('{').trim_end_matches('}').trim().trim_start_matches('(').trim_end_matches(')').to_string();
    let p = e.coordinate(&inner)?.p;
    let full = e.st.transform.compose(&e.st.pending);
    Ok(crate::reader::invert(&full).apply(p))
}

fn dash_from(e: &mut Engine, spec: &str) -> Result<Option<Vec<f64>>, Error> {
    // on 3pt off 2pt on \pgflinewidth off 2pt
    let words: Vec<&str> = spec.split_whitespace().collect();
    let mut v = Vec::new();
    let mut i = 0;
    while i + 1 < words.len() {
        match words[i] {
            "on" | "off" => {
                let w = words[i + 1];
                let len = if w.contains("pgflinewidth") {
                    let f: f64 = w.replace("\\pgflinewidth", "").trim_end_matches('*').parse().unwrap_or(1.0);
                    f * e.st.line_width
                } else {
                    e.length(w)?
                };
                v.push(len);
                i += 2;
            }
            _ => i += 1,
        }
    }
    Ok(if v.is_empty() { None } else { Some(v) })
}

macro_rules! keyfn {
    ($name:ident, |$e:ident, $v:ident| $body:expr) => {
        fn $name($e: &mut Engine, $v: Option<Vec<Token>>) -> Result<(), Error> {
            $body
        }
    };
}

keyfn!(k_draw, |e, v| {
    let s = text(e, &v)?;
    match s.as_str() {
        "" => e.st.mode.draw = true,
        "none" => e.st.mode.draw = false,
        c => {
            e.st.mode.draw = true;
            match e.color(c) {
                Some(c) => e.st.draw_color = Some(c),
                None => e.warn(format!("unknown color '{c}'")),
            }
        }
    }
    Ok(())
});

keyfn!(k_fill, |e, v| {
    let s = text(e, &v)?;
    match s.as_str() {
        "" => e.st.mode.fill = true,
        "none" => e.st.mode.fill = false,
        c => {
            e.st.mode.fill = true;
            match e.color(c) {
                Some(c) => e.st.fill_color = Some(c),
                None => e.warn(format!("unknown color '{c}'")),
            }
        }
    }
    Ok(())
});

keyfn!(k_color, |e, v| {
    let s = text(e, &v)?;
    match e.color(&s) {
        Some(c) => {
            e.st.color = c;
            e.st.draw_color = None;
            e.st.fill_color = None;
            e.st.text_color = None;
        }
        None => e.warn(format!("unknown color '{s}'")),
    }
    Ok(())
});

keyfn!(k_text, |e, v| {
    let s = text(e, &v)?;
    match e.color(&s) {
        Some(c) => e.st.text_color = Some(c),
        None => e.warn(format!("unknown color '{s}'")),
    }
    Ok(())
});

keyfn!(k_clip, |e, _v| {
    e.st.mode.clip = true;
    Ok(())
});

keyfn!(k_bounding_box, |e, _v| {
    e.st.mode.bounding_box = true;
    Ok(())
});

keyfn!(k_shade, |e, _v| {
    e.st.mode.shade = true;
    Ok(())
});

keyfn!(k_pattern, |e, v| {
    let s = text(e, &v)?;
    e.st.mode.pattern = true;
    e.st.pattern_name = Some(if s.is_empty() { "north east lines".into() } else { s });
    Ok(())
});

keyfn!(k_pattern_color, |e, v| {
    let s = text(e, &v)?;
    e.st.pattern_color = e.color(&s);
    Ok(())
});

keyfn!(k_line_width, |e, v| {
    if let Some(w) = length(e, &v)? {
        e.st.line_width = w;
        // \pgflinewidth follows
        let sp = (w / PT * 65536.0).round() as i64;
        if let Meaning::Dimen(r) = e.tex.meaning_of("pgflinewidth") {
            e.tex.set_dimen(r, sp, false);
        }
    }
    Ok(())
});

keyfn!(k_dash_pattern, |e, v| {
    let s = text(e, &v)?;
    let d = dash_from(e, &s)?;
    let phase = e.st.dash.as_ref().map_or(0.0, |d| d.1);
    e.st.dash = d.map(|d| (d, phase));
    Ok(())
});

keyfn!(k_dash_phase, |e, v| {
    let p = length(e, &v)?.unwrap_or(0.0);
    if let Some(d) = &mut e.st.dash {
        d.1 = p;
    } else {
        e.st.dash = Some((Vec::new(), p));
    }
    Ok(())
});

keyfn!(k_dash, |e, v| {
    // dash=on 2pt off 1pt phase 1pt
    let s = text(e, &v)?;
    if s == "solid" {
        e.st.dash = None;
        return Ok(());
    }
    let (pattern, phase) = match s.split_once("phase") {
        Some((a, b)) => (a.to_string(), e.length(b.trim())?),
        None => (s.clone(), 0.0),
    };
    e.st.dash = dash_from(e, &pattern)?.map(|d| (d, phase));
    Ok(())
});

keyfn!(k_solid, |e, _v| {
    e.st.dash = None;
    Ok(())
});

keyfn!(k_line_cap, |e, v| {
    e.st.cap = match text(e, &v)?.as_str() {
        "round" => Cap::Round,
        "rect" => Cap::Square,
        _ => Cap::Butt,
    };
    Ok(())
});

keyfn!(k_line_join, |e, v| {
    e.st.join = match text(e, &v)?.as_str() {
        "round" => Join::Round,
        "bevel" => Join::Bevel,
        _ => Join::Miter,
    };
    Ok(())
});

keyfn!(k_miter_limit, |e, v| {
    if let Some(m) = number(e, &v)? {
        e.st.miter_limit = m;
    }
    Ok(())
});

keyfn!(k_opacity, |e, v| {
    let o = number(e, &v)?.unwrap_or(1.0);
    e.st.draw_opacity = o;
    e.st.fill_opacity = o;
    e.st.text_opacity = o;
    Ok(())
});

keyfn!(k_draw_opacity, |e, v| {
    e.st.draw_opacity = number(e, &v)?.unwrap_or(1.0);
    Ok(())
});

keyfn!(k_fill_opacity, |e, v| {
    e.st.fill_opacity = number(e, &v)?.unwrap_or(1.0);
    Ok(())
});

keyfn!(k_text_opacity, |e, v| {
    e.st.text_opacity = number(e, &v)?.unwrap_or(1.0);
    Ok(())
});

keyfn!(k_even_odd, |e, _v| {
    e.st.even_odd = true;
    Ok(())
});

keyfn!(k_nonzero, |e, _v| {
    e.st.even_odd = false;
    Ok(())
});

keyfn!(k_rounded, |e, v| {
    e.st.rounded = length(e, &v)?.unwrap_or(4.0 * PT);
    Ok(())
});

keyfn!(k_sharp, |e, _v| {
    e.st.rounded = 0.0;
    Ok(())
});

keyfn!(k_arrows, |e, v| {
    let s = text(e, &v)?;
    match super::arrows::parse_arrows(&s, &e.st.tips) {
        Some((a, b)) => {
            e.st.start_arrow = a;
            e.st.end_arrow = b;
        }
        None => e.warn(format!("arrow specification '{s}' not supported: ignored")),
    }
    Ok(())
});

keyfn!(k_default_arrow, |e, v| {
    // TikZ's `>`: `\pgfdeclarearrow{name=<->, means=…}`
    let s = text(e, &v)?;
    std::rc::Rc::make_mut(&mut e.st.tips).declare("<->", &s);
    Ok(())
});

keyfn!(k_tips, |e, v| {
    let s = text(e, &v)?;
    match super::arrows::TipsMode::from_name(&s) {
        Some(m) => e.st.tips_mode = m,
        None => e.warn(format!("tips={s} not supported: ignored")),
    }
    Ok(())
});

keyfn!(k_shorten_end, |e, v| {
    e.st.shorten_end = length(e, &v)?.unwrap_or(0.0);
    Ok(())
});

keyfn!(k_shorten_start, |e, v| {
    e.st.shorten_start = length(e, &v)?.unwrap_or(0.0);
    Ok(())
});

keyfn!(k_double, |e, v| {
    let s = text(e, &v)?;
    e.st.double = Some(if s.is_empty() { crate::color::Color::WHITE } else { e.color(&s).unwrap_or(crate::color::Color::WHITE) });
    Ok(())
});

keyfn!(k_double_distance, |e, v| {
    e.st.double_distance = length(e, &v)?.unwrap_or(0.6 * PT);
    if e.st.double.is_none() {
        e.st.double = Some(crate::color::Color::WHITE);
    }
    Ok(())
});

keyfn!(k_shift, |e, v| {
    let p = shift_vector(e, &v)?;
    add_transform(e, Transform::translation(p.x, p.y));
    Ok(())
});

keyfn!(k_xshift, |e, v| {
    let d = length(e, &v)?.unwrap_or(0.0);
    add_transform(e, Transform::translation(d, 0.0));
    Ok(())
});

keyfn!(k_yshift, |e, v| {
    let d = length(e, &v)?.unwrap_or(0.0);
    add_transform(e, Transform::translation(0.0, d));
    Ok(())
});

keyfn!(k_scale, |e, v| {
    let s = number(e, &v)?.unwrap_or(1.0);
    add_transform(e, Transform::scaling(s, s));
    Ok(())
});

keyfn!(k_xscale, |e, v| {
    let s = number(e, &v)?.unwrap_or(1.0);
    add_transform(e, Transform::scaling(s, 1.0));
    Ok(())
});

keyfn!(k_yscale, |e, v| {
    let s = number(e, &v)?.unwrap_or(1.0);
    add_transform(e, Transform::scaling(1.0, s));
    Ok(())
});

keyfn!(k_rotate, |e, v| {
    let a = number(e, &v)?.unwrap_or(0.0);
    add_transform(e, Transform::rotation(a));
    Ok(())
});

keyfn!(k_xslant, |e, v| {
    let s = number(e, &v)?.unwrap_or(0.0);
    add_transform(e, Transform { c: s, ..Transform::default() });
    Ok(())
});

keyfn!(k_yslant, |e, v| {
    let s = number(e, &v)?.unwrap_or(0.0);
    add_transform(e, Transform { b: s, ..Transform::default() });
    Ok(())
});

keyfn!(k_rotate_around, |e, v| {
    // rotate around={angle:(point)}
    let s = text(e, &v)?;
    let Some((a, p)) = s.split_once(':') else { return Ok(()) };
    let angle = e.number(a)?;
    let tv = Some(tikra_tokens(p.trim()));
    let c = shift_vector(e, &tv)?;
    add_transform(e, Transform::translation(c.x, c.y).compose(&Transform::rotation(angle)).compose(&Transform::translation(-c.x, -c.y)));
    Ok(())
});

keyfn!(k_cm, |e, v| {
    // cm={a,b,c,d,(tx,ty)}
    let s = text(e, &v)?;
    let parts = super::coords::split_commas(&s);
    if parts.len() >= 5 {
        let n: Vec<f64> = parts[..4].iter().map(|x| e.number(x).unwrap_or(0.0)).collect();
        let tv = Some(tikra_tokens(parts[4..].join(",").trim()));
        let t = shift_vector(e, &tv).unwrap_or_default();
        add_transform(e, Transform { a: n[0], b: n[1], c: n[2], d: n[3], e: t.x, f: t.y });
    }
    Ok(())
});

keyfn!(k_reset_cm, |e, _v| {
    e.st.transform = Transform::default();
    e.st.pending = Transform::default();
    Ok(())
});

fn unit_vector(e: &mut Engine, v: &Option<Vec<Token>>, axis: usize) -> Result<Point, Error> {
    let s = text(e, v)?;
    let t = s.trim();
    if t.starts_with('(') || t.starts_with('{') {
        let inner = crate::options::strip_braces(t).trim().trim_start_matches('(').trim_end_matches(')').to_string();
        // a point, in canvas units without the current transformation
        let saved = e.st.transform;
        e.st.transform = Transform::default();
        let r = e.coordinate(&inner);
        e.st.transform = saved;
        return Ok(r?.p);
    }
    let m = e.length(t)?;
    Ok(match axis {
        0 => Point::new(m, 0.0),
        1 => Point::new(0.0, m),
        _ => Point::new(-std::f64::consts::FRAC_1_SQRT_2 * m, -std::f64::consts::FRAC_1_SQRT_2 * m),
    })
}

keyfn!(k_x, |e, v| {
    e.st.x = unit_vector(e, &v, 0)?;
    Ok(())
});
keyfn!(k_y, |e, v| {
    e.st.y = unit_vector(e, &v, 1)?;
    Ok(())
});
keyfn!(k_z, |e, v| {
    e.st.z = unit_vector(e, &v, 2)?;
    Ok(())
});

keyfn!(k_inner_sep, |e, v| {
    let s = text(e, &v)?;
    set_value(e, "/pgf/inner xsep", &s);
    set_value(e, "/pgf/inner ysep", &s);
    Ok(())
});

keyfn!(k_outer_sep, |e, v| {
    let s = text(e, &v)?;
    let s = if s == "auto" { "0.5\\pgflinewidth".to_string() } else { s };
    set_value(e, "/pgf/outer xsep", &s);
    set_value(e, "/pgf/outer ysep", &s);
    Ok(())
});

keyfn!(k_minimum_size, |e, v| {
    let s = text(e, &v)?;
    set_value(e, "/pgf/minimum width", &s);
    set_value(e, "/pgf/minimum height", &s);
    Ok(())
});

keyfn!(k_font, |e, v| {
    let s = text(e, &v)?;
    // the font commands, for the node's text
    for name in ["tiny", "scriptsize", "footnotesize", "small", "normalsize", "large", "Large", "LARGE", "huge", "Huge"] {
        if s.contains(&format!("\\{name}")) && !s.contains(&format!("\\{name}er")) {
            e.st.font_size = super::font_size(name);
        }
    }
    e.st.font.push_str(&s);
    Ok(())
});

keyfn!(k_node_font, |e, v| k_font(e, v));

keyfn!(k_anchor, |e, v| {
    let s = text(e, &v)?;
    set_value(e, "/tikz/anchor", &s);
    Ok(())
});

/// `above`, `below left=2mm`, `below=1cm of a`, `right=of a.north`: an anchor and an offset or a reference.
fn placement(e: &mut Engine, dir: &str, v: Option<Vec<Token>>) -> Result<(), Error> {
    let anchor = match dir {
        "above" => "south",
        "below" => "north",
        "left" => "east",
        "right" => "west",
        "above left" => "south east",
        "above right" => "south west",
        "below left" => "north east",
        _ => "north west",
    };
    set_value(e, "/tikz/anchor", anchor);
    let (sy, sx) = (
        if dir.contains("above") {
            1.0
        } else if dir.contains("below") {
            -1.0
        } else {
            0.0
        },
        if dir.contains("left") {
            -1.0
        } else if dir.contains("right") {
            1.0
        } else {
            0.0
        },
    );
    let Some(v) = v else { return Ok(()) };
    let s = e.tex.expand_to_string(v)?.trim().to_string();
    if s.is_empty() {
        return Ok(());
    }
    // positioning: `<vertical> and <horizontal> of <node>`; without `of`, an offset from the position
    let (dist, target) = match s.split_once("of ").filter(|_| s.contains("of ")) {
        Some((d, t)) => (d.trim().to_string(), Some(t.trim().to_string())),
        None => (s.clone(), None),
    };
    let (dv, dh) = node_distance(e)?;
    let (v_, h_) = if dist.is_empty() {
        (dv, dh)
    } else if let Some((a, b)) = dist.split_once(" and ") {
        (place_length(e, a, 1)?, place_length(e, b, 0)?)
    } else {
        let m = place_length(e, &dist, 1)?;
        (m, if target.is_some() { m } else { place_length(e, &dist, 0)? })
    };
    let diagonal = sx != 0.0 && sy != 0.0 && !dist.contains(" and ") && !dist.is_empty();
    let offset = if diagonal && target.is_some() {
        // a single distance along a diagonal is measured along it
        Point::new(sx * h_ * std::f64::consts::FRAC_1_SQRT_2, sy * v_ * std::f64::consts::FRAC_1_SQRT_2)
    } else {
        Point::new(sx * h_, sy * v_)
    };
    let Some(target) = target else {
        add_transform(e, Transform::translation(offset.x, offset.y));
        return Ok(());
    };
    // the distance is between borders; the reference is the target's matching anchor
    let target_anchor = match dir {
        "above" => "north",
        "below" => "south",
        "left" => "west",
        "right" => "east",
        "above left" => "north west",
        "above right" => "north east",
        "below left" => "south west",
        _ => "south east",
    };
    let (name, explicit) = match target.split_once('.') {
        Some((n, a)) if e.nodes.contains_key(n.trim()) => (n.trim().to_string(), Some(a.trim().to_string())),
        _ => (target.trim_start_matches('(').trim_end_matches(')').to_string(), None),
    };
    let base = match e.nodes.get(&name) {
        Some(n) => n.anchor(explicit.as_deref().unwrap_or(target_anchor)).unwrap_or(n.center()),
        None => e.coordinate(&name)?.p,
    };
    e.st_node().at = Some(base.plus(offset));
    Ok(())
}

/// A positioning distance: a length, or a number in units of the x or y vector.
fn place_length(e: &mut Engine, text: &str, axis: usize) -> Result<f64, Error> {
    let s = e.math_string(tikra_tokens(text.trim()))?;
    let (v, unit) = crate::expr::eval_length(&s, &e.vars())?;
    Ok(if unit {
        v
    } else if axis == 0 {
        v * e.st.x.norm()
    } else {
        v * e.st.y.norm()
    })
}

fn node_distance(e: &mut Engine) -> Result<(f64, f64), Error> {
    let s = keys::value(&e.tex, "/tikz/node distance").map(|v| tex::tokens_to_string(&v)).unwrap_or_else(|| "1cm".into());
    let read = |e: &mut Engine, t: &str| match place_length(e, t, 1) {
        Ok(v) => v,
        Err(err) => {
            e.warn(format!("node distance '{}': {err}; 1cm used", t.trim()));
            1.0
        }
    };
    Ok(match s.split_once(" and ") {
        Some((a, b)) => (read(e, a), read(e, b)),
        None => {
            let m = read(e, &s);
            (m, m)
        }
    })
}

keyfn!(k_above, |e, v| placement(e, "above", v));
keyfn!(k_below, |e, v| placement(e, "below", v));
keyfn!(k_left, |e, v| placement(e, "left", v));
keyfn!(k_right, |e, v| placement(e, "right", v));
keyfn!(k_above_left, |e, v| placement(e, "above left", v));
keyfn!(k_above_right, |e, v| placement(e, "above right", v));
keyfn!(k_below_left, |e, v| placement(e, "below left", v));
keyfn!(k_below_right, |e, v| placement(e, "below right", v));

keyfn!(k_pos, |e, v| {
    e.st_node().pos = number(e, &v)?;
    Ok(())
});
keyfn!(k_midway, |e, _v| {
    e.st_node().pos = Some(0.5);
    Ok(())
});
keyfn!(k_near_start, |e, _v| {
    e.st_node().pos = Some(0.25);
    Ok(())
});
keyfn!(k_near_end, |e, _v| {
    e.st_node().pos = Some(0.75);
    Ok(())
});
keyfn!(k_very_near_start, |e, _v| {
    e.st_node().pos = Some(0.125);
    Ok(())
});
keyfn!(k_very_near_end, |e, _v| {
    e.st_node().pos = Some(0.875);
    Ok(())
});
keyfn!(k_at_start, |e, _v| {
    e.st_node().pos = Some(0.0);
    Ok(())
});
keyfn!(k_at_end, |e, _v| {
    e.st_node().pos = Some(1.0);
    Ok(())
});
keyfn!(k_sloped, |e, _v| {
    e.st_node().sloped = true;
    Ok(())
});
keyfn!(k_transform_shape, |e, _v| {
    e.st_node().transform_shape = true;
    Ok(())
});
keyfn!(k_name, |e, v| {
    let s = text(e, &v)?;
    e.st_node().name = Some(s);
    Ok(())
});
keyfn!(k_at, |e, v| {
    let s = text(e, &v)?;
    let inner = s.trim().trim_start_matches('(').trim_end_matches(')').to_string();
    let p = e.coordinate(&inner)?.p;
    e.st_node().at = Some(p);
    Ok(())
});
keyfn!(k_label, |e, v| {
    if let Some(v) = v {
        e.st_node().labels.push((v, false));
    }
    Ok(())
});
keyfn!(k_pin, |e, v| {
    if let Some(v) = v {
        e.st_node().labels.push((v, true));
    }
    Ok(())
});

keyfn!(k_out, |e, v| {
    if let Some(x) = number(e, &v)? {
        e.st.to.out = x;
    }
    e.st.to.curve = true;
    Ok(())
});
keyfn!(k_in, |e, v| {
    if let Some(x) = number(e, &v)? {
        e.st.to.inn = x;
    }
    e.st.to.curve = true;
    Ok(())
});
keyfn!(k_relative, |e, v| {
    e.st.to.relative = text(e, &v)? != "false";
    Ok(())
});
keyfn!(k_bend_angle, |e, v| {
    if let Some(x) = number(e, &v)? {
        e.st.to.bend = x;
    }
    Ok(())
});
/// `bend left[=angle]` and `bend right`: `out` the angle off the line, `in` its mirror, relative.
fn bend(e: &mut Engine, v: Option<Vec<Token>>, sign: f64) -> Result<(), Error> {
    let v = v.filter(|v| !crate::keys::trim(v).is_empty());
    if let Some(x) = number(e, &v)? {
        e.st.to.bend = x;
    }
    e.st.to.out = sign * e.st.to.bend;
    e.st.to.inn = 180.0 - e.st.to.out;
    e.st.to.curve = true;
    e.st.to.relative = true;
    Ok(())
}
keyfn!(k_bend_left, |e, v| bend(e, v, 1.0));
keyfn!(k_bend_right, |e, v| bend(e, v, -1.0));
keyfn!(k_looseness, |e, v| {
    let x = number(e, &v)?.unwrap_or(1.0);
    e.st.to.out_looseness = x;
    e.st.to.in_looseness = x;
    e.st.to.out_control = None;
    e.st.to.in_control = None;
    e.st.to.curve = true;
    Ok(())
});
keyfn!(k_out_looseness, |e, v| {
    e.st.to.out_looseness = number(e, &v)?.unwrap_or(1.0);
    e.st.to.out_control = None;
    e.st.to.curve = true;
    Ok(())
});
keyfn!(k_in_looseness, |e, v| {
    e.st.to.in_looseness = number(e, &v)?.unwrap_or(1.0);
    e.st.to.in_control = None;
    e.st.to.curve = true;
    Ok(())
});
/// `\tikz@to@set@distances{in min}{in max}{out min}{out max}`: which of them a key sets.
fn distances(e: &mut Engine, v: Option<Vec<Token>>, which: [bool; 4]) -> Result<(), Error> {
    let Some(d) = length(e, &v)? else { return Ok(()) };
    let t = &mut e.st.to;
    for (k, set) in which.iter().enumerate() {
        if *set {
            *[&mut t.in_min, &mut t.in_max, &mut t.out_min, &mut t.out_max][k] = d;
        }
    }
    if which[0] || which[1] {
        t.in_control = None;
    }
    if which[2] || which[3] {
        t.out_control = None;
    }
    t.curve = true;
    Ok(())
}
keyfn!(k_in_min_distance, |e, v| distances(e, v, [true, false, false, false]));
keyfn!(k_in_max_distance, |e, v| distances(e, v, [false, true, false, false]));
keyfn!(k_in_distance, |e, v| distances(e, v, [true, true, false, false]));
keyfn!(k_out_min_distance, |e, v| distances(e, v, [false, false, true, false]));
keyfn!(k_out_max_distance, |e, v| distances(e, v, [false, false, false, true]));
keyfn!(k_out_distance, |e, v| distances(e, v, [false, false, true, true]));
keyfn!(k_min_distance, |e, v| distances(e, v, [true, false, true, false]));
keyfn!(k_max_distance, |e, v| distances(e, v, [false, true, false, true]));
keyfn!(k_distance, |e, v| distances(e, v, [true, true, true, true]));
keyfn!(k_in_control, |e, v| {
    e.st.to.in_control = v;
    e.st.to.curve = true;
    Ok(())
});
keyfn!(k_out_control, |e, v| {
    e.st.to.out_control = v;
    e.st.to.curve = true;
    Ok(())
});
// `controls=(a) and (b)`: the out control, then the in control.
keyfn!(k_controls, |e, v| {
    let v = v.unwrap_or_default();
    let mut split = None;
    for i in 0..v.len().saturating_sub(2) {
        if v[i].is_char('a') && v[i + 1].is_char('n') && v[i + 2].is_char('d') {
            split = Some(i);
        }
    }
    let Some(i) = split else {
        e.warn("controls without 'and': ignored");
        return Ok(());
    };
    e.st.to.out_control = Some(crate::keys::trim(&v[..i]).to_vec());
    e.st.to.in_control = Some(crate::keys::trim(&v[i + 3..]).to_vec());
    e.st.to.curve = true;
    Ok(())
});
keyfn!(k_ampersand_replacement, |e, v| {
    e.st.matrix.ampersand = v.unwrap_or_default().into_iter().find_map(|t| match t {
        Token::Cs(n) => Some(n.to_string()),
        _ => None,
    });
    Ok(())
});
keyfn!(k_at_begin_cell, |e, v| {
    e.st.matrix.at_begin_cell.extend(v.unwrap_or_default());
    Ok(())
});
keyfn!(k_at_end_cell, |e, v| {
    e.st.matrix.at_end_cell.extend(v.unwrap_or_default());
    Ok(())
});
keyfn!(k_at_empty_cell, |e, v| {
    e.st.matrix.at_empty_cell = v.unwrap_or_default();
    Ok(())
});
keyfn!(k_start_chain, |e, v| {
    let s = text(e, &v)?;
    e.start_chain(&s);
    Ok(())
});
keyfn!(k_continue_chain, |e, v| {
    let s = text(e, &v)?;
    e.continue_chain(&s);
    Ok(())
});
keyfn!(k_chain_default_direction, |e, v| {
    let s = text(e, &v)?;
    e.chain_default_direction(&s);
    Ok(())
});
keyfn!(k_on_chain, |e, v| {
    let s = text(e, &v)?;
    e.on_chain(&s)
});
keyfn!(k_join, |e, v| {
    let s = text(e, &v)?;
    e.join(&s);
    Ok(())
});
keyfn!(k_after_node_path, |e, v| {
    e.st.after_node_path.extend(v.unwrap_or_default());
    Ok(())
});
keyfn!(k_alias, |e, v| {
    let s = text(e, &v)?;
    e.st.aliases.push(s);
    Ok(())
});
keyfn!(k_to_path, |e, v| {
    e.st.to.path = v;
    Ok(())
});
// `line to`, `move to` and `curve to`: the topaths library's own paths.
keyfn!(k_line_to, |e, _v| {
    e.st.to.path = None;
    e.st.to.curve = false;
    Ok(())
});
keyfn!(k_curve_to, |e, _v| {
    e.st.to.path = None;
    e.st.to.curve = true;
    Ok(())
});
keyfn!(k_loop, |e, _v| {
    e.st.to.path = None;
    e.st.to.is_loop = true;
    e.st.to.curve = true;
    e.process_keys(tikra_tokens("every loop/.try"), Some("/tikz"))
});

/// A grid step in the units of the xy system: a number counts in x and y units, a length in cm.
fn grid_step(e: &mut Engine, v: &Option<Vec<Token>>, axis: usize) -> Result<f64, Error> {
    let s = text(e, v)?;
    let (d, unit) = crate::expr::eval_length(&e.math_string(tikra_tokens(&s))?, &e.vars())?;
    Ok(if unit {
        let n = if axis == 0 { e.st.x.norm() } else { e.st.y.norm() };
        d / n.max(1e-12)
    } else {
        d
    })
}

keyfn!(k_step, |e, v| {
    let s = text(e, &v)?;
    if s.starts_with('(') {
        let p = e.coordinate(s.trim_start_matches('(').trim_end_matches(')'))?.p;
        let inv = crate::reader::invert(&e.st.transform);
        let d = inv.apply(p);
        e.st.grid_step = (d.x.abs(), d.y.abs());
    } else {
        e.st.grid_step = (grid_step(e, &v, 0)?, grid_step(e, &v, 1)?);
    }
    Ok(())
});
keyfn!(k_xstep, |e, v| {
    e.st.grid_step.0 = grid_step(e, &v, 0)?;
    Ok(())
});
keyfn!(k_ystep, |e, v| {
    e.st.grid_step.1 = grid_step(e, &v, 1)?;
    Ok(())
});

keyfn!(k_domain, |e, v| {
    let s = text(e, &v)?;
    if let Some((a, b)) = s.split_once(':') {
        e.st.domain = (e.number(a)?, e.number(b)?);
    }
    Ok(())
});
keyfn!(k_samples, |e, v| {
    e.st.samples = number(e, &v)?.map_or(25, |s| s.max(2.0) as usize).min(2000);
    Ok(())
});
keyfn!(k_variable, |e, v| {
    let s = text(e, &v)?;
    e.st.variable = s.trim().trim_start_matches('\\').to_string();
    Ok(())
});
keyfn!(k_smooth, |e, _v| {
    e.st.smooth = true;
    Ok(())
});
keyfn!(k_sharp_plot, |e, _v| {
    e.st.smooth = false;
    Ok(())
});
keyfn!(k_tension, |e, v| {
    e.st.tension = number(e, &v)?.unwrap_or(0.55);
    Ok(())
});
keyfn!(k_mark, |e, v| {
    let s = text(e, &v)?;
    e.st.plot_mark = if s == "none" || s.is_empty() { None } else { Some(s) };
    Ok(())
});
keyfn!(k_mark_size, |e, v| {
    if let Some(d) = length(e, &v)? {
        e.st.mark_size = d;
    }
    Ok(())
});
keyfn!(k_mark_repeat, |e, v| {
    e.st.mark_repeat = number(e, &v)?.unwrap_or(1.0).max(1.0) as usize;
    Ok(())
});
keyfn!(k_mark_phase, |e, v| {
    e.st.mark_phase = number(e, &v)?.unwrap_or(1.0).max(0.0) as usize;
    Ok(())
});
keyfn!(k_mark_indices, |e, v| {
    let list = e.tex.expand_fully(v.unwrap_or_default())?;
    let mut out = Vec::new();
    for item in super::foreach_items(&list) {
        if let Ok(n) = e.tex.expand_to_string(item)?.trim().parse::<f64>() {
            out.push(n as usize);
        }
    }
    e.st.mark_indices = if out.is_empty() { None } else { Some(out) };
    Ok(())
});
keyfn!(k_mark_color, |e, v| {
    let s = text(e, &v)?;
    e.st.mark_color = if s.is_empty() { None } else { Some(s) };
    Ok(())
});
keyfn!(k_only_marks, |e, _v| {
    e.st.only_marks = true;
    Ok(())
});

keyfn!(k_declare_function, |e, v| {
    let s = text(e, &v)?;
    for def in s.split(';') {
        let Some((head, body)) = def.split_once('=') else { continue };
        // a constant (`C=4;`) has no parameters
        let (name, params) = head.trim().split_once('(').unwrap_or((head.trim(), ""));
        let params = params.trim_end_matches(')').split(',').map(|p| p.trim().trim_start_matches('\\').to_string()).filter(|p| !p.is_empty()).collect();
        Rc::make_mut(&mut e.st.functions).insert(name.trim().to_string(), (params, body.trim().to_string()));
    }
    Ok(())
});

keyfn!(k_tikz_unknown, |e, v| e.tikz_unknown(v));

keyfn!(k_ignore, |_e, _v| Ok(()));

// `fit=(a)(b)(1,2)`: the node covers the points (a node by its west, east, north and south).
keyfn!(k_fit, |e, v| {
    let s = text(e, &v)?;
    let (mut lo, mut hi) = (Point::new(f64::INFINITY, f64::INFINITY), Point::new(f64::NEG_INFINITY, f64::NEG_INFINITY));
    let mut grow = |p: Point| {
        lo = Point::new(lo.x.min(p.x), lo.y.min(p.y));
        hi = Point::new(hi.x.max(p.x), hi.y.max(p.y));
    };
    let mut rest = s.as_str();
    while let Some(i) = rest.find('(') {
        let Some(j) = rest[i..].find(')') else { break };
        let inner = rest[i + 1..i + j].trim().to_string();
        rest = &rest[i + j + 1..];
        match e.nodes.get(&inner) {
            Some(n) => {
                for a in ["west", "east", "north", "south"] {
                    if let Some(p) = n.anchor(a) {
                        grow(p);
                    }
                }
            }
            None => grow(e.coordinate(&inner)?.p),
        }
    }
    if lo.x > hi.x {
        return Ok(());
    }
    let center = lo.lerp(hi, 0.5);
    e.st_node().at = Some(center);
    e.st_node().fit = Some((hi.x - lo.x, hi.y - lo.y));
    set_value(e, "/tikz/anchor", "center");
    set_value(e, "/tikz/align", "center");
    e.process_keys(tikra_tokens("every fit/.try"), None)?;
    Ok(())
});

keyfn!(k_name_path, |e, v| {
    let s = text(e, &v)?;
    e.st.name_path = Some(s);
    Ok(())
});

keyfn!(k_name_intersections, |e, v| super::intersections::name_intersections(e, v));

keyfn!(k_at_begin_scope, |e, v| {
    e.st.at_begin_scope.extend(v.unwrap_or_default());
    Ok(())
});
keyfn!(k_at_end_scope, |e, v| {
    e.st.at_end_scope.extend(v.unwrap_or_default());
    Ok(())
});
keyfn!(k_at_begin_picture, |e, v| {
    e.st.at_begin_picture.extend(v.unwrap_or_default());
    Ok(())
});
keyfn!(k_at_end_picture, |e, v| {
    e.st.at_end_picture.extend(v.unwrap_or_default());
    Ok(())
});

keyfn!(k_pic_actions, |e, _v| {
    let p = e.st.pic_mode.clone();
    let m = &mut e.st.mode;
    m.draw |= p.draw;
    m.fill |= p.fill;
    m.pattern |= p.pattern;
    m.shade |= p.shade;
    m.clip |= p.clip;
    Ok(())
});

keyfn!(k_level_distance, |e, v| {
    e.st.level_distance = length(e, &v)?.unwrap_or(1.5);
    Ok(())
});
keyfn!(k_sibling_distance, |e, v| {
    e.st.sibling_distance = length(e, &v)?.unwrap_or(1.5);
    Ok(())
});

/// `grow=down`, `grow=30`, `grow'=…`: the direction of the tree; set in a child, it is that child's.
fn set_grow(e: &mut Engine, v: &Option<Vec<Token>>, swapped: bool) -> Result<(), Error> {
    let s = text(e, v)?;
    e.st.grow_angle = match s.as_str() {
        "down" | "south" => -90.0,
        "up" | "north" => 90.0,
        "left" | "west" => 180.0,
        "right" | "east" => 0.0,
        "north east" => 45.0,
        "north west" => 135.0,
        "south east" => -45.0,
        "south west" => -135.0,
        other => e.number(other)?,
    };
    e.st.grow_swapped = swapped;
    e.st.special_level = Some(e.st.tree_level);
    Ok(())
}
keyfn!(k_grow, |e, v| set_grow(e, &v, false));
keyfn!(k_grow_swapped, |e, v| set_grow(e, &v, true));
keyfn!(k_missing, |e, v| {
    e.st.child_missing = text(e, &v)? != "false";
    Ok(())
});
keyfn!(k_growth_parent_anchor, |e, v| {
    e.st.growth_anchor = text(e, &v)?;
    Ok(())
});
keyfn!(k_parent_anchor, |e, v| {
    let s = text(e, &v)?;
    e.st.parent_anchor = if s == "border" || s.is_empty() { String::new() } else { format!(".{s}") };
    Ok(())
});
keyfn!(k_child_anchor, |e, v| {
    let s = text(e, &v)?;
    e.st.child_anchor = if s == "border" || s.is_empty() { String::new() } else { format!(".{s}") };
    Ok(())
});
keyfn!(k_edge_from_parent_path, |e, v| {
    e.st.edge_from_parent_path = v;
    Ok(())
});

keyfn!(k_column_sep, |e, v| {
    e.st.matrix.column_sep = e.tex.expand_fully(v.unwrap_or_default())?;
    Ok(())
});
keyfn!(k_row_sep, |e, v| {
    e.st.matrix.row_sep = e.tex.expand_fully(v.unwrap_or_default())?;
    Ok(())
});
keyfn!(k_matrix_of_nodes, |e, _v| {
    e.st.matrix.of_nodes = true;
    Ok(())
});
keyfn!(k_matrix_of_math_nodes, |e, _v| {
    e.st.matrix.of_nodes = true;
    e.st.matrix.math = true;
    Ok(())
});
keyfn!(k_matrix_nodes, |e, v| {
    if let Some(v) = v {
        if !e.st.matrix.nodes_style.is_empty() {
            e.st.matrix.nodes_style.push(Token::other(','));
        }
        e.st.matrix.nodes_style.extend(v);
    }
    Ok(())
});

fn shade_color(e: &mut Engine, v: &Option<Vec<Token>>) -> Result<crate::color::Color, Error> {
    let s = text(e, v)?;
    e.color(&s).ok_or_else(|| Error::Figure(format!("unknown color '{s}'")))
}
fn mix(a: crate::color::Color, b: crate::color::Color) -> crate::color::Color {
    crate::color::Color { r: (a.r + b.r) / 2.0, g: (a.g + b.g) / 2.0, b: (a.b + b.b) / 2.0 }
}
keyfn!(k_shading, |e, v| {
    e.st.shading = text(e, &v)?;
    e.st.mode.shade = true;
    Ok(())
});
keyfn!(k_shading_angle, |e, v| {
    e.st.shade_angle = number(e, &v)?.unwrap_or(0.0);
    e.st.mode.shade = true;
    Ok(())
});
/// `top color`, `bottom color` (angle 0), `left color`, `right color` (angle 90): the middle becomes their mix.
fn axis_color(e: &mut Engine, v: &Option<Vec<Token>>, top: bool, angle: f64) -> Result<(), Error> {
    let c = shade_color(e, v)?;
    if top {
        e.st.axis_top = c;
    } else {
        e.st.axis_bottom = c;
    }
    e.st.axis_middle = mix(e.st.axis_top, e.st.axis_bottom);
    e.st.shading = "axis".into();
    e.st.shade_angle = angle;
    e.st.mode.shade = true;
    Ok(())
}
keyfn!(k_top_color, |e, v| axis_color(e, &v, true, 0.0));
keyfn!(k_bottom_color, |e, v| axis_color(e, &v, false, 0.0));
keyfn!(k_left_color, |e, v| axis_color(e, &v, true, 90.0));
keyfn!(k_right_color, |e, v| axis_color(e, &v, false, 90.0));
keyfn!(k_middle_color, |e, v| {
    e.st.axis_middle = shade_color(e, &v)?;
    e.st.shading = "axis".into();
    e.st.mode.shade = true;
    Ok(())
});
keyfn!(k_ball_color, |e, v| {
    e.st.ball = shade_color(e, &v)?;
    e.st.shading = "ball".into();
    e.st.mode.shade = true;
    Ok(())
});
keyfn!(k_inner_color, |e, v| {
    e.st.radial_inner = shade_color(e, &v)?;
    e.st.shading = "radial".into();
    e.st.mode.shade = true;
    Ok(())
});
keyfn!(k_outer_color, |e, v| {
    e.st.radial_outer = shade_color(e, &v)?;
    e.st.shading = "radial".into();
    e.st.mode.shade = true;
    Ok(())
});

keyfn!(k_path_fading, |e, v| {
    let s = text(e, &v)?;
    e.st.path_fading = if s.is_empty() || s == "none" { None } else { Some(s) };
    Ok(())
});
keyfn!(k_transparency_group, |e, _v| {
    e.st.transparency_group = true;
    Ok(())
});

keyfn!(k_auto, |e, v| {
    let s = text(e, &v)?;
    e.st.auto = match s.as_str() {
        "false" => None,
        "right" => Some('r'),
        _ => Some('l'),
    };
    Ok(())
});

keyfn!(k_swap, |e, _v| {
    e.st.swap = !e.st.swap;
    Ok(())
});

keyfn!(k_baseline, |e, v| {
    e.st.baseline = Some(text(e, &v)?);
    Ok(())
});

/// Keys done natively: the path and its function.
const NATIVE: &[(&str, KeyFn)] = &[
    ("/tikz/.unknown", k_tikz_unknown),
    ("/tikz/draw", k_draw),
    ("/tikz/fill", k_fill),
    ("/tikz/color", k_color),
    ("/tikz/text", k_text),
    ("/tikz/clip", k_clip),
    ("/tikz/use as bounding box", k_bounding_box),
    ("/tikz/shade", k_shade),
    ("/tikz/pattern", k_pattern),
    ("/tikz/pattern color", k_pattern_color),
    ("/tikz/line width", k_line_width),
    ("/tikz/dash pattern", k_dash_pattern),
    ("/tikz/dash phase", k_dash_phase),
    ("/tikz/dash", k_dash),
    ("/tikz/solid", k_solid),
    ("/tikz/line cap", k_line_cap),
    ("/tikz/line join", k_line_join),
    ("/tikz/miter limit", k_miter_limit),
    ("/tikz/opacity", k_opacity),
    ("/tikz/draw opacity", k_draw_opacity),
    ("/tikz/fill opacity", k_fill_opacity),
    ("/tikz/text opacity", k_text_opacity),
    ("/tikz/even odd rule", k_even_odd),
    ("/tikz/nonzero rule", k_nonzero),
    ("/tikz/rounded corners", k_rounded),
    ("/tikz/sharp corners", k_sharp),
    ("/tikz/arrows", k_arrows),
    ("/tikz/>", k_default_arrow),
    ("/pgf/tips", k_tips),
    ("/tikz/shorten >", k_shorten_end),
    ("/tikz/shorten <", k_shorten_start),
    ("/tikz/double", k_double),
    ("/tikz/double distance", k_double_distance),
    ("/tikz/shift", k_shift),
    ("/tikz/xshift", k_xshift),
    ("/tikz/yshift", k_yshift),
    ("/tikz/scale", k_scale),
    ("/tikz/xscale", k_xscale),
    ("/tikz/yscale", k_yscale),
    ("/tikz/rotate", k_rotate),
    ("/tikz/xslant", k_xslant),
    ("/tikz/yslant", k_yslant),
    ("/tikz/rotate around", k_rotate_around),
    ("/tikz/cm", k_cm),
    ("/tikz/reset cm", k_reset_cm),
    ("/tikz/x", k_x),
    ("/tikz/y", k_y),
    ("/tikz/z", k_z),
    ("/pgf/inner sep", k_inner_sep),
    ("/pgf/outer sep", k_outer_sep),
    ("/pgf/minimum size", k_minimum_size),
    ("/tikz/font", k_font),
    ("/tikz/node font", k_node_font),
    ("/tikz/anchor", k_anchor),
    ("/tikz/above", k_above),
    ("/tikz/below", k_below),
    ("/tikz/left", k_left),
    ("/tikz/right", k_right),
    ("/tikz/above left", k_above_left),
    ("/tikz/above right", k_above_right),
    ("/tikz/below left", k_below_left),
    ("/tikz/below right", k_below_right),
    ("/tikz/pos", k_pos),
    ("/tikz/midway", k_midway),
    ("/tikz/near start", k_near_start),
    ("/tikz/near end", k_near_end),
    ("/tikz/very near start", k_very_near_start),
    ("/tikz/very near end", k_very_near_end),
    ("/tikz/at start", k_at_start),
    ("/tikz/at end", k_at_end),
    ("/tikz/sloped", k_sloped),
    ("/tikz/transform shape", k_transform_shape),
    ("/tikz/name", k_name),
    ("/tikz/at", k_at),
    ("/tikz/label", k_label),
    ("/tikz/pin", k_pin),
    ("/tikz/out", k_out),
    ("/tikz/in", k_in),
    ("/tikz/bend left", k_bend_left),
    ("/tikz/bend right", k_bend_right),
    ("/tikz/looseness", k_looseness),
    ("/tikz/out looseness", k_out_looseness),
    ("/tikz/in looseness", k_in_looseness),
    ("/tikz/bend angle", k_bend_angle),
    ("/tikz/relative", k_relative),
    ("/tikz/in min distance", k_in_min_distance),
    ("/tikz/in max distance", k_in_max_distance),
    ("/tikz/in distance", k_in_distance),
    ("/tikz/out min distance", k_out_min_distance),
    ("/tikz/out max distance", k_out_max_distance),
    ("/tikz/out distance", k_out_distance),
    ("/tikz/min distance", k_min_distance),
    ("/tikz/max distance", k_max_distance),
    ("/tikz/distance", k_distance),
    ("/tikz/in control", k_in_control),
    ("/tikz/out control", k_out_control),
    ("/tikz/controls", k_controls),
    ("/tikz/to path", k_to_path),
    ("/tikz/start chain", k_start_chain),
    ("/tikz/continue chain", k_continue_chain),
    ("/tikz/chain default direction", k_chain_default_direction),
    ("/tikz/on chain", k_on_chain),
    ("/tikz/join", k_join),
    ("/tikz/after node path", k_after_node_path),
    ("/tikz/alias", k_alias),
    ("/tikz/execute at begin cell", k_at_begin_cell),
    ("/tikz/ampersand replacement", k_ampersand_replacement),
    ("/tikz/execute at end cell", k_at_end_cell),
    ("/tikz/execute at empty cell", k_at_empty_cell),
    ("/tikz/line to", k_line_to),
    ("/tikz/curve to", k_curve_to),
    ("/tikz/loop", k_loop),
    ("/tikz/step", k_step),
    ("/tikz/xstep", k_xstep),
    ("/tikz/ystep", k_ystep),
    ("/tikz/domain", k_domain),
    ("/tikz/samples", k_samples),
    ("/tikz/variable", k_variable),
    ("/tikz/smooth", k_smooth),
    ("/tikz/sharp plot", k_sharp_plot),
    ("/tikz/tension", k_tension),
    ("/tikz/mark", k_mark),
    ("/tikz/only marks", k_only_marks),
    ("/tikz/mark size", k_mark_size),
    ("/tikz/mark repeat", k_mark_repeat),
    ("/tikz/mark phase", k_mark_phase),
    ("/tikz/mark indices", k_mark_indices),
    ("/pgf/mark color", k_mark_color),
    ("/pgf/declare function", k_declare_function),
    ("/tikz/declare function", k_declare_function),
    ("/tikz/overlay", k_ignore),
    ("/tikz/on grid", k_ignore),
    ("/tikz/inner frame sep", k_ignore),
    ("/tikz/baseline", k_baseline),
    ("/tikz/auto", k_auto),
    ("/tikz/path fading", k_path_fading),
    ("/tikz/transparency group", k_transparency_group),
    ("/tikz/shading", k_shading),
    ("/tikz/shading angle", k_shading_angle),
    ("/tikz/top color", k_top_color),
    ("/tikz/bottom color", k_bottom_color),
    ("/tikz/left color", k_left_color),
    ("/tikz/right color", k_right_color),
    ("/tikz/middle color", k_middle_color),
    ("/tikz/ball color", k_ball_color),
    ("/tikz/inner color", k_inner_color),
    ("/tikz/outer color", k_outer_color),
    ("/tikz/column sep", k_column_sep),
    ("/tikz/row sep", k_row_sep),
    ("/tikz/matrix of nodes", k_matrix_of_nodes),
    ("/tikz/matrix of math nodes", k_matrix_of_math_nodes),
    ("/tikz/nodes", k_matrix_nodes),
    ("/tikz/matrix", k_ignore),
    ("/tikz/level distance", k_level_distance),
    ("/tikz/sibling distance", k_sibling_distance),
    ("/tikz/grow", k_grow),
    ("/tikz/grow'", k_grow_swapped),
    ("/tikz/missing", k_missing),
    ("/tikz/growth parent anchor", k_growth_parent_anchor),
    ("/tikz/parent anchor", k_parent_anchor),
    ("/tikz/child anchor", k_child_anchor),
    ("/tikz/edge from parent path", k_edge_from_parent_path),
    ("/tikz/pic actions", k_pic_actions),
    ("/tikz/execute at begin scope", k_at_begin_scope),
    ("/tikz/execute at end scope", k_at_end_scope),
    ("/tikz/execute at begin picture", k_at_begin_picture),
    ("/tikz/execute at end picture", k_at_end_picture),
    ("/tikz/fit", k_fit),
    ("/tikz/name path", k_name_path),
    ("/tikz/name path global", k_name_path),
    ("/tikz/name intersections", k_name_intersections),
    ("/tikz/swap", k_swap),
    ("/tikz/'", k_swap),
];

/// TikZ's own definitions of the keys it defines as styles or values (tikz.code.tex, pgfmoduleshapes).
const DEFINITIONS: &str = r"
\tikzset{
  every picture/.style={}, every scope/.style={}, every path/.style={}, every node/.style={},
  every label/.style={}, every pin/.style={}, every pin edge/.style={help lines},
  every edge/.style={draw}, every to/.style={}, every mark/.style={}, mark options/.style={every mark/.style={#1}},
  no marks/.style={mark=none}, no markers/.style={mark=none}, every curve to/.style={},
  every loop/.style={->,shorten >=1pt}, loop right/.style={right,out=15,in=-15,loop},
  loop above/.style={above,out=105,in=75,loop}, loop left/.style={left,out=195,in=165,loop},
  loop below/.style={below,out=285,in=255,loop}, move to/.style={to path={(\tikztotarget) \tikztonodes}}, every circle node/.style={}, every rectangle node/.style={},
  every coordinate node/.style={}, every ellipse node/.style={}, every diamond node/.style={},
  ultra thin/.style={line width=0.1pt}, very thin/.style={line width=0.2pt}, thin/.style={line width=0.4pt},
  semithick/.style={line width=0.6pt}, thick/.style={line width=0.8pt}, very thick/.style={line width=1.2pt},
  ultra thick/.style={line width=1.6pt},
  help lines/.style={color=gray,line width=0.2pt},
  dotted/.style={dash pattern=on \pgflinewidth off 2pt},
  densely dotted/.style={dash pattern=on \pgflinewidth off 1pt},
  loosely dotted/.style={dash pattern=on \pgflinewidth off 4pt},
  dashed/.style={dash pattern=on 3pt off 3pt},
  densely dashed/.style={dash pattern=on 3pt off 2pt},
  loosely dashed/.style={dash pattern=on 3pt off 6pt},
  dash dot/.style={dash pattern=on 3pt off 2pt on \pgflinewidth off 2pt},
  densely dash dot/.style={dash pattern=on 3pt off 1pt on \pgflinewidth off 1pt},
  loosely dash dot/.style={dash pattern=on 3pt off 4pt on \pgflinewidth off 4pt},
  dash dot dot/.style={dash pattern=on 3pt off 2pt on \pgflinewidth off 2pt on \pgflinewidth off 2pt},
  densely dash dot dot/.style={dash pattern=on 3pt off 1pt on \pgflinewidth off 1pt on \pgflinewidth off 1pt},
  loosely dash dot dot/.style={dash pattern=on 3pt off 4pt on \pgflinewidth off 4pt on \pgflinewidth off 4pt},
  transparent/.style={opacity=0}, ultra nearly transparent/.style={opacity=0.05},
  very nearly transparent/.style={opacity=0.1}, nearly transparent/.style={opacity=0.25},
  semitransparent/.style={opacity=0.5}, nearly opaque/.style={opacity=0.75},
  very nearly opaque/.style={opacity=0.9}, ultra nearly opaque/.style={opacity=0.95}, opaque/.style={opacity=1},
  rounded corners/.default=4pt, double/.default=white, 
  draw opacity/.default=1, help lines/.append style={},
  text centered/.style={align=center}, text ragged/.style={align=left}, text badly ragged/.style={align=left},
  text justified/.style={align=justify}, text ragged left/.style={align=right},
  shape/.initial=rectangle, anchor/.initial=center, node distance/.initial=1cm and 1cm,
  align/.initial=, text width/.initial=, text height/.initial=, text depth/.initial=,
  label distance/.initial=0pt, pin distance/.initial=3ex, label position/.initial=above,
  /pgf/inner xsep/.initial=.3333em, /pgf/inner ysep/.initial=.3333em,
  /pgf/outer xsep/.initial=.5\pgflinewidth, /pgf/outer ysep/.initial=.5\pgflinewidth,
  /pgf/minimum width/.initial=1pt, /pgf/minimum height/.initial=1pt,
  /pgf/aspect/.initial=1, /pgf/shape aspect/.forward to=/pgf/aspect,
  shape aspect/.forward to=/pgf/aspect,
  inner xsep/.forward to=/pgf/inner xsep, inner ysep/.forward to=/pgf/inner ysep,
  inner sep/.forward to=/pgf/inner sep, outer sep/.forward to=/pgf/outer sep,
  outer xsep/.forward to=/pgf/outer xsep, outer ysep/.forward to=/pgf/outer ysep,
  minimum width/.forward to=/pgf/minimum width, minimum height/.forward to=/pgf/minimum height,
  minimum size/.forward to=/pgf/minimum size, aspect/.forward to=/pgf/aspect,
  coordinate/.style={shape=coordinate},
  circle/.style={shape=circle}, rectangle/.style={shape=rectangle}, ellipse/.style={shape=ellipse},
  diamond/.style={shape=diamond},
  behind path/.style={}, in front of path/.style={},
  execute at begin node/.code={}, execute at end node/.code={},
  style/.code={\pgfkeysalso{#1}},
  every shadow/.style={}, alias/.code={},
  /tikz/radius/.initial=, /tikz/x radius/.initial=, /tikz/y radius/.initial=,
  /tikz/start angle/.initial=, /tikz/end angle/.initial=, /tikz/delta angle/.initial=,
  /tikz/relative/.code={},
  /pgfplots/table/search path/.code={}, /pgfplots/compat/.code={},
  /tikz/pics/setup code/.initial=, /tikz/pics/code/.initial=, /tikz/pics/background code/.initial=,
  /tikz/pics/foreground code/.initial=, angle radius/.initial=5mm, angle eccentricity/.initial=.6,
  every pic/.style={}, every matrix/.style={}, every cell/.style={}, every child/.style={}, every child node/.style={}, edge from parent/.style={draw}, name prefix/.code={\tikra@nameprefix{#1}},
  every edge quotes/.style={auto}, every label quotes/.style={}, every node quotes/.style={}, every fit/.style={},
}
\def\tikzoption#1#2{\tikzset{#1/.code={#2}}}
\def\pgfsetmatrixcolumnsep#1{\tikzset{column sep={#1}}}
\def\pgfsetmatrixrowsep#1{\tikzset{row sep={#1}}}
\def\pgfmatrixnextcell{\tikra@nextcell}
\def\pgfmatrixendrow{\tikra@endrow}
\tikzset{
  on background layer/.style={execute at begin scope={\pgfonlayer{background}\tikzset{every on background layer/.try}},
    execute at end scope={\endpgfonlayer}},
  background rectangle/.style={draw}, background top/.style={draw}, background bottom/.style={draw},
  background left/.style={draw}, background right/.style={draw}, background grid/.style={help lines,draw},
  inner frame sep/.code={\def\tikz@framexsep{#1}\def\tikz@frameysep{#1}},
  inner frame xsep/.code={\def\tikz@framexsep{#1}}, inner frame ysep/.code={\def\tikz@frameysep{#1}},
  outer frame sep/.code={\def\tikz@outerframexsep{#1}\def\tikz@outerframeysep{#1}},
  outer frame xsep/.code={\def\tikz@outerframexsep{#1}}, outer frame ysep/.code={\def\tikz@outerframeysep{#1}},
  loose background/.style={inner frame sep=2ex}, tight background/.style={inner frame sep=0ex},
  framed/.style={show background rectangle}, gridded/.style={show background grid},
  show background rectangle/.style={execute at end picture={\tikra@frame\tikra@onbg{\path[style=background rectangle] (\tikra@xa,\tikra@ya) rectangle (\tikra@xb,\tikra@yb);}}},
  show background top/.style={execute at end picture={\tikra@frame\tikra@onbg{\path[style=background top] (\tikra@oxa,\tikra@yb) -- (\tikra@oxb,\tikra@yb);}}},
  show background bottom/.style={execute at end picture={\tikra@frame\tikra@onbg{\path[style=background bottom] (\tikra@oxa,\tikra@ya) -- (\tikra@oxb,\tikra@ya);}}},
  show background left/.style={execute at end picture={\tikra@frame\tikra@onbg{\path[style=background left] (\tikra@xa,\tikra@oya) -- (\tikra@xa,\tikra@oyb);}}},
  show background right/.style={execute at end picture={\tikra@frame\tikra@onbg{\path[style=background right] (\tikra@xb,\tikra@oya) -- (\tikra@xb,\tikra@oyb);}}},
  show background grid/.style={execute at end picture={\tikra@frame\tikra@onbg{\path[style=background grid] (\tikra@xa,\tikra@ya) grid (\tikra@xb,\tikra@yb);}}},
}
\def\tikz@framexsep{1ex}\def\tikz@frameysep{1ex}\def\tikz@outerframexsep{0ex}\def\tikz@outerframeysep{0ex}
\def\tikra@onbg#1{\pgfonlayer{background}\pgftransformreset#1\endpgfonlayer}
\def\tikz@picture@start{\pgfkeysalso{/tikz/every picture/.try}}
\def\tikz@scope@options{\pgfkeysalso{/tikz/every scope/.try}}
";

/// Defines every native key and TikZ's own styles.
pub fn define_keys(e: &mut Engine) {
    e.tex.define("pgflinewidth", Meaning::Dimen(4000));
    e.tex.dimens.insert(4000, (0.4 * 65536.0) as i64);
    super::decorations::define(e);
    for (path, f) in NATIVE.iter().chain(super::decorations::KEYS) {
        let id = NATIVE_KEY_BASE + e.key_fns.len() as u16;
        e.key_fns.push(*f);
        e.tex.define(&macro_name(&format!("{path}/.@cmd")), Meaning::Native(id));
    }
    e.st.key_path = "/tikz".into();
    for (path, value) in super::shapes::INITIAL {
        e.tex.set_meaning(&Token::cs(&macro_name(path)), macro_from(value), false);
    }
    // the definitions run as TeX, with @ a letter
    let mut lexer = crate::tex::Tex::new_lexer();
    lexer.push_text(DEFINITIONS);
    lexer.push_text(super::decorations::DEFINITIONS);
    lexer.push_text(super::decorations::DECLARATIONS);
    lexer.push_text(super::decorations::textfx::DEFINITIONS);
    lexer.push_text(super::shapes::DEFINITIONS);
    let mut tokens = Vec::new();
    while let Some(t) = lexer.get_token() {
        tokens.push(t);
    }
    if let Err(err) = e.run_tokens(tokens) {
        e.warn(format!("internal: TikZ definitions failed: {err}"));
    }
    e.tex.warnings.clear();
    e.st.key_path = "/".into();
}
