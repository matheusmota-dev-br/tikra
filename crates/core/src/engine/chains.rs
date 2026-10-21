//! The chains library: `start chain=name going right`, then nodes `on chain` are placed one after
//! the other (`right=of` the previous one), named `name-1`, `name-2`… (with `name-begin` and
//! `name-end`), and `join=by ->` draws an edge from the previous node to each. `\chainin (a)` puts a
//! node that already exists on the chain.

use crate::tex::Token;
use crate::Error;

use super::{tikra_tokens, Engine};

/// How a chain places its next node: `going right` (after a previous one only), `placed {at=…}`.
#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Going(String),
    Placed(String),
}

/// A chain: how many nodes it has, and where the next goes.
#[derive(Debug, Clone)]
pub struct Chain {
    pub count: usize,
    pub direction: Direction,
}

/// What a node `on chain` does once its options are read: its chain and its own direction, and the
/// `join`s (`with` node, edge options).
#[derive(Debug, Clone, Default)]
pub struct OnChain {
    pub chain: String,
    pub direction: Option<Direction>,
    pub joins: Vec<(Option<String>, String)>,
}

/// `name going below`, `going right`, `placed {at=(0,\tikzchaincount)}`, `name`.
fn parse(spec: &str) -> (String, Option<Direction>) {
    let s = spec.trim();
    let split = |key: &str| -> Option<(String, String)> {
        let i = s.find(key).filter(|&i| i == 0 || s[..i].ends_with(' '))?;
        Some((s[..i].trim().to_string(), s[i + key.len()..].trim().to_string()))
    };
    if let Some((name, d)) = split("going ") {
        return (name, Some(Direction::Going(d)));
    }
    if let Some((name, d)) = split("placed ") {
        let d = d.trim_start_matches('{').trim_end_matches('}').to_string();
        return (name, Some(Direction::Placed(d)));
    }
    (s.to_string(), None)
}

impl Engine {
    /// `start chain=spec`: a new chain, the current one.
    pub fn start_chain(&mut self, spec: &str) {
        let (name, direction) = parse(spec);
        let name = if name.is_empty() { "chain".to_string() } else { name };
        let direction = direction.unwrap_or_else(|| self.st.chain_direction.clone());
        self.chain_list.insert(name.clone(), Chain { count: 0, direction });
        self.st.current_chain = name;
    }

    /// `continue chain=spec`: an existing chain made current again, its direction maybe changed.
    pub fn continue_chain(&mut self, spec: &str) {
        let (name, direction) = parse(spec);
        let name = if name.is_empty() { self.st.current_chain.clone() } else { name };
        match self.chain_list.get_mut(&name) {
            Some(c) => {
                if let Some(d) = direction {
                    c.direction = d;
                }
                self.st.current_chain = name;
            }
            None => self.warn(format!("continue chain: unknown chain '{name}'")),
        }
    }

    /// `chain default direction=going below`.
    pub fn chain_default_direction(&mut self, spec: &str) {
        if let (_, Some(d)) = parse(spec) {
            self.st.chain_direction = d;
        }
    }

    /// `on chain=spec`: the node will be the chain's next; it is placed now (and again once all its
    /// options are read, as `\tikz@lib@chain@last@found` does).
    pub fn on_chain(&mut self, spec: &str) -> Result<(), Error> {
        let (name, direction) = parse(spec);
        let name = if name.is_empty() { self.st.current_chain.clone() } else { name };
        let Some(chain) = self.chain_list.get(&name).cloned() else {
            self.warn(format!("on chain: unknown chain '{name}'"));
            return Ok(());
        };
        let previous = if chain.count == 0 { String::new() } else { format!("{name}-{}", chain.count) };
        self.tex.define_string("tikzchainprevious", &previous, true);
        self.tex.define_string("tikzchaincount", &(chain.count + 1).to_string(), true);
        self.place_on_chain(direction.as_ref().unwrap_or(&chain.direction), &previous)?;
        self.st.on_chain = Some(OnChain { chain: name, direction, joins: Vec::new() });
        Ok(())
    }

    fn place_on_chain(&mut self, direction: &Direction, previous: &str) -> Result<(), Error> {
        let (d, always) = match direction {
            Direction::Going(d) => (d, false),
            Direction::Placed(d) => (d, true),
        };
        if !always && previous.is_empty() {
            return Ok(());
        }
        let keys = if d.contains('=') { d.clone() } else { format!("{d}=of {previous}") };
        self.process_keys(tikra_tokens(&keys), Some("/tikz"))
    }

    /// `join`, `join=by ->`, `join=with a by red`: an edge to this node, drawn after it.
    pub fn join(&mut self, spec: &str) {
        let s = spec.trim();
        let (with, by) = match s.strip_prefix("with ") {
            Some(rest) => match rest.split_once(" by ") {
                Some((w, b)) => (Some(w.trim().to_string()), b.trim().to_string()),
                None => (Some(rest.trim().to_string()), String::new()),
            },
            None => (None, s.strip_prefix("by").unwrap_or(s).trim().to_string()),
        };
        match self.st.on_chain.as_mut() {
            Some(c) => c.joins.push((with, by)),
            None => self.st.pending_joins.push((with, by)),
        }
    }

    /// The node's options are read: it becomes the chain's next node. Returns its names on the
    /// chain and the edges of its joins (a path to run after it).
    pub fn chain_node_found(&mut self) -> Result<(Vec<String>, Vec<Token>), Error> {
        let Some(mut on) = self.st.on_chain.take() else { return Ok((Vec::new(), Vec::new())) };
        on.joins.splice(0..0, std::mem::take(&mut self.st.pending_joins));
        let Some(chain) = self.chain_list.get_mut(&on.chain) else { return Ok((Vec::new(), Vec::new())) };
        let previous = if chain.count == 0 { String::new() } else { format!("{}-{}", on.chain, chain.count) };
        chain.count += 1;
        let k = chain.count;
        let direction = on.direction.clone().unwrap_or_else(|| chain.direction.clone());
        let current = format!("{}-{k}", on.chain);
        self.tex.define_string("tikzchaincurrent", &current, true);
        let mut names = vec![current.clone(), format!("{}-end", on.chain)];
        if k == 1 {
            names.push(format!("{}-begin", on.chain));
        }
        self.place_on_chain(&direction, &previous)?;
        // every on chain can join too
        self.st.on_chain = Some(OnChain { joins: Vec::new(), ..on.clone() });
        self.process_keys(tikra_tokens("every on chain/.try"), Some("/tikz"))?;
        if let Some(more) = self.st.on_chain.take() {
            on.joins.extend(more.joins);
        }
        let mut path = Vec::new();
        for (with, by) in on.joins {
            let from = match with {
                Some(w) => w,
                None if previous.is_empty() => continue,
                None => previous.clone(),
            };
            path.extend(tikra_tokens(&format!("({from}) edge[every join/.try,")));
            path.extend(tikra_tokens(&by));
            path.extend(tikra_tokens(&format!("] ({current}) ")));
        }
        Ok((names, path))
    }

    /// `\chainin (a) [options];`: the existing node `a` on the chain, its joins drawn.
    pub fn chainin(&mut self) -> Result<(), Error> {
        let name = match self.tex.next_nonspace() {
            Some(t) if t.is_char('(') => {
                let n = super::coords::read_coordinate_tokens(self)?;
                self.tex.expand_to_string(n)?.trim().to_string()
            }
            _ => return Err(Error::Figure("\\chainin without (node)".into())),
        };
        let options = match self.tex.next_nonspace() {
            Some(t) if t.is_char('[') => self.tex.read_until_char(']', false)?,
            Some(t) => {
                self.tex.back(t);
                Vec::new()
            }
            None => Vec::new(),
        };
        let Some(info) = self.nodes.get(&name).cloned() else {
            return Err(Error::Figure(format!("\\chainin: unknown node ({name})")));
        };
        self.begin_group();
        let r = (|| {
            let mut v = tikra_tokens("on chain,every chain in/.try,");
            v.extend(options);
            self.process_keys(v, Some("/tikz"))?;
            self.chain_node_found()
        })();
        self.end_group();
        let (names, path) = r?;
        for n in names {
            self.nodes.insert(n, info.clone());
        }
        // the rest of the statement, as the \path it is in TikZ
        self.tex.push_tokens(path);
        super::path::statement(self, "")
    }
}
