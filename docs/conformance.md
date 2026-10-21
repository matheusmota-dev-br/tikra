# Conformance with TikZ

How far tikra is from TikZ itself: every example of the TikZ manual (pgf 3.1.11a) and of the pgfplots
manual (1.18.2), drawn by pdflatex (TeX Live 2025) and by tikra, and compared. How it works and how
to run it: [tools/conformance](../tools/conformance/README.md).

**70 of 2448 figures (2.9%) match TeX**; 539 are close, 1838 differ and 1 fail in tikra. 905 of the ones that do not match came out with no warning.

| | Figures | Match | Close | Different | Fails | Mean F1 |
|---|---:|---:|---:|---:|---:|---:|
| All | 2448 | 70 (2.9%) | 539 (22.0%) | 1838 (75.1%) | 1 (0.0%) | 0.34 |
| TikZ manual | 1810 | 68 (3.8%) | 525 (29.0%) | 1217 (67.2%) | 0 (0.0%) | 0.41 |
| pgfplots manual | 638 | 2 (0.3%) | 14 (2.2%) | 621 (97.3%) | 1 (0.2%) | 0.16 |

*Match*: same shape (F1 ≥ 0.9 on the edges, with 0.5 mm of tolerance), no missing or extra piece, the same colors and size. *Close*: F1 ≥ 0.75.
*Fails*: tikra returned an error, panicked or took more than 10 s. Mean F1 counts a failure as 0.

Not counted: 136 where tex could not compile it, 18 where tex draws nothing visible, and 535 examples that are not TikZ figures (pgf basic layer, text, tables).

## Speed

The same 192 figures (a fixed sample of the manuals), one at a time, on Darwin arm64:

| | Median per figure | p95 |
|---|---:|---:|
| pdflatex (texlive/texlive:latest) | 647 ms | 1031 ms |
| tikra, TikZ to SVG | 0.05 ms | 0.34 ms |

Median ratio: tikra is **11544× faster**. pdflatex time includes starting TeX and loading the libraries of the manual, which is what a user
waits for; tikra's is the conversion inside a running process (the command line adds a few ms to start).

## By chapter

Sorted by how many figures do not match, the work left.

| Chapter | Figures | Match | Close | Different | Fails | Mean F1 |
|---|---:|---:|---:|---:|---:|---:|
| Specifying Graphs <sub>tikz</sub> | 159 | 0 (0.0%) | 2 (1.3%) | 157 (98.7%) | 0 (0.0%) | 0.02 |
| Nodes and Edges <sub>tikz</sub> | 118 | 5 (4.2%) | 63 (53.4%) | 50 (42.4%) | 0 (0.0%) | 0.68 |
| Arrows <sub>tikz</sub> | 104 | 2 (1.9%) | 67 (64.4%) | 35 (33.7%) | 0 (0.0%) | 0.77 |
| Shape Library <sub>tikz</sub> | 97 | 0 (0.0%) | 13 (13.4%) | 84 (86.6%) | 0 (0.0%) | 0.37 |
| Style Sheets and Legends <sub>tikz</sub> | 85 | 0 (0.0%) | 0 (0.0%) | 85 (100.0%) | 0 (0.0%) | 0.00 |
| Three Dimensional Plot Types <sub>pgfplots</sub> | 85 | 0 (0.0%) | 0 (0.0%) | 85 (100.0%) | 0 (0.0%) | 0.11 |
| Axes <sub>tikz</sub> | 78 | 0 (0.0%) | 1 (1.3%) | 77 (98.7%) | 0 (0.0%) | 0.01 |
| Axis Descriptions <sub>pgfplots</sub> | 77 | 0 (0.0%) | 0 (0.0%) | 77 (100.0%) | 0 (0.0%) | 0.17 |
| Decoration Library <sub>tikz</sub> | 81 | 6 (7.4%) | 31 (38.3%) | 44 (54.3%) | 0 (0.0%) | 0.53 |
| Two Dimensional Plot Types <sub>pgfplots</sub> | 72 | 0 (0.0%) | 0 (0.0%) | 72 (100.0%) | 0 (0.0%) | 0.14 |
| Circuit Libraries <sub>tikz</sub> | 64 | 0 (0.0%) | 9 (14.1%) | 55 (85.9%) | 0 (0.0%) | 0.30 |
| Specifying Coordinates <sub>tikz</sub> | 66 | 3 (4.5%) | 47 (71.2%) | 16 (24.2%) | 0 (0.0%) | 0.71 |
| Actions on Paths <sub>tikz</sub> | 72 | 18 (25.0%) | 36 (50.0%) | 18 (25.0%) | 0 (0.0%) | 0.81 |
| Syntax for Path Specifications <sub>tikz</sub> | 52 | 2 (3.8%) | 34 (65.4%) | 16 (30.8%) | 0 (0.0%) | 0.71 |
| Markers, Linestyles, (Background) Colors and Colormaps <sub>pgfplots</sub> | 50 | 1 (2.0%) | 10 (20.0%) | 39 (78.0%) | 0 (0.0%) | 0.41 |
| Tutorial: A Picture for Karl's Students <sub>tikz</sub> | 46 | 0 (0.0%) | 24 (52.2%) | 22 (47.8%) | 0 (0.0%) | 0.55 |
| Transparency <sub>tikz</sub> | 46 | 5 (10.9%) | 20 (43.5%) | 21 (45.7%) | 0 (0.0%) | 0.68 |
| Calendar Library <sub>tikz</sub> | 41 | 0 (0.0%) | 0 (0.0%) | 41 (100.0%) | 0 (0.0%) | 0.01 |
| Coordinate, Canvas, and Nonlinear Transformations <sub>tikz</sub> | 38 | 3 (7.9%) | 24 (63.2%) | 11 (28.9%) | 0 (0.0%) | 0.78 |
| Tick Options <sub>pgfplots</sub> | 32 | 0 (0.0%) | 0 (0.0%) | 32 (100.0%) | 0 (0.0%) | 0.16 |
| Plots of Functions <sub>tikz</sub> | 30 | 1 (3.3%) | 1 (3.3%) | 28 (93.3%) | 0 (0.0%) | 0.22 |
| Patchplots Library <sub>pgfplots</sub> | 29 | 0 (0.0%) | 0 (0.0%) | 29 (100.0%) | 0 (0.0%) | 0.17 |
| Decorated Paths <sub>tikz</sub> | 31 | 3 (9.7%) | 14 (45.2%) | 14 (45.2%) | 0 (0.0%) | 0.67 |
| Fill between <sub>pgfplots</sub> | 29 | 1 (3.4%) | 3 (10.3%) | 25 (86.2%) | 0 (0.0%) | 0.29 |
| Making Trees Grow <sub>tikz</sub> | 27 | 0 (0.0%) | 2 (7.4%) | 25 (92.6%) | 0 (0.0%) | 0.25 |
| Matrices and Alignment <sub>tikz</sub> | 26 | 0 (0.0%) | 1 (3.8%) | 25 (96.2%) | 0 (0.0%) | 0.05 |
| Statistics <sub>pgfplots</sub> | 26 | 0 (0.0%) | 0 (0.0%) | 26 (100.0%) | 0 (0.0%) | 0.15 |
| Animations <sub>tikz</sub> | 24 | 0 (0.0%) | 5 (20.8%) | 19 (79.2%) | 0 (0.0%) | 0.27 |
| Plot Handler Library <sub>tikz</sub> | 23 | 0 (0.0%) | 1 (4.3%) | 22 (95.7%) | 0 (0.0%) | 0.42 |
| Transformations <sub>tikz</sub> | 22 | 0 (0.0%) | 19 (86.4%) | 3 (13.6%) | 0 (0.0%) | 0.80 |
| Scaling Options <sub>pgfplots</sub> | 22 | 0 (0.0%) | 0 (0.0%) | 22 (100.0%) | 0 (0.0%) | 0.14 |
| The Axis Environments <sub>pgfplots</sub> | 21 | 0 (0.0%) | 0 (0.0%) | 21 (100.0%) | 0 (0.0%) | 0.16 |
| Tutorial: Diagrams as Simple Graphs <sub>tikz</sub> | 19 | 0 (0.0%) | 7 (36.8%) | 12 (63.2%) | 0 (0.0%) | 0.49 |
| Visualizers <sub>tikz</sub> | 18 | 0 (0.0%) | 0 (0.0%) | 18 (100.0%) | 0 (0.0%) | 0.00 |
| Paper-Folding Diagrams Library <sub>tikz</sub> | 18 | 0 (0.0%) | 0 (0.0%) | 18 (100.0%) | 0 (0.0%) | 0.00 |
| Grid Options <sub>pgfplots</sub> | 18 | 0 (0.0%) | 0 (0.0%) | 18 (100.0%) | 0 (0.0%) | 0.15 |
| Constructing Paths <sub>tikz</sub> | 16 | 0 (0.0%) | 11 (68.8%) | 5 (31.2%) | 0 (0.0%) | 0.71 |
| Mindmap Drawing Library <sub>tikz</sub> | 16 | 1 (6.2%) | 2 (12.5%) | 13 (81.2%) | 0 (0.0%) | 0.26 |
| Three Point Perspective Drawing Library <sub>tikz</sub> | 15 | 0 (0.0%) | 0 (0.0%) | 15 (100.0%) | 0 (0.0%) | 0.09 |
| Chains <sub>tikz</sub> | 14 | 0 (0.0%) | 1 (7.1%) | 13 (92.9%) | 0 (0.0%) | 0.40 |
| Matrix Library <sub>tikz</sub> | 14 | 0 (0.0%) | 0 (0.0%) | 14 (100.0%) | 0 (0.0%) | 0.05 |
| Tutorial: Euclid's Amber Version of the Elements <sub>tikz</sub> | 14 | 0 (0.0%) | 4 (28.6%) | 10 (71.4%) | 0 (0.0%) | 0.51 |
| Tutorial: A Petri-Net for Hagen <sub>tikz</sub> | 28 | 14 (50.0%) | 8 (28.6%) | 6 (21.4%) | 0 (0.0%) | 0.89 |
| Creating Data Visualizations <sub>tikz</sub> | 13 | 0 (0.0%) | 0 (0.0%) | 13 (100.0%) | 0 (0.0%) | 0.00 |
| Petri-Net Drawing Library <sub>tikz</sub> | 13 | 0 (0.0%) | 0 (0.0%) | 13 (100.0%) | 0 (0.0%) | 0.18 |
| Shadows Library <sub>tikz</sub> | 14 | 1 (7.1%) | 10 (71.4%) | 3 (21.4%) | 0 (0.0%) | 0.85 |
| Repeating Things: The Foreach Statement <sub>tikz</sub> | 13 | 0 (0.0%) | 7 (53.8%) | 6 (46.2%) | 0 (0.0%) | 0.60 |
| Pics: Small Pictures on Paths <sub>tikz</sub> | 14 | 1 (7.1%) | 2 (14.3%) | 11 (78.6%) | 0 (0.0%) | 0.20 |
| pgfmanual-en-tikz-scopes <sub>tikz</sub> | 13 | 0 (0.0%) | 2 (15.4%) | 11 (84.6%) | 0 (0.0%) | 0.42 |
| Polar Axes <sub>pgfplots</sub> | 13 | 0 (0.0%) | 0 (0.0%) | 13 (100.0%) | 0 (0.0%) | 0.00 |
| Ternary Diagrams <sub>pgfplots</sub> | 13 | 0 (0.0%) | 0 (0.0%) | 13 (100.0%) | 0 (0.0%) | 0.01 |
| 3D Axis Configuration <sub>pgfplots</sub> | 13 | 0 (0.0%) | 0 (0.0%) | 13 (100.0%) | 0 (0.0%) | 0.11 |
| Specifying the Plotted Range <sub>pgfplots</sub> | 13 | 0 (0.0%) | 0 (0.0%) | 13 (100.0%) | 0 (0.0%) | 0.13 |
| Background Library <sub>tikz</sub> | 12 | 0 (0.0%) | 1 (8.3%) | 11 (91.7%) | 0 (0.0%) | 0.52 |
| To Path Library <sub>tikz</sub> | 13 | 1 (7.7%) | 3 (23.1%) | 9 (69.2%) | 0 (0.0%) | 0.52 |
| Smith Charts <sub>pgfplots</sub> | 12 | 0 (0.0%) | 0 (0.0%) | 12 (100.0%) | 0 (0.0%) | 0.00 |
| Polar Axes <sub>tikz</sub> | 11 | 0 (0.0%) | 0 (0.0%) | 11 (100.0%) | 0 (0.0%) | 0.00 |
| Shadings Library <sub>tikz</sub> | 11 | 0 (0.0%) | 10 (90.9%) | 1 (9.1%) | 0 (0.0%) | 0.88 |
| Spy Library: Magnifying Parts of Pictures <sub>tikz</sub> | 11 | 0 (0.0%) | 0 (0.0%) | 11 (100.0%) | 0 (0.0%) | 0.07 |
| Decorations <sub>tikz</sub> | 10 | 0 (0.0%) | 5 (50.0%) | 5 (50.0%) | 0 (0.0%) | 0.45 |
| pgfmanual-en-base-scopes <sub>tikz</sub> | 11 | 1 (9.1%) | 10 (90.9%) | 0 (0.0%) | 0 (0.0%) | 0.83 |
| Tutorial: A Lecture Map for Johannes <sub>tikz</sub> | 10 | 0 (0.0%) | 0 (0.0%) | 10 (100.0%) | 0 (0.0%) | 0.06 |
| Error Bars <sub>pgfplots</sub> | 9 | 0 (0.0%) | 0 (0.0%) | 9 (100.0%) | 0 (0.0%) | 0.14 |
| Nodes and Shapes <sub>tikz</sub> | 8 | 0 (0.0%) | 4 (50.0%) | 4 (50.0%) | 0 (0.0%) | 0.69 |
| Providing Data for a Data Visualization <sub>tikz</sub> | 8 | 0 (0.0%) | 0 (0.0%) | 8 (100.0%) | 0 (0.0%) | 0.00 |
| Automata Drawing Library <sub>tikz</sub> | 8 | 0 (0.0%) | 0 (0.0%) | 8 (100.0%) | 0 (0.0%) | 0.23 |
| Pattern Library <sub>tikz</sub> | 8 | 0 (0.0%) | 1 (12.5%) | 7 (87.5%) | 0 (0.0%) | 0.10 |
| Tutorials and Guidelines <sub>tikz</sub> | 9 | 1 (11.1%) | 2 (22.2%) | 6 (66.7%) | 0 (0.0%) | 0.51 |
| Grouping plots <sub>pgfplots</sub> | 8 | 0 (0.0%) | 0 (0.0%) | 8 (100.0%) | 0 (0.0%) | 0.01 |
| Alignment Options <sub>pgfplots</sub> | 8 | 0 (0.0%) | 0 (0.0%) | 8 (100.0%) | 0 (0.0%) | 0.16 |
| Matrices <sub>tikz</sub> | 7 | 0 (0.0%) | 0 (0.0%) | 7 (100.0%) | 0 (0.0%) | 0.00 |
| Entity-Relationship Diagram Drawing Library <sub>tikz</sub> | 7 | 0 (0.0%) | 0 (0.0%) | 7 (100.0%) | 0 (0.0%) | 0.33 |
| Tree Library <sub>tikz</sub> | 7 | 0 (0.0%) | 0 (0.0%) | 7 (100.0%) | 0 (0.0%) | 0.24 |
| Utilities and Basic Level Commands <sub>pgfplots</sub> | 7 | 0 (0.0%) | 0 (0.0%) | 7 (100.0%) | 0 (0.0%) | 0.16 |
| Skipping Or Changing Coordinates -- Filters <sub>pgfplots</sub> | 7 | 0 (0.0%) | 0 (0.0%) | 7 (100.0%) | 0 (0.0%) | 0.12 |
| Miscellaneous Options <sub>pgfplots</sub> | 7 | 0 (0.0%) | 0 (0.0%) | 7 (100.0%) | 0 (0.0%) | 0.12 |
| Solving a Real Use Case: Scientific Data Analysis <sub>pgfplots</sub> | 7 | 0 (0.0%) | 0 (0.0%) | 7 (100.0%) | 0 (0.0%) | 0.22 |
| Angle Library <sub>tikz</sub> | 6 | 0 (0.0%) | 4 (66.7%) | 2 (33.3%) | 0 (0.0%) | 0.72 |
| Lindenmayer System Drawing Library <sub>tikz</sub> | 6 | 0 (0.0%) | 1 (16.7%) | 5 (83.3%) | 0 (0.0%) | 0.15 |
| Math Library <sub>tikz</sub> | 6 | 0 (0.0%) | 1 (16.7%) | 5 (83.3%) | 0 (0.0%) | 0.13 |
| Design Principles <sub>tikz</sub> | 6 | 0 (0.0%) | 3 (50.0%) | 3 (50.0%) | 0 (0.0%) | 0.53 |
| pgfplots.reference.closingplots <sub>pgfplots</sub> | 6 | 0 (0.0%) | 0 (0.0%) | 6 (100.0%) | 0 (0.0%) | 0.14 |
| Number Formatting Options <sub>pgfplots</sub> | 6 | 0 (0.0%) | 0 (0.0%) | 6 (100.0%) | 0 (0.0%) | 0.23 |
| Solving a Real Use Case: Function Visualization <sub>pgfplots</sub> | 6 | 0 (0.0%) | 0 (0.0%) | 6 (100.0%) | 0 (0.0%) | 0.10 |
| Three Dimensional Drawing Library <sub>tikz</sub> | 5 | 0 (0.0%) | 1 (20.0%) | 4 (80.0%) | 0 (0.0%) | 0.24 |
| Key Management <sub>tikz</sub> | 5 | 0 (0.0%) | 1 (20.0%) | 4 (80.0%) | 0 (0.0%) | 0.36 |
| The Picture's Size: Bounding Box and Clipping <sub>pgfplots</sub> | 5 | 0 (0.0%) | 0 (0.0%) | 5 (100.0%) | 0 (0.0%) | 0.13 |
| Fitting Lines -- Regression <sub>pgfplots</sub> | 5 | 0 (0.0%) | 0 (0.0%) | 5 (100.0%) | 0 (0.0%) | 0.18 |
| Symbolic Coordinates and User Transformations <sub>pgfplots</sub> | 5 | 0 (0.0%) | 0 (0.0%) | 4 (80.0%) | 1 (20.0%) | 0.06 |
| Use Cases involving Scatter Plots <sub>pgfplots</sub> | 5 | 0 (0.0%) | 0 (0.0%) | 5 (100.0%) | 0 (0.0%) | 0.13 |
| Solving a Real Use Case: Functions of Two Variables <sub>pgfplots</sub> | 5 | 0 (0.0%) | 0 (0.0%) | 5 (100.0%) | 0 (0.0%) | 0.09 |
| Patterns <sub>tikz</sub> | 4 | 0 (0.0%) | 1 (25.0%) | 3 (75.0%) | 0 (0.0%) | 0.38 |
| Fitting Library <sub>tikz</sub> | 4 | 0 (0.0%) | 1 (25.0%) | 3 (75.0%) | 0 (0.0%) | 0.45 |
| Turtle Graphics Library <sub>tikz</sub> | 4 | 0 (0.0%) | 0 (0.0%) | 4 (100.0%) | 0 (0.0%) | 0.00 |
| Views Library <sub>tikz</sub> | 4 | 0 (0.0%) | 2 (50.0%) | 2 (50.0%) | 0 (0.0%) | 0.72 |
| Transforming Coordinate Systems <sub>pgfplots</sub> | 4 | 0 (0.0%) | 0 (0.0%) | 4 (100.0%) | 0 (0.0%) | 0.09 |
| Memory and Speed considerations <sub>pgfplots</sub> | 4 | 0 (0.0%) | 0 (0.0%) | 4 (100.0%) | 0 (0.0%) | 0.06 |
| Creating Plots <sub>tikz</sub> | 3 | 0 (0.0%) | 0 (0.0%) | 3 (100.0%) | 0 (0.0%) | 0.13 |
| Quick Commands <sub>tikz</sub> | 3 | 0 (0.0%) | 2 (66.7%) | 1 (33.3%) | 0 (0.0%) | 0.76 |
| pgfmanual-en-dv-examples <sub>tikz</sub> | 3 | 0 (0.0%) | 0 (0.0%) | 3 (100.0%) | 0 (0.0%) | 0.00 |
| Mathematical Expressions <sub>tikz</sub> | 3 | 0 (0.0%) | 0 (0.0%) | 3 (100.0%) | 0 (0.0%) | 0.56 |
| Interoperability <sub>pgfplots</sub> | 3 | 0 (0.0%) | 0 (0.0%) | 3 (100.0%) | 0 (0.0%) | 0.13 |
| Shadings <sub>tikz</sub> | 2 | 0 (0.0%) | 0 (0.0%) | 2 (100.0%) | 0 (0.0%) | 0.07 |
| Introduction to Data Visualization <sub>tikz</sub> | 2 | 0 (0.0%) | 0 (0.0%) | 2 (100.0%) | 0 (0.0%) | 0.00 |
| Babel Library <sub>tikz</sub> | 2 | 0 (0.0%) | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0.95 |
| Resource Description Framework Library <sub>tikz</sub> | 2 | 0 (0.0%) | 0 (0.0%) | 2 (100.0%) | 0 (0.0%) | 0.00 |
| Customizing the Mathematical Engine <sub>tikz</sub> | 2 | 0 (0.0%) | 0 (0.0%) | 2 (100.0%) | 0 (0.0%) | 0.63 |
| Units in Labels <sub>pgfplots</sub> | 2 | 0 (0.0%) | 0 (0.0%) | 2 (100.0%) | 0 (0.0%) | 0.18 |
| Style Options <sub>pgfplots</sub> | 2 | 0 (0.0%) | 0 (0.0%) | 2 (100.0%) | 0 (0.0%) | 0.16 |
| Layered Graphics <sub>tikz</sub> | 1 | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0.89 |
| Through Library <sub>tikz</sub> | 1 | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0.81 |
| Additional Mathematical Commands <sub>tikz</sub> | 1 | 0 (0.0%) | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0.00 |
| Date and Calendar Utility Macros <sub>tikz</sub> | 1 | 0 (0.0%) | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0.00 |
| Animation System Layer <sub>tikz</sub> | 1 | 0 (0.0%) | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0.00 |
| Decoration: Soft Clipping <sub>pgfplots</sub> | 1 | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0.94 |
| Components <sub>pgfplots</sub> | 1 | 0 (0.0%) | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0.09 |

## What tikra reported

Warnings, by how many figures they appear in (the varying part replaced by …):

| Figures | Warning |
|---:|---|
| 254 | `plot without data: drew the default window from N to N` |
| 219 | `\datavisualization not supported: … ignored` |
| 169 | `unrecognized path fragment: …` |
| 168 | `\graph not supported: … ignored` |
| 133 | `text outside a statement ignored` |
| 60 | `\draw: figure: unknown coordinate: (…); statement ignored` |
| 51 | `\matrix is not supported: ignored` |
| 50 | `\tikzdatavisualizationset not supported: … ignored` |
| 48 | `\tikz not supported: … ignored` |
| 40 | `\calendar not supported: … ignored` |
| 36 | `more than one tikzpicture: only the first was drawn` |
| 35 | `\Huge not supported: … ignored` |
| 35 | `\addplot not supported: … ignored` |
| 29 | `E-REC-N column N: unsupported text command: \vrule; written as is` |
| 25 | `\pic is not supported: ignored` |
| 18 | `E-REC-N the font has no … (…): drew a box` |
| 17 | `E-REC-N column N: unsupported text command: \nodepart; written as is` |
| 16 | `\usepgfmodule not supported: … ignored` |
| 16 | `\wall not supported: … ignored` |
| 14 | `\pgfpathmoveto not supported: … ignored` |
| 14 | `\draw: figure: plot without an expression; statement ignored` |
| 14 | `environment polaraxis not supported: content read as statements` |
| 13 | `\pgftext not supported: … ignored` |
| 13 | `environment ternaryaxis not supported: content read as statements` |
| 12 | `\spy not supported: … ignored` |
| 12 | `\draw: figure: controls without a point; statement ignored` |
| 12 | `environment smithchart not supported: content read as statements` |
| 11 | `\pgfpathcircle not supported: … ignored` |
| 11 | `\path: figure: unknown coordinate: (…); statement ignored` |
| 11 | `\draw: figure: rectangle without a corner; statement ignored` |
| 11 | `\usepgflibrary not supported: … ignored` |
| 10 | `\simplecuboid not supported: … ignored` |
| 9 | `\pgfmathsetseed not supported: … ignored` |
| 8 | `\node: figure: unknown coordinate: (…); statement ignored` |
| 8 | `\vbox not supported: … ignored` |
| 8 | `\nextgroupplot not supported: … ignored` |
| 8 | `environment groupplot not supported: content read as statements` |
| 7 | `\pgfsetlinewidth not supported: … ignored` |
| 7 | `\path: figure: controls without a point; statement ignored` |
| 7 | `\useasboundingbox not supported: … ignored` |

Errors of the figures that fail:

| Figures | Error |
|---:|---|
| 1 | `png: image too large: N × N px at N dpi (…)` |
