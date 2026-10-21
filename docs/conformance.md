# Conformance with TikZ

How far tikra is from TikZ itself: every example of the TikZ manual (pgf 3.1.11a) and of the pgfplots
manual (1.18.2), drawn by pdflatex (TeX Live 2025) and by tikra, and compared. How it works and how
to run it: [tools/conformance](../tools/conformance/README.md).

**873 of 2448 figures (35.7%) match TeX**; 695 are close, 876 differ and 4 fail in tikra. 664 of the ones that do not match came out with no warning.

| | Figures | Match | Close | Different | Fails | Mean F1 |
|---|---:|---:|---:|---:|---:|---:|
| All | 2448 | 873 (35.7%) | 695 (28.4%) | 876 (35.8%) | 4 (0.2%) | 0.70 |
| TikZ manual | 1810 | 756 (41.8%) | 366 (20.2%) | 684 (37.8%) | 4 (0.2%) | 0.67 |
| pgfplots manual | 638 | 117 (18.3%) | 329 (51.6%) | 192 (30.1%) | 0 (0.0%) | 0.79 |

*Match*: same shape (F1 ≥ 0.9 on the edges, with 0.5 mm of tolerance), no missing or extra piece, the same colors and size. *Close*: F1 ≥ 0.75.
*Fails*: tikra returned an error, panicked or took more than 10 s. Mean F1 counts a failure as 0.

Not counted: 136 where tex could not compile it, 18 where tex draws nothing visible, and 535 examples that are not TikZ figures (pgf basic layer, text, tables).

## Speed

The same 192 figures (a fixed sample of the manuals), one at a time, on Darwin arm64:

| | Median per figure | p95 |
|---|---:|---:|
| pdflatex (texlive/texlive:latest) | 647 ms | 1031 ms |
| tikra, TikZ to SVG | 4.71 ms | 12.14 ms |

Median ratio: tikra is **130× faster**. pdflatex time includes starting TeX and loading the libraries of the manual, which is what a user
waits for; tikra's is the conversion inside a running process (the command line adds a few ms to start).

## By chapter

Sorted by how many figures do not match, the work left.

| Chapter | Figures | Match | Close | Different | Fails | Mean F1 |
|---|---:|---:|---:|---:|---:|---:|
| Style Sheets and Legends <sub>tikz</sub> | 85 | 0 (0.0%) | 0 (0.0%) | 85 (100.0%) | 0 (0.0%) | 0.00 |
| Axes <sub>tikz</sub> | 78 | 0 (0.0%) | 1 (1.3%) | 77 (98.7%) | 0 (0.0%) | 0.01 |
| Axis Descriptions <sub>pgfplots</sub> | 77 | 11 (14.3%) | 55 (71.4%) | 11 (14.3%) | 0 (0.0%) | 0.87 |
| Circuit Libraries <sub>tikz</sub> | 64 | 0 (0.0%) | 9 (14.1%) | 55 (85.9%) | 0 (0.0%) | 0.38 |
| Three Dimensional Plot Types <sub>pgfplots</sub> | 85 | 23 (27.1%) | 36 (42.4%) | 26 (30.6%) | 0 (0.0%) | 0.80 |
| Two Dimensional Plot Types <sub>pgfplots</sub> | 72 | 12 (16.7%) | 42 (58.3%) | 18 (25.0%) | 0 (0.0%) | 0.84 |
| Nodes and Edges <sub>tikz</sub> | 118 | 60 (50.8%) | 33 (28.0%) | 25 (21.2%) | 0 (0.0%) | 0.84 |
| Shape Library <sub>tikz</sub> | 97 | 43 (44.3%) | 17 (17.5%) | 37 (38.1%) | 0 (0.0%) | 0.80 |
| Markers, Linestyles, (Background) Colors and Colormaps <sub>pgfplots</sub> | 50 | 6 (12.0%) | 36 (72.0%) | 8 (16.0%) | 0 (0.0%) | 0.86 |
| Calendar Library <sub>tikz</sub> | 41 | 0 (0.0%) | 0 (0.0%) | 41 (100.0%) | 0 (0.0%) | 0.01 |
| Specifying Graphs <sub>tikz</sub> | 159 | 128 (80.5%) | 30 (18.9%) | 1 (0.6%) | 0 (0.0%) | 0.99 |
| Coordinate, Canvas, and Nonlinear Transformations <sub>tikz</sub> | 38 | 8 (21.1%) | 20 (52.6%) | 10 (26.3%) | 0 (0.0%) | 0.80 |
| Decoration Library <sub>tikz</sub> | 81 | 51 (63.0%) | 25 (30.9%) | 5 (6.2%) | 0 (0.0%) | 0.92 |
| Specifying Coordinates <sub>tikz</sub> | 66 | 37 (56.1%) | 26 (39.4%) | 3 (4.5%) | 0 (0.0%) | 0.91 |
| Fill between <sub>pgfplots</sub> | 29 | 2 (6.9%) | 26 (89.7%) | 1 (3.4%) | 0 (0.0%) | 0.95 |
| Patchplots Library <sub>pgfplots</sub> | 29 | 3 (10.3%) | 20 (69.0%) | 6 (20.7%) | 0 (0.0%) | 0.80 |
| Statistics <sub>pgfplots</sub> | 26 | 0 (0.0%) | 1 (3.8%) | 25 (96.2%) | 0 (0.0%) | 0.52 |
| Transparency <sub>tikz</sub> | 46 | 21 (45.7%) | 11 (23.9%) | 14 (30.4%) | 0 (0.0%) | 0.80 |
| Animations <sub>tikz</sub> | 24 | 0 (0.0%) | 9 (37.5%) | 15 (62.5%) | 0 (0.0%) | 0.35 |
| Arrows <sub>tikz</sub> | 104 | 80 (76.9%) | 24 (23.1%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Tick Options <sub>pgfplots</sub> | 32 | 8 (25.0%) | 22 (68.8%) | 2 (6.2%) | 0 (0.0%) | 0.96 |
| Plot Handler Library <sub>tikz</sub> | 23 | 1 (4.3%) | 2 (8.7%) | 20 (87.0%) | 0 (0.0%) | 0.46 |
| Actions on Paths <sub>tikz</sub> | 72 | 50 (69.4%) | 14 (19.4%) | 8 (11.1%) | 0 (0.0%) | 0.92 |
| Syntax for Path Specifications <sub>tikz</sub> | 52 | 32 (61.5%) | 12 (23.1%) | 8 (15.4%) | 0 (0.0%) | 0.87 |
| Plots of Functions <sub>tikz</sub> | 30 | 10 (33.3%) | 2 (6.7%) | 18 (60.0%) | 0 (0.0%) | 0.41 |
| Visualizers <sub>tikz</sub> | 18 | 0 (0.0%) | 0 (0.0%) | 18 (100.0%) | 0 (0.0%) | 0.00 |
| Paper-Folding Diagrams Library <sub>tikz</sub> | 18 | 0 (0.0%) | 0 (0.0%) | 18 (100.0%) | 0 (0.0%) | 0.00 |
| Matrices and Alignment <sub>tikz</sub> | 26 | 9 (34.6%) | 9 (34.6%) | 8 (30.8%) | 0 (0.0%) | 0.80 |
| The Axis Environments <sub>pgfplots</sub> | 21 | 4 (19.0%) | 11 (52.4%) | 6 (28.6%) | 0 (0.0%) | 0.81 |
| Tutorial: A Picture for Karl's Students <sub>tikz</sub> | 46 | 30 (65.2%) | 12 (26.1%) | 4 (8.7%) | 0 (0.0%) | 0.91 |
| Mindmap Drawing Library <sub>tikz</sub> | 16 | 1 (6.2%) | 3 (18.8%) | 12 (75.0%) | 0 (0.0%) | 0.33 |
| Three Point Perspective Drawing Library <sub>tikz</sub> | 15 | 0 (0.0%) | 0 (0.0%) | 15 (100.0%) | 0 (0.0%) | 0.22 |
| Scaling Options <sub>pgfplots</sub> | 22 | 7 (31.8%) | 7 (31.8%) | 8 (36.4%) | 0 (0.0%) | 0.80 |
| Creating Data Visualizations <sub>tikz</sub> | 13 | 0 (0.0%) | 0 (0.0%) | 13 (100.0%) | 0 (0.0%) | 0.00 |
| Petri-Net Drawing Library <sub>tikz</sub> | 13 | 0 (0.0%) | 0 (0.0%) | 13 (100.0%) | 0 (0.0%) | 0.12 |
| Tutorial: Diagrams as Simple Graphs <sub>tikz</sub> | 19 | 6 (31.6%) | 12 (63.2%) | 1 (5.3%) | 0 (0.0%) | 0.89 |
| Polar Axes <sub>pgfplots</sub> | 13 | 0 (0.0%) | 0 (0.0%) | 13 (100.0%) | 0 (0.0%) | 0.20 |
| Ternary Diagrams <sub>pgfplots</sub> | 13 | 0 (0.0%) | 0 (0.0%) | 13 (100.0%) | 0 (0.0%) | 0.29 |
| Grid Options <sub>pgfplots</sub> | 18 | 5 (27.8%) | 11 (61.1%) | 2 (11.1%) | 0 (0.0%) | 0.89 |
| Smith Charts <sub>pgfplots</sub> | 12 | 0 (0.0%) | 0 (0.0%) | 12 (100.0%) | 0 (0.0%) | 0.20 |
| Polar Axes <sub>tikz</sub> | 11 | 0 (0.0%) | 0 (0.0%) | 11 (100.0%) | 0 (0.0%) | 0.00 |
| Chains <sub>tikz</sub> | 14 | 3 (21.4%) | 8 (57.1%) | 3 (21.4%) | 0 (0.0%) | 0.85 |
| Spy Library: Magnifying Parts of Pictures <sub>tikz</sub> | 11 | 0 (0.0%) | 1 (9.1%) | 10 (90.9%) | 0 (0.0%) | 0.56 |
| Transformations <sub>tikz</sub> | 22 | 11 (50.0%) | 10 (45.5%) | 1 (4.5%) | 0 (0.0%) | 0.92 |
| Tutorial: Euclid's Amber Version of the Elements <sub>tikz</sub> | 14 | 3 (21.4%) | 3 (21.4%) | 8 (57.1%) | 0 (0.0%) | 0.64 |
| Constructing Paths <sub>tikz</sub> | 16 | 7 (43.8%) | 7 (43.8%) | 2 (12.5%) | 0 (0.0%) | 0.80 |
| Error Bars <sub>pgfplots</sub> | 9 | 0 (0.0%) | 6 (66.7%) | 3 (33.3%) | 0 (0.0%) | 0.69 |
| Providing Data for a Data Visualization <sub>tikz</sub> | 8 | 0 (0.0%) | 0 (0.0%) | 8 (100.0%) | 0 (0.0%) | 0.00 |
| Automata Drawing Library <sub>tikz</sub> | 8 | 0 (0.0%) | 0 (0.0%) | 8 (100.0%) | 0 (0.0%) | 0.25 |
| Matrix Library <sub>tikz</sub> | 14 | 6 (42.9%) | 4 (28.6%) | 4 (28.6%) | 0 (0.0%) | 0.83 |
| Pattern Library <sub>tikz</sub> | 8 | 0 (0.0%) | 8 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0.94 |
| Tutorial: A Lecture Map for Johannes <sub>tikz</sub> | 10 | 2 (20.0%) | 0 (0.0%) | 8 (80.0%) | 0 (0.0%) | 0.30 |
| Grouping plots <sub>pgfplots</sub> | 8 | 0 (0.0%) | 0 (0.0%) | 8 (100.0%) | 0 (0.0%) | 0.35 |
| Alignment Options <sub>pgfplots</sub> | 8 | 0 (0.0%) | 5 (62.5%) | 3 (37.5%) | 0 (0.0%) | 0.70 |
| Specifying the Plotted Range <sub>pgfplots</sub> | 13 | 5 (38.5%) | 4 (30.8%) | 4 (30.8%) | 0 (0.0%) | 0.84 |
| Nodes and Shapes <sub>tikz</sub> | 8 | 1 (12.5%) | 4 (50.0%) | 1 (12.5%) | 2 (25.0%) | 0.65 |
| Entity-Relationship Diagram Drawing Library <sub>tikz</sub> | 7 | 0 (0.0%) | 0 (0.0%) | 7 (100.0%) | 0 (0.0%) | 0.43 |
| Tree Library <sub>tikz</sub> | 7 | 0 (0.0%) | 2 (28.6%) | 5 (71.4%) | 0 (0.0%) | 0.33 |
| Repeating Things: The Foreach Statement <sub>tikz</sub> | 13 | 6 (46.2%) | 2 (15.4%) | 5 (38.5%) | 0 (0.0%) | 0.64 |
| Pics: Small Pictures on Paths <sub>tikz</sub> | 14 | 7 (50.0%) | 3 (21.4%) | 4 (28.6%) | 0 (0.0%) | 0.78 |
| Making Trees Grow <sub>tikz</sub> | 27 | 20 (74.1%) | 4 (14.8%) | 3 (11.1%) | 0 (0.0%) | 0.94 |
| Tutorial: A Petri-Net for Hagen <sub>tikz</sub> | 28 | 21 (75.0%) | 0 (0.0%) | 7 (25.0%) | 0 (0.0%) | 0.87 |
| Utilities and Basic Level Commands <sub>pgfplots</sub> | 7 | 0 (0.0%) | 6 (85.7%) | 1 (14.3%) | 0 (0.0%) | 0.89 |
| Skipping Or Changing Coordinates -- Filters <sub>pgfplots</sub> | 7 | 0 (0.0%) | 7 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0.91 |
| Background Library <sub>tikz</sub> | 12 | 6 (50.0%) | 4 (33.3%) | 2 (16.7%) | 0 (0.0%) | 0.87 |
| Lindenmayer System Drawing Library <sub>tikz</sub> | 6 | 0 (0.0%) | 1 (16.7%) | 5 (83.3%) | 0 (0.0%) | 0.15 |
| Math Library <sub>tikz</sub> | 6 | 0 (0.0%) | 1 (16.7%) | 5 (83.3%) | 0 (0.0%) | 0.25 |
| Shadings Library <sub>tikz</sub> | 11 | 5 (45.5%) | 0 (0.0%) | 6 (54.5%) | 0 (0.0%) | 0.72 |
| pgfmanual-en-tikz-scopes <sub>tikz</sub> | 13 | 7 (53.8%) | 3 (23.1%) | 3 (23.1%) | 0 (0.0%) | 0.83 |
| Solving a Real Use Case: Function Visualization <sub>pgfplots</sub> | 6 | 0 (0.0%) | 2 (33.3%) | 4 (66.7%) | 0 (0.0%) | 0.78 |
| Matrices <sub>tikz</sub> | 7 | 2 (28.6%) | 3 (42.9%) | 2 (28.6%) | 0 (0.0%) | 0.69 |
| Three Dimensional Drawing Library <sub>tikz</sub> | 5 | 0 (0.0%) | 1 (20.0%) | 4 (80.0%) | 0 (0.0%) | 0.27 |
| To Path Library <sub>tikz</sub> | 13 | 8 (61.5%) | 2 (15.4%) | 3 (23.1%) | 0 (0.0%) | 0.86 |
| Shadows Library <sub>tikz</sub> | 14 | 9 (64.3%) | 3 (21.4%) | 2 (14.3%) | 0 (0.0%) | 0.88 |
| Tutorials and Guidelines <sub>tikz</sub> | 9 | 4 (44.4%) | 0 (0.0%) | 5 (55.6%) | 0 (0.0%) | 0.58 |
| The Picture's Size: Bounding Box and Clipping <sub>pgfplots</sub> | 5 | 0 (0.0%) | 1 (20.0%) | 4 (80.0%) | 0 (0.0%) | 0.41 |
| Fitting Lines -- Regression <sub>pgfplots</sub> | 5 | 0 (0.0%) | 1 (20.0%) | 4 (80.0%) | 0 (0.0%) | 0.71 |
| Symbolic Coordinates and User Transformations <sub>pgfplots</sub> | 5 | 0 (0.0%) | 1 (20.0%) | 4 (80.0%) | 0 (0.0%) | 0.54 |
| Patterns <sub>tikz</sub> | 4 | 0 (0.0%) | 2 (50.0%) | 2 (50.0%) | 0 (0.0%) | 0.59 |
| Turtle Graphics Library <sub>tikz</sub> | 4 | 0 (0.0%) | 0 (0.0%) | 4 (100.0%) | 0 (0.0%) | 0.00 |
| Views Library <sub>tikz</sub> | 4 | 0 (0.0%) | 2 (50.0%) | 2 (50.0%) | 0 (0.0%) | 0.71 |
| Miscellaneous Options <sub>pgfplots</sub> | 7 | 3 (42.9%) | 4 (57.1%) | 0 (0.0%) | 0 (0.0%) | 0.91 |
| Transforming Coordinate Systems <sub>pgfplots</sub> | 4 | 0 (0.0%) | 0 (0.0%) | 4 (100.0%) | 0 (0.0%) | 0.41 |
| Use Cases involving Scatter Plots <sub>pgfplots</sub> | 5 | 1 (20.0%) | 4 (80.0%) | 0 (0.0%) | 0 (0.0%) | 0.95 |
| Solving a Real Use Case: Functions of Two Variables <sub>pgfplots</sub> | 5 | 1 (20.0%) | 3 (60.0%) | 1 (20.0%) | 0 (0.0%) | 0.84 |
| Decorations <sub>tikz</sub> | 10 | 7 (70.0%) | 3 (30.0%) | 0 (0.0%) | 0 (0.0%) | 0.94 |
| Creating Plots <sub>tikz</sub> | 3 | 0 (0.0%) | 0 (0.0%) | 3 (100.0%) | 0 (0.0%) | 0.23 |
| Quick Commands <sub>tikz</sub> | 3 | 0 (0.0%) | 2 (66.7%) | 1 (33.3%) | 0 (0.0%) | 0.76 |
| pgfmanual-en-dv-examples <sub>tikz</sub> | 3 | 0 (0.0%) | 0 (0.0%) | 3 (100.0%) | 0 (0.0%) | 0.00 |
| Fitting Library <sub>tikz</sub> | 4 | 1 (25.0%) | 2 (50.0%) | 1 (25.0%) | 0 (0.0%) | 0.85 |
| Mathematical Expressions <sub>tikz</sub> | 3 | 0 (0.0%) | 1 (33.3%) | 2 (66.7%) | 0 (0.0%) | 0.55 |
| Decorated Paths <sub>tikz</sub> | 31 | 28 (90.3%) | 3 (9.7%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Design Principles <sub>tikz</sub> | 6 | 3 (50.0%) | 1 (16.7%) | 2 (33.3%) | 0 (0.0%) | 0.82 |
| 3D Axis Configuration <sub>pgfplots</sub> | 13 | 10 (76.9%) | 3 (23.1%) | 0 (0.0%) | 0 (0.0%) | 0.97 |
| pgfplots.reference.closingplots <sub>pgfplots</sub> | 6 | 3 (50.0%) | 3 (50.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Shadings <sub>tikz</sub> | 2 | 0 (0.0%) | 0 (0.0%) | 2 (100.0%) | 0 (0.0%) | 0.19 |
| Introduction to Data Visualization <sub>tikz</sub> | 2 | 0 (0.0%) | 0 (0.0%) | 2 (100.0%) | 0 (0.0%) | 0.00 |
| Babel Library <sub>tikz</sub> | 2 | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 2 (100.0%) | 0.00 |
| Key Management <sub>tikz</sub> | 5 | 3 (60.0%) | 2 (40.0%) | 0 (0.0%) | 0 (0.0%) | 0.93 |
| Units in Labels <sub>pgfplots</sub> | 2 | 0 (0.0%) | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0.92 |
| Number Formatting Options <sub>pgfplots</sub> | 6 | 4 (66.7%) | 2 (33.3%) | 0 (0.0%) | 0 (0.0%) | 0.97 |
| Style Options <sub>pgfplots</sub> | 2 | 0 (0.0%) | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Interoperability <sub>pgfplots</sub> | 3 | 1 (33.3%) | 1 (33.3%) | 1 (33.3%) | 0 (0.0%) | 0.68 |
| Memory and Speed considerations <sub>pgfplots</sub> | 4 | 2 (50.0%) | 2 (50.0%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Solving a Real Use Case: Scientific Data Analysis <sub>pgfplots</sub> | 7 | 5 (71.4%) | 2 (28.6%) | 0 (0.0%) | 0 (0.0%) | 0.92 |
| pgfmanual-en-base-scopes <sub>tikz</sub> | 11 | 10 (90.9%) | 1 (9.1%) | 0 (0.0%) | 0 (0.0%) | 0.96 |
| Angle Library <sub>tikz</sub> | 6 | 5 (83.3%) | 0 (0.0%) | 1 (16.7%) | 0 (0.0%) | 0.83 |
| Through Library <sub>tikz</sub> | 1 | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0.81 |
| Customizing the Mathematical Engine <sub>tikz</sub> | 2 | 1 (50.0%) | 1 (50.0%) | 0 (0.0%) | 0 (0.0%) | 0.95 |
| Additional Mathematical Commands <sub>tikz</sub> | 1 | 0 (0.0%) | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0.00 |
| Date and Calendar Utility Macros <sub>tikz</sub> | 1 | 0 (0.0%) | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0.00 |
| Animation System Layer <sub>tikz</sub> | 1 | 0 (0.0%) | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0.00 |
| Components <sub>pgfplots</sub> | 1 | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0.84 |
| Layered Graphics <sub>tikz</sub> | 1 | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Resource Description Framework Library <sub>tikz</sub> | 2 | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Decoration: Soft Clipping <sub>pgfplots</sub> | 1 | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |

## What tikra reported

Warnings, by how many figures they appear in (the varying part replaced by …):

| Figures | Warning |
|---:|---|
| 423 | `option … not supported: ignored` |
| 219 | `\datavisualization not supported: … ignored` |
| 48 | `axis without data: drew the default window from N to N` |
| 42 | `\calendar not supported: … ignored` |
| 37 | `unrecognized path fragment: …` |
| 24 | `\draw: figure: unknown coordinate: (…); statement ignored` |
| 17 | `node without text: … missing` |
| 15 | `shape … not supported: drawn as a rectangle` |
| 14 | `\draw: figure: plot file (…) not supported; statement ignored` |
| 12 | `unknown color …` |
| 12 | `\spy not supported: … ignored` |
| 12 | `key handler … not supported (…): ignored` |
| 11 | `\fill: figure: coordinate system … not supported; statement ignored` |
| 9 | `\draw: expression: function without parentheses: right; statement ignored` |
| 9 | `\draw: figure: unknown anchor: s.output; statement ignored` |
| 9 | `\path: figure: unknown coordinate: (…); statement ignored` |
| 9 | `fill between: no path named … or …` |
| 8 | `text outside a statement ignored` |
| 8 | `\nextgroupplot not supported: the plots of all groups are drawn in one axis` |
| 7 | `\draw: figure: unknown anchor: s.input; statement ignored` |
| 6 | `\pgftransformnonlinear not supported: … ignored` |
| 6 | `\shade: figure: unknown color …; statement ignored` |
| 6 | `shape backgrounds: shape … not supported: drawn as a rectangle` |
| 6 | `E-REC-N column N: … without a matching …; closed at the end` |
| 5 | `\pgfslopedattimetrue not supported: … ignored` |
| 5 | `\pgfsetcurvilinearbeziercurve not supported: … ignored` |
| 5 | `\pgfsetfading not supported: … ignored` |
| 5 | `\draw: figure: coordinate system … not supported; statement ignored` |
| 5 | `\draw: expression: function without parentheses: up; statement ignored` |
| 5 | `\draw: figure: unknown anchor: A.input N; statement ignored` |
| 4 | `\draw: figure: unknown anchor: aN.output; statement ignored` |
| 4 | `\draw: figure: unknown anchor: o.output; statement ignored` |
| 4 | `\draw: figure: unknown anchor: A.output; statement ignored` |
| 4 | `E-REC-N column N: unsupported text command: \textcolor; written as is` |
| 4 | `\path: figure: name intersections: no path named …; statement ignored` |
| 4 | `\addplot table: file … not found` |
| 3 | `\pgfplothandlerlineto not supported: … ignored` |
| 3 | `\end{document} without a matching \begin: ignored` |
| 3 | `\end{tikzpicture} without a matching \begin: ignored` |
| 3 | `E-REC-N column N: unsupported text command: \begin; written as is` |

Options tikra does not know, in figures that do not match, by how many figures they appear in:

| Figures | Option |
|---:|---|
| 32 | `circuit ee IEC` |
| 20 | `resistor` |
| 18 | `folding line length` |
| 17 | `concept` |
| 16 | `mindmap` |
| 15 | `numbered faces` |
| 14 | `concept color` |
| 13 | `place` |
| 12 | `3d view` |
| 12 | `perspective` |
| 11 | `boxplot prepared` |
| 11 | `hist` |
| 9 | `state` |
| 9 | `sibling angle` |
| 9 | `symbolic x coords` |
| 8 | `grow cyclic` |
| 7 | `initial` |
| 7 | `and gate` |
| 7 | `diode` |
| 7 | `entity` |
| 7 | `spy using outlines` |
| 7 | `circle through` |
| 6 | `make snapshot of` |
| 6 | `animate` |
| 6 | `logic gate inputs` |
| 6 | `token` |
| 6 | `quiver` |
| 6 | `contour filled` |
| 5 | `tokens` |
| 5 | `intersection segments` |
| 5 | `cycle multi list` |
| 5 | `color of colormap` |
| 4 | `nand gate` |
| 4 | `or gate` |
| 4 | `circuit logic IEC` |
| 4 | `left delimiter` |
| 4 | `right delimiter` |
| 4 | `circle connection bar` |
| 4 | `children are tokens` |
| 4 | `turtle` |
| 4 | `double equal sign distance` |
| 4 | `circular drop shadow` |
| 4 | `boxplot` |
| 4 | `contour prepared` |
| 4 | `axis z line shift` |
| 4 | `index of colormap` |
| 3 | `make contact` |
| 3 | `point up` |
| 3 | `info` |
| 3 | `capacitor` |
| 3 | `logic gate inverted radius` |
| 3 | `evaluate` |
| 3 | `relationship` |
| 3 | `clockwise from` |
| 3 | `isometric view` |
| 3 | `structured tokens` |
| 3 | `circular glow` |
| 3 | `allow upside down` |
| 3 | `spy using overlays` |
| 3 | `grow via three points` |

Commands, environments and path operations tikra does not know, likewise:

| Figures | Command |
|---:|---|
| 310 | `\datavisualization` |
| 42 | `\calendar` |
| 19 | `\spy` |
| 8 | `\nextgroupplot` |
| 6 | `\pgftransformnonlinear` |
| 5 | `\pgfsetcurvilinearbeziercurve` |
| 5 | `\pgfsetfading` |
| 4 | `\pgfslopedattimetrue` |
| 3 | `\pgfplothandlerlineto` |
| 3 | `\pgfplotstableread` |
| 2 | `\pgfanimateattribute` |
| 2 | `\pgfpatharcto` |
| 2 | `\pgfsetfillpattern` |
| 2 | `\pgfpathqmoveto` |
| 2 | `\pgftransformationadjustments` |
| 2 | `pgfviewboxscope` |
| 2 | `\pgfsetfadingforcurrentpath` |
| 2 | `\pgfdeclarelindenmayersystem` |
| 2 | `\tikzmath` |
| 2 | `\pgfplothandlerybarinterval` |
| 2 | `\pgfplothandlermark` |
| 2 | `\breakforeach` |
| 2 | `\tikzgetnamedpath` |
| 2 | `\pgfplotscolormaptodatafile` |
| 1 | `\pgfmathdivide` |
| 1 | `\pgfpathcurvebetweentime` |
| 1 | `\pgfpatharctoprecomputed` |
| 1 | `\pgfpathqcircle` |
| 1 | `\pgftransformtriangle` |
| 1 | `\pgftransformarrow` |
| 1 | `\pgftransformlineattime` |
| 1 | `\pgftransformcurveattime` |
| 1 | `\pgftransforminvert` |
| 1 | `\pgflowlevelsynccm` |
| 1 | `\pgflowlevel` |
| 1 | `\pgflowlevelobj` |
| 1 | `pgflowlevelscope` |
| 1 | `\pgfapproximatenonlineartransformation` |
| 1 | `\pgfapproximatenonlineartranslation` |
| 1 | `\pgfsetblendmode` |

Errors of the figures that fail:

| Figures | Error |
|---:|---|
| 3 | `figure: missing number` |
| 1 | `figure: you can't use \pgf@xc after \advance` |
