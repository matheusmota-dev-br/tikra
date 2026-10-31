"""The Python package, used the way someone who installed it would."""

import pytest

import tikra

FIGURE = r"\begin{tikzpicture}\draw[->] (0,0) -- (1,1) node[right] {$x$};\end{tikzpicture}"


def test_svg():
    svg = tikra.tikz_to_svg(FIGURE)
    assert svg.startswith("<svg")


def test_every_input_form():
    for code in [
        r"\documentclass[tikz]{standalone}\begin{document}\begin{tikzpicture}\draw (0,0)--(1,0);\end{tikzpicture}\end{document}",
        r"\tikz \draw (0,0) circle (1);",
        r"\begin{axis}\addplot {x^2};\end{axis}",
        r"\draw (0,0) -- (1,1);",
    ]:
        assert tikra.render(code)["svg"].startswith("<svg")


def test_render_reports_warnings():
    r = tikra.render(FIGURE)
    assert r["warnings"] == []
    r = tikra.render(r"\begin{tikzpicture}\pic {x}; \draw (0,0) -- (1,0);\end{tikzpicture}")
    assert r["svg"].startswith("<svg")
    assert r["warnings"]


def test_render_describes_the_figure():
    r = tikra.render(FIGURE)
    assert r["alt"] == "Drawing of one shape with the text “x”"
    assert r["description"] == "Text in the figure: x."
    assert "<desc" in r["svg"]
    assert tikra.render(FIGURE, locale="pt-BR")["description"] == "Texto na figura: x."
    assert "Texte de la figure" in tikra.tikz_to_svg(FIGURE, locale="fr")
    with pytest.raises(ValueError, match="unknown locale"):
        tikra.render(FIGURE, locale="xx")


def test_no_figure_is_value_error():
    with pytest.raises(ValueError, match="no tikzpicture"):
        tikra.tikz_to_svg("just text")
    with pytest.raises(ValueError):
        tikra.render("just text")


def test_png():
    svg = tikra.tikz_to_svg(FIGURE)
    opaque = tikra.png(svg)
    assert isinstance(opaque, bytes)
    assert opaque.startswith(b"\x89PNG")
    assert tikra.png(svg, background=False) != opaque
    assert len(tikra.png(svg, dpi=72)) < len(opaque)


def test_png_errors():
    with pytest.raises(ValueError, match="invalid dpi"):
        tikra.png(tikra.tikz_to_svg(FIGURE), dpi=0)
    with pytest.raises(ValueError):
        tikra.png("not svg")


def test_versions():
    assert tikra.__version__.count(".") == 2
    assert tikra.core_version.count(".") == 2


def test_the_source_map():
    figure = r"\tikz \draw (0,0) -- (1,1);"
    assert "data-src" not in tikra.tikz_to_svg(figure)
    assert 'data-src="1:' in tikra.tikz_to_svg(figure, source=True)


def test_the_animation_as_gif_and_apng():
    figure = r"\tikz \draw (0,0) -- (1,1);"
    assert tikra.tikz_to_gif(figure, width=64, fps=5).startswith(b"GIF89a")
    assert tikra.tikz_to_apng(figure, width=64, fps=5).startswith(b"\x89PNG")


def test_the_parts():
    svg = tikra.tikz_to_svg(r"\tikz \draw (0,0) -- (1,1);", parts=True)
    assert 'tabindex="0"' in svg and 'role="group"' in svg


def test_slides():
    figure = r"\begin{tikzpicture}\draw (0,0) -- (1,0); \only<2>{\draw (0,1) -- (1,1);}\end{tikzpicture}"
    assert len(tikra.slides(figure)) == 2
    assert 'data-slides="2"' in tikra.click_through(figure)


def test_animations_played():
    figure = r"\usetikzlibrary{animations}\tikz \fill[red] :fill = {0s=\"red\", 1s=\"blue\"} (0,0) circle (1pt);".replace('\\"', '"')
    assert "<animate" in tikra.tikz_to_svg(figure, play=True)
    assert "<animate" not in tikra.tikz_to_svg(figure)
