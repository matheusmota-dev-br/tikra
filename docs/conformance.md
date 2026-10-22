# Conformance with TikZ

How far tikra is from TikZ itself: every example of the TikZ manual (pgf 3.1.11a) and of the pgfplots
manual (1.18.2), drawn by pdflatex (TeX Live 2025) and by tikra, and compared. How it works and how
to run it: [tools/conformance](../tools/conformance/README.md).

**1868 of 2502 figures (74.7%) match TeX**; 420 are close, 214 differ and 0 fail in tikra. 428 of the ones that do not match came out with no warning.

| | Figures | Match | Close | Different | Fails | Mean F1 |
|---|---:|---:|---:|---:|---:|---:|
| All | 2502 | 1868 (74.7%) | 420 (16.8%) | 214 (8.6%) | 0 (0.0%) | 0.93 |
| TikZ manual | 1864 | 1488 (79.8%) | 223 (12.0%) | 153 (8.2%) | 0 (0.0%) | 0.93 |
| pgfplots manual | 638 | 380 (59.6%) | 197 (30.9%) | 61 (9.6%) | 0 (0.0%) | 0.92 |

*Match*: same shape (F1 ≥ 0.9 on the edges, with 0.5 mm of tolerance), no missing or extra piece, the same colors and size. *Close*: F1 ≥ 0.75.
*Fails*: tikra returned an error, panicked or took more than 10 s. Mean F1 counts a failure as 0.

Not counted: 137 where tex could not compile it, 19 where tex draws nothing visible, and 479 examples that are not figures (text, tables, key listings).

## Speed

The same 192 figures (a fixed sample of the manuals), one at a time, on Darwin arm64:

| | Median per figure | p95 |
|---|---:|---:|
| pdflatex (texlive/texlive:latest) | 647 ms | 1031 ms |
| tikra, TikZ to SVG | 0.77 ms | 7.54 ms |

Median ratio: tikra is **900× faster**. pdflatex time includes starting TeX and loading the libraries of the manual, which is what a user
waits for; tikra's is the conversion inside a running process: the first figure of a process also builds the engine, 24.3 ms more.

## By chapter

Sorted by how many figures do not match, the work left.

| Chapter | Figures | Match | Close | Different | Fails | Mean F1 |
|---|---:|---:|---:|---:|---:|---:|
| Shape Library <sub>tikz</sub> | 97 | 62 (63.9%) | 4 (4.1%) | 31 (32.0%) | 0 (0.0%) | 0.83 |
| Three Dimensional Plot Types <sub>pgfplots</sub> | 85 | 54 (63.5%) | 26 (30.6%) | 5 (5.9%) | 0 (0.0%) | 0.95 |
| Decoration Library <sub>tikz</sub> | 81 | 52 (64.2%) | 26 (32.1%) | 3 (3.7%) | 0 (0.0%) | 0.93 |
| Nodes and Edges <sub>tikz</sub> | 118 | 90 (76.3%) | 17 (14.4%) | 11 (9.3%) | 0 (0.0%) | 0.93 |
| Patchplots Library <sub>pgfplots</sub> | 29 | 4 (13.8%) | 22 (75.9%) | 3 (10.3%) | 0 (0.0%) | 0.86 |
| Specifying Coordinates <sub>tikz</sub> | 68 | 44 (64.7%) | 21 (30.9%) | 3 (4.4%) | 0 (0.0%) | 0.91 |
| Coordinate, Canvas, and Nonlinear Transformations <sub>tikz</sub> | 38 | 14 (36.8%) | 22 (57.9%) | 2 (5.3%) | 0 (0.0%) | 0.87 |
| Two Dimensional Plot Types <sub>pgfplots</sub> | 72 | 52 (72.2%) | 17 (23.6%) | 3 (4.2%) | 0 (0.0%) | 0.96 |
| Transparency <sub>tikz</sub> | 47 | 28 (59.6%) | 9 (19.1%) | 10 (21.3%) | 0 (0.0%) | 0.85 |
| Markers, Linestyles, (Background) Colors and Colormaps <sub>pgfplots</sub> | 50 | 31 (62.0%) | 14 (28.0%) | 5 (10.0%) | 0 (0.0%) | 0.92 |
| Axis Descriptions <sub>pgfplots</sub> | 77 | 59 (76.6%) | 12 (15.6%) | 6 (7.8%) | 0 (0.0%) | 0.94 |
| Fill between <sub>pgfplots</sub> | 29 | 14 (48.3%) | 14 (48.3%) | 1 (3.4%) | 0 (0.0%) | 0.97 |
| Syntax for Path Specifications <sub>tikz</sub> | 52 | 38 (73.1%) | 10 (19.2%) | 4 (7.7%) | 0 (0.0%) | 0.95 |
| Polar Axes <sub>pgfplots</sub> | 13 | 0 (0.0%) | 4 (30.8%) | 9 (69.2%) | 0 (0.0%) | 0.69 |
| Tick Options <sub>pgfplots</sub> | 32 | 19 (59.4%) | 12 (37.5%) | 1 (3.1%) | 0 (0.0%) | 0.96 |
| The Axis Environments <sub>pgfplots</sub> | 21 | 9 (42.9%) | 6 (28.6%) | 6 (28.6%) | 0 (0.0%) | 0.82 |
| Animations <sub>tikz</sub> | 24 | 13 (54.2%) | 7 (29.2%) | 4 (16.7%) | 0 (0.0%) | 0.90 |
| Circuit Libraries <sub>tikz</sub> | 64 | 53 (82.8%) | 5 (7.8%) | 6 (9.4%) | 0 (0.0%) | 0.95 |
| Actions on Paths <sub>tikz</sub> | 72 | 62 (86.1%) | 6 (8.3%) | 4 (5.6%) | 0 (0.0%) | 0.97 |
| Matrices and Alignment <sub>tikz</sub> | 26 | 17 (65.4%) | 4 (15.4%) | 5 (19.2%) | 0 (0.0%) | 0.87 |
| Error Bars <sub>pgfplots</sub> | 9 | 0 (0.0%) | 7 (77.8%) | 2 (22.2%) | 0 (0.0%) | 0.78 |
| Constructing Paths <sub>tikz</sub> | 26 | 18 (69.2%) | 5 (19.2%) | 3 (11.5%) | 0 (0.0%) | 0.84 |
| Smith Charts <sub>pgfplots</sub> | 12 | 4 (33.3%) | 7 (58.3%) | 1 (8.3%) | 0 (0.0%) | 0.90 |
| Nodes and Shapes <sub>tikz</sub> | 12 | 5 (41.7%) | 2 (16.7%) | 5 (41.7%) | 0 (0.0%) | 0.74 |
| Axes <sub>tikz</sub> | 78 | 71 (91.0%) | 7 (9.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Transformations <sub>tikz</sub> | 22 | 15 (68.2%) | 6 (27.3%) | 1 (4.5%) | 0 (0.0%) | 0.92 |
| Tutorial: A Lecture Map for Johannes <sub>tikz</sub> | 10 | 3 (30.0%) | 3 (30.0%) | 4 (40.0%) | 0 (0.0%) | 0.69 |
| Background Library <sub>tikz</sub> | 12 | 6 (50.0%) | 4 (33.3%) | 2 (16.7%) | 0 (0.0%) | 0.87 |
| Matrix Library <sub>tikz</sub> | 14 | 8 (57.1%) | 4 (28.6%) | 2 (14.3%) | 0 (0.0%) | 0.87 |
| Shadings Library <sub>tikz</sub> | 11 | 5 (45.5%) | 0 (0.0%) | 6 (54.5%) | 0 (0.0%) | 0.72 |
| Pics: Small Pictures on Paths <sub>tikz</sub> | 14 | 8 (57.1%) | 3 (21.4%) | 3 (21.4%) | 0 (0.0%) | 0.80 |
| Grid Options <sub>pgfplots</sub> | 18 | 12 (66.7%) | 5 (27.8%) | 1 (5.6%) | 0 (0.0%) | 0.95 |
| Specifying the Plotted Range <sub>pgfplots</sub> | 13 | 7 (53.8%) | 5 (38.5%) | 1 (7.7%) | 0 (0.0%) | 0.93 |
| Matrices <sub>tikz</sub> | 7 | 2 (28.6%) | 3 (42.9%) | 2 (28.6%) | 0 (0.0%) | 0.69 |
| Shadings <sub>tikz</sub> | 6 | 1 (16.7%) | 1 (16.7%) | 4 (66.7%) | 0 (0.0%) | 0.48 |
| Calendar Library <sub>tikz</sub> | 41 | 36 (87.8%) | 5 (12.2%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Pattern Library <sub>tikz</sub> | 8 | 3 (37.5%) | 3 (37.5%) | 2 (25.0%) | 0 (0.0%) | 0.74 |
| Utilities and Basic Level Commands <sub>pgfplots</sub> | 7 | 2 (28.6%) | 5 (71.4%) | 0 (0.0%) | 0 (0.0%) | 0.92 |
| 3D Axis Configuration <sub>pgfplots</sub> | 13 | 8 (61.5%) | 5 (38.5%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Alignment Options <sub>pgfplots</sub> | 8 | 3 (37.5%) | 4 (50.0%) | 1 (12.5%) | 0 (0.0%) | 0.82 |
| The Picture's Size: Bounding Box and Clipping <sub>pgfplots</sub> | 5 | 0 (0.0%) | 1 (20.0%) | 4 (80.0%) | 0 (0.0%) | 0.43 |
| Skipping Or Changing Coordinates -- Filters <sub>pgfplots</sub> | 7 | 2 (28.6%) | 5 (71.4%) | 0 (0.0%) | 0 (0.0%) | 0.91 |
| Fitting Lines -- Regression <sub>pgfplots</sub> | 5 | 0 (0.0%) | 4 (80.0%) | 1 (20.0%) | 0 (0.0%) | 0.87 |
| Scaling Options <sub>pgfplots</sub> | 22 | 17 (77.3%) | 1 (4.5%) | 4 (18.2%) | 0 (0.0%) | 0.89 |
| pgfmanual-en-base-scopes <sub>tikz</sub> | 19 | 15 (78.9%) | 1 (5.3%) | 3 (15.8%) | 0 (0.0%) | 0.87 |
| Style Sheets and Legends <sub>tikz</sub> | 85 | 81 (95.3%) | 4 (4.7%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Visualizers <sub>tikz</sub> | 18 | 14 (77.8%) | 4 (22.2%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Tutorials and Guidelines <sub>tikz</sub> | 9 | 5 (55.6%) | 1 (11.1%) | 3 (33.3%) | 0 (0.0%) | 0.75 |
| Tutorial: Euclid's Amber Version of the Elements <sub>tikz</sub> | 14 | 10 (71.4%) | 2 (14.3%) | 2 (14.3%) | 0 (0.0%) | 0.89 |
| Tutorial: A Picture for Karl's Students <sub>tikz</sub> | 46 | 42 (91.3%) | 3 (6.5%) | 1 (2.2%) | 0 (0.0%) | 0.95 |
| Statistics <sub>pgfplots</sub> | 26 | 22 (84.6%) | 1 (3.8%) | 3 (11.5%) | 0 (0.0%) | 0.95 |
| Miscellaneous Options <sub>pgfplots</sub> | 7 | 3 (42.9%) | 4 (57.1%) | 0 (0.0%) | 0 (0.0%) | 0.92 |
| Solving a Real Use Case: Scientific Data Analysis <sub>pgfplots</sub> | 7 | 3 (42.9%) | 4 (57.1%) | 0 (0.0%) | 0 (0.0%) | 0.92 |
| Quick Commands <sub>tikz</sub> | 3 | 0 (0.0%) | 2 (66.7%) | 1 (33.3%) | 0 (0.0%) | 0.76 |
| Math Library <sub>tikz</sub> | 6 | 3 (50.0%) | 1 (16.7%) | 2 (33.3%) | 0 (0.0%) | 0.75 |
| Additional Mathematical Commands <sub>tikz</sub> | 3 | 0 (0.0%) | 0 (0.0%) | 3 (100.0%) | 0 (0.0%) | 0.03 |
| Mathematical Expressions <sub>tikz</sub> | 3 | 0 (0.0%) | 1 (33.3%) | 2 (66.7%) | 0 (0.0%) | 0.59 |
| Repeating Things: The Foreach Statement <sub>tikz</sub> | 13 | 10 (76.9%) | 1 (7.7%) | 2 (15.4%) | 0 (0.0%) | 0.84 |
| Making Trees Grow <sub>tikz</sub> | 27 | 24 (88.9%) | 2 (7.4%) | 1 (3.7%) | 0 (0.0%) | 0.98 |
| Using Paths <sub>tikz</sub> | 15 | 13 (86.7%) | 0 (0.0%) | 2 (13.3%) | 0 (0.0%) | 0.87 |
| Decorations <sub>tikz</sub> | 10 | 8 (80.0%) | 2 (20.0%) | 0 (0.0%) | 0 (0.0%) | 0.95 |
| Declaring and Using Images <sub>tikz</sub> | 2 | 0 (0.0%) | 0 (0.0%) | 2 (100.0%) | 0 (0.0%) | 0.02 |
| Patterns <sub>tikz</sub> | 4 | 2 (50.0%) | 0 (0.0%) | 2 (50.0%) | 0 (0.0%) | 0.63 |
| Creating Plots <sub>tikz</sub> | 7 | 5 (71.4%) | 0 (0.0%) | 2 (28.6%) | 0 (0.0%) | 0.88 |
| To Path Library <sub>tikz</sub> | 13 | 11 (84.6%) | 0 (0.0%) | 2 (15.4%) | 0 (0.0%) | 0.93 |
| Fitting Library <sub>tikz</sub> | 4 | 2 (50.0%) | 2 (50.0%) | 0 (0.0%) | 0 (0.0%) | 0.94 |
| Lindenmayer System Drawing Library <sub>tikz</sub> | 6 | 4 (66.7%) | 2 (33.3%) | 0 (0.0%) | 0 (0.0%) | 0.96 |
| Key Management <sub>tikz</sub> | 5 | 3 (60.0%) | 2 (40.0%) | 0 (0.0%) | 0 (0.0%) | 0.92 |
| Arrows <sub>tikz</sub> | 104 | 102 (98.1%) | 2 (1.9%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Decorated Paths <sub>tikz</sub> | 31 | 29 (93.5%) | 2 (6.5%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Specifying Graphs <sub>tikz</sub> | 159 | 157 (98.7%) | 1 (0.6%) | 1 (0.6%) | 0 (0.0%) | 1.00 |
| Tutorial: Diagrams as Simple Graphs <sub>tikz</sub> | 19 | 17 (89.5%) | 2 (10.5%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Tutorial: A Petri-Net for Hagen <sub>tikz</sub> | 28 | 26 (92.9%) | 2 (7.1%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Units in Labels <sub>pgfplots</sub> | 2 | 0 (0.0%) | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0.92 |
| Symbolic Coordinates and User Transformations <sub>pgfplots</sub> | 5 | 3 (60.0%) | 1 (20.0%) | 1 (20.0%) | 0 (0.0%) | 0.82 |
| Interoperability <sub>pgfplots</sub> | 3 | 1 (33.3%) | 1 (33.3%) | 1 (33.3%) | 0 (0.0%) | 0.67 |
| Solving a Real Use Case: Function Visualization <sub>pgfplots</sub> | 6 | 4 (66.7%) | 0 (0.0%) | 2 (33.3%) | 0 (0.0%) | 0.89 |
| Use Cases involving Scatter Plots <sub>pgfplots</sub> | 5 | 3 (60.0%) | 2 (40.0%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| pgfmanual-en-dv-examples <sub>tikz</sub> | 3 | 2 (66.7%) | 1 (33.3%) | 0 (0.0%) | 0 (0.0%) | 0.94 |
| Polar Axes <sub>tikz</sub> | 11 | 10 (90.9%) | 1 (9.1%) | 0 (0.0%) | 0 (0.0%) | 0.97 |
| Angle Library <sub>tikz</sub> | 6 | 5 (83.3%) | 0 (0.0%) | 1 (16.7%) | 0 (0.0%) | 0.83 |
| Automata Drawing Library <sub>tikz</sub> | 8 | 7 (87.5%) | 1 (12.5%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Chains <sub>tikz</sub> | 14 | 13 (92.9%) | 0 (0.0%) | 1 (7.1%) | 0 (0.0%) | 0.98 |
| Three Point Perspective Drawing Library <sub>tikz</sub> | 15 | 14 (93.3%) | 0 (0.0%) | 1 (6.7%) | 0 (0.0%) | 0.92 |
| SVG-Path Library <sub>tikz</sub> | 1 | 0 (0.0%) | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0.00 |
| Views Library <sub>tikz</sub> | 4 | 3 (75.0%) | 1 (25.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Customizing the Mathematical Engine <sub>tikz</sub> | 2 | 1 (50.0%) | 1 (50.0%) | 0 (0.0%) | 0 (0.0%) | 0.93 |
| Date and Calendar Utility Macros <sub>tikz</sub> | 1 | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Animation System Layer <sub>tikz</sub> | 1 | 0 (0.0%) | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0.26 |
| pgfmanual-en-tikz-scopes <sub>tikz</sub> | 13 | 12 (92.3%) | 1 (7.7%) | 0 (0.0%) | 0 (0.0%) | 0.96 |
| Decoration: Soft Clipping <sub>pgfplots</sub> | 1 | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Components <sub>pgfplots</sub> | 1 | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0.83 |
| pgfplots.reference.closingplots <sub>pgfplots</sub> | 6 | 5 (83.3%) | 1 (16.7%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Number Formatting Options <sub>pgfplots</sub> | 6 | 5 (83.3%) | 1 (16.7%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Transforming Coordinate Systems <sub>pgfplots</sub> | 4 | 3 (75.0%) | 1 (25.0%) | 0 (0.0%) | 0 (0.0%) | 0.94 |
| Solving a Real Use Case: Functions of Two Variables <sub>pgfplots</sub> | 5 | 4 (80.0%) | 1 (20.0%) | 0 (0.0%) | 0 (0.0%) | 0.96 |
| Design Principles <sub>tikz</sub> | 7 | 7 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Layered Graphics <sub>tikz</sub> | 1 | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Providing Data for a Data Visualization <sub>tikz</sub> | 8 | 8 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Introduction to Data Visualization <sub>tikz</sub> | 2 | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Creating Data Visualizations <sub>tikz</sub> | 13 | 13 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Three Dimensional Drawing Library <sub>tikz</sub> | 5 | 5 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Babel Library <sub>tikz</sub> | 2 | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Entity-Relationship Diagram Drawing Library <sub>tikz</sub> | 7 | 7 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Paper-Folding Diagrams Library <sub>tikz</sub> | 18 | 18 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Mindmap Drawing Library <sub>tikz</sub> | 16 | 16 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Petri-Net Drawing Library <sub>tikz</sub> | 13 | 13 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Plot Handler Library <sub>tikz</sub> | 23 | 23 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Resource Description Framework Library <sub>tikz</sub> | 2 | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Shadows Library <sub>tikz</sub> | 14 | 14 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Spy Library: Magnifying Parts of Pictures <sub>tikz</sub> | 11 | 11 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Through Library <sub>tikz</sub> | 1 | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.94 |
| Tree Library <sub>tikz</sub> | 7 | 7 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Turtle Graphics Library <sub>tikz</sub> | 4 | 4 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Plots of Functions <sub>tikz</sub> | 30 | 30 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Grouping plots <sub>pgfplots</sub> | 8 | 8 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Ternary Diagrams <sub>pgfplots</sub> | 13 | 13 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Style Options <sub>pgfplots</sub> | 2 | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Memory and Speed considerations <sub>pgfplots</sub> | 4 | 4 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.98 |

## What tikra reported

Warnings, by how many figures they appear in (the varying part replaced by …):

| Figures | Warning |
|---:|---|
| 79 | `option … not supported: ignored` |
| 12 | `\draw: figure: unknown coordinate: (…); statement ignored` |
| 12 | `axis without data: drew the default window from N to N` |
| 10 | `unrecognized path fragment: …` |
| 10 | `key handler … not supported (…): ignored` |
| 9 | `fill between: no path named … or …` |
| 8 | `\path: figure: unknown coordinate: (…); statement ignored` |
| 8 | `fill between: … not supported: the whole area is filled` |
| 6 | `E-REC-N column N: … without a matching …; closed at the end` |
| 6 | `node without text: … missing` |
| 5 | `text outside a statement ignored` |
| 5 | `\pgfslopedattimetrue not supported: … ignored` |
| 4 | `\path: figure: name intersections: no path named …; statement ignored` |
| 4 | `\addplot table: file … not found` |
| 3 | `\end{tikzpicture} without a matching \begin: ignored` |
| 3 | `E-REC-N column N: unsupported text command: \end; written as is` |
| 3 | `E-REC-N column N: unsupported text command: \node; written as is` |
| 3 | `E-REC-N column N: unsupported text command: \begin; written as is` |
| 3 | `\fill: figure: unknown coordinate: (…); statement ignored` |
| 3 | `fading … not supported: drawn without it` |
| 3 | `\addplot graphics not supported: it needs an external program or file` |
| 2 | `\pgfsetinnerlinewidth not supported: … ignored` |
| 2 | `\pgfanimateattribute not supported: … ignored` |
| 2 | `unknown color …` |
| 2 | `E-REC-N column N: unsupported text command: \color; written as is` |
| 2 | `\pgfpatharcto not supported: … ignored` |
| 2 | `\pgfpathqmoveto not supported: … ignored` |
| 2 | `\pgfsetbaseline not supported: … ignored` |
| 2 | `E-REC-N column N: unsupported text command: \pgfuseshading; written as is` |
| 2 | `\pgftransformationadjustments not supported: … ignored` |
| 2 | `\pgfpointcurvilinearbezierpolar not supported: … ignored` |
| 2 | `unknown anchor …: center used` |
| 2 | `\tikzmath: figure: \tikzmath: an assignment needs a variable; statement ignored` |
| 2 | `E-REC-N column N: unsupported text command: \draw; written as is` |
| 2 | `\draw: figure: controls without a point; statement ignored` |
| 2 | `\node: figure: unknown coordinate: (…); statement ignored` |
| 2 | `\tikzgetnamedpath not supported: … ignored` |
| 2 | `\addplot: figure: \addplot input … not supported; plot ignored` |
| 2 | `\pgfplotscolormaptodatafile not supported: … ignored` |
| 2 | `colormap … not defined` |

Options tikra does not know, in figures that do not match, by how many figures they appear in:

| Figures | Option |
|---:|---|
| 3 | `patch table` |
| 3 | `interior colormap` |
| 2 | `pin in data` |
| 2 | `trig format` |
| 2 | `first char syntax` |
| 2 | `soft clip` |
| 2 | `soft clip path` |
| 2 | `patch type sampling` |
| 2 | `patch table with point meta` |
| 2 | `y unit` |
| 2 | `x unit` |
| 2 | `area legend` |
| 2 | `visualization depends on` |
| 2 | `cycle multiindex list` |
| 2 | `cycle multiindex* list` |
| 2 | `RdPu-6` |
| 2 | `log identify minor tick positions` |
| 1 | `view` |
| 1 | `rings` |
| 1 | `thickness` |
| 1 | `logic gate symbol align` |
| 1 | `use IEC style logic gates` |
| 1 | `rotate fit` |
| 1 | `lower right` |
| 1 | `upper left` |
| 1 | `upper right` |
| 1 | `lower left` |
| 1 | `-leaf` |
| 1 | `(pi>2) circle` |
| 1 | `(pi>3*2) draw` |
| 1 | `(pi>4*2) draw` |
| 1 | `(pi>4) circle` |
| 1 | `(pi>2*2) draw` |
| 1 | `(pi>1) circle` |
| 1 | `(pi>3) circle` |
| 1 | `(pi>1*2) draw` |
| 1 | `current point is local` |
| 1 | `inner style order` |
| 1 | `pic type` |
| 1 | `transform shape nonlinear` |
| 1 | `absolute` |
| 1 | `red,draw,thick` |
| 1 | `late options` |
| 1 | `rotate around x` |
| 1 | `rotate around y` |
| 1 | `rotate around z` |
| 1 | `blend mode` |
| 1 | `blend group` |
| 1 | `fading transform` |
| 1 | `pgfpoint substitution` |
| 1 | `position` |
| 1 | `vertex count` |
| 1 | `symbolic coords` |
| 1 | `use units` |
| 1 | `x unit prefix` |
| 1 | `y unit prefix` |
| 1 | `x SI prefix` |
| 1 | `change x base` |
| 1 | `y SI prefix` |
| 1 | `path replacement` |

Commands, environments and path operations tikra does not know, likewise:

| Figures | Command |
|---:|---|
| 8 | `\pgfpointcurvilinearbezierpolar` |
| 4 | `\pgfslopedattimetrue` |
| 2 | `\pgfsetinnerlinewidth` |
| 2 | `\pgfanimateattribute` |
| 2 | `\pgfpatharcto` |
| 2 | `\pgfpathqmoveto` |
| 2 | `\pgfsetbaseline` |
| 2 | `\pgftransformationadjustments` |
| 2 | `\breakforeach` |
| 2 | `\tikzgetnamedpath` |
| 2 | `\pgfplotscolormaptodatafile` |
| 1 | `\pgfmathdivide` |
| 1 | `colormixin` |
| 1 | `\box` |
| 1 | `\pgfpositionnodelater` |
| 1 | `\pgfpatharctoprecomputed` |
| 1 | `\pgfplotgnuplot` |
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
| 1 | `\pgfsettransformnonlinearflatness` |
| 1 | `\pgfsetblendmode` |
| 1 | `\pgfpathsvg` |
| 1 | `\pgfmathrandominteger` |
| 1 | `\pgfmathdeclarerandomlist` |
| 1 | `\pgfmathdectobase` |
| 1 | `\pgfidrefnextuse` |
| 1 | `\pgfplotstransformcoordinatex` |
| 1 | `\tikzfillbetween` |

Errors of the figures that fail:

| Figures | Error |
|---:|---|
