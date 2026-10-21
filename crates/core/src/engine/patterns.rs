//! The basic layer's patterns: `\pgfdeclarepatternformonly[vars]{name}{lower left}{upper right}{tile
//! size}{code}` (drawn in the color the pattern is used with) and
//! `\pgfdeclarepatterninherentlycolored` (in its own colors); TikZ's `pattern=name` and
//! `\pgfsetfillpattern{name}{color}` fill with them, the tile repeated from the page's origin.

use crate::canvas::{Canvas, Item, Paint, TileItems};
use crate::color::Color;
use crate::scene::Point;
use crate::Error;

use super::Engine;

/// A declared pattern: one tile's items and its size (canvas cm), and whether it has its own colors.
#[derive(Debug, Clone)]
pub struct UserPattern {
    items: Vec<Item>,
    size: Point,
    colored: bool,
}

impl Engine {
    /// `\pgfdeclarepatternformonly` (`colored` false) and `\pgfdeclarepatterninherentlycolored`.
    pub fn declare_pattern(&mut self, colored: bool) -> Result<(), Error> {
        let _ = self.optional()?;
        let name = self.argument_string()?;
        let _lower_left = self.point_arg()?;
        let _upper_right = self.point_arg()?;
        let size = self.point_arg()?;
        let code = self.argument()?;
        // the code draws one tile, in a picture of its own, its coordinates kept
        let saved_canvas = std::mem::take(&mut self.canvas);
        let saved_depth = std::mem::replace(&mut self.picture_depth, 1);
        let saved_frozen = std::mem::replace(&mut self.canvas_frozen, false);
        self.begin_group();
        self.begin_scope();
        self.st.transform = Default::default();
        let r = self.run_tokens(code);
        self.end_scope();
        self.end_group();
        let Canvas { items, .. } = std::mem::replace(&mut self.canvas, saved_canvas);
        self.picture_depth = saved_depth;
        self.canvas_frozen = saved_frozen;
        r?;
        self.user_patterns.insert(name.trim().to_string(), UserPattern { items, size, colored });
        Ok(())
    }

    /// A declared pattern as a paint, form-only ones in `color`.
    pub fn pattern_paint(&self, name: &str, color: Color) -> Option<Paint> {
        let p = self.user_patterns.get(name)?;
        let mut items = p.items.clone();
        if !p.colored {
            for i in items.iter_mut() {
                if let Item::Path { fill, stroke, .. } = i {
                    if let Some(f) = fill {
                        f.paint = Paint::Solid(color);
                    }
                    if let Some(s) = stroke {
                        s.color = color;
                    }
                }
            }
        }
        Some(Paint::Tile { items: TileItems(std::rc::Rc::new(items)), size: p.size })
    }

    /// `\pgfsetfillpattern{name}{color}`: the basic layer's fills from here on use the pattern.
    pub fn set_fill_pattern(&mut self) -> Result<(), Error> {
        let name = self.argument_string()?;
        let color = self.argument_string()?;
        let c = self.color(color.trim()).unwrap_or(Color::BLACK);
        self.st.pattern_name = Some(name.trim().to_string());
        self.st.pattern_color = Some(c);
        self.st.fill_pattern = true;
        Ok(())
    }
}
