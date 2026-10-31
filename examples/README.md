# Examples

One small figure for each thing the construction animation's steps made possible. `examples/run.sh` runs the ones that use the command line and leaves the results in `examples/out/`, which Git ignores. The others are for the playground: open `site/index.html` after `site/build.sh`, no server needed, and paste the file in the editor.

| File | Where | What to try |
|---|---|---|
| `source-map.tex` | playground, `--source` | Click a stroke to find its line, or put the cursor on a line to see what it drew. The macro and the loop point at their calls, and each `\addplot` at its own line. |
| `diff-before.tex`, `diff-after.tex` | playground | Paste the first, press Keep this version, paste the second, press Compare. The figure grew to the left, and only what changed is colored. |
| `construction.tex` | `-o .gif`, `-o .apng` | The figure drawing itself, from the terminal. Try `--width` and `--fps`. |
