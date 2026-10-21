//! The calendar library and pgf's date macros (`pgfcalendar.code.tex`, `tikzlibrarycalendar`).
//!
//! `\pgfcalendar{prefix}{start}{end}{code}` runs `code` once for every date from `start` to `end`
//! (`2000-01-01`, `2000-02-last`, `2000-01-01+5`), in one group, with `\pgfcalendarcurrentyear`,
//! `…month`, `…day` (two digits), `…weekday` (0 is Monday) and `…julian` set, and `\ifdate{tests}{yes}{no}`
//! choosing by the date: `Sunday`, `weekend`, `equals=…`, `day of month=1`, `between=… and …`…
//!
//! `\calendar (name) at (point) [options] if (tests) [options] {code} else …;` is a `\pgfcalendar`
//! whose code puts each day in a scope: `execute before day scope`, then the scope with `execute at
//! begin day scope`, the `if`s, the day code (a node with the day's number, named
//! `name-2000-01-20`) and `execute at end day scope`, then `execute after day scope`. The
//! arrangements (`week list`, `day list downward`…) and month labels are styles on these hooks;
//! the shifts they make stay from one day to the next, as the transformation of the group does.

use crate::scene::Transform;
use crate::tex::{self, Cat, Meaning, Token};
use crate::Error;

use super::{macro_from, macro_tokens, tikra_tokens, Engine};

/// Ids of the calendar's commands (`Meaning::Native`).
pub const BASE: u16 = 860;

const COMMANDS: &[&str] = &[
    "calendar",
    "pgfcalendar",
    "ifdate",
    "pgfcalendarifdate",
    "pgfcalendardatetojulian",
    "pgfcalendarjuliantodate",
    "pgfcalendarjuliantoweekday",
    "tikra@calendar",
    "tikra@calendarif",
];

pub fn define(e: &mut Engine) {
    for (i, name) in COMMANDS.iter().enumerate() {
        e.tex.define(name, Meaning::Native(BASE + i as u16));
    }
}

pub fn is_calendar(id: u16) -> bool {
    (BASE..BASE + COMMANDS.len() as u16).contains(&id)
}

/// The calendar's keys and templates, and the names of days and months.
pub const DEFINITIONS: &str = r"
\def\tikra@cal@append#1#2{\expandafter\def\expandafter#1\expandafter{#1#2}}
\def\tikra@cal@prepend#1#2{\def\tikra@cal@temp{#2}\expandafter\expandafter\expandafter\def\expandafter\expandafter\expandafter#1\expandafter\expandafter\expandafter{\expandafter\tikra@cal@temp#1}}
\def\tikra@cal@before{}\def\tikra@cal@after{}\def\tikra@cal@atbegin{}\def\tikra@cal@atend{}
\def\tikra@cal@dates{}\def\tikra@cal@width{1}
\def\tikra@cal@xshift{3.5ex}\def\tikra@cal@yshift{3ex}
\def\tikra@cal@monthxshift{9ex}\def\tikra@cal@monthyshift{9ex}
\def\tikzdaycode{\node[name=\pgfcalendarsuggestedname,every day]{\tikzdaytext};}
\def\tikzmonthcode{\node[every month]{\tikzmonthtext};}
\def\tikzyearcode{\node[every year]{\tikzyeartext};}
\def\tikzdaytext{\%d-}\def\tikzmonthtext{\%mt}\def\tikzyeartext{\%y0}
\tikzset{
  dates/.code={\def\tikra@cal@dates{#1}},
  execute before day scope/.code={\tikra@cal@append\tikra@cal@before{#1}},
  execute after day scope/.code={\tikra@cal@append\tikra@cal@after{#1}},
  execute at begin day scope/.code={\tikra@cal@append\tikra@cal@atbegin{#1}},
  execute at end day scope/.code={\tikra@cal@prepend\tikra@cal@atend{#1}},
  day xshift/.code={\def\tikra@cal@xshift{#1}}, day yshift/.code={\def\tikra@cal@yshift{#1}},
  month xshift/.code={\def\tikra@cal@monthxshift{#1}}, month yshift/.code={\def\tikra@cal@monthyshift{#1}},
  day code/.code={\def\tikzdaycode{#1}}, day text/.code={\def\tikzdaytext{#1}},
  month code/.code={\def\tikzmonthcode{#1}}, month text/.code={\def\tikzmonthtext{#1}},
  year code/.code={\def\tikzyearcode{#1}}, year text/.code={\def\tikzyeartext{#1}},
  if/.code={\tikra@calendarif{#1}}, days/.style={every day/.append style={#1}}, every day/.style={anchor=base east},
  every month/.style={}, every year/.style={},
  tikra@cal@width/.code={\def\tikra@cal@width{#1}},
  day list downward/.style={execute before day scope={\tikra@calendar{month down}},
    execute after day scope={\tikra@calendar{day down}}, tikra@cal@width=1},
  day list upward/.style={execute before day scope={\tikra@calendar{month up}},
    execute after day scope={\tikra@calendar{day up}}, tikra@cal@width=1},
  day list right/.style={execute before day scope={\tikra@calendar{month right}},
    execute after day scope={\tikra@calendar{day right}}, tikra@cal@width=30},
  day list left/.style={execute before day scope={\tikra@calendar{month left}},
    execute after day scope={\tikra@calendar{day left}}, tikra@cal@width=30},
  week list/.style={execute before day scope={\tikra@calendar{month down}},
    execute at begin day scope={\tikra@calendar{weekday}},
    execute after day scope={\tikra@calendar{week}}, tikra@cal@width=7},
  month list/.style={execute before day scope={\tikra@calendar{month down}\tikra@calendar{month start}},
    execute at begin day scope={\tikra@calendar{day of month}}, tikra@cal@width=37},
  month label left/.style={execute before day scope={\tikra@calendar{label left}},
    every month/.append style={anchor=base east,xshift=-3.5ex}},
  month label left vertical/.style={execute before day scope={\tikra@calendar{label left}},
    every month/.append style={anchor=base east,xshift=-4.5ex,yshift=2.25ex,rotate=90}},
  month label right/.style={execute before day scope={\tikra@calendar{label right}},
    every month/.append style={anchor=base west,xshift=1ex}},
  month label right vertical/.style={execute before day scope={\tikra@calendar{label right}},
    every month/.append style={anchor=base west,xshift=2ex,yshift=2.25ex,rotate=-90}},
  month label above centered/.style={execute before day scope={\tikra@calendar{label above centered}},
    every month/.append style={anchor=base}},
  month label above left/.style={execute before day scope={\tikra@calendar{label above left}},
    every month/.append style={anchor=base west}},
  month label above right/.style={execute before day scope={\tikra@calendar{label above right}},
    every month/.append style={anchor=base east}},
  month label below centered/.style={execute before day scope={\tikra@calendar{label below centered}},
    every month/.append style={anchor=base}},
  month label below left/.style={execute before day scope={\tikra@calendar{label below left}},
    every month/.append style={anchor=base west}},
}
\def\pgfcalendarweekdayname#1{\ifcase#1 Monday\or Tuesday\or Wednesday\or Thursday\or Friday\or Saturday\or Sunday\fi}
\def\pgfcalendarweekdayshortname#1{\ifcase#1 Mon\or Tue\or Wed\or Thu\or Fri\or Sat\or Sun\fi}
\def\pgfcalendarmonthname#1{\ifcase#1\or January\or February\or March\or April\or May\or June\or July\or August\or September\or October\or November\or December\fi}
\def\pgfcalendarmonthshortname#1{\ifcase#1\or Jan\or Feb\or Mar\or Apr\or May\or Jun\or Jul\or Aug\or Sep\or Oct\or Nov\or Dec\fi}
\def\pgfcalendarshorthand#1#2{\csname tikra@cal@short@#1#2\endcsname}
\expandafter\def\csname tikra@cal@short@d-\endcsname{\tikra@cal@day}
\expandafter\def\csname tikra@cal@short@d=\endcsname{\tikra@cal@paddedday}
\expandafter\def\csname tikra@cal@short@d0\endcsname{\pgfcalendarcurrentday}
\expandafter\def\csname tikra@cal@short@m-\endcsname{\tikra@cal@month}
\expandafter\def\csname tikra@cal@short@m=\endcsname{\tikra@cal@paddedmonth}
\expandafter\def\csname tikra@cal@short@m0\endcsname{\pgfcalendarcurrentmonth}
\expandafter\def\csname tikra@cal@short@y-\endcsname{\pgfcalendarcurrentyear}
\expandafter\def\csname tikra@cal@short@y=\endcsname{\pgfcalendarcurrentyear}
\expandafter\def\csname tikra@cal@short@y0\endcsname{\pgfcalendarcurrentyear}
\expandafter\def\csname tikra@cal@short@w.\endcsname{\pgfcalendarweekdayshortname{\pgfcalendarcurrentweekday}}
\expandafter\def\csname tikra@cal@short@wt\endcsname{\pgfcalendarweekdayname{\pgfcalendarcurrentweekday}}
\expandafter\def\csname tikra@cal@short@m.\endcsname{\pgfcalendarmonthshortname{\pgfcalendarcurrentmonth}}
\expandafter\def\csname tikra@cal@short@mt\endcsname{\pgfcalendarmonthname{\pgfcalendarcurrentmonth}}
";

/// The Julian day number of a date of the Gregorian calendar.
pub fn julian(year: i64, month: i64, day: i64) -> i64 {
    let a = (14 - month) / 12;
    let y = year + 4800 - a;
    let m = month + 12 * a - 3;
    day + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045
}

/// The date (year, month, day) of a Julian day number.
pub fn date(j: i64) -> (i64, i64, i64) {
    let a = j + 32044;
    let b = (4 * a + 3) / 146097;
    let c = a - 146097 * b / 4;
    let d = (4 * c + 3) / 1461;
    let e = c - 1461 * d / 4;
    let m = (5 * e + 2) / 153;
    let day = e - (153 * m + 2) / 5 + 1;
    let month = m + 3 - 12 * (m / 10);
    let year = 100 * b + d - 4800 + m / 10;
    (year, month, day)
}

/// The day of the week of a Julian day number, 0 for Monday.
pub fn weekday(j: i64) -> i64 {
    j.rem_euclid(7)
}

/// `2000-01-20`, `2000-02-last` (the month's last day), with an offset in days (`2000-01-01+-25`).
/// Without a year (`01-20`, in tests), the year is `year`.
pub fn parse_date(s: &str, year: Option<i64>) -> Option<i64> {
    let s = s.trim();
    let (d, offset) = match s.split_once('+') {
        Some((d, o)) => (d, o.trim().parse::<i64>().ok()?),
        None => (s, 0),
    };
    let parts: Vec<&str> = d.trim().split('-').map(str::trim).collect();
    let (y, m, day) = match parts.as_slice() {
        [y, m, d] => (y.parse::<i64>().ok()?, m.parse::<i64>().ok()?, *d),
        [m, d] => (year?, m.parse::<i64>().ok()?, *d),
        _ => return None,
    };
    let j = if day == "last" { julian(y + (m == 12) as i64, m % 12 + 1, 1) - 1 } else { julian(y, m, day.parse().ok()?) };
    Some(j + offset)
}

/// The Easter Sunday of a year (the Gregorian computus), as a Julian day number.
fn easter(y: i64) -> i64 {
    let g = y % 19 + 1;
    let c = y / 100 + 1;
    let x = 3 * c / 4 - 12;
    let z = (8 * c + 5) / 25 - 5;
    let d = 5 * y / 4 - x - 10;
    let mut e = (11 * g + 20 + z - x).rem_euclid(30);
    if (e == 25 && g > 11) || e == 24 {
        e += 1;
    }
    let mut n = 44 - e;
    if n < 21 {
        n += 30;
    }
    n += 7 - (d + n).rem_euclid(7);
    if n > 31 {
        julian(y, 4, n - 31)
    } else {
        julian(y, 3, n)
    }
}

impl Engine {
    pub fn calendar_command(&mut self, id: u16) -> Result<(), Error> {
        match COMMANDS[(id - BASE) as usize] {
            "calendar" => self.calendar(),
            "pgfcalendar" => {
                let prefix = self.argument_string()?;
                let start = self.argument_string()?;
                let end = self.argument_string()?;
                let code = self.argument()?;
                self.pgfcalendar(&prefix, &start, &end, code)
            }
            "ifdate" => {
                let tests = self.argument()?;
                let yes = self.argument()?;
                let no = self.argument()?;
                let j = self.macro_int("pgfcalendarcurrentjulian").ok_or_else(|| Error::Figure("\\ifdate outside a calendar".into()))?;
                let m = self.date_matches(j, tests)?;
                self.tex.push_tokens(if m { yes } else { no });
                Ok(())
            }
            "pgfcalendarifdate" => {
                let d = self.argument_string()?;
                let tests = self.argument()?;
                let yes = self.argument()?;
                let no = self.argument()?;
                let j = parse_date(&d, None).ok_or_else(|| Error::Figure(format!("not a date: '{d}'")))?;
                let m = self.date_matches(j, tests)?;
                self.tex.push_tokens(if m { yes } else { no });
                Ok(())
            }
            "pgfcalendardatetojulian" => {
                let d = self.argument_string()?;
                let target = self.argument()?;
                let j = parse_date(&d, None).ok_or_else(|| Error::Figure(format!("not a date: '{d}'")))?;
                self.set_int(target, j);
                Ok(())
            }
            "pgfcalendarjuliantodate" => {
                let j = self.argument()?;
                let j = self.tex.int_of(j)?;
                let (y, m, d) = date(j);
                for s in [y.to_string(), format!("{m:02}"), format!("{d:02}")] {
                    let target = self.argument()?;
                    if let Some(t) = target.into_iter().find(|t| !t.is_space()) {
                        self.tex.set_meaning(&t, macro_from(&s), false);
                    }
                }
                Ok(())
            }
            "pgfcalendarjuliantoweekday" => {
                let j = self.argument()?;
                let j = self.tex.int_of(j)?;
                let target = self.argument()?;
                self.set_int(target, weekday(j));
                Ok(())
            }
            "tikra@calendarif" => {
                // the `if` key: its value read as the `if` of a \calendar
                let mut v = self.argument()?;
                v.push(Token::cs("relax"));
                let depth = self.tex.depth();
                self.tex.push_frame(v);
                let r = self.calendar_if();
                self.tex.drain_to(depth);
                r
            }
            _ => {
                let what = self.argument_string()?;
                self.calendar_step(&what)
            }
        }
    }

    /// A count register or a macro set to a number (`\pgfcalendardatetojulian{…}{\c}`).
    fn set_int(&mut self, target: Vec<Token>, v: i64) {
        let Some(t) = target.into_iter().find(|t| !t.is_space()) else { return };
        match self.tex.meaning(&t) {
            Meaning::Count(r) => self.tex.set_count(r, v, false),
            _ => self.tex.set_meaning(&t, macro_from(&v.to_string()), false),
        }
    }

    /// A macro holding a number, as a number.
    fn macro_int(&mut self, name: &str) -> Option<i64> {
        if !self.tex.is_defined(name) {
            return None;
        }
        self.tex.expand_to_string(vec![Token::cs(name)]).ok()?.trim().parse().ok()
    }

    /// The text of a macro, expanded.
    fn macro_text(&mut self, name: &str) -> String {
        self.tex.expand_to_string(vec![Token::cs(name)]).map(|s| s.trim().to_string()).unwrap_or_default()
    }

    /// `\pgfcalendar`: `code` for every date from `start` to `end`, in one group.
    pub fn pgfcalendar(&mut self, prefix: &str, start: &str, end: &str, code: Vec<Token>) -> Result<(), Error> {
        let first = parse_date(start, None).ok_or_else(|| Error::Figure(format!("not a date: '{start}'")))?;
        let last = parse_date(end, None).ok_or_else(|| Error::Figure(format!("not a date: '{end}'")))?;
        if last - first > 20_000 {
            return Err(Error::Figure(format!("calendar from {start} to {end}: too many days")));
        }
        self.begin_group();
        self.tex.define_string("pgfcalendarprefix", prefix, false);
        self.tex.define_string("pgfcalendarbeginiso", start.trim(), false);
        self.tex.define_string("pgfcalendarendiso", end.trim(), false);
        self.tex.define_string("pgfcalendarbeginjulian", &first.to_string(), false);
        self.tex.define_string("pgfcalendarendjulian", &last.to_string(), false);
        let mut r = Ok(());
        for j in first..=last {
            let (y, m, d) = date(j);
            for (name, v) in [
                ("pgfcalendarcurrentjulian", j.to_string()),
                ("pgfcalendarcurrentyear", y.to_string()),
                ("pgfcalendarcurrentmonth", format!("{m:02}")),
                ("pgfcalendarcurrentday", format!("{d:02}")),
                ("pgfcalendarcurrentweekday", weekday(j).to_string()),
                ("tikra@cal@day", d.to_string()),
                ("tikra@cal@month", m.to_string()),
                ("pgfcalendarsuggestedname", if prefix.is_empty() { String::new() } else { format!("{prefix}-{y}-{m:02}-{d:02}") }),
            ] {
                self.tex.define_string(name, &v, false);
            }
            // `\%d=`: a figure's width before a single digit
            let pad = |v: i64| if v < 10 { format!("\\phantom{{1}}{v}") } else { v.to_string() };
            self.tex.define_macro("tikra@cal@paddedday", tikra_tokens(&pad(d)), false);
            self.tex.define_macro("tikra@cal@paddedmonth", tikra_tokens(&pad(m)), false);
            self.count_step()?;
            r = self.run_tokens(code.clone());
            if r.is_err() {
                break;
            }
        }
        self.end_group();
        r
    }

    /// Whether the date `j` passes one of `tests` (a key list: `Sunday, equals=2000-01-20`).
    fn date_matches(&mut self, j: i64, tests: Vec<Token>) -> Result<bool, Error> {
        let (year, _, day) = date(j);
        let wd = weekday(j);
        let mut matches = false;
        for item in crate::keys::split_list(&tests) {
            let value = match item.value {
                Some(v) => Some(self.tex.expand_to_string(v)?.trim().to_string()),
                None => None,
            };
            let at = |s: &str| parse_date(s, Some(year)).ok_or_else(|| Error::Figure(format!("not a date: '{s}'")));
            let v = value.as_deref().unwrap_or("");
            matches |= match item.key.as_str() {
                "all" => true,
                "Monday" => wd == 0,
                "Tuesday" => wd == 1,
                "Wednesday" => wd == 2,
                "Thursday" => wd == 3,
                "Friday" => wd == 4,
                "Saturday" => wd == 5,
                "Sunday" => wd == 6,
                "workday" => wd < 5,
                "weekend" => wd > 4,
                "equals" => j == at(v)?,
                "day of month" => v.parse::<i64>().ok() == Some(day),
                "end of month" => date(j + value.as_deref().and_then(|v| v.parse().ok()).unwrap_or(1)).2 == 1,
                "at least" => j >= at(v)?,
                "at most" => j <= at(v)?,
                "between" => {
                    let (a, b) = v.split_once(" and ").or_else(|| v.split_once("and")).ok_or_else(|| Error::Figure(format!("between={v}: no 'and'")))?;
                    j >= at(a)? && j <= at(b)?
                }
                "Easter" => j == easter(year) + value.as_deref().and_then(|v| v.parse().ok()).unwrap_or(0),
                other => {
                    self.warn(format!("\\ifdate: unknown test '{other}'"));
                    false
                }
            };
        }
        Ok(matches)
    }

    /// `\calendar (name) at (point) [options] if (tests) … ;`
    fn calendar(&mut self) -> Result<(), Error> {
        self.begin_group();
        let r = self.calendar_body();
        self.end_group();
        r
    }

    fn calendar_body(&mut self) -> Result<(), Error> {
        let shorthand = self.tex.meaning_of("pgfcalendarshorthand");
        self.tex.set_meaning(&Token::cs("%"), shorthand, false);
        for name in ["tikra@cal@before", "tikra@cal@after", "tikra@cal@atbegin", "tikra@cal@atend", "tikra@cal@ifs"] {
            self.tex.define_string(name, "", false);
        }
        self.st.pending = Transform::default();
        self.st.node.at = None;
        self.process_keys(tikra_tokens("every calendar/.try"), Some("/tikz"))?;
        let mut prefix = String::new();
        let mut guard = 0;
        loop {
            guard += 1;
            let Some(t) = self.tex.next_nonspace() else { break };
            if guard > 1000 {
                return Err(Error::Figure("\\calendar: giving up on this calendar".into()));
            }
            match &t {
                t if t.is_char(';') => break,
                t if t.is_char('(') => {
                    let name = self.tex.read_until_char(')', false)?;
                    prefix = self.tex.expand_to_string(name)?.trim().to_string();
                }
                t if t.is_char('[') => {
                    let o = self.tex.read_until_char(']', false)?;
                    self.process_keys(o, Some("/tikz"))?;
                }
                t if t.is_char('a') => {
                    // at (point)
                    let _ = self.tex.get_token();
                    let p = match self.tex.next_nonspace() {
                        Some(x) if x.is_char('(') => self.tex.read_until_char(')', false)?,
                        _ => return Err(Error::Figure("\\calendar at: a point is missing".into())),
                    };
                    let s = self.tex.expand_to_string(p)?;
                    self.st.node.at = Some(self.coordinate(&s)?.p);
                }
                t if t.is_char('i') => {
                    let _ = self.tex.get_token();
                    self.calendar_if()?;
                }
                _ => {
                    let m = self.tex.meaning(&t);
                    if m.is_expandable() {
                        self.tex.expand(&t, &m)?;
                    } else {
                        return Err(Error::Figure(format!("\\calendar: unexpected {}", tex::show_token(&t))));
                    }
                }
            }
        }
        self.st.apply_pending();
        // `at (point)` or `at=`: where the calendar's origin goes
        if let Some(p) = self.st.node.at.take() {
            let local = crate::reader::invert(&self.st.transform).apply(p);
            self.st.transform = self.st.transform.compose(&Transform::translation(local.x, local.y));
        }
        let dates = self.macro_text("tikra@cal@dates");
        let Some((start, end)) = dates.split_once("to") else {
            return Err(Error::Figure(format!("\\calendar: dates={dates} is not 'start to end'")));
        };
        let code = tikra_tokens(r"\tikra@cal@before\begin{scope}\tikra@cal@atbegin\tikra@cal@ifs\tikzdaycode\tikra@cal@atend\end{scope}\tikra@cal@after");
        self.pgfcalendar(&prefix, start, end, code)
    }

    /// `if (tests) [options] else {code}` (either code an `[options]` or a `{code}`, the `else` part
    /// optional): appended to what each day's scope runs.
    fn calendar_if(&mut self) -> Result<(), Error> {
        let tests = match self.tex.next_nonspace() {
            Some(x) if x.is_char('(') => self.tex.read_until_char(')', false)?,
            _ => return Err(Error::Figure("\\calendar if: the tests are missing".into())),
        };
        let yes = self.calendar_if_branch()?;
        let no = match self.tex.next_nonspace() {
            Some(x) if x.is_char('e') => {
                for _ in 0..3 {
                    let _ = self.tex.get_token();
                }
                self.calendar_if_branch()?
            }
            Some(x) => {
                self.tex.back(x);
                Vec::new()
            }
            None => Vec::new(),
        };
        let mut ifs = match self.tex.meaning_of("tikra@cal@ifs") {
            Meaning::Macro(m) => tex::substitute(&m.body, &[]),
            _ => Vec::new(),
        };
        ifs.push(Token::cs("ifdate"));
        for v in [tests, yes, no] {
            ifs.push(Token::Char('{', Cat::Begin));
            ifs.extend(v);
            ifs.push(Token::Char('}', Cat::End));
        }
        self.tex.set_meaning(&Token::cs("tikra@cal@ifs"), macro_tokens(ifs), false);
        Ok(())
    }

    /// The code of an `if`: `[options]` (set in the day's scope) or `{code}`.
    fn calendar_if_branch(&mut self) -> Result<Vec<Token>, Error> {
        match self.tex.next_nonspace() {
            Some(x) if x.is_char('[') => {
                let o = self.tex.read_until_char(']', false)?;
                let mut v = vec![Token::cs("tikzset"), Token::Char('{', Cat::Begin)];
                v.extend(o);
                v.push(Token::Char('}', Cat::End));
                Ok(v)
            }
            Some(Token::Char(_, Cat::Begin)) => self.tex.read_group_body(),
            Some(x) => {
                self.tex.back(x);
                Ok(Vec::new())
            }
            None => Ok(Vec::new()),
        }
    }

    /// A length from a calendar macro (`\tikra@cal@xshift`), in cm; `em` and `ex` are the current
    /// font's.
    fn calendar_length(&mut self, name: &str) -> Result<f64, Error> {
        let s = self.macro_text(name);
        self.font_length(&s)
    }

    fn font_length(&mut self, s: &str) -> Result<f64, Error> {
        let v = self.length(s)?;
        Ok(if s.contains("em") || s.contains("ex") { v * self.st.font_size / 10.0 } else { v })
    }

    fn calendar_shift(&mut self, x: f64, y: f64) {
        self.st.transform = self.st.transform.compose(&Transform::translation(x, y));
    }

    /// `\tikra@calendar{what}`: one step of an arrangement or a month label, for the current day.
    fn calendar_step(&mut self, what: &str) -> Result<(), Error> {
        let j = self.macro_int("pgfcalendarcurrentjulian").unwrap_or(0);
        let begin = self.macro_int("pgfcalendarbeginjulian").unwrap_or(0);
        let (_, _, day) = date(j);
        let wd = weekday(j);
        // a new month moves the days on, except at the calendar's first day
        let new_month = day == 1 && j != begin;
        match what {
            "month down" | "month up" if new_month => {
                let d = self.calendar_length("tikra@cal@monthyshift")?;
                self.calendar_shift(0.0, if what == "month up" { d } else { -d });
            }
            "month right" | "month left" if new_month => {
                let d = self.calendar_length("tikra@cal@monthxshift")?;
                self.calendar_shift(if what == "month right" { d } else { -d }, 0.0);
            }
            "day down" | "day up" => {
                let d = self.calendar_length("tikra@cal@yshift")?;
                self.calendar_shift(0.0, if what == "day up" { d } else { -d });
            }
            "day right" | "day left" => {
                let d = self.calendar_length("tikra@cal@xshift")?;
                self.calendar_shift(if what == "day right" { d } else { -d }, 0.0);
            }
            "weekday" => {
                let d = self.calendar_length("tikra@cal@xshift")?;
                self.calendar_shift(wd as f64 * d, 0.0);
            }
            "week" if wd == 6 => {
                let d = self.calendar_length("tikra@cal@yshift")?;
                self.calendar_shift(0.0, -d);
            }
            "month start" if day == 1 || j == begin => {
                // the weekday of the month's first day, for `month list`
                let first = weekday(j - day + 1);
                self.tex.define_string("tikra@cal@monthstart", &first.to_string(), false);
            }
            "day of month" => {
                let d = self.calendar_length("tikra@cal@xshift")?;
                let first = self.macro_int("tikra@cal@monthstart").unwrap_or(0);
                self.calendar_shift((day + first - 1) as f64 * d, 0.0);
            }
            label if label.starts_with("label ") && day == 1 => self.calendar_month_label(&label[6..])?,
            _ => {}
        }
        Ok(())
    }

    /// The month's label on its first day, placed as `month label …` says, in a group.
    fn calendar_month_label(&mut self, place: &str) -> Result<(), Error> {
        let x = self.calendar_length("tikra@cal@xshift")?;
        let y = self.calendar_length("tikra@cal@yshift")?;
        let width = self.macro_text("tikra@cal@width").parse::<f64>().unwrap_or(1.0);
        let ex = self.font_length("1ex")?;
        let row = (width - 1.0) * x;
        let (dx, dy) = match place {
            "right" => (row, 0.0),
            "above centered" => (row / 2.0 - 1.5 * ex, 1.25 * y),
            "above left" => (-3.25 * ex, 1.25 * y),
            "above right" => (row, 1.25 * y),
            "below centered" => (row / 2.0 - 1.5 * ex, -1.25 * y),
            "below left" => (-3.25 * ex, -1.25 * y),
            _ => (0.0, 0.0),
        };
        self.begin_group();
        self.calendar_shift(dx, dy);
        let r = self.run_tokens(vec![Token::cs("tikzmonthcode")]);
        self.end_group();
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn julian_days_and_back() {
        assert_eq!(julian(2000, 1, 1), 2451545);
        assert_eq!(date(2451545), (2000, 1, 1));
        // 2000-01-01 was a Saturday
        assert_eq!(weekday(2451545), 5);
        assert_eq!(parse_date("2000-02-last", None), Some(julian(2000, 2, 29)));
        assert_eq!(parse_date("2000-12-last", None), Some(julian(2000, 12, 31)));
        assert_eq!(parse_date("2000-01-01+-1", None), Some(julian(1999, 12, 31)));
        assert_eq!(parse_date("01-20", Some(2007)), Some(julian(2007, 1, 20)));
        assert_eq!(date(easter(2024)), (2024, 3, 31));
    }

    #[test]
    fn a_week_list_names_its_days() {
        let (canvas, warnings) = super::super::run_document(
            r"\begin{tikzpicture}\calendar (c) [dates=2000-01-01 to 2000-01-31,week list] if (Sunday) [red];\draw (c-2000-01-20) circle (4pt);\end{tikzpicture}",
        )
        .unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        assert!(canvas.items.len() > 31);
    }
}
