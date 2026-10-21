//! TikZ libraries that are styles on top of nodes, paths and trees: `automata` (states, initial and
//! accepting marks), `petri` (places, transitions, tokens), `er` (entities, relationships,
//! attributes) and `shadows` (copies of a node's background drawn behind it). Their keys are TikZ
//! styles with the values of `tikzlibraryautomata.code.tex`, `tikzlibrarypetri.code.tex`,
//! `tikzlibraryer.code.tex` and `tikzlibraryshadows.code.tex`; what they need from TikZ itself
//! (`append after command`, `prefix after command`, `growth function`) is here too.

use std::rc::Rc;

use crate::scene::{Point, Transform};
use crate::tex::Token;
use crate::Error;

use super::nodes::NodeInfo;
use super::options::KeyFn;
use super::Engine;

macro_rules! keyfn {
    ($name:ident, |$e:ident, $v:ident| $body:expr) => {
        fn $name($e: &mut Engine, $v: Option<Vec<Token>>) -> Result<(), Error> {
            $body
        }
    };
}

// `append after command` adds path operations after the node (as `after node path`), `prefix after
// command` puts them before those already there
keyfn!(k_append_after_command, |e, v| {
    e.st.after_node_path.extend(v.unwrap_or_default());
    Ok(())
});
keyfn!(k_prefix_after_command, |e, v| {
    let mut path = v.unwrap_or_default();
    path.append(&mut e.st.after_node_path);
    e.st.after_node_path = path;
    Ok(())
});
// `growth function=\macro`: how a tree places its children instead of `grow` and the distances; the
// macro transforms the coordinate system from the parent, knowing `\tikznumberofchildren` and
// `\tikznumberofcurrentchild`
keyfn!(k_growth_function, |e, v| {
    e.st.growth_function = v.filter(|v| !crate::keys::trim(v).is_empty());
    Ok(())
});

pub const KEYS: &[(&str, KeyFn)] =
    &[("/tikz/append after command", k_append_after_command), ("/tikz/prefix after command", k_prefix_after_command), ("/tikz/growth function", k_growth_function)];

/// The libraries' styles, read as TeX with `@` a letter.
pub const DEFINITIONS: &str = r"
\newcount\tikznumberofchildren
\newcount\tikznumberofcurrentchild
\def\tikzaddafternodepathoption#1{#1\tikzset{prefix after command={\pgfextra{#1}}}}
\tikzset{
  every state/.style={}, every accepting by arrow/.style={}, every initial by arrow/.style={},
  state without output/.style={circle,draw,minimum size=2.5em,every state},
  state with output/.style={circle split,draw,minimum size=2.5em,every state},
  state/.style={state without output},
  accepting by double/.style={double,outer sep=.5\pgflinewidth+.3pt},
  accepting by arrow/.style={after node path={{[to path={[->,double=none,every accepting by arrow] --
    ([shift=(\tikz@accepting@angle:\tikz@accepting@distance)]\tikztostart.\tikz@accepting@angle)
    node[shape=rectangle,anchor=\tikz@accepting@anchor]{\tikz@accepting@text}}] edge ()}}},
  initial by arrow/.style={after node path={{[to path={[->,double=none,every initial by arrow]
    ([shift=(\tikz@initial@angle:\tikz@initial@distance)]\tikztostart.\tikz@initial@angle)
    node[shape=rectangle,anchor=\tikz@initial@anchor]{\tikz@initial@text} -- (\tikztostart)}] edge ()}}},
  initial by diamond/.style={shape=diamond},
  initial text/.code={\tikzaddafternodepathoption{\def\tikz@initial@text{#1}}},
  accepting text/.code={\tikzaddafternodepathoption{\def\tikz@accepting@text{#1}}},
  initial where/.code={\tikzaddafternodepathoption{\csname tikz@initial@where@#1\endcsname}},
  accepting where/.code={\tikzaddafternodepathoption{\csname tikz@accepting@where@#1\endcsname}},
  initial distance/.code={\tikzaddafternodepathoption{\def\tikz@initial@distance{#1}}},
  accepting distance/.code={\tikzaddafternodepathoption{\def\tikz@accepting@distance{#1}}},
  initial above/.style={initial by arrow,initial where=above},
  initial below/.style={initial by arrow,initial where=below},
  initial left/.style={initial by arrow,initial where=left},
  initial right/.style={initial by arrow,initial where=right},
  accepting above/.style={accepting by arrow,accepting where=above},
  accepting below/.style={accepting by arrow,accepting where=below},
  accepting left/.style={accepting by arrow,accepting where=left},
  accepting right/.style={accepting by arrow,accepting where=right},
  accepting/.style={accepting by double}, initial/.style={initial by arrow},
}
\def\tikz@initial@text{start}\def\tikz@accepting@text{}
\def\tikz@initial@distance{3ex}\def\tikz@accepting@distance{3ex}
\def\tikz@initial@angle{180}\def\tikz@initial@anchor{east}
\def\tikz@accepting@angle{0}\def\tikz@accepting@anchor{west}
\def\tikra@where#1#2#3{%
  \expandafter\def\csname tikz@initial@where@#1\endcsname{\def\tikz@initial@angle{#2}\def\tikz@initial@anchor{#3}}%
  \expandafter\def\csname tikz@accepting@where@#1\endcsname{\def\tikz@accepting@angle{#2}\def\tikz@accepting@anchor{#3}}}
\tikra@where{above}{90}{south}\tikra@where{below}{270}{north}
\tikra@where{left}{180}{east}\tikra@where{right}{0}{west}
\tikzset{
  every place/.style={}, every transition/.style={}, every token/.style={},
  place/.style={circle,draw,inner sep=0pt,minimum size=5ex,every place},
  transition/.style={rectangle,draw,inner sep=0pt,minimum size=4mm,every transition},
  pre/.style={<-,shorten <=1pt}, post/.style={->,shorten >=1pt},
  pre and post/.style={<->,shorten >=1pt,shorten <=1pt},
  token/.style={fill=black,draw=none,circle,inner sep=0.5pt,minimum size=1ex,text=white,font=\tiny,every token},
  token distance/.code={\def\tikz@token@distance{#1}},
  children are tokens/.style={growth function=\tikra@grow@tokens,edge from parent path=},
  tokens/.code={\ifnum#1>0 \tikzset{append after command={{[children are tokens]
    child foreach \tikz@dummy in {1,...,#1} {node[token]{}}}}}\fi},
  colored tokens/.code={\tikzset{append after command={{[children are tokens]
    child foreach \tikz@child@color in {#1} {node[token,\tikz@child@color]{}}}}}},
  structured tokens/.code={\tikzset{append after command={{[children are tokens]
    child foreach \tikzpetrichildname in {#1} {node[token]{\tikzpetrichildname}}}}}},
}
\def\tikz@token@distance{1.5ex}
\def\tikra@grow@tokens{\csname tikra@token@\the\tikznumberofchildren @\the\tikznumberofcurrentchild\endcsname}
\def\tikra@token#1#2#3#4{\expandafter\def\csname tikra@token@#1@#2\endcsname{%
  \pgftransformshift{\pgfpoint{#3*\tikz@token@distance}{#4*\tikz@token@distance}}}}
\tikra@token11{0}{0}
\tikra@token21{-.5}{0}\tikra@token22{.5}{0}
\tikra@token31{0}{.57}\tikra@token32{-.5}{-.306025}\tikra@token33{.5}{-.306025}
\tikra@token41{-.5}{.5}\tikra@token42{.5}{.5}\tikra@token43{-.5}{-.5}\tikra@token44{.5}{-.5}
\tikra@token51{0}{.85}\tikra@token52{-.808398}{.26266}\tikra@token53{.808398}{.26266}
\tikra@token54{-.499617}{-.687664}\tikra@token55{.499617}{-.687664}
\tikra@token61{-1}{.5}\tikra@token62{0}{.5}\tikra@token63{1}{.5}
\tikra@token64{-1}{-.5}\tikra@token65{0}{-.5}\tikra@token66{1}{-.5}
\tikra@token71{0}{1}\tikra@token72{-1}{.5}\tikra@token73{0}{0}\tikra@token74{1}{.5}
\tikra@token75{-1}{-.5}\tikra@token76{0}{-1}\tikra@token77{1}{-.5}
\tikra@token81{-.5}{1}\tikra@token82{.5}{1}\tikra@token83{-1}{0}\tikra@token84{0}{0}
\tikra@token85{1}{0}\tikra@token86{-1}{-1}\tikra@token87{0}{-1}\tikra@token88{1}{-1}
\tikra@token91{-1}{1}\tikra@token92{0}{1}\tikra@token93{1}{1}\tikra@token94{-1}{0}\tikra@token95{0}{0}
\tikra@token96{1}{0}\tikra@token97{-1}{-1}\tikra@token98{0}{-1}\tikra@token99{1}{-1}
\tikzset{
  every entity/.style={}, every relationship/.style={}, every attribute/.style={},
  entity/.style={rectangle,draw,minimum height=2\baselineskip,minimum width=4\baselineskip,every entity},
  relationship/.style={diamond,draw,minimum size=1.5\baselineskip,inner sep=1pt,every relationship},
  attribute/.style={minimum size=1.5\baselineskip,ellipse,draw,every attribute},
  key attribute/.style={font=\itshape,attribute},
}
\tikzset{
  shadow scale/.initial=1, shadow xshift/.initial=0pt, shadow yshift/.initial=0pt,
  general shadow/.style={preaction={#1,transform canvas={
    scale around=\pgfkeysvalueof{/tikz/shadow scale}:(current path bounding box.center),
    shift={(\pgfkeysvalueof{/tikz/shadow xshift},\pgfkeysvalueof{/tikz/shadow yshift})}}}},
  drop shadow/.style={general shadow={shadow scale=1,shadow xshift=.5ex,shadow yshift=-.5ex,
    opacity=.5,fill=black!50,every shadow,#1}},
  circular drop shadow/.style={general shadow={shadow scale=1.1,shadow xshift=.3ex,shadow yshift=-.3ex,
    fill=black!50,path fading={circle with fuzzy edge 15 percent},every shadow,#1}},
  circular glow/.style={general shadow={shadow scale=1.25,shadow xshift=0pt,shadow yshift=0pt,
    fill=black!50,path fading={circle with fuzzy edge 15 percent},every shadow,#1}},
  copy shadow/.style={general shadow={shadow scale=1,shadow xshift=.5ex,shadow yshift=.5ex,
    fill=\tikz@fillcolor,draw=\tikz@strokecolor,every shadow,#1}},
  double copy shadow/.style={
    general shadow={shadow scale=1,shadow xshift=.5ex,shadow yshift=.5ex,
      fill=\tikz@fillcolor,draw=\tikz@strokecolor,#1,
      shadow xshift/.expanded=2*\pgfkeysvalueof{/tikz/shadow xshift},
      shadow yshift/.expanded=2*\pgfkeysvalueof{/tikz/shadow yshift}},
    general shadow={shadow scale=1,shadow xshift=.5ex,shadow yshift=.5ex,
      fill=\tikz@fillcolor,draw=\tikz@strokecolor,#1}},
  general shadow/.default=, drop shadow/.default=, circular drop shadow/.default=, circular glow/.default=,
  copy shadow/.default=, double copy shadow/.default=,
}
";

/// Shape keys with nothing set: every key at its default.
struct NoKeys;

impl super::shapes::Keys for NoKeys {
    fn length(&mut self, _: &str, default: f64) -> f64 {
        default
    }
    fn length_of(&mut self, _: &str, default: f64) -> f64 {
        default
    }
    fn number(&mut self, _: &str, default: f64) -> f64 {
        default
    }
    fn text(&mut self, _: &str) -> String {
        String::new()
    }
    fn flag(&mut self, _: &str) -> bool {
        false
    }
    fn point(&mut self, _: &str) -> Option<Point> {
        None
    }
}

/// A rectangle node over a box (`current bounding box`, `current path bounding box`), with its anchors.
pub fn box_node(lo: Point, hi: Point) -> NodeInfo {
    use super::shapes::{compute, BoxSize, Input};
    let parts = [("text".to_string(), BoxSize { wd: hi.x - lo.x, ht: hi.y - lo.y, dp: 0.0 })];
    let mut keys = NoKeys;
    let mut input = Input {
        parts: &parts,
        inner_xsep: 0.0,
        inner_ysep: 0.0,
        minimum_width: 0.0,
        minimum_height: 0.0,
        outer_xsep: 0.0,
        outer_ysep: 0.0,
        aspect: 1.0,
        ex: 0.0,
        line_width: 0.0,
        pointer: None,
        keys: &mut keys,
    };
    let geometry = compute("rectangle", &mut input).expect("a rectangle");
    NodeInfo { shape: "rectangle".into(), geometry: Rc::new(geometry), transform: Transform::translation(lo.x, lo.y) }
}
