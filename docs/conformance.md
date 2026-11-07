# Conformance with TikZ

How far tikra is from TikZ itself: every example of the TikZ manual (pgf 3.1.11a) and of the pgfplots
manual (1.18.2), drawn by pdflatex (TeX Live 2025) and by tikra, and compared. How it works and how
to run it: [tools/conformance](../tools/conformance/README.md).

**2464 of 2502 figures (98.4%) match TeX**; 30 are close, 8 differ and 0 fail in tikra. 34 of the ones that do not match came out with no warning.

**Our 100%: 2485 of 2486 (99.9%)**. It leaves out the 16 figures no converter can reproduce (9 with random numbers TeX draws anew on every run, 3 with an image file or external program, 4 with the day's date), and counts as done 24 that differ only by antialiasing. The list, checked by eye: [unreproducible.json](../tools/conformance/unreproducible.json).

| | Figures | Match | Close | Different | Fails | Mean F1 |
|---|---:|---:|---:|---:|---:|---:|
| All | 2502 | 2464 (98.4%) | 30 (1.1%) | 8 (0.3%) | 0 (0.0%) | 1.00 |
| TikZ manual | 1864 | 1836 (98.4%) | 23 (1.2%) | 5 (0.2%) | 0 (0.0%) | 1.00 |
| pgfplots manual | 638 | 628 (98.4%) | 7 (1.0%) | 3 (0.4%) | 0 (0.0%) | 1.00 |

*Match*: same shape (F1 ≥ 0.9 on the edges, with 0.5 mm of tolerance), no missing or extra piece, the same colors and size. *Close*: F1 ≥ 0.75.
*Fails*: tikra returned an error, panicked or took more than 10 s. Mean F1 counts a failure as 0.

Not counted: 137 where tex could not compile it, 19 where tex draws nothing visible, and 479 examples that are not figures (text, tables, key listings).

## Speed

The same 192 figures (a fixed sample of the manuals), one at a time, on Darwin arm64:

| | Median per figure | p95 |
|---|---:|---:|
| pdflatex (texlive/texlive:latest) | 647 ms | 1031 ms |
| tikra, TikZ to SVG | 0.85 ms | 6.85 ms |

Median ratio: tikra is **846× faster**. pdflatex time includes starting TeX and loading the libraries of the manual, which is what a user
waits for; tikra's is the conversion inside a running process: the first figure of a process also builds the engine, 26.9 ms more.

## By chapter

Sorted by how many figures do not match, the work left.

| Chapter | Figures | Match | Close | Different | Fails | Mean F1 |
|---|---:|---:|---:|---:|---:|---:|
| Pattern Library <sub>tikz</sub> | 8 | 5 (62.5%) | 3 (37.5%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Actions on Paths <sub>tikz</sub> | 72 | 69 (95.8%) | 3 (4.1%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Axis Descriptions <sub>pgfplots</sub> | 77 | 74 (96.1%) | 2 (2.5%) | 1 (1.2%) | 0 (0.0%) | 0.99 |
| Using Paths <sub>tikz</sub> | 15 | 13 (86.6%) | 2 (13.3%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Declaring and Using Images <sub>tikz</sub> | 2 | 0 (0.0%) | 1 (50.0%) | 1 (50.0%) | 0 (0.0%) | 0.51 |
| Transparency <sub>tikz</sub> | 47 | 45 (95.7%) | 2 (4.2%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Style Sheets and Legends <sub>tikz</sub> | 85 | 83 (97.6%) | 2 (2.3%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Math Library <sub>tikz</sub> | 6 | 4 (66.6%) | 1 (16.6%) | 1 (16.6%) | 0 (0.0%) | 0.92 |
| Additional Mathematical Commands <sub>tikz</sub> | 3 | 1 (33.3%) | 0 (0.0%) | 2 (66.6%) | 0 (0.0%) | 0.73 |
| Animations <sub>tikz</sub> | 24 | 23 (95.8%) | 1 (4.1%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Creating Plots <sub>tikz</sub> | 7 | 6 (85.7%) | 1 (14.2%) | 0 (0.0%) | 0 (0.0%) | 0.97 |
| Specifying Coordinates <sub>tikz</sub> | 68 | 67 (98.5%) | 1 (1.4%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Calendar Library <sub>tikz</sub> | 41 | 40 (97.5%) | 0 (0.0%) | 1 (2.4%) | 0 (0.0%) | 0.99 |
| Circuit Libraries <sub>tikz</sub> | 64 | 63 (98.4%) | 1 (1.5%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Decoration Library <sub>tikz</sub> | 81 | 80 (98.7%) | 1 (1.2%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Fitting Library <sub>tikz</sub> | 4 | 3 (75.0%) | 1 (25.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Lindenmayer System Drawing Library <sub>tikz</sub> | 6 | 5 (83.3%) | 1 (16.6%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Shape Library <sub>tikz</sub> | 97 | 96 (98.9%) | 1 (1.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Arrows <sub>tikz</sub> | 104 | 103 (99.0%) | 1 (0.9%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Fill between <sub>pgfplots</sub> | 29 | 28 (96.5%) | 1 (3.4%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Statistics <sub>pgfplots</sub> | 26 | 25 (96.1%) | 1 (3.8%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Two Dimensional Plot Types <sub>pgfplots</sub> | 72 | 71 (98.6%) | 0 (0.0%) | 1 (1.3%) | 0 (0.0%) | 0.99 |
| 3D Axis Configuration <sub>pgfplots</sub> | 13 | 12 (92.3%) | 1 (7.6%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Three Dimensional Plot Types <sub>pgfplots</sub> | 85 | 84 (98.8%) | 1 (1.1%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Grid Options <sub>pgfplots</sub> | 18 | 17 (94.4%) | 0 (0.0%) | 1 (5.5%) | 0 (0.0%) | 0.98 |
| Specifying the Plotted Range <sub>pgfplots</sub> | 13 | 12 (92.3%) | 1 (7.6%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Decorations <sub>tikz</sub> | 10 | 10 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Design Principles <sub>tikz</sub> | 7 | 7 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Layered Graphics <sub>tikz</sub> | 1 | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Matrices <sub>tikz</sub> | 7 | 7 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Nodes and Shapes <sub>tikz</sub> | 12 | 12 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Constructing Paths <sub>tikz</sub> | 26 | 26 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Patterns <sub>tikz</sub> | 4 | 4 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Quick Commands <sub>tikz</sub> | 3 | 3 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| pgfmanual-en-base-scopes <sub>tikz</sub> | 19 | 19 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Shadings <sub>tikz</sub> | 6 | 6 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Coordinate, Canvas, and Nonlinear Transformations <sub>tikz</sub> | 38 | 38 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Axes <sub>tikz</sub> | 78 | 78 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| pgfmanual-en-dv-examples <sub>tikz</sub> | 3 | 3 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Providing Data for a Data Visualization <sub>tikz</sub> | 8 | 8 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Introduction to Data Visualization <sub>tikz</sub> | 2 | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Creating Data Visualizations <sub>tikz</sub> | 13 | 13 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Polar Axes <sub>tikz</sub> | 11 | 11 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Visualizers <sub>tikz</sub> | 18 | 18 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Three Dimensional Drawing Library <sub>tikz</sub> | 5 | 5 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Angle Library <sub>tikz</sub> | 6 | 6 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Automata Drawing Library <sub>tikz</sub> | 8 | 8 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Babel Library <sub>tikz</sub> | 2 | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Background Library <sub>tikz</sub> | 12 | 12 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Chains <sub>tikz</sub> | 14 | 14 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| To Path Library <sub>tikz</sub> | 13 | 13 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Entity-Relationship Diagram Drawing Library <sub>tikz</sub> | 7 | 7 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Paper-Folding Diagrams Library <sub>tikz</sub> | 18 | 18 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Matrix Library <sub>tikz</sub> | 14 | 14 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Mindmap Drawing Library <sub>tikz</sub> | 16 | 16 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Three Point Perspective Drawing Library <sub>tikz</sub> | 15 | 15 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Petri-Net Drawing Library <sub>tikz</sub> | 13 | 13 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Plot Handler Library <sub>tikz</sub> | 23 | 23 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Resource Description Framework Library <sub>tikz</sub> | 2 | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Shadings Library <sub>tikz</sub> | 11 | 11 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Shadows Library <sub>tikz</sub> | 14 | 14 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Spy Library: Magnifying Parts of Pictures <sub>tikz</sub> | 11 | 11 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| SVG-Path Library <sub>tikz</sub> | 1 | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Through Library <sub>tikz</sub> | 1 | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Tree Library <sub>tikz</sub> | 7 | 7 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Turtle Graphics Library <sub>tikz</sub> | 4 | 4 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Views Library <sub>tikz</sub> | 4 | 4 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Tutorials and Guidelines <sub>tikz</sub> | 9 | 9 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Customizing the Mathematical Engine <sub>tikz</sub> | 2 | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Mathematical Expressions <sub>tikz</sub> | 3 | 3 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Date and Calendar Utility Macros <sub>tikz</sub> | 1 | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Repeating Things: The Foreach Statement <sub>tikz</sub> | 13 | 13 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Key Management <sub>tikz</sub> | 5 | 5 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Animation System Layer <sub>tikz</sub> | 1 | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Decorated Paths <sub>tikz</sub> | 31 | 31 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Specifying Graphs <sub>tikz</sub> | 159 | 159 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Matrices and Alignment <sub>tikz</sub> | 26 | 26 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Syntax for Path Specifications <sub>tikz</sub> | 52 | 52 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Pics: Small Pictures on Paths <sub>tikz</sub> | 14 | 14 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Plots of Functions <sub>tikz</sub> | 30 | 30 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| pgfmanual-en-tikz-scopes <sub>tikz</sub> | 13 | 13 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Nodes and Edges <sub>tikz</sub> | 118 | 118 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Transformations <sub>tikz</sub> | 22 | 22 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Making Trees Grow <sub>tikz</sub> | 27 | 27 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Tutorial: Euclid's Amber Version of the Elements <sub>tikz</sub> | 14 | 14 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Tutorial: Diagrams as Simple Graphs <sub>tikz</sub> | 19 | 19 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Tutorial: A Lecture Map for Johannes <sub>tikz</sub> | 10 | 10 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Tutorial: A Petri-Net for Hagen <sub>tikz</sub> | 28 | 28 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Tutorial: A Picture for Karl's Students <sub>tikz</sub> | 46 | 46 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Utilities and Basic Level Commands <sub>pgfplots</sub> | 7 | 7 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Decoration: Soft Clipping <sub>pgfplots</sub> | 1 | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Grouping plots <sub>pgfplots</sub> | 8 | 8 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Patchplots Library <sub>pgfplots</sub> | 29 | 29 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Polar Axes <sub>pgfplots</sub> | 13 | 13 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |
| Smith Charts <sub>pgfplots</sub> | 12 | 12 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Ternary Diagrams <sub>pgfplots</sub> | 13 | 13 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Units in Labels <sub>pgfplots</sub> | 2 | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Components <sub>pgfplots</sub> | 1 | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.98 |
| Alignment Options <sub>pgfplots</sub> | 8 | 8 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| The Axis Environments <sub>pgfplots</sub> | 21 | 21 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| The Picture's Size: Bounding Box and Clipping <sub>pgfplots</sub> | 5 | 5 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| pgfplots.reference.closingplots <sub>pgfplots</sub> | 6 | 6 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Skipping Or Changing Coordinates -- Filters <sub>pgfplots</sub> | 7 | 7 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Error Bars <sub>pgfplots</sub> | 9 | 9 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Fitting Lines -- Regression <sub>pgfplots</sub> | 5 | 5 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Markers, Linestyles, (Background) Colors and Colormaps <sub>pgfplots</sub> | 50 | 50 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Miscellaneous Options <sub>pgfplots</sub> | 7 | 7 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Number Formatting Options <sub>pgfplots</sub> | 6 | 6 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Scaling Options <sub>pgfplots</sub> | 22 | 22 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Style Options <sub>pgfplots</sub> | 2 | 2 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Symbolic Coordinates and User Transformations <sub>pgfplots</sub> | 5 | 5 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Tick Options <sub>pgfplots</sub> | 32 | 32 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Interoperability <sub>pgfplots</sub> | 3 | 3 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Transforming Coordinate Systems <sub>pgfplots</sub> | 4 | 4 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Memory and Speed considerations <sub>pgfplots</sub> | 4 | 4 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Solving a Real Use Case: Function Visualization <sub>pgfplots</sub> | 6 | 6 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Solving a Real Use Case: Scientific Data Analysis <sub>pgfplots</sub> | 7 | 7 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Use Cases involving Scatter Plots <sub>pgfplots</sub> | 5 | 5 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 1.00 |
| Solving a Real Use Case: Functions of Two Variables <sub>pgfplots</sub> | 5 | 5 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0.99 |

## What tikra reported

Warnings, by how many figures they appear in (the varying part replaced by …):

| Figures | Warning |
|---:|---|
| 11 | `option … not supported: ignored` |
| 3 | `text outside a statement ignored` |
| 3 | `E-REC-N column N: … without a matching …; closed at the end` |
| 2 | `image not drawn: file … not found` |
| 2 | `\draw: figure: unknown coordinate: (…); statement ignored` |
| 2 | `key handler … not supported (…): ignored` |
| 2 | `fill between: soft clip … not understood: ignored` |
| 2 | `axis without data: drew the default window from N to N` |
| 1 | `environment colormixin not supported: content read as statements` |
| 1 | `\pgfpathcurvebetweentime not supported: … ignored` |
| 1 | `\pgfplotgnuplot not supported: … ignored` |
| 1 | `\pgfpathcircle: figure: not a point: \pgfpointborderrectangle; statement ignored` |
| 1 | `\pgfpathcircle: figure: not a point: \pgfpointborderellipse; statement ignored` |
| 1 | `\pgfpathcircle: figure: not a point: \pgfpointintersectionofcircles; statement ignored` |
| 1 | `\pgfqbox{\mybox}: the box is not drawn` |
| 1 | `E-REC-N column N: unsupported text command: \end; written as is` |
| 1 | `unrecognized path fragment: …` |
| 1 | `\pgftransparencygroup not supported: … ignored` |
| 1 | `E-REC-N column N: unsupported text command: \begin; written as is` |
| 1 | `\endpgftransparencygroup not supported: … ignored` |
| 1 | `E-REC-N column N: unsupported text command: \pgftransparencygroup; written as is` |
| 1 | `E-REC-N column N: unsupported text command: \pgfsetfillopacity; written as is` |
| 1 | `E-REC-N column N: unsupported text command: \endpgftransparencygroup; written as is` |
| 1 | `E-REC-N column N: unsupported text command: \node; written as is` |
| 1 | `\pgfmathparse: expression: unexpected character: …` |
| 1 | `E-REC-N column N: unsupported text command: \-; written as is` |
| 1 | `unknown color …` |
| 1 | `soft clip path … not understood: path drawn unclipped` |

Options tikra does not know, in figures that do not match, by how many figures they appear in:

| Figures | Option |
|---:|---|

Commands, environments and path operations tikra does not know, likewise:

| Figures | Command |
|---:|---|
| 1 | `colormixin` |
| 1 | `\pgfplotgnuplot` |

Errors of the figures that fail:

| Figures | Error |
|---:|---|
