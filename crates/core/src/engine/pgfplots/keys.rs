//! The keys of `/pgfplots`, defined the way `pgfplots.code.tex` defines them: styles are copied
//! from it, values are `.initial` keys that the axis reads when it is drawn, and the keys whose
//! code does work (`compat`, `axis x line`, `/pgf/number format/…`, the plot handlers) are native.
//! The defaults are those of `compat=default`, as the pgfplots manual compiles.

use crate::keys;
use crate::tex::{self, Meaning, Token};
use crate::Error;

use super::super::{options::KeyFn, tikra_tokens, Engine, NATIVE_KEY_BASE};
use super::number::{SciStyle, Style};

/// pgfplots' own definitions (`\pgfkeys{…}` blocks of `pgfplots.code.tex`, trimmed to the keys
/// that tikra draws, plus the values the native code reads).
const DEFINITIONS: &str = r"
\def\pgfplotscreateplotcyclelist#1#2{\expandafter\def\csname pgfk@/pgfplots/@cycle lists/#1\endcsname{#2}}
\let\pgfcreateplotcyclelist\pgfplotscreateplotcyclelist
\def\pgfkeyslet#1#2{\expandafter\let\csname pgfk@#1\endcsname#2}
\def\axisdefaultwidth{240pt}\def\axisdefaultheight{207pt}
\def\pgfplotsmarklistfill{.!80!black}
\def\numplots{0}\def\numplotsofactualtype{1}\def\plotnumofactualtype{0}\def\plotnum{0}
\def\pgfplotbarwidth{10pt}\def\pgfplotbarshift{0pt}
\def\pgfplotscolormaplastindexof#1{\csname pgfpl@cm@#1@last\endcsname}
\pgfplotscreateplotcyclelist{color}{%
	blue,every mark/.append style={fill=blue!80!black},mark=*\\%
	red,every mark/.append style={fill=red!80!black},mark=square*\\%
	brown!60!black,every mark/.append style={fill=brown!80!black},mark=otimes*\\%
	black,mark=star\\%
	blue,every mark/.append style={fill=blue!80!black},mark=diamond*\\%
	red,densely dashed,every mark/.append style={solid,fill=red!80!black},mark=*\\%
	brown!60!black,densely dashed,every mark/.append style={solid,fill=brown!80!black},mark=square*\\%
	black,densely dashed,every mark/.append style={solid,fill=gray},mark=otimes*\\%
	blue,densely dashed,mark=star,every mark/.append style=solid\\%
	red,densely dashed,every mark/.append style={solid,fill=red!80!black},mark=diamond*\\%
}
\pgfplotscreateplotcyclelist{black white}{%
	every mark/.append style={fill=gray},mark=*\\%
	every mark/.append style={fill=gray},mark=square*\\%
	every mark/.append style={fill=gray},mark=otimes*\\%
	mark=star\\%
	every mark/.append style={fill=gray},mark=diamond*\\%
	densely dashed,every mark/.append style={solid,fill=gray},mark=*\\%
	densely dashed,every mark/.append style={solid,fill=gray},mark=square*\\%
	densely dashed,every mark/.append style={solid,fill=gray},mark=otimes*\\%
	densely dashed,every mark/.append style={solid},mark=star\\%
	densely dashed,every mark/.append style={solid,fill=gray},mark=diamond*\\%
}
\pgfplotscreateplotcyclelist{linestyles}{solid,dashed,dotted}
\pgfplotscreateplotcyclelist{linestyles*}{solid,dashed,dotted,dashdotted,dashdotdotted}
\pgfplotscreateplotcyclelist{color list}{red,blue,black,yellow,brown,teal,orange,violet,cyan,green!70!black,magenta,gray}
\pgfplotscreateplotcyclelist{mark list}{%
	every mark/.append style={solid,fill=\pgfplotsmarklistfill},mark=*\\%
	every mark/.append style={solid,fill=\pgfplotsmarklistfill},mark=square*\\%
	every mark/.append style={solid,fill=\pgfplotsmarklistfill},mark=triangle*\\%
	every mark/.append style={solid},mark=star\\%
	every mark/.append style={solid,fill=\pgfplotsmarklistfill},mark=diamond*\\%
	every mark/.append style={solid,fill=\pgfplotsmarklistfill!40},mark=otimes*\\%
	every mark/.append style={solid},mark=|\\%
	every mark/.append style={solid,fill=\pgfplotsmarklistfill},mark=pentagon*\\%
	every mark/.append style={solid},mark=text,text mark=p\\%
	every mark/.append style={solid},mark=text,text mark=a\\%
}
\pgfplotscreateplotcyclelist{mark list*}{%
	every mark/.append style={solid,fill=\pgfplotsmarklistfill},mark=*\\%
	every mark/.append style={solid,fill=\pgfplotsmarklistfill},mark=square*\\%
	every mark/.append style={solid,fill=\pgfplotsmarklistfill},mark=triangle*\\%
	every mark/.append style={solid,fill=\pgfplotsmarklistfill},mark=halfsquare*\\%
	every mark/.append style={solid,fill=\pgfplotsmarklistfill},mark=pentagon*\\%
	every mark/.append style={solid,fill=\pgfplotsmarklistfill},mark=halfcircle*\\%
	every mark/.append style={solid,fill=\pgfplotsmarklistfill,rotate=180},mark=halfdiamond*\\%
	every mark/.append style={solid,fill=\pgfplotsmarklistfill!40},mark=otimes*\\%
	every mark/.append style={solid,fill=\pgfplotsmarklistfill},mark=diamond*\\%
	every mark/.append style={solid,fill=\pgfplotsmarklistfill},mark=halfsquare right*\\%
	every mark/.append style={solid,fill=\pgfplotsmarklistfill},mark=halfsquare left*\\%
}
\pgfplotscreateplotcyclelist{exotic}{%
	teal,every mark/.append style={fill=teal!80!black},mark=*\\%
	orange,every mark/.append style={fill=orange!80!black},mark=square*\\%
	cyan!60!black,every mark/.append style={fill=cyan!80!black},mark=otimes*\\%
	red!70!white,mark=star\\%
	lime!80!black,every mark/.append style={fill=lime},mark=diamond*\\%
	red,densely dashed,every mark/.append style={solid,fill=red!80!black},mark=*\\%
	yellow!60!black,densely dashed,every mark/.append style={solid,fill=yellow!80!black},mark=square*\\%
	black,every mark/.append style={solid,fill=gray},mark=otimes*\\%
	blue,densely dashed,mark=star,every mark/.append style=solid\\%
	red,densely dashed,every mark/.append style={solid,fill=red!80!black},mark=diamond*\\%
}
\pgfkeys{
	/tikz/dashdotted/.style={dash pattern=on 3pt off 2pt on \the\pgflinewidth off 2pt},
	/tikz/densely dashdotted/.style={dash pattern=on 3pt off 1pt on \the\pgflinewidth off 1pt},
	/tikz/loosely dashdotted/.style={dash pattern=on 3pt off 4pt on \the\pgflinewidth off 4pt},
	/tikz/dashdotdotted/.style={dash pattern=on 3pt off 2pt on \the\pgflinewidth off 2pt on \the\pgflinewidth off 2pt},
	/tikz/densely dashdotdotted/.style={dash pattern=on 3pt off 1pt on \the\pgflinewidth off 1pt on \the\pgflinewidth off 1pt},
	/tikz/loosely dashdotdotted/.style={dash pattern=on 3pt off 4pt on \the\pgflinewidth off 4pt on \the\pgflinewidth off 4pt},
	/tikz/every mark/.style={},
	/tikz/mark options/.style={/tikz/every mark/.style={#1}},
	/tikz/mark size/.initial=2pt,
	/tikz/mark repeat/.initial=1, /tikz/mark phase/.initial=1, /tikz/mark indices/.initial=,
	/tikz/text mark/.initial=p, /tikz/text mark style/.style={}, /tikz/text mark as node/.initial=false,
	/tikz/nodes/.style={/tikz/every node/.append style={#1}},
	/tikz/every cell/.style={}, /tikz/cells/.style={/tikz/every cell/.append style={#1}},
	/tikz/column sep/.initial=0pt, /tikz/row sep/.initial=0pt,
	/tikz/@plot handler/.initial=,
	/pgf/bar width/.initial=10pt, /pgf/bar shift/.initial=0pt,
	/pgf/bar interval width/.initial=1, /pgf/bar interval shift/.initial=0.5,
	/tikz/bar width/.forward to=/pgf/bar width, /tikz/bar shift/.forward to=/pgf/bar shift,
	/tikz/bar interval width/.forward to=/pgf/bar interval width,
	/tikz/bar interval shift/.forward to=/pgf/bar interval shift,
	/pgf/number format/.cd,
	/pgf/number format/fixed relative/.code={\pgfkeysalso{/pgf/number format/@fixed relative}},
	/pgfplots/.cd,
	/pgfplots/.search also={/tikz},
	/pgfplots/every axis/.style={}, /pgfplots/every axis post/.style={},
	/pgfplots/every semilogx axis/.style={}, /pgfplots/every semilogy axis/.style={},
	/pgfplots/every loglog axis/.style={}, /pgfplots/every linear axis/.style={},
	/pgfplots/every axis plot/.style={}, /pgfplots/every axis plot post/.style={},
	/pgfplots/every axis plot except legend/.style={},
	/pgfplots/axis plot except legend style/.style={/pgfplots/every axis plot except legend/.append style={#1}},
	/pgfplots/every forget plot/.style={},
	/pgfplots/forget plot style/.style={/pgfplots/every forget plot/.append style={#1}},
	/pgfplots/forget plot/.initial=false, /pgfplots/forget plot/.default=true,
	/pgfplots/no marks/.style={/pgfplots/no markers},
	/pgfplots/no markers/.style={/pgfplots/every axis plot post/.append style={mark=none}},
	/pgfplots/every axis label/.style={},
	/pgfplots/xlabel near ticks/.style={%
		/pgfplots/every axis x label/.style={at={(ticklabel cs:0.5)},anchor=near ticklabel},%
		/pgfplots/every x tick scale label/.style={at={(xticklabel cs:0.9,5pt)},anchor=near xticklabel,inner sep=0pt},
	},%
	/pgfplots/xlabel absolute/.style={%
		/pgfplots/every axis x label/.style={at={(0.5,0)},below,yshift=-15pt},%
		/pgfplots/every x tick scale label/.style={at={(1,0)},yshift=-2em,left,inner sep=0pt},
	},%
	/pgfplots/ylabel near ticks/.style={%
		/pgfplots/every axis y label/.style={at={(ticklabel cs:0.5)},rotate=90,anchor=near ticklabel},%
		/pgfplots/every y tick scale label/.style={at={(yticklabel* cs:1.03,-0.3em)},/pgfplots/near ticklabel align=outside,anchor=near yticklabel opposite,inner sep=0pt},
	},%
	/pgfplots/ylabel absolute/.style={%
		/pgfplots/every axis y label/.style={at={(0,0.5)},xshift=-35pt,rotate=90},
		/pgfplots/every y tick scale label/.style={at={(0,1)},above right,inner sep=0pt,yshift=0.3em},
	},%
	/pgfplots/every axis z label/.style={at={(ticklabel cs:0.5)},rotate=90, anchor=near ticklabel},
	/pgfplots/every z tick scale label/.style={at={(zticklabel* cs:1.2,-0.3em)},anchor=near zticklabel,inner sep=0pt},
	/pgfplots/near ticklabel align/.initial=center,
	/pgfplots/every axis title shift/.initial=6pt,
	/pgfplots/every axis title/.style={at={(0.5,1)},above,yshift=\pgfkeysvalueof{/pgfplots/every axis title shift}},
	/pgfplots/every tick/.style={very thin,gray},
	/pgfplots/every inner x axis line/.style={}, /pgfplots/every inner y axis line/.style={},
	/pgfplots/every inner z axis line/.style={}, /pgfplots/every outer x axis line/.style={},
	/pgfplots/every outer y axis line/.style={}, /pgfplots/every outer z axis line/.style={},
	/pgfplots/x axis line style/.style={/pgfplots/every outer x axis line/.append style={#1},/pgfplots/every inner x axis line/.append style={#1}},
	/pgfplots/y axis line style/.style={/pgfplots/every outer y axis line/.append style={#1},/pgfplots/every inner y axis line/.append style={#1}},
	/pgfplots/z axis line style/.style={/pgfplots/every outer z axis line/.append style={#1},/pgfplots/every inner z axis line/.append style={#1}},
	/pgfplots/outer axis line style/.style={/pgfplots/every outer x axis line/.append style={#1},/pgfplots/every outer y axis line/.append style={#1},/pgfplots/every outer z axis line/.append style={#1}},
	/pgfplots/inner axis line style/.style={/pgfplots/every inner x axis line/.append style={#1},/pgfplots/every inner y axis line/.append style={#1},/pgfplots/every inner z axis line/.append style={#1}},
	/pgfplots/axis line style/.style={/pgfplots/inner axis line style={#1},/pgfplots/outer axis line style={#1}},
	/pgfplots/separate axis lines/.initial=false, /pgfplots/separate axis lines/.default=true,
	/pgfplots/every minor tick/.style={}, /pgfplots/every major tick/.style={},
	/pgfplots/every x tick/.style={}, /pgfplots/every minor x tick/.style={}, /pgfplots/every major x tick/.style={},
	/pgfplots/every y tick/.style={}, /pgfplots/every minor y tick/.style={}, /pgfplots/every major y tick/.style={},
	/pgfplots/every z tick/.style={}, /pgfplots/every minor z tick/.style={}, /pgfplots/every major z tick/.style={},
	/pgfplots/every axis grid/.style={thin,black!25},
	/pgfplots/every minor grid/.style={}, /pgfplots/every major grid/.style={},
	/pgfplots/every axis x grid/.style={}, /pgfplots/every minor x grid/.style={}, /pgfplots/every major x grid/.style={},
	/pgfplots/every axis y grid/.style={}, /pgfplots/every minor y grid/.style={}, /pgfplots/every major y grid/.style={},
	/pgfplots/every axis z grid/.style={}, /pgfplots/every minor z grid/.style={}, /pgfplots/every major z grid/.style={},
	/pgfplots/every tick label/.style={}, /pgfplots/every x tick label/.style={},
	/pgfplots/every y tick label/.style={}, /pgfplots/every z tick label/.style={},
	/pgfplots/log identify minor tick positions/.initial=false, /pgfplots/log identify minor tick positions/.default=true,
	/pgfplots/every extra x tick/.style={/pgfplots/log identify minor tick positions=true},
	/pgfplots/every extra y tick/.style={/pgfplots/log identify minor tick positions=true},
	/pgfplots/every extra z tick/.style={/pgfplots/log identify minor tick positions=true},
	/pgfplots/xtickten/.initial=, /pgfplots/ytickten/.initial=, /pgfplots/ztickten/.initial=,
	/pgfplots/extra x tick style/.code={\pgfkeysalso{/pgfplots/every extra x tick/.append style={#1}}},
	/pgfplots/extra y tick style/.code={\pgfkeysalso{/pgfplots/every extra y tick/.append style={#1}}},
	/pgfplots/extra tick style/.code={\pgfkeysalso{/pgfplots/every extra x tick/.append style={#1},/pgfplots/every extra y tick/.append style={#1},/pgfplots/every extra z tick/.append style={#1}}},
	/pgfplots/every axis legend/.style={%
		cells={anchor=center},
		inner xsep=3pt,inner ysep=2pt,nodes={inner sep=2pt,text depth=0.15em},
		anchor=north east,%
		shape=rectangle,%
		fill=white,%
		draw=black,
		at={(0.98,0.98)},
	},
	/pgfplots/legend style/.code={\pgfkeysalso{/pgfplots/every axis legend/.append style={#1}}},
	/pgfplots/legend entries/.initial=,
	/pgfplots/legend to name/.initial=,
	/pgfplots/legend pos/.is choice,
	/pgfplots/legend pos/south west/.style={/pgfplots/legend style={at={(0.03,0.03)},anchor=south west}},
	/pgfplots/legend pos/south east/.style={/pgfplots/legend style={at={(0.97,0.03)},anchor=south east}},
	/pgfplots/legend pos/north west/.style={/pgfplots/legend style={at={(0.03,0.97)},anchor=north west}},
	/pgfplots/legend pos/north east/.style={/pgfplots/legend style={at={(0.97,0.97)},anchor=north east}},
	/pgfplots/legend pos/outer north east/.style={/pgfplots/legend style={at={(1.03,1)},anchor=north west}},
	/pgfplots/legend cell align/.is choice,
	/pgfplots/legend cell align/left/.style={/pgfplots/legend style={cells={anchor=west}}},
	/pgfplots/legend cell align/right/.style={/pgfplots/legend style={cells={anchor=east}}},
	/pgfplots/legend cell align/center/.style={/pgfplots/legend style={cells={anchor=center}}},
	/pgfplots/legend columns/.initial=1,
	/pgfplots/legend plot pos/.initial=left,
	/pgfplots/reverse legend/.initial=false, /pgfplots/reverse legend/.default=true,
	/pgfplots/transpose legend/.initial=false, /pgfplots/transpose legend/.default=true,
	/pgfplots/legend reversed/.style={/pgfplots/reverse legend=#1}, /pgfplots/legend reversed/.default=true,
	/pgfplots/legend transposed/.style={/pgfplots/transpose legend=#1}, /pgfplots/legend transposed/.default=true,
	/pgfplots/every legend image post/.style={},
	/pgfplots/legend image post style/.style={/pgfplots/every legend image post/.append style={#1}},
	/pgfplots/@legend image/.initial=line,
	/pgfplots/line legend/.code={\pgfkeyssetvalue{/pgfplots/@legend image}{line}},
	/pgfplots/area legend/.code={\pgfkeyssetvalue{/pgfplots/@legend image}{area}},
	/pgfplots/every fill between plot/.style={/pgfplots/area legend,/tikz/fill},
	/pgfplots/empty legend/.code={\pgfkeyssetvalue{/pgfplots/@legend image}{empty}},
	/pgfplots/ybar legend/.code={\pgfkeyssetvalue{/pgfplots/@legend image}{ybar}},
	/pgfplots/xbar legend/.code={\pgfkeyssetvalue{/pgfplots/@legend image}{xbar}},
	/pgfplots/single ybar legend/.code={\pgfkeyssetvalue{/pgfplots/@legend image}{single ybar}},
	/pgfplots/single xbar legend/.code={\pgfkeyssetvalue{/pgfplots/@legend image}{single xbar}},
	/pgfplots/ybar interval legend/.code={\pgfkeyssetvalue{/pgfplots/@legend image}{ybar interval}},
	/pgfplots/xbar interval legend/.code={\pgfkeyssetvalue{/pgfplots/@legend image}{xbar interval}},
	/pgfplots/legend image code/.code={\pgfkeyssetvalue{/pgfplots/@legend image}{line}},
	/pgfplots/legend image with text/.code={\pgfkeyssetvalue{/pgfplots/@legend image}{empty}},
	/pgfplots/title/.initial=, /pgfplots/xlabel/.initial=, /pgfplots/ylabel/.initial=, /pgfplots/zlabel/.initial=,
	/pgfplots/use units/.initial=true, /pgfplots/use units/.default=true,
	/pgfplots/x unit/.initial=, /pgfplots/y unit/.initial=, /pgfplots/z unit/.initial=,
	/pgfplots/x unit prefix/.initial=, /pgfplots/y unit prefix/.initial=, /pgfplots/z unit prefix/.initial=,
	/pgfplots/change x base/.initial=false, /pgfplots/change x base/.default=true,
	/pgfplots/change y base/.initial=false, /pgfplots/change y base/.default=true,
	/pgfplots/change z base/.initial=false, /pgfplots/change z base/.default=true,
	/pgfplots/@x scale label off/.initial=false, /pgfplots/@y scale label off/.initial=false, /pgfplots/@z scale label off/.initial=false,
	/pgfplots/xlabel shift/.initial=0pt, /pgfplots/ylabel shift/.initial=0pt, /pgfplots/zlabel shift/.initial=0pt,
	/pgfplots/label shift/.style={/pgfplots/xlabel shift={#1},/pgfplots/ylabel shift={#1},/pgfplots/zlabel shift={#1}},
	/pgfplots/extra description/.code=,
	/pgfplots/label style/.style={/pgfplots/every axis x label/.append style={#1},/pgfplots/every axis y label/.append style={#1},/pgfplots/every axis z label/.append style={#1}},
	/pgfplots/x label style/.code={\pgfkeysalso{/pgfplots/every axis x label/.append style={#1}}},
	/pgfplots/y label style/.code={\pgfkeysalso{/pgfplots/every axis y label/.append style={#1}}},
	/pgfplots/z label style/.code={\pgfkeysalso{/pgfplots/every axis z label/.append style={#1}}},
	/pgfplots/xlabel style/.style={/pgfplots/x label style={#1}},
	/pgfplots/ylabel style/.style={/pgfplots/y label style={#1}},
	/pgfplots/zlabel style/.style={/pgfplots/z label style={#1}},
	/pgfplots/title style/.code={\pgfkeysalso{/pgfplots/every axis title/.append style={#1}}},
	/pgfplots/tick label style/.code={\pgfkeysalso{/pgfplots/every tick label/.append style={#1}}},
	/pgfplots/ticklabel style/.style={/pgfplots/tick label style={#1}},
	/pgfplots/x tick label style/.code={\pgfkeysalso{/pgfplots/every x tick label/.append style={#1}}},
	/pgfplots/y tick label style/.code={\pgfkeysalso{/pgfplots/every y tick label/.append style={#1}}},
	/pgfplots/z tick label style/.code={\pgfkeysalso{/pgfplots/every z tick label/.append style={#1}}},
	/pgfplots/xticklabel style/.style={/pgfplots/x tick label style={#1}},
	/pgfplots/yticklabel style/.style={/pgfplots/y tick label style={#1}},
	/pgfplots/zticklabel style/.style={/pgfplots/z tick label style={#1}},
	/pgfplots/x tick scale label style/.code={\pgfkeysalso{/pgfplots/every x tick scale label/.append style={#1}}},
	/pgfplots/y tick scale label style/.code={\pgfkeysalso{/pgfplots/every y tick scale label/.append style={#1}}},
	/pgfplots/tick style/.code={\pgfkeysalso{/pgfplots/every tick/.append style={#1}}},
	/pgfplots/minor tick style/.code={\pgfkeysalso{/pgfplots/every minor tick/.append style={#1}}},
	/pgfplots/major tick style/.code={\pgfkeysalso{/pgfplots/every major tick/.append style={#1}}},
	/pgfplots/x tick style/.code={\pgfkeysalso{/pgfplots/every x tick/.append style={#1}}},
	/pgfplots/y tick style/.code={\pgfkeysalso{/pgfplots/every y tick/.append style={#1}}},
	/pgfplots/minor x tick style/.code={\pgfkeysalso{/pgfplots/every minor x tick/.append style={#1}}},
	/pgfplots/minor y tick style/.code={\pgfkeysalso{/pgfplots/every minor y tick/.append style={#1}}},
	/pgfplots/major x tick style/.code={\pgfkeysalso{/pgfplots/every major x tick/.append style={#1}}},
	/pgfplots/major y tick style/.code={\pgfkeysalso{/pgfplots/every major y tick/.append style={#1}}},
	/pgfplots/grid style/.code={\pgfkeysalso{/pgfplots/every axis grid/.append style={#1}}},
	/pgfplots/minor grid style/.code={\pgfkeysalso{/pgfplots/every minor grid/.append style={#1}}},
	/pgfplots/major grid style/.code={\pgfkeysalso{/pgfplots/every major grid/.append style={#1}}},
	/pgfplots/x grid style/.code={\pgfkeysalso{/pgfplots/every axis x grid/.append style={#1}}},
	/pgfplots/y grid style/.code={\pgfkeysalso{/pgfplots/every axis y grid/.append style={#1}}},
	/pgfplots/minor x grid style/.code={\pgfkeysalso{/pgfplots/every minor x grid/.append style={#1}}},
	/pgfplots/minor y grid style/.code={\pgfkeysalso{/pgfplots/every minor y grid/.append style={#1}}},
	/pgfplots/major x grid style/.code={\pgfkeysalso{/pgfplots/every major x grid/.append style={#1}}},
	/pgfplots/major y grid style/.code={\pgfkeysalso{/pgfplots/every major y grid/.append style={#1}}},
	/pgfplots/every error bar/.style={thin},
	/pgfplots/error bars/error bar style/.code={\pgfkeysalso{/pgfplots/every error bar/.append style={#1}}},
	/pgfplots/error bars/error mark/.initial=-,
	/pgfplots/error bars/error mark options/.initial={rotate=90},
	/pgfplots/error bars/x dir/.initial=none, /pgfplots/error bars/y dir/.initial=none,
	/pgfplots/error bars/x explicit/.code={\pgfkeyssetvalue{/pgfplots/error bars/@x mode}{explicit}},
	/pgfplots/error bars/y explicit/.code={\pgfkeyssetvalue{/pgfplots/error bars/@y mode}{explicit}},
	/pgfplots/error bars/x explicit relative/.code={\pgfkeyssetvalue{/pgfplots/error bars/@x mode}{explicit relative}},
	/pgfplots/error bars/y explicit relative/.code={\pgfkeyssetvalue{/pgfplots/error bars/@y mode}{explicit relative}},
	/pgfplots/error bars/x fixed/.code={\pgfkeyssetvalue{/pgfplots/error bars/@x mode}{fixed}\pgfkeyssetvalue{/pgfplots/error bars/@x value}{#1}},
	/pgfplots/error bars/y fixed/.code={\pgfkeyssetvalue{/pgfplots/error bars/@y mode}{fixed}\pgfkeyssetvalue{/pgfplots/error bars/@y value}{#1}},
	/pgfplots/error bars/x fixed relative/.code={\pgfkeyssetvalue{/pgfplots/error bars/@x mode}{fixed relative}\pgfkeyssetvalue{/pgfplots/error bars/@x value}{#1}},
	/pgfplots/error bars/y fixed relative/.code={\pgfkeyssetvalue{/pgfplots/error bars/@y mode}{fixed relative}\pgfkeyssetvalue{/pgfplots/error bars/@y value}{#1}},
	/pgfplots/error bars/@x mode/.initial=fixed, /pgfplots/error bars/@y mode/.initial=fixed,
	/pgfplots/error bars/@x value/.initial=0, /pgfplots/error bars/@y value/.initial=0,
	/pgfplots/error bars/.cd,
	/pgfplots/.cd,
	/pgfplots/xticklabel/.initial=, /pgfplots/yticklabel/.initial=, /pgfplots/zticklabel/.initial=,
	/pgfplots/xticklabels/.initial=, /pgfplots/yticklabels/.initial=, /pgfplots/zticklabels/.initial=,
	/pgfplots/x tick label as interval/.initial=false, /pgfplots/x tick label as interval/.default=true,
	/pgfplots/y tick label as interval/.initial=false, /pgfplots/y tick label as interval/.default=true,
	/pgfplots/extra x tick label/.initial=, /pgfplots/extra y tick label/.initial=,
	/pgfplots/extra x tick labels/.initial=, /pgfplots/extra y tick labels/.initial=,
	/pgfplots/xtick/.initial=, /pgfplots/ytick/.initial=, /pgfplots/ztick/.initial=,
	/pgfplots/extra x ticks/.initial=, /pgfplots/extra y ticks/.initial=, /pgfplots/extra z ticks/.initial=,
	/pgfplots/tick scale binop/.initial=\cdot,
	/pgfplots/scaled x ticks/.initial=true, /pgfplots/scaled y ticks/.initial=true, /pgfplots/scaled z ticks/.initial=true,
	/pgfplots/scaled ticks/.style={/pgfplots/scaled x ticks=#1,/pgfplots/scaled y ticks=#1,/pgfplots/scaled z ticks=#1},
	/pgfplots/scale ticks above exponent/.initial=3, /pgfplots/scale ticks below exponent/.initial=-1,
	/pgfplots/minor x tick num/.initial=0, /pgfplots/minor y tick num/.initial=0, /pgfplots/minor z tick num/.initial=0,
	/pgfplots/minor tick num/.style={/pgfplots/minor x tick num=#1,/pgfplots/minor y tick num=#1,/pgfplots/minor z tick num=#1},
	/pgfplots/minor xtick/.initial=, /pgfplots/minor ytick/.initial=, /pgfplots/minor ztick/.initial=,
	/pgfplots/minor tick/.style={/pgfplots/minor xtick={#1},/pgfplots/minor ytick={#1},/pgfplots/minor ztick={#1}},
	/pgfplots/minor tick length/.initial=0.1cm, /pgfplots/major tick length/.initial=0.15cm,
	/pgfplots/subtickwidth/.style={/pgfplots/minor tick length={#1}}, /pgfplots/tickwidth/.style={/pgfplots/major tick length={#1}},
	/pgfplots/xtick distance/.initial=, /pgfplots/ytick distance/.initial=, /pgfplots/ztick distance/.initial=,
	/pgfplots/max space between ticks/.initial=35, /pgfplots/try min ticks/.initial=4, /pgfplots/try min ticks log/.initial=3,
	/pgfplots/log basis x/.initial=, /pgfplots/log basis y/.initial=, /pgfplots/log basis z/.initial=,
	/pgfplots/log base x/.style={/pgfplots/log basis x={#1}}, /pgfplots/log base y/.style={/pgfplots/log basis y={#1}},
	/pgfplots/log ticks with fixed point/.code={\pgfkeyssetvalue{/pgfplots/@log fixed point}{true}},
	/pgfplots/@log fixed point/.initial=false,
	/pgfplots/xtick pos/.initial=both, /pgfplots/ytick pos/.initial=both, /pgfplots/ztick pos/.initial=both,
	/pgfplots/tickpos/.style={/pgfplots/xtick pos={#1},/pgfplots/ytick pos={#1},/pgfplots/ztick pos={#1}},
	/pgfplots/tick pos/.style={/pgfplots/tickpos={#1}},
	/pgfplots/xticklabel pos/.initial=default, /pgfplots/yticklabel pos/.initial=default, /pgfplots/zticklabel pos/.initial=default,
	/pgfplots/ticklabel pos/.style={/pgfplots/xticklabel pos={#1},/pgfplots/yticklabel pos={#1},/pgfplots/zticklabel pos={#1}},
	/pgfplots/ticklabelpos/.style={/pgfplots/ticklabel pos={#1}},
	/pgfplots/xticklabel shift/.initial=, /pgfplots/yticklabel shift/.initial=,
	/pgfplots/ticklabel shift/.style={/pgfplots/xticklabel shift={#1},/pgfplots/yticklabel shift={#1},/pgfplots/zticklabel shift={#1}},
	/pgfplots/xtick align/.initial=inside, /pgfplots/ytick align/.initial=inside, /pgfplots/ztick align/.initial=inside,
	/pgfplots/tick align/.style={/pgfplots/xtick align=#1,/pgfplots/ytick align=#1,/pgfplots/ztick align=#1},
	/pgfplots/anchor/.initial=south west,
	/pgfplots/at/.initial=,
	/pgfplots/name/.initial=,
	/pgfplots/alias/.code=,
	/pgfplots/ticks/.is choice,
	/pgfplots/ticks/none/.style={xmajorticks=false,ymajorticks=false,zmajorticks=false,xminorticks=false,yminorticks=false,zminorticks=false},
	/pgfplots/ticks/major/.style={xmajorticks=true,ymajorticks=true,zmajorticks=true,xminorticks=false,yminorticks=false,zminorticks=false},
	/pgfplots/ticks/minor/.style={xmajorticks=false,ymajorticks=false,zmajorticks=false,xminorticks=true,yminorticks=true,zminorticks=true},
	/pgfplots/ticks/both/.style={xmajorticks=true,ymajorticks=true,zmajorticks=true,xminorticks=true,yminorticks=true,zminorticks=true},
	/pgfplots/grid/.is choice, /pgfplots/grid/.default=major,
	/pgfplots/grid/none/.style={xmajorgrids=false,ymajorgrids=false,zmajorgrids=false,xminorgrids=false,yminorgrids=false,zminorgrids=false},
	/pgfplots/grid/major/.style={xmajorgrids=true,ymajorgrids=true,zmajorgrids=true,xminorgrids=false,yminorgrids=false,zminorgrids=false},
	/pgfplots/grid/minor/.style={xmajorgrids=false,ymajorgrids=false,zmajorgrids=false,xminorgrids=true,yminorgrids=true,zminorgrids=true},
	/pgfplots/grid/both/.style={xmajorgrids=true,ymajorgrids=true,zmajorgrids=true,xminorgrids=true,yminorgrids=true,zminorgrids=true},
	/pgfplots/xminorticks/.initial=true, /pgfplots/xmajorticks/.initial=true,
	/pgfplots/yminorticks/.initial=true, /pgfplots/ymajorticks/.initial=true,
	/pgfplots/zminorticks/.initial=true, /pgfplots/zmajorticks/.initial=true,
	/pgfplots/xminorgrids/.initial=false, /pgfplots/xmajorgrids/.initial=false,
	/pgfplots/yminorgrids/.initial=false, /pgfplots/ymajorgrids/.initial=false,
	/pgfplots/zminorgrids/.initial=false, /pgfplots/zmajorgrids/.initial=false,
	/pgfplots/xminorticks/.default=true, /pgfplots/xmajorticks/.default=true,
	/pgfplots/yminorticks/.default=true, /pgfplots/ymajorticks/.default=true,
	/pgfplots/zminorticks/.default=true, /pgfplots/zmajorticks/.default=true,
	/pgfplots/xminorgrids/.default=true, /pgfplots/xmajorgrids/.default=true,
	/pgfplots/yminorgrids/.default=true, /pgfplots/ymajorgrids/.default=true,
	/pgfplots/zminorgrids/.default=true, /pgfplots/zmajorgrids/.default=true,
	/pgfplots/hide x axis/.initial=false, /pgfplots/hide x axis/.default=true,
	/pgfplots/hide y axis/.initial=false, /pgfplots/hide y axis/.default=true,
	/pgfplots/hide z axis/.initial=false, /pgfplots/hide z axis/.default=true,
	/pgfplots/hide axis/.style={/pgfplots/hide x axis=#1,/pgfplots/hide y axis=#1,/pgfplots/hide z axis=#1},
	/pgfplots/hide axis/.default=true,
	/pgfplots/axis on top/.initial=false, /pgfplots/axis on top/.default=true,
	/pgfplots/clip/.initial=true, /pgfplots/clip/.default=true,
	/pgfplots/clip mode/.initial=global,
	/pgfplots/clip marker paths/.initial=false, /pgfplots/clip marker paths/.default=true,
	/pgfplots/axis background/.style={},
	/pgfplots/axis background style/.style={/pgfplots/axis background/.append style={#1}},
	/pgfplots/every non boxed x axis/.style={xtick align=center,enlarge x limits=false,x axis line style={-stealth}},
	/pgfplots/every non boxed y axis/.style={ytick align=center,enlarge y limits=false,y axis line style={-stealth}},
	/pgfplots/every non boxed z axis/.style={ztick align=center,enlarge z limits=false,z axis line style={-stealth}},
	/pgfplots/every boxed x axis/.style={}, /pgfplots/every boxed y axis/.style={}, /pgfplots/every boxed z axis/.style={},
	/pgfplots/axis x line shift/.initial=, /pgfplots/axis y line shift/.initial=, /pgfplots/axis z line shift/.initial=,
	/pgfplots/axis line shift/.style={/pgfplots/axis x line shift={#1},/pgfplots/axis y line shift={#1},/pgfplots/axis z line shift={#1}},
	/pgfplots/@x line/.initial=box, /pgfplots/@y line/.initial=box, /pgfplots/@z line/.initial=box,
	/pgfplots/axis lines/.style={axis x line=#1,axis y line=#1,axis z line=#1},
	/pgfplots/axis lines*/.style={axis x line*=#1,axis y line*=#1,axis z line*=#1},
	/pgfplots/axis x discontinuity/.initial=none, /pgfplots/axis y discontinuity/.initial=none,
	/pgfplots/scale only axis/.initial=false, /pgfplots/scale only axis/.default=true,
	/pgfplots/width/.initial=, /pgfplots/height/.initial=,
	/pgfplots/x/.initial=, /pgfplots/y/.initial=, /pgfplots/z/.initial=,
	/pgfplots/xmin/.initial=, /pgfplots/xmax/.initial=, /pgfplots/ymin/.initial=, /pgfplots/ymax/.initial=,
	/pgfplots/zmin/.initial=, /pgfplots/zmax/.initial=,
	/pgfplots/min/.style={xmin={#1},ymin={#1},zmin={#1}}, /pgfplots/max/.style={xmax={#1},ymax={#1},zmax={#1}},
	/pgfplots/xtickmin/.initial=, /pgfplots/xtickmax/.initial=, /pgfplots/ytickmin/.initial=, /pgfplots/ytickmax/.initial=,
	/pgfplots/enlarge x limits/.initial=auto, /pgfplots/enlarge x limits/.default=true,
	/pgfplots/enlarge y limits/.initial=auto, /pgfplots/enlarge y limits/.default=true,
	/pgfplots/enlarge z limits/.initial=auto, /pgfplots/enlarge z limits/.default=true,
	/pgfplots/enlargelimits/.style={/pgfplots/enlarge x limits={#1},/pgfplots/enlarge y limits={#1},/pgfplots/enlarge z limits={#1}},
	/pgfplots/enlargelimits/.default=true,
	/pgfplots/x dir/.initial=normal, /pgfplots/y dir/.initial=normal, /pgfplots/z dir/.initial=normal,
	/pgfplots/x post scale/.initial=, /pgfplots/y post scale/.initial=, /pgfplots/z post scale/.initial=,
	/pgfplots/scale/.style={/pgfplots/x post scale={#1},/pgfplots/y post scale={#1},/pgfplots/z post scale={#1}},
	/pgfplots/unit vector ratio/.initial=, /pgfplots/unit vector ratio axis/.initial=,
	/pgfplots/scale mode/.initial=auto, /pgfplots/scale uniformly strategy/.initial=auto,
	/pgfplots/unit vector ratio*/.style={/pgfplots/unit vector ratio={#1},/pgfplots/unit rescale keep size=false},
	/pgfplots/unit rescale keep size/.initial=unless limits declared, /pgfplots/unit rescale keep size/.default=true,
	/pgfplots/axis equal/.is choice, /pgfplots/axis equal/.default=true,
	/pgfplots/axis equal/true/.style={/pgfplots/unit vector ratio=1 1 1,/pgfplots/unit rescale keep size=true},
	/pgfplots/axis equal/false/.style={/pgfplots/unit vector ratio=},
	/pgfplots/axis equal image/.is choice, /pgfplots/axis equal image/.default=true,
	/pgfplots/axis equal image/true/.style={/pgfplots/unit vector ratio=1 1 1,/pgfplots/unit rescale keep size=false},
	/pgfplots/axis equal image/false/.style={/pgfplots/unit vector ratio=},
	/pgfplots/xmode/.initial=linear, /pgfplots/ymode/.initial=linear, /pgfplots/zmode/.initial=linear,
	/pgfplots/view/az/.initial=25, /pgfplots/view/el/.initial=30,
	/pgfplots/view/.style 2 args={/pgfplots/view/az={#1},/pgfplots/view/el={#2}},
	/pgfplots/view/h/.style={/pgfplots/view/az={#1}}, /pgfplots/view/v/.style={/pgfplots/view/el={#1}},
	/pgfplots/plot box ratio/.initial=1 1 1,
	/pgfplots/every 3d description/.style={%
		/pgfplots/every axis x label/.style={at={(ticklabel cs:0.5)},anchor=near ticklabel},%
		/pgfplots/every axis y label/.style={at={(ticklabel cs:0.5)},anchor=near ticklabel},%
		/pgfplots/every x tick scale label/.style={at={(xticklabel cs:0.95,5pt)},anchor=near xticklabel,inner sep=0pt},
		/pgfplots/every y tick scale label/.style={at={(yticklabel cs:0.95,5pt)},anchor=near yticklabel,inner sep=0pt},
		/pgfplots/try min ticks=3,%
	},%
	/pgfplots/every 3d view {0}{90}/.style={/pgfplots/xlabel near ticks,/pgfplots/ylabel near ticks,/pgfplots/axis on top=true},
	/pgfplots/3d box/.initial=background, /pgfplots/3d box/.default=complete,
	/pgfplots/every 3d box foreground/.style={},
	/pgfplots/3d box foreground style/.style={/pgfplots/every 3d box foreground/.append style={#1}},
	/pgfplots/cycle list name/.initial=color,
	/pgfplots/cycle multi list/.code={\expandafter\def\csname pgfk@/pgfplots/@cycle lists/@multi@\endcsname{#1}\pgfkeyssetvalue{/pgfplots/cycle list name}{@multi@}},
	/pgfplots/cycle multiindex list/.code={\expandafter\def\csname pgfk@/pgfplots/@cycle lists/@multiindex@\endcsname{#1}\pgfkeyssetvalue{/pgfplots/cycle list name}{@multiindex@}},
	/pgfplots/cycle multiindex* list/.code={\expandafter\def\csname pgfk@/pgfplots/@cycle lists/@multiindex*@\endcsname{#1}\pgfkeyssetvalue{/pgfplots/cycle list name}{@multiindex*@}},
	/pgfplots/cycle list/.code={\expandafter\def\csname pgfk@/pgfplots/@cycle lists/@internal@\endcsname{#1}\pgfkeyssetvalue{/pgfplots/cycle list name}{@internal@}},
	/pgfplots/cycle list shift/.initial=,
	/pgfplots/mark list fill/.initial=.!80!black,
	/pgfplots/bar cycle list/.style={/pgfplots/cycle list={%
		{blue,fill=blue!30!white,mark=none},%
		{red,fill=red!30!white,mark=none},%
		{brown!60!black,fill=brown!30!white,mark=none},%
		{black,fill=gray,mark=none},%
		{violet!80!black,fill=violet,mark=none},%
		{green,fill=green!80!black,mark=none}%
		}
	},
	/pgfplots/area cycle list/.style={bar cycle list},
	/pgfplots/area style/.style={area cycle list,area legend,axis on top},
	/pgfplots/domain/.initial=, /pgfplots/y domain/.initial=,
	/pgfplots/domain y/.style={/pgfplots/y domain={#1}},
	/pgfplots/samples at/.initial=, /pgfplots/samples/.initial=, /pgfplots/samples y/.initial=,
	/pgfplots/variable/.code={\pgfkeysalso{/tikz/variable={#1}}},
	/pgfplots/variable y/.initial=y,
	/pgfplots/parametric/var 1d/.initial=t,
	/pgfplots/stack plots/.initial=false, /pgfplots/stack dir/.initial=plus,
	/pgfplots/reverse stacked plots/.initial=true, /pgfplots/reverse stacked plots/.default=true,
	/pgfplots/stacked ignores zero/.initial=false, /pgfplots/stacked ignores zero/.default=true,
	/pgfplots/stack negative/.initial=on previous,
	/pgfplots/bar direction/.initial=auto,
	/pgfplots/bar width/.code={\pgfkeysalso{/pgf/bar width={#1}}},
	/pgfplots/bar shift/.code={\pgfkeysalso{/pgf/bar shift={#1}}},
	/pgfplots/bar shift auto/.style={%
		/pgf/bar shift={-0.5*(\numplotsofactualtype*\pgfplotbarwidth + (\numplotsofactualtype-1)*(#1)) + (.5+\plotnumofactualtype)*\pgfplotbarwidth + \plotnumofactualtype*(#1)},%
	},
	/pgfplots/bar shift auto/.default={2pt},
	/pgfplots/ybar/.style={bar direction=y,bar cycle list,xtick align=outside,ybar legend,/pgfplots/bar shift auto={#1},/tikz/ybar},
	/pgfplots/ybar/.default=2pt,
	/pgfplots/xbar/.style={bar direction=x,bar cycle list,ytick align=outside,/pgfplots/xbar legend,/pgfplots/bar shift auto={#1},/tikz/xbar},
	/pgfplots/xbar/.default=2pt,
	/pgfplots/ybar interval/.style={bar direction=y,bar cycle list,x tick label as interval,xmajorgrids,xtick align=outside,xtick=data,ybar interval legend,
		bar interval width={#1/\numplotsofactualtype},bar interval shift={(\plotnumofactualtype+0.5)/\numplotsofactualtype},/tikz/ybar interval},
	/pgfplots/ybar interval/.default=1,
	/pgfplots/contour/draw color/.initial=mapped color, /pgfplots/contour/fill color/.initial=mapped color,
	/pgfplots/contour/label distance/.initial=70pt,
	/pgfplots/contour/labels/.initial=true, /pgfplots/contour/labels/.default=true,
	/pgfplots/contour/filled/.initial=false, /pgfplots/contour/filled/.default=true,
	/pgfplots/contour/number/.initial=5, /pgfplots/contour/levels/.initial=,
	/pgfplots/contour/levels from colormap/.initial=,
	/pgfplots/contour/every filled contour plot/.style={/pgfplots/area legend},
	/pgfplots/contour/every contour plot/.style={},
	/pgfplots/contour/every contour label/.style={inner sep=2pt, font=\scriptsize,
		every node/.style={mapped color!50!black, fill=white},
		/pgf/number format/relative*={\pgfplotspointmetarangeexponent}},
	/pgfplots/contour/labels over line/.style={/pgfplots/contour/every contour label/.append style={
		every node/.append style={fill=none, anchor=base, yshift=1pt}}},
	/pgfplots/contour/contour label style/.style={/pgfplots/contour/every contour label/.append style={#1}},
	/pgfplots/@contour format/.initial=standard,
	/pgfplots/contour prepared format/.code={\pgfkeyssetvalue{/pgfplots/@contour format}{#1}},
	/pgfplots/contour prepared filled/.style={/pgfplots/contour prepared={draw color=mapped color!80!black, labels=false, filled}},
	/pgfplots/@quiver/.initial=false,
	/pgfplots/quiver/u/.initial=, /pgfplots/quiver/v/.initial=, /pgfplots/quiver/w/.initial=,
	/pgfplots/quiver/scale arrows/.initial=1, /pgfplots/quiver/every arrow/.style={},
	/pgfplots/quiver/update limits/.initial=true, /pgfplots/quiver/update limits/.default=true,
	/pgfplots/quiver/arrow color/.initial=,
	/pgfplots/quiver/colored/.code={\pgfkeyssetvalue{/pgfplots/quiver/arrow color}{#1}\pgfkeysalso{/pgfplots/set point meta if empty=f(x)}},
	/pgfplots/quiver/colored/.default=mapped color,
	/pgfplots/quiver/u value/.code={\pgfkeyssetvalue{/pgfplots/quiver/u}{#1}},
	/pgfplots/quiver/v value/.code={\pgfkeyssetvalue{/pgfplots/quiver/v}{#1}},
	/pgfplots/quiver/w value/.code={\pgfkeyssetvalue{/pgfplots/quiver/w}{#1}},
	/pgfplots/quiver/before arrow/.code=, /pgfplots/quiver/after arrow/.code=,
	/pgfplots/quiver/u filter/.code=, /pgfplots/quiver/v filter/.code=, /pgfplots/quiver/w filter/.code=,
	/pgfplots/quiver/quiver legend/.style={},
	/pgfplots/@hist/.initial=false, /pgfplots/@boxplot/.initial=,
	/pgfplots/every boxplot/.style={},
	/pgfplots/boxplot/data/.initial=y, /pgfplots/boxplot/data value/.initial=, /pgfplots/boxplot/data filter/.code=,
	/pgfplots/boxplot/median/.initial=auto, /pgfplots/boxplot/lower quartile/.initial=auto,
	/pgfplots/boxplot/upper quartile/.initial=auto, /pgfplots/boxplot/lower whisker/.initial=auto,
	/pgfplots/boxplot/upper whisker/.initial=auto, /pgfplots/boxplot/average/.initial=,
	/pgfplots/boxplot/ensure mark/.initial=false, /pgfplots/boxplot/ensure mark/.default=true,
	/pgfplots/boxplot/estimator/.initial=legacy,
	/pgfplots/boxplot/lower quartile off/.initial=0.25, /pgfplots/boxplot/upper quartile off/.initial=0.75,
	/pgfplots/boxplot/whisker range/.initial=1.5,
	/pgfplots/boxplot/whisker extend/.initial=\pgfkeysvalueof{/pgfplots/boxplot/box extend}*0.8,
	/pgfplots/boxplot/box extend/.initial=0.8, /pgfplots/boxplot/draw direction/.initial=x,
	/pgfplots/boxplot/variable width/.initial=false, /pgfplots/boxplot/variable width/.default=true,
	/pgfplots/boxplot/sample size/.initial=auto, /pgfplots/boxplot/sample size min/.initial=,
	/pgfplots/boxplot/sample size max/.initial=, /pgfplots/boxplot/variable width min target/.initial=0.2,
	/pgfplots/boxplot/draw position/.initial=1+\plotnumofactualtype,
	/pgfplots/boxplot/draw relative anchor/.initial=0.5,
	/pgfplots/boxplot/every whisker/.style={}, /pgfplots/boxplot/every box/.style={},
	/pgfplots/boxplot/every median/.style={}, /pgfplots/boxplot/every average/.style={/tikz/mark=diamond*},
	/pgfplots/@boxplot macros/.code={\def\pgfplotsboxplotvalue##1{\pgfkeysvalueof{/pgfplots/boxplot/##1}}\let\boxplotvalue=\pgfplotsboxplotvalue},
	/pgfplots/@boxplot macros,
	/pgfplots/hist/data/.initial=y, /pgfplots/hist/data value/.initial=,
	/pgfplots/hist/data min/.initial=, /pgfplots/hist/data max/.initial=, /pgfplots/hist/bins/.initial=10,
	/pgfplots/hist/intervals/.initial=true, /pgfplots/hist/intervals/.default=true,
	/pgfplots/hist/cumulative/.initial=false, /pgfplots/hist/cumulative/.default=true,
	/pgfplots/hist/density/.initial=false, /pgfplots/hist/density/.default=true,
	/pgfplots/hist/handler/.style={/tikz/ybar interval},
	/pgfplots/hist/data filter/.code=,
	/pgfplots/hist/@symbolic coords/.initial=,
	/pgfplots/trim axis left/.style={/tikz/trim left={(current axis.south west)}}, /pgfplots/trim axis right/.style={/tikz/trim right={(current axis.south east)}},
	/tikz/trim axis left/.style={/pgfplots/trim axis left}, /tikz/trim axis right/.style={/pgfplots/trim axis right},
	/pgfplots/scatter/@pre marker code/.code={\def\tikra@scatter@usemapped{1}}, /pgfplots/scatter/@post marker code/.code={},
	/pgf/decoration/soft clip/.style={/pgf/decoration/name=soft clip}, /pgf/decoration/soft clip path/.initial=,
	/pgfplots/@date coordinates in/.initial=, /pgfplots/date ZERO/.initial=,
	/pgfplots/date coordinates in/.style={/pgfplots/@date coordinates in={#1},/pgfplots/#1ticklabel={\tick},/pgfplots/scaled #1 ticks=false},
	/pgfplots/hist/symbolic coords/.style={/pgfplots/hist/@symbolic coords={#1},/pgfplots/symbolic x coords={#1}},
	/pgfplots/xbar interval/.style={bar direction=x,bar cycle list,y tick label as interval,ytick=data,ymajorgrids,ytick align=outside,xbar interval legend,
		bar interval width={#1/\numplotsofactualtype},bar interval shift={(\plotnumofactualtype+0.5)/\numplotsofactualtype},/tikz/xbar interval},
	/pgfplots/xbar interval/.default=1,
	/pgfplots/bar interval width/.code={\pgfkeysalso{/pgf/bar interval width={#1}}},
	/pgfplots/bar interval shift/.code={\pgfkeysalso{/pgf/bar interval shift={#1}}},
	/pgfplots/xbar stacked/.style={bar direction=x,bar cycle list,single xbar legend,stack plots=x,stack dir=#1,/tikz/xbar},
	/pgfplots/xbar stacked/.default=plus,
	/pgfplots/ybar stacked/.style={bar direction=y,bar cycle list,single ybar legend,stack plots=y,stack dir=#1,/tikz/ybar},
	/pgfplots/ybar stacked/.default=plus,
	/pgfplots/ybar interval stacked/.style={bar direction=y,bar cycle list,single ybar legend,stack plots=y,stack dir=#1,/tikz/ybar interval},
	/pgfplots/ybar interval stacked/.default=plus,
	/pgfplots/xbar interval stacked/.style={bar direction=x,bar cycle list,single xbar legend,stack plots=x,stack dir=#1,/tikz/xbar interval},
	/pgfplots/xbar interval stacked/.default=plus,
	/pgfplots/xticklabel interval boundaries/.style={x tick label as interval,xticklabel={$\pgfmathprintnumber{\tick}$ -- $\pgfmathprintnumber{\nexttick}$}},
	/pgfplots/yticklabel interval boundaries/.style={y tick label as interval,yticklabel={$\pgfmathprintnumber{\tick}$ -- $\pgfmathprintnumber{\nexttick}$}},
	/pgfplots/unbounded coords/.initial=discard,
	/pgfplots/filter discard warning/.initial=true,
	/pgfplots/restrict x to domain/.initial=, /pgfplots/restrict y to domain/.initial=,
	/pgfplots/restrict x to domain*/.initial=, /pgfplots/restrict y to domain*/.initial=,
	/pgfplots/x filter/.code=, /pgfplots/y filter/.code=, /pgfplots/z filter/.code=, /pgfplots/pre filter/.code=, /pgfplots/filter point/.code=,
	/pgfplots/@restrict expr/.initial=, /pgfplots/@restrict expr domain/.initial=,
	/pgfplots/@visualization depends on/.initial=,
	/tikz/pos segment/.code=,
	/pgfplots/@ticklabel strut/.initial=false, /pgfplots/typeset ticklabels with strut/.style={/pgfplots/@ticklabel strut=true},
	/pgfplots/table/create col/linear regression/variance format/.initial=log,
	/pgfplots/restrict expr to domain/.code 2 args={\pgfkeyssetvalue{/pgfplots/@restrict expr}{#1}\pgfkeyssetvalue{/pgfplots/@restrict expr domain}{#2}\pgfkeysalso{/pgfplots/unbounded coords=jump}},
	/pgfplots/restrict expr to domain*/.code 2 args={\pgfkeyssetvalue{/pgfplots/@restrict expr}{#1}\pgfkeyssetvalue{/pgfplots/@restrict expr domain}{#2}},
	/pgfplots/skip coords between index/.style 2 args={/pgfplots/x filter/.append code={\ifnum\coordindex<#1 \else\ifnum\coordindex<#2 \def\pgfmathresult{}\fi\fi}},
	/pgfplots/point meta/.initial=, /pgfplots/point meta min/.initial=, /pgfplots/point meta max/.initial=,
	/pgfplots/every node near coord/.style={},
	/pgfplots/nodes near coords style/.style={/pgfplots/every node near coord/.append style={#1}},
	/pgfplots/node near coords style/.style={/pgfplots/every node near coord/.append style={#1}},
	/pgfplots/node near coord style/.style={/pgfplots/every node near coord/.append style={#1}},
	/pgfplots/nodes near coords align/.initial=auto,
	/pgfplots/@nodes near coords/.initial=,
	/pgfplots/nodes near coords/.code={\pgfkeyssetvalue{/pgfplots/@nodes near coords}{#1}\pgfkeysalso{/pgfplots/set point meta if empty=f(x)}},
	/pgfplots/nodes near coords/.default={\pgfmathprintnumber\pgfplotspointmeta},
	/pgfplots/nodes near coords*/.code={\pgfkeyssetvalue{/pgfplots/@nodes near coords}{#1}\pgfkeysalso{/pgfplots/set point meta if empty=f(x)}},
	/pgfplots/nodes near coords*/.default={\pgfmathprintnumber\pgfplotspointmeta},
	/pgfplots/nodes near coords bar offset/.initial=0.5,
	/pgfplots/execute at begin axis/.initial=, /pgfplots/execute at end axis/.initial=,
	/pgfplots/execute at begin plot/.initial=, /pgfplots/execute at end plot/.initial=,
	/pgfplots/execute at begin plot visualization/.initial=, /pgfplots/execute at end plot visualization/.initial=,
	/pgfplots/before end axis/.code=, /pgfplots/after end axis/.code=,
	/pgfplots/trig format plots/.initial=default,
	/pgfplots/set layers/.code=, /pgfplots/on layer/.initial=, /pgfplots/mark layer/.initial=,
	/pgfplots/cell picture/.code=, /pgfplots/lua backend/.code=, /pgfplots/lua debug/.code=,
	/pgfplots/disabledatascaling/.code={\pgfkeyssetvalue{/pgfplots/@disabledatascaling}{true}}, /pgfplots/@disabledatascaling/.initial=false, /pgfplots/data cs/.initial=cart,
	/pgfplots/colorbar/.initial=false, /pgfplots/colorbar/.default=true,
	/pgfplots/colormap name/.initial=hot,
	/pgfplots/legend image post style/.style={/pgfplots/every legend image post/.append style={#1}},
	/pgfplots/every axis plot no 0/.style={},
	/pgfplots/table/x/.initial=, /pgfplots/table/y/.initial=, /pgfplots/table/z/.initial=,
	/pgfplots/table/x index/.initial=, /pgfplots/table/y index/.initial=, /pgfplots/table/z index/.initial=,
	/pgfplots/table/x expr/.initial=, /pgfplots/table/y expr/.initial=, /pgfplots/table/z expr/.initial=,
	/pgfplots/table/meta/.initial=, /pgfplots/table/meta index/.initial=, /pgfplots/table/meta expr/.initial=,
	/pgfplots/table/x error/.initial=, /pgfplots/table/y error/.initial=,
	/pgfplots/table/x error index/.initial=, /pgfplots/table/y error index/.initial=,
	/pgfplots/table/x error plus/.initial=, /pgfplots/table/y error plus/.initial=,
	/pgfplots/table/x error minus/.initial=, /pgfplots/table/y error minus/.initial=,
	/pgfplots/table/y error plus index/.initial=, /pgfplots/table/y error minus index/.initial=,
	/pgfplots/table/col sep/.initial=space, /pgfplots/table/row sep/.initial=newline,
	/pgfplots/table/header/.initial=true, /pgfplots/table/skip first n/.initial=0,
	/pgfplots/table/format/.initial=auto, /pgfplots/table/ignore chars/.initial=,
	/pgfplots/table/comment chars/.initial=,
	/pgfplots/table/@search path/.initial=., /pgfplots/table/search path/.code={\pgfkeyssetvalue{/pgfplots/table/@search path}{#1}},
	/pgfplots/table/trim cells/.initial=false,
	/pgfplots/table/x error expr/.initial=, /pgfplots/table/y error expr/.initial=,
	/pgfplots/table/x error plus expr/.initial=, /pgfplots/table/y error plus expr/.initial=,
	/pgfplots/table/x error minus expr/.initial=, /pgfplots/table/y error minus expr/.initial=,
	/pgfplots/table/columns/.initial=,
	/pgfplots/small/.style={width=6.5cm,height=,tick label style={font=\footnotesize},label style={font=\small},max space between ticks=25},
	/pgfplots/footnotesize/.style={width=5cm,height=,legend style={font=\footnotesize},tick label style={font=\footnotesize},label style={font=\small},title style={font=\small},every axis title shift=0pt,max space between ticks=15,every mark/.append style={mark size=8},major tick length=0.1cm,minor tick length=0.066cm},
	/pgfplots/tiny/.style={width=4cm,height=,legend style={font=\tiny},tick label style={font=\tiny},label style={font=\tiny},title style={font=\footnotesize},every axis title shift=0pt,max space between ticks=12,every mark/.append style={mark size=6},major tick length=0.1cm,minor tick length=0.066cm,every legend image post/.append style={scale=0.8}},
	/pgfplots/normalsize/.style={/pgfplots/width=240pt,/pgfplots/height=207pt,/pgfplots/max space between ticks=35},
	/pgfplots/compat/.initial=default,
	/pgfplots/@compat/.initial=0,
	/pgfplots/xlabel absolute, /pgfplots/ylabel absolute,
	/pgfplots/group/horizontal sep/.initial=1cm, /pgfplots/group/vertical sep/.initial=1cm,
	/pgfplots/group/columns/.initial=1, /pgfplots/group/rows/.initial=1,
	/pgfplots/group/every plot/.style={}, /pgfplots/group/group name/.initial=group,
	/pgfplots/group/shared labels/.initial=false, /pgfplots/group/shared labels/.default=true,
	/pgfplots/group/xticklabels at/.initial=all, /pgfplots/group/yticklabels at/.initial=all,
	/pgfplots/group/xlabels at/.initial=all, /pgfplots/group/ylabels at/.initial=all,
	/pgfplots/group/x descriptions at/.code={\pgfkeyssetvalue{/pgfplots/group/xlabels at}{#1}\pgfkeyssetvalue{/pgfplots/group/xticklabels at}{#1}},
	/pgfplots/group/y descriptions at/.code={\pgfkeyssetvalue{/pgfplots/group/ylabels at}{#1}\pgfkeyssetvalue{/pgfplots/group/yticklabels at}{#1}},
	/pgfplots/group/empty plot/.style={/pgfplots/hide axis=true},
	/pgfplots/@empty xticklabels/.initial=false, /pgfplots/@empty yticklabels/.initial=false,
	/pgfplots/@empty zticklabels/.initial=false,
	/pgfplots/colormap/.code 2 args={\expandafter\def\csname pgfk@/pgfplots/@colormaps/#1\endcsname{#2}\pgfkeyssetvalue{/pgfplots/colormap name}{#1}\pgfkeysalso{/pgfplots/@colormap defined={#1}}},
	/pgfplots/colormap access/.initial=map,
	/pgfplots/colormap/hot/.style={/pgfplots/colormap={hot}{color(0cm)=(blue); color(1cm)=(yellow); color(2cm)=(orange); color(3cm)=(red)}},
	/pgfplots/colormap/viridis/.style={/pgfplots/colormap={viridis}{rgb=(0.267,0.00487,0.32942) rgb=(0.28192,0.08966,0.41241) rgb=(0.28026,0.1657,0.4765) rgb=(0.26366,0.23763,0.51877) rgb=(0.23744,0.3052,0.54192) rgb=(0.20862,0.36775,0.55267) rgb=(0.18225,0.42618,0.55711) rgb=(0.1592,0.48224,0.55807) rgb=(0.13777,0.53749,0.5549) rgb=(0.12115,0.59274,0.54465) rgb=(0.12808,0.64775,0.5235) rgb=(0.18065,0.7014,0.48819) rgb=(0.27415,0.75198,0.4366) rgb=(0.39517,0.79747,0.36775) rgb=(0.53561,0.83578,0.2819) rgb=(0.68895,0.86545,0.18272) rgb=(0.84557,0.88733,0.0997) rgb=(0.99324,0.90616,0.14394)}},
	/pgfplots/colormap/plasma/.style={/pgfplots/colormap={plasma}{rgb=(0.050383, 0.029803, 0.527975) rgb=(0.171574, 0.019706, 0.580806) rgb=(0.261183, 0.013308, 0.617911) rgb=(0.35015, 0.004382, 0.646298) rgb=(0.429719, 0.000831, 0.659425) rgb=(0.512206, 0.018833, 0.655209) rgb=(0.584391, 0.068579, 0.632812) rgb=(0.65558, 0.129725, 0.592317) rgb=(0.714883, 0.187299, 0.546338) rgb=(0.771958, 0.249237, 0.494813) rgb=(0.819651, 0.306812, 0.448306) rgb=(0.866078, 0.36966, 0.400126) rgb=(0.904601, 0.429797, 0.356329) rgb=(0.940313, 0.497642, 0.309197) rgb=(0.966798, 0.564396, 0.265118) rgb=(0.986345, 0.640969, 0.217948) rgb=(0.994324, 0.716681, 0.177208) rgb=(0.989587, 0.803205, 0.146529) rgb=(0.970533, 0.887896, 0.145919) rgb=(0.940015, 0.975158, 0.131326)}},
	/pgfplots/colormap/hot2/.style={/pgfplots/colormap={hot2}{[1cm]rgb255(0cm)=(0,0,0) rgb255(3cm)=(255,0,0) rgb255(6cm)=(255,255,0) rgb255(8cm)=(255,255,255)}},
	/pgfplots/colormap/bluered/.style={/pgfplots/colormap={bluered}{rgb255(0cm)=(0,0,180); rgb255(1cm)=(0,255,255); rgb255(2cm)=(100,255,0); rgb255(3cm)=(255,255,0); rgb255(4cm)=(255,0,0); rgb255(5cm)=(128,0,0)}},
	/pgfplots/colormap/cool/.style={/pgfplots/colormap={cool}{rgb255(0cm)=(255,255,255); rgb255(1cm)=(0,128,255); rgb255(2cm)=(255,0,255)}},
	/pgfplots/colormap/greenyellow/.style={/pgfplots/colormap={greenyellow}{rgb255(0cm)=(0,128,0); rgb255(1cm)=(255,255,0)}},
	/pgfplots/colormap/redyellow/.style={/pgfplots/colormap={redyellow}{rgb255(0cm)=(255,0,0); rgb255(1cm)=(255,255,0)}},
	/pgfplots/colormap/blackwhite/.style={/pgfplots/colormap={blackwhite}{gray(0cm)=(0); gray(1cm)=(1)}},
	/pgfplots/colormap/violet/.style={/pgfplots/colormap={violet}{rgb255=(25,25,122) color=(white) rgb255=(238,140,238)}},
	/pgfplots/colormap/jet/.style={/pgfplots/colormap={jet}{rgb255(0cm)=(0,0,128) rgb255(1cm)=(0,0,255) rgb255(3cm)=(0,255,255) rgb255(5cm)=(255,255,0) rgb255(7cm)=(255,0,0) rgb255(8cm)=(128,0,0)}},
	/pgfplots/colormap/viridis, /pgfplots/colormap/hot,
	/pgfplots/@mesh/.initial=, /pgfplots/@patch/.initial=false,
	/pgfplots/every mesh/.style={}, /pgfplots/every patch/.style={miter limit=1},
	/pgfplots/mesh legend/.code={\pgfkeyssetvalue{/pgfplots/@legend image}{line}},
	/pgfplots/shader/.initial=faceted, /pgfplots/faceted color/.initial=mapped color!80!black,
	/pgfplots/z buffer/.initial=default,
	/pgfplots/mesh input/.initial=lattice, /pgfplots/patch type/.initial=,
	/pgfplots/patch refines/.initial=0, /pgfplots/vertex count/.initial=, /pgfplots/patch table/.initial=, /pgfplots/patch table with point meta/.initial=,
	/pgfplots/patch table with individual point meta/.initial=, /pgfplots/patch to triangles/.initial=false, /pgfplots/patch to triangles/.default=true,
	/pgfplots/patch type sampling/.initial=false, /pgfplots/patch type sampling/.default=true,
	/pgfplots/mesh/rows/.initial=, /pgfplots/mesh/cols/.initial=,
	/pgfplots/mesh/ordering/.initial=x varies, /pgfplots/mesh/color input/.initial=colormap,
	/pgfplots/mesh/check/.initial=error,
	/pgfplots/mesh/interior colormap name/.initial=, /pgfplots/mesh/interior colormap thresh/.initial=0,
	/pgfplots/mesh/interior colormap/.code 2 args={\expandafter\def\csname pgfk@/pgfplots/@colormaps/#1\endcsname{#2}\pgfkeyssetvalue{/pgfplots/mesh/interior colormap name}{#1}},
	/pgfplots/late options/.initial=,
	/pgfplots/symbolic x coords/.initial=, /pgfplots/symbolic y coords/.initial=, /pgfplots/symbolic z coords/.initial=,
	/pgfplots/@axis type/.initial=cart, /pgfplots/data cs/.initial=cart,
	/pgfplots/every polar axis/.style={grid=major, xtick pos=right, ytick pos=right,
		every axis title shift=1.2\baselineskip, legend style={anchor=center}, /pgfplots/polar labels},
	/pgfplots/polar labels/.style={
		every axis x label/.style={at={(rel axis cs:0,0.5)}, anchor=north},
		every axis y label/.style={at={(0.2,0.85)}, anchor=center},
	},
	/pgfplots/polar labels style/.style={/pgfplots/polar labels/.append style={#1}},
	/pgfplots/xticklabel in circle/.style={xtick align=inside, xticklabel style={rotate=90, anchor=south west}},
	% ternary axes: limits and labels relative to the ranges (in percent) unless turned off
	/pgfplots/ternary limits relative/.initial=true, /pgfplots/ternary limits relative/.default=true,
	/pgfplots/ternary relative limits/.style={/pgfplots/ternary limits relative={#1}},
	/pgfplots/ternary start x/.initial=, /pgfplots/ternary start y/.initial=, /pgfplots/ternary start z/.initial=,
	/pgfplots/every ternary axis/.style={
		grid=major, /pgfplots/tick align=outside, xticklabel style={anchor=west},
		/pgfplots/every 3d description/.style={},
		every axis title shift=15pt,
		every axis x label/.style={anchor=near ticklabel, at={(ticklabel cs:0.5)}},
		every axis y label/.style={anchor=near ticklabel, at={(ticklabel cs:0.5)}},
		every axis z label/.style={anchor=near ticklabel, at={(ticklabel cs:0.5)}},
		every x tick scale label/.style={inner sep=0pt, anchor=near xticklabel, at={(xticklabel cs:0.95,5pt)}},
		every y tick scale label/.style={inner sep=0pt, anchor=near yticklabel, at={(yticklabel cs:0.95,5pt)}},
		every z tick scale label/.style={inner sep=0pt, anchor=near yticklabel, at={(yticklabel cs:0.95,5pt)}},
		every axis legend/.style={shape=rectangle, draw=black, fill=white, anchor=north west, at={(1.03,1.03)},
			cells={anchor=center}, inner xsep=3pt, inner ysep=2pt, nodes={inner sep=2pt, text depth=0.15em}},
	},
	% smith charts: where grid lines stop, the input's plane, the ring at the center
	/pgfplots/xgrid each nth passes y/.initial=, /pgfplots/xgrid each nth passes y start/.initial=0,
	/pgfplots/ygrid each nth passes x/.initial=, /pgfplots/ygrid each nth passes x start/.initial=0,
	/pgfplots/xgrid stop at y/.initial=, /pgfplots/ygrid stop at x/.initial=,
	/pgfplots/smithchart mirrored/.initial=false, /pgfplots/smithchart mirrored/.default=true,
	/pgfplots/is smithchart cs/.initial=false, /pgfplots/is smithchart cs/.default=true,
	/pgfplots/show origin/.initial=false, /pgfplots/show origin/.default=true,
	/pgfplots/every smithchart axis/.style={
		xmin=0, xmax=16000, ymin=-16000, ymax=16000, grid=both, scaled ticks=false,
		xtick align=center, ytick align=center, major tick style={draw=black},
		legend style={anchor=center}, every axis title shift=1.2\baselineskip,
	},
	% the ticks by the chart's size: few below 14cm, many below 20cm, dense beyond
	/pgfplots/default smithchart xtick/.style={}, /pgfplots/default smithchart ytick/.style={},
	/pgfplots/default smithchart xytick/.style={},
	/pgfplots/few smithchart ticks*/.style={
		default smithchart xtick/.style={xtick={0.2,0.5,1,2,5}},
		default smithchart ytick/.style={ytick={0,0.2,0.5,1,2,5,-0.2,-0.5,-1,-2,-5}},
		default smithchart xytick/.style={xgrid each nth passes y={2}, ygrid each nth passes x={2}},
	},
	/pgfplots/few smithchart ticks/.style={/pgfplots/few smithchart ticks*},
	/pgfplots/many smithchart ticks*/.style={
		default smithchart xtick/.style={xtick={0.1,0.2,0.3,0.4,0.5,1,1.5,2,3,4,5,10,20},
			minor xtick={0.6,0.7,0.8,0.9,1.1,1.2,1.3,1.4,1.6,1.7,1.8,1.9,2.2,2.4,2.6,2.8,3.2,3.4,3.6,3.8,4.5,6,7,8,9,50}},
		default smithchart ytick/.style={
			ytick={0,0.1,0.2,...,1,1.5,2,3,4,5,10,20,-0.1,-0.2,...,-1,-1.5,-2,-3,-4,-5,-10,-20},
			minor ytick={1.1,1.2,1.3,1.4,1.6,1.7,1.8,1.9,2.2,2.4,2.6,2.8,3.2,3.4,3.6,3.8,4.5,6,7,8,9,50,
				-1.1,-1.2,-1.3,-1.4,-1.6,-1.7,-1.8,-1.9,-2.2,-2.4,-2.6,-2.8,-3.2,-3.4,-3.6,-3.8,-4.5,-6,-7,-8,-9,-50}},
		default smithchart xytick/.style={xgrid each nth passes y={1,2,4,5,10,20}, ygrid each nth passes x={1,2,3,5,10:3,20:3}},
	},
	/pgfplots/many smithchart ticks/.style={many smithchart ticks*, every axis title shift=6pt, yticklabel in circle, show origin=true},
	/pgfplots/dense smithchart ticks*/.style={
		default smithchart xtick/.style={xtick={0.1,0.2,0.3,0.4,0.5,0.6,0.7,0.8,0.9,1,1.2,1.4,1.6,1.8,2,3,4,5,10,20},
			minor xtick={0.01,0.02,0.03,0.04,0.05,0.06,0.07,0.08,0.09,0.11,0.12,0.13,0.14,0.15,0.16,0.17,0.18,0.19,
				0.22,0.24,0.26,0.28,0.32,0.34,0.36,0.38,0.42,0.44,0.46,0.48,0.52,0.55,0.65,0.75,0.85,0.95,
				1.1,1.3,1.5,1.7,1.9,2.2,2.4,2.6,2.8,3.2,3.4,3.6,3.8,4.5,6,7,8,9,50}},
		default smithchart ytick/.style={
			ytick={0,0.1,0.2,...,1,1.2,1.4,1.6,1.8,2,3,4,5,10,20,-0.1,-0.2,...,-1,-1.2,-1.4,-1.6,-1.8,-2,-3,-4,-5,-10,-20},
			minor ytick={0.01,0.02,0.03,0.04,0.05,0.06,0.07,0.08,0.09,0.11,0.12,0.13,0.14,0.15,0.16,0.17,0.18,0.19,
				0.22,0.24,0.26,0.28,0.32,0.34,0.36,0.38,0.42,0.44,0.46,0.48,0.55,0.65,0.75,0.85,0.95,
				1.1,1.3,1.5,1.7,1.9,2.2,2.4,2.6,2.8,3.2,3.4,3.6,3.8,4.5,6,7,8,9,50,
				-0.01,-0.02,-0.03,-0.04,-0.05,-0.06,-0.07,-0.08,-0.09,-0.11,-0.12,-0.13,-0.14,-0.15,-0.16,-0.17,-0.18,-0.19,
				-0.22,-0.24,-0.26,-0.28,-0.32,-0.34,-0.36,-0.38,-0.42,-0.44,-0.46,-0.48,-0.55,-0.65,-0.75,-0.85,-0.95,
				-1.1,-1.3,-1.5,-1.7,-1.9,-2.2,-2.4,-2.6,-2.8,-3.2,-3.4,-3.6,-3.8,-4.5,-6,-7,-8,-9,-50}},
		default smithchart xytick/.style={
			xgrid each nth passes y={0.2 if < 0.2001,0.5 if < 0.50001,1 if < 1.001,2,4,5,10,20},
			ygrid each nth passes x={0.2 if < 0.2001,0.52 if < 0.52001,1 if < 1.001,2,3,5,10:3,20:3}},
	},
	/pgfplots/dense smithchart ticks/.style={yticklabel in circle, every axis title shift=6pt,
		dense smithchart ticks*, show origin=true, every major grid/.style={black!60}},
	% y tick labels along the circle, inside or around it
	/pgfplots/yticklabel in circle/.style={ytick align=inside, yticklabel style={rotate=90,
		sloped like y axis={execute for upside down={\tikzset{anchor=north east}}, reset nontranslations=false}, anchor=south west}},
	/pgfplots/yticklabel around circle/.style={ytick align=center, yticklabel style={rotate=90,
		sloped like y axis={execute for upside down={\tikzset{anchor=south west}}, reset nontranslations=false}, anchor=south east}},
	/pgfplots/yticklabel around circle*/.style={ytick align=center, yticklabel style={rotate=90,
		sloped like y axis={execute for upside down={\tikzset{anchor=north west}}, reset nontranslations=false}, anchor=north east}},
	% fill between: the styles of its segments
	/tikz/fill between/every segment/.style={}, /tikz/fill between/every last segment/.style={},
	/tikz/fill between/every odd segment/.style={}, /tikz/fill between/every even segment/.style={},
	/tikz/fill between/.search also={/pgfplots,/tikz},
	% tieline plots: each table row has a point and its tie's other end
	/pgfplots/tieline/every tieline/.style={}, /pgfplots/tieline/every curve/.style={},
	/pgfplots/tieline/tieline style/.style={/pgfplots/tieline/every tieline/.append style={#1}},
	/pgfplots/tieline/curve style/.style={/pgfplots/tieline/every curve/.append style={#1}},
	/pgfplots/tieline/each nth tie/.initial=,
	/pgfplots/table/tie end x/.initial=, /pgfplots/table/tie end y/.initial=, /pgfplots/table/tie end z/.initial=,
	/pgfplots/table/tie end x index/.initial=, /pgfplots/table/tie end y index/.initial=, /pgfplots/table/tie end z index/.initial=,
	/pgfplots/polar comb/.code={\pgfkeyssetvalue{/tikz/@plot handler}{polar comb}},
	/pgfplots/matrix plot/.style={/pgfplots/late options={/pgfplots/y dir=reverse,/pgfplots/axis on top},/pgfplots/matrix plot*},
	/pgfplots/matrix plot*/.style={/pgfplots/surf,/pgfplots/mesh input=image,/pgfplots/shader=flat corner,/pgfplots/z buffer=none},
	/pgfplots/scatter/.initial=false,
	/pgfplots/scatter src/.code={\pgfkeysalso{/pgfplots/point meta={#1}}},
	/pgfplots/scatter/use mapped color/.initial={draw=mapped color!80!black,fill=mapped color},
	/pgfplots/scatter/classes/.initial=,
	/pgfplots/only marks scatter/.style={/pgfplots/scatter,/tikz/only marks},
	/pgfplots/point meta rel/.initial=axis wide,
	/pgfplots/colorbar to name/.initial=, /pgfplots/colorbar pos*/.initial=,
	/pgfplots/parent axis width/.initial=6cm, /pgfplots/parent axis height/.initial=6cm,
	/pgfplots/@colorbar orientation/.initial=v, /pgfplots/@colorbar sampled/.initial=,
	/pgfplots/colorbar vertical/lowlevel/.code={\pgfkeyssetvalue{/pgfplots/@colorbar orientation}{v}},
	/pgfplots/colorbar horizontal/lowlevel/.code={\pgfkeyssetvalue{/pgfplots/@colorbar orientation}{h}},
	/pgfplots/every colorbar global/.style={
		axis on top, title=, xlabel=, ylabel=, zlabel=, legend entries=, grid=none,
		enlargelimits=false, scale only axis, xtick=, xticklabel=, ytick=, yticklabel=, zmin=, zmax=,
	},
	/pgfplots/colorbar right/.style={%
		/pgfplots/colorbar=true, /pgfplots/colorbar pos*=r,
		/pgfplots/colorbar shift/.style={xshift=0.3cm},
		/pgfplots/every colorbar/.style={%
			/pgfplots/every colorbar global,
			at={(parent axis.right of north east)}, anchor=north west,
			xmin=0, xmax=1,
			ymin=\pgfkeysvalueof{/pgfplots/point meta min}, ymax=\pgfkeysvalueof{/pgfplots/point meta max},
			height=\pgfkeysvalueof{/pgfplots/parent axis height}, width=\pgfkeysvalueof{/pgfplots/colorbar/width},
			xtick=\empty, yticklabel pos=right,
			colorbar vertical/lowlevel,
		}%
	},%
	/pgfplots/colorbar left/.style={%
		/pgfplots/colorbar right, /pgfplots/colorbar pos*=l,
		/pgfplots/colorbar shift/.style={xshift=-0.3cm},
		/pgfplots/every colorbar/.append style={at={(parent axis.left of north west)}, anchor=north east, yticklabel pos=left},
	},%
	/pgfplots/colorbar horizontal/.style={%
		/pgfplots/colorbar=true, /pgfplots/colorbar pos*=b,
		/pgfplots/colorbar shift/.style={yshift=-0.3cm},
		/pgfplots/every colorbar/.style={%
			/pgfplots/every colorbar global,
			at={(parent axis.below south west)}, anchor=north west,
			ymin=0, ymax=1,
			xmin=\pgfkeysvalueof{/pgfplots/point meta min}, xmax=\pgfkeysvalueof{/pgfplots/point meta max},
			width=\pgfkeysvalueof{/pgfplots/parent axis width}, height=\pgfkeysvalueof{/pgfplots/colorbar/width},
			xticklabel pos=left, ytick=\empty,
			colorbar horizontal/lowlevel,
		}%
	},%
	/pgfplots/colorbar right,
	/pgfplots/colorbar=false,
	/pgfplots/colorbar style/.style={/pgfplots/every colorbar/.append style={#1}},
	/pgfplots/colorbar/width/.initial=0.5cm,
	/pgfplots/colorbar sampled/.code={\pgfkeysalso{/pgfplots/colorbar=true}\pgfkeyssetvalue{/pgfplots/@colorbar sampled}{#1}\pgfkeyssetvalue{/pgfplots/@colorbar line}{false}},
	/pgfplots/colorbar sampled/.default={surf,mark=none,shader=flat},
	/pgfplots/colorbar sampled line/.code={\pgfkeysalso{/pgfplots/colorbar=true}\pgfkeyssetvalue{/pgfplots/@colorbar sampled}{#1}\pgfkeyssetvalue{/pgfplots/@colorbar line}{true}},
	/pgfplots/@colorbar line/.initial=false,
	/pgfplots/colorbar sampled line/.default={scatter,only marks},
	/pgfplots/colorbar as palette/.style={/pgfplots/colorbar sampled={surf,shader=flat corner}},
	/pgfplots/colorbar as legend/.style={/pgfplots/colorbar sampled={surf,shader=flat corner}},
	/pgfplots/every colorbar sampled line/.style={help lines},
	/pgfplots/colorbar sampled line style/.style={/pgfplots/every colorbar sampled line/.append style={#1}},
	/pgfplots/colorbar source/.code=,
	/tikz/legend columns/.style={/pgfplots/legend columns={#1}},
	/tikz/legend cell align/.style={/pgfplots/legend cell align={#1}},
	/tikz/legend plot pos/.style={/pgfplots/legend plot pos={#1}},
}
";

macro_rules! keyfn {
    ($name:ident, |$e:ident, $v:ident| $body:expr) => {
        fn $name($e: &mut Engine, $v: Option<Vec<Token>>) -> Result<(), Error> {
            $body
        }
    };
}

fn text(e: &mut Engine, v: &Option<Vec<Token>>) -> Result<String, Error> {
    Ok(match v {
        Some(v) => e.tex.expand_to_string(v.clone())?.trim().to_string(),
        None => String::new(),
    })
}

/// The key's last name, for keys that share their code (`/pgfplots/axis x line`).
fn current_name(e: &Engine) -> String {
    keys::name(&e.current_key).to_string()
}

/// Sets a stored value (`\pgfkeyssetvalue`).
pub fn set(e: &mut Engine, path: &str, value: &str) {
    e.tex.define_string(&keys::macro_name(path), value, false);
}

/// A stored value as tokens, if it is not empty.
pub fn value_tokens(e: &Engine, path: &str) -> Option<Vec<Token>> {
    keys::value(&e.tex, path).filter(|v| !keys::trim(v).is_empty())
}

/// A stored value, expanded, as text (empty if unset).
pub fn value(e: &mut Engine, path: &str) -> String {
    match keys::value(&e.tex, path) {
        Some(v) => e.tex.expand_to_string(v).map(|s| s.trim().to_string()).unwrap_or_default(),
        None => String::new(),
    }
}

/// A boolean value (`true`, `false`).
pub fn flag(e: &mut Engine, path: &str) -> bool {
    value(e, path) == "true"
}

/// A numeric value, if set and valid.
pub fn number(e: &mut Engine, path: &str) -> Option<f64> {
    let s = value(e, path);
    if s.is_empty() {
        return None;
    }
    e.number(&s).ok().filter(|v| v.is_finite())
}

/// A length value in cm, if set and valid (a number without unit is in pt).
pub fn length(e: &mut Engine, path: &str) -> Option<f64> {
    let s = value(e, path);
    if s.is_empty() {
        return None;
    }
    e.length(&s).ok().filter(|v| v.is_finite())
}

/// Runs a key list under `/pgfplots`.
pub fn run(e: &mut Engine, list: &str) -> Result<(), Error> {
    let saved = e.st.key_path.clone();
    let r = e.process_keys(tikra_tokens(list), Some("/pgfplots"));
    e.st.key_path = saved;
    r
}

/// The compatibility level, as a number: 1.18 → 118, `default` → 0.
pub fn compat(e: &Engine) -> u32 {
    keys::value(&e.tex, "/pgfplots/@compat").map(|v| tex::tokens_to_string(&v)).and_then(|s| s.trim().parse().ok()).unwrap_or(0)
}

// `/pgfplots/table/.unknown`: a key of `table[…]` that is not the table's is the plot's
// (`\pgfplots@addplotimpl@table@installkeypath`), `table[point meta=…]` for one.
keyfn!(k_table_unknown, |e, v| {
    let name = match e.tex.meaning_of("pgfkeyscurrentname") {
        Meaning::Macro(m) => tex::tokens_to_string(&tex::substitute(&m.body, &[])),
        _ => String::new(),
    };
    let saved = e.st.key_path.clone();
    let r = e.invoke(&format!("/pgfplots/{name}"), v);
    e.st.key_path = saved;
    r
});

// `/pgfplots/.unknown`: a key pgfplots does not know is a TikZ key (`.search also=/tikz`).
keyfn!(k_unknown, |e, v| {
    let name = match e.tex.meaning_of("pgfkeyscurrentname") {
        Meaning::Macro(m) => tex::tokens_to_string(&tex::substitute(&m.body, &[])),
        _ => String::new(),
    };
    let saved = e.st.key_path.clone();
    let r = e.invoke(&format!("/tikz/{name}"), v);
    e.st.key_path = saved;
    r
});

// `compat=1.x`: the level, and the label placement it implies (`compat/labels`).
// `visualization depends on={expr \as \macro}`: one more, kept as a group
keyfn!(k_visualization_depends_on, |e, v| {
    let path = keys::macro_name("/pgfplots/@visualization depends on");
    let mut list = keys::value(&e.tex, "/pgfplots/@visualization depends on").unwrap_or_default();
    list.push(Token::Char('{', tex::Cat::Begin));
    list.extend(v.unwrap_or_default());
    list.push(Token::Char('}', tex::Cat::End));
    e.tex.set_meaning(&Token::cs(&path), super::super::macro_tokens(list), false);
    Ok(())
});

keyfn!(k_compat, |e, v| {
    let s = text(e, &v)?;
    let level = match s.as_str() {
        "newest" | "1.18" => 118,
        "default" | "pre 1.3" | "" => 0,
        other => {
            let mut parts = other.split('.');
            let major: u32 = parts.next().and_then(|p| p.trim().parse().ok()).unwrap_or(1);
            let minor: u32 = parts.next().and_then(|p| p.trim().parse().ok()).unwrap_or(0);
            major * 100 + minor
        }
    };
    set(e, "/pgfplots/compat", &s);
    set(e, "/pgfplots/@compat", &level.to_string());
    if level >= 103 {
        run(e, "xlabel near ticks,ylabel near ticks")?;
    } else {
        run(e, "xlabel absolute,ylabel absolute")?;
    }
    Ok(())
});

/// `axis x line*=bottom`: where the line is, and the ticks go with it.
fn line_star(e: &mut Engine, axis: char, value: &str) {
    let (line, pos) = match value {
        "box" => ("box", "both"),
        "bottom" | "left" => ("low", "left"),
        "middle" | "center" => ("center", "center"),
        "top" | "right" => ("high", "right"),
        _ => ("none", "none"),
    };
    set(e, &format!("/pgfplots/@{axis} line"), line);
    set(e, &format!("/pgfplots/{axis}tick pos"), pos);
}

keyfn!(k_line_star, |e, v| {
    let name = current_name(e);
    let axis = name.chars().nth(5).unwrap_or('x');
    let value = text(e, &v)?;
    line_star(e, axis, &value);
    Ok(())
});

// `axis x line=middle`: the line, and the styles of `compat/axis line style` for it.
keyfn!(k_line, |e, v| {
    let name = current_name(e);
    let axis = name.chars().nth(5).unwrap_or('x');
    let value = text(e, &v)?;
    line_star(e, axis, &value);
    let post18 = compat(e) >= 108;
    let other = if axis == 'x' { 'y' } else { 'x' };
    let other_line = keys::value(&e.tex, &format!("/pgfplots/@{other} line")).map(|v| tex::tokens_to_string(&v)).unwrap_or_default();
    let list = match (value.as_str(), axis) {
        ("box", _) if post18 => format!("{axis}label near ticks,every boxed {axis} axis"),
        ("box", _) => format!("every boxed {axis} axis"),
        ("none", _) => format!("hide {axis} axis"),
        ("middle" | "center", 'x') if post18 => {
            let at = if other_line == "high" { "0" } else { "1" };
            format!("every axis x label/.style={{/pgfplots/near ticklabel align=inside,at={{(ticklabel* cs:{at})}},anchor=near ticklabel opposite}},every non boxed x axis")
        }
        ("middle" | "center", 'x') => {
            if other_line == "high" {
                "every axis x label/.style={at={(current axis.left of origin)},anchor=south west},every non boxed x axis".into()
            } else {
                "every axis x label/.style={at={(current axis.right of origin)},anchor=south east},every non boxed x axis".into()
            }
        }
        ("middle" | "center", 'y') if post18 => {
            if other_line == "high" {
                "every axis y label/.style={/pgfplots/near ticklabel align=outside,at={(ticklabel* cs:0)},anchor=near ticklabel opposite},every non boxed y axis".into()
            } else {
                "every axis y label/.style={/pgfplots/near ticklabel align=inside,at={(ticklabel* cs:1)},anchor=near ticklabel opposite},every non boxed y axis".into()
            }
        }
        ("middle" | "center", 'y') => {
            if other_line == "high" {
                "every axis y label/.style={at={(current axis.below origin)},anchor=south west},every non boxed y axis".into()
            } else {
                "every axis y label/.style={at={(current axis.above origin)},anchor=north west},every non boxed y axis".into()
            }
        }
        ("top" | "right", 'x') if post18 => "every axis x label/.style={at={(ticklabel cs:0.5)},anchor=near ticklabel},every non boxed x axis".into(),
        ("top" | "right", 'x') => "every axis x label/.style={at={(0.5,1)},anchor=south,yshift=15pt},every non boxed x axis".into(),
        ("top" | "right", 'y') if post18 => "every axis y label/.style={at={(ticklabel cs:0.5)},anchor=near ticklabel,rotate=90},every non boxed y axis".into(),
        ("top" | "right", 'y') => "every axis y label/.style={at={(1,0.5)},xshift=33pt,rotate=90},every non boxed y axis".into(),
        (_, a) => format!("every non boxed {a} axis"),
    };
    run(e, &list)
});

/// The plot handlers of TikZ that pgfplots draws (`const plot`, `ybar`…): stored as a value so
/// that groups restore them.
fn handler(e: &mut Engine, name: &str) {
    set(e, "/tikz/@plot handler", name);
}

keyfn!(k_handler, |e, _v| {
    let name = current_name(e);
    // TikZ's own `plot` draws with the same handler
    e.st.plot_handler = name.clone();
    let h = match name.as_str() {
        "smooth" | "sharp plot" => "",
        "const plot" | "const plot mark left" => "const",
        "const plot mark right" => "const right",
        "const plot mark mid" => "const mid",
        "jump mark left" => "jump left",
        "jump mark right" => "jump right",
        "jump mark mid" => "jump mid",
        other => other,
    };
    handler(e, h);
    Ok(())
});

// `mesh`, `surf`, `patch`: the plot is a mesh colored by its point meta.
keyfn!(k_mesh, |e, v| {
    let name = current_name(e);
    let value = text(e, &v)?;
    if value == "false" {
        set(e, "/pgfplots/@mesh", "");
        return Ok(());
    }
    set(e, "/pgfplots/@mesh", if name == "mesh" { "m" } else { "s" });
    if name == "patch" {
        set(e, "/pgfplots/@patch", "true");
        set(e, "/pgfplots/mesh input", "patches");
    }
    run(e, "set point meta if empty=f(x),every mesh/.try,/tikz/color=mapped color")?;
    if name == "patch" {
        run(e, "every patch")?;
    }
    Ok(())
});

// `scatter`: marks colored by the colormap.
keyfn!(k_scatter, |e, v| {
    let value = text(e, &v)?;
    let on = value != "false";
    set(e, "/pgfplots/scatter", if on { "true" } else { "false" });
    if on {
        run(e, "set point meta if empty=f(x)")?;
    }
    Ok(())
});

// `color of colormap={300 of viridis}` (0 to 1000), `const color of colormap`, `index of
// colormap={4 of viridis}` (the colormap's 5th color): the current color, as
// `\pgfplots@color@of@colormap` sets it.
keyfn!(k_colormap_color, |e, v| {
    let name = current_name(e);
    let s = text(e, &v)?;
    // `\i of jet` reads `0of jet`: the first `of` after the number
    let at = s.char_indices().find(|&(i, _)| s[i..].starts_with("of") && s[..i].trim_end().chars().last().is_some_and(|c| c.is_ascii_digit() || c == '.' || c == ')'));
    let (value, map) = match at {
        Some((i, _)) => (s[..i].trim().to_string(), Some(s[i + 2..].trim().to_string())),
        None => (s.trim().to_string(), None),
    };
    let mut map_name = map.unwrap_or_else(|| keys::value(&e.tex, "/pgfplots/colormap name").map(|v| tex::tokens_to_string(&v)).unwrap_or_default());
    // `of colormap/cool`: the style defines the map
    let spec_of = |e: &Engine, n: &str| keys::value(&e.tex, &format!("/pgfplots/@colormaps/{n}")).map(|v| tex::tokens_to_string(&v)).unwrap_or_default();
    let spec = if let Some(style) = map_name.strip_prefix("colormap/").map(str::to_string) {
        e.begin_group();
        let r = run(e, &format!("/pgfplots/colormap/{style}"));
        let spec = spec_of(e, &style);
        e.end_group();
        r?;
        map_name = style;
        spec
    } else {
        spec_of(e, &map_name)
    };
    let Some(cm) = super::colormap::Colormap::parse(&spec, &e.colors) else {
        e.warn(format!("{name}: colormap '{map_name}' not defined"));
        return Ok(());
    };
    let x = e.number(&value)?;
    let c = if name.starts_with("index") {
        let k = (x.max(0.0) as usize).min(cm.stops.len() - 1);
        cm.stops[k].1
    } else if name.starts_with("const") {
        cm.at_const(x / 1000.0)
    } else {
        cm.at(x / 1000.0)
    };
    let id = format!("pgfplots@colormap@{:.0}@{:.0}@{:.0}", c.r * 1e6, c.g * 1e6, c.b * 1e6);
    e.colors.insert(id.clone(), c);
    e.process_keys(super::toks(&format!("/tikz/color={id}")), Some("/tikz"))
});

// After a colormap is defined: `\pgfplotscolormaplastindexof{name}` (the index of its last color).
keyfn!(k_colormap_defined, |e, v| {
    let name = text(e, &v)?;
    let mut spec = value(e, &format!("/pgfplots/@colormaps/{name}"));
    // `samples of colormap=(10 of viridis)`: that many colors of the other colormap, equally spaced
    if let Some(rest) = spec.trim().strip_prefix("samples of colormap") {
        let arg = rest.trim().trim_start_matches('=').trim().trim_matches(|c| matches!(c, '(' | ')' | '{' | '}'));
        if let Some((n, other)) = arg.split_once(" of ") {
            let n: usize = n.trim().parse().unwrap_or(0);
            if let Some(cm) = super::colormap::named(e, other.split(',').next().unwrap_or("").trim()).filter(|_| n >= 2) {
                let colors: Vec<String> = (0..n).map(|k| cm.at(k as f64 / (n - 1) as f64)).map(|c| format!("rgb=({},{},{})", c.r, c.g, c.b)).collect();
                spec = colors.join(" ");
                set(e, &format!("/pgfplots/@colormaps/{name}"), &spec);
            }
        }
    }
    if let Some(cm) = super::colormap::Colormap::parse(&spec, &e.colors) {
        e.tex.define_string(&format!("pgfpl@cm@{name}@last"), &(cm.stops.len() - 1).to_string(), true);
    }
    Ok(())
});

// `x SI prefix=kilo` (the units library): the prefix of the unit in the label; with `change x base`
// the tick labels in that unit too (`scaled x ticks=base 10:-3`, without a scale label).
keyfn!(k_si_prefix, |e, v| {
    let c = current_key_axis(e);
    let name = text(e, &v)?;
    let table: &[(&str, i32, &str)] = &[
        ("yocto", 24, "y"),
        ("zepto", 21, "z"),
        ("atto", 18, "a"),
        ("femto", 15, "f"),
        ("pico", 12, "p"),
        ("nano", 9, "n"),
        ("micro", 6, "\\mu"),
        ("milli", 3, "m"),
        ("centi", 2, "c"),
        ("deci", 1, "d"),
        ("deca", -1, "da"),
        ("hecto", -2, "h"),
        ("kilo", -3, "k"),
        ("mega", -6, "M"),
        ("giga", -9, "G"),
        ("tera", -12, "T"),
        ("peta", -15, "P"),
        ("exa", -18, "E"),
        ("zetta", -21, "Z"),
        ("yotta", -24, "Y"),
    ];
    let Some(&(_, exp, prefix)) = table.iter().find(|(n, _, _)| *n == name) else {
        e.warn(format!("{c} SI prefix={name} not supported: ignored"));
        return Ok(());
    };
    set(e, &format!("/pgfplots/{c} unit prefix"), prefix);
    if flag(e, &format!("/pgfplots/change {c} base")) {
        set(e, &format!("/pgfplots/scaled {c} ticks"), &format!("base 10:{exp}"));
        set(e, &format!("/pgfplots/@{c} scale label off"), "true");
    }
    Ok(())
});

/// The axis letter of the key being run (`/pgfplots/x SI prefix` → `x`).
fn current_key_axis(e: &Engine) -> char {
    current_name(e).chars().next().unwrap_or('x')
}

// `colormap/PuBu`, `colormap/RdPu-6`: the ColorBrewer colormaps (`\usepgfplotslibrary{colorbrewer}`),
// equally spaced colors.
keyfn!(k_colormap_unknown, |e, _v| {
    let name = e.tex.expand_to_string(vec![Token::cs("pgfkeyscurrentname")])?;
    let Some(colors) = crate::color::brewer::colormap(&name) else {
        e.warn(format!("option '{name}' not supported: ignored"));
        return Ok(());
    };
    let spec: Vec<String> = colors.iter().map(|c| format!("rgb=({},{},{})", c.r, c.g, c.b)).collect();
    run(e, &format!("/pgfplots/colormap={{{name}}}{{{}}}", spec.join(" ")))
});

// `contour prepared={…}`: the contour handler, its keys in /pgfplots/contour.
keyfn!(k_contour_prepared, |e, v| {
    set(e, "/tikz/@plot handler", "contour prepared");
    let saved = e.st.key_path.clone();
    let mut list = super::toks("every contour plot,");
    list.extend(v.unwrap_or_default());
    let r = e.process_keys(list, Some("/pgfplots/contour"));
    e.st.key_path = saved;
    r
});

// `contour filled={…}`: a mesh drawn in bands of the levels, its keys in /pgfplots/contour.
keyfn!(k_contour_filled, |e, v| {
    run(e, "/pgfplots/surf")?;
    set(e, "/tikz/@plot handler", "contour filled");
    let saved = e.st.key_path.clone();
    let mut list = super::toks("every filled contour plot,");
    list.extend(v.unwrap_or_default());
    let r = e.process_keys(list, Some("/pgfplots/contour"));
    e.st.key_path = saved;
    r
});

// `sloped like x axis`: the node turned to the axis' direction on paper, never upside down; it
// replaces rotations given before (`\pgfplotstransformtoaxisdirection`).
// Its options: `execute for upside down={…}` (kept for after the node's other options, as
// pgfplots runs it when the node is placed), `allow upside down`, `reset nontranslations=false`.
fn sloped_like(e: &mut Engine, a: usize, v: Option<Vec<Token>>) -> Result<(), Error> {
    let Some(f) = e.frame.clone() else { return Ok(()) };
    let (code, allow, reset) = super::smith::sloped_options(&v.unwrap_or_default());
    // the direction at the tick being labelled (a smith chart's circle), else the axis'
    let d = f.slope.get().unwrap_or_else(|| f.e[a].scale(if f.max[a] < f.min[a] { -1.0 } else { 1.0 }));
    if d.norm() < 1e-12 {
        return Ok(());
    }
    let mut d = d;
    if d.x < 0.0 {
        if let Some(code) = code {
            // `\tikzset{anchor=…}`: its keys
            let t = crate::keys::trim(&code).to_vec();
            let keys = match t.split_first() {
                Some((first, rest)) if first.is_cs("tikzset") => crate::tex::strip_outer_braces(crate::keys::trim(rest).to_vec()),
                _ => t,
            };
            f.upside_down.replace(Some(keys));
        }
        if !allow {
            d = d.scale(-1.0);
        }
    }
    let p = e.st.pending;
    let base = if reset { crate::scene::Transform { a: 1.0, b: 0.0, c: 0.0, d: 1.0, ..p } } else { p };
    e.st.pending = base.compose(&crate::scene::Transform::rotation(d.y.atan2(d.x).to_degrees()));
    Ok(())
}
keyfn!(k_sloped_x, |e, v| sloped_like(e, 0, v));
keyfn!(k_sloped_y, |e, v| sloped_like(e, 1, v));
keyfn!(k_sloped_z, |e, v| sloped_like(e, 2, v));

// `intersection segments={of=A and B, sequence=…}` (the fillbetween library)
keyfn!(k_intersection_segments, |e, v| super::fillbetween::intersection_segments(e, v));

// `tieline={…}`: the curve through the ties' ends, and the ties.
keyfn!(k_tieline, |e, v| {
    set(e, "/tikz/@plot handler", "tieline");
    let saved = e.st.key_path.clone();
    let r = e.process_keys(v.unwrap_or_default(), Some("/pgfplots/tieline"));
    e.st.key_path = saved;
    r
});

// `quiver={u=…, v=…}`: arrows from each point by (u, v, w).
keyfn!(k_quiver, |e, v| {
    set(e, "/pgfplots/@quiver", "true");
    set(e, "/tikz/@plot handler", "quiver");
    let saved = e.st.key_path.clone();
    let r = e.process_keys(v.unwrap_or_default(), Some("/pgfplots/quiver"));
    e.st.key_path = saved;
    r
});

// `hist={bins=…}`: the plot's values are counted in bins, then drawn with `hist/handler`.
keyfn!(k_hist, |e, v| {
    set(e, "/pgfplots/@hist", "true");
    let saved = e.st.key_path.clone();
    let r = e.process_keys(v.unwrap_or_default(), Some("/pgfplots/hist"));
    e.st.key_path = saved;
    r?;
    run(e, "/pgfplots/hist/handler")
});

// `boxplot={…}`, `boxplot prepared={…}`: the box plot handler, its keys in /pgfplots/boxplot.
keyfn!(k_boxplot, |e, v| {
    let name = current_name(e);
    set(e, "/pgfplots/@boxplot", if name.ends_with("prepared") { "prepared" } else { "raw" });
    set(e, "/tikz/@plot handler", "boxplot");
    run(e, "/pgfplots/every boxplot")?;
    let saved = e.st.key_path.clone();
    let r = e.process_keys(v.unwrap_or_default(), Some("/pgfplots/boxplot"));
    e.st.key_path = saved;
    r?;
    if flag(e, "/pgfplots/boxplot/ensure mark") && e.st.plot_mark.is_none() {
        run(e, "/tikz/mark=*")?;
    }
    Ok(())
});

// `group style={…}`: keys of /pgfplots/group.
keyfn!(k_group_style, |e, v| {
    let saved = e.st.key_path.clone();
    let r = e.process_keys(v.unwrap_or_default(), Some("/pgfplots/group"));
    e.st.key_path = saved;
    r
});

// `group size=2 by 3`: columns by rows.
keyfn!(k_group_size, |e, v| {
    let s = text(e, &v)?;
    let (c, r) = s.split_once("by").unwrap_or((s.as_str(), "1"));
    set(e, "/pgfplots/group/columns", c.trim());
    set(e, "/pgfplots/group/rows", r.trim());
    Ok(())
});

keyfn!(k_set_meta, |e, v| {
    if keys::value(&e.tex, "/pgfplots/point meta").is_none_or(|m| keys::trim(&m).is_empty() || tex::tokens_to_string(&m).trim() == "none") {
        let value = v.unwrap_or_default();
        e.tex.set_meaning(&Token::cs(&keys::macro_name("/pgfplots/point meta")), super::super::macro_tokens(value), false);
    }
    Ok(())
});

// ------------------------------------------------------------------ /pgf/number format

fn nf(e: &mut Engine) -> &mut super::number::NumberFormat {
    &mut e.st.number_format
}

keyfn!(k_nf, |e, v| {
    let name = current_name(e);
    let value = text(e, &v)?;
    let on = |s: &str| !(s == "false");
    let f = nf(e);
    match name.as_str() {
        "fixed" => f.style = Style::Fixed,
        "frac" => f.style = Style::Frac,
        "sci" => f.style = Style::Sci,
        "std" => {
            f.style = match value.split_once(':') {
                Some((a, b)) => Style::Std { lower: a.trim().parse().ok(), upper: b.trim().parse().unwrap_or(4) },
                None if value.is_empty() => Style::Std { lower: None, upper: 4 },
                None => Style::Std { lower: value.trim().parse().ok(), upper: 4 },
            }
        }
        "int detect" => f.style = Style::IntDetect,
        "int trunc" => f.style = Style::IntTrunc,
        "relative" | "relative*" => f.style = Style::Relative(value.trim().parse().unwrap_or(0)),
        "@fixed relative" => f.style = Style::FixedRelative,
        "precision" => f.precision = value.trim().parse::<f64>().map_or(2, |p| p.max(0.0) as usize),
        "sci precision" => f.sci_precision = value.trim().parse().ok(),
        "fixed zerofill" => f.fixed_zerofill = on(&value),
        "sci zerofill" => f.sci_zerofill = on(&value),
        "zerofill" => {
            f.fixed_zerofill = on(&value);
            f.sci_zerofill = on(&value);
        }
        "set thousands separator" | "1000 sep" => f.thousands_sep = value,
        "set decimal separator" | "dec sep" => f.dec_sep = value,
        "use comma" => {
            f.dec_sep = "{,}".into();
            f.thousands_sep = ".".into();
        }
        "use period" => {
            f.dec_sep = ".".into();
            f.thousands_sep = "{,}".into();
        }
        "min exponent for 1000 sep" => f.min_exponent_1000 = value.trim().parse().unwrap_or(0),
        "showpos" | "print sign" => f.showpos = on(&value),
        "skip 0." => f.skip_zero = on(&value),
        "sci 10^e" | "sci 10e" => f.sci_style = SciStyle::TenE,
        "sci e" => f.sci_style = SciStyle::LowerE,
        "sci E" => f.sci_style = SciStyle::UpperE,
        "sci subscript" => f.sci_style = SciStyle::Subscript,
        "sci superscript" => f.sci_style = SciStyle::Superscript,
        "retain unit mantissa" => f.unit_mantissa = on(&value),
        _ => {}
    }
    Ok(())
});

const NATIVE: &[(&str, KeyFn)] = &[
    ("/pgfplots/.unknown", k_unknown),
    ("/pgfplots/table/.unknown", k_table_unknown),
    ("/pgfplots/compat", k_compat),
    ("/pgfplots/visualization depends on", k_visualization_depends_on),
    ("/pgfplots/mesh", k_mesh),
    ("/pgfplots/surf", k_mesh),
    ("/pgfplots/patch", k_mesh),
    ("/pgfplots/scatter", k_scatter),
    ("/pgfplots/set point meta if empty", k_set_meta),
    ("/pgfplots/axis x line", k_line),
    ("/pgfplots/axis y line", k_line),
    ("/pgfplots/axis z line", k_line),
    ("/pgfplots/axis x line*", k_line_star),
    ("/pgfplots/axis y line*", k_line_star),
    ("/pgfplots/axis z line*", k_line_star),
    ("/pgfplots/group style", k_group_style),
    ("/pgfplots/hist", k_hist),
    ("/pgfplots/quiver", k_quiver),
    ("/pgfplots/tieline", k_tieline),
    ("/tikz/intersection segments", k_intersection_segments),
    ("/tikz/sloped like x axis", k_sloped_x),
    ("/tikz/sloped like y axis", k_sloped_y),
    ("/tikz/sloped like z axis", k_sloped_z),
    ("/pgfplots/contour prepared", k_contour_prepared),
    ("/pgfplots/contour filled", k_contour_filled),
    ("/pgfplots/boxplot", k_boxplot),
    ("/pgfplots/boxplot prepared", k_boxplot),
    ("/pgfplots/colormap/.unknown", k_colormap_unknown),
    ("/pgfplots/x SI prefix", k_si_prefix),
    ("/pgfplots/y SI prefix", k_si_prefix),
    ("/pgfplots/z SI prefix", k_si_prefix),
    ("/pgfplots/@colormap defined", k_colormap_defined),
    ("/pgfplots/color of colormap", k_colormap_color),
    ("/pgfplots/const color of colormap", k_colormap_color),
    ("/pgfplots/index of colormap", k_colormap_color),
    ("/tikz/color of colormap", k_colormap_color),
    ("/tikz/const color of colormap", k_colormap_color),
    ("/tikz/index of colormap", k_colormap_color),
    ("/pgfplots/group/group size", k_group_size),
    ("/tikz/smooth", k_handler),
    ("/tikz/sharp plot", k_handler),
    ("/tikz/const plot", k_handler),
    ("/tikz/const plot mark left", k_handler),
    ("/tikz/const plot mark right", k_handler),
    ("/tikz/const plot mark mid", k_handler),
    ("/tikz/jump mark left", k_handler),
    ("/tikz/jump mark right", k_handler),
    ("/tikz/jump mark mid", k_handler),
    ("/tikz/ycomb", k_handler),
    ("/tikz/xcomb", k_handler),
    ("/tikz/ybar", k_handler),
    ("/tikz/xbar", k_handler),
    ("/tikz/ybar interval", k_handler),
    ("/tikz/xbar interval", k_handler),
    ("/tikz/polar comb", k_handler),
];

const NUMBER_FORMAT: &[&str] = &[
    "frac",
    "fixed",
    "sci",
    "std",
    "int detect",
    "int trunc",
    "relative",
    "relative*",
    "@fixed relative",
    "precision",
    "sci precision",
    "fixed zerofill",
    "sci zerofill",
    "zerofill",
    "set thousands separator",
    "1000 sep",
    "set decimal separator",
    "dec sep",
    "use comma",
    "use period",
    "min exponent for 1000 sep",
    "showpos",
    "print sign",
    "skip 0.",
    "sci 10^e",
    "sci 10e",
    "sci e",
    "sci E",
    "sci subscript",
    "sci superscript",
    "retain unit mantissa",
];

fn native(e: &mut Engine, path: &str, f: KeyFn) {
    let id = NATIVE_KEY_BASE + e.key_fns.len() as u16;
    e.key_fns.push(f);
    e.tex.define(&keys::macro_name(&format!("{path}/.@cmd")), Meaning::Native(id));
}

/// Defines the pgfplots keys (after TikZ's).
pub fn define(e: &mut Engine) {
    // what the colormap gives each point; set while a plot is drawn
    e.colors.insert("mapped color".into(), crate::color::Color::BLACK);
    for (path, f) in NATIVE {
        native(e, path, *f);
    }
    for name in NUMBER_FORMAT {
        native(e, &format!("/pgf/number format/{name}"), k_nf);
    }
    for (k, v) in [
        ("fixed zerofill", "true"),
        ("sci zerofill", "true"),
        ("zerofill", "true"),
        ("showpos", "true"),
        ("print sign", "true"),
        ("skip 0.", "true"),
        ("retain unit mantissa", "true"),
    ] {
        e.tex.set_meaning(&Token::cs(&keys::macro_name(&format!("/pgf/number format/{k}/.@def"))), super::super::macro_from(v), false);
    }
    let saved = e.st.key_path.clone();
    e.st.key_path = "/pgfplots".into();
    let mut lexer = crate::tex::Tex::new_lexer();
    lexer.push_text(DEFINITIONS);
    let mut tokens = Vec::new();
    while let Some(t) = lexer.get_token() {
        tokens.push(t);
    }
    if let Err(err) = e.run_tokens(tokens) {
        e.warn(format!("internal: pgfplots definitions failed: {err}"));
    }
    e.tex.warnings.clear();
    e.st.key_path = saved;
}
