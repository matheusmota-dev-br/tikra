mod pen;

use std::process::ExitCode;
use std::{env, fs, io};

const HELP: &str = "\
tikra — TikZ figures to SVG and PNG, no TeX install

USAGE
  tikra <file.tex> [-o out.svg|out.png|out.gif|out.apng|out.gcode|out.hpgl] [options]
  cat figure.tex | tikra -o figure.png

OPTIONS
  -o, --output <file>     .svg, .png, or the animation as .gif or .apng (by its name); without
                          -o, prints the SVG
                          .gcode, .hpgl or .plt are strokes for a pen plotter
      --keep-order        plotter: TikZ's order, not the one that lifts the pen least
      --paper <w>x<h>     plotter: the paper in mm; centers the figure, and shrinks it to fit
      --pen-up <code>     G-code that lifts the pen (default G0 Z5)
      --pen-down <code>   G-code that lowers the pen (default G0 Z0)
      --width <n>         GIF and APNG width in pixels (default 560)
      --fps <n>           GIF and APNG frames per second (default 15)
      --dpi <n>           PNG resolution (default 300)
      --transparent       PNG without the white background
      --warnings          lists what was ignored or approximated
      --describe          prints the figure in words: its alt text and its description
      --animate           the SVG draws itself in the browser, in the order TikZ drew it
      --steps             each element numbered by its step (data-step), with no animation
      --source            and with the place in the code that drew it (data-src)
      --slides            Beamer's overlays: one file per slide, out-1.svg, out-2.svg…
      --click             Beamer's overlays: one SVG that advances on a click
      --play              the animations library played in a browser, not a still picture
      --parts             each thing drawn a group a screen reader and the keyboard can reach
  -l, --locale <code>     the language of the figure in words: en (default), pt-BR, es, fr, de,
                          it, ru, zh, ja, ko, hi, ar, id, tr, nl, pl (the drawing is the same)
  -V, --version
  -h, --help
";

fn main() -> ExitCode {
    let mut input: Option<String> = None;
    let mut output: Option<String> = None;
    let mut dpi = 300.0f32;
    let mut background = true;
    let mut show_warnings = false;
    let mut describe = false;
    let mut options = tikra::Options::default();
    let mut motion = tikra::motion::Motion::default();
    let (mut slides, mut click) = (false, false);
    let mut pen = pen::Pen::default();
    let mut it = env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "-h" | "--help" => {
                print!("{HELP}");
                return ExitCode::SUCCESS;
            }
            "-V" | "--version" => {
                println!("tikra {}", tikra::VERSION);
                return ExitCode::SUCCESS;
            }
            "-o" | "--output" => output = it.next(),
            "--dpi" => dpi = it.next().and_then(|v| v.parse().ok()).unwrap_or(300.0),
            "--width" => motion.width = it.next().and_then(|v| v.parse().ok()).unwrap_or(motion.width),
            "--fps" => motion.fps = it.next().and_then(|v| v.parse().ok()).unwrap_or(motion.fps),
            "--transparent" => background = false,
            "--warnings" => show_warnings = true,
            "--describe" => describe = true,
            "--animate" => options = options.animated(),
            "--steps" => options = options.steps(),
            "--source" => options = options.source(),
            "--parts" => options = options.parts(),
            "--play" => options = options.play(),
            "--keep-order" => pen.keep_order = true,
            "--paper" => pen.paper = it.next().and_then(|v| v.split_once('x').and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)))),
            "--pen-up" => pen.up = it.next().unwrap_or(pen.up),
            "--pen-down" => pen.down = it.next().unwrap_or(pen.down),
            "--slides" => slides = true,
            "--click" => click = true,
            "-l" | "--locale" => match it.next().unwrap_or_default().parse() {
                Ok(l) => options.locale = l,
                Err(e) => {
                    eprintln!("tikra: {e}");
                    return ExitCode::from(2);
                }
            },
            _ => input = Some(a),
        }
    }
    let code = match &input {
        Some(c) => match fs::read_to_string(c) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("tikra: could not read {c}: {e}");
                return ExitCode::from(1);
            }
        },
        None => match io::read_to_string(io::stdin()) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("tikra: could not read standard input: {e}");
                return ExitCode::from(1);
            }
        },
    };
    if slides || click {
        return write_slides(&code, &options, output.as_deref(), click);
    }
    let moving = |s: &str, ext: &str| s.len() > ext.len() && s.ends_with(ext);
    let animation = output.as_deref().is_some_and(|s| moving(s, ".gif") || moving(s, ".apng"));
    if animation {
        options = options.animated();
    }
    let r = match tikra::convert_with(&code, &options) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("tikra: {e}");
            return ExitCode::from(1);
        }
    };
    for w in &r.warnings {
        if show_warnings {
            println!("warning: {w}");
        } else {
            eprintln!("tikra: warning: {w}");
        }
    }
    if describe {
        println!("alt: {}", r.alt);
        println!("description: {}", r.description);
    }
    match output {
        Some(s) if s.ends_with(".gcode") || s.ends_with(".hpgl") || s.ends_with(".plt") => {
            let text = if s.ends_with(".gcode") { pen::gcode(&r.svg, &pen) } else { pen::hpgl(&r.svg, &pen) };
            match text.map(|t| fs::write(&s, t)) {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    eprintln!("tikra: could not write {s}: {e}");
                    return ExitCode::from(1);
                }
                Err(e) => {
                    eprintln!("tikra: {e}");
                    return ExitCode::from(1);
                }
            }
        }
        Some(s) if animation => {
            let bytes = if s.ends_with(".gif") { tikra::motion::gif(&r.svg, motion) } else { tikra::motion::apng(&r.svg, motion) };
            match bytes.map(|b| fs::write(&s, b)) {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    eprintln!("tikra: could not write {s}: {e}");
                    return ExitCode::from(1);
                }
                Err(e) => {
                    eprintln!("tikra: {e}");
                    return ExitCode::from(1);
                }
            }
        }
        Some(s) if s.ends_with(".png") => match tikra::png(&r.svg, dpi, background) {
            Ok(b) => {
                if let Err(e) = fs::write(&s, b) {
                    eprintln!("tikra: could not write {s}: {e}");
                    return ExitCode::from(1);
                }
            }
            Err(e) => {
                eprintln!("tikra: {e}");
                return ExitCode::from(1);
            }
        },
        Some(s) => {
            if let Err(e) = fs::write(&s, &r.svg) {
                eprintln!("tikra: could not write {s}: {e}");
                return ExitCode::from(1);
            }
        }
        None if !show_warnings && !describe => println!("{}", r.svg),
        None => {}
    }
    ExitCode::SUCCESS
}

/// The slides of a figure with overlays, as files or as one SVG that advances on a click.
fn write_slides(code: &str, options: &tikra::Options, output: Option<&str>, click: bool) -> ExitCode {
    let slides = match tikra::slides(code, options) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("tikra: {e}");
            return ExitCode::from(1);
        }
    };
    let mut files: Vec<(String, String)> = Vec::new();
    let out = output.unwrap_or("slides.svg");
    if click {
        let Some(svg) = tikra::click_through(&slides) else {
            eprintln!("tikra: could not join the slides");
            return ExitCode::from(1);
        };
        files.push((out.to_string(), svg));
    } else {
        let stem = out.strip_suffix(".svg").unwrap_or(out);
        files.extend(slides.iter().enumerate().map(|(k, s)| (format!("{stem}-{}.svg", k + 1), s.svg.clone())));
    }
    for (name, svg) in files {
        if let Err(e) = fs::write(&name, svg) {
            eprintln!("tikra: could not write {name}: {e}");
            return ExitCode::from(1);
        }
        println!("{name}");
    }
    ExitCode::SUCCESS
}
