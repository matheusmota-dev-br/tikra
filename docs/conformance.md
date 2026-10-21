# Conformance with TikZ

How far tikra is from TikZ itself: every example of the TikZ manual (pgf 3.1.11a) and of the pgfplots
manual (1.18.2), drawn by pdflatex (TeX Live 2025) and by tikra, and compared. How it works and how
to run it: [tools/conformance](../tools/conformance/README.md).

**86 of 2448 figures (3.5%) match TeX**; 596 are close, 1759 differ and 7 fail in tikra. 618 of the ones that do not match came out with no warning.

| | Figures | Match | Close | Different | Fails | Mean F1 |
|---|---:|---:|---:|---:|---:|---:|
| All | 2448 | 86 (3.5%) | 596 (24.3%) | 1759 (71.9%) | 7 (0.3%) | 0.37 |
| TikZ manual | 1810 | 82 (4.5%) | 581 (32.1%) | 1141 (63.0%) | 6 (0.3%) | 0.44 |
| pgfplots manual | 638 | 4 (0.6%) | 15 (2.4%) | 618 (96.9%) | 1 (0.2%) | 0.16 |

*Match*: same shape (F1 ≥ 0.9 on the edges, with 0.5 mm of tolerance), no missing or extra piece, the same colors and size. *Close*: F1 ≥ 0.75.
*Fails*: tikra returned an error, panicked or took more than 10 s. Mean F1 counts a failure as 0.

Not counted: 136 where tex could not compile it, 18 where tex draws nothing visible, and 535 examples that are not TikZ figures (pgf basic layer, text, tables).

## Speed

The same 192 figures (a fixed sample of the manuals), one at a time, on Darwin arm64:

| | Median per figure | p95 |
|---|---:|---:|
| pdflatex (texlive/texlive:latest) | 647 ms | 1031 ms |
| tikra, TikZ to SVG | 0.45 ms | 0.63 ms |

Median ratio: tikra is **1409× faster**. pdflatex time includes starting TeX and loading the libraries of the manual, which is what a user
waits for; tikra's is the conversion inside a running process (the command line adds a few ms to start).

## By chapter

Sorted by how many figures do not match, the work left.

| Chapter | Figures | Match | Close | Different | Fails | Mean F1 |
|---|---:|---:|---:|---:|---:|---:|
| Specifying Graphs <sub>tikz</sub> | 159 | 0 (0.0%) | 2 (1.3%) | 156 (98.1%) | 1 (0.6%) | 0.02 |
| Nodes and Edges <sub>tikz</sub> | 118 | 2 (1.7%) | 71 (60.2%) | 45 (38.1%) | 0 (0.0%) | 0.73 |
| Shape Library <sub>tikz</sub> | 97 | 1 (1.0%) | 16 (16.5%) | 80 (82.5%) | 0 (0.0%) | 0.41 |
| Arrows <sub>tikz</sub> | 104 | 13 (12.5%) | 79 (76.0%) | 12 (11.5%) | 0 (0.0%) | 0.88 |
| Style Sheets and Legends <sub>tikz</sub> | 85 | 0 (0.0%) | 0 (0.0%) | 85 (100.0%) | 0 (0.0%) | 0.00 |
| Three Dimensional Plot Types <sub>pgfplots</sub> | 85 | 0 (0.0%) | 0 (0.0%) | 85 (100.0%) | 0 (0.0%) | 0.10 |
| Axes <sub>tikz</sub> | 78 | 0 (0.0%) | 1 (1.3%) | 77 (98.7%) | 0 (0.0%) | 0.01 |
| Axis Descriptions <sub>pgfplots</sub> | 77 | 0 (0.0%) | 0 (0.0%) | 77 (100.0%) | 0 (0.0%) | 0.17 |
| Decoration Library <sub>tikz</sub> | 81 | 5 (6.2%) | 33 (40.7%) | 43 (53.1%) | 0 (0.0%) | 0.55 |
| Two Dimensional Plot Types <sub>pgfplots</sub> | 72 | 0 (0.0%) | 0 (0.0%) | 72 (100.0%) | 0 (0.0%) | 0.13 |
| Specifying Coordinates <sub>tikz</sub> | 66 | 2 (3.0%) | 48 (72.7%) | 16 (24.2%) | 0 (0.0%) | 0.73 |
| Circuit Libraries <sub>tikz</sub> | 64 | 0 (0.0%) | 9 (14.1%) | 55 (85.9%) | 0 (0.0%) | 0.37 |
| Actions on Paths <sub>tikz</sub> | 72 | 12 (16.7%) | 46 (63.9%) | 14 (19.4%) | 0 (0.0%) | 0.86 |
| Syntax for Path Specifications <sub>tikz</sub> | 52 | 1 (1.9%) | 39 (75.0%) | 12 (23.1%) | 0 (0.0%) | 0.80 |
| Markers, Linestyles, (Background) Colors and Colormaps <sub>pgfplots</sub> | 50 | 3 (6.0%) | 8 (16.0%) | 39 (78.0%) | 0 (0.0%) | 0.40 |
| Tutorial: A Picture for Karl's Students <sub>tikz</sub> | 46 | 3 (6.5%) | 27 (58.7%) | 16 (34.8%) | 0 (0.0%) | 0.67 |
| Calendar Library <sub>tikz</sub> | 41 | 0 (0.0%) | 0 (0.0%) | 41 (100.0%) | 0 (0.0%) | 0.01 |
| Coordinate, Canvas, and Nonlinear Transformations <sub>tikz</sub> | 38 | 1 (2.6%) | 27 (71.1%) | 10 (26.3%) | 0 (0.0%) | 0.76 |
| Transparency <sub>tikz</sub> | 46 | 11 (23.9%) | 14 (30.4%) | 21 (45.7%) | 0 (0.0%) | 0.72 |
| Tick Options <sub>pgfplots</sub> | 32 | 0 (0.0%) | 0 (0.0%) | 32 (100.0%) | 0 (0.0%) | 0.15 |
| Plots of Functions <sub>tikz</sub> | 30 | 0 (0.0%) | 5 (16.7%) | 25 (83.3%) | 0 (0.0%) | 0.28 |
| Fill between <sub>pgfplots</sub> | 29 | 0 (0.0%) | 6 (20.7%) | 23 (79.3%) | 0 (0.0%) | 0.31 |
| Patchplots Library <sub>pgfplots</sub> | 29 | 0 (0.0%) | 0 (0.0%) | 29 (100.0%) | 0 (0.0%) | 0.16 |
| Decorated Paths <sub>tikz</sub> | 31 | 4 (12.9%) | 14 (45.2%) | 13 (41.9%) | 0 (0.0%) | 0.67 |
| Making Trees Grow <sub>tikz</sub> | 27 | 0 (0.0%) | 2 (7.4%) | 25 (92.6%) | 0 (0.0%) | 0.26 |
| Matrices and Alignment <sub>tikz</sub> | 26 | 0 (0.0%) | 1 (3.8%) | 25 (96.2%) | 0 (0.0%) | 0.04 |
| Statistics <sub>pgfplots</sub> | 26 | 0 (0.0%) | 0 (0.0%) | 26 (100.0%) | 0 (0.0%) | 0.15 |
| Animations <sub>tikz</sub> | 24 | 0 (0.0%) | 5 (20.8%) | 19 (79.2%) | 0 (0.0%) | 0.29 |
| Plot Handler Library <sub>tikz</sub> | 23 | 0 (0.0%) | 3 (13.0%) | 20 (87.0%) | 0 (0.0%) | 0.46 |
| Scaling Options <sub>pgfplots</sub> | 22 | 0 (0.0%) | 0 (0.0%) | 22 (100.0%) | 0 (0.0%) | 0.14 |
| The Axis Environments <sub>pgfplots</sub> | 21 | 0 (0.0%) | 0 (0.0%) | 21 (100.0%) | 0 (0.0%) | 0.15 |
| Transformations <sub>tikz</sub> | 22 | 2 (9.1%) | 19 (86.4%) | 1 (4.5%) | 0 (0.0%) | 0.92 |
| Tutorial: Diagrams as Simple Graphs <sub>tikz</sub> | 19 | 0 (0.0%) | 9 (47.4%) | 10 (52.6%) | 0 (0.0%) | 0.51 |
| Visualizers <sub>tikz</sub> | 18 | 0 (0.0%) | 0 (0.0%) | 18 (100.0%) | 0 (0.0%) | 0.00 |
| Paper-Folding Diagrams Library <sub>tikz</sub> | 18 | 0 (0.0%) | 0 (0.0%) | 18 (100.0%) | 0 (0.0%) | 0.00 |
| Grid Options <sub>pgfplots</sub> | 18 | 0 (0.0%) | 0 (0.0%) | 18 (100.0%) | 0 (0.0%) | 0.14 |
| Constructing Paths <sub>tikz</sub> | 16 | 0 (0.0%) | 10 (62.5%) | 6 (37.5%) | 0 (0.0%) | 0.68 |
| Mindmap Drawing Library <sub>tikz</sub> | 16 | 1 (6.2%) | 2 (12.5%) | 13 (81.2%) | 0 (0.0%) | 0.25 |
| Three Point Perspective Drawing Library <sub>tikz</sub> | 15 | 0 (0.0%) | 0 (0.0%) | 15 (100.0%) | 0 (0.0%) | 0.23 |
| Chains <sub>tikz</sub> | 14 | 0 (0.0%) | 1 (7.1%) | 13 (92.9%) | 0 (0.0%) | 0.40 |
| Matrix Library <sub>tikz</sub> | 14 | 0 (0.0%) | 0 (0.0%) | 14 (100.0%) | 0 (0.0%) | 0.05 |
| Shadows Library <sub>tikz</sub> | 14 | 0 (0.0%) | 11 (78.6%) | 3 (21.4%) | 0 (0.0%) | 0.84 |
| Pics: Small Pictures on Paths <sub>tikz</sub> | 14 | 0 (0.0%) | 3 (21.4%) | 11 (78.6%) | 0 (0.0%) | 0.19 |
| Tutorial: Euclid's Amber Version of the Elements <sub>tikz</sub> | 14 | 0 (0.0%) | 3 (21.4%) | 11 (78.6%) | 0 (0.0%) | 0.46 |
| Creating Data Visualizations <sub>tikz</sub> | 13 | 0 (0.0%) | 0 (0.0%) | 13 (100.0%) | 0 (0.0%) | 0.00 |
| To Path Library <sub>tikz</sub> | 13 | 0 (0.0%) | 7 (53.8%) | 6 (46.2%) | 0 (0.0%) | 0.72 |
| Petri-Net Drawing Library <sub>tikz</sub> | 13 | 0 (0.0%) | 0 (0.0%) | 13 (100.0%) | 0 (0.0%) | 0.19 |
| Polar Axes <sub>pgfplots</sub> | 13 | 0 (0.0%) | 0 (0.0%) | 13 (100.0%) | 0 (0.0%) | 0.00 |
| Ternary Diagrams <sub>pgfplots</sub> | 13 | 0 (0.0%) | 0 (0.0%) | 13 (100.0%) | 0 (0.0%) | 0.01 |
| 3D Axis Configuration <sub>pgfplots</sub> | 13 | 0 (0.0%) | 0 (0.0%) | 13 (100.0%) | 0 (0.0%) | 0.10 |
| Specifying the Plotted Range <sub>pgfplots</sub> | 13 | 0 (0.0%) | 0 (0.0%) | 13 (100.0%) | 0 (0.0%) | 0.13 |
| Background Library <sub>tikz</sub> | 12 | 0 (0.0%) | 1 (8.3%) | 11 (91.7%) | 0 (0.0%) | 0.56 |
| Repeating Things: The Foreach Statement <sub>tikz</sub> | 13 | 1 (7.7%) | 7 (53.8%) | 5 (38.5%) | 0 (0.0%) | 0.64 |
| Smith Charts <sub>pgfplots</sub> | 12 | 0 (0.0%) | 0 (0.0%) | 12 (100.0%) | 0 (0.0%) | 0.00 |
| pgfmanual-en-base-scopes <sub>tikz</sub> | 11 | 0 (0.0%) | 8 (72.7%) | 3 (27.3%) | 0 (0.0%) | 0.78 |
| Polar Axes <sub>tikz</sub> | 11 | 0 (0.0%) | 0 (0.0%) | 11 (100.0%) | 0 (0.0%) | 0.00 |
| Shadings Library <sub>tikz</sub> | 11 | 0 (0.0%) | 10 (90.9%) | 1 (9.1%) | 0 (0.0%) | 0.90 |
| Spy Library: Magnifying Parts of Pictures <sub>tikz</sub> | 11 | 0 (0.0%) | 0 (0.0%) | 11 (100.0%) | 0 (0.0%) | 0.06 |
| Decorations <sub>tikz</sub> | 10 | 0 (0.0%) | 4 (40.0%) | 5 (50.0%) | 1 (10.0%) | 0.43 |
| Tutorial: A Lecture Map for Johannes <sub>tikz</sub> | 10 | 0 (0.0%) | 0 (0.0%) | 10 (100.0%) | 0 (0.0%) | 0.05 |
| Tutorial: A Petri-Net for Hagen <sub>tikz</sub> | 28 | 18 (64.3%) | 3 (10.7%) | 7 (25.0%) | 0 (0.0%) | 0.85 |
| pgfmanual-en-tikz-scopes <sub>tikz</sub> | 13 | 4 (30.8%) | 5 (38.5%) | 4 (30.8%) | 0 (0.0%) | 0.83 |
| Error Bars <sub>pgfplots</sub> | 9 | 0 (0.0%) | 0 (0.0%) | 9 (100.0%) | 0 (0.0%) | 0.13 |
| Nodes and Shapes <sub>tikz</sub> | 8 | 0 (0.0%) | 3 (37.5%) | 3 (37.5%) | 2 (25.0%) | 0.54 |
| Providing Data for a Data Visualization <sub>tikz</sub> | 8 | 0 (0.0%) | 0 (0.0%) | 8 (100.0%) | 0 (0.0%) | 0.00 |
| Automata Drawing Library <sub>tikz</sub> | 8 | 0 (0.0%) | 0 (0.0%) | 8 (100.0%) | 0 (0.0%) | 0.28 |
| Pattern Library <sub>tikz</sub> | 8 | 0 (0.0%) | 8 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0.94 |
| Tutorials and Guidelines <sub>tikz</sub> | 9 | 1 (11.1%) | 4 (44.4%) | 4 (44.4%) | 0 (0.0%) | 0.60 |
| Grouping plots <sub>pgfplots</sub> | 8 | 0 (0.0%) | 0 (0.0%) | 8 (100.0%) | 0 (0.0%) | 0.01 |
| Alignment Options <sub>pgfplots</sub> | 8 | 0 (0.0%) | 0 (0.0%) | 8 (100.0%) | 0 (0.0%) | 0.15 |
| Matrices <sub>tikz</sub> | 7 | 0 (0.0%) | 0 (0.0%) | 7 (100.0%) | 0 (0.0%) | 0.00 |
| Entity-Relationship Diagram Drawing Library <sub>tikz</sub> | 7 | 0 (0.0%) | 0 (0.0%) | 7 (100.0%) | 0 (0.0%) | 0.41 |
| Tree Library <sub>tikz</sub> | 7 | 0 (0.0%) | 0 (0.0%) | 7 (100.0%) | 0 (0.0%) | 0.24 |
| Utilities and Basic Level Commands <sub>pgfplots</sub> | 7 | 0 (0.0%) | 0 (0.0%) | 7 (100.0%) | 0 (0.0%) | 0.15 |
| Skipping Or Changing Coordinates -- Filters <sub>pgfplots</sub> | 7 | 0 (0.0%) | 0 (0.0%) | 7 (100.0%) | 0 (0.0%) | 0.12 |
| Miscellaneous Options <sub>pgfplots</sub> | 7 | 0 (0.0%) | 0 (0.0%) | 7 (100.0%) | 0 (0.0%) | 0.13 |
| Solving a Real Use Case: Scientific Data Analysis <sub>pgfplots</sub> | 7 | 0 (0.0%) | 0 (0.0%) | 7 (100.0%) | 0 (0.0%) | 0.16 |
| Angle Library <sub>tikz</sub> | 6 | 0 (0.0%) | 3 (50.0%) | 3 (50.0%) | 0 (0.0%) | 0.70 |
| Lindenmayer System Drawing Library <sub>tikz</sub> | 6 | 0 (0.0%) | 1 (16.7%) | 5 (83.3%) | 0 (0.0%) | 0.14 |
| Math Library <sub>tikz</sub> | 6 | 0 (0.0%) | 0 (0.0%) | 6 (100.0%) | 0 (0.0%) | 0.24 |
| Design Principles <sub>tikz</sub> | 6 | 0 (0.0%) | 3 (50.0%) | 3 (50.0%) | 0 (0.0%) | 0.53 |
| pgfplots.reference.closingplots <sub>pgfplots</sub> | 6 | 0 (0.0%) | 0 (0.0%) | 6 (100.0%) | 0 (0.0%) | 0.12 |
| Number Formatting Options <sub>pgfplots</sub> | 6 | 0 (0.0%) | 0 (0.0%) | 6 (100.0%) | 0 (0.0%) | 0.22 |
| Solving a Real Use Case: Function Visualization <sub>pgfplots</sub> | 6 | 0 (0.0%) | 0 (0.0%) | 6 (100.0%) | 0 (0.0%) | 0.10 |
| Three Dimensional Drawing Library <sub>tikz</sub> | 5 | 0 (0.0%) | 1 (20.0%) | 4 (80.0%) | 0 (0.0%) | 0.27 |
| Key Management <sub>tikz</sub> | 5 | 0 (0.0%) | 2 (40.0%) | 3 (60.0%) | 0 (0.0%) | 0.45 |
| The Picture's Size: Bounding Box and Clipping <sub>pgfplots</sub> | 5 | 0 (0.0%) | 0 (0.0%) | 5 (100.0%) | 0 (0.0%) | 0.12 |
| Fitting Lines -- Regression <sub>pgfplots</sub> | 5 | 0 (0.0%) | 0 (0.0%) | 5 (100.0%) | 0 (0.0%) | 0.15 |
| Symbolic Coordinates and User Transformations <sub>pgfplots</sub> | 5 | 0 (0.0%) | 0 (0.0%) | 4 (80.0%) | 1 (20.0%) | 0.09 |
| Use Cases involving Scatter Plots <sub>pgfplots</sub> | 5 | 0 (0.0%) | 0 (0.0%) | 5 (100.0%) | 0 (0.0%) | 0.11 |
| Solving a Real Use Case: Functions of Two Variables <sub>pgfplots</sub> | 5 | 0 (0.0%) | 0 (0.0%) | 5 (100.0%) | 0 (0.0%) | 0.10 |
| Patterns <sub>tikz</sub> | 4 | 0 (0.0%) | 2 (50.0%) | 2 (50.0%) | 0 (0.0%) | 0.59 |
| Fitting Library <sub>tikz</sub> | 4 | 0 (0.0%) | 1 (25.0%) | 3 (75.0%) | 0 (0.0%) | 0.45 |
| Turtle Graphics Library <sub>tikz</sub> | 4 | 0 (0.0%) | 0 (0.0%) | 4 (100.0%) | 0 (0.0%) | 0.00 |
| Views Library <sub>tikz</sub> | 4 | 0 (0.0%) | 2 (50.0%) | 2 (50.0%) | 0 (0.0%) | 0.73 |
| Transforming Coordinate Systems <sub>pgfplots</sub> | 4 | 0 (0.0%) | 0 (0.0%) | 4 (100.0%) | 0 (0.0%) | 0.08 |
| Memory and Speed considerations <sub>pgfplots</sub> | 4 | 0 (0.0%) | 0 (0.0%) | 4 (100.0%) | 0 (0.0%) | 0.05 |
| Creating Plots <sub>tikz</sub> | 3 | 0 (0.0%) | 0 (0.0%) | 3 (100.0%) | 0 (0.0%) | 0.17 |
| Quick Commands <sub>tikz</sub> | 3 | 0 (0.0%) | 2 (66.7%) | 1 (33.3%) | 0 (0.0%) | 0.72 |
| pgfmanual-en-dv-examples <sub>tikz</sub> | 3 | 0 (0.0%) | 0 (0.0%) | 3 (100.0%) | 0 (0.0%) | 0.00 |
| Mathematical Expressions <sub>tikz</sub> | 3 | 0 (0.0%) | 1 (33.3%) | 2 (66.7%) | 0 (0.0%) | 0.56 |
| Shadings <sub>tikz</sub> | 2 | 0 (0.0%) | 0 (0.0%) | 2 (100.0%) | 0 (0.0%) | 0.34 |
| Introduction to Data Visualization <sub>tikz</sub> | 2 | 0 (0.0%) | 0 (0.0%) | 2 (100.0%) | 0 (0.0%) | 0.00 |
| Babel Library <sub>tikz</sub> | 2 | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 2 (100.0%) | 0.00 |
| Resource Description Framework Library <sub>tikz</sub> | 2 | 0 (0.0%) | 0 (0.0%) | 2 (100.0%) | 0 (0.0%) | 0.00 |
| Customizing the Mathematical Engine <sub>tikz</sub> | 2 | 0 (0.0%) | 1 (50.0%) | 1 (50.0%) | 0 (0.0%) | 0.76 |
| Units in Labels <sub>pgfplots</sub> | 2 | 0 (0.0%) | 0 (0.0%) | 2 (100.0%) | 0 (0.0%) | 0.16 |
| Style Options <sub>pgfplots</sub> | 2 | 0 (0.0%) | 0 (0.0%) | 2 (100.0%) | 0 (0.0%) | 0.14 |
| Interoperability <sub>pgfplots</sub> | 3 | 1 (33.3%) | 0 (0.0%) | 2 (66.7%) | 0 (0.0%) | 0.39 |
| Layered Graphics <sub>tikz</sub> | 1 | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0.89 |
| Through Library <sub>tikz</sub> | 1 | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0.81 |
| Additional Mathematical Commands <sub>tikz</sub> | 1 | 0 (0.0%) | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0.00 |
| Date and Calendar Utility Macros <sub>tikz</sub> | 1 | 0 (0.0%) | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0.00 |
| Animation System Layer <sub>tikz</sub> | 1 | 0 (0.0%) | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0.00 |
| Decoration: Soft Clipping <sub>pgfplots</sub> | 1 | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0 (0.0%) | 0.93 |
| Components <sub>pgfplots</sub> | 1 | 0 (0.0%) | 0 (0.0%) | 1 (100.0%) | 0 (0.0%) | 0.09 |

## What tikra reported

Warnings, by how many figures they appear in (the varying part replaced by …):

| Figures | Warning |
|---:|---|
| 566 | `option … not supported: ignored` |
| 254 | `plot without data: drew the default window from N to N` |
| 219 | `\datavisualization not supported: … ignored` |
| 170 | `\graph is not supported: ignored` |
| 151 | `more than one tikzpicture: only the first was drawn` |
| 108 | `path operation … not supported: rest of the path ignored` |
| 78 | `decoration option … not supported: ignored` |
| 57 | `text outside a statement ignored` |
| 57 | `decoration … not supported: path drawn undecorated` |
| 55 | `\draw: figure: unknown coordinate: (…); statement ignored` |
| 51 | `\matrix is not supported: ignored` |
| 44 | `shape … not supported: drawn as a rectangle` |
| 41 | `\calendar not supported: … ignored` |
| 35 | `unrecognized path fragment: …` |
| 35 | `\addplot not supported: … ignored` |
| 29 | `E-REC-N column N: unsupported text command: \vrule; written as is` |
| 25 | `\pic is not supported: ignored` |
| 24 | `\pgfpathmoveto not supported: … ignored` |
| 20 | `node without text: … missing` |
| 18 | `key handler … not supported (…): ignored` |
| 17 | `E-REC-N column N: unsupported text command: \nodepart; written as is` |
| 14 | `\draw: figure: plot file (…) not supported; statement ignored` |
| 14 | `environment polaraxis not supported: content read as statements` |
| 13 | `\pgftext not supported: … ignored` |
| 13 | `environment ternaryaxis not supported: content read as statements` |
| 12 | `\spy not supported: … ignored` |
| 12 | `environment smithchart not supported: content read as statements` |
| 11 | `\pgfpathcircle not supported: … ignored` |
| 11 | `\draw: figure: unmatched bracket in (…); statement ignored` |
| 11 | `\fill: figure: coordinate system … not supported; statement ignored` |
| 9 | `\draw: figure: unknown anchor: s.output; statement ignored` |
| 9 | `\fill: figure: unknown coordinate: (…); statement ignored` |
| 8 | `environment groupplot not supported: content read as statements` |
| 8 | `\nextgroupplot not supported: … ignored` |
| 7 | `\pgfsetlinewidth not supported: … ignored` |
| 7 | `\draw: figure: unknown anchor: s.input; statement ignored` |
| 6 | `\pgftransformrotate not supported: … ignored` |
| 6 | `\pgftransformnonlinear not supported: … ignored` |
| 6 | `\path: figure: unknown coordinate: (…); statement ignored` |
| 6 | `unknown color …` |

Options tikra does not know, in figures that do not match, by how many figures they appear in:

| Figures | Option |
|---:|---|
| 32 | `circuit ee IEC` |
| 26 | `postaction` |
| 20 | `resistor` |
| 20 | `top color` |
| 18 | `bottom color` |
| 17 | `level distance` |
| 17 | `concept` |
| 16 | `left color` |
| 16 | `mindmap` |
| 15 | `shape border rotate` |
| 14 | `right color` |
| 14 | `concept color` |
| 13 | `auto` |
| 13 | `on chain` |
| 13 | `place` |
| 12 | `3d view` |
| 12 | `perspective` |
| 12 | `name intersections` |
| 11 | `rounded rectangle` |
| 10 | `loop above` |
| 10 | `text effects` |
| 10 | `shape border uses incircle` |
| 9 | `shading` |
| 9 | `state` |
| 9 | `start chain` |
| 9 | `sibling distance` |
| 9 | `path fading` |
| 8 | `loop below` |
| 8 | `fit` |
| 8 | `grow cyclic` |
| 8 | `grow` |
| 7 | `transparency group` |
| 7 | `initial` |
| 7 | `diode` |
| 7 | `entity` |
| 7 | `spy using outlines` |
| 6 | `make snapshot of` |
| 6 | `animate` |
| 6 | `logic gate inputs` |
| 6 | `ball color` |
| 6 | `tape` |
| 6 | `rectangle split parts` |
| 6 | `rectangle split` |
| 6 | `color of colormap` |
| 5 | `swap` |
| 5 | `on background layer` |
| 5 | `preaction` |
| 5 | `circular drop shadow` |
| 5 | `cylinder` |
| 5 | `forbidden sign` |
| 5 | `cloud puffs` |
| 5 | `circle through` |
| 5 | `colormap` |
| 4 | `framed` |
| 4 | `show background top` |
| 4 | `circuit logic IEC` |
| 4 | `tokens` |
| 4 | `children are tokens` |
| 4 | `drop shadow` |
| 4 | `circular glow` |

Commands, environments and path operations tikra does not know, likewise:

| Figures | Command |
|---:|---|
| 219 | `\datavisualization` |
| 170 | `\graph` |
| 61 | `child` |
| 51 | `\matrix` |
| 48 | `\addplot` |
| 41 | `\calendar` |
| 32 | `decorate` |
| 25 | `\pgfpathmoveto` |
| 25 | `\pic` |
| 19 | `\spy` |
| 16 | `\nextgroupplot` |
| 14 | `pic` |
| 14 | `polaraxis` |
| 13 | `\pgfpathcircle` |
| 13 | `\pgftext` |
| 13 | `ternaryaxis` |
| 12 | `smithchart` |
| 8 | `\pgfpoint` |
| 8 | `groupplot` |
| 8 | `\addlegendentry` |
| 7 | `\pgfsetlinewidth` |
| 7 | `\chainin` |
| 6 | `\pgftransformrotate` |
| 6 | `\pgftransformnonlinear` |
| 5 | `\pgfpathlineto` |
| 5 | `\pgfsetcurvilinearbeziercurve` |
| 4 | `pgfdecoration` |
| 4 | `\pgfmatrix` |
| 4 | `\pgftransformshift` |
| 4 | `\pgftransformxshift` |
| 4 | `\pgftransformscale` |
| 4 | `\legend` |
| 3 | `\pgfdecoratedremainingdistance` |
| 3 | `\pgfdecoratedpathlength` |
| 3 | `\pgfsetmatrixcolumnsep` |
| 3 | `\pgfnode` |
| 3 | `\pgfpathrectangle` |
| 3 | `\pgfplothandlerlineto` |
| 3 | `\pgfslopedattimetrue` |
| 3 | `\pgfplothandlermark` |

Errors of the figures that fail:

| Figures | Error |
|---:|---|
| 4 | `figure: missing number` |
| 1 | `figure: missing dimension` |
| 1 | `figure: you can't use \pgf@xc after \advance` |
| 1 | `png: image too large: N × N px at N dpi (…)` |
