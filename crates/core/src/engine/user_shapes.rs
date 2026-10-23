//! Shapes declared in the document: `\pgfdeclareshape{name}{…}` with the saved anchors, the
//! anchors and the border of a shape it inherits from (`\inheritsavedanchors[from=rectangle]`),
//! and paths of its own, `\backgroundpath{code}` and `\foregroundpath{code}`: basic-layer code that may read the
//! saved anchors (`\southwest`, `\northeast`, `\centerpoint` put a point in `\pgf@x`, `\pgf@y`).

use crate::scene::{Point, Segment, Transform};
use crate::tex::{self, Token};
use crate::{Error, PT};

use super::shapes::{Geometry, Layer, ShapePath, Usage};
use super::Engine;

/// A declared shape: the shape whose geometry it takes, and its paths' code by layer.
#[derive(Debug, Clone, Default)]
pub struct UserShape {
    pub base: String,
    pub paths: Vec<(Layer, Vec<Token>)>,
}

impl Engine {
    /// `\pgfdeclareshape{name}{body}`.
    pub fn declare_shape(&mut self) -> Result<(), Error> {
        let name = self.argument_string()?;
        let body = self.argument()?;
        let mut shape = UserShape { base: "rectangle".into(), paths: Vec::new() };
        let mut rest: std::collections::VecDeque<Token> = body.into_iter().collect();
        // the argument at the front, a group's content or one token
        let take_group = |rest: &mut std::collections::VecDeque<Token>| -> Vec<Token> {
            while rest.front().is_some_and(|t| t.is_space()) {
                rest.pop_front();
            }
            match rest.pop_front() {
                Some(Token::Char(_, tex::Cat::Begin)) => {
                    let mut depth = 1;
                    let mut out = Vec::new();
                    while let Some(t) = rest.pop_front() {
                        match t {
                            Token::Char(_, tex::Cat::Begin) => depth += 1,
                            Token::Char(_, tex::Cat::End) => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                        out.push(t);
                    }
                    out
                }
                Some(t) => vec![t],
                None => Vec::new(),
            }
        };
        // `[from=name]`
        let take_option = |rest: &mut std::collections::VecDeque<Token>| -> String {
            while rest.front().is_some_and(|t| t.is_space()) {
                rest.pop_front();
            }
            if !rest.front().is_some_and(|t| t.is_char('[')) {
                return String::new();
            }
            rest.pop_front();
            let mut out = Vec::new();
            while let Some(t) = rest.pop_front() {
                if t.is_char(']') {
                    break;
                }
                out.push(t);
            }
            let s = tex::tokens_to_string(&out);
            s.split_once('=').map_or(s.clone(), |(_, v)| v.to_string()).trim().to_string()
        };
        while let Some(t) = rest.pop_front() {
            let Token::Cs(cs) = &t else { continue };
            match &**cs {
                "inheritsavedanchors" => shape.base = take_option(&mut rest),
                "inheritanchor" | "inheritanchorborder" | "inheritbackgroundpath" | "inheritforegroundpath" | "inheritbeforebackgroundpath" | "inheritbehindforegroundpath" => {
                    let from = take_option(&mut rest);
                    if &**cs == "inheritanchor" {
                        take_group(&mut rest);
                    }
                    let _ = from;
                }
                // the paths the node uses as it says (drawn, filled)
                "backgroundpath" | "foregroundpath" => {
                    let layer = if &**cs == "backgroundpath" { Layer::Background } else { Layer::Foreground };
                    let code = take_group(&mut rest);
                    shape.paths.push((layer, code));
                }
                "anchor"
                | "savedanchor"
                | "saveddimen"
                | "savedmacro"
                | "anchorborder"
                | "nodeparts"
                | "beforebackgroundpath"
                | "behindforegroundpath"
                | "behindbackgroundpath"
                | "beforeforegroundpath" => {
                    self.warn(format!("\\pgfdeclareshape{{{}}}: \\{cs} not supported: the inherited ones used", name.trim()));
                    if matches!(&**cs, "anchor" | "savedanchor" | "saveddimen" | "savedmacro") {
                        take_group(&mut rest);
                    }
                    take_group(&mut rest);
                }
                _ => {}
            }
        }
        if !super::shapes::is_shape(&shape.base) {
            self.warn(format!("\\pgfdeclareshape{{{}}}: shape '{}' to inherit from not supported: a rectangle", name.trim(), shape.base));
            shape.base = "rectangle".into();
        }
        self.user_shapes.insert(name.trim().to_string(), shape);
        Ok(())
    }

    /// The geometry of the inherited shape with the declared shape's paths: its code run in the
    /// node's coordinates (pt), the saved anchors those of the inherited shape.
    pub fn user_shape_geometry(&mut self, shape: &UserShape, mut g: Geometry) -> Result<Geometry, Error> {
        let define = |e: &mut Engine, name: &str, p: Option<Point>| {
            if let Some(p) = p {
                let code = super::tikra_tokens(&format!("\\pgf@x={}pt\\pgf@y={}pt\\relax", p.x / PT, p.y / PT));
                e.tex.set_meaning(&Token::cs(name), super::macro_tokens(code), false);
            }
        };
        let (sw, ne, c) = (g.anchor("south west"), g.anchor("north east"), g.anchor("center"));
        for (layer, code) in &shape.paths {
            self.begin_group();
            define(self, "southwest", sw);
            define(self, "northeast", ne);
            define(self, "centerpoint", c);
            let saved_path = std::mem::take(&mut self.pgf_path);
            self.st.transform = Transform::default();
            let r = self.run_tokens(code.clone());
            let path = std::mem::replace(&mut self.pgf_path, saved_path);
            self.end_group();
            r?;
            let segments: Vec<Segment> = path.segments;
            // the declared path replaces the inherited one of its layer
            if *layer == Layer::Background {
                g.paths.retain(|p| !(p.layer == Layer::Background && matches!(p.usage, Usage::Node)));
                g.roundable = false;
            }
            g.paths.push(ShapePath { layer: *layer, segments, usage: Usage::Node });
        }
        Ok(g)
    }
}
