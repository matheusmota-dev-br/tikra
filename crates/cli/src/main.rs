use std::process::ExitCode;
use std::{env, fs, io};

const HELP: &str = "\
tikra — TikZ figures to SVG and PNG, no TeX install

USAGE
  tikra <file.tex> [-o out.svg|out.png] [options]
  cat figure.tex | tikra -o figure.png

OPTIONS
  -o, --output <file>     .svg or .png (by its name); without -o, prints the SVG
      --dpi <n>           PNG resolution (default 300)
      --transparent       PNG without the white background
      --warnings          lists what was ignored or approximated
  -V, --version
  -h, --help
";

fn main() -> ExitCode {
    let mut input: Option<String> = None;
    let mut output: Option<String> = None;
    let mut dpi = 300.0f32;
    let mut background = true;
    let mut show_warnings = false;
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
            "--transparent" => background = false,
            "--warnings" => show_warnings = true,
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
    let r = match tikra::convert(&code) {
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
    match output {
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
        None if !show_warnings => println!("{}", r.svg),
        None => {}
    }
    ExitCode::SUCCESS
}
