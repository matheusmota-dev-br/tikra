//! The fillbetween library: `\tikzfillbetween` and `\addplot fill between` (the area between two
//! named paths, split where they cross, soft clipped), `intersection segments`, and the commands
//! on soft paths (`\tikzgetnamedpath`, `\pgfcomputeintersectionsegments`, `\pgfsetpathandBB` …).

mod common;
use common::*;
use tikra::canvas::{Canvas, Paint};
use tikra::color::Color;

/// The filled paths of a canvas: their points (cm) and fill color.
fn filled(c: &Canvas) -> Vec<(Vec<(f64, f64)>, Color)> {
    paths(c)
        .into_iter()
        .filter_map(|d| {
            let f = d.fill?;
            let Paint::Solid(color) = f.paint else { return None };
            Some((points(&d.segments), color))
        })
        .collect()
}

/// Whether two point lists are the same, to 1e-6 cm.
fn same(a: &[(f64, f64)], b: &[(f64, f64)]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(p, q)| (p.0 - q.0).abs() < 1e-6 && (p.1 - q.1).abs() < 1e-6)
}

/// Two horizontal lines, one unit apart, named `A` (below) and `B` (above).
const LINES: &str = r"\path[name path=A] (0,0) -- (2,0); \path[name path=B] (0,1) -- (2,1);";

/// Two lines crossing at (1,1): `A` rising, `B` falling.
const CROSS: &str = r"\path[name path=A] (0,0) -- (2,2); \path[name path=B] (0,2) -- (2,0);";

/// The area is the first path, then the second one backwards when both run the same way.
#[test]
fn the_area_is_the_first_path_then_the_second_backwards() {
    let (c, w) = canvas_and_warnings(&format!(r"{LINES} \tikzfillbetween[of=A and B]{{red}};"));
    assert!(w.is_empty(), "{w:?}");
    let f = filled(&c);
    assert_eq!(f.len(), 1);
    assert!(same(&f[0].0, &[(0.0, 0.0), (2.0, 0.0), (2.0, 1.0), (0.0, 1.0)]), "{:?}", f[0].0);
    assert_eq!(f[0].1, Color::rgb(1.0, 0.0, 0.0));
}

/// A second path that runs the other way is followed forwards; `reverse` forces the walk.
#[test]
fn reverse_follows_the_directions_or_the_key() {
    let back = r"\path[name path=A] (0,0) -- (2,0); \path[name path=B] (2,1) -- (0,1);";
    let cases = [
        // auto: opposite directions, B taken as it is
        (back, "", vec![(0.0, 0.0), (2.0, 0.0), (2.0, 1.0), (0.0, 1.0)]),
        (back, ", reverse=auto", vec![(0.0, 0.0), (2.0, 0.0), (2.0, 1.0), (0.0, 1.0)]),
        // forced: B walked backwards, the area crosses itself
        (back, ", reverse", vec![(0.0, 0.0), (2.0, 0.0), (0.0, 1.0), (2.0, 1.0)]),
        (LINES, ", reverse=false", vec![(0.0, 0.0), (2.0, 0.0), (0.0, 1.0), (2.0, 1.0)]),
    ];
    for (paths, key, want) in cases {
        let f = filled(&canvas(&format!(r"{paths} \tikzfillbetween[of=A and B{key}]{{blue}};")));
        assert!(same(&f[0].0, &want), "{key}: {:?}", f[0].0);
    }
    // a vertical path's direction is upwards
    let up = r"\path[name path=A] (0,0) -- (0,2); \path[name path=B] (1,0) -- (1,2);";
    let f = filled(&canvas(&format!(r"{up} \tikzfillbetween[of=A and B]{{blue}};")));
    assert!(same(&f[0].0, &[(0.0, 0.0), (0.0, 2.0), (1.0, 2.0), (1.0, 0.0)]), "{:?}", f[0].0);
}

/// A move inside a named path becomes a line; a move at its end is dropped.
#[test]
fn inner_moves_are_connected() {
    let body = r"\path[name path=A] (0,0) -- (1,0) (1.5,0) -- (2,0) (5,5); \path[name path=B] (0,1) -- (2,1);
        \tikzfillbetween[of=A and B]{red};";
    let f = filled(&canvas(body));
    assert!(same(&f[0].0, &[(0.0, 0.0), (1.0, 0.0), (1.5, 0.0), (2.0, 0.0), (2.0, 1.0), (0.0, 1.0)]), "{:?}", f[0].0);
}

/// Without `of=A and B`, or with a name no path has, nothing is filled and a warning says why.
#[test]
fn missing_paths_are_warned() {
    let (c, w) = canvas_and_warnings(&format!(r"{LINES} \tikzfillbetween[of=A]{{red}};"));
    assert!(filled(&c).is_empty());
    assert!(w.iter().any(|a| a.contains("'of=A and B' missing")), "{w:?}");
    let (c, w) = canvas_and_warnings(&format!(r"{LINES} \tikzfillbetween[of=A and C]{{red}};"));
    assert!(filled(&c).is_empty());
    assert!(w.iter().any(|a| a.contains("no path named 'A' or 'C'")), "{w:?}");
}

/// `split` cuts the area where the paths cross: each piece in its own styles.
#[test]
fn split_fills_each_piece_in_its_style() {
    let body = format!(r"{CROSS} \tikzfillbetween[of=A and B, split, every segment no 0/.style={{green}}, every odd segment/.style={{black}}]{{red}};");
    let (c, w) = canvas_and_warnings(&body);
    assert!(w.is_empty(), "{w:?}");
    let f = filled(&c);
    assert_eq!(f.len(), 2);
    // left: A to the crossing, B back from it; right: from the crossing on
    assert!(same(&f[0].0, &[(0.0, 0.0), (1.0, 1.0), (1.0, 1.0), (0.0, 2.0)]), "{:?}", f[0].0);
    assert!(same(&f[1].0, &[(1.0, 1.0), (2.0, 2.0), (2.0, 0.0), (1.0, 1.0)]), "{:?}", f[1].0);
    assert_eq!(f[0].1, Color::rgb(0.0, 1.0, 0.0));
    assert_eq!(f[1].1, Color::rgb(0.0, 0.0, 0.0));
    // every last segment, every even segment
    let body = format!(r"{CROSS} \tikzfillbetween[of=A and B, split, every even segment/.style={{blue}}, every last segment/.style={{black}}]{{red}};");
    let f = filled(&canvas(&body));
    assert_eq!(f[0].1, Color::rgb(0.0, 0.0, 1.0));
    assert_eq!(f[1].1, Color::rgb(0.0, 0.0, 0.0));
    // `split=false` is one area
    assert_eq!(filled(&canvas(&format!(r"{CROSS} \tikzfillbetween[of=A and B, split=false]{{red}};"))).len(), 1);
}

/// Paths that touch where they start leave an empty piece there: it is not filled.
#[test]
fn split_skips_empty_pieces() {
    let body = r"\path[name path=A] (0,0) -- (2,2); \path[name path=B] (0,0) -- (2,1); \tikzfillbetween[of=A and B, split]{red};";
    let f = filled(&canvas(body));
    assert_eq!(f.len(), 1, "{f:?}");
    assert!(same(&f[0].0, &[(0.0, 0.0), (2.0, 2.0), (2.0, 1.0), (0.0, 0.0)]), "{:?}", f[0].0);
}

/// Curves are flattened when the paths are cut; a closed path comes back to its start.
#[test]
fn split_flattens_curves_and_closes() {
    let body = r"\path[name path=A] (0,0) .. controls (1,0) and (1,2) .. (2,2); \path[name path=B] (0,2) -- (2,0);
        \tikzfillbetween[of=A and B, split]{red};";
    let f = filled(&canvas(body));
    assert_eq!(f.len(), 2);
    // the first piece follows the curve in 32 steps per curve, up to the crossing at (1,1)
    assert!(f[0].0.len() > 10, "{:?}", f[0].0);
    assert!(f[0].0.iter().any(|p| close_to(*p, (1.0, 1.0))), "{:?}", f[0].0);
    let body = r"\path[name path=A] (0,0) -- (2,0) -- (2,2) -- cycle; \path[name path=B] (0,1.5) -- (3,1.5);
        \tikzfillbetween[of=A and B, split]{red};";
    let f = filled(&canvas(body));
    // the triangle is crossed twice: three pieces, the last back to (0,0)
    assert_eq!(f.len(), 3, "{f:?}");
    assert!(f[2].0.contains(&(0.0, 0.0)), "{:?}", f[2].0);
}

/// A path that is a single point fills nothing when split, and is filled whole when soft clipped.
#[test]
fn a_point_path() {
    let body = r"\path[name path=A] (1,1) -- (1,1); \path[name path=B] (0,0) -- (2,0);";
    assert!(filled(&canvas(&format!(r"{body} \tikzfillbetween[of=A and B, split]{{red}};"))).is_empty());
    let f = filled(&canvas(&format!(r"{body} \tikzfillbetween[of=A and B, soft clip={{(0,0) rectangle (0.5,2)}}]{{red}};")));
    assert_eq!(f.len(), 1);
    // the point's direction is not `+`, so B is taken forwards
    assert!(same(&f[0].0, &[(1.0, 1.0), (1.0, 1.0), (0.0, 0.0), (2.0, 0.0)]), "{:?}", f[0].0);
}

/// A closed path back to its start is a point too: the area is the other path closed through it.
#[test]
fn a_closed_point_path() {
    let f = filled(&canvas(r"\path[name path=A] (0,0) -- cycle; \path[name path=B] (0,1) -- (2,1); \tikzfillbetween[of=A and B]{red};"));
    assert!(same(&f[0].0, &[(0.0, 0.0), (2.0, 1.0), (0.0, 1.0)]), "{:?}", f[0].0);
}

/// `soft clip` cuts the area to a rectangle; `soft clip first` and `second` cut one path.
#[test]
fn soft_clip_cuts_to_a_rectangle() {
    let (c, w) = canvas_and_warnings(&format!(r"{LINES} \tikzfillbetween[of=A and B, soft clip={{(0.5,-1) rectangle (1.5,3)}}]{{red}};"));
    assert!(w.is_empty(), "{w:?}");
    let f = filled(&c);
    assert_eq!(f.len(), 1);
    let xs: Vec<f64> = f[0].0.iter().map(|p| p.0).collect();
    assert!(xs.iter().all(|x| (0.5 - 1e-9..=1.5 + 1e-9).contains(x)), "{xs:?}");
    assert!(xs.iter().any(|x| close(*x, 0.5)) && xs.iter().any(|x| close(*x, 1.5)), "{xs:?}");
    // a rectangle off the area leaves nothing
    assert!(filled(&canvas(&format!(r"{LINES} \tikzfillbetween[of=A and B, soft clip={{(5,5) rectangle (6,6)}}]{{red}};"))).is_empty());
    // only A cut to x in [0.5,1.5]: A's piece, then B backwards
    let f = filled(&canvas(&format!(r"{LINES} \tikzfillbetween[of=A and B, soft clip first={{(0.5,-1) rectangle (1.5,3)}}]{{red}};")));
    assert!(same(&f[0].0, &[(0.5, 0.0), (1.5, 0.0), (2.0, 1.0), (0.0, 1.0)]), "{:?}", f[0].0);
    let f = filled(&canvas(&format!(r"{LINES} \tikzfillbetween[of=A and B, soft clip second={{(0.5,-1) rectangle (1.5,3)}}]{{red}};")));
    assert!(same(&f[0].0, &[(0.0, 0.0), (2.0, 0.0), (1.5, 1.0), (0.5, 1.0)]), "{:?}", f[0].0);
}

/// A soft clip that is neither a rectangle nor a domain is ignored, with a warning.
#[test]
fn soft_clip_not_understood() {
    for spec in ["{circle}", "{(0,0) rectangle 1}", "{domain=1:2}", "{range=1:2}", "{domain=1}"] {
        let (c, w) = canvas_and_warnings(&format!(r"{LINES} \tikzfillbetween[of=A and B, soft clip={spec}]{{red}};"));
        assert!(w.iter().any(|a| a.contains("not understood")), "{spec}: {w:?}");
        assert_eq!(filled(&c).len(), 1, "{spec}");
    }
}

/// A soft clip rectangle cuts a path's lines where they leave it and where they come back.
#[test]
fn soft_clip_cuts_lines_in_and_out() {
    // a zigzag in and out of the band y in [0,1]
    let body = r"\path[name path=A] (0,0.5) -- (1,0.5) -- (1,3) -- (2,3) -- (2,0.5) -- (3,0.5); \path[name path=B] (0,-1) -- (3,-1);
        \tikzfillbetween[of=A and B, soft clip first={(-1,0) rectangle (4,1)}]{red};";
    let f = filled(&canvas(body));
    let pts = &f[0].0;
    // the vertical lines are cut at y=1; the horizontal one above is dropped
    assert!(same(&pts[..6], &[(0.0, 0.5), (1.0, 0.5), (1.0, 1.0), (2.0, 1.0), (2.0, 0.5), (3.0, 0.5)]), "{pts:?}");
}

/// `\addplot fill between` in an axis: soft clip by `domain` (x) and `domain y`.
#[test]
fn addplot_fill_between_with_domains() {
    let axis = |opts: &str| {
        let body = format!(
            r"\begin{{axis}}[xmin=0, xmax=4, ymin=0, ymax=4, width=6cm, height=6cm, scale only axis]
            \addplot[name path=A, draw=none] coordinates {{(0,1) (4,1)}};
            \addplot[name path=B, draw=none] coordinates {{(0,3) (4,3)}};
            \addplot[red] fill between[of=A and B{opts}];
            \end{{axis}}"
        );
        let (c, w) = canvas_and_warnings(&body);
        assert!(w.is_empty(), "{opts}: {w:?}");
        let f = filled(&c);
        assert_eq!(f.len(), 1, "{opts}");
        let (xs, ys): (Vec<f64>, Vec<f64>) = f[0].0.iter().copied().unzip();
        let span = |v: &[f64]| v.iter().cloned().fold(f64::NEG_INFINITY, f64::max) - v.iter().cloned().fold(f64::INFINITY, f64::min);
        (span(&xs), span(&ys))
    };
    // 6cm for 4 units: 1.5cm a unit
    let (w, h) = axis("");
    assert!(close(w, 6.0) && close(h, 3.0), "{w} {h}");
    let (w, h) = axis(", soft clip={domain=1:3}");
    assert!(close(w, 3.0) && close(h, 3.0), "{w} {h}");
    let (w, h) = axis(", soft clip={domain y=1.5:2.5}");
    assert!(close(w, 6.0) && close(h, 1.5), "{w} {h}");
    // a key other than `domain` and `domain y` is not understood
    let body = r"\begin{axis}[xmin=0, xmax=4, ymin=0, ymax=4]
        \addplot[name path=A, draw=none] coordinates {(0,1) (4,1)};
        \addplot[name path=B, draw=none] coordinates {(0,3) (4,3)};
        \addplot[red] fill between[of=A and B, soft clip={range=1:3}];
        \end{axis}";
    let (c, w) = canvas_and_warnings(body);
    assert!(w.iter().any(|a| a.contains("soft clip 'range=1:3' not understood")), "{w:?}");
    assert_eq!(filled(&c).len(), 1);
    // a domain soft clip with a point path fills nothing
    let body = r"\begin{axis}[xmin=0, xmax=4, ymin=0, ymax=4]
        \addplot[name path=A, draw=none] coordinates {(1,1) (1,1)};
        \addplot[name path=B, draw=none] coordinates {(0,3) (4,3)};
        \addplot[red] fill between[of=A and B, soft clip={domain=1:3}];
        \end{axis}";
    assert!(filled(&canvas(body)).is_empty());
}

/// A key the library does not handle is a TikZ key of the area, with a value or without.
#[test]
fn other_keys_are_tikz_keys() {
    let (c, w) = canvas_and_warnings(&format!(r"{LINES} \tikzfillbetween[of=A and B, every segment/.style={{fill=green}}, even odd rule]{{}};"));
    assert!(w.is_empty(), "{w:?}");
    let d = paths(&c).into_iter().find(|d| d.fill.is_some()).expect("an area");
    let fill = d.fill.unwrap();
    assert_eq!(fill.paint, Paint::Solid(Color::rgb(0.0, 1.0, 0.0)));
    assert!(fill.even_odd);
}

/// `intersection segments` puts pieces of two crossing paths into the path being built.
#[test]
fn intersection_segments_sequence() {
    let seq = |s: &str| {
        let (c, w) = canvas_and_warnings(&format!(r"{CROSS} \fill[intersection segments={{of=A and B, {s}}}];"));
        (filled(&c).into_iter().next().map(|f| f.0).unwrap_or_default(), w)
    };
    // the default: A's first piece, then B's second
    let (p, w) = seq("");
    assert!(w.is_empty(), "{w:?}");
    assert!(same(&p, &[(0.0, 0.0), (1.0, 1.0), (1.0, 1.0), (2.0, 0.0)]), "{p:?}");
    // counted from 1 (L, R), backwards, from the end
    let (p, w) = seq("sequence={L2 -- R1[reverse]}");
    assert!(w.is_empty(), "{w:?}");
    assert!(same(&p, &[(1.0, 1.0), (2.0, 2.0), (1.0, 1.0), (0.0, 2.0)]), "{p:?}");
    let (p, _) = seq("sequence={A-1 -- B0[reverse=false]}");
    assert!(same(&p, &[(1.0, 1.0), (2.0, 2.0), (0.0, 2.0), (1.0, 1.0)]), "{p:?}");
    // `*` is the whole path; `reverse` walks every piece backwards; a piece without `--` starts anew
    let (p, _) = seq("sequence={A* B*}, reverse");
    assert!(same(&p, &[(2.0, 2.0), (0.0, 0.0), (2.0, 0.0), (0.0, 2.0)]), "{p:?}");
    let (p, _) = seq("sequence={A0 [] -- B1}, reverse=false");
    assert!(same(&p, &[(0.0, 0.0), (1.0, 1.0), (1.0, 1.0), (2.0, 0.0)]), "{p:?}");
}

/// What `intersection segments` cannot use is warned about.
#[test]
fn intersection_segments_warnings() {
    let cases = [
        (CROSS, "of=A", "'of=A and B' missing"),
        (CROSS, "of=A and C", "no path named 'A' or 'C'"),
        (CROSS, "of=A and B, color=red", "option 'color' not supported"),
        (CROSS, "of=A and B, sequence={A0 -- B1[blue]}", "option 'blue' not supported"),
        (CROSS, "of=A and B, sequence={A0 -- X1}", "unexpected 'X1'"),
        (CROSS, "of=A and B, sequence={A7 -- B1}", "no piece 'A7'"),
        (LINES, "of=A and B, sequence={A0 -- B1}", "do not cross"),
    ];
    for (paths, spec, warning) in cases {
        let (_, w) = canvas_and_warnings(&format!(r"{paths} \fill[intersection segments={{{spec}}}];"));
        assert!(w.iter().any(|a| a.contains(warning)), "{spec}: {w:?}");
    }
    // no crossing: the whole paths
    let (c, _) = canvas_and_warnings(&format!(r"{LINES} \fill[intersection segments={{of=A and B, sequence={{A0 -- B1[reverse]}}}}];"));
    let f = filled(&c);
    assert!(same(&f[0].0, &[(0.0, 0.0), (2.0, 0.0), (2.0, 1.0), (0.0, 1.0)]), "{:?}", f[0].0);
    // the pieces go into the path even when `\tikra@insertpath` was undone
    let (c, _) = canvas_and_warnings(&format!(r"{CROSS} \expandafter\let\csname tikra@insertpath\endcsname\relax \fill[intersection segments={{of=A and B}}];"));
    let f = filled(&c);
    assert!(same(&f[0].0, &[(0.0, 0.0), (1.0, 1.0), (1.0, 1.0), (2.0, 0.0)]), "{:?}", f[0].0);
    // a missing piece is skipped, and the next one starts the path anew
    let (c, _) = canvas_and_warnings(&format!(r"{CROSS} \fill[intersection segments={{of=A and B, sequence={{A0 -- A9 -- B1}}}}];"));
    let f = filled(&c);
    assert!(same(&f[0].0, &[(0.0, 0.0), (1.0, 1.0), (1.0, 1.0), (2.0, 0.0)]), "{:?}", f[0].0);
}

/// The commands on soft paths: a named path, the pieces of two crossing paths, set and added.
#[test]
fn soft_path_commands() {
    // \tikzgetnamedpath and \pgfsetpathandBB: the named path drawn again
    let (c, w) = canvas_and_warnings(&format!(r"{CROSS} \tikzgetnamedpath{{A}} \let\a=\pgfretval \pgfsetpathandBB\a \pgfusepath{{fill}}"));
    assert!(w.is_empty(), "{w:?}");
    let f = filled(&c);
    assert!(same(&f[0].0, &[(0.0, 0.0), (2.0, 2.0)]), "{f:?}");
    // a missing name
    let (_, w) = canvas_and_warnings(r"\tikzgetnamedpath{nothing}");
    assert!(w.iter().any(|a| a.contains("no path named 'nothing'")), "{w:?}");
    // the pieces of the first path of \pgfintersectionofpaths, cut where they cross
    let body = r"\pgfintersectionofpaths{\pgfpathmoveto{\pgfpoint{0cm}{0cm}}\pgfpathlineto{\pgfpoint{2cm}{2cm}}}{\pgfpathmoveto{\pgfpoint{0cm}{2cm}}\pgfpathlineto{\pgfpoint{2cm}{0cm}}}
        \pgfcomputeintersectionsegments{1}
        \pgfgetintersectionsegmentpath{1}{1}\let\b=\pgfretval
        \pgfgetintersectionsegmentpath{1}{0}\let\a=\pgfretval
        \pgfpathreplacefirstmoveto\b
        \pgfsetpathandBB{\a}\pgfaddpathandBB\b \pgfusepath{fill}";
    let (c, w) = canvas_and_warnings(body);
    assert!(w.is_empty(), "{w:?}");
    let f = filled(&c);
    // the first piece, then the second joined to it by a line
    assert!(same(&f[0].0, &[(0.0, 0.0), (1.0, 1.0), (1.0, 1.0), (2.0, 2.0)]), "{f:?}");
    // with no path made by \pgfintersectionofpaths there are no pieces: an empty path
    let (c, _) = canvas_and_warnings(
        r"\pgfcomputeintersectionsegments{2} \pgfgetintersectionsegmentpath{2}{0}\let\a=\pgfretval \pgfsetpathandBB\a \pgfpathreplacefirstmoveto\a \pgfusepath{fill}",
    );
    assert!(filled(&c).is_empty());
    // a macro that is not a soft path adds nothing; a command of the library it does not know does nothing
    let (c, _) = canvas_and_warnings(r"\def\a{x} \pgfsetpathandBB\a \pgfusepath{fill} \pgfsetpathandBB{} \pgfusepath{fill}");
    assert!(filled(&c).is_empty());
}

/// `\pgfplotspathaxisoutline` adds the axis' rectangle to the path; outside an axis, nothing.
#[test]
fn axis_outline() {
    let body = r"\begin{axis}[xmin=0, xmax=1, ymin=0, ymax=1, width=4cm, height=3cm, scale only axis, hide axis]
        \pgfplotsextra{\pgfplotspathaxisoutline \pgfusepath{fill}}
        \end{axis}";
    let f = filled(&canvas(body));
    assert_eq!(f.len(), 1);
    let (xs, ys): (Vec<f64>, Vec<f64>) = f[0].0.iter().copied().unzip();
    assert!(close(xs[1] - xs[0], 4.0) && close(ys[2] - ys[1], 3.0), "{:?}", f[0].0);
    let (c, w) = canvas_and_warnings(r"\pgfplotspathaxisoutline \pgfusepath{fill}");
    assert!(filled(&c).is_empty());
    assert!(w.is_empty(), "{w:?}");
}
