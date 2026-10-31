"""TikZ figures to SVG and PNG, without a TeX install.

>>> import tikra
>>> tikra.tikz_to_svg(r"\\tikz \\draw (0,0) circle (1);")[:4]
'<svg'
"""

from __future__ import annotations

from typing import List, TypedDict

from . import _tikra

__version__: str = _tikra.__version__
#: The version of tikra's core inside this package.
core_version: str = _tikra.core_version


class Result(TypedDict):
    svg: str
    """Standalone SVG, with math labels typeset by mathra."""
    alt: str
    """The figure in a sentence, for readers who cannot see it: the SVG's ``<title>``, an image's alt text."""
    description: str
    """The figure in words (axes, data, nodes and their connections): the SVG's ``<desc>``."""
    warnings: List[str]
    """What was ignored or approximated: unsupported commands, labels with broken LaTeX."""


def tikz_to_svg(code: str, *, locale: str = "en", animate: bool = False, steps: bool = False, source: bool = False, parts: bool = False, play: bool = False, points: bool = False) -> str:
    """Converts a TikZ figure to SVG, with the figure in words (its ``<title>`` and ``<desc>``) in
    ``locale``: one of mathra's languages, ``en``, ``pt-BR``, ``es``, ``fr``, ``de``, ``it``, ``ru``,
    ``zh``, ``ja``, ``ko``, ``hi``, ``ar``, ``id``, ``tr``, ``nl``, ``pl``. The drawing is the same in all.

    ``code`` is a full ``standalone`` document, a bare ``tikzpicture``, an inline ``\\tikz``, an
    ``axis`` environment or loose TikZ statements. What tikra does not understand is skipped (see
    :func:`render` for the warnings). Raises ValueError when there is no figure to draw or the
    locale is unknown.

    ``animate=True`` adds the construction animation: in a browser, the figure draws itself path by
    path, in the order TikZ drew it. ``steps=True`` numbers each element by its step instead
    (``data-step``, and ``data-steps`` on the ``<svg>``), with no CSS, for a page that walks through
    the figure itself. ``source=True`` numbers them and adds the place in the code that drew each
    one (``data-src="line:column-line:column"``, from 1). ``parts=True`` makes each thing drawn a
    group a screen reader and the keyboard can reach. ``play=True`` plays the animations library in a
    browser, instead of the still picture. ``points=True`` makes each bar and mark of a plot an
    element of its own, with ``data-x``, ``data-y`` and a tip.
    """
    return _tikra.tikz_to_svg(code, locale=locale, animate=animate, steps=steps, source=source, parts=parts, play=play, points=points)


def slides(code: str, *, locale: str = "en") -> list[str]:
    """The SVG of each slide of a figure with Beamer's overlays. A figure without them has one."""
    return _tikra.slides(code, locale=locale)


def click_through(code: str, *, locale: str = "en") -> str:
    """The slides as one SVG that advances on a click. It takes a browser, not an ``<img>``."""
    return _tikra.click_through(code, locale=locale)


def tikz_to_gcode(code: str, *, keep_order: bool = False, paper: tuple[float, float] | None = None) -> str:
    """The figure's strokes as G-code for a pen plotter, in mm."""
    return _tikra.tikz_to_gcode(code, keep_order=keep_order, paper=paper)


def tikz_to_hpgl(code: str, *, keep_order: bool = False, paper: tuple[float, float] | None = None) -> str:
    """The figure's strokes as HPGL for a pen plotter."""
    return _tikra.tikz_to_hpgl(code, keep_order=keep_order, paper=paper)


def tikz_to_pen_svg(code: str, *, seconds: float = 10.0, keep_order: bool = False, paper: tuple[float, float] | None = None) -> str:
    """An SVG where a pen draws the figure in a loop, with no script: it plays in a browser."""
    return _tikra.tikz_to_pen_svg(code, seconds=seconds, keep_order=keep_order, paper=paper)


def tikz_to_pen_gif(code: str, *, seconds: float = 10.0, width: int = 560, fps: int = 15, keep_order: bool = False, paper: tuple[float, float] | None = None) -> bytes:
    """A pen drawing the figure, as GIF bytes."""
    return _tikra.tikz_to_pen_gif(code, seconds=seconds, width=width, fps=fps, keep_order=keep_order, paper=paper)


def tikz_to_pen_apng(code: str, *, seconds: float = 10.0, width: int = 560, fps: int = 15, keep_order: bool = False, paper: tuple[float, float] | None = None) -> bytes:
    """A pen drawing the figure, as APNG bytes."""
    return _tikra.tikz_to_pen_apng(code, seconds=seconds, width=width, fps=fps, keep_order=keep_order, paper=paper)


def tikz_to_gif(code: str, *, width: int = 560, fps: int = 15) -> bytes:
    """The figure's construction animation as GIF bytes, on white, looping."""
    return _tikra.tikz_to_gif(code, width=width, fps=fps)


def tikz_to_apng(code: str, *, width: int = 560, fps: int = 15) -> bytes:
    """The figure's construction animation as APNG bytes, on white, looping."""
    return _tikra.tikz_to_apng(code, width=width, fps=fps)


def render(code: str, *, locale: str = "en", animate: bool = False, steps: bool = False, source: bool = False, parts: bool = False, play: bool = False, points: bool = False) -> Result:
    """The SVG, the figure in words (in ``locale``) and the warnings at once."""
    svg, alt, description, warnings = _tikra.render(code, locale=locale, animate=animate, steps=steps, source=source, parts=parts, play=play, points=points)
    return {"svg": svg, "alt": alt, "description": description, "warnings": warnings}


def png(svg: str, *, dpi: float = 300, background: bool = True) -> bytes:
    """Rasterizes an SVG written by tikra to PNG bytes.

    ``dpi`` is the resolution (300 for print); ``background`` False keeps it transparent instead of
    white. Raises ValueError on an invalid SVG or dpi, or past 100 megapixels.
    """
    return bytes(_tikra.png(svg, dpi=dpi, background=background))


__all__ = ["Result", "__version__", "click_through", "core_version", "png", "render", "slides", "tikz_to_apng", "tikz_to_gcode", "tikz_to_gif", "tikz_to_hpgl", "tikz_to_pen_apng", "tikz_to_pen_gif", "tikz_to_pen_svg", "tikz_to_svg"]
