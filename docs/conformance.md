# Conformance with TikZ

How far tikra is from TikZ itself: every example of the TikZ manual (pgf 3.1.11a) and of the pgfplots
manual (1.18.2), drawn by pdflatex (TeX Live 2025) and by tikra, and compared. How it works and how
to run it: [tools/conformance](../tools/conformance/README.md).

**2237 of 2502 figures (89.4%) match TeX**; 204 are close, 61 differ and 0 fail in tikra. 220 of the ones that do not match came out with no warning.

**Our 100%: 2255 of 2486 (90.7%)**. It leaves out the 16 figures no converter can reproduce (9 with random numbers TeX draws anew on every run, 3 with an image file or external program, 4 with the day's date), and counts as done 21 that differ only by antialiasing. The list, checked by eye: [unreproducible.json](../tools/conformance/unreproducible.json).

| | Figures | Match | Close | Different | Fails | Mean F1 |
|---|---:|---:|---:|---:|---:|---:|
| All | 2502 | 2237 (89.4%) | 204 (8.2%) | 61 (2.4%) | 0 (0.0%) | 0.97 |
| TikZ manual | 1864 | 1668 (89.5%) | 160 (8.6%) | 36 (1.9%) | 0 (0.0%) | 0.97 |
| pgfplots manual | 638 | 569 (89.2%) | 44 (6.9%) | 25 (3.9%) | 0 (0.0%) | 0.97 |

*Match*: same shape (F1 ≥ 0.9 on the edges, with 0.5 mm of tolerance), no missing or extra piece, the same colors and size. *Close*: F1 ≥ 0.75.
*Fails*: tikra returned an error, panicked or took more than 10 s. Mean F1 counts a failure as 0.

Not counted: 137 where tex could not compile it, 19 where tex draws nothing visible, and 479 examples that are not figures (text, tables, key listings).

## Speed

The same 192 figures (a fixed sample of the manuals), one at a time, on Darwin arm64:

| | Median per figure | p95 |
|---|---:|---:|
| pdflatex (texlive/texlive:latest) | 647 ms | 1031 ms |
| tikra, TikZ to SVG | 1.18 ms | 11.31 ms |

Median ratio: tikra is **604× faster**. pdflatex time includes starting TeX and loading the libraries of the manual, which is what a user
waits for; tikra's is the conversion inside a running process: the first figure of a process also builds the engine, 15.7 ms more.

## By chapter

Sorted by how many figures do not match, the work left.

| Chapter | Figures | Match | Close | Different | Fails | Mean F1 |
|---|---:|---:|---:|---:|---:|---:|
| Shape Library <sub>tikz</sub> | 97 | 67 (69.1%) | 30 (30.9%) | 0 (0.0%) | 0 (0.0%) | 0.97 |
| Decoration Library <sub>tikz</sub> | 81 | 59 (72.8%) | 19 (23.5%) | 3 (3.7%) | 0 (0.0%) | 0.93 |
| Transparency <sub>tikz</sub> | 47 | 29 (61.7%) | 13 (27.7%) | 5 (10.6%) | 0 (0.0%) | 0.89 |
| Specifying Coordinates <sub>tikz</sub> | 68 | 56 (82.4%) | 12 (17.6%) | 0 (0.0%) | 0 (0.0%) | 0.96 |
| Three Dimensional Plot Types <sub>pgfplots</sub> | 85 | 74 (87.1%) | 8 (9.4%) | 3 (3.5%) | 0 (0.0%) | 0.98 |
| Coordinate, Canvas, and Nonlinear Transformations <sub>tikz</sub> | 38 | 28 (73.7%) | 10 (26.3%) | 0 (0.0%) | 0 (0.0%) | 0.93 |
| Circuit Libraries <sub>tikz</sub> | 64 | 54 (84.4%) | 6 (9.4%) | 4 (6.2%) | 0 (0.0%) | 0.96 |
| Nodes and Edges <sub>tikz</sub> | 118 | 111 (94.1%) | 2 (1.7%) | 5 (4.2%) | 0 (0.0%) | 0.97 |
| Actions on Paths <sub>tikz</sub> | 72 | 66 (91.7%) | 5 (6.9%) | 1 (1.4%) | 0 (0.0%) | 0.98 |
| Fill between <sub>pgfplots</sub> | 29 | 23 (79.3%) | 5 (17.2%) | 1 (3.4%) | 0 (0.0%) | 0.98 |
| The Axis Environments <sub>pgfplots</sub> | 21 | 15 (71.4%) | 1 (4.8%) | 5 (23.8%) | 0 (0.0%) | 0.88 |
| Axis Descriptions <sub>pgfplots</sub> | 77 | 71 (92.2%) | 2 (2.6%) | 4 (5.2%) | 0 (0.0%) | 0.96 |
| Axes <sub>tikz</sub> | 78 | 73 (93.6%) | 5 (6.4%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Syntax for Path Specifications <sub>tikz</sub> | 52 | 47 (90.4%) | 5 (9.6%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Tutorial: A Lecture Map for Johannes <sub>tikz</sub> | 10 | 5 (50.0%) | 4 (40.0%) | 1 (10.0%) | 0 (0.0%) | 0.90 |
| Two Dimensional Plot Types <sub>pgfplots</sub> | 72 | 67 (93.1%) | 4 (5.6%) | 1 (1.4%) | 0 (0.0%) | 0.99 |
| Scaling Options <sub>pgfplots</sub> | 22 | 17 (77.3%) | 1 (4.5%) | 4 (18.2%) | 0 (0.0%) | 0.89 |
| Nodes and Shapes <sub>tikz</sub> | 12 | 8 (66.7%) | 3 (25.0%) | 1 (8.3%) | 0 (0.0%) | 0.87 |
| Pattern Library <sub>tikz</sub> | 8 | 4 (50.0%) | 4 (50.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Utilities and Basic Level Commands <sub>pgfplots</sub> | 7 | 3 (42.9%) | 4 (57.1%) | 0 (0.0%) | 0 (0.0%) | 0.94 |
| Constructing Paths <sub>tikz</sub> | 26 | 23 (88.5%) | 3 (11.5%) | 0 (0.0%) | 0 (0.0%) | 0.96 |
| Style Sheets and Legends <sub>tikz</sub> | 85 | 82 (96.5%) | 3 (3.5%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Visualizers <sub>tikz</sub> | 18 | 15 (83.3%) | 3 (16.7%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Matrix Library <sub>tikz</sub> | 14 | 11 (78.6%) | 3 (21.4%) | 0 (0.0%) | 0 (0.0%) | 0.97 |
| Transformations <sub>tikz</sub> | 22 | 19 (86.4%) | 3 (13.6%) | 0 (0.0%) | 0 (0.0%) | 0.96 |
| Tutorial: A Picture for Karl's Students <sub>tikz</sub> | 46 | 43 (93.5%) | 3 (6.5%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Smith Charts <sub>pgfplots</sub> | 12 | 9 (75.0%) | 2 (16.7%) | 1 (8.3%) | 0 (0.0%) | 0.90 |
| Using Paths <sub>tikz</sub> | 15 | 13 (86.7%) | 2 (13.3%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Animations <sub>tikz</sub> | 24 | 22 (91.7%) | 2 (8.3%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Declaring and Using Images <sub>tikz</sub> | 2 | 0 (0.0%) | 0 (0.0%) | 2 (100.0%) | 0 (0.0%) | 0.02 |
| Patterns <sub>tikz</sub> | 4 | 2 (50.0%) | 1 (25.0%) | 1 (25.0%) | 0 (0.0%) | 0.83 |
| Quick Commands <sub>tikz</sub> | 3 | 1 (33.3%) | 2 (66.7%) | 0 (0.0%) | 0 (0.0%) | 0.87 |
| pgfmanual-en-base-scopes <sub>tikz</sub> | 19 | 17 (89.5%) | 1 (5.3%) | 1 (5.3%) | 0 (0.0%) | 0.92 |
| Lindenmayer System Drawing Library <sub>tikz</sub> | 6 | 4 (66.7%) | 2 (33.3%) | 0 (0.0%) | 0 (0.0%) | 0.96 |
| Math Library <sub>tikz</sub> | 6 | 4 (66.7%) | 1 (16.7%) | 1 (16.7%) | 0 (0.0%) | 0.94 |
| Additional Mathematical Commands <sub>tikz</sub> | 3 | 1 (33.3%) | 0 (0.0%) | 2 (66.7%) | 0 (0.0%) | 0.33 |
| Key Management <sub>tikz</sub> | 5 | 3 (60.0%) | 2 (40.0%) | 0 (0.0%) | 0 (0.0%) | 0.92 |
| Specifying Graphs <sub>tikz</sub> | 159 | 157 (98.7%) | 1 (0.6%) | 1 (0.6%) | 0 (0.0%) | 1.00 |
| Tutorial: A Petri-Net for Hagen <sub>tikz</sub> | 28 | 26 (92.9%) | 2 (7.1%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Polar Axes <sub>pgfplots</sub> | 13 | 11 (84.6%) | 2 (15.4%) | 0 (0.0%) | 0 (0.0%) | 0.91 |
| Statistics <sub>pgfplots</sub> | 26 | 24 (92.3%) | 0 (0.0%) | 2 (7.7%) | 0 (0.0%) | 0.97 |
| Alignment Options <sub>pgfplots</sub> | 8 | 6 (75.0%) | 1 (12.5%) | 1 (12.5%) | 0 (0.0%) | 0.88 |
| The Picture's Size: Bounding Box and Clipping <sub>pgfplots</sub> | 5 | 3 (60.0%) | 2 (40.0%) | 0 (0.0%) | 0 (0.0%) | 0.96 |
| Miscellaneous Options <sub>pgfplots</sub> | 7 | 5 (71.4%) | 2 (28.6%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Specifying the Plotted Range <sub>pgfplots</sub> | 13 | 11 (84.6%) | 2 (15.4%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Symbolic Coordinates and User Transformations <sub>pgfplots</sub> | 5 | 3 (60.0%) | 1 (20.0%) | 1 (20.0%) | 0 (0.0%) | 0.83 |
| Decorations <sub>tikz</sub> | 10 | 9 (90.0%) | 1 (10.0%) | 0 (0.0%) | 0 (0.0%) | 0.96 |
| Creating Plots <sub>tikz</sub> | 7 | 6 (85.7%) | 0 (0.0%) | 1 (14.3%) | 0 (0.0%) | 0.96 |
| Shadings <sub>tikz</sub> | 6 | 5 (83.3%) | 0 (0.0%) | 1 (16.7%) | 0 (0.0%) | 0.85 |
| Polar Axes <sub>tikz</sub> | 11 | 10 (90.9%) | 1 (9.1%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Automata Drawing Library <sub>tikz</sub> | 8 | 7 (87.5%) | 1 (12.5%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Calendar Library <sub>tikz</sub> | 41 | 40 (97.6%) | 0 (0.0%) | 1 (2.4%) | 0 (0.0%) | 0.99 |
| Fitting Library <sub>tikz</sub> | 4 | 3 (75.0%) | 1 (25.0%) | 0 (0.0%) | 0 (0.0%) | 0.97 |
| Three Point Perspective Drawing Library <sub>tikz</sub> | 15 | 14 (93.3%) | 0 (0.0%) | 1 (6.7%) | 0 (0.0%) | 0.92 |
| Shadings Library <sub>tikz</sub> | 11 | 10 (90.9%) | 0 (0.0%) | 1 (9.1%) | 0 (0.0%) | 0.90 |
| Views Library <sub>tikz</sub> | 4 | 3 (75.0%) | 1 (25.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Tutorials and Guidelines <sub>tikz</sub> | 9 | 8 (88.9%) | 0 (0.0%) | 1 (11.1%) | 0 (0.0%) | 0.88 |
| Mathematical Expressions <sub>tikz</sub> | 3 | 2 (66.7%) | 0 (0.0%) | 1 (33.3%) | 0 (0.0%) | 0.73 |
| Animation System Layer <sub>tikz</sub> | 1 | 0 (0.0%) | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0.71 |
| Arrows <sub>tikz</sub> | 104 | 103 (99.0%) | 1 (1.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Decorated Paths <sub>tikz</sub> | 31 | 30 (96.8%) | 1 (3.2%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Matrices and Alignment <sub>tikz</sub> | 26 | 25 (96.2%) | 1 (3.8%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Decoration: Soft Clipping <sub>pgfplots</sub> | 1 | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Patchplots Library <sub>pgfplots</sub> | 29 | 28 (96.6%) | 1 (3.4%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| 3D Axis Configuration <sub>pgfplots</sub> | 13 | 12 (92.3%) | 1 (7.7%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Skipping Or Changing Coordinates -- Filters <sub>pgfplots</sub> | 7 | 6 (85.7%) | 1 (14.3%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Grid Options <sub>pgfplots</sub> | 18 | 17 (94.4%) | 0 (0.0%) | 1 (5.6%) | 0 (0.0%) | 0.97 |
| Markers, Linestyles, (Background) Colors and Colormaps <sub>pgfplots</sub> | 50 | 49 (98.0%) | 1 (2.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Tick Options <sub>pgfplots</sub> | 32 | 31 (96.9%) | 1 (3.1%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Interoperability <sub>pgfplots</sub> | 3 | 2 (66.7%) | 0 (0.0%) | 1 (33.3%) | 0 (0.0%) | 0.86 |
| Transforming Coordinate Systems <sub>pgfplots</sub> | 4 | 3 (75.0%) | 1 (25.0%) | 0 (0.0%) | 0 (0.0%) | 0.97 |
| Design Principles <sub>tikz</sub> | 7 | 7 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Layered Graphics <sub>tikz</sub> | 1 | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Matrices <sub>tikz</sub> | 7 | 7 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| pgfmanual-en-dv-examples <sub>tikz</sub> | 3 | 3 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.97 |
| Providing Data for a Data Visualization <sub>tikz</sub> | 8 | 8 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Introduction to Data Visualization <sub>tikz</sub> | 2 | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Creating Data Visualizations <sub>tikz</sub> | 13 | 13 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Three Dimensional Drawing Library <sub>tikz</sub> | 5 | 5 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Angle Library <sub>tikz</sub> | 6 | 6 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Babel Library <sub>tikz</sub> | 2 | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Background Library <sub>tikz</sub> | 12 | 12 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Chains <sub>tikz</sub> | 14 | 14 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| To Path Library <sub>tikz</sub> | 13 | 13 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Entity-Relationship Diagram Drawing Library <sub>tikz</sub> | 7 | 7 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Paper-Folding Diagrams Library <sub>tikz</sub> | 18 | 18 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Mindmap Drawing Library <sub>tikz</sub> | 16 | 16 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Petri-Net Drawing Library <sub>tikz</sub> | 13 | 13 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Plot Handler Library <sub>tikz</sub> | 23 | 23 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Resource Description Framework Library <sub>tikz</sub> | 2 | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Shadows Library <sub>tikz</sub> | 14 | 14 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Spy Library: Magnifying Parts of Pictures <sub>tikz</sub> | 11 | 11 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| SVG-Path Library <sub>tikz</sub> | 1 | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Through Library <sub>tikz</sub> | 1 | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.96 |
| Tree Library <sub>tikz</sub> | 7 | 7 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Turtle Graphics Library <sub>tikz</sub> | 4 | 4 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Customizing the Mathematical Engine <sub>tikz</sub> | 2 | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.94 |
| Date and Calendar Utility Macros <sub>tikz</sub> | 1 | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Repeating Things: The Foreach Statement <sub>tikz</sub> | 13 | 13 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Pics: Small Pictures on Paths <sub>tikz</sub> | 14 | 14 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Plots of Functions <sub>tikz</sub> | 30 | 30 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| pgfmanual-en-tikz-scopes <sub>tikz</sub> | 13 | 13 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Making Trees Grow <sub>tikz</sub> | 27 | 27 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Tutorial: Euclid's Amber Version of the Elements <sub>tikz</sub> | 14 | 14 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Tutorial: Diagrams as Simple Graphs <sub>tikz</sub> | 19 | 19 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Grouping plots <sub>pgfplots</sub> | 8 | 8 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Ternary Diagrams <sub>pgfplots</sub> | 13 | 13 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Units in Labels <sub>pgfplots</sub> | 2 | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Components <sub>pgfplots</sub> | 1 | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.94 |
| pgfplots.reference.closingplots <sub>pgfplots</sub> | 6 | 6 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Error Bars <sub>pgfplots</sub> | 9 | 9 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Fitting Lines -- Regression <sub>pgfplots</sub> | 5 | 5 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Number Formatting Options <sub>pgfplots</sub> | 6 | 6 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Style Options <sub>pgfplots</sub> | 2 | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Memory and Speed considerations <sub>pgfplots</sub> | 4 | 4 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Solving a Real Use Case: Function Visualization <sub>pgfplots</sub> | 6 | 6 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Solving a Real Use Case: Scientific Data Analysis <sub>pgfplots</sub> | 7 | 7 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Use Cases involving Scatter Plots <sub>pgfplots</sub> | 5 | 5 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Solving a Real Use Case: Functions of Two Variables <sub>pgfplots</sub> | 5 | 5 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.98 |

## What tikra reported

Warnings, by how many figures they appear in (the varying part replaced by …):

| Figures | Warning |
|---:|---|
| 29 | `option … not supported: ignored` |
| 8 | `fill between: … not supported: the whole area is filled` |
| 5 | `axis without data: drew the default window from N to N` |
| 4 | `text outside a statement ignored` |
| 4 | `E-REC-N column N: … without a matching …; closed at the end` |
| 4 | `unrecognized path fragment: …` |
| 3 | `\addplot graphics not supported: it needs an external program or file` |
| 2 | `E-REC-N column N: unsupported text command: \end; written as is` |
| 2 | `E-REC-N column N: unsupported text command: \begin; written as is` |
| 2 | `\draw: figure: unknown coordinate: (…); statement ignored` |
| 2 | `key handler … not supported (…): ignored` |
| 2 | `\tikzgetnamedpath not supported: … ignored` |
| 1 | `E-REC-N column N: unsupported text command: \pgfuseimage; written as is` |
| 1 | `E-REC-N column N: unsupported text command: \pgfimage; written as is` |
| 1 | `environment colormixin not supported: content read as statements` |
| 1 | `\box not supported: … ignored` |
| 1 | `\pgfpositionnodelater not supported: … ignored` |
| 1 | `shape … not supported: drawn as a rectangle` |
| 1 | `\pgfpathcurvebetweentime not supported: … ignored` |
| 1 | `\pgfplotgnuplot not supported: … ignored` |
| 1 | `\pgfpathcircle: figure: not a point: \pgfpointborderrectangle; statement ignored` |
| 1 | `\pgfpathcircle: figure: not a point: \pgfpointborderellipse; statement ignored` |
| 1 | `\pgfpathcircle: figure: not a point: \pgfpointintersectionofcircles; statement ignored` |
| 1 | `\newbox not supported: … ignored` |
| 1 | `E-REC-N column N: unsupported text command: \pgfuseshading; written as is` |
| 1 | `E-REC-N column N: unsupported text command: \node; written as is` |
| 1 | `E-REC-N column N: unsupported text command: \pgfsetfillopacity; written as is` |
| 1 | `E-REC-N column N: unsupported text command: \endpgftransparencygroup; written as is` |
| 1 | `E-REC-N column N: unsupported text command: \pgftransparencygroup; written as is` |
| 1 | `\pgfmathparse: expression: unexpected character: …` |
| 1 | `\draw: figure: unknown anchor: A.input N; statement ignored` |
| 1 | `\draw: figure: unknown anchor: B.output; statement ignored` |
| 1 | `\draw: figure: unknown anchor: B.input N; statement ignored` |
| 1 | `\draw: figure: unknown anchor: A.output; statement ignored` |
| 1 | `E-REC-N column N: \rmoustache is not a delimiter; used the empty one` |
| 1 | `E-REC-N column N: unsupported text command: \-; written as is` |
| 1 | `\pgfsetlinewidth: expression: unknown variable: \hatchlinewidth; statement ignored` |
| 1 | `\pgftransformrotate: expression: unknown variable: \hatchangle; statement ignored` |
| 1 | `\pgfmathrandominteger not supported: … ignored` |
| 1 | `\pgfmathdeclarerandomlist not supported: … ignored` |

Options tikra does not know, in figures that do not match, by how many figures they appear in:

| Figures | Option |
|---:|---|
| 2 | `first char syntax` |
| 2 | `soft clip` |
| 2 | `soft clip path` |
| 1 | `view` |
| 1 | `pin in data` |
| 1 | `logic gate symbol align` |
| 1 | `use IEC style logic gates` |
| 1 | `-leaf` |
| 1 | `(pi>2) circle` |
| 1 | `(pi>2*2) draw` |
| 1 | `(pi>1*2) draw` |
| 1 | `(pi>4) circle` |
| 1 | `(pi>1) circle` |
| 1 | `(pi>4*2) draw` |
| 1 | `(pi>3) circle` |
| 1 | `(pi>3*2) draw` |
| 1 | `transform shape nonlinear` |
| 1 | `pgfpoint substitution` |
| 1 | `symbolic coords` |
| 1 | `trim axis left` |
| 1 | `trim axis right` |
| 1 | `scale mode` |
| 1 | `date coordinates in` |

Commands, environments and path operations tikra does not know, likewise:

| Figures | Command |
|---:|---|
| 2 | `\tikzgetnamedpath` |
| 1 | `colormixin` |
| 1 | `\box` |
| 1 | `\pgfpositionnodelater` |
| 1 | `\pgfplotgnuplot` |
| 1 | `\newbox` |
| 1 | `\pgfmathrandominteger` |
| 1 | `\pgfmathdeclarerandomlist` |
| 1 | `\pgfplotstransformcoordinatex` |
| 1 | `tabular` |
| 1 | `\pgfplotspathaxisoutline` |

Errors of the figures that fail:

| Figures | Error |
|---:|---|
