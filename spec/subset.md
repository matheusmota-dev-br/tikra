# TikZ subset

What tikra reads and how it draws it. Anything outside this list becomes a warning and the rest of the figure is still drawn.

## Model

A figure becomes a **scene**: paths (straight segments, cubic curves, closings) and nodes (mathra text in a shape). Everything in centimeters, y pointing up. The SVG is written in points (1 cm = 28.4528 pt); the PNG is rasterized by resvg.

## Rules that matter for the drawing to match TeX

- **Coordinates:** Cartesian `(x,y)`, polar `(angle:radius)` (the angle may be a braced expression, `({90+72*\k}:2)`), named nodes and anchors, `+`/`++` relative, `calc`.
- **Unit:** a number without a unit is cm (or the scope's `x=`/`y=` vector); with a unit it is converted (`pt`, `mm`, `in`, `em`).
- **Transforms:** `scale`, `rotate`, `shift`, `xshift`, `yshift` apply to coordinates and to the corners of `rectangle` (which rotates along), but not to node sizes, as in TikZ.
- **Nodes inside a segment:** in `-- node {a} (B)` and `to node {a} (B)` the node goes to the middle of the segment (`pos=0.5`); after `-|` or `|-`, to the corner. An explicit `pos`, `near start` and so on still wins.
- **Lines between nodes** (`(A) -- (B)` with named nodes) leave from and arrive at the shape's **border**, not its center.
- **Nodes:** default `inner sep` of 0.3333 em; a rectangle is text + 2 × inner sep, at least `minimum width/height`; a circle through half the diagonal; a diamond with `aspect`. `above` puts the `south` anchor on the point, and so on; `below=of A` measures the distance border to border (`node distance`). `base`, `base west`, `base east` and `text` sit on the first text line's baseline.
- **Widths:** `ultra thin` 0.1 pt … `thin` 0.4 pt (default) … `ultra thick` 1.6 pt.
- **Tips:** `->`, `->>`, `-Latex`, `-Stealth`, `-|`, `-o` and their reverses; the stroke is pulled back so a Latex or Stealth tip does not leak.
- **Hatching:** `pattern=…` is the fill of the shape; with `\fill`, no solid color is painted underneath unless `fill=` names one.
- **`to` options** (`bend left`, `out`, `in`, `looseness`) can be given to the whole statement and apply to every `to`.
- **Color:** nodes on a path inherit the path's bare color or `color=` (including from a style), not its `draw` or `fill`.
- **`random steps`** uses a fixed pseudo-random sequence: the same figure yields the same bytes.

## pgfplots

`axis` with `width`, `height`, `xmin`…`ymax` (or automatic), `axis lines=left|middle|box|none`, `xlabel`, `ylabel`, `title`, `samples`, `domain`, `legend style={at={(x,y)}}`; `\addplot[…] {expr in x}`, `\addplot coordinates {…}`, `only marks`, `mark=…`, `no marks`, `\addlegendentry`. Ticks with a step of 1, 2 or 5 × 10ⁿ; values written by mathra (decimal comma under the pt-BR locale). Without data it draws the default window from 0 to 1, as pgfplots does.

## Out, for now

`\matrix`, `\pic`, `\graph`, `shade` with a gradient (becomes a solid fill), `transform shape`, the `mindmap`, `trees`, `circuits`, `3d` libraries, `tikz-cd`, `\chemfig`. They become warnings.

## Known differences from TeX

- Latin Modern (Computer Modern) from its math font: text italic is the math italic, and typewriter punctuation is the roman one at typewriter width; no sans-serif family (`\sffamily` is ignored).
- No hyphenation: `text width` breaks only between words.
- Font size is per node, not per span (`{\small …}` in the middle of the text applies to the whole node).
- pgfplots ticks may use a different step (pgfplots' algorithm depends on the width).
