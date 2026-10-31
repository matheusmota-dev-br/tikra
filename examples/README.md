# Examples

One small figure for each thing the construction animation's steps made possible. `examples/run.sh` runs the ones that use the command line and leaves the results in `examples/out/`, which Git ignores. The others are for the playground: open `site/index.html` after `site/build.sh`, no server needed, and paste the file in the editor.

| File | Where | What to try |
|---|---|---|
| `source-map.tex` | playground, `--source` | Click a stroke to find its line, or put the cursor on a line to see what it drew. The macro and the loop point at their calls, and each `\addplot` at its own line. |
| `diff-before.tex`, `diff-after.tex` | playground | Paste the first, press Keep this version, paste the second, press Compare. The figure grew to the left, and only what changed is colored. |
| `construction.tex` | `-o .gif`, `-o .apng` | The figure drawing itself, from the terminal. Try `--width` and `--fps`. |
| `walk.tex` | playground, `--parts` | Press Tab to reach the figure and walk it with the arrows. With a screen reader, each part is read on its own. |
| `slides.tex` | `--slides`, `--click` | Four slides from one figure: `\pause`, `\uncover`, `\only` and `\draw<4>`. The `--click` SVG advances on each click in a browser. |
| `plotter.tex` | `-o .gcode`, `-o .hpgl` | Strokes for a pen plotter. Paste the G-code in any online viewer. Try `--paper 210x297` and `--keep-order`. |
| `narration.tex` | playground | Move the step slider: the nodes and connections have a sentence under it, and the free line stays silent. Change the language to hear them in it. |
