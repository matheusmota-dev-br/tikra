//! The circuit libraries of TikZ (`circuits`, `circuits.logic`, `circuits.logic.US`,
//! `circuits.logic.IEC`, `circuits.logic.CDH`, `circuits.ee`, `circuits.ee.IEC`), from their
//! `tikzlibrarycircuits.*.code.tex`.
//!
//! A circuit symbol (`resistor`, `and gate`…) is a key declared by `circuit declare symbol`: given
//! to a node, it makes the node look like the symbol (`set resistor graphic`, the shapes of
//! [`super::shapes`]); given to a `to`, it is `circuit handle symbol`, which puts the symbol as a node
//! on the line at its `pos` (`.5` by default), turned along it, the line stopping at its border: a
//! `markings` decoration whose marks are the symbols and whose `mark connection node` cuts the line.
//! `info`, `ohm`, `volt`… add labels; annotations (`light emitting`, `direction info`) draw arrows
//! by the symbol. The definitions are pgf's, run as TeX, in the order the manual loads the libraries
//! (US, IEC, CDH, then ee); only the position grabber and the turn of a symbol at an end of the line
//! are done natively.

use crate::keys;
use crate::scene::{Point, Transform};
use crate::tex::Token;
use crate::Error;

use super::Engine;

/// `circuits/tikra pos`: the `pos` of a symbol's options (the keys of pgf's `pos grabber` family,
/// the last one winning), in `\tikz@lib@circ@pos`.
fn k_pos(e: &mut Engine, v: Option<Vec<Token>>) -> Result<(), Error> {
    let mut pos = 0.5;
    for item in keys::split_list(&v.unwrap_or_default()) {
        let p = match item.key.as_str() {
            "pos" => match item.value {
                Some(v) => e.number_of(v)?,
                None => continue,
            },
            "at start" => 0.0,
            "very near start" => 0.125,
            "near start" => 0.25,
            "midway" => 0.5,
            "near end" => 0.75,
            "very near end" => 0.875,
            "at end" => 1.0,
            _ => continue,
        };
        pos = p;
    }
    e.tex.define_string("tikz@lib@circ@pos", &format!("{pos}"), false);
    Ok(())
}

/// `circuits/tikra direction=start` (or `target`): `\tikz@lib@circ@compute@direction`, the
/// coordinate system turned along the `to` from its start to its target, with its origin at that end.
fn k_direction(e: &mut Engine, v: Option<Vec<Token>>) -> Result<(), Error> {
    let which = e.tex.expand_to_string(v.unwrap_or_default())?;
    let point = |e: &mut Engine, m: &str| -> Result<Point, Error> {
        let name = e.tex.expand_to_string(vec![Token::cs(m)])?;
        Ok(e.coordinate(&name)?.p)
    };
    let a = point(e, "tikztostart")?;
    let b = point(e, "tikztotarget")?;
    let d = b.minus(a);
    let angle = d.y.atan2(d.x).to_degrees();
    let at = if which.trim() == "target" { b } else { a };
    let t = e.st.transform;
    let linear = Transform { e: 0.0, f: 0.0, ..t };
    e.st.transform = Transform::translation(at.x, at.y).compose(&Transform::rotation(angle)).compose(&linear);
    Ok(())
}

/// `\pgfsetarrowoptions{name}{value}`, as `circuits/tikra arrow options={name}{value}`: the option of
/// the arrow tip `direction ee` (its length), kept in `/pgf/arrow options/name` for the shape
/// `direction ee` and given to the tip.
fn k_arrow_options(e: &mut Engine, v: Option<Vec<Token>>) -> Result<(), Error> {
    let args = super::options::split_arguments(&v.unwrap_or_default(), 2);
    let name = e.tex.expand_to_string(args[0].clone())?.trim().to_string();
    let d = e.length_of(args[1].clone())? / crate::PT;
    e.tex.define_string(&keys::macro_name(&format!("/pgf/arrow options/{name}")), &format!("{d}pt"), false);
    if name == "direction ee" {
        std::rc::Rc::make_mut(&mut e.st.tips).declare(&name, &format!("tikra direction ee[length={d}pt]"));
    }
    Ok(())
}

/// `circuits/tikra annotation labels={options}{distance}`: the labels among an annotation's options
/// (`light emitting={info=…}`), on the symbol, at the annotation's label distance (TikZ puts them
/// after the annotation's path, where `\tikzlastnode` is still the symbol).
fn k_annotation_labels(e: &mut Engine, v: Option<Vec<Token>>) -> Result<(), Error> {
    let args = super::options::split_arguments(&v.unwrap_or_default(), 2);
    let Some(node) = e.last_node.clone() else { return Ok(()) };
    e.begin_group();
    let saved = std::mem::take(&mut e.st.node.labels);
    let r = e.process_keys(args[0].clone(), Some("/tikz"));
    let labels = std::mem::replace(&mut e.st.node.labels, saved);
    e.end_group();
    r?;
    for (spec, pin) in labels {
        e.begin_group();
        let mut d = crate::tex::string_tokens("label distance=");
        d.push(Token::Char('{', crate::tex::Cat::Begin));
        d.extend(args[1].clone());
        d.push(Token::Char('}', crate::tex::Cat::End));
        let r = e.process_keys(d, Some("/tikz")).and_then(|_| e.label(&node, spec, pin));
        e.end_group();
        e.canvas.items.extend(r?);
    }
    Ok(())
}

/// The keys done natively.
pub const KEYS: &[(&str, super::options::KeyFn)] = &[
    ("/tikz/circuits/tikra pos", k_pos),
    ("/tikz/circuits/tikra direction", k_direction),
    ("/tikz/circuits/tikra arrow options", k_arrow_options),
    ("/tikz/circuits/tikra annotation labels", k_annotation_labels),
];

/// The libraries' definitions.
pub const DEFINITIONS: &str = r#"
\def\pgfsetarrowoptions#1#2{\pgfkeys{/tikz/circuits/tikra arrow options={{#1}{#2}}}}
\newif\iftikz@lib@circ@on@to
\newcount\tikz@lib@circ@count
\newdimen\tikzcircuitssizeunit
\tikzcircuitssizeunit=7pt
\let\tikz@lib@circ@start@node\pgfutil@empty
\def\tikz@lib@circ@end{-- (\tikztotarget) \tikztonodes}
\tikzset{
  circuit/.style={
    every circuit/.try,
    /utils/exec=\tikz@lib@circ@on@tofalse,
    execute at begin to={%
      \tikz@lib@circ@on@totrue
      \let\tikz@lib@circ@start@node\pgfutil@empty
      \def\tikz@lib@circ@end{-- (\tikztotarget) \tikztonodes}%
    }
  },
  current point is local/.code={},
  /utils/exec/.code={#1},
  cap/.style={line cap=#1},
  circuit handle symbol/.code={%
    \global\advance\tikz@lib@circ@count by1\relax
    \iftikz@lib@circ@on@to
      \pgfkeys{/tikz/circuits/tikra pos={#1}}%
      \pgfkeysalso{to path={\pgfextra{\tikz@lib@circ@on@tofalse} decorate [decoration={markings,mark connection node=mark node}] {\tikz@lib@circ@start@node \tikz@lib@circ@end}}}%
      \ifdim\tikz@lib@circ@pos pt=0pt\relax
        \def\tikz@lib@circ@start@node{{\pgfextra{\pgfkeys{/tikz/circuits/tikra direction=start}}node[alias=tikz@lib@circ@node@start,at={(0,0)},#1,pos=]{}}(tikz@lib@circ@node@start)}%
      \else
        \ifdim\tikz@lib@circ@pos pt=1pt\relax
          \edef\tikz@lib@circ@endname{tikz@lib@circ@node\the\tikz@lib@circ@count}%
          \def\tikz@lib@circ@end{{\pgfextra{\pgfkeys{/tikz/circuits/tikra direction=target}}node[alias=\tikz@lib@circ@endname,at={(0,0)},#1,pos=]{}} -- (\tikz@lib@circ@endname) \tikztonodes}%
        \else
          \edef\tikz@marshal{mark=at position \tikz@lib@circ@pos\space with}%
          \def\tikz@marshala{decoration=}%
          \expandafter\expandafter\expandafter\tikzset\expandafter\expandafter\expandafter{\expandafter\tikz@marshala\expandafter{\tikz@marshal{\node[alias=mark node,#1,pos=]{};}}}%
        \fi
      \fi
    \else
      \pgfkeysalso{#1}%
    \fi
  },
  circuit declare symbol/.style={
    #1/.style={circuit handle symbol={inner sep=0.5pt,every circuit symbol,#1/graphic,every #1/.try,##1}},
    #1/graphic/.style={},
    set #1 graphic/.style={#1/graphic/.style={##1}}
  },
  circuit symbol unit/.code=\pgfmathsetlength\tikzcircuitssizeunit{#1},
  circuit symbol size/.style args={width #1 height #2}{
    minimum width={(#1)*\tikzcircuitssizeunit},
    minimum height={(#2)*\tikzcircuitssizeunit}
  },
  huge circuit symbols/.style={circuit symbol unit=10pt},
  large circuit symbols/.style={circuit symbol unit=8pt},
  medium circuit symbols/.style={circuit symbol unit=7pt},
  small circuit symbols/.style={circuit symbol unit=6pt},
  tiny circuit symbols/.style={circuit symbol unit=5pt},
  circuit declare annotation/.style args={#1#2#3}{
    #1/.style={after node path={\pgfextra{\tikra@circ@annotation{#1}{#2}{##1}{north}{}{#3}}}},
    #1'/.style={after node path={\pgfextra{\tikra@circ@annotation{#1}{#2}{##1}{south}{yscale=-1}{#3}}}}
  },
  annotation arrow/.style={
    /utils/exec={\pgfsetarrowoptions{direction ee}{.4*\the\tikzcircuitssizeunit+.3*\the\pgflinewidth}},
    >=direction ee
  },
  point up/.style={rotate=90},
  point down/.style={rotate=-90},
  point left/.style={rotate=180},
  point right/.style={},
  every circuit symbol/.style={},
  circuit symbol open/.style={draw},
  circuit symbol filled/.style={draw,fill=black},
  circuit symbol lines/.style={draw},
  circuit symbol wires/.style={draw},
  info/.code={\pgfutil@ifnextchar[\tikz@lib@circ@lab@plain{\tikz@lib@circ@lab@plain[]}#1\pgf@stop},
  info'/.code={\pgfutil@ifnextchar[\tikz@lib@circ@labp@plain{\tikz@lib@circ@labp@plain[]}#1\pgf@stop},
  info sloped/.code={\pgfutil@ifnextchar[\tikz@lib@circ@lab@sloped@plain{\tikz@lib@circ@lab@sloped@plain[]}#1\pgf@stop},
  info' sloped/.code={\pgfutil@ifnextchar[\tikz@lib@circ@lab@slopedp@plain{\tikz@lib@circ@lab@slopedp@plain[]}#1\pgf@stop},
  circuit declare unit/.style 2 args={
    #1/.code={\pgfutil@ifnextchar[\tikz@lib@circ@lab{\tikz@lib@circ@lab[]}##1\pgf@stop{#2}{#1}},
    #1 sloped/.code={\pgfutil@ifnextchar[\tikz@lib@circ@lab@sloped{\tikz@lib@circ@lab@sloped[]}##1\pgf@stop{#2}{#1}},
    #1'/.code={\pgfutil@ifnextchar[\tikz@lib@circ@labp{\tikz@lib@circ@labp[]}##1\pgf@stop{#2}{#1}},
    #1' sloped/.code={\pgfutil@ifnextchar[\tikz@lib@circ@lab@slopedp{\tikz@lib@circ@lab@slopedp[]}##1\pgf@stop{#2}{#1}}
  }
}
\def\tikra@circ@annotation#1#2#3#4#5#6{\path[every circuit annotation/.try,every #1/.try,shift={(\tikzlastnode.#4)},#5,annotation arrow,->,label distance=#2,#3] #6;\pgfkeys{/tikz/circuits/tikra annotation labels={{#3}{#2}}}}
\def\tikz@lib@circ@lab[#1]#2\pgf@stop#3#4{\tikzset{label={[every info/.try,every #4/.try,#1]$\mathrm{#2#3}$}}}
\def\tikz@lib@circ@lab@sloped[#1]#2\pgf@stop#3#4{\tikzset{label={[every info/.try,every #4/.try,transform shape,#1]$\mathrm{#2#3}$}}}
\def\tikz@lib@circ@labp[#1]#2\pgf@stop#3#4{\tikzset{label={[label position=below,every info/.try,every #4/.try,#1]$\mathrm{#2#3}$}}}
\def\tikz@lib@circ@lab@slopedp[#1]#2\pgf@stop#3#4{\tikzset{label={[label position=below,every info/.try,every #4/.try,transform shape,#1]$\mathrm{#2#3}$}}}
\def\tikz@lib@circ@lab@plain[#1]#2\pgf@stop{\tikzset{label={[every info/.try,#1]#2}}}
\def\tikz@lib@circ@lab@sloped@plain[#1]#2\pgf@stop{\tikzset{label={[every info/.try,transform shape,#1]#2}}}
\def\tikz@lib@circ@labp@plain[#1]#2\pgf@stop{\tikzset{label={[label position=below,every info/.try,#1]#2}}}
\def\tikz@lib@circ@lab@slopedp@plain[#1]#2\pgf@stop{\tikzset{label={[label position=below,every info/.try,transform shape,#1]#2}}}
\tikzset{
  circuit logic/.style={
    circuit,
    inputs/.style={logic gate inputs={##1}},
    logic gate inverted radius=.25\tikzcircuitssizeunit,
    every circuit logic/.try,
  },
  circuit declare symbol=and gate, circuit declare symbol=nand gate,
  circuit declare symbol=or gate, circuit declare symbol=nor gate,
  circuit declare symbol=xor gate, circuit declare symbol=xnor gate,
  circuit declare symbol=not gate, circuit declare symbol=buffer gate,
  circuit logic US/.style={
    circuit logic,
    set and gate graphic=and gate US graphic, set nand gate graphic=nand gate US graphic,
    set or gate graphic=or gate US graphic, set nor gate graphic=nor gate US graphic,
    set xor gate graphic=xor gate US graphic, set xnor gate graphic=xnor gate US graphic,
    set not gate graphic=not gate US graphic, set buffer gate graphic=buffer gate US graphic
  },
  circuit logic US make graphic/.style={
    #1 graphic/.style={circuit symbol open,circuit symbol size=width 0 height 2.5,shape=#1,transform shape}
  },
  circuit logic US make graphic=and gate US, circuit logic US make graphic=nand gate US,
  circuit logic US make graphic=or gate US, circuit logic US make graphic=nor gate US,
  circuit logic US make graphic=xor gate US, circuit logic US make graphic=xnor gate US,
  circuit logic US make graphic=not gate US, circuit logic US make graphic=buffer gate US,
  circuit logic IEC/.style={
    circuit logic,
    set and gate graphic=and gate IEC graphic, set nand gate graphic=nand gate IEC graphic,
    set or gate graphic=or gate IEC graphic, set nor gate graphic=nor gate IEC graphic,
    set xor gate graphic=xor gate IEC graphic, set xnor gate graphic=xnor gate IEC graphic,
    set not gate graphic=not gate IEC graphic, set buffer gate graphic=buffer gate IEC graphic,
    and gate IEC symbol={$\scriptstyle\char`\&$}, nand gate IEC symbol={$\scriptstyle\char`\&$},
    or gate IEC symbol={$\scriptstyle\geq1$}, nor gate IEC symbol={$\scriptstyle\geq1$},
    xor gate IEC symbol={$\scriptstyle=1$}, xnor gate IEC symbol={$\scriptstyle=1$},
    not gate IEC symbol={$\scriptstyle1$}, buffer gate IEC symbol={$\scriptstyle1$},
  },
  circuit logic IEC make graphic/.style={
    #1 graphic/.style={circuit symbol open,circuit symbol size=width 2.5 height 4,shape=#1,inner sep=.5ex}
  },
  circuit logic IEC make graphic=and gate IEC, circuit logic IEC make graphic=nand gate IEC,
  circuit logic IEC make graphic=or gate IEC, circuit logic IEC make graphic=nor gate IEC,
  circuit logic IEC make graphic=xor gate IEC, circuit logic IEC make graphic=xnor gate IEC,
  circuit logic IEC make graphic=not gate IEC, circuit logic IEC make graphic=buffer gate IEC,
  circuit logic CDH/.style={
    circuit logic US,
    set and gate graphic=and gate CDH graphic, set nand gate graphic=nand gate CDH graphic,
  },
  circuit logic US make graphic=and gate CDH, circuit logic US make graphic=nand gate CDH,
}
\tikzset{
  circuit ee/.style={circuit,every circuit ee/.try},
  circuit declare symbol=resistor, circuit declare symbol=inductor,
  circuit declare symbol=capacitor, circuit declare symbol=contact,
  circuit declare symbol=ground, circuit declare symbol=battery,
  circuit declare symbol=diode, circuit declare symbol=Zener diode,
  circuit declare symbol=Schottky diode, circuit declare symbol=tunnel diode,
  circuit declare symbol=backward diode, circuit declare symbol=breakdown diode,
  circuit declare symbol=bulb, circuit declare symbol=current source,
  circuit declare symbol=voltage source, circuit declare symbol=current direction,
  circuit declare symbol=current direction', circuit declare symbol=make contact,
  circuit declare symbol=break contact, circuit declare symbol=amperemeter,
  circuit declare symbol=voltmeter, circuit declare symbol=ohmmeter,
  circuit declare symbol=ac source, circuit declare symbol=dc source,
  set current direction graphic=current direction ee graphic,
  set current direction' graphic=current direction' ee graphic,
  circuit declare unit={ampere}{A}, circuit declare unit={volt}{V},
  circuit declare unit={ohm}{\Omega}, circuit declare unit={siemens}{S},
  circuit declare unit={henry}{H}, circuit declare unit={farad}{F},
  circuit declare unit={coulomb}{C}, circuit declare unit={voltampere}{VA},
  circuit declare unit={watt}{W}, circuit declare unit={hertz}{Hz},
  current direction arrow/.style={
    /utils/exec={\pgfsetarrowoptions{direction ee}{1.3065*.5*\the\tikzcircuitssizeunit+1.3065*.3*\the\pgflinewidth}},
    >=direction ee,
    direction ee arrow=direction ee,
  },
  current direction ee graphic/.style={
    shape=direction ee, circuit symbol filled, current direction arrow,
    minimum width=.5*\the\tikzcircuitssizeunit+.3*\the\pgflinewidth,
    minimum height=.5*\the\tikzcircuitssizeunit+.3*\the\pgflinewidth,
    transform shape
  },
  current direction' ee graphic/.style={current direction ee graphic,rotate=180},
  circuit declare annotation={direction info}{.5\tikzcircuitssizeunit}
  {(-1.25\tikzcircuitssizeunit,.3333\tikzcircuitssizeunit) edge[line to] (1.25\tikzcircuitssizeunit,.3333\tikzcircuitssizeunit)},
  circuit declare annotation={light emitting}{1.75\tikzcircuitssizeunit}
  {(-.2\tikzcircuitssizeunit,.65\tikzcircuitssizeunit) edge[line to] ++(45:1.25\tikzcircuitssizeunit)
   (.2\tikzcircuitssizeunit,.25\tikzcircuitssizeunit) edge[line to] ++(45:1.25\tikzcircuitssizeunit)},
  circuit declare annotation={light dependent}{1.75\tikzcircuitssizeunit}
  {[shift=(135:1.25\tikzcircuitssizeunit)]
   (.2\tikzcircuitssizeunit,.65\tikzcircuitssizeunit) edge[line to] ++(-45:1.25\tikzcircuitssizeunit)
   (-.2\tikzcircuitssizeunit,.25\tikzcircuitssizeunit) edge[line to] ++(-45:1.25\tikzcircuitssizeunit)},
  circuit declare annotation={adjustable}{1.5\tikzcircuitssizeunit}
  {[shift=(\tikzlastnode.center)]
   (-1.5\tikzcircuitssizeunit,-1.5\tikzcircuitssizeunit) edge[line to] (1.5\tikzcircuitssizeunit,1.5\tikzcircuitssizeunit)},
  circuit declare annotation={adjustable'}{1.5\tikzcircuitssizeunit}
  {[shift=(\tikzlastnode.center)]
   (-1.5\tikzcircuitssizeunit,1.5\tikzcircuitssizeunit) edge[line to] (1.5\tikzcircuitssizeunit,-1.5\tikzcircuitssizeunit)},
}
\tikzset{
  circuit ee IEC/.style={
    circuit ee,
    set resistor graphic=resistor IEC graphic, set inductor graphic=inductor IEC graphic,
    set capacitor graphic=capacitor IEC graphic, set contact graphic=contact IEC graphic,
    set ground graphic=ground IEC graphic, set battery graphic=battery IEC graphic,
    set diode graphic=diode IEC graphic, set Zener diode graphic=Zener diode IEC graphic,
    set tunnel diode graphic=tunnel diode IEC graphic, set backward diode graphic=backward diode IEC graphic,
    set Schottky diode graphic=Schottky diode IEC graphic, set breakdown diode graphic=breakdown diode IEC graphic,
    set bulb graphic=bulb IEC graphic, set voltage source graphic=voltage source IEC graphic,
    set current source graphic=current source IEC graphic, set make contact graphic=make contact IEC graphic,
    set break contact graphic=break contact IEC graphic, set amperemeter graphic=amperemeter graphic,
    set voltmeter graphic=voltmeter graphic, set ohmmeter graphic=ohmmeter graphic,
    set ac source graphic=ac source graphic, set dc source graphic=dc source graphic,
  },
  resistor IEC graphic/.style={circuit symbol open,circuit symbol size=width 4 height 1,shape=rectangle ee,transform shape},
  var resistor IEC graphic/.style={circuit symbol lines,circuit symbol size=width 4.8 height 0.8,shape=var resistor IEC,transform shape,outer sep=0pt,cap=round},
  inductor IEC graphic/.style={circuit symbol lines,circuit symbol size=width 4 height .5,transform shape,shape=inductor IEC,outer sep=0pt,cap=round},
  var inductor IEC graphic/.style={circuit symbol filled,circuit symbol size=width 4 height 1,transform shape,shape=rectangle ee},
  capacitor IEC graphic/.style={circuit symbol lines,circuit symbol size=width .5 height 2,transform shape,shape=capacitor IEC},
  ground IEC graphic/.style={circuit symbol lines,circuit symbol size=width .75 height 2,transform shape,shape=ground IEC},
  battery IEC graphic/.style={circuit symbol lines,circuit symbol size=width .5 height 2.5,transform shape,shape=battery IEC},
  diode IEC graphic/.style={circuit symbol open,circuit symbol size=width 1.25 height 1.25,transform shape,shape=generic diode IEC,
    /pgf/generic diode IEC/before background={\pgfpathmoveto{\pgfqpoint{0pt}{-1pt}}\pgfpathlineto{\pgfqpoint{0pt}{1pt}}\pgfusepathqstroke}},
  var diode IEC graphic/.style={diode IEC graphic,circuit symbol filled},
  Zener diode IEC graphic/.style={diode IEC graphic,
    /pgf/generic diode IEC/before background={\pgfpathmoveto{\pgfqpoint{-.5pt}{-1pt}}\pgfpathlineto{\pgfqpoint{0pt}{-1pt}}\pgfpathlineto{\pgfqpoint{0pt}{1pt}}\pgfusepathqstroke}},
  var Zener diode IEC graphic/.style={Zener diode IEC graphic,circuit symbol filled},
  Schottky diode IEC graphic/.style={diode IEC graphic,
    /pgf/generic diode IEC/before background={\pgfpathmoveto{\pgfqpoint{-.4pt}{-.6pt}}\pgfpathlineto{\pgfqpoint{-.4pt}{-1pt}}\pgfpathlineto{\pgfqpoint{0pt}{-1pt}}\pgfpathlineto{\pgfqpoint{0pt}{1pt}}\pgfpathlineto{\pgfqpoint{.4pt}{1pt}}\pgfpathlineto{\pgfqpoint{.4pt}{.6pt}}\pgfusepathqstroke}},
  var Schottky diode IEC graphic/.style={Schottky diode IEC graphic,circuit symbol filled},
  tunnel diode IEC graphic/.style={diode IEC graphic,
    /pgf/generic diode IEC/before background={\pgfpathmoveto{\pgfqpoint{-.5pt}{-1pt}}\pgfpathlineto{\pgfqpoint{0pt}{-1pt}}\pgfpathlineto{\pgfqpoint{0pt}{1pt}}\pgfpathlineto{\pgfqpoint{-.5pt}{1pt}}\pgfusepathqstroke}},
  var tunnel diode IEC graphic/.style={tunnel diode IEC graphic,circuit symbol filled},
  backward diode IEC graphic/.style={diode IEC graphic,
    /pgf/generic diode IEC/before background={\pgfpathmoveto{\pgfqpoint{-.5pt}{-1pt}}\pgfpathlineto{\pgfqpoint{.5pt}{-1pt}}\pgfpathmoveto{\pgfqpoint{0pt}{-1pt}}\pgfpathlineto{\pgfqpoint{0pt}{1pt}}\pgfpathmoveto{\pgfqpoint{-.5pt}{1pt}}\pgfpathlineto{\pgfqpoint{.5pt}{1pt}}\pgfusepathqstroke}},
  var backward diode IEC graphic/.style={backward diode IEC graphic,circuit symbol filled},
  breakdown diode IEC graphic/.style={circuit symbol open,circuit symbol size=width 2.5 height 1.25,transform shape,shape=breakdown diode IEC},
  var breakdown diode IEC graphic/.style={breakdown diode IEC graphic,circuit symbol filled},
  contact IEC graphic/.style={circuit symbol filled,circuit symbol size=width .5 height .5,shape=circle ee,outer sep=.25\pgflinewidth},
  bulb IEC graphic/.style={circuit symbol lines,circuit symbol size=width 1.5 height 1.5,shape=generic circle IEC,
    /pgf/generic circle IEC/before background={\pgfpathmoveto{\pgfpointpolar{-135}{1pt}}\pgfpathlineto{\pgfpointpolar{45}{1pt}}\pgfpathmoveto{\pgfpointpolar{-45}{1pt}}\pgfpathlineto{\pgfpointpolar{135}{1pt}}\pgfusepathqstroke},
    transform shape},
  current source IEC graphic/.style={circuit symbol lines,circuit symbol size=width 2 height 2,shape=generic circle IEC,
    /pgf/generic circle IEC/before background={\pgfpathmoveto{\pgfqpoint{0pt}{-1pt}}\pgfpathlineto{\pgfqpoint{0pt}{1pt}}\pgfusepathqstroke},
    transform shape},
  voltage source IEC graphic/.style={circuit symbol lines,circuit symbol size=width 2 height 2,shape=generic circle IEC,
    /pgf/generic circle IEC/before background={\pgfpathmoveto{\pgfqpoint{-1pt}{0pt}}\pgfpathlineto{\pgfqpoint{1pt}{0pt}}\pgfusepathqstroke},
    transform shape},
  make contact IEC graphic/.style={circuit symbol wires,circuit symbol size=width 2 height 1,transform shape,shape=make contact IEC,outer sep=0pt,cap=round},
  var make contact IEC graphic/.style={circuit symbol wires,circuit symbol size=width 2.4 height 1,transform shape,shape=var make contact IEC,outer sep=0pt,cap=round},
  break contact IEC graphic/.style={circuit symbol wires,circuit symbol size=width 2 height 1,transform shape,shape=break contact IEC,outer sep=0pt,cap=round},
  amperemeter graphic/.style={circuit symbol open,circuit symbol size=width 2 height 2,transform shape,shape=generic circle IEC,info=center:A},
  voltmeter graphic/.style={circuit symbol open,circuit symbol size=width 2 height 2,transform shape,shape=generic circle IEC,info=center:V},
  ohmmeter graphic/.style={circuit symbol open,circuit symbol size=width 2 height 2,transform shape,shape=generic circle IEC,info=center:$\Omega$},
  ac source graphic/.style={circuit symbol lines,circuit symbol size=width 2 height 2,shape=generic circle IEC,
    /pgf/generic circle IEC/before background={\pgfpathmoveto{\pgfqpoint{0.7pt}{0pt}}\pgfpathquadraticcurveto{\pgfqpoint{0.35pt}{-0.5pt}}{\pgfqpoint{0pt}{0pt}}\pgfpathquadraticcurveto{\pgfqpoint{-0.35pt}{0.5pt}}{\pgfqpoint{-0.7pt}{0pt}}\pgfusepathqstroke},
    transform shape},
  dc source graphic/.style={circuit symbol lines,circuit symbol size=width 2 height 2,shape=generic circle IEC,
    /pgf/generic circle IEC/before background={\pgfpathmoveto{\pgfqpoint{0.7pt}{0.2pt}}\pgfpathlineto{\pgfqpoint{-0.7pt}{0.2pt}}\pgfusepathqstroke\pgfpathmoveto{\pgfqpoint{0.7pt}{-0.2pt}}\pgfpathlineto{\pgfqpoint{-0.7pt}{-0.2pt}}\pgfusepathqstroke},
    transform shape},
}
"#;
