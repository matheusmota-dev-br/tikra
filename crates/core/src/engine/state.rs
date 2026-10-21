//! The graphic state, as pgf and TikZ keep it: saved when a TeX group starts and put back when it ends.

use std::collections::HashMap;
use std::rc::Rc;

use crate::canvas::{Cap, Join};
use crate::color::Color;
use crate::scene::{Point, Transform};
use crate::PT;

/// What a path does once built (`\tikz@mode`): set by `draw`, `fill`… for one path only.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mode {
    pub draw: bool,
    pub fill: bool,
    pub clip: bool,
    pub pattern: bool,
    pub shade: bool,
    /// `use as bounding box`
    pub bounding_box: bool,
}

#[derive(Debug, Clone)]
pub struct GState {
    /// pgfkeys' default path (`\pgfkeysdefaultpath`)
    pub key_path: String,
    /// The current transformation (pgf's), from canvas cm.
    pub transform: Transform,
    /// Transformations given in options and not applied yet (`\tikz@transform`).
    pub pending: Transform,
    /// Unit vectors of the xy (and z) coordinate system, in cm.
    pub x: Point,
    pub y: Point,
    pub z: Point,
    pub mode: Mode,

    /// The current color (`color=`, a color name): stroke, fill and text unless they are set apart.
    pub color: Color,
    pub draw_color: Option<Color>,
    pub fill_color: Option<Color>,
    pub text_color: Option<Color>,
    pub pattern_name: Option<String>,
    pub pattern_color: Option<Color>,
    pub line_width: f64,
    pub dash: Option<(Vec<f64>, f64)>,
    pub cap: Cap,
    pub join: Join,
    pub miter_limit: f64,
    pub draw_opacity: f64,
    pub fill_opacity: f64,
    pub text_opacity: f64,
    pub even_odd: bool,
    /// `rounded corners=` radius, or 0
    pub rounded: f64,
    /// Arrow specifications at the start and the end (`->`, `-{Latex[length=3mm]}`).
    pub start_arrow: Option<String>,
    pub end_arrow: Option<String>,
    /// What `>` stands for (`>=Latex`).
    pub default_arrow: String,
    pub shorten_start: f64,
    pub shorten_end: f64,
    /// `double`: the color between the two lines, and the distance between them
    pub double: Option<Color>,
    pub double_distance: f64,
    /// Font: size in pt, and the TeX commands that set it (`\bfseries`, `\itshape`)
    pub font_size: f64,
    pub font: String,

    // path construction settings
    pub to_out: Option<f64>,
    pub to_in: Option<f64>,
    pub bend: Option<f64>,
    pub looseness: f64,
    pub out_looseness: Option<f64>,
    pub in_looseness: Option<f64>,
    pub grid_step: (f64, f64),
    pub domain: (f64, f64),
    pub samples: usize,
    pub variable: String,
    pub smooth: bool,
    pub tension: f64,
    pub plot_mark: Option<String>,
    pub only_marks: bool,
    pub decorate: bool,
    pub decoration: String,
    pub amplitude: f64,
    pub segment_length: f64,
    /// `declare function`
    pub functions: Rc<HashMap<String, (Vec<String>, String)>>,
    /// settings of the node being created
    pub node: super::nodes::NodeSetup,
    /// `baseline=` of the picture
    pub baseline: Option<String>,
    /// `auto`: nodes on a segment go to its left (`'l'`) or right (`'r'`)
    pub auto: Option<char>,
    /// `swap` (or `'`): the other side
    pub swap: bool,
    /// quotes (`"text" above`) in the options being read make nodes on the path, not labels
    pub quotes_on_path: bool,
    /// `execute at begin scope`… code gathered from the options, run when TikZ runs it
    pub at_begin_scope: Vec<crate::tex::Token>,
    pub at_end_scope: Vec<crate::tex::Token>,
    pub at_begin_picture: Vec<crate::tex::Token>,
    pub at_end_picture: Vec<crate::tex::Token>,
    /// `name path=`: the path is kept under this name (intersections)
    pub name_path: Option<String>,
    /// nodes made by quotes in `to`/`edge` options: text and options
    pub quoted: Vec<(Vec<crate::tex::Token>, Vec<crate::tex::Token>)>,
}

impl Default for GState {
    fn default() -> Self {
        GState {
            key_path: "/".into(),
            transform: Transform::default(),
            pending: Transform::default(),
            x: Point::new(1.0, 0.0),
            y: Point::new(0.0, 1.0),
            z: Point::new(-0.3851, -0.3851),
            mode: Mode::default(),
            color: Color::BLACK,
            draw_color: None,
            fill_color: None,
            text_color: None,
            pattern_name: None,
            pattern_color: None,
            line_width: 0.4 * PT,
            dash: None,
            cap: Cap::Butt,
            join: Join::Miter,
            miter_limit: 10.0,
            draw_opacity: 1.0,
            fill_opacity: 1.0,
            text_opacity: 1.0,
            even_odd: false,
            rounded: 0.0,
            start_arrow: None,
            end_arrow: None,
            default_arrow: "to".into(),
            shorten_start: 0.0,
            shorten_end: 0.0,
            double: None,
            double_distance: 0.6 * PT,
            font_size: 10.0,
            font: String::new(),
            to_out: None,
            to_in: None,
            bend: None,
            looseness: 1.0,
            out_looseness: None,
            in_looseness: None,
            grid_step: (1.0, 1.0),
            domain: (-5.0, 5.0),
            samples: 25,
            variable: "x".into(),
            smooth: false,
            tension: 0.55,
            plot_mark: None,
            only_marks: false,
            decorate: false,
            decoration: String::new(),
            amplitude: 2.5 * PT,
            segment_length: 10.0 * PT,
            functions: Rc::new(HashMap::new()),
            node: Default::default(),
            baseline: None,
            auto: None,
            swap: false,
            quotes_on_path: false,
            name_path: None,
            at_begin_scope: Vec::new(),
            at_end_scope: Vec::new(),
            at_begin_picture: Vec::new(),
            at_end_picture: Vec::new(),
            quoted: Vec::new(),
        }
    }
}

impl GState {
    pub fn stroke_color(&self) -> Color {
        self.draw_color.unwrap_or(self.color)
    }
    pub fn fill_paint_color(&self) -> Color {
        self.fill_color.unwrap_or(self.color)
    }
    pub fn text_paint_color(&self) -> Color {
        self.text_color.unwrap_or(self.color)
    }
    /// Applies the pending transformations (`\tikz@transform`) and clears them.
    pub fn apply_pending(&mut self) {
        self.transform = self.transform.compose(&self.pending);
        self.pending = Transform::default();
    }
}
