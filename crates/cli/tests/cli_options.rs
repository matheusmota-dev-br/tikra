//! The command line's other outputs: Beamer's slides as files or as one SVG, the construction
//! animation as GIF and APNG, a pen drawing the figure, the strokes for a plotter, and what each of
//! them says when it cannot be written.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

const FIGURE: &str = r"\begin{tikzpicture} \draw[->] (0,0) -- (1,1); \fill[red] (2,0) circle (2pt); \end{tikzpicture}";
const SLIDES: &str = r"\begin{tikzpicture} \draw (0,0) -- (2,0); \only<2>{\draw (0,1) -- (2,1);} \end{tikzpicture}";

fn tikra(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tikra")).args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("the binary runs");
    if let Err(e) = child.stdin.take().unwrap().write_all(stdin.as_bytes()) {
        assert_eq!(e.kind(), std::io::ErrorKind::BrokenPipe, "{e}");
    }
    child.wait_with_output().unwrap()
}

/// A temporary directory for this test only.
fn folder(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("tikra-cli-options-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// A file in a folder that does not exist: nothing can be written there.
const NOWHERE: &str = "/path/that/does/not/exist/out";

mod slides {
    use super::*;

    /// `--slides` writes one SVG per slide, numbered after the output's name, and lists them.
    #[test]
    fn one_file_per_slide() {
        let d = folder("slides");
        let out = d.join("talk.svg");
        let o = tikra(&["--slides", "-o", out.to_str().unwrap()], SLIDES);
        assert!(o.status.success(), "{}", text(&o.stderr));
        let first = d.join("talk-1.svg");
        let second = d.join("talk-2.svg");
        assert_eq!(text(&o.stdout), format!("{}\n{}\n", first.display(), second.display()));
        let paths = |f: &PathBuf| std::fs::read_to_string(f).unwrap().split("</metadata>").nth(1).unwrap().matches("<path").count();
        assert_eq!((paths(&first), paths(&second)), (1, 2));
        std::fs::remove_dir_all(&d).unwrap();
    }

    /// `--click` joins the slides in one SVG that advances on a click.
    #[test]
    fn one_svg_that_advances_on_a_click() {
        let d = folder("click");
        let out = d.join("talk.svg");
        let o = tikra(&["--click", "-o", out.to_str().unwrap()], SLIDES);
        assert!(o.status.success(), "{}", text(&o.stderr));
        assert_eq!(text(&o.stdout), format!("{}\n", out.display()));
        let svg = std::fs::read_to_string(&out).unwrap();
        assert!(svg.contains(r#"data-slides="2""#) && svg.contains("tr-slide-2") && svg.contains("trNext1.click"), "{svg}");
        std::fs::remove_dir_all(&d).unwrap();
    }

    /// Without `-o` the slides are named after `slides.svg`, in the current directory.
    #[test]
    fn named_after_slides_svg_by_default() {
        let d = folder("default");
        let o = Command::new(env!("CARGO_BIN_EXE_tikra")).arg("--slides").arg(d.join("in.tex")).current_dir(&d).output();
        // the input file does not exist yet: an error, nothing written
        assert_eq!(o.unwrap().status.code(), Some(1));
        std::fs::write(d.join("in.tex"), SLIDES).unwrap();
        let o = Command::new(env!("CARGO_BIN_EXE_tikra")).arg("--slides").arg(d.join("in.tex")).current_dir(&d).output().unwrap();
        assert!(o.status.success(), "{}", text(&o.stderr));
        assert_eq!(text(&o.stdout), "slides-1.svg\nslides-2.svg\n");
        assert!(d.join("slides-2.svg").exists());
        std::fs::remove_dir_all(&d).unwrap();
    }

    /// Input with no figure, and a folder that does not exist, are errors.
    #[test]
    fn errors() {
        let o = tikra(&["--slides"], "no figure here");
        assert_eq!(o.status.code(), Some(1));
        assert!(text(&o.stderr).contains("no tikzpicture"), "{}", text(&o.stderr));
        let o = tikra(&["--click", "-o", &format!("{NOWHERE}.svg")], SLIDES);
        assert_eq!(o.status.code(), Some(1));
        assert!(text(&o.stderr).contains("could not write"));
    }
}

mod animation {
    use super::*;

    /// An output named `.gif` or `.apng` is the construction animation, as those bytes.
    #[test]
    fn gif_and_apng_by_their_names() {
        let d = folder("animation");
        let gif = d.join("a.gif");
        let o = tikra(&["-o", gif.to_str().unwrap(), "--width", "100", "--fps", "5"], FIGURE);
        assert!(o.status.success(), "{}", text(&o.stderr));
        let bytes = std::fs::read(&gif).unwrap();
        assert!(bytes.starts_with(b"GIF89a"));
        assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), 100);
        let apng = d.join("a.apng");
        let o = tikra(&["-o", apng.to_str().unwrap(), "--width", "100", "--fps", "5"], FIGURE);
        assert!(o.status.success(), "{}", text(&o.stderr));
        let bytes = std::fs::read(&apng).unwrap();
        assert!(bytes.starts_with(b"\x89PNG") && bytes.windows(4).any(|w| w == b"acTL"));
        std::fs::remove_dir_all(&d).unwrap();
    }

    /// No frames per second is refused; a file that cannot be written is said so.
    #[test]
    fn errors() {
        let o = tikra(&["-o", "/tmp/never.gif", "--fps", "0"], FIGURE);
        assert_eq!(o.status.code(), Some(1));
        assert!(text(&o.stderr).contains("must be over 0"), "{}", text(&o.stderr));
        let o = tikra(&["-o", &format!("{NOWHERE}.apng"), "--width", "40", "--fps", "2"], FIGURE);
        assert_eq!(o.status.code(), Some(1));
        assert!(text(&o.stderr).contains("could not write"));
    }
}

mod pen {
    use super::*;

    /// `--pen` to `.svg`: an SVG where a pen draws the strokes; to `.gif` and `.apng`, its frames.
    #[test]
    fn a_pen_drawing_the_figure() {
        let d = folder("pen");
        let svg = d.join("p.svg");
        let o = tikra(&["--pen", "--seconds", "3", "-o", svg.to_str().unwrap()], FIGURE);
        assert!(o.status.success(), "{}", text(&o.stderr));
        let s = std::fs::read_to_string(&svg).unwrap();
        assert!(s.starts_with("<svg") && s.contains("#C8102E"), "{s}");
        assert!(s.contains("3s") || s.contains("dur=\"4.6s\""), "{s}");
        let gif = d.join("p.gif");
        let o = tikra(&["--pen", "--seconds", "1", "--fps", "4", "--width", "80", "-o", gif.to_str().unwrap()], FIGURE);
        assert!(o.status.success(), "{}", text(&o.stderr));
        let bytes = std::fs::read(&gif).unwrap();
        assert!(bytes.starts_with(b"GIF89a"));
        assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), 80);
        let apng = d.join("p.apng");
        let o = tikra(&["--pen", "--seconds", "1", "--fps", "4", "--width", "80", "-o", apng.to_str().unwrap()], FIGURE);
        assert!(o.status.success(), "{}", text(&o.stderr));
        let bytes = std::fs::read(&apng).unwrap();
        let at = bytes.windows(4).position(|w| w == b"acTL").unwrap();
        // four frames a second for a second, the blank page and the finished one included
        assert_eq!(u32::from_be_bytes(bytes[at + 4..at + 8].try_into().unwrap()), 5);
        std::fs::remove_dir_all(&d).unwrap();
    }

    /// No frames per second is refused; a file that cannot be written is said so.
    #[test]
    fn errors() {
        let o = tikra(&["--pen", "--fps", "0", "-o", "/tmp/never.gif"], FIGURE);
        assert_eq!(o.status.code(), Some(1));
        assert!(text(&o.stderr).contains("must be over 0"), "{}", text(&o.stderr));
        let o = tikra(&["--pen", "-o", &format!("{NOWHERE}.svg")], FIGURE);
        assert_eq!(o.status.code(), Some(1));
        assert!(text(&o.stderr).contains("could not write"));
    }

    /// `.gcode` is G-code with the pen's own codes; `.hpgl` and `.plt` are HPGL; `--paper` centers the
    /// figure on the sheet and `--keep-order` keeps TikZ's order.
    #[test]
    fn strokes_for_a_plotter() {
        let d = folder("plotter");
        let g = d.join("f.gcode");
        let o = tikra(&["-o", g.to_str().unwrap(), "--pen-up", "M3 S0", "--pen-down", "M3 S90", "--keep-order"], FIGURE);
        assert!(o.status.success(), "{}", text(&o.stderr));
        let code = std::fs::read_to_string(&g).unwrap();
        assert!(code.starts_with("G21\nG90\nM3 S0\n") && code.contains("\nM3 S90\n") && code.ends_with("G0 X0 Y0\n"), "{code}");
        let (centered, plain) = (d.join("c.hpgl"), d.join("p.plt"));
        let o = tikra(&["-o", centered.to_str().unwrap(), "--paper", "210x297"], FIGURE);
        assert!(o.status.success(), "{}", text(&o.stderr));
        let o = tikra(&["-o", plain.to_str().unwrap(), "--paper", "a4"], FIGURE);
        assert!(o.status.success(), "{}", text(&o.stderr));
        let (centered, plain) = (std::fs::read_to_string(&centered).unwrap(), std::fs::read_to_string(&plain).unwrap());
        assert!(centered.starts_with("IN;SP1;\n") && plain.starts_with("IN;SP1;\n"));
        // the first pen-up move: on A4 the figure starts away from the corner
        let first = |s: &str| s.lines().nth(1).unwrap().split(';').next().unwrap().trim_start_matches("PU").split(',').map(|v| v.parse::<i64>().unwrap()).collect::<Vec<_>>();
        assert!(first(&centered)[0] > first(&plain)[0] + 1000, "{centered}\n{plain}");
        let o = tikra(&["-o", &format!("{NOWHERE}.gcode")], FIGURE);
        assert_eq!(o.status.code(), Some(1));
        assert!(text(&o.stderr).contains("could not write"));
        std::fs::remove_dir_all(&d).unwrap();
    }
}
