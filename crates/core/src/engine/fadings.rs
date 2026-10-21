//! Fadings drawn as pictures: `\pgfdeclarefading{name}{code}` and the TikZ environment
//! `tikzfadingfrompicture[name=…]`. The picture is typeset on its own; where it is white what it
//! fades shows, where it is black (`pgftransparent`, `transparent`) or empty it does not. Used by
//! `\pgfsetfading{name}{transformation}` (in the coordinate system, centered at its origin),
//! `\pgfsetfadingforcurrentpath` (fitted to the path being built), `scope fading` and `path fading`.

use crate::canvas::{Canvas, Item};
use crate::scene::{Point, Segment, Transform};
use crate::tex::{Cat, Token};
use crate::Error;

use super::{tikra_tokens, Engine};

/// A fading's picture: what it draws, and its box (the picture's center is the fading's).
#[derive(Debug, Clone)]
pub struct PictureFading {
    pub items: Vec<Item>,
    pub min: Point,
    pub max: Point,
}

impl PictureFading {
    fn center(&self) -> Point {
        self.min.lerp(self.max, 0.5)
    }

    /// The mask on the canvas: the picture moved so its center is at the origin, then `t`.
    fn mask(&self, t: &Transform) -> Item {
        let c = self.center();
        let t = t.compose(&Transform::translation(-c.x, -c.y));
        let mut items = self.items.clone();
        items.iter_mut().for_each(|i| i.transform(&t));
        Item::FadePicture { items }
    }

    /// The transformation that fits the picture's box onto the box `lo`–`hi` (`fit fading`).
    fn fit(&self, lo: Point, hi: Point) -> Transform {
        let (w, h) = ((self.max.x - self.min.x).max(1e-9), (self.max.y - self.min.y).max(1e-9));
        let c = lo.lerp(hi, 0.5);
        Transform::translation(c.x, c.y).compose(&Transform::scaling((hi.x - lo.x) / w, (hi.y - lo.y) / h))
    }
}

impl Engine {
    /// `\pgfdeclarefading{name}{code}`.
    pub fn declare_fading(&mut self) -> Result<(), Error> {
        let name = self.argument_string()?;
        let code = self.argument()?;
        self.fading_from(name.trim().to_string(), code)
    }

    /// The fading `name` drawn by `code`: typeset as a paragraph of its own (a `\tikz`, a shading,
    /// text), the drawing around it set aside meanwhile.
    fn fading_from(&mut self, name: String, code: Vec<Token>) -> Result<(), Error> {
        let saved_canvas = std::mem::take(&mut self.canvas);
        let saved_boxes = std::mem::take(&mut self.boxes);
        let saved_layout = std::mem::take(&mut self.layout);
        let saved_words = std::mem::take(&mut self.words);
        let saved_depth = std::mem::replace(&mut self.picture_depth, 0);
        let saved_frozen = std::mem::replace(&mut self.canvas_frozen, false);
        self.begin_group();
        // a bare shading or text is a picture's content
        let first = code.iter().find(|t| !t.is_space()).cloned();
        let code = match first {
            Some(Token::Cs(n)) if &*n == "tikz" || &*n == "begin" => code,
            Some(Token::Cs(n)) if &*n == "pgfuseshading" || &*n == "pgftext" => {
                let mut v = tikra_tokens("\\begin{pgfpicture}");
                v.extend(code);
                v.extend(tikra_tokens("\\end{pgfpicture}"));
                v
            }
            _ => {
                let mut v = tikra_tokens("\\begin{pgfpicture}\\pgftext");
                v.push(Token::Char('{', Cat::Begin));
                v.extend(code);
                v.push(Token::Char('}', Cat::End));
                v.extend(tikra_tokens("\\end{pgfpicture}"));
                v
            }
        };
        let r = self.run_tokens(code);
        while self.picture_depth > 0 {
            self.end_picture();
        }
        self.end_group();
        let canvas = super::typeset(&self.boxes, &self.layout, &self.words, 0.0);
        self.canvas = saved_canvas;
        self.boxes = saved_boxes;
        self.layout = saved_layout;
        self.words = saved_words;
        self.picture_depth = saved_depth;
        self.canvas_frozen = saved_frozen;
        r?;
        let Canvas { items, min, max, .. } = canvas;
        if let (Some(min), Some(max)) = (min, max) {
            self.picture_fadings.insert(name, PictureFading { items, min, max });
        }
        Ok(())
    }

    /// `\begin{tikzfadingfrompicture}[name=…] … \end{tikzfadingfrompicture}` (the `\begin` read).
    pub fn fading_from_picture(&mut self) -> Result<(), Error> {
        let options = self.optional()?.unwrap_or_default();
        let mut name = String::new();
        let mut rest = Vec::new();
        for item in crate::keys::split_list(&options) {
            if item.key == "name" {
                name = self.tex.expand_to_string(item.value.unwrap_or_default())?.trim().to_string();
            } else {
                if !rest.is_empty() {
                    rest.push(Token::other(','));
                }
                rest.extend(tikra_tokens(&item.key));
                if let Some(v) = item.value {
                    rest.push(Token::other('='));
                    rest.push(Token::Char('{', Cat::Begin));
                    rest.extend(v);
                    rest.push(Token::Char('}', Cat::End));
                }
            }
        }
        // the body, up to \end{tikzfadingfrompicture}
        let mut body = Vec::new();
        while let Some(t) = self.tex.get_token() {
            if t.is_cs("end") {
                let arg = self.tex.read_undelimited()?;
                if crate::tex::tokens_to_string(&arg).trim() == "tikzfadingfrompicture" {
                    break;
                }
                body.push(t);
                body.push(Token::Char('{', Cat::Begin));
                body.extend(arg);
                body.push(Token::Char('}', Cat::End));
                continue;
            }
            body.push(t);
        }
        let mut code = tikra_tokens("\\begin{tikzpicture}[");
        code.extend(rest);
        code.extend(tikra_tokens("]"));
        code.extend(body);
        code.extend(tikra_tokens("\\end{tikzpicture}"));
        self.fading_from(name, code)
    }

    /// `\pgfsetfading{name}{transformation}`: what the scope draws from here on is faded, the
    /// fading centered at the origin of the coordinate system the code makes.
    pub fn set_fading(&mut self) -> Result<(), Error> {
        let name = self.argument_string()?;
        let code = self.argument()?;
        let t = self.fading_transform(code)?;
        self.open_fading(name.trim(), t);
        Ok(())
    }

    /// `\pgfsetfadingforcurrentpath{name}{transformation}` (and `…stroked`, the path's box widened by
    /// half the line width): the fading fitted to the path being built.
    pub fn set_fading_for_path(&mut self, stroked: bool) -> Result<(), Error> {
        let name = self.argument_string()?;
        let code = self.argument()?;
        let mut c = Canvas::default();
        c.include_path(&self.pgf_path.segments);
        let (Some(mut lo), Some(mut hi)) = (c.min, c.max) else { return Ok(()) };
        if stroked {
            let w = self.st.line_width / 2.0;
            (lo, hi) = (Point::new(lo.x - w, lo.y - w), Point::new(hi.x + w, hi.y + w));
        }
        let extra = self.fading_transform(code)?;
        let (fit, inner) = match self.picture_fadings.get(name.trim()) {
            Some(f) => (f.fit(lo, hi), self.st.transform.inverse().compose(&extra)),
            None => (Transform::default(), extra),
        };
        // the code's transformation acts in the fitted box
        let t = fit.compose(&Transform { e: 0.0, f: 0.0, ..inner });
        self.open_fading(name.trim(), t);
        Ok(())
    }

    /// The transformation the code (`\pgftransformshift{…}`…) makes, on top of the current one.
    fn fading_transform(&mut self, code: Vec<Token>) -> Result<Transform, Error> {
        self.begin_group();
        let r = self.run_tokens(code);
        let t = self.st.transform;
        self.end_group();
        r?;
        Ok(t)
    }

    /// Opens the fading `name` placed by `t` in the current scope, closed with it.
    pub fn open_fading(&mut self, name: &str, t: Transform) {
        match self.picture_fadings.get(name) {
            Some(f) => {
                let mask = f.mask(&t);
                self.open_in_scope(mask);
            }
            None => match self.fadings.get(name).cloned() {
                Some(mut sh) => {
                    let (crate::canvas::Shading::Axial { transform, .. } | crate::canvas::Shading::Radial { transform, .. }) = &mut sh;
                    *transform = t.compose(transform);
                    self.open_in_scope(Item::Fade { mask: sh });
                }
                None => self.warn(format!("fading '{name}' not supported: drawn without it")),
            },
        }
    }

    /// A picture fading for one path, fitted to it (`path fading` with `fit fading`).
    pub fn picture_fading_for(&self, name: &str, segs: &[Segment], fit: bool) -> Option<Item> {
        let f = self.picture_fadings.get(name)?;
        let mut c = Canvas::default();
        c.include_path(segs);
        let (lo, hi) = (c.min?, c.max?);
        let t = if fit { f.fit(lo, hi) } else { self.st.transform };
        Some(f.mask(&t))
    }
}
