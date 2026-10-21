# Conformance with TikZ

How far tikra is from TikZ itself: every example of the TikZ manual (pgf 3.1.11a) and of the pgfplots
manual (1.18.2), drawn by pdflatex (TeX Live 2025) and by tikra, and compared. How it works and how
to run it: [tools/conformance](../tools/conformance/README.md).

**1392 of 2502 figures (55.6%) match TeX**; 762 are close, 343 differ and 5 fail in tikra. 787 of the ones that do not match came out with no warning.

| | Figures | Match | Close | Different | Fails | Mean F1 |
|---|---:|---:|---:|---:|---:|---:|
| All | 2502 | 1392 (55.6%) | 762 (30.5%) | 343 (13.7%) | 5 (0.2%) | 0.89 |
| TikZ manual | 1864 | 1219 (65.4%) | 439 (23.6%) | 201 (10.8%) | 5 (0.3%) | 0.91 |
| pgfplots manual | 638 | 173 (27.1%) | 323 (50.6%) | 142 (22.3%) | 0 (0.0%) | 0.84 |

*Match*: same shape (F1 ≥ 0.9 on the edges, with 0.5 mm of tolerance), no missing or extra piece, the same colors and size. *Close*: F1 ≥ 0.75.
*Fails*: tikra returned an error, panicked or took more than 10 s. Mean F1 counts a failure as 0.

Not counted: 137 where tex could not compile it, 19 where tex draws nothing visible, and 479 examples that are not figures (text, tables, key listings).

## Speed

The same 192 figures (a fixed sample of the manuals), one at a time, on Darwin arm64:

| | Median per figure | p95 |
|---|---:|---:|
| pdflatex (texlive/texlive:latest) | 647 ms | 1031 ms |
| tikra, TikZ to SVG | 0.77 ms | 8.79 ms |

Median ratio: tikra is **880× faster**. pdflatex time includes starting TeX and loading the libraries of the manual, which is what a user
waits for; tikra's is the conversion inside a running process: the first figure of a process also builds the engine, 16.0 ms more.

## By chapter

Sorted by how many figures do not match, the work left.

| Chapter | Figures | Match | Close | Different | Fails | Mean F1 |
|---|---:|---:|---:|---:|---:|---:|
| Axis Descriptions <sub>pgfplots</sub> | 77 | 13 (16.9%) | 47 (61.0%) | 17 (22.1%) | 0 (0.0%) | 0.86 |
| Two Dimensional Plot Types <sub>pgfplots</sub> | 72 | 18 (25.0%) | 40 (55.6%) | 14 (19.4%) | 0 (0.0%) | 0.86 |
| Shape Library <sub>tikz</sub> | 97 | 46 (47.4%) | 19 (19.6%) | 32 (33.0%) | 0 (0.0%) | 0.83 |
| Nodes and Edges <sub>tikz</sub> | 118 | 73 (61.9%) | 31 (26.3%) | 14 (11.9%) | 0 (0.0%) | 0.92 |
| Three Dimensional Plot Types <sub>pgfplots</sub> | 85 | 40 (47.1%) | 28 (32.9%) | 17 (20.0%) | 0 (0.0%) | 0.87 |
| Markers, Linestyles, (Background) Colors and Colormaps <sub>pgfplots</sub> | 50 | 17 (34.0%) | 25 (50.0%) | 8 (16.0%) | 0 (0.0%) | 0.89 |
| Specifying Graphs <sub>tikz</sub> | 159 | 128 (80.5%) | 30 (18.9%) | 1 (0.6%) | 0 (0.0%) | 0.99 |
| Coordinate, Canvas, and Nonlinear Transformations <sub>tikz</sub> | 38 | 8 (21.1%) | 20 (52.6%) | 10 (26.3%) | 0 (0.0%) | 0.80 |
| Decoration Library <sub>tikz</sub> | 81 | 51 (63.0%) | 25 (30.9%) | 5 (6.2%) | 0 (0.0%) | 0.92 |
| Specifying Coordinates <sub>tikz</sub> | 68 | 39 (57.4%) | 26 (38.2%) | 3 (4.4%) | 0 (0.0%) | 0.91 |
| Circuit Libraries <sub>tikz</sub> | 64 | 36 (56.2%) | 22 (34.4%) | 6 (9.4%) | 0 (0.0%) | 0.93 |
| Patchplots Library <sub>pgfplots</sub> | 29 | 4 (13.8%) | 19 (65.5%) | 6 (20.7%) | 0 (0.0%) | 0.81 |
| Tick Options <sub>pgfplots</sub> | 32 | 8 (25.0%) | 22 (68.8%) | 2 (6.2%) | 0 (0.0%) | 0.95 |
| Axes <sub>tikz</sub> | 78 | 55 (70.5%) | 22 (28.2%) | 1 (1.3%) | 0 (0.0%) | 0.97 |
| Arrows <sub>tikz</sub> | 104 | 81 (77.9%) | 23 (22.1%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Style Sheets and Legends <sub>tikz</sub> | 85 | 63 (74.1%) | 22 (25.9%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Fill between <sub>pgfplots</sub> | 29 | 8 (27.6%) | 20 (69.0%) | 1 (3.4%) | 0 (0.0%) | 0.95 |
| Transparency <sub>tikz</sub> | 47 | 28 (59.6%) | 9 (19.1%) | 10 (21.3%) | 0 (0.0%) | 0.85 |
| Statistics <sub>pgfplots</sub> | 26 | 7 (26.9%) | 16 (61.5%) | 3 (11.5%) | 0 (0.0%) | 0.95 |
| Actions on Paths <sub>tikz</sub> | 72 | 54 (75.0%) | 15 (20.8%) | 3 (4.2%) | 0 (0.0%) | 0.97 |
| Syntax for Path Specifications <sub>tikz</sub> | 52 | 34 (65.4%) | 11 (21.2%) | 7 (13.5%) | 0 (0.0%) | 0.89 |
| The Axis Environments <sub>pgfplots</sub> | 21 | 4 (19.0%) | 11 (52.4%) | 6 (28.6%) | 0 (0.0%) | 0.81 |
| Matrices and Alignment <sub>tikz</sub> | 26 | 10 (38.5%) | 10 (38.5%) | 6 (23.1%) | 0 (0.0%) | 0.87 |
| Tutorial: A Picture for Karl's Students <sub>tikz</sub> | 46 | 31 (67.4%) | 14 (30.4%) | 1 (2.2%) | 0 (0.0%) | 0.95 |
| Scaling Options <sub>pgfplots</sub> | 22 | 8 (36.4%) | 7 (31.8%) | 7 (31.8%) | 0 (0.0%) | 0.83 |
| Tutorial: Diagrams as Simple Graphs <sub>tikz</sub> | 19 | 6 (31.6%) | 12 (63.2%) | 1 (5.3%) | 0 (0.0%) | 0.89 |
| Polar Axes <sub>pgfplots</sub> | 13 | 0 (0.0%) | 4 (30.8%) | 9 (69.2%) | 0 (0.0%) | 0.70 |
| Ternary Diagrams <sub>pgfplots</sub> | 13 | 0 (0.0%) | 0 (0.0%) | 13 (100.0%) | 0 (0.0%) | 0.29 |
| Grid Options <sub>pgfplots</sub> | 18 | 5 (27.8%) | 11 (61.1%) | 2 (11.1%) | 0 (0.0%) | 0.89 |
| Smith Charts <sub>pgfplots</sub> | 12 | 0 (0.0%) | 0 (0.0%) | 12 (100.0%) | 0 (0.0%) | 0.20 |
| Animations <sub>tikz</sub> | 24 | 13 (54.2%) | 7 (29.2%) | 4 (16.7%) | 0 (0.0%) | 0.89 |
| Nodes and Shapes <sub>tikz</sub> | 12 | 1 (8.3%) | 4 (33.3%) | 4 (33.3%) | 3 (25.0%) | 0.57 |
| Chains <sub>tikz</sub> | 14 | 3 (21.4%) | 8 (57.1%) | 3 (21.4%) | 0 (0.0%) | 0.84 |
| Constructing Paths <sub>tikz</sub> | 26 | 16 (61.5%) | 7 (26.9%) | 3 (11.5%) | 0 (0.0%) | 0.84 |
| Plot Handler Library <sub>tikz</sub> | 23 | 13 (56.5%) | 2 (8.7%) | 8 (34.8%) | 0 (0.0%) | 0.73 |
| Transformations <sub>tikz</sub> | 22 | 13 (59.1%) | 8 (36.4%) | 1 (4.5%) | 0 (0.0%) | 0.92 |
| Error Bars <sub>pgfplots</sub> | 9 | 0 (0.0%) | 7 (77.8%) | 2 (22.2%) | 0 (0.0%) | 0.78 |
| Matrix Library <sub>tikz</sub> | 14 | 6 (42.9%) | 5 (35.7%) | 3 (21.4%) | 0 (0.0%) | 0.86 |
| Pattern Library <sub>tikz</sub> | 8 | 0 (0.0%) | 5 (62.5%) | 3 (37.5%) | 0 (0.0%) | 0.84 |
| Plots of Functions <sub>tikz</sub> | 30 | 22 (73.3%) | 6 (20.0%) | 2 (6.7%) | 0 (0.0%) | 0.95 |
| Specifying the Plotted Range <sub>pgfplots</sub> | 13 | 5 (38.5%) | 6 (46.2%) | 2 (15.4%) | 0 (0.0%) | 0.91 |
| pgfmanual-en-base-scopes <sub>tikz</sub> | 19 | 12 (63.2%) | 4 (21.1%) | 3 (15.8%) | 0 (0.0%) | 0.87 |
| Calendar Library <sub>tikz</sub> | 41 | 34 (82.9%) | 7 (17.1%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Tutorial: A Lecture Map for Johannes <sub>tikz</sub> | 10 | 3 (30.0%) | 3 (30.0%) | 4 (40.0%) | 0 (0.0%) | 0.69 |
| Utilities and Basic Level Commands <sub>pgfplots</sub> | 7 | 0 (0.0%) | 7 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0.92 |
| Alignment Options <sub>pgfplots</sub> | 8 | 1 (12.5%) | 5 (62.5%) | 2 (25.0%) | 0 (0.0%) | 0.77 |
| Skipping Or Changing Coordinates -- Filters <sub>pgfplots</sub> | 7 | 0 (0.0%) | 7 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0.91 |
| Creating Plots <sub>tikz</sub> | 7 | 1 (14.3%) | 0 (0.0%) | 6 (85.7%) | 0 (0.0%) | 0.32 |
| Creating Data Visualizations <sub>tikz</sub> | 13 | 7 (53.8%) | 5 (38.5%) | 1 (7.7%) | 0 (0.0%) | 0.96 |
| Background Library <sub>tikz</sub> | 12 | 6 (50.0%) | 4 (33.3%) | 2 (16.7%) | 0 (0.0%) | 0.87 |
| Shadings Library <sub>tikz</sub> | 11 | 5 (45.5%) | 0 (0.0%) | 6 (54.5%) | 0 (0.0%) | 0.72 |
| Pics: Small Pictures on Paths <sub>tikz</sub> | 14 | 8 (57.1%) | 2 (14.3%) | 4 (28.6%) | 0 (0.0%) | 0.78 |
| Solving a Real Use Case: Function Visualization <sub>pgfplots</sub> | 6 | 0 (0.0%) | 2 (33.3%) | 4 (66.7%) | 0 (0.0%) | 0.78 |
| Matrices <sub>tikz</sub> | 7 | 2 (28.6%) | 3 (42.9%) | 2 (28.6%) | 0 (0.0%) | 0.69 |
| Shadings <sub>tikz</sub> | 6 | 1 (16.7%) | 1 (16.7%) | 4 (66.7%) | 0 (0.0%) | 0.47 |
| Making Trees Grow <sub>tikz</sub> | 27 | 22 (81.5%) | 4 (14.8%) | 1 (3.7%) | 0 (0.0%) | 0.98 |
| Grouping plots <sub>pgfplots</sub> | 8 | 3 (37.5%) | 4 (50.0%) | 1 (12.5%) | 0 (0.0%) | 0.96 |
| The Picture's Size: Bounding Box and Clipping <sub>pgfplots</sub> | 5 | 0 (0.0%) | 1 (20.0%) | 4 (80.0%) | 0 (0.0%) | 0.41 |
| Fitting Lines -- Regression <sub>pgfplots</sub> | 5 | 0 (0.0%) | 1 (20.0%) | 4 (80.0%) | 0 (0.0%) | 0.71 |
| Symbolic Coordinates and User Transformations <sub>pgfplots</sub> | 5 | 0 (0.0%) | 1 (20.0%) | 4 (80.0%) | 0 (0.0%) | 0.54 |
| Using Paths <sub>tikz</sub> | 15 | 11 (73.3%) | 2 (13.3%) | 2 (13.3%) | 0 (0.0%) | 0.87 |
| Patterns <sub>tikz</sub> | 4 | 0 (0.0%) | 0 (0.0%) | 4 (100.0%) | 0 (0.0%) | 0.13 |
| Math Library <sub>tikz</sub> | 6 | 2 (33.3%) | 2 (33.3%) | 2 (33.3%) | 0 (0.0%) | 0.75 |
| Views Library <sub>tikz</sub> | 4 | 0 (0.0%) | 2 (50.0%) | 2 (50.0%) | 0 (0.0%) | 0.71 |
| Tutorials and Guidelines <sub>tikz</sub> | 9 | 5 (55.6%) | 1 (11.1%) | 3 (33.3%) | 0 (0.0%) | 0.75 |
| Miscellaneous Options <sub>pgfplots</sub> | 7 | 3 (42.9%) | 4 (57.1%) | 0 (0.0%) | 0 (0.0%) | 0.92 |
| Use Cases involving Scatter Plots <sub>pgfplots</sub> | 5 | 1 (20.0%) | 4 (80.0%) | 0 (0.0%) | 0 (0.0%) | 0.94 |
| Solving a Real Use Case: Functions of Two Variables <sub>pgfplots</sub> | 5 | 1 (20.0%) | 3 (60.0%) | 1 (20.0%) | 0 (0.0%) | 0.82 |
| Quick Commands <sub>tikz</sub> | 3 | 0 (0.0%) | 2 (66.7%) | 1 (33.3%) | 0 (0.0%) | 0.76 |
| Providing Data for a Data Visualization <sub>tikz</sub> | 8 | 5 (62.5%) | 2 (25.0%) | 1 (12.5%) | 0 (0.0%) | 0.96 |
| Tree Library <sub>tikz</sub> | 7 | 4 (57.1%) | 3 (42.9%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Additional Mathematical Commands <sub>tikz</sub> | 3 | 0 (0.0%) | 0 (0.0%) | 3 (100.0%) | 0 (0.0%) | 0.02 |
| Mathematical Expressions <sub>tikz</sub> | 3 | 0 (0.0%) | 1 (33.3%) | 2 (66.7%) | 0 (0.0%) | 0.59 |
| Repeating Things: The Foreach Statement <sub>tikz</sub> | 13 | 10 (76.9%) | 1 (7.7%) | 2 (15.4%) | 0 (0.0%) | 0.84 |
| Decorated Paths <sub>tikz</sub> | 31 | 28 (90.3%) | 3 (9.7%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| pgfmanual-en-tikz-scopes <sub>tikz</sub> | 13 | 10 (76.9%) | 3 (23.1%) | 0 (0.0%) | 0 (0.0%) | 0.97 |
| Tutorial: Euclid's Amber Version of the Elements <sub>tikz</sub> | 14 | 11 (78.6%) | 1 (7.1%) | 2 (14.3%) | 0 (0.0%) | 0.89 |
| Tutorial: A Petri-Net for Hagen <sub>tikz</sub> | 28 | 25 (89.3%) | 1 (3.6%) | 2 (7.1%) | 0 (0.0%) | 0.97 |
| 3D Axis Configuration <sub>pgfplots</sub> | 13 | 10 (76.9%) | 3 (23.1%) | 0 (0.0%) | 0 (0.0%) | 0.97 |
| pgfplots.reference.closingplots <sub>pgfplots</sub> | 6 | 3 (50.0%) | 3 (50.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Transforming Coordinate Systems <sub>pgfplots</sub> | 4 | 1 (25.0%) | 3 (75.0%) | 0 (0.0%) | 0 (0.0%) | 0.95 |
| Decorations <sub>tikz</sub> | 10 | 8 (80.0%) | 2 (20.0%) | 0 (0.0%) | 0 (0.0%) | 0.95 |
| Declaring and Using Images <sub>tikz</sub> | 2 | 0 (0.0%) | 0 (0.0%) | 2 (100.0%) | 0 (0.0%) | 0.02 |
| pgfmanual-en-dv-examples <sub>tikz</sub> | 3 | 1 (33.3%) | 2 (66.7%) | 0 (0.0%) | 0 (0.0%) | 0.96 |
| Polar Axes <sub>tikz</sub> | 11 | 9 (81.8%) | 1 (9.1%) | 1 (9.1%) | 0 (0.0%) | 0.95 |
| Visualizers <sub>tikz</sub> | 18 | 16 (88.9%) | 1 (5.6%) | 1 (5.6%) | 0 (0.0%) | 0.96 |
| Babel Library <sub>tikz</sub> | 2 | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 2 (100.0%) | 0.00 |
| To Path Library <sub>tikz</sub> | 13 | 11 (84.6%) | 0 (0.0%) | 2 (15.4%) | 0 (0.0%) | 0.92 |
| Fitting Library <sub>tikz</sub> | 4 | 2 (50.0%) | 2 (50.0%) | 0 (0.0%) | 0 (0.0%) | 0.94 |
| Lindenmayer System Drawing Library <sub>tikz</sub> | 6 | 4 (66.7%) | 2 (33.3%) | 0 (0.0%) | 0 (0.0%) | 0.96 |
| Key Management <sub>tikz</sub> | 5 | 3 (60.0%) | 2 (40.0%) | 0 (0.0%) | 0 (0.0%) | 0.93 |
| Units in Labels <sub>pgfplots</sub> | 2 | 0 (0.0%) | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0.92 |
| Number Formatting Options <sub>pgfplots</sub> | 6 | 4 (66.7%) | 2 (33.3%) | 0 (0.0%) | 0 (0.0%) | 0.97 |
| Style Options <sub>pgfplots</sub> | 2 | 0 (0.0%) | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Interoperability <sub>pgfplots</sub> | 3 | 1 (33.3%) | 1 (33.3%) | 1 (33.3%) | 0 (0.0%) | 0.68 |
| Memory and Speed considerations <sub>pgfplots</sub> | 4 | 2 (50.0%) | 2 (50.0%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Solving a Real Use Case: Scientific Data Analysis <sub>pgfplots</sub> | 7 | 5 (71.4%) | 2 (28.6%) | 0 (0.0%) | 0 (0.0%) | 0.92 |
| Design Principles <sub>tikz</sub> | 7 | 6 (85.7%) | 1 (14.3%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Introduction to Data Visualization <sub>tikz</sub> | 2 | 1 (50.0%) | 1 (50.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Three Dimensional Drawing Library <sub>tikz</sub> | 5 | 4 (80.0%) | 1 (20.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Angle Library <sub>tikz</sub> | 6 | 5 (83.3%) | 0 (0.0%) | 1 (16.7%) | 0 (0.0%) | 0.83 |
| Automata Drawing Library <sub>tikz</sub> | 8 | 7 (87.5%) | 1 (12.5%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Entity-Relationship Diagram Drawing Library <sub>tikz</sub> | 7 | 6 (85.7%) | 1 (14.3%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Paper-Folding Diagrams Library <sub>tikz</sub> | 18 | 17 (94.4%) | 0 (0.0%) | 1 (5.6%) | 0 (0.0%) | 0.97 |
| Three Point Perspective Drawing Library <sub>tikz</sub> | 15 | 14 (93.3%) | 0 (0.0%) | 1 (6.7%) | 0 (0.0%) | 0.93 |
| SVG-Path Library <sub>tikz</sub> | 1 | 0 (0.0%) | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0.00 |
| Customizing the Mathematical Engine <sub>tikz</sub> | 2 | 1 (50.0%) | 1 (50.0%) | 0 (0.0%) | 0 (0.0%) | 0.93 |
| Date and Calendar Utility Macros <sub>tikz</sub> | 1 | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Animation System Layer <sub>tikz</sub> | 1 | 0 (0.0%) | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0.32 |
| Components <sub>pgfplots</sub> | 1 | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0.84 |
| Layered Graphics <sub>tikz</sub> | 1 | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Mindmap Drawing Library <sub>tikz</sub> | 16 | 16 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Petri-Net Drawing Library <sub>tikz</sub> | 13 | 13 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Resource Description Framework Library <sub>tikz</sub> | 2 | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Shadows Library <sub>tikz</sub> | 14 | 14 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Spy Library: Magnifying Parts of Pictures <sub>tikz</sub> | 11 | 11 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Through Library <sub>tikz</sub> | 1 | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.94 |
| Turtle Graphics Library <sub>tikz</sub> | 4 | 4 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Decoration: Soft Clipping <sub>pgfplots</sub> | 1 | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |

## What tikra reported

Warnings, by how many figures they appear in (the varying part replaced by …):

| Figures | Warning |
|---:|---|
| 136 | `option … not supported: ignored` |
| 39 | `axis without data: drew the default window from N to N` |
| 13 | `\draw: figure: unknown coordinate: (…); statement ignored` |
| 13 | `ternaryaxis not supported: drawn as a cartesian axis` |
| 12 | `smithchart not supported: drawn as a cartesian axis` |
| 11 | `unrecognized path fragment: …` |
| 10 | `key handler … not supported (…): ignored` |
| 9 | `fill between: no path named … or …` |
| 8 | `\path: figure: unknown coordinate: (…); statement ignored` |
| 7 | `text outside a statement ignored` |
| 6 | `\pgftransformnonlinear not supported: … ignored` |
| 6 | `E-REC-N column N: unsupported text command: \textcolor; written as is` |
| 6 | `shape backgrounds: shape … not supported: drawn as a rectangle` |
| 6 | `E-REC-N column N: … without a matching …; closed at the end` |
| 6 | `node without text: … missing` |
| 5 | `\pgfslopedattimetrue not supported: … ignored` |
| 5 | `\pgfsetcurvilinearbeziercurve not supported: … ignored` |
| 4 | `\path: figure: name intersections: no path named …; statement ignored` |
| 4 | `\addplot table: file … not found` |
| 3 | `E-REC-N column N: unsupported text command: \node; written as is` |
| 3 | `E-REC-N column N: unsupported text command: \end; written as is` |
| 3 | `E-REC-N column N: unsupported text command: \begin; written as is` |
| 3 | `\fill: figure: unknown coordinate: (…); statement ignored` |
| 3 | `fading … not supported: drawn without it` |
| 3 | `\pgfplotstableread not supported: … ignored` |
| 3 | `\addplot graphics not supported: it needs an external program or file` |
| 2 | `\pgfsetinnerlinewidth not supported: … ignored` |
| 2 | `\pgfanimateattribute not supported: … ignored` |
| 2 | `unknown color …` |
| 2 | `E-REC-N column N: unsupported text command: \color; written as is` |
| 2 | `\pgfpatharcto not supported: … ignored` |
| 2 | `\pgfplotfunction not supported: … ignored` |
| 2 | `\pgfpathqmoveto not supported: … ignored` |
| 2 | `\pgfsetbaseline not supported: … ignored` |
| 2 | `E-REC-N column N: unsupported text command: \pgfuseshading; written as is` |
| 2 | `\pgftransformationadjustments not supported: … ignored` |
| 2 | `environment pgfviewboxscope not supported: content read as statements` |
| 2 | `unknown anchor …: center used` |
| 2 | `\tikzmath: figure: \tikzmath: an assignment needs a variable; statement ignored` |
| 2 | `E-REC-N column N: unsupported text command: \draw; written as is` |

Options tikra does not know, in figures that do not match, by how many figures they appear in:

| Figures | Option |
|---:|---|
| 9 | `symbolic x coords` |
| 6 | `contour filled` |
| 5 | `intersection segments` |
| 4 | `contour prepared` |
| 4 | `axis z line shift` |
| 3 | `ternary limits relative` |
| 3 | `tieline` |
| 3 | `interior colormap` |
| 2 | `pin in data` |
| 2 | `start branch` |
| 2 | `meet` |
| 2 | `slice` |
| 2 | `trig format` |
| 2 | `first char syntax` |
| 2 | `fading angle` |
| 2 | `base right` |
| 2 | `centered` |
| 2 | `patch type sampling` |
| 2 | `patch table with point meta` |
| 2 | `patch table` |
| 2 | `sloped like x axis` |
| 2 | `smithchart mirrored` |
| 2 | `yticklabel around circle` |
| 2 | `yticklabel around circle*` |
| 2 | `show origin` |
| 2 | `x unit` |
| 2 | `y unit` |
| 2 | `symbolic y coords` |
| 2 | `area legend` |
| 2 | `visualization depends on` |
| 2 | `cycle multiindex list` |
| 2 | `cycle multiindex* list` |
| 2 | `RdPu-6` |
| 2 | `log identify minor tick positions` |
| 2 | `ytickten` |
| 1 | `view` |
| 1 | `local bounding box` |
| 1 | `continue branch` |
| 1 | `logic gate symbol align` |
| 1 | `use IEC style logic gates` |
| 1 | `rotate fit` |
| 1 | `lower left` |
| 1 | `upper left` |
| 1 | `lower right` |
| 1 | `upper right` |
| 1 | `-leaf` |
| 1 | `(pi>3) circle` |
| 1 | `(pi>1) circle` |
| 1 | `(pi>1*2) draw` |
| 1 | `(pi>2*2) draw` |
| 1 | `(pi>3*2) draw` |
| 1 | `(pi>2) circle` |
| 1 | `(pi>4) circle` |
| 1 | `(pi>4*2) draw` |
| 1 | `dash expand off` |
| 1 | `current point is local` |
| 1 | `inner style order` |
| 1 | `use path` |
| 1 | `save path` |
| 1 | `pic type` |

Commands, environments and path operations tikra does not know, likewise:

| Figures | Command |
|---:|---|
| 6 | `\pgftransformnonlinear` |
| 5 | `\pgfsetcurvilinearbeziercurve` |
| 4 | `\pgfslopedattimetrue` |
| 3 | `\pgfplotstableread` |
| 2 | `\pgfsetinnerlinewidth` |
| 2 | `\pgfanimateattribute` |
| 2 | `\pgfpatharcto` |
| 2 | `\pgfsetfillpattern` |
| 2 | `\pgfplotfunction` |
| 2 | `\pgfpathqmoveto` |
| 2 | `\pgfsetbaseline` |
| 2 | `\pgftransformationadjustments` |
| 2 | `pgfviewboxscope` |
| 2 | `\pgfplothandlerybarinterval` |
| 2 | `\pgfuseplotmark` |
| 2 | `\breakforeach` |
| 2 | `\tikzgetnamedpath` |
| 2 | `\pgfplotscolormaptodatafile` |
| 1 | `\pgfmathdivide` |
| 1 | `colormixin` |
| 1 | `\pgfpathcurvebetweentime` |
| 1 | `\pgfpatharctoprecomputed` |
| 1 | `\pgfplotgnuplot` |
| 1 | `\pgfsetlinetofirstplotpoint` |
| 1 | `\pgfplothandlerrecord` |
| 1 | `\myhandler` |
| 1 | `\pgfintersectionofpaths` |
| 1 | `\pgfpathqcircle` |
| 1 | `\pgfsetbaselinepointlater` |
| 1 | `\newbox` |
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

Errors of the figures that fail:

| Figures | Error |
|---:|---|
| 4 | `figure: missing number` |
| 1 | `figure: you can't use \pgf@xc after \advance` |
