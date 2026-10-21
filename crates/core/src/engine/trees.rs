//! Trees: `node {root} child {node {a}} child[options] {node {b} child {…}}`, as `tikz.code.tex`
//! grows them. The children of a node are collected; for each one the coordinate system moves to
//! the parent's center, then `level distance` along `grow` and its place among the siblings
//! (`sibling distance`); the child's node is named `parent-k`, its own children follow, and the
//! `edge from parent` is drawn last, with its label nodes midway.

use crate::scene::{Point, Transform};
use crate::tex::{self, Cat, Token};
use crate::Error;

use super::{macro_from, tikra_tokens, Engine};

/// One child: what sets it up (`child foreach`'s variable), its options and its body.
struct Child {
    setup: Vec<Token>,
    options: Vec<Token>,
    body: Vec<Token>,
}

impl Engine {
    /// `child …` after a node on a path: the word `child` already read.
    pub fn children(&mut self, parent: Option<String>) -> Result<(), Error> {
        let Some(info) = self.last_node.clone() else {
            return Err(Error::Figure("child without a node before it".into()));
        };
        // the parent needs a name for the edges: an unnamed one gets one, as TikZ does
        let parent = match parent {
            Some(p) => p,
            None => {
                self.tree_count += 1;
                let n = format!("tikra@tree@{}", self.tree_count);
                self.nodes.insert(n.clone(), info);
                n
            }
        };
        let children = self.collect_children()?;
        self.begin_group();
        let r = self.grow_children(&parent, children);
        self.end_group();
        r
    }

    /// Reads `child[options]{body}` (and `child foreach \x in {…} {body}`) while they follow.
    fn collect_children(&mut self) -> Result<Vec<Child>, Error> {
        let mut out = Vec::new();
        let mut first = true;
        loop {
            if !first {
                // another `child`?
                let mut read = Vec::new();
                let mut word = String::new();
                while let Some(t) = self.tex.get_token() {
                    match &t {
                        Token::Char(c, Cat::Letter) => {
                            word.push(*c);
                            read.push(t);
                        }
                        x if x.is_space() && word.is_empty() => read.push(t),
                        _ => {
                            self.tex.back(t);
                            break;
                        }
                    }
                }
                if word != "child" {
                    self.tex.push_tokens(read);
                    return Ok(out);
                }
            }
            first = false;
            let options = match self.tex.next_nonspace() {
                Some(t) if t.is_char('[') => self.tex.read_until_char(']', false)?,
                Some(t) => {
                    self.tex.back(t);
                    Vec::new()
                }
                None => Vec::new(),
            };
            // child foreach \x in {list} {body}
            let mut peek = Vec::new();
            let mut word = String::new();
            while let Some(t) = self.tex.get_token() {
                match &t {
                    Token::Char(c, Cat::Letter) => {
                        word.push(*c);
                        peek.push(t);
                    }
                    x if x.is_space() && word.is_empty() => peek.push(t),
                    _ => {
                        self.tex.back(t);
                        break;
                    }
                }
            }
            if word == "foreach" {
                let var = self.tex.next_nonspace().ok_or_else(|| Error::Figure("child foreach without a variable".into()))?;
                // up to `in`
                loop {
                    match self.tex.next_nonspace() {
                        Some(t) if t.is_char('i') => {
                            if let Some(n) = self.tex.get_token() {
                                if n.is_char('n') {
                                    break;
                                }
                            }
                        }
                        Some(_) => {}
                        None => return Err(Error::Figure("child foreach without 'in'".into())),
                    }
                }
                let list = self.tex.read_undelimited()?;
                let list = self.tex.expand_fully(list)?;
                let body = match self.tex.next_nonspace() {
                    Some(Token::Char(_, Cat::Begin)) => self.tex.read_group_body()?,
                    Some(t) => {
                        self.tex.back(t);
                        Vec::new()
                    }
                    None => Vec::new(),
                };
                for item in super::foreach_items(&list) {
                    // the variable is fixed by a \def at the start of the child
                    let mut setup = vec![Token::cs("def"), var.clone(), Token::Char('{', Cat::Begin)];
                    setup.extend(item);
                    setup.push(Token::Char('}', Cat::End));
                    out.push(Child { setup, options: options.clone(), body: body.clone() });
                }
                continue;
            }
            self.tex.push_tokens(peek);
            let body = match self.tex.next_nonspace() {
                Some(Token::Char(_, Cat::Begin)) => self.tex.read_group_body()?,
                Some(t) => {
                    self.tex.back(t);
                    Vec::new()
                }
                None => Vec::new(),
            };
            out.push(Child { setup: Vec::new(), options, body });
        }
    }

    fn grow_children(&mut self, parent: &str, children: Vec<Child>) -> Result<(), Error> {
        self.st.tree_level += 1;
        let level = self.st.tree_level;
        self.st.pending = Transform::default();
        self.process_keys(tikra_tokens(&format!("level/.try={level},level {level}/.try")), Some("/tikz"))?;
        self.st.apply_pending();
        let n = children.len();
        let Some(info) = self.nodes.get(parent).cloned() else { return Ok(()) };
        let anchor = self.st.growth_anchor.clone();
        let origin = info.anchor(&anchor).unwrap_or(info.center());
        let local = crate::reader::invert(&self.st.transform).apply(origin);
        self.st.transform = self.st.transform.compose(&Transform::translation(local.x, local.y));
        for (k, child) in children.into_iter().enumerate() {
            self.begin_group();
            let r = self.grow_child(parent, k + 1, n, child);
            self.end_group();
            r?;
        }
        Ok(())
    }

    fn grow_child(&mut self, parent: &str, k: usize, n: usize, child: Child) -> Result<(), Error> {
        self.tex.set_meaning(&Token::cs("tikzparentnode"), macro_from(parent), false);
        self.st.pending = Transform::default();
        self.st.child_missing = false;
        self.run_tokens(child.setup)?;
        let mut options = tikra_tokens("every child/.try,");
        options.extend(child.options);
        self.process_keys(options, Some("/tikz"))?;
        self.st.apply_pending();
        if self.st.child_missing {
            return Ok(());
        }
        // grow: level distance along the direction, then the place among the siblings
        let polar = |angle: f64, d: f64| {
            let (s, c) = angle.to_radians().sin_cos();
            Point::new(d * c, d * s)
        };
        let (g, ld, sd) = (self.st.grow_angle, self.st.level_distance, self.st.sibling_distance);
        let mut t = self.st.transform.compose(&Transform::translation(polar(g, ld).x, polar(g, ld).y));
        if let Some(f) = self.st.growth_function.clone() {
            // a growth function of the options: it moves the coordinate system itself
            for (name, v) in [("tikznumberofchildren", n), ("tikznumberofcurrentchild", k)] {
                if let tex::Meaning::Count(r) = self.tex.meaning_of(name) {
                    self.tex.set_count(r, v as i64, false);
                }
            }
            self.run_tokens(f)?;
            t = self.st.transform;
        } else if self.st.special_level != Some(self.st.tree_level) {
            let (left, right) = if self.st.grow_swapped { (g + 90.0, g - 90.0) } else { (g - 90.0, g + 90.0) };
            let a = polar(left, (n as f64 + 1.0) * sd / 2.0);
            let b = polar(right, k as f64 * sd);
            t = t.compose(&Transform::translation(a.x, a.y)).compose(&Transform::translation(b.x, b.y));
        }
        self.st.transform = t;
        // the child's node, named parent-k unless it names itself, then its own children
        let body = keys_trimmed(&child.body);
        let (kind, rest) = split_first_word(&body);
        let mut statement = tikra_tokens(&format!("\\path {} [name={parent}-{k},every child node/.try]", if kind == "node" { "node" } else { "coordinate" }));
        if kind == "node" || kind == "coordinate" {
            statement.extend(rest);
        } else {
            statement.extend(body);
        }
        statement.extend(tikra_tokens(";"));
        self.tree_edge_out = None;
        self.tree_child_names.push(None);
        let r = self.run_tokens(statement);
        let named = self.tree_child_names.pop().flatten().filter(|n| !n.is_empty());
        r?;
        let child_name = named.unwrap_or_else(|| format!("{parent}-{k}"));
        self.tex.set_meaning(&Token::cs("tikzchildnode"), macro_from(&child_name), false);
        // the edge from the parent, in the parent's coordinate system
        let (edge_options, edge_nodes) = self.tree_edge_out.take().unwrap_or_default();
        let pa = self.st.parent_anchor.clone();
        let ca = self.st.child_anchor.clone();
        self.tex.define_string("tikzparentanchor", &pa, false);
        self.tex.define_string("tikzchildanchor", &ca, false);
        let mut edge = tikra_tokens("\\path[edge from parent/.try,");
        edge.extend(edge_options);
        edge.extend(tikra_tokens("] "));
        match self.st.edge_from_parent_path.clone() {
            // a path of its own (edge from parent path={…}, the trees library's forks); labels after it
            Some(p) => {
                edge.extend(p);
                edge.extend(edge_nodes);
            }
            None => {
                edge.extend(tikra_tokens(&format!("({parent}{pa}) -- ")));
                edge.extend(edge_nodes);
                edge.extend(tikra_tokens(&format!(" ({child_name}{ca})")));
            }
        }
        edge.extend(tikra_tokens(";"));
        self.st.transform = Transform::default();
        self.run_tokens(edge)
    }
}

fn keys_trimmed(v: &[Token]) -> Vec<Token> {
    crate::keys::trim(v).to_vec()
}

/// `node[…] {…} child …` → (`node`, the rest).
fn split_first_word(v: &[Token]) -> (String, Vec<Token>) {
    let mut word = String::new();
    let mut i = 0;
    while i < v.len() {
        match &v[i] {
            Token::Char(c, Cat::Letter) => word.push(*c),
            _ => break,
        }
        i += 1;
    }
    (word, v[i..].to_vec())
}

#[allow(dead_code)]
fn show(v: &[Token]) -> String {
    tex::tokens_to_string(v)
}
