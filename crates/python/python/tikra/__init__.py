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
    warnings: List[str]
    """What was ignored or approximated: unsupported commands, labels with broken LaTeX."""


def tikz_to_svg(code: str) -> str:
    """Converts a TikZ figure to SVG.

    ``code`` is a full ``standalone`` document, a bare ``tikzpicture``, an inline ``\\tikz``, an
    ``axis`` environment or loose TikZ statements. What tikra does not understand is skipped (see
    :func:`render` for the warnings). Raises ValueError when there is no figure to draw.
    """
    return _tikra.tikz_to_svg(code)


def render(code: str) -> Result:
    """The SVG and the warnings at once."""
    svg, warnings = _tikra.render(code)
    return {"svg": svg, "warnings": warnings}


def png(svg: str, *, dpi: float = 300, background: bool = True) -> bytes:
    """Rasterizes an SVG written by tikra to PNG bytes.

    ``dpi`` is the resolution (300 for print); ``background`` False keeps it transparent instead of
    white. Raises ValueError on an invalid SVG or dpi, or past 100 megapixels.
    """
    return bytes(_tikra.png(svg, dpi=dpi, background=background))


__all__ = ["Result", "__version__", "core_version", "png", "render", "tikz_to_svg"]
