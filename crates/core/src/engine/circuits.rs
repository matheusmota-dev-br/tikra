//! TikZ's circuit libraries (`circuits`, `circuits.logic` with its US, IEC and CDH styles,
//! `circuits.ee` and `circuits.ee.IEC`): their keys, implemented here.
//!
//! A *symbol* (`resistor`, `and gate`, …) is a key that a circuit declares. On a node it gives the node
//! the symbol's *graphic* (a shape of [`super::shapes`] with its size in symbol units, chosen by `set
//! resistor graphic=…`). On a `to` it places the symbol as a node at its `pos` on the line, turned along
//! it, and the line stops at the node's border on both sides: the `to` path becomes a `markings`
//! decoration whose marks are the symbols, joined by `mark connection node`. Units (`ohm=5`) and
//! `info` are labels; annotations (`light emitting`) are short arrows drawn next to the symbol after
//! it. The styles that only gather options are generated from the tables below.

use crate::keys;
use crate::scene::{Point, Transform};
use crate::tex::{Cat, Token};
use crate::{Error, PT};

use super::{macro_tokens, tikra_tokens, Engine};

fn braced(v: Vec<Token>) -> Vec<Token> {
    let mut out = vec![Token::Char('{', Cat::Begin)];
    out.extend(v);
    out.push(Token::Char('}', Cat::End));
    out
}

/// `text` with `opts` spliced where `@@` is.
fn with_options(text: &str, opts: &[Token]) -> Vec<Token> {
    let (a, b) = text.split_once("@@").unwrap_or((text, ""));
    let mut v = tikra_tokens(a);
    v.extend(opts.iter().cloned());
    v.extend(tikra_tokens(b));
    v
}

fn arguments(e: &mut Engine, v: Option<Vec<Token>>, n: usize) -> Vec<Vec<Token>> {
    let _ = e;
    super::options::split_arguments(&v.unwrap_or_default(), n)
}

fn string(e: &mut Engine, v: Vec<Token>) -> Result<String, Error> {
    Ok(e.tex.expand_to_string(v)?.trim().to_string())
}

fn define(e: &mut Engine, name: &str, body: Vec<Token>) {
    e.tex.set_meaning(&Token::cs(name), macro_tokens(body), false);
}

/// The text of a macro without parameters.
fn macro_text(e: &Engine, name: &str) -> String {
    match e.tex.meaning_of(name) {
        crate::tex::Meaning::Macro(m) => crate::tex::tokens_to_string(&crate::tex::substitute(&m.body, &[])).trim().to_string(),
        _ => String::new(),
    }
}

/// Whether the `to` being read is a circuit's (set at its start by `circuit`).
fn on_to(e: &Engine) -> bool {
    macro_text(e, "tikra@circ@onto") == "1"
}

// ------------------------------------------------------------------------------------------ keys

/// `circuit`: every `to` of the scope may carry symbols; each starts with an ordinary line.
fn k_circuit(e: &mut Engine, _v: Option<Vec<Token>>) -> Result<(), Error> {
    define(e, "tikra@circ@onto", tikra_tokens("0"));
    e.process_keys(tikra_tokens("every circuit/.try, execute at begin to={\\tikra@circ@reset}"), Some("/tikz"))
}

/// The position of a symbol on its line: the last of its `pos`, `near start`… keys, `.5` without.
fn position(e: &mut Engine, opts: &[Token]) -> Result<f64, Error> {
    let mut pos = 0.5;
    for item in keys::split_list(opts) {
        pos = match item.key.as_str() {
            "pos" => match item.value {
                Some(v) if !keys::trim(&v).is_empty() => e.number_of(v)?,
                _ => continue,
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
    }
    Ok(pos)
}

/// `circuit handle symbol={options}`: on a circuit's `to`, a node with the options on the line;
/// elsewhere the options themselves.
fn k_handle_symbol(e: &mut Engine, v: Option<Vec<Token>>) -> Result<(), Error> {
    let opts = v.unwrap_or_default();
    let n = 1 + macro_text(e, "tikra@circ@n").parse::<u64>().unwrap_or(0);
    e.tex.define_string("tikra@circ@n", &n.to_string(), true);
    if !on_to(e) {
        return e.process_keys(opts, Some("/tikz"));
    }
    let pos = position(e, &opts)?;
    // the line from the start to the target, decorated by the symbols' marks
    let path = "\\pgfextra{\\def\\tikra@circ@onto{0}} decorate [decoration={markings,mark connection node=tikra@circ@mark}] {\\tikra@circ@first \\tikra@circ@last}";
    let mut kv = tikra_tokens("to path=");
    kv.extend(braced(tikra_tokens(path)));
    e.process_keys(kv, Some("/tikz"))?;
    if pos == 0.0 {
        // at the start: a node there, turned toward the target, from whose border the line goes
        let first = with_options("{\\pgfextra{\\pgfkeys{/tikz/circuits/tikra turn=start}} node[alias=tikra@circ@a,at={(0,0)},@@,pos=]{}} (tikra@circ@a)", &opts);
        define(e, "tikra@circ@first", first);
    } else if pos == 1.0 {
        let name = format!("tikra@circ@z{n}");
        let last =
            with_options(&format!("{{\\pgfextra{{\\pgfkeys{{/tikz/circuits/tikra turn=target}}}} node[alias={name},at={{(0,0)}},@@,pos=]{{}}}} -- ({name}) \\tikztonodes"), &opts);
        define(e, "tikra@circ@last", last);
    } else {
        let mark = with_options(&format!("decoration={{mark=at position {pos} with {{\\node[alias=tikra@circ@mark,@@,pos=]{{}};}}}}"), &opts);
        e.process_keys(mark, Some("/tikz"))?;
    }
    Ok(())
}

/// `circuits/tikra turn=start` (or `target`): the coordinate system at that end of the `to`, its x
/// axis along the line.
fn k_turn(e: &mut Engine, v: Option<Vec<Token>>) -> Result<(), Error> {
    let which = string(e, v.unwrap_or_default())?;
    let mut ends = [Point::default(); 2];
    for (k, m) in ["tikztostart", "tikztotarget"].iter().enumerate() {
        let name = e.tex.expand_to_string(vec![Token::cs(m)])?;
        ends[k] = e.coordinate(&name)?.p;
    }
    let d = ends[1].minus(ends[0]);
    let at = if which == "target" { ends[1] } else { ends[0] };
    let linear = Transform { e: 0.0, f: 0.0, ..e.st.transform };
    e.st.transform = Transform::translation(at.x, at.y).compose(&Transform::rotation(d.y.atan2(d.x).to_degrees())).compose(&linear);
    Ok(())
}

/// `circuit declare symbol=name`: the keys `name`, `name/graphic` and `set name graphic`.
fn k_declare_symbol(e: &mut Engine, v: Option<Vec<Token>>) -> Result<(), Error> {
    let name = string(e, v.unwrap_or_default())?;
    let text = format!(
        "{name}/.style={{circuit handle symbol={{inner sep=0.5pt,every circuit symbol,{name}/graphic,every {name}/.try,#1}}}},\
         {name}/graphic/.style={{}}, set {name} graphic/.style={{{name}/graphic/.style={{#1}}}}"
    );
    e.process_keys(tikra_tokens(&text), Some("/tikz"))
}

/// `circuit declare unit={name}{unit}`: `name=value` and its `'` and `sloped` forms, labels of the
/// value in that unit.
fn k_declare_unit(e: &mut Engine, v: Option<Vec<Token>>) -> Result<(), Error> {
    let a = arguments(e, v, 2);
    let name = string(e, a[0].clone())?;
    let unit = crate::tex::tokens_to_string(&a[1]);
    for (suffix, mode) in [("", "a"), (" sloped", "as"), ("'", "b"), ("' sloped", "bs")] {
        let text = format!("{name}{suffix}/.code={{\\pgfkeys{{/tikz/circuits/tikra label={{{name}}}{{{unit}}}{{{mode}}}{{#1}}}}}}");
        e.process_keys(tikra_tokens(&text), Some("/tikz"))?;
    }
    Ok(())
}

/// `circuits/tikra label={name}{unit}{mode}{[options]value}`: the label of a unit (`$\mathrm{5\Omega}$`)
/// or, without a name, of `info`; `mode` says below (`b`) and sloped (`s`).
fn k_label(e: &mut Engine, v: Option<Vec<Token>>) -> Result<(), Error> {
    let a = arguments(e, v, 4);
    let name = string(e, a[0].clone())?;
    let mode = string(e, a[2].clone())?;
    let mut value = keys::trim(&a[3]).to_vec();
    let mut opts = Vec::new();
    if value.first().is_some_and(|t| t.is_char('[')) {
        let mut depth = 0;
        if let Some(end) = value.iter().position(|t| {
            if t.is_char('[') {
                depth += 1;
            } else if t.is_char(']') {
                depth -= 1;
            }
            depth == 0
        }) {
            opts = value[1..end].to_vec();
            value = value[end + 1..].to_vec();
        }
    }
    let mut spec = tikra_tokens("[");
    if mode.starts_with('b') {
        spec.extend(tikra_tokens("label position=below,"));
    }
    spec.extend(tikra_tokens("every info/.try,"));
    if !name.is_empty() {
        spec.extend(tikra_tokens(&format!("every {name}/.try,")));
    }
    if mode.ends_with('s') {
        spec.extend(tikra_tokens("transform shape,"));
    }
    spec.extend(opts);
    spec.extend(tikra_tokens("]"));
    if name.is_empty() {
        spec.extend(value);
    } else {
        spec.extend(tikra_tokens("$\\mathrm{"));
        spec.extend(value);
        spec.extend(a[1].clone());
        spec.extend(tikra_tokens("}$"));
    }
    let mut kv = tikra_tokens("label=");
    kv.extend(braced(spec));
    e.process_keys(kv, Some("/tikz"))
}

/// `circuit declare annotation={name}{label distance}{path}`: `name` and `name'` draw the path after
/// the symbol, from its north (south, upside down), with `->` tips.
fn k_declare_annotation(e: &mut Engine, v: Option<Vec<Token>>) -> Result<(), Error> {
    let a = arguments(e, v, 3);
    let name = string(e, a[0].clone())?;
    define(e, &format!("tikra@circ@ann@{name}@d"), a[1].clone());
    define(e, &format!("tikra@circ@ann@{name}@p"), a[2].clone());
    for (suffix, side) in [("", "north"), ("'", "south")] {
        let text = format!("{name}{suffix}/.style={{after node path={{\\pgfextra{{\\pgfkeys{{/tikz/circuits/tikra annotate={{{name}}}{{{side}}}{{#1}}}}}}}}}}");
        e.process_keys(tikra_tokens(&text), Some("/tikz"))?;
    }
    Ok(())
}

/// `circuits/tikra annotate={name}{side}{options}`: an annotation of the last node, as a path of its
/// own, then the labels among its options on the node.
fn k_annotate(e: &mut Engine, v: Option<Vec<Token>>) -> Result<(), Error> {
    let a = arguments(e, v, 3);
    let name = string(e, a[0].clone())?;
    let side = string(e, a[1].clone())?;
    let body = |e: &Engine, which: &str| match e.tex.meaning_of(&format!("tikra@circ@ann@{name}@{which}")) {
        crate::tex::Meaning::Macro(m) => crate::tex::substitute(&m.body, &[]),
        _ => Vec::new(),
    };
    let (distance, path) = (body(e, "d"), body(e, "p"));
    let flip = if side == "south" { "yscale=-1," } else { "" };
    let mut t = tikra_tokens(&format!("\\path[every circuit annotation/.try,every {name}/.try,shift={{(\\tikzlastnode.{side})}},{flip}annotation arrow,->,"));
    t.extend(a[2].clone());
    t.extend(tikra_tokens("]"));
    t.extend(path);
    t.extend(tikra_tokens(";"));
    e.run_tokens(t)?;
    // TikZ puts the labels after the annotation, where `\tikzlastnode` is still the symbol
    let Some(node) = e.last_node.clone() else { return Ok(()) };
    e.begin_group();
    let saved = std::mem::take(&mut e.st.node.labels);
    let r = e.process_keys(a[2].clone(), Some("/tikz"));
    let labels = std::mem::replace(&mut e.st.node.labels, saved);
    e.end_group();
    r?;
    for (spec, pin) in labels {
        e.begin_group();
        let mut d = tikra_tokens("label distance=");
        d.extend(braced(distance.clone()));
        let r = e.process_keys(d, Some("/tikz")).and_then(|_| e.label(&node, spec, pin));
        e.end_group();
        e.canvas.items.extend(r?);
    }
    Ok(())
}

/// `circuit symbol unit=length`: `\tikzcircuitssizeunit`, which symbol sizes are counted in.
fn k_unit(e: &mut Engine, v: Option<Vec<Token>>) -> Result<(), Error> {
    let d = e.length_of(v.unwrap_or_default())? / PT;
    e.run_tokens(tikra_tokens(&format!("\\tikzcircuitssizeunit={d}pt\\relax")))
}

/// `circuit symbol size=width w height h`: the minimum size, in symbol units.
fn k_size(e: &mut Engine, v: Option<Vec<Token>>) -> Result<(), Error> {
    let s = crate::tex::tokens_to_string(&v.unwrap_or_default());
    let (w, h) = s.trim().strip_prefix("width").and_then(|r| r.split_once("height")).map(|(w, h)| (w.trim().to_string(), h.trim().to_string())).unwrap_or_default();
    let text = format!("minimum width={{({w})*\\tikzcircuitssizeunit}},minimum height={{({h})*\\tikzcircuitssizeunit}}");
    e.process_keys(tikra_tokens(&text), Some("/tikz"))
}

/// The tip `direction ee` of a given length, as `>`.
fn direction_tips(e: &mut Engine, length: f64) -> Result<(), Error> {
    let d = length / PT;
    e.tex.define_string(&keys::macro_name("/pgf/arrow options/direction ee"), &format!("{d}pt"), false);
    std::rc::Rc::make_mut(&mut e.st.tips).declare("direction ee", &format!("tikra direction ee[length={d}pt]"));
    e.process_keys(tikra_tokens(">=direction ee"), Some("/tikz"))
}

fn size_unit(e: &mut Engine) -> f64 {
    e.length("\\tikzcircuitssizeunit").unwrap_or(7.0 * PT)
}

/// `annotation arrow`: the tips of annotations, growing with the symbol unit and the line width.
fn k_annotation_arrow(e: &mut Engine, _v: Option<Vec<Token>>) -> Result<(), Error> {
    let l = 0.4 * size_unit(e) + 0.3 * e.st.line_width;
    direction_tips(e, l)
}

/// `current direction arrow`: the tip of the `current direction` symbols.
fn k_current_arrow(e: &mut Engine, _v: Option<Vec<Token>>) -> Result<(), Error> {
    let l = 1.3065 * (0.5 * size_unit(e) + 0.3 * e.st.line_width);
    direction_tips(e, l)?;
    e.process_keys(tikra_tokens("/pgf/direction ee arrow=direction ee"), Some("/tikz"))
}

/// `\pgfsetarrowoptions{direction ee}{length}`, as `circuits/tikra arrow options={name}{length}`.
fn k_arrow_options(e: &mut Engine, v: Option<Vec<Token>>) -> Result<(), Error> {
    let a = arguments(e, v, 2);
    if string(e, a[0].clone())? == "direction ee" {
        let l = e.length_of(a[1].clone())?;
        direction_tips(e, l)?;
    }
    Ok(())
}

/// The keys done natively.
pub const KEYS: &[(&str, super::options::KeyFn)] = &[
    ("/tikz/circuit", k_circuit),
    ("/tikz/circuit handle symbol", k_handle_symbol),
    ("/tikz/circuit declare symbol", k_declare_symbol),
    ("/tikz/circuit declare unit", k_declare_unit),
    ("/tikz/circuit declare annotation", k_declare_annotation),
    ("/tikz/circuit symbol unit", k_unit),
    ("/tikz/circuit symbol size", k_size),
    ("/tikz/annotation arrow", k_annotation_arrow),
    ("/tikz/current direction arrow", k_current_arrow),
    ("/tikz/circuits/tikra turn", k_turn),
    ("/tikz/circuits/tikra label", k_label),
    ("/tikz/circuits/tikra annotate", k_annotate),
    ("/tikz/circuits/tikra arrow options", k_arrow_options),
];

// ---------------------------------------------------------------------------------------- tables

/// How a symbol is drawn, as the `circuit symbol …` style of that name.
#[derive(Clone, Copy)]
enum Look {
    Open,
    Filled,
    Lines,
    Wires,
}

impl Look {
    fn style(self) -> &'static str {
        match self {
            Look::Open => "circuit symbol open",
            Look::Filled => "circuit symbol filled",
            Look::Lines => "circuit symbol lines",
            Look::Wires => "circuit symbol wires",
        }
    }
}

/// The logic gates, by the first word of their names.
const GATES: [&str; 8] = ["and", "nand", "or", "nor", "xor", "xnor", "not", "buffer"];

/// The IEC gates' symbols in a circuit (the shapes' own are larger).
const IEC_SYMBOLS: [&str; 8] = ["\\char`\\&", "\\char`\\&", "\\geq1", "\\geq1", "=1", "=1", "1", "1"];

/// The symbols of `circuits.ee`.
const EE_SYMBOLS: &[&str] = &[
    "resistor",
    "inductor",
    "capacitor",
    "contact",
    "ground",
    "battery",
    "diode",
    "Zener diode",
    "Schottky diode",
    "tunnel diode",
    "backward diode",
    "breakdown diode",
    "bulb",
    "current source",
    "voltage source",
    "current direction",
    "current direction'",
    "make contact",
    "break contact",
    "amperemeter",
    "voltmeter",
    "ohmmeter",
    "ac source",
    "dc source",
];

/// The units of `circuits.ee`: key and symbol.
const UNITS: &[(&str, &str)] =
    &[("ampere", "A"), ("volt", "V"), ("ohm", "\\Omega"), ("siemens", "S"), ("henry", "H"), ("farad", "F"), ("coulomb", "C"), ("voltampere", "VA"), ("watt", "W"), ("hertz", "Hz")];

/// The graphics of `circuits.ee.IEC`: name (without ` graphic`), look, width and height in symbol
/// units, shape, and the mark that a generic shape draws (see [`super::shapes`]'s `generic circle
/// IEC`), then other keys.
const EE_GRAPHICS: &[(&str, Look, f64, f64, &str, &str, &str)] = &[
    ("resistor IEC", Look::Open, 4.0, 1.0, "rectangle ee", "", "transform shape"),
    ("var resistor IEC", Look::Lines, 4.8, 0.8, "var resistor IEC", "", "transform shape,outer sep=0pt,line cap=round"),
    ("inductor IEC", Look::Lines, 4.0, 0.5, "inductor IEC", "", "transform shape,outer sep=0pt,line cap=round"),
    ("var inductor IEC", Look::Filled, 4.0, 1.0, "rectangle ee", "", "transform shape"),
    ("capacitor IEC", Look::Lines, 0.5, 2.0, "capacitor IEC", "", "transform shape"),
    ("ground IEC", Look::Lines, 0.75, 2.0, "ground IEC", "", "transform shape"),
    ("battery IEC", Look::Lines, 0.5, 2.5, "battery IEC", "", "transform shape"),
    ("diode IEC", Look::Open, 1.25, 1.25, "generic diode IEC", "0,-1 0,1", "transform shape"),
    ("Zener diode IEC", Look::Open, 1.25, 1.25, "generic diode IEC", "-.5,-1 0,-1 0,1", "transform shape"),
    ("Schottky diode IEC", Look::Open, 1.25, 1.25, "generic diode IEC", "-.4,-.6 -.4,-1 0,-1 0,1 .4,1 .4,.6", "transform shape"),
    ("tunnel diode IEC", Look::Open, 1.25, 1.25, "generic diode IEC", "-.5,-1 0,-1 0,1 -.5,1", "transform shape"),
    ("backward diode IEC", Look::Open, 1.25, 1.25, "generic diode IEC", "-.5,-1 .5,-1; 0,-1 0,1; -.5,1 .5,1", "transform shape"),
    ("breakdown diode IEC", Look::Open, 2.5, 1.25, "breakdown diode IEC", "", "transform shape"),
    ("contact IEC", Look::Filled, 0.5, 0.5, "circle ee", "", "outer sep=.25\\pgflinewidth"),
    ("bulb IEC", Look::Lines, 1.5, 1.5, "generic circle IEC", "-.7071,-.7071 .7071,.7071; .7071,-.7071 -.7071,.7071", "transform shape"),
    ("current source IEC", Look::Lines, 2.0, 2.0, "generic circle IEC", "0,-1 0,1", "transform shape"),
    ("voltage source IEC", Look::Lines, 2.0, 2.0, "generic circle IEC", "-1,0 1,0", "transform shape"),
    ("make contact IEC", Look::Wires, 2.0, 1.0, "make contact IEC", "", "transform shape,outer sep=0pt,line cap=round"),
    ("var make contact IEC", Look::Wires, 2.4, 1.0, "var make contact IEC", "", "transform shape,outer sep=0pt,line cap=round"),
    ("break contact IEC", Look::Wires, 2.0, 1.0, "break contact IEC", "", "transform shape,outer sep=0pt,line cap=round"),
    ("amperemeter", Look::Open, 2.0, 2.0, "generic circle IEC", "", "transform shape,info=center:A"),
    ("voltmeter", Look::Open, 2.0, 2.0, "generic circle IEC", "", "transform shape,info=center:V"),
    ("ohmmeter", Look::Open, 2.0, 2.0, "generic circle IEC", "", "transform shape,info=center:$\\Omega$"),
    ("ac source", Look::Lines, 2.0, 2.0, "generic circle IEC", "wave", "transform shape"),
    ("dc source", Look::Lines, 2.0, 2.0, "generic circle IEC", ".7,.2 -.7,.2; .7,-.2 -.7,-.2", "transform shape"),
];

/// The diodes that have a filled form, `var … IEC graphic`.
const FILLED_DIODES: [&str; 6] = ["diode", "Zener diode", "Schottky diode", "tunnel diode", "backward diode", "breakdown diode"];

/// A piece of an annotation, in symbol units from the symbol's north: a line between two points, or
/// from a point at an angle and length.
enum Stroke {
    Line((f64, f64), (f64, f64)),
    Ray((f64, f64), f64, f64),
}

/// The annotations of `circuits.ee`: name, label distance (symbol units), a shift of their origin
/// (an angle and length, or the node's center for `None`), and their strokes.
#[allow(clippy::type_complexity)]
const ANNOTATIONS: &[(&str, f64, Option<(f64, f64)>, &[Stroke])] = &[
    ("direction info", 0.5, Some((0.0, 0.0)), &[Stroke::Line((-1.25, 0.3333), (1.25, 0.3333))]),
    ("light emitting", 1.75, Some((0.0, 0.0)), &[Stroke::Ray((-0.2, 0.65), 45.0, 1.25), Stroke::Ray((0.2, 0.25), 45.0, 1.25)]),
    ("light dependent", 1.75, Some((135.0, 1.25)), &[Stroke::Ray((0.2, 0.65), -45.0, 1.25), Stroke::Ray((-0.2, 0.25), -45.0, 1.25)]),
    ("adjustable", 1.5, None, &[Stroke::Line((-1.5, -1.5), (1.5, 1.5))]),
    ("adjustable'", 1.5, None, &[Stroke::Line((-1.5, 1.5), (1.5, -1.5))]),
];

/// The libraries' styles, made from the tables, and the registers they use.
pub fn definitions() -> String {
    let mut s = String::new();
    s.push_str("\\newdimen\\tikzcircuitssizeunit\\tikzcircuitssizeunit=7pt\\relax\n");
    s.push_str("\\def\\tikra@circ@onto{0}\\let\\tikra@circ@first\\pgfutil@empty\\def\\tikra@circ@last{-- (\\tikztotarget) \\tikztonodes}\n");
    s.push_str("\\def\\tikra@circ@reset{\\def\\tikra@circ@onto{1}\\let\\tikra@circ@first\\pgfutil@empty\\def\\tikra@circ@last{-- (\\tikztotarget) \\tikztonodes}}\n");
    s.push_str("\\def\\pgfsetarrowoptions#1#2{\\pgfkeys{/tikz/circuits/tikra arrow options={{#1}{#2}}}}\n");
    let mut keys: Vec<String> = Vec::new();
    // the general part: sizes, looks, directions, labels
    for (name, pt) in [("huge", 10), ("large", 8), ("medium", 7), ("small", 6), ("tiny", 5)] {
        keys.push(format!("{name} circuit symbols/.style={{circuit symbol unit={pt}pt}}"));
    }
    keys.push("every circuit symbol/.style={}".into());
    for (look, keys_of) in [("open", "draw"), ("filled", "draw,fill=black"), ("lines", "draw"), ("wires", "draw")] {
        keys.push(format!("circuit symbol {look}/.style={{{keys_of}}}"));
    }
    for (dir, angle) in [("up", "rotate=90"), ("down", "rotate=-90"), ("left", "rotate=180"), ("right", "")] {
        keys.push(format!("point {dir}/.style={{{angle}}}"));
    }
    for (suffix, mode) in [("", "a"), (" sloped", "as"), ("'", "b"), ("' sloped", "bs")] {
        keys.push(format!("info{suffix}/.code={{\\pgfkeys{{/tikz/circuits/tikra label={{}}{{}}{{{mode}}}{{#1}}}}}}"));
    }
    // circuits.logic, with its three styles
    keys.push("circuit logic/.style={circuit,inputs/.style={logic gate inputs={##1}},logic gate inverted radius=.25\\tikzcircuitssizeunit,every circuit logic/.try}".into());
    for style in ["US", "IEC"] {
        let mut sets: Vec<String> = GATES.iter().map(|g| format!("set {g} gate graphic={g} gate {style} graphic")).collect();
        if style == "IEC" {
            sets.extend(GATES.iter().zip(IEC_SYMBOLS).map(|(g, sym)| format!("{g} gate IEC symbol={{$\\scriptstyle{sym}$}}")));
        }
        keys.push(format!("circuit logic {style}/.style={{circuit logic,{}}}", sets.join(",")));
    }
    keys.push("circuit logic CDH/.style={circuit logic US,set and gate graphic=and gate CDH graphic,set nand gate graphic=nand gate CDH graphic}".into());
    let us = "circuit symbol open,circuit symbol size=width 0 height 2.5,shape=##1,transform shape";
    let iec = "circuit symbol open,circuit symbol size=width 2.5 height 4,shape=##1,inner sep=.5ex";
    keys.push(format!("circuit logic US make graphic/.style={{#1 graphic/.style={{{}}}}}", us.replace("##", "#")));
    keys.push(format!("circuit logic IEC make graphic/.style={{#1 graphic/.style={{{}}}}}", iec.replace("##", "#")));
    for g in GATES {
        keys.push(format!("circuit declare symbol={g} gate"));
        keys.push(format!("{g} gate US graphic/.style={{{}}}", us.replace("##1", &format!("{g} gate US"))));
        keys.push(format!("{g} gate IEC graphic/.style={{{}}}", iec.replace("##1", &format!("{g} gate IEC"))));
    }
    for g in ["and", "nand"] {
        keys.push(format!("{g} gate CDH graphic/.style={{{}}}", us.replace("##1", &format!("{g} gate CDH"))));
    }
    // circuits.ee and its IEC graphics
    keys.push("circuit ee/.style={circuit,every circuit ee/.try}".into());
    for sym in EE_SYMBOLS {
        keys.push(format!("circuit declare symbol={sym}"));
    }
    keys.push("set current direction graphic=current direction ee graphic".into());
    keys.push("set current direction' graphic=current direction' ee graphic".into());
    keys.push("current direction ee graphic/.style={shape=direction ee,circuit symbol filled,current direction arrow,minimum width=.5*\\the\\tikzcircuitssizeunit+.3*\\the\\pgflinewidth,minimum height=.5*\\the\\tikzcircuitssizeunit+.3*\\the\\pgflinewidth,transform shape}".into());
    keys.push("current direction' ee graphic/.style={current direction ee graphic,rotate=180}".into());
    for (unit, symbol) in UNITS {
        keys.push(format!("circuit declare unit={{{unit}}}{{{symbol}}}"));
    }
    for (name, look, w, h, shape, mark, more) in EE_GRAPHICS {
        let mark = if shape.starts_with("generic") {
            let kind = shape.trim_start_matches("generic ");
            format!(",/pgf/generic {kind}/before background=,/pgf/generic {kind}/tikra mark={{{mark}}}")
        } else {
            String::new()
        };
        keys.push(format!("{name} graphic/.style={{{},circuit symbol size=width {w} height {h},shape={shape}{mark},{more}}}", look.style()));
    }
    for d in FILLED_DIODES {
        keys.push(format!("var {d} IEC graphic/.style={{{d} IEC graphic,circuit symbol filled}}"));
    }
    let mut ee: Vec<String> = EE_SYMBOLS.iter().filter(|s| !s.starts_with("current direction")).map(|s| format!("set {s} graphic={s} IEC graphic")).collect();
    for m in ["amperemeter", "voltmeter", "ohmmeter", "ac source", "dc source"] {
        let i = ee.iter().position(|x| x.starts_with(&format!("set {m} graphic"))).expect("in the table");
        ee[i] = format!("set {m} graphic={m} graphic");
    }
    keys.push(format!("circuit ee IEC/.style={{circuit ee,{}}}", ee.join(",")));
    let u = |x: f64| format!("{x}\\tikzcircuitssizeunit");
    for (name, distance, shift, strokes) in ANNOTATIONS {
        let mut path = match shift {
            None => "[shift=(\\tikzlastnode.center)]".to_string(),
            Some((_, l)) if *l == 0.0 => String::new(),
            Some((a, l)) => format!("[shift=({a}:{})]", u(*l)),
        };
        for st in strokes.iter() {
            path.push_str(&match st {
                Stroke::Line(a, b) => format!(" ({},{}) edge[line to] ({},{})", u(a.0), u(a.1), u(b.0), u(b.1)),
                Stroke::Ray(a, angle, l) => format!(" ({},{}) edge[line to] ++({angle}:{})", u(a.0), u(a.1), u(*l)),
            });
        }
        keys.push(format!("circuit declare annotation={{{name}}}{{{}}}{{{path}}}", u(*distance)));
    }
    s.push_str("\\tikzset{\n");
    for k in keys {
        s.push_str("  ");
        s.push_str(&k);
        s.push_str(",\n");
    }
    s.push_str("}\n");
    s
}
