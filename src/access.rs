//! Accessibility in three layers that do not depend on one another:
//!
//! 1. The **registry** (`assets/tex/accessgate-registry.toml`): data, not code, saying which commands
//!    and environments carry structure and what each must provide. A file of the same name beside the
//!    presentation adds the author's own macros.
//! 2. The **source layer** (`accessgate.sty`, used by texoravideo and texoraimage): tags only where
//!    tagging is on, and otherwise typesets as before, in every editor and engine.
//! 3. **Checking**: `check` reads the source against the registry (any engine, any class, no LaTeX
//!    needed); `build` makes a tagged copy through a wrapper file (for Beamer, which refuses tagging,
//!    a copy converted to the `ltx-talk` class), leaving the document and its PDF alone, then reads
//!    the log's `[code]` reports and the tags in the PDF it made.
use crate::video_prep::{balanced, resolve, uncomment};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const REGISTRY: &str = include_str!("../assets/tex/accessgate-registry.toml");
pub const GATE_STY: &str = include_str!("../assets/tex/accessgate.sty");
/// The registry's file name; one beside the presentation adds to TeXora's.
pub const REGISTRY_FILE: &str = "accessgate-registry.toml";

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Code {
    #[serde(default)]
    pub wcag: String,
    #[serde(default)]
    pub summary: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Entry {
    /// The tagged-PDF structure it maps to; kept as data for other tools, not used by the check.
    #[serde(default)]
    #[allow(dead_code)]
    pub role: String,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub checks: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct Policy {
    #[serde(default)]
    default: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct Raw {
    #[serde(default)]
    policy: Policy,
    #[serde(default)]
    codes: BTreeMap<String, Code>,
    #[serde(default)]
    command: BTreeMap<String, Entry>,
    #[serde(default)]
    environment: BTreeMap<String, Entry>,
}

/// The registry, TeXora's entries with the author's added.
#[derive(Clone, Debug, Default)]
pub struct Registry {
    pub codes: BTreeMap<String, Code>,
    pub commands: BTreeMap<String, Entry>,
    pub environments: BTreeMap<String, Entry>,
    /// Reports are errors by default (`[policy] default = "error"`).
    pub errors: bool,
}

impl Registry {
    pub fn parse(text: &str) -> Result<Registry, String> {
        let raw: Raw = toml::from_str(text).map_err(|e| e.to_string())?;
        Ok(Registry { codes: raw.codes, commands: raw.command, environments: raw.environment, errors: raw.policy.default.trim() == "error" })
    }

    /// TeXora's registry, with the author's from `dir` added (a same-named entry replaces TeXora's).
    /// A file that cannot be read is reported, and TeXora's registry is used alone.
    pub fn load(dir: &Path) -> (Registry, Option<String>) {
        let mut reg = Registry::parse(REGISTRY).expect("TeXora's registry is valid");
        let Ok(text) = std::fs::read_to_string(dir.join(REGISTRY_FILE)) else { return (reg, None) };
        match Registry::parse(&text) {
            Ok(mine) => {
                reg.codes.extend(mine.codes);
                reg.commands.extend(mine.commands);
                reg.environments.extend(mine.environments);
                reg.errors |= mine.errors;
                (reg, None)
            }
            Err(e) => (reg, Some(format!("{REGISTRY_FILE} could not be read: {}", e.lines().next().unwrap_or("").trim()))),
        }
    }

    fn wcag(&self, code: &str) -> String {
        self.codes.get(code).map(|c| c.wcag.clone()).unwrap_or_default()
    }
}

/// One `requires` group: "code = key | key".
#[derive(Clone, Debug, PartialEq)]
pub struct Requirement {
    pub code: String,
    pub keys: Vec<String>,
}

pub fn requirement(s: &str) -> Requirement {
    let (code, list) = match s.split_once('=') {
        Some((c, l)) => (Some(c.trim().to_string()), l),
        None => (None, s),
    };
    let keys: Vec<String> = list.split('|').map(|k| k.trim().to_string()).filter(|k| !k.is_empty()).collect();
    Requirement { code: code.filter(|c| !c.is_empty()).unwrap_or_else(|| keys.first().cloned().unwrap_or_default()), keys }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

impl Severity {
    pub fn word(self) -> &'static str {
        match self {
            Severity::Info => "note",
            Severity::Warning => "warning",
            Severity::Error => "error",
        }
    }
}

/// Something to fix (or to know) for assistive technology.
#[derive(Clone, Debug, PartialEq)]
pub struct Finding {
    pub file: PathBuf,
    pub line: usize,
    /// The command or environment, or "(document)".
    pub subject: String,
    pub code: String,
    /// What is wrong.
    pub message: String,
    pub wcag: String,
    pub severity: Severity,
    /// Where on the line, counted in characters from 1 (0: the whole line), and the text found there
    /// that the finding is about (`\includegraphics`, `urlcolor=linkblue`), to point at and select.
    pub col: usize,
    pub target: String,
    /// How to fix it.
    pub fix: String,
}

/// What to do about a finding of this code, when it has no fix of its own.
fn fix_for(code: &str, subject: &str) -> String {
    match code {
        "alt" if subject == "includevideo" => "add alt={Play video: what it shows, its length} to the second options".into(),
        "alt" => "add alt={what it shows} to its options, or artifact if it is decoration only; wrap drawings and animations in \\AccFigure{description}{...}".into(),
        "transcript" => "add caption={a short line}, transcript={the full text} or transcriptfile=<file> to the second options".into(),
        "motion" => "drop autoplay, or add controls: movement that starts by itself and lasts over 5 seconds needs a way to pause it".into(),
        "link" => "write what the link leads to, for example \\href{...}{the full results table}".into(),
        "title" => "add {Title} after \\begin{frame}, or \\frametitle{Title} inside the frame".into(),
        "table" => "if the first row is a header: put \\midrule (or \\hline) under it and \\AccHeaderRows{1} before the table".into(),
        "placeholder" => "open the presentation in TeXora once, or keep its texora-video and texora-images folders with it".into(),
        _ => String::new(),
    }
}

/// Where `needle` first appears in the document's files: file, line, column (characters, from 1).
/// Text in comments is skipped, unless the needle itself starts a comment (`!TEX program`).
fn locate(docs: &[(PathBuf, String)], needle: &str) -> Option<(PathBuf, usize, usize)> {
    if needle.is_empty() {
        return None;
    }
    for (file, src) in docs {
        for (n, line) in src.lines().enumerate() {
            let Some(at) = line.find(needle) else { continue };
            let comment = (0..at).any(|i| line.as_bytes()[i] == b'%' && (i == 0 || line.as_bytes()[i - 1] != b'\\'));
            if comment && !needle.starts_with('!') {
                continue;
            }
            return Some((file.clone(), n + 1, line[..at].chars().count() + 1));
        }
    }
    None
}

impl Finding {
    /// One line, the same for every engine and class: `talk.tex:42:3  includegraphics  [alt] ...  WCAG 1.1.1`
    /// (file, line and column, as compilers write them, so editors can jump there).
    pub fn line_text(&self, dir: &Path) -> String {
        let file = self.file.strip_prefix(dir).unwrap_or(&self.file).to_string_lossy().to_string();
        let wcag = if self.wcag.is_empty() { String::new() } else { format!("  WCAG {}", self.wcag) };
        let at = if self.col > 0 { format!("{}:{}", self.line, self.col) } else { self.line.to_string() };
        format!("{file}:{at}  {}  [{}] {}{wcag}", self.subject, self.code, self.message)
    }
}

/// Splits `a=1, b={x, y}, c` at its top-level commas into keys (the text before `=`).
pub fn keys_of(list: &str) -> Vec<String> {
    let mut out = vec![];
    let (mut depth, mut start) = (0i32, 0usize);
    let b = list.as_bytes();
    for i in 0..=b.len() {
        let c = b.get(i).copied();
        match c {
            Some(b'{') => depth += 1,
            Some(b'}') => depth -= 1,
            Some(b',') | None if depth <= 0 => {
                let part = list[start..i].trim();
                let key = part.split('=').next().unwrap_or("").trim();
                if !key.is_empty() {
                    out.push(key.to_string());
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    out
}

/// The arguments right after a command or `\begin{env}`: optional `[..]` and `<..>` groups and up to
/// `mandatory` `{..}` groups, with only spaces between them. Returns (optional, mandatory, end).
fn args_after(clean: &str, mut i: usize, mandatory: usize) -> (Vec<String>, Vec<String>, usize) {
    let b = clean.as_bytes();
    let (mut opts, mut mand) = (vec![], vec![]);
    loop {
        let mut j = i;
        while j < b.len() && (b[j] == b' ' || b[j] == b'\t' || b[j] == b'\n') {
            j += 1;
        }
        match b.get(j) {
            Some(b'[') if mand.is_empty() => match balanced(clean, j) {
                Some((inner, end)) => {
                    opts.push(inner.to_string());
                    i = end;
                }
                None => break,
            },
            Some(b'<') if mand.is_empty() => match clean[j..].find('>') {
                Some(e) => i = j + e + 1,
                None => break,
            },
            Some(b'{') if mand.len() < mandatory => match balanced(clean, j) {
                Some((inner, end)) => {
                    mand.push(inner.to_string());
                    i = end;
                }
                None => break,
            },
            _ => break,
        }
    }
    (opts, mand, i)
}

/// Every place `\name` is used (not `\namefoo`), as byte offsets.
fn command_uses(clean: &str, name: &str) -> Vec<usize> {
    let pat = format!("\\{name}");
    clean
        .match_indices(&pat)
        .filter(|(k, _)| !clean[k + pat.len()..].starts_with(|c: char| c.is_ascii_alphabetic() || c == '@'))
        .filter(|(k, _)| *k == 0 || clean.as_bytes()[k - 1] != b'\\')
        .map(|(k, _)| k)
        .collect()
}

/// The bodies of `\AccFigure{alt}{...}`, as byte ranges: what lies inside one is described.
/// Where `\AccFigure{description}{...}` and `\AccArtifact{...}` (decoration, nothing to describe)
/// wrap content.
fn figure_spans(clean: &str) -> Vec<(usize, usize)> {
    let mut out: Vec<(usize, usize)> = command_uses(clean, "AccFigure")
        .into_iter()
        .filter_map(|k| {
            let (_, mand, end) = args_after(clean, k + "\\AccFigure".len(), 2);
            (mand.len() == 2).then_some((k, end))
        })
        .collect();
    out.extend(command_uses(clean, "AccArtifact").into_iter().filter_map(|k| {
        let (_, mand, end) = args_after(clean, k + "\\AccArtifact".len(), 1);
        (mand.len() == 1).then_some((k, end))
    }));
    out
}

const VAGUE_LINKS: [&str; 9] = ["here", "click here", "link", "this link", "more", "read more", "this", "click", "website"];

/// The document's files: the main one and those it `\input`s or `\include`s inside its folder.
fn files(root: &Path) -> Vec<(PathBuf, String)> {
    let dir = root.parent().unwrap_or(Path::new("."));
    let mut out = vec![];
    let mut todo = vec![root.to_path_buf()];
    while let Some(f) = todo.pop() {
        if out.iter().any(|(p, _): &(PathBuf, String)| p == &f) || out.len() > 200 {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(&f) else { continue };
        let clean = uncomment(&src);
        for cmd in ["\\input{", "\\include{", "\\subfile{"] {
            for (k, _) in clean.match_indices(cmd) {
                if let Some((name, _)) = balanced(&clean, k + cmd.len() - 1) {
                    let name = name.trim();
                    if let Ok(p) = resolve(dir, name).or_else(|_| resolve(dir, &format!("{name}.tex"))) {
                        todo.push(p);
                    }
                }
            }
        }
        out.push((f, src));
    }
    out
}

/// Checks a document's source against the registry. Works for any engine and class, with no LaTeX.
pub fn check(root: &Path) -> Vec<Finding> {
    let dir = root.parent().unwrap_or(Path::new(".")).to_path_buf();
    let (reg, reg_problem) = Registry::load(&dir);
    let mut out = vec![];
    let sev = if reg.errors { Severity::Error } else { Severity::Warning };
    if let Some(p) = reg_problem {
        out.push(Finding { file: dir.join(REGISTRY_FILE), line: 1, subject: "(registry)".into(), code: "registry".into(), message: p, wcag: String::new(), severity: Severity::Warning, col: 0, target: String::new(), fix: "correct the file (it is TOML); until then TeXora's own registry is used".into() });
    }
    let docs = files(root);
    let all: String = docs.iter().map(|(_, s)| uncomment(s)).collect::<Vec<_>>().join("\n");
    let main_src = docs.first().map(|(_, s)| s.clone()).unwrap_or_default();
    let frame_envs = crate::notes::frame_envs(&main_src, Some(&dir));
    for (file, src) in &docs {
        let clean = uncomment(src);
        let line_at = |k: usize| clean[..k].matches('\n').count() + 1;
        let figures = figure_spans(&clean);
        let mut add = |k: usize, subject: &str, code: &str, message: String, severity: Severity| {
            // What to point at: the command or `\begin{env}` at k; for vague link text, the text itself.
            let line_start = clean[..k].rfind('\n').map_or(0, |n| n + 1);
            let line_end = clean[k..].find('\n').map_or(clean.len(), |n| k + n);
            let mut at = k;
            let mut target = if clean[k..].starts_with("\\begin{") {
                clean[k..].split_once('}').map(|(a, _)| format!("{a}}}")).unwrap_or_default()
            } else {
                format!("\\{subject}")
            };
            if code == "link" {
                if let Some(text) = message.split('"').nth(1).filter(|t| !t.is_empty()) {
                    let braced = format!("{{{text}}}");
                    if let Some(p) = clean[k..line_end].find(&braced) {
                        at = k + p + 1;
                        target = text.to_string();
                    }
                }
            }
            let col = clean[line_start..at].chars().count() + 1;
            let (message, fix) = match message.split_once(" -- ") {
                Some((m, f)) => (m.to_string(), f.to_string()),
                None => (message, fix_for(code, subject)),
            };
            out.push(Finding { file: file.clone(), line: line_at(k), subject: subject.to_string(), code: code.to_string(), message, wcag: reg.wcag(code), severity, col, target, fix });
        };
        for (name, entry) in &reg.commands {
            for k in command_uses(&clean, name) {
                let (opts, mand, _) = args_after(&clean, k + name.len() + 1, 2);
                let keys: Vec<String> = opts.iter().flat_map(|o| keys_of(o)).collect();
                let inside_figure = figures.iter().any(|&(a, b)| k > a && k < b);
                for req in entry.requires.iter().map(|r| requirement(r)) {
                    if !inside_figure && !req.keys.iter().any(|want| keys.iter().any(|k| k == want)) {
                        let first = req.keys.first().cloned().unwrap_or_default();
                        let others = if req.keys.len() > 1 { format!(" (or {})", req.keys[1..].join(", ")) } else { String::new() };
                        add(k, name, &req.code, format!("missing {first}={{...}}{others}"), sev);
                    }
                }
                run_checks(&entry.checks, &keys, &mand, inside_figure, &mut |code: &str, msg: String| add(k, name, code, msg, sev));
            }
        }
        for (name, entry) in &reg.environments {
            let names: Vec<String> = if name == "frame" { frame_envs.clone() } else { vec![name.clone()] };
            for env in names {
                let begin = format!("\\begin{{{env}}}");
                for (k, _) in clean.match_indices(&begin) {
                    let (opts, mand, after) = args_after(&clean, k + begin.len(), 1);
                    let end = clean[after..].find(&format!("\\end{{{env}}}")).map(|e| after + e).unwrap_or(clean.len());
                    let body = &clean[after..end];
                    let keys: Vec<String> = opts.iter().flat_map(|o| keys_of(o)).collect();
                    let inside_figure = figures.iter().any(|&(a, b)| k > a && k < b);
                    for check in &entry.checks {
                        match check.as_str() {
                            "title" => {
                                let titled = mand.first().is_some_and(|t| !t.trim().is_empty()) || body.contains("\\frametitle") || body.contains("\\titlepage") || body.contains("\\maketitle") || body.contains("\\inserttitle");
                                if !titled {
                                    add(k, &env, "title", "this slide has no title -- add {Title} after \\begin{frame}, or \\frametitle{Title} inside the frame".into(), sev);
                                }
                            }
                            "header-row" => {
                                let first_row_end = body.find("\\\\");
                                let ruled = body.contains("\\toprule") || body.contains("\\midrule") || first_row_end.is_some_and(|e| body[e + 2..].trim_start().starts_with("\\hline"));
                                // \AccHeaderRows{n} (or LaTeX's table/header-rows) before it marks the headers in the tags.
                                let tagged = clean[..k].contains("\\AccHeaderRows") || clean[..k].contains("header-rows");
                                if !ruled && !tagged {
                                    add(k, &env, "table", "no header row is marked -- if the first row is a header: put \\midrule (or \\hline) under it, and \\AccHeaderRows{1} before the table".into(), Severity::Info);
                                } else if !tagged {
                                    add(k, &env, "table", "the header row is shown by a rule, but not marked in the tags -- add \\AccHeaderRows{1} before the table: its first row then becomes header cells (TH)".into(), Severity::Info);
                                }
                            }
                            other => run_checks(&[other.to_string()], &keys, &mand, inside_figure, &mut |code: &str, msg: String| add(k, &env, code, msg, sev)),
                        }
                    }
                }
            }
        }
        // \frame{...}: the command form of a frame.
        if reg.environments.contains_key("frame") {
            for k in command_uses(&clean, "frame") {
                let (_, mand, _) = args_after(&clean, k + 6, 1);
                if let Some(body) = mand.first() {
                    if !(body.contains("\\frametitle") || body.contains("\\titlepage")) {
                        add(k, "frame", "title", "this slide has no title -- add \\frametitle{Title} at the start of the frame".into(), sev);
                    }
                }
            }
        }
    }
    // The document as a whole.
    let main = root.to_path_buf();
    // A finding about the document as a whole points at the line it is about (`anchor`, the first
    // place that text appears), or at line 1 when there is none.
    let doc = |code: &str, message: String, fix: String, severity: Severity, reg: &Registry, anchor: &str| {
        let (file, line, col) = locate(&docs, anchor).unwrap_or((main.clone(), 1, 0));
        let target = if col > 0 { anchor.to_string() } else { String::new() };
        Finding { file, line, subject: "(document)".into(), code: code.into(), message, wcag: reg.wcag(code), severity, col, target, fix }
    };
    let has_meta = all.contains("\\DocumentMetadata");
    let meta_lang = all.find("\\DocumentMetadata").and_then(|k| balanced(&all, k + "\\DocumentMetadata".len())).is_some_and(|(m, _)| keys_of(m).iter().any(|k| k == "lang"));
    let babel = all.contains("{babel}") || all.contains("{polyglossia}") || all.contains("pdflang");
    if !meta_lang && !babel {
        out.push(doc("language", "no document language is set, so a screen reader may read it with the wrong voice".into(),
            "add \\usepackage[english]{babel} (or your language) after \\documentclass, and pdflang=en-US in \\hypersetup; or lang=en-US in \\DocumentMetadata. Without it the tagged copy uses en-US".into(), sev, &reg, "\\documentclass"));
    }
    if !all.contains("\\title") {
        out.push(doc("doctitle", "the document has no \\title, so the PDF window shows the file name".into(), "add \\title{...} before \\begin{document} (and pdftitle={...} in \\hypersetup)".into(), sev, &reg, "\\begin{document}"));
    }
    let beamer = beamer_based(root.parent().unwrap_or(Path::new(".")), &main_src);
    if beamer {
        let fix = if ltx_talk() { "Build tagged copy makes a tagged copy through the ltx-talk class, without Beamer's theme" } else { "install the ltx-talk class (TeX Live 2026); then Build tagged copy makes a tagged copy, using the alt text and transcripts" };
        out.push(doc("tagging", "Beamer slides are not tagged themselves: Beamer refuses LaTeX's tagging (\\DocumentMetadata)".into(), fix.into(), Severity::Info, &reg, "\\documentclass"));
    } else if !has_meta {
        out.push(doc("tagging", "not tagged: there is no \\DocumentMetadata".into(), "Build tagged copy makes a tagged copy without changing this file; or put \\DocumentMetadata{lang=en-US,pdfstandard=ua-2,tagging=on} before \\documentclass".into(), Severity::Info, &reg, "\\documentclass"));
    }
    if main_src.lines().take(5).any(|l| l.to_lowercase().contains("!tex program") && l.to_lowercase().contains("xelatex")) {
        out.push(doc("tagging", "XeLaTeX makes incomplete tags".into(), "compile with LuaLaTeX for a tagged PDF; the tagged copy uses LuaLaTeX by itself".into(), Severity::Info, &reg, "!TEX program"));
    }
    for c in colour_findings(&uncomment(&all)) {
        let mut fix = c.fix;
        // Where the colour itself is defined, when that is another line.
        if let Some(name) = &c.colour {
            if let Some((_, line, _)) = locate(&docs, &format!("\\definecolor{{{name}}}")) {
                fix = format!("{fix} (\\definecolor{{{name}}} is on line {line})");
            }
        }
        out.push(doc(c.code, c.message, fix, if c.info { Severity::Info } else { sev }, &reg, &c.anchor));
    }
    out.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    out
}

/// xcolor's base colours.
fn base_colour(name: &str) -> Option<[f64; 3]> {
    Some(match name {
        "black" => [0.0, 0.0, 0.0],
        "white" => [1.0, 1.0, 1.0],
        "red" => [1.0, 0.0, 0.0],
        "green" => [0.0, 1.0, 0.0],
        "blue" => [0.0, 0.0, 1.0],
        "cyan" => [0.0, 1.0, 1.0],
        "magenta" => [1.0, 0.0, 1.0],
        "yellow" => [1.0, 1.0, 0.0],
        "gray" => [0.5, 0.5, 0.5],
        "darkgray" => [0.25, 0.25, 0.25],
        "lightgray" => [0.75, 0.75, 0.75],
        "brown" => [0.75, 0.5, 0.25],
        "lime" => [0.75, 1.0, 0.0],
        "olive" => [0.5, 0.5, 0.0],
        "orange" => [1.0, 0.5, 0.0],
        "pink" => [1.0, 0.75, 0.75],
        "purple" => [0.75, 0.0, 0.25],
        "teal" => [0.0, 0.5, 0.5],
        "violet" => [0.5, 0.0, 0.5],
        _ => return None,
    })
}

/// A colour expression as xcolor reads it: a name, or a mix `a!40!b!20` (with white where the last
/// colour is left out).
fn colour_value(expr: &str, defined: &BTreeMap<String, [f64; 3]>) -> Option<[f64; 3]> {
    let get = |n: &str| defined.get(n.trim()).copied().or_else(|| base_colour(n.trim()));
    let mut parts = expr.split('!');
    let mut c = get(parts.next()?)?;
    loop {
        let Some(p) = parts.next() else { return Some(c) };
        let p: f64 = p.trim().parse::<f64>().ok()? / 100.0;
        let other = match parts.next() {
            Some(n) if !n.trim().is_empty() => get(n)?,
            _ => [1.0, 1.0, 1.0],
        };
        c = [0, 1, 2].map(|i| p * c[i] + (1.0 - p) * other[i]);
    }
}

/// WCAG's contrast ratio of two sRGB colours, 1 to 21.
pub fn contrast(a: [f64; 3], b: [f64; 3]) -> f64 {
    let lum = |c: [f64; 3]| {
        let l = c.map(|v| if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) });
        0.2126 * l[0] + 0.7152 * l[1] + 0.0722 * l[2]
    };
    let (x, y) = (lum(a), lum(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

/// A colour finding, with where it is (`anchor`, text to look for in the source) and the colour it is
/// about, when that is one of the document's own (`\definecolor{name}`).
#[derive(Debug)]
struct ColourFinding {
    code: &'static str,
    message: String,
    fix: String,
    info: bool,
    anchor: String,
    colour: Option<String>,
}

/// The nearest colour to `c`, darker or lighter, that passes `ok`: `RGB r,g,b`, or None if neither
/// way gets there.
fn suggest_colour(c: [f64; 3], ok: &dyn Fn([f64; 3]) -> bool) -> Option<[u32; 3]> {
    for step in 1..=100 {
        let t = step as f64 / 100.0;
        for towards in [0.0, 1.0] {
            let m = c.map(|v| v + (towards - v) * t);
            if ok(m) {
                return Some(m.map(|v| (v * 255.0).round() as u32));
            }
        }
    }
    None
}

/// The colours the source sets, checked against WCAG 2.2: text contrast (1.4.3; 4.5:1, titles 3:1)
/// for each Beamer colour that gives both fg and bg, and for normal text on the slide background;
/// coloured links (1.4.1: links set apart by colour alone need 3:1 with the text around them and
/// 4.5:1 with the background). Only what the source says is seen: a theme's own colours are not.
fn colour_findings(src: &str) -> Vec<ColourFinding> {
    let mut defined: BTreeMap<String, [f64; 3]> = BTreeMap::new();
    for (_, e) in uses(src, "definecolor") {
        let (_, mand, _) = args_after(src, e, 3);
        let [name, model, spec] = [0, 1, 2].map(|i| mand.get(i).map(|s| s.trim().to_string()).unwrap_or_default());
        let nums: Vec<f64> = spec.split(',').filter_map(|v| v.trim().parse().ok()).collect();
        let c = match model.as_str() {
            "RGB" if nums.len() == 3 => Some([nums[0] / 255.0, nums[1] / 255.0, nums[2] / 255.0]),
            "rgb" if nums.len() == 3 => Some([nums[0], nums[1], nums[2]]),
            "gray" if nums.len() == 1 => Some([nums[0]; 3]),
            "HTML" if spec.len() == 6 => u32::from_str_radix(&spec, 16).ok().map(|v| [(v >> 16) & 255, (v >> 8) & 255, v & 255].map(|x| x as f64 / 255.0)),
            _ => None,
        };
        if let Some(c) = c {
            defined.insert(name, c);
        }
    }
    // \setbeamercolor{element}{fg=..,bg=..}, in the preamble: one set inside a frame or a group in
    // the document (a dark title slide) is local to it.
    let pre = src.split("\\begin{document}").next().unwrap_or(src);
    // An element with its text (fg) and background (bg) colours, when set.
    type ElementColours = (String, Option<[f64; 3]>, Option<[f64; 3]>);
    let mut beamer: Vec<ElementColours> = vec![];
    let mut fg_names: BTreeMap<String, String> = BTreeMap::new();
    for (_, e) in uses(pre, "setbeamercolor") {
        let e = if pre[e..].starts_with('*') { e + 1 } else { e };
        let (_, mand, _) = args_after(pre, e, 2);
        let (Some(elem), Some(spec)) = (mand.first(), mand.get(1)) else { continue };
        let mut fg = None;
        let mut bg = None;
        for item in items(spec) {
            if let Some((k, v)) = item.split_once('=') {
                match k.trim() {
                    "fg" => {
                        fg = colour_value(v, &defined);
                        fg_names.insert(elem.trim().to_string(), v.trim().to_string());
                    }
                    "bg" => bg = colour_value(v, &defined),
                    _ => {}
                }
            }
        }
        beamer.push((elem.trim().to_string(), fg, bg));
    }
    let own = |name: Option<&String>| name.filter(|n| defined.contains_key(n.as_str())).cloned();
    let find = |name: &str, fg: bool| beamer.iter().rev().find(|(n, f, b)| n == name && if fg { f.is_some() } else { b.is_some() }).and_then(|(_, f, b)| if fg { *f } else { *b });
    let canvas = find("background canvas", false).or_else(|| find("normal text", false)).unwrap_or([1.0; 3]);
    let text = find("normal text", true).unwrap_or([0.0; 3]);
    let mut out = vec![];
    let fmt = |r: f64| format!("{:.1}:1", (r * 10.0).floor() / 10.0);
    let mut seen = std::collections::HashSet::new();
    for (elem, fg, bg) in &beamer {
        let (Some(fg), Some(bg)) = (fg, bg) else { continue };
        if !seen.insert(elem.clone()) {
            continue;
        }
        let large = elem.contains("title") || elem.contains("part name") || elem.contains("section name");
        let need = if large { 3.0 } else { 4.5 };
        let r = contrast(*fg, *bg);
        if r < need {
            let bgc = *bg;
            let better = suggest_colour(*fg, &|c| contrast(c, bgc) >= need);
            out.push(ColourFinding {
                code: "contrast",
                message: format!("the text of \"{elem}\" has a contrast of {} with its background; {} is needed{}", fmt(r), fmt(need), if large { " for large text" } else { "" }),
                fix: match better {
                    Some([r, g, b]) => match own(fg_names.get(elem)) {
                        Some(n) => format!("make its fg darker or lighter: \\definecolor{{{n}}}{{RGB}}{{{r},{g},{b}}} meets it; or change its bg"),
                        None => format!("make its fg darker or lighter, for example {{RGB}}{{{r},{g},{b}}}, or change its bg"),
                    },
                    None => "change its bg: no shade of its fg reaches the contrast on it".into(),
                },
                info: false,
                anchor: format!("\\setbeamercolor{{{elem}}}"),
                colour: own(fg_names.get(elem)),
            });
        }
    }
    if find("normal text", true).is_some() || find("background canvas", false).is_some() {
        let r = contrast(text, canvas);
        if r < 4.5 {
            let better = suggest_colour(text, &|c| contrast(c, canvas) >= 4.5);
            let anchor = if find("normal text", true).is_some() { "\\setbeamercolor{normal text}" } else { "\\setbeamercolor{background canvas}" };
            out.push(ColourFinding {
                code: "contrast",
                message: format!("normal text has a contrast of {} with the slide background; 4.5:1 is needed", fmt(r)),
                fix: match better {
                    Some([r, g, b]) => format!("make the text colour darker or lighter, for example {{RGB}}{{{r},{g},{b}}}, or change the background"),
                    None => "change the background colour".into(),
                },
                info: false,
                anchor: anchor.into(),
                colour: own(fg_names.get("normal text")),
            });
        }
    }
    // Links.
    let setup: String = uses(src, "hypersetup").into_iter().filter_map(|(_, e)| args_after(src, e, 1).1.first().cloned()).collect::<Vec<_>>().join(",");
    let opts: String = uses(src, "usepackage").into_iter().filter_map(|(_, e)| {
        let (o, m, _) = args_after(src, e, 1);
        m.first().is_some_and(|p| p.split(',').any(|x| x.trim() == "hyperref")).then(|| o.join(","))
    }).collect::<Vec<_>>().join(",");
    let keys: Vec<(String, String)> = items(&format!("{opts},{setup}")).iter().map(|i| {
        let (k, v) = i.split_once('=').unwrap_or((i, "true"));
        (k.trim().to_string(), v.trim().to_string())
    }).filter(|(k, _)| !k.is_empty()).collect();
    let key = |k: &str| keys.iter().rev().find(|(n, _)| n == k).map(|(_, v)| v.clone());
    let hidden = key("hidelinks").is_some();
    let coloured = !hidden && key("colorlinks").is_some_and(|v| v != "false");
    let any = |cmds: &[&str]| cmds.iter().any(|c| !uses(src, c).is_empty());
    let has_links = any(&["href", "url", "ref", "autoref", "hyperlink", "cite", "tableofcontents"]);
    if coloured && has_links {
        // Each colour is for its own kind of link: web links, links inside the document, citations.
        let mut checked: Vec<String> = vec![];
        for (k, default, used) in [
            ("urlcolor", "magenta", any(&["href", "url"])),
            ("linkcolor", "red", any(&["ref", "autoref", "hyperlink", "tableofcontents", "footnote"])),
            ("citecolor", "green", any(&["cite"])),
        ] {
            let name = key(k).unwrap_or_else(|| default.to_string());
            if !used || checked.contains(&name) {
                continue;
            }
            checked.push(name.clone());
            let Some(c) = colour_value(&name, &defined) else { continue };
            let (with_text, with_bg) = (contrast(c, text), contrast(c, canvas));
            if with_text < 3.0 || with_bg < 4.5 {
                let mut problems = vec![];
                if with_text < 3.0 {
                    problems.push(format!("{} with the text around them (3:1 is needed)", fmt(with_text)));
                }
                if with_bg < 4.5 {
                    problems.push(format!("{} with the background (4.5:1 is needed)", fmt(with_bg)));
                }
                let better = suggest_colour(c, &|x| contrast(x, text) >= 3.0 && contrast(x, canvas) >= 4.5);
                let set = key(k).is_some();
                out.push(ColourFinding {
                    code: "linkcolour",
                    message: format!("{k}={name}: links are told apart from the text by colour alone, and the colour has a contrast of {}", problems.join(" and ")),
                    fix: match better {
                        Some([r, g, b]) if own(Some(&name)).is_some() => format!("redefine the colour: \\definecolor{{{name}}}{{RGB}}{{{r},{g},{b}}} meets both; or underline the links"),
                        Some([r, g, b]) if set => format!("choose a link colour that meets both, for example {{RGB}}{{{r},{g},{b}}}, or underline the links"),
                        Some([r, g, b]) => format!("{k} is not set, so hyperref's {name} is used: set {k} to a colour that meets both, for example \\definecolor{{linkcolour}}{{RGB}}{{{r},{g},{b}}} and {k}=linkcolour; or underline the links"),
                        None => "no shade of this colour meets both: choose another colour, or underline the links".into(),
                    },
                    info: false,
                    anchor: if set { format!("{k}={name}") } else { "colorlinks".into() },
                    colour: own(Some(&name)),
                });
            }
        }
    }
    if hidden && has_links && !uses(src, "href").is_empty() {
        out.push(ColourFinding {
            code: "linkcolour",
            message: "hidelinks: links in the text look like the text around them".into(),
            fix: "fine for posters and buttons; for a link in a sentence give another cue (WCAG 1.4.1): colorlinks with a colour of 3:1 against the text, or an underline".into(),
            info: true,
            anchor: "hidelinks".into(),
            colour: None,
        });
    }
    out
}

/// The named checks of one use.
fn run_checks(checks: &[String], keys: &[String], mand: &[String], inside_figure: bool, add: &mut dyn FnMut(&str, String)) {
    let has = |k: &str| keys.iter().any(|x| x == k);
    for check in checks {
        match check.as_str() {
            "motion" if has("autoplay") && has("loop") && !has("controls") => {
                add("motion", "starts by itself (autoplay) and loops, with no way to pause it -- drop autoplay, or add controls: movement over 5 seconds needs a way to pause it (in TeXora a click pauses a video)".into())
            }
            "descriptive-text" => {
                let url = mand.first().map(|s| s.trim().to_string()).unwrap_or_default();
                let text = mand.get(1).map(|s| s.trim().to_string()).unwrap_or_default();
                let plain = text.to_lowercase();
                let vague = plain.is_empty() || VAGUE_LINKS.contains(&plain.as_str()) || plain == url.to_lowercase() || plain.starts_with("http") || plain.starts_with("www.");
                if vague && mand.len() == 2 {
                    add("link", format!("link text \"{text}\" does not say where it goes -- write what it leads to, for example \\href{{{url}}}{{the full results table}}"));
                }
            }
            "figure-wrapper" if !inside_figure => add("alt", "has no alt key of its own, so a screen reader gets no description -- wrap it in \\AccFigure{what it shows}{...}".into()),
            _ => {}
        }
    }
}

fn class_of(src: &str) -> Option<String> {
    let clean = uncomment(src);
    clean.find("\\documentclass").and_then(|k| {
        let (_, mand, _) = args_after(&clean, k + "\\documentclass".len(), 1);
        mand.into_iter().next()
    }).map(|c| c.trim().to_string())
}

/// Beamer, or a custom class beside the presentation that is built on it (`\LoadClass{beamer}`).
pub fn beamer_based(dir: &Path, src: &str) -> bool {
    let Some(class) = class_of(src) else { return false };
    class == "beamer"
        || std::fs::read_to_string(dir.join(format!("{class}.cls"))).is_ok_and(|c| {
            let c = uncomment(&c);
            ["\\LoadClass", "\\LoadClassWithOptions"].iter().any(|cmd| uses(&c, &cmd[1..]).iter().any(|&(_, e)| {
                let (_, mand, _) = args_after(&c, e, 1);
                mand.first().is_some_and(|m| m.trim() == "beamer")
            }))
        })
}

pub(crate) fn ltx_talk() -> bool {
    ltx_talk_file().is_some()
}

fn ltx_talk_file() -> Option<PathBuf> {
    let kpse = crate::latex::find_tool("kpsewhich")?;
    let o = std::process::Command::new(kpse).arg("ltx-talk.cls").output().ok()?;
    let p = String::from_utf8_lossy(&o.stdout).trim().to_string();
    (o.status.success() && !p.is_empty()).then(|| PathBuf::from(p))
}

/// The installed ltx-talk's version, from `\ProvidesExplClass {ltx-talk} {2026-11-12} {0.6.12}`.
pub(crate) fn ltx_talk_version() -> Option<String> {
    let text = std::fs::read_to_string(ltx_talk_file()?).ok()?;
    let line = text.lines().find(|l| l.contains("\\ProvidesExplClass") && l.contains("ltx-talk"))?;
    let parts: Vec<&str> = line.split('{').map(|p| p.trim().trim_end_matches('}').trim()).collect();
    parts.get(3).filter(|v| !v.is_empty()).map(|v| v.to_string())
}

/// The language for the accessible build: `\DocumentMetadata{lang=..}`, else babel's or polyglossia's
/// main language, else en-US.
pub fn language(src: &str) -> String {
    let clean = uncomment(src);
    let map = [("english", "en"), ("american", "en-US"), ("british", "en-GB"), ("UKenglish", "en-GB"), ("USenglish", "en-US"), ("french", "fr"), ("ngerman", "de"), ("german", "de"), ("spanish", "es"), ("italian", "it"), ("portuguese", "pt"), ("brazilian", "pt-BR"), ("dutch", "nl"), ("arabic", "ar"), ("hebrew", "he"), ("russian", "ru"), ("polish", "pl"), ("turkish", "tr"), ("greek", "el"), ("japanese", "ja"), ("chinese", "zh"), ("korean", "ko"), ("swedish", "sv"), ("danish", "da"), ("norsk", "nb"), ("finnish", "fi"), ("czech", "cs")];
    for (k, _) in clean.match_indices("\\usepackage[") {
        if let Some((opts, end)) = balanced(&clean, k + "\\usepackage".len()) {
            if clean[end..].trim_start().starts_with("{babel}") {
                // babel's main language is the last one given.
                if let Some(last) = opts.split(',').map(str::trim).rfind(|o| !o.contains('=')) {
                    if let Some((_, code)) = map.iter().find(|(n, _)| *n == last) {
                        return code.to_string();
                    }
                }
            }
        }
    }
    for cmd in ["\\setdefaultlanguage", "\\setmainlanguage"] {
        if let Some(k) = clean.find(cmd) {
            let (_, mand, _) = args_after(&clean, k + cmd.len(), 1);
            if let Some((_, code)) = mand.first().and_then(|l| map.iter().find(|(n, _)| *n == l.trim())) {
                return code.to_string();
            }
        }
    }
    "en-US".to_string()
}

/// The wrapper the accessible build compiles: `\DocumentMetadata` first (the right keys for this LaTeX:
/// `tagging=on` from LaTeX 2025-06, the test phase before), then the document, unchanged. A document
/// that already has its own `\DocumentMetadata` is only `\input`.
pub fn wrapper(file_name: &str, lang: &str, has_metadata: bool, gate: &str) -> String {
    let mut s = format!("% Made by TeXora: an accessible (tagged) build of {file_name}. That file and its own PDF\n% are not changed; this one makes {}.\n", file_name.trim_end_matches(".tex").to_string() + ".acc.pdf");
    if !has_metadata {
        s.push_str(&format!("\\IfFormatAtLeastTF{{2025-06-01}}\n  {{\\DocumentMetadata{{lang={lang},pdfstandard=ua-2,tagging=on}}}}\n  {{\\DocumentMetadata{{lang={lang},pdfstandard=ua-1,testphase={{phase-III}}}}}}\n"));
    }
    if !gate.is_empty() {
        s.push_str(&format!("\\PassOptionsToPackage{{{gate}}}{{accessgate}}\n"));
    }
    s.push_str(&format!("\\input{{{file_name}}}\n"));
    s
}

/// Beamer commands `ltx-talk` lacks, defined only where it lacks them (notes, logos and theme colours do
/// nothing; text stays text).
pub const TALK_SHIM: &str = include_str!("../assets/tex/accessgate-talk.tex");

/// Beamer commands that only set the look, with their number of `{}` arguments; the tagged copy leaves
/// them out. (`\appendix` becomes `\AccAppendix`: before ltx-talk 0.6.12 its own broke the tag structure
/// between frames, so the preamble makes it do nothing there.)
const LOOK_ONLY: &[(&str, usize)] = &[
    ("usetheme", 1), ("usecolortheme", 1), ("usefonttheme", 1), ("useinnertheme", 1), ("useoutertheme", 1),
    ("setbeamercolor", 2), ("setbeamerfont", 2), ("setbeamertemplate", 1), ("setbeameroption", 1),
    ("setbeamersize", 1), ("setbeamercovered", 1), ("defbeamertemplate", 3), ("addtobeamertemplate", 3),
    ("transduration", 1),
    ("transblindshorizontal", 0), ("transblindsvertical", 0), ("transboxin", 0), ("transboxout", 0),
    ("transcover", 0), ("transdissolve", 0), ("transfade", 0), ("transglitter", 0), ("transpush", 0),
    ("transreplace", 0), ("transsplithorizontalin", 0), ("transsplithorizontalout", 0),
    ("transsplitverticalin", 0), ("transsplitverticalout", 0), ("transuncover", 0), ("transwipe", 0),
];

/// Whether `pos` is inside a `%` comment.
fn in_comment(s: &str, pos: usize) -> bool {
    let start = s[..pos].rfind('\n').map_or(0, |k| k + 1);
    let b = s.as_bytes();
    (start..pos).any(|i| b[i] == b'%' && (i == 0 || b[i - 1] != b'\\'))
}

/// Where the group opened at `k` (`[`, `{` or `<`) ends.
fn group_end(s: &str, k: usize) -> Option<usize> {
    if s.as_bytes()[k] == b'<' {
        return s[k..].find('>').map(|e| k + e + 1);
    }
    balanced(s, k).map(|(_, e)| e)
}

/// Where a command's arguments end: an optional `*`, `<>` and `[]` groups, `mand` `{}` groups (which
/// may be on the next lines), and further groups on the same line (`\setbeamertemplate{x}[circle]`).
fn command_end(s: &str, mut j: usize, mand: usize) -> usize {
    let b = s.as_bytes();
    if b.get(j) == Some(&b'*') {
        j += 1;
    }
    let mut got = 0;
    loop {
        let mut k = j;
        let mut newline = false;
        while k < b.len() && b[k].is_ascii_whitespace() {
            newline |= b[k] == b'\n';
            k += 1;
        }
        if k >= b.len() || (got >= mand && newline) || !matches!(b[k], b'[' | b'<' | b'{') {
            break;
        }
        let Some(e) = group_end(s, k) else { break };
        if b[k] == b'{' {
            got += 1;
        }
        j = e;
    }
    j
}

/// Each use of `\name` outside comments (and not the start of a longer name): where it starts and where
/// the name ends.
fn uses(s: &str, name: &str) -> Vec<(usize, usize)> {
    let pat = format!("\\{name}");
    s.match_indices(&pat)
        .map(|(k, _)| (k, k + pat.len()))
        .filter(|&(k, e)| !s[e..].starts_with(|c: char| c.is_ascii_alphabetic() || c == '@') && !in_comment(s, k))
        .collect()
}

/// Replaces the spans, given in order, with their texts.
fn splice(s: &str, edits: Vec<(usize, usize, String)>) -> String {
    let mut out = String::with_capacity(s.len());
    let mut at = 0;
    for (a, e, text) in edits {
        if a < at {
            continue;
        }
        out.push_str(&s[at..a]);
        out.push_str(&text);
        at = e;
    }
    out.push_str(&s[at..]);
    out
}

/// Comma-separated items at brace depth 0.
fn items(list: &str) -> Vec<&str> {
    let (mut out, mut depth, mut from) = (vec![], 0i32, 0);
    for (i, c) in list.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            ',' if depth == 0 => {
                out.push(&list[from..i]);
                from = i + 1;
            }
            _ => {}
        }
    }
    out.push(&list[from..]);
    out
}

/// Beamer frame options as ltx-talk ones, and whether the frame is fragile. ltx-talk 0.6 stops at any
/// key it lacks (0.4 ignored lone words), so only these are kept: `label=` (0.6 repeats the frame with
/// \againframe; for older ones the preamble adds a label key that does nothing), `allowframebreaks`
/// as `auto-break`, `t`/`c`/`b` as `vertical-alignment`. `fragile` makes the frame a frame*, which
/// takes verbatim text. The rest only sets the look (plain, shrink, squeeze, standout, ...).
fn frame_opts(o: &str) -> (Vec<String>, bool) {
    let (mut out, mut fragile) = (vec![], false);
    for item in items(o) {
        let item = item.trim();
        match item.split('=').next().unwrap_or("").trim() {
            "label" => out.push(item.to_string()),
            "allowframebreaks" => out.push("auto-break".into()),
            "t" => out.push("vertical-alignment=top".into()),
            "c" => out.push("vertical-alignment=center".into()),
            "b" => out.push("vertical-alignment=bottom".into()),
            "fragile" => fragile = true,
            _ => {}
        }
    }
    (out, fragile)
}

/// Beamer source (a presentation or a class built on Beamer) made fit for `ltx-talk`: the look-only
/// commands and Beamer-only packages go, frame labels go, `\column` becomes the column environment.
/// `lua`: compiled with LuaLaTeX, where ltx-talk's unicode-math replaces amssymb (which clashes with it).
/// `local`: theme files beside the presentation (`beamerthemeAcme`); they may define environments the
/// slides use, so they are loaded as their converted copies (`beamerthemeAcme.acc`) instead.
fn talk_convert(src: &str, lua: bool, local: &[String]) -> String {
    let mut s = src.to_string();
    // The look; a local theme as its converted copy.
    for (name, mand) in LOOK_ONLY {
        let edits = uses(&s, name).into_iter().map(|(k, e)| {
            let end = command_end(&s, e, *mand);
            let theme = name.strip_prefix("use").filter(|t| t.ends_with("theme")).and_then(|t| {
                let (_, m, _) = args_after(&s, e, 1);
                let kind = t.trim_end_matches("theme");
                m.first().map(|x| format!("beamer{kind}theme{}", x.trim()))
            }).filter(|f| local.contains(f));
            (k, end, theme.map(|f| format!("\\usepackage{{{f}.acc}}")).unwrap_or_default())
        }).collect();
        s = splice(&s, edits);
    }
    let edits = uses(&s, "appendix").into_iter().map(|(k, e)| (k, e, "\\AccAppendix".to_string())).collect();
    s = splice(&s, edits);
    // Beamer's switch form, `\mode<presentation>` with no braces after it (ltx-talk's \mode takes them).
    let edits = uses(&s, "mode").into_iter().filter_map(|(k, e)| {
        let mut j = e;
        let b = s.as_bytes();
        if b.get(j) == Some(&b'*') {
            j += 1;
        }
        if b.get(j) == Some(&b'<') {
            j = group_end(&s, j)?;
        }
        (!s[j..].trim_start().starts_with('{')).then(|| (k, j, String::new()))
    }).collect();
    s = splice(&s, edits);
    let edits = s.match_indices("\\beamertemplate")
        .filter(|(k, _)| !in_comment(&s, *k))
        .map(|(k, m)| (k, k + m.len() + s[k + m.len()..].find(|c: char| !c.is_ascii_alphabetic()).unwrap_or(0), String::new()))
        .collect();
    s = splice(&s, edits);
    // Packages for Beamer only, and amssymb/amsfonts under LuaLaTeX.
    let mut edits = vec![];
    for (k, e) in uses(&s, "usepackage") {
        let end = command_end(&s, e, 1);
        let Some(open) = s[e..end].rfind('{').map(|o| e + o) else { continue };
        let Some((list, _)) = balanced(&s, open) else { continue };
        let all = items(list);
        let keep: Vec<String> = all.iter().filter_map(|p| {
            let p = p.trim();
            if local.iter().any(|l| l == p) {
                Some(format!("{p}.acc"))
            } else if (p.starts_with("beamer") && !p.ends_with(".acc")) || p == "appendixnumberbeamer" || (lua && (p == "amssymb" || p == "amsfonts")) {
                None
            } else {
                Some(p.to_string())
            }
        }).collect();
        if keep.iter().map(String::as_str).ne(all.iter().map(|p| p.trim())) {
            let text = if keep.is_empty() { String::new() } else { format!("{}{{{}}}", &s[k..open], keep.join(",")) };
            edits.push((k, end, text));
        }
    }
    s = splice(&s, edits);
    // Frame openings, `\begin{frame}<o>[opts]{title}{subtitle}` as Beamer reads them:
    // - the title becomes \frametitle (ltx-talk's frame has no title argument by default, so it would
    //   be plain text; \frametitle is tagged as a heading);
    // - the options become ones every ltx-talk understands (`frame_opts`);
    // - fragile becomes ltx-talk's frame*, which takes verbatim text, up to the frame's \end{frame}.
    let mut edits = vec![];
    let skip_space = |s: &str, mut j: usize| {
        let b = s.as_bytes();
        let from = j;
        while j < b.len() && b[j].is_ascii_whitespace() {
            j += 1;
        }
        if s[from..j].matches('\n').count() > 1 { from } else { j }
    };
    for (k, _) in s.match_indices("\\begin{frame}").filter(|(k, _)| !in_comment(&s, *k)) {
        let mut j = k + "\\begin{frame}".len();
        let b = s.as_bytes();
        let mut overlay = "";
        let mut opts: Vec<String> = vec![];
        let mut fragile = false;
        let mut t = skip_space(&s, j);
        if b.get(t) == Some(&b'<') {
            let Some(e) = group_end(&s, t) else { continue };
            overlay = &s[t..e];
            j = e;
            t = skip_space(&s, j);
        }
        if b.get(t) == Some(&b'[') {
            let Some((o, e)) = balanced(&s, t) else { continue };
            (opts, fragile) = frame_opts(o);
            j = e;
            t = skip_space(&s, j);
        }
        let mut titles = vec![];
        while titles.len() < 2 && b.get(t) == Some(&b'{') {
            let Some((x, e)) = balanced(&s, t) else { break };
            titles.push(x);
            j = e;
            t = skip_space(&s, j);
        }
        let mut text = format!("\\begin{{frame{}}}{overlay}", if fragile { "*" } else { "" });
        if !opts.is_empty() {
            text.push_str(&format!("[{}]", opts.join(",")));
        }
        if fragile {
            // frame* reads its text from the next line on; the rest of this line would be lost.
            text.push('\n');
        }
        if let Some(title) = titles.first() {
            text.push_str(&format!("\\frametitle{{{title}}}"));
        }
        if let Some(sub) = titles.get(1) {
            text.push_str(&format!("\\framesubtitle{{{sub}}}"));
        }
        edits.push((k, j, text));
        if fragile {
            if let Some(end) = s[j..].find("\\end{frame}").map(|e| j + e) {
                // frame* reads its text up to here; the rest of this line would be lost.
                edits.push((end, end + "\\end{frame}".len(), "\\end{frame*}\n".into()));
            }
        }
    }
    edits.sort_by_key(|e| e.0);
    s = splice(&s, edits);
    // Beamer's command form, `\frame<o>[opts]{body}`: in ltx-talk \frame is LaTeX's box, not a slide.
    let mut edits = vec![];
    for (k, e) in uses(&s, "frame") {
        let b = s.as_bytes();
        let (mut j, mut overlay, mut opts, mut fragile): (usize, &str, Vec<String>, bool) = (e, "", vec![], false);
        if b.get(j) == Some(&b'<') {
            let Some(x) = group_end(&s, j) else { continue };
            overlay = &s[j..x];
            j = x;
        }
        if b.get(j) == Some(&b'[') {
            let Some((o, x)) = balanced(&s, j) else { continue };
            (opts, fragile) = frame_opts(o);
            j = x;
        }
        let t = j + s[j..].len() - s[j..].trim_start().len();
        if b.get(t) != Some(&b'{') {
            continue;
        }
        let Some((body, end)) = balanced(&s, t) else { continue };
        let star = if fragile { "*" } else { "" };
        let opts = if opts.is_empty() { String::new() } else { format!("[{}]", opts.join(",")) };
        let after = if fragile { "\n" } else { "\\relax" };
        edits.push((k, end, format!("\\begin{{frame{star}}}{overlay}{opts}{after}{body}\\end{{frame{star}}}{}", if fragile { "\n" } else { "" })));
    }
    s = splice(&s, edits);
    // `\column{w}` inside columns: ltx-talk has only the column environment.
    let mut edits = vec![];
    for (k, _) in s.match_indices("\\begin{columns}").filter(|(k, _)| !in_comment(&s, *k)) {
        let Some(close) = s[k..].find("\\end{columns}").map(|c| k + c) else { continue };
        let inner = &s[k..close];
        let cols: Vec<(usize, usize)> = uses(inner, "column").into_iter().map(|(a, e)| (k + a, k + e)).collect();
        for (n, (a, e)) in cols.iter().enumerate() {
            let end = command_end(&s, *e, 1);
            let args = &s[*e..end];
            edits.push((*a, end, format!("{}\\begin{{column}}{args}", if n > 0 { "\\end{column}" } else { "" })));
        }
        if !cols.is_empty() {
            edits.push((close, close, "\\end{column}".into()));
        }
    }
    edits.sort_by_key(|e| e.0);
    splice(&s, edits)
}

/// A Beamer presentation as an `ltx-talk` one, for the tagged copy: Beamer refuses `\DocumentMetadata`,
/// and ltx-talk is the LaTeX project's tagging-ready class with the same frames and overlays. The text,
/// frames, overlays, pictures and alt text stay; the theme goes. `class`: `ltx-talk`, or the converted
/// copy of a custom class built on Beamer (`talk_class`).
/// What the tagged copy of a Beamer presentation leaves out, changes, or may not reproduce, in plain
/// words, for `<name>.acc-losses.txt`. The conversion is source to source, so this is a list of what
/// it saw, not a guarantee: compare the copy with the slides.
pub fn talk_losses(file_name: &str, src: &str, class_src: Option<&str>, lua: bool, local: &[String], ltx_talk: Option<&str>, handout: bool) -> String {
    let clean = uncomment(src);
    let both = format!("{clean}\n{}", class_src.map(uncomment).unwrap_or_default());
    let count = |name: &str| uses(&both, name).len();
    let (mut out, mut changed, mut risky) = (vec![], vec![], vec![]);
    let names = |cmd: &str| -> Vec<String> {
        uses(&both, cmd).into_iter().filter_map(|(_, e)| args_after(&both, e, 1).1.first().map(|n| n.trim().to_string())).collect()
    };
    for (cmd, what) in [("usetheme", "theme"), ("usecolortheme", "colour theme"), ("usefonttheme", "font theme"), ("useinnertheme", "inner theme"), ("useoutertheme", "outer theme")] {
        for n in names(cmd) {
            let kind = what.split(' ').next().unwrap_or("");
            let file = format!("beamer{}theme{n}", if kind == "theme" { "" } else { kind });
            if local.contains(&file) {
                changed.push(format!("{what} {n}: its file beside the presentation is used as {file}.acc.sty, without its colours and templates"));
            } else {
                out.push(format!("{what} {n} (the look)"));
            }
        }
    }
    let settings: usize = ["setbeamercolor", "setbeamerfont", "setbeamertemplate", "setbeameroption", "setbeamersize", "setbeamercovered", "defbeamertemplate", "addtobeamertemplate"].iter().map(|c| count(c)).sum::<usize>() + both.matches("\\beamertemplate").count();
    if settings > 0 {
        out.push(format!("{settings} colour, font and template setting{} (the look)", if settings == 1 { "" } else { "s" }));
    }
    let notes = count("note");
    if notes > 0 {
        out.push(format!("{notes} speaker note{} (for the presenter, not the audience)", if notes == 1 { "" } else { "s" }));
    }
    let trans: usize = LOOK_ONLY.iter().filter(|(n, _)| n.starts_with("trans")).map(|(n, _)| count(n)).sum();
    if trans > 0 {
        out.push(format!("{trans} slide transition{}", if trans == 1 { "" } else { "s" }));
    }
    for (cmd, what) in [("logo", "the logo"), ("titlegraphic", "the title picture"), ("AtBeginSection", "the slide shown at each section"), ("AtBeginSubsection", "the slide shown at each subsection")] {
        if count(cmd) > 0 {
            out.push(what.to_string());
        }
    }
    let again = count("againframe");
    if again > 0 {
        let repeats = ltx_talk.is_some_and(|v| v.split('.').take(2).filter_map(|p| p.parse::<u32>().ok()).collect::<Vec<_>>() >= vec![0, 6]);
        if repeats {
            changed.push(format!("{again} \\againframe: repeated by ltx-talk {}", ltx_talk.unwrap_or("")));
        } else {
            out.push(format!("{again} \\againframe: the slide is not repeated (ltx-talk 0.6 or later repeats it)"));
        }
    }
    // Frame options that only set the look.
    let mut dropped: Vec<String> = vec![];
    for (k, _) in clean.match_indices("\\begin{frame}") {
        let rest = &clean[k + "\\begin{frame}".len()..];
        let rest = rest.trim_start();
        let rest = if rest.starts_with('<') { rest.split_once('>').map(|(_, r)| r.trim_start()).unwrap_or(rest) } else { rest };
        if rest.starts_with('[') {
            if let Some((o, _)) = balanced(rest, 0) {
                for item in items(o) {
                    let key = item.split('=').next().unwrap_or("").trim().to_string();
                    if !matches!(key.as_str(), "label" | "allowframebreaks" | "t" | "c" | "b" | "fragile" | "") && !dropped.contains(&key) {
                        dropped.push(key);
                    }
                }
            }
        }
    }
    if !dropped.is_empty() {
        out.push(format!("frame options {} (the look)", dropped.join(", ")));
    }
    let pkgs: Vec<String> = uses(&clean, "usepackage").into_iter().filter_map(|(_, e)| args_after(&clean, e, 1).1.first().cloned()).flat_map(|l| l.split(',').map(|p| p.trim().to_string()).collect::<Vec<_>>()).collect();
    for p in &pkgs {
        if (p.starts_with("beamer") && !local.contains(p)) || p == "appendixnumberbeamer" {
            out.push(format!("the package {p} (for Beamer only)"));
        }
        if lua && (p == "amssymb" || p == "amsfonts") {
            changed.push(format!("{p} is not loaded: with LuaLaTeX ltx-talk's maths fonts have its symbols (\\square and \\blacksquare are mapped)"));
        }
        if matches!(p.as_str(), "minted" | "pgfpages" | "beamerposter" | "beamerarticle" | "pdfpc" | "pdfpcnotes" | "tcolorbox" | "multimedia" | "media9" | "animate") {
            risky.push(format!("{p}: made for Beamer or for interactive PDF; check its slides in the copy"));
        }
    }
    if clean.contains("\\column{") || clean.contains("\\column[") {
        changed.push("\\column{...} became the column environment".into());
    }
    let fragile = clean.match_indices("fragile").count();
    if fragile > 0 {
        changed.push("fragile frames became frame*, which reads verbatim text".into());
    }
    if count("frame") > 0 {
        changed.push("\\frame{...} became \\begin{frame} ... \\end{frame}".into());
    }
    if class_src.is_some() {
        changed.push("the custom class is used as <class>.acc.cls, built on ltx-talk instead of Beamer".into());
    }
    // Overlays: one frame becomes several pages.
    let frames: Vec<&str> = clean.split("\\begin{frame}").skip(1).map(|f| f.split("\\end{frame}").next().unwrap_or("")).collect();
    let stepped = frames.iter().filter(|f| f.contains("\\pause") || f.contains("\\only<") || f.contains("\\uncover<") || f.contains("\\visible<") || f.contains("<+") || f.contains("\\item<") || f.contains("\\onslide")).count();
    if stepped > 0 {
        let s = if stepped == 1 { "" } else { "s" };
        if handout {
            changed.push(format!("{stepped} frame{s} with overlay steps: one page each (handout), with every step's content at once"));
        } else {
            changed.push(format!("{stepped} frame{s} with overlay steps: each step is its own page, so text that stays on screen is read again on every page (screen readers hear it once per step); \"One page per slide\" (--handout) makes one page per frame"));
        }
    }
    if frames.iter().any(|f| f.contains("tikzpicture") && (f.contains("visible on") || f.contains("\\pause") || f.contains("<") && f.contains("\\only"))) {
        risky.push("TikZ with overlays (visible on=<...>, \\only or \\pause inside a picture): may show every step at once".into());
    }
    let head5: String = src.lines().take(5).collect::<Vec<_>>().join("\n").to_lowercase();
    if head5.contains("!tex program") && head5.contains("xelatex") {
        changed.push(format!("built with {} instead of XeLaTeX (its tags are incomplete): fonts set with fontspec are kept, others may differ", if lua { "LuaLaTeX" } else { "pdfLaTeX" }));
    }
    let list = |title: &str, v: &[String]| if v.is_empty() { String::new() } else { format!("\n{title}\n{}\n", v.iter().map(|x| format!("  - {x}")).collect::<Vec<_>>().join("\n")) };
    format!(
        "What TeXora's tagged copy of {file_name} changed ({stem}.acc.tex, ltx-talk {v}).\n{file_name} and its own PDF are not changed. The copy is made by converting the source, so compare its\npages with the slides; this list is what the conversion saw.\n{}{}{}",
        list("Left out:", &out),
        list("Changed:", &changed),
        list("Check by hand:", &risky),
        stem = file_name.trim_end_matches(".tex"),
        v = ltx_talk.unwrap_or("?"),
    )
}

pub fn talk_copy(file_name: &str, src: &str, lang: &str, gate: &str, lua: bool, class: &str, local: &[String]) -> String {
    let mut s = talk_convert(src, lua, local);
    // The class (options are Beamer's), with the commands ltx-talk lacks.
    if let Some((k, e)) = uses(&s, "documentclass").into_iter().next() {
        let end = command_end(&s, e, 1);
        s = splice(&s, vec![(k, end, format!("\\documentclass{{{class}}}\n{TALK_SHIM}"))]);
    }
    let mut head = format!("% Made by TeXora: a tagged copy of {file_name} (Beamer) made with the ltx-talk class. That\n% file and its own PDF are not changed; Beamer's theme is not used here.\n");
    if !uncomment(&s).contains("\\DocumentMetadata") {
        head.push_str(&format!("\\DocumentMetadata{{lang={lang},pdfstandard=ua-2,tagging=on}}\n"));
    }
    if !gate.is_empty() {
        head.push_str(&format!("\\PassOptionsToPackage{{{gate}}}{{accessgate}}\n"));
    }
    head + &s
}

/// The Beamer commands ltx-talk lacks, for a class or package file (where `@` is already a letter).
fn shim_in_package() -> String {
    TALK_SHIM.lines().filter(|l| !l.starts_with("\\makeat")).collect::<Vec<_>>().join("\n") + "\n"
}

/// A local theme file (`beamerthemeAcme.sty`) as `beamerthemeAcme.acc.sty`: its colours and templates
/// go; environments and commands it defines stay.
pub fn talk_theme(name: &str, src: &str, lua: bool, local: &[String]) -> String {
    format!("{}{}", generated_header(&format!("{name}.acc.sty"), &format!("{name}.sty"), "theme"), talk_convert(src, lua, local))
}

/// Today as LaTeX writes dates, `2026/11/13` (UTC).
pub fn today() -> String {
    let days = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() / 86_400).unwrap_or(0) as i64;
    // Days since 1970-01-01 to a civil date (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}/{m:02}/{d:02}")
}

/// The header of a file TeXora generates from one of the author's (a theme or class beside the
/// presentation): it names TeXora, its source and the date, and does not relabel the author's content,
/// which keeps its own terms. Only TeXora's additions (the preamble in a class copy) are under TeXora's
/// license.
fn generated_header(file: &str, from: &str, what: &str) -> String {
    format!(
        "%% {file} -- generated by TeXora from {from}, for the tagged copy of a presentation that uses it.\n\
%%\n\
%% Generated by TeXora, a LaTeX-native presenter for Beamer and PDF slides.\n\
%% Author:    Khalid Alkhaldi (TeXora)\n\
%% Source:    https://github.com/khalidt/TeXora\n\
%% Date:      {}\n\
%% The {what} below is {from}, converted; it keeps its own author and license. TeXora's additions\n\
%% are under PolyForm Noncommercial 1.0.0 (SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0).\n\
%% Made again on every tagged build; edit {from}, not this file.\n",
        today()
    )
}

/// A custom class built on Beamer, as one built on `ltx-talk` (saved as `<class>.acc.cls` for the
/// tagged copy): its own environments and commands stay; `\LoadClass{beamer}` becomes ltx-talk, and
/// options passed to Beamer are dropped.
pub fn talk_class(name: &str, src: &str, lua: bool, local: &[String]) -> String {
    let mut s = talk_convert(src, lua, local);
    let mut edits = vec![];
    for cmd in ["LoadClass", "LoadClassWithOptions", "PassOptionsToClass", "ProvidesClass"] {
        for (k, e) in uses(&s, cmd) {
            let n = if cmd == "PassOptionsToClass" { 2 } else { 1 };
            let end = command_end(&s, e, n);
            let (_, mand, _) = args_after(&s, e, n);
            let text = match cmd {
                "ProvidesClass" => format!("\\ProvidesClass{{{name}.acc}}[TeXora's tagged copy of {name}]"),
                _ if mand.last().is_some_and(|m| m.trim() == "beamer") => if cmd == "PassOptionsToClass" { String::new() } else { format!("\\LoadClass{{ltx-talk}}\n{}", shim_in_package()) },
                _ => continue,
            };
            edits.push((k, end, text));
        }
    }
    edits.sort_by_key(|e| e.0);
    s = splice(&s, edits).replace("\\usepackage{beamer", "\\RequirePackage{beamer");
    format!("{}{s}", generated_header(&format!("{name}.acc.cls"), &format!("{name}.cls"), "class"))
}

/// What the tags of a PDF say.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tags {
    pub tagged: bool,
    pub marked: bool,
    pub lang: Option<String>,
    pub figures: usize,
    pub figures_without_alt: usize,
    pub links: usize,
}

/// Reads the structure of a PDF: its tree, language, and each Figure's alt text. Compressed object
/// streams are opened (Flate), so tagged PDFs from any engine are read.
pub fn tags(pdf: &Path) -> Tags {
    let Ok(raw) = std::fs::read(pdf) else { return Tags::default() };
    let mut all = raw.clone();
    let mut i = 0;
    while let Some(p) = find(&raw[i..], b"stream") {
        let mut start = i + p + 6;
        if raw.get(start) == Some(&b'\r') {
            start += 1;
        }
        if raw.get(start) == Some(&b'\n') {
            start += 1;
        }
        let Some(e) = find(&raw[start..], b"endstream") else { break };
        if let Ok(d) = miniz_oxide::inflate::decompress_to_vec_zlib(&raw[start..start + e]) {
            all.extend_from_slice(&d);
            all.push(b'\n');
        }
        i = start + e + 9;
    }
    let text = String::from_utf8_lossy(&all).to_string();
    let compact = text.replace(' ', "");
    let mut t = Tags { tagged: compact.contains("/StructTreeRoot"), marked: compact.contains("/Markedtrue"), ..Default::default() };
    if let Some(k) = text.find("/Lang") {
        let rest = text[k + 5..].trim_start();
        t.lang = rest.strip_prefix('(').and_then(|r| r.split_once(')')).map(|(l, _)| l.to_string());
    }
    // Each structure element whose type is Figure: does its dictionary have /Alt?
    for pat in ["/S/Figure", "/S /Figure"] {
        for (k, _) in text.match_indices(pat) {
            let after = &text[k + pat.len()..];
            if after.starts_with(|c: char| c.is_ascii_alphanumeric()) {
                continue;
            }
            t.figures += 1;
            let open = text[..k].rfind("<<").unwrap_or(0);
            let close = dict_end(&text, open);
            if !text[open..close].contains("/Alt") {
                t.figures_without_alt += 1;
            }
        }
    }
    t.links = compact.matches("/S/Link").count();
    t
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

/// Where the dictionary opened at `open` (`<<`) closes.
fn dict_end(text: &str, open: usize) -> usize {
    let b = text.as_bytes();
    let (mut depth, mut i) = (0i32, open);
    while i + 1 < b.len() {
        if b[i] == b'<' && b[i + 1] == b'<' {
            depth += 1;
            i += 2;
            continue;
        }
        if b[i] == b'>' && b[i + 1] == b'>' {
            depth -= 1;
            i += 2;
            if depth == 0 {
                return i;
            }
            continue;
        }
        i += 1;
    }
    text.len()
}

/// The `[code] message` reports accessgate wrote into a LaTeX log.
pub fn log_reports(log: &str) -> Vec<(String, String)> {
    let joined = log.replace("\n(accessgate)", " ").replace("\n(accessgate)                ", " ");
    let mut out = vec![];
    for line in joined.lines() {
        for lead in ["Package accessgate Warning: [", "Package accessgate Error: ["] {
            if let Some(k) = line.find(lead) {
                let rest = &line[k + lead.len()..];
                if let Some((code, msg)) = rest.split_once(']') {
                    let msg = msg.trim().split(" on input line").next().unwrap_or("").trim().to_string();
                    out.push((code.to_string(), msg));
                }
            }
        }
    }
    out
}

/// What the accessible build made.
#[derive(Clone, Debug, Default)]
pub struct Build {
    /// `<name>.acc.pdf` beside the presentation, when it was made.
    pub pdf: Option<PathBuf>,
    pub ok: bool,
    /// In plain words: what was made, or why not.
    pub note: String,
    pub tags: Option<Tags>,
    /// The source check, the log's reports and the PDF check together.
    pub findings: Vec<Finding>,
}

/// Makes `<name>.acc.pdf`, a tagged copy, through `<name>.acc.tex`; the document and its PDF are left
/// alone. LuaLaTeX when installed (the best tags), else pdfLaTeX. Beamer slides go through `talk_copy`.
#[cfg(test)]
pub fn build(root: &Path, strict: bool) -> Build {
    build_with(root, &BuildOptions { strict, ..Default::default() })
}

/// How the tagged copy is made.
#[derive(Clone, Copy, Debug, Default)]
pub struct BuildOptions {
    /// Every report is a LaTeX error (accessgate's `strict`).
    pub strict: bool,
    /// Every `\AccMotion` shows its still picture (accessgate's `still`).
    pub still: bool,
    /// Beamer slides: one page per frame, the overlay steps together (ltx-talk's `handout`), so a
    /// screen reader does not read text that stays on screen again on every step.
    pub handout: bool,
}

pub fn build_with(root: &Path, o: &BuildOptions) -> Build {
    let strict = o.strict;
    let gate = [("strict", o.strict), ("still", o.still)].iter().filter(|(_, on)| *on).map(|(n, _)| *n).collect::<Vec<_>>().join(",");
    let mut b = Build { findings: check(root), ..Default::default() };
    let src = std::fs::read_to_string(root).unwrap_or_default();
    let beamer = beamer_based(root.parent().unwrap_or(Path::new(".")), &src);
    if beamer && !ltx_talk() {
        b.note = "No tagged copy: Beamer refuses LaTeX's tagging, and the ltx-talk class (TeX Live 2026), through which TeXora tags Beamer slides, is not installed. The check below still applies.".into();
        return b;
    }
    let dir = root.parent().unwrap_or(Path::new(".")).to_path_buf();
    let name = root.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let stem = root.file_stem().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let wrap = dir.join(format!("{stem}.acc.tex"));
    let has_meta = uncomment(&src).contains("\\DocumentMetadata");
    let lua = crate::latex::find_tool("lualatex").is_some();
    let engine = if lua { crate::latex::Engine::LuaLatex } else { crate::latex::Engine::PdfLatex };
    // A custom class built on Beamer goes along as <class>.acc.cls, built on ltx-talk.
    // Theme files beside it go along as <theme>.acc.sty (their environments and commands stay).
    let mut class = "ltx-talk".to_string();
    let mut local = vec![];
    if beamer {
        let themes: Vec<(String, String)> = std::fs::read_dir(&dir).into_iter().flatten().flatten().filter_map(|e| {
            let f = e.file_name().to_string_lossy().to_string();
            let stem = f.strip_suffix(".sty")?;
            let is_theme = stem.starts_with("beamer") && stem.contains("theme") && !stem.ends_with(".acc");
            is_theme.then(|| (stem.to_string(), std::fs::read_to_string(e.path()).unwrap_or_default()))
        }).collect();
        local = themes.iter().map(|(s, _)| s.clone()).collect();
        for (stem, text) in &themes {
            let _ = std::fs::write(dir.join(format!("{stem}.acc.sty")), talk_theme(stem, text, lua, &local));
        }
        if let Some(own) = class_of(&src).filter(|c| c != "beamer") {
            let cls = std::fs::read_to_string(dir.join(format!("{own}.cls"))).unwrap_or_default();
            if std::fs::write(dir.join(format!("{own}.acc.cls")), talk_class(&own, &cls, lua, &local)).is_ok() {
                class = format!("{own}.acc");
            }
        }
    }
    let mut text = if beamer { talk_copy(&name, &src, &language(&src), &gate, lua, &class, &local) } else { wrapper(&name, &language(&src), has_meta, &gate) };
    if beamer && o.handout {
        // After \DocumentMetadata (which comes first), before the class.
        let at = text.find("\\DocumentMetadata").and_then(|k| text[k..].find('\n').map(|n| k + n + 1)).unwrap_or(0);
        text.insert_str(at, "\\PassOptionsToClass{handout}{ltx-talk}\n");
    }
    if std::fs::write(&wrap, text).is_err() {
        b.note = format!("{stem}.acc.tex could not be written beside the presentation.");
        return b;
    }
    if beamer {
        let class_src = class_of(&src).filter(|c| c != "beamer").and_then(|c| std::fs::read_to_string(dir.join(format!("{c}.cls"))).ok());
        let report = talk_losses(&name, &src, class_src.as_deref(), lua, &local, ltx_talk_version().as_deref(), o.handout);
        let _ = std::fs::write(dir.join(format!("{stem}.acc-losses.txt")), report);
    }
    let r = crate::latex::compile(&wrap, engine);
    let reg = Registry::load(&dir).0;
    let sev = if strict || reg.errors { Severity::Error } else { Severity::Warning };
    for (code, msg) in log_reports(&r.log) {
        if code == "placeholder" {
            b.findings.push(Finding { file: root.to_path_buf(), line: 1, subject: "(build)".into(), code: code.clone(), message: msg, wcag: reg.wcag(&code), severity: sev, col: 0, target: String::new(), fix: fix_for(&code, "") });
        }
    }
    if !r.ok || !r.pdf.exists() {
        let why = r.problems.iter().find(|p| p.is_error).map(|p| p.message.clone()).unwrap_or_else(|| "see the log".into());
        b.note = if !r.note.is_empty() {
            r.note.clone()
        } else if beamer {
            if why.contains("requires LaTeX") {
                // ltx-talk updated on its own: it needs the LaTeX it was made for.
                format!("The tagged copy could not be built: {why}. ltx-talk was updated without LaTeX itself; update both, for example with `tlmgr update --self --all` (with sudo where TeX Live belongs to the system).")
            } else {
                format!("The tagged copy could not be built ({why}). Beamer slides are tagged through the ltx-talk class, which lacks some Beamer commands; {stem}.acc.tex and {stem}.acc.log show where it stopped, and {stem}.acc-losses.txt what the copy changed.")
            }
        } else {
            format!("The tagged copy could not be built ({why}).")
        };
        return b;
    }
    let t = tags(&r.pdf);
    let doc = |code: &str, message: String, severity: Severity| Finding { file: root.to_path_buf(), line: 1, subject: "(PDF)".into(), code: code.into(), message, wcag: reg.wcag(code), severity, col: 0, target: String::new(), fix: if code == "alt" { "give the pictures alt={...} in the source (the check above lists them), then build again".into() } else { "update TeX Live (2023 or later), then build again".into() } };
    if !t.tagged {
        b.findings.push(doc("tagging", "the copy has no tags: this LaTeX does not tag documents yet (TeX Live 2023 or later is needed)".into(), sev));
    }
    if t.figures_without_alt > 0 {
        b.findings.push(doc("alt", format!("{} of {} pictures in the tagged copy have no alt text", t.figures_without_alt, t.figures), sev));
    }
    b.ok = t.tagged;
    b.tags = Some(t.clone());
    b.note = if t.tagged {
        format!(
            "Tagged copy (not validated) made with {}{}: {} ({} picture{}, {} with alt text; language {}).{}",
            engine.name(),
            if beamer { format!(" and the ltx-talk class {}, without Beamer's theme", ltx_talk_version().unwrap_or_default()) } else { String::new() },
            r.pdf.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
            t.figures,
            if t.figures == 1 { "" } else { "s" },
            t.figures - t.figures_without_alt,
            t.lang.clone().unwrap_or_else(|| "not set".into()),
            if beamer { format!(" What the copy changed: {stem}.acc-losses.txt.") } else { String::new() }
        )
    } else {
        "A copy was built, but it has no tags.".into()
    };
    b.pdf = Some(r.pdf);
    b
}

/// The command line, for any editor, script or CI:
///   `--check-accessibility <file.tex> [--strict]`   the source check, one finding per line
///   `--build-accessible <file.tex> [--strict] [--still] [--handout]`  the tagged copy (--still: animations
///                                                     as still pictures; --handout: one page per frame), then every finding
/// Exit code: 0 with nothing to fix (notes allowed), 1 when something is to fix with `--strict` or an
/// error is found, 2 when the file cannot be read.
pub fn cli(build_it: bool, rest: &[String]) -> i32 {
    let strict = rest.iter().any(|a| a == "--strict");
    let still = rest.iter().any(|a| a == "--still");
    let Some(file) = rest.iter().find(|a| !a.starts_with("--")) else {
        eprintln!("Give a .tex file, for example: --check-accessibility talk.tex");
        return 2;
    };
    let root = std::path::absolute(Path::new(file)).unwrap_or_else(|_| PathBuf::from(file));
    if !root.is_file() {
        eprintln!("{file} was not found.");
        return 2;
    }
    let dir = root.parent().unwrap_or(Path::new(".")).to_path_buf();
    let findings = if build_it {
        let handout = rest.iter().any(|a| a == "--handout");
        let b = build_with(&root, &BuildOptions { strict, still, handout });
        println!("{}", b.note);
        b.findings
    } else {
        check(&root)
    };
    for f in &findings {
        println!("{}  ({})", f.line_text(&dir), f.severity.word());
        if !f.fix.is_empty() {
            println!("    fix: {}", f.fix);
        }
    }
    let to_fix = findings.iter().filter(|f| f.severity >= Severity::Warning).count();
    let errors = findings.iter().filter(|f| f.severity == Severity::Error).count();
    let reg = Registry::load(&dir).0;
    let codes: Vec<String> = {
        let mut c: Vec<&str> = findings.iter().filter(|f| f.severity >= Severity::Warning).map(|f| f.code.as_str()).collect();
        c.sort();
        c.dedup();
        c.iter().map(|c| match reg.codes.get(*c) {
            Some(x) if !x.summary.is_empty() => format!("  [{c}] {}", x.summary),
            _ => format!("  [{c}]"),
        }).collect()
    };
    if to_fix == 0 {
        println!("Nothing to fix for screen readers ({} note{}).", findings.len(), if findings.len() == 1 { "" } else { "s" });
    } else {
        println!("{to_fix} thing{} to fix:\n{}", if to_fix == 1 { "" } else { "s" }, codes.join("\n"));
    }
    if errors > 0 || (strict && to_fix > 0) { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deck(dir_name: &str, src: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("texora-access-{dir_name}-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let p = d.join("talk.tex");
        std::fs::write(&p, src).unwrap();
        p
    }

    #[test]
    fn the_registry_is_valid_data_and_is_extended_by_the_author() {
        let r = Registry::parse(REGISTRY).unwrap();
        assert!(r.commands.contains_key("includegraphics") && r.commands.contains_key("includevideo") && r.environments.contains_key("tabular"));
        assert_eq!(r.codes["alt"].wcag, "1.1.1");
        assert_eq!(requirement("alt = alt | artifact"), Requirement { code: "alt".into(), keys: vec!["alt".into(), "artifact".into()] });
        assert_eq!(requirement("caption | transcript").code, "caption");
        // A wrapper macro the author declared is checked like the built-in ones.
        let p = deck("reg", "\\title{T}\\DocumentMetadata{lang=en}\n\\photo[width=2cm]{a.png}\n\\photo[alt={A cat}]{b.png}\n");
        std::fs::write(p.parent().unwrap().join(REGISTRY_FILE), "[command.photo]\nrole = \"Figure\"\nrequires = [\"alt = alt | artifact\"]\n").unwrap();
        let f: Vec<_> = check(&p).into_iter().filter(|f| f.subject == "photo").collect();
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!((f[0].line, f[0].code.as_str()), (2, "alt"));
        // A broken registry file is reported, and TeXora's is used.
        std::fs::write(p.parent().unwrap().join(REGISTRY_FILE), "[command.photo\n").unwrap();
        assert!(check(&p).iter().any(|f| f.code == "registry"));
    }

    #[test]
    fn options_with_commas_brackets_and_braces_are_read_whole() {
        assert_eq!(keys_of("width=2cm, transcript={Hello, world [1] {x}}, alt={a, b}, autoplay"), vec!["width", "transcript", "alt", "autoplay"]);
    }

    #[test]
    fn the_check_finds_what_screen_readers_need() {
        let src = "\\documentclass{article}\n\\usepackage{graphicx,hyperref,texoravideo}\n\\begin{document}\n\
\\includegraphics[width=2cm]{a.png}\n\
\\includegraphics[width=2cm,alt={A grey square}]{a.png}\n\
\\includegraphics[artifact]{line.png}\n\
\\includevideo[width=4cm][autoplay,loop,alt={Play video: demo},transcript={Hi, there [1]}]{clip.mp4}\n\
\\includevideo{clip2.mp4}\n\
\\href{https://example.com}{here}\n\
\\href{https://example.com}{the TeXora project page}\n\
\\begin{tabular}{ll} A & B\\\\ 1 & 2\\end{tabular}\n\
\\begin{tabular}{ll} A & B\\\\ \\hline 1 & 2\\end{tabular}\n\
\\AccFigure{Bars growing}{\\animategraphics[autoplay]{10}{f-}{1}{9}}\n\
\\animategraphics[autoplay,loop]{10}{f-}{1}{9}\n\
% \\includegraphics{commented.png}\n\
\\end{document}\n";
        let p = deck("lint", src);
        let f = check(&p);
        let got: Vec<(usize, &str, &str)> = f.iter().map(|x| (x.line, x.subject.as_str(), x.code.as_str())).collect();
        assert!(got.contains(&(4, "includegraphics", "alt")), "{got:?}");
        assert!(!got.iter().any(|g| g.0 == 5 || g.0 == 6), "alt and artifact satisfy it: {got:?}");
        assert!(got.contains(&(7, "includevideo", "motion")) && !got.contains(&(7, "includevideo", "alt")) && !got.contains(&(7, "includevideo", "transcript")), "{got:?}");
        assert!(got.contains(&(8, "includevideo", "alt")) && got.contains(&(8, "includevideo", "transcript")), "{got:?}");
        assert!(got.contains(&(9, "href", "link")) && !got.iter().any(|g| g.0 == 10), "{got:?}");
        assert!(got.contains(&(11, "tabular", "table")) && got.contains(&(12, "tabular", "table")), "{got:?}");
        assert!(f.iter().filter(|x| x.code == "table").all(|x| x.severity == Severity::Info && x.fix.contains("\\AccHeaderRows{1}")), "table findings are notes whose fix names \\AccHeaderRows");
        let marked = deck("lint-table", "\\title{T}\\DocumentMetadata{lang=en}\n\\AccHeaderRows{1}\n\\begin{tabular}{ll} A & B\\\\ \\hline 1 & 2\\end{tabular}\n");
        assert!(!check(&marked).iter().any(|x| x.code == "table"), "\\AccHeaderRows marks the header row");
        assert!(!got.iter().any(|g| g.0 == 13), "inside \\AccFigure it is described: {got:?}");
        assert!(got.contains(&(14, "animategraphics", "alt")) && got.contains(&(14, "animategraphics", "motion")), "{got:?}");
        assert!(!got.iter().any(|g| g.0 == 15), "comments are skipped");
        assert!(got.contains(&(1, "(document)", "language")) && got.contains(&(3, "(document)", "doctitle")), "the language at \\documentclass, the title before \\begin{{document}}: {got:?}");
        let link = f.iter().find(|x| x.code == "link").unwrap();
        assert_eq!((link.line, link.col, link.target.as_str()), (9, 28, "here"), "the vague link text itself is pointed at");
        assert!(link.fix.contains("\\href{https://example.com}{"), "{}", link.fix);
        assert!(f.iter().all(|x| !x.fix.is_empty()), "every finding says how to fix it");
        let line = f.iter().find(|x| x.line == 4).unwrap().line_text(p.parent().unwrap());
        assert_eq!(line, "talk.tex:4:1  includegraphics  [alt] missing alt={...} (or artifact, decorative)  WCAG 1.1.1");
    }

    #[test]
    fn slides_need_titles_in_every_frame_form() {
        let src = "\\documentclass{beamer}\n\\usepackage[ngerman]{babel}\\title{Talk}\n\\begin{document}\n\\begin{frame}\\titlepage\\end{frame}\n\\begin{frame}{Intro}x\\end{frame}\n\\begin{frame}\nno title\n\\end{frame}\n\\frame{\\frametitle{Two} y}\n\\frame{z}\n\\end{document}\n";
        let p = deck("frames", src);
        let f = check(&p);
        let titles: Vec<usize> = f.iter().filter(|x| x.code == "title").map(|x| x.line).collect();
        assert_eq!(titles, vec![6, 10], "{f:?}");
        assert!(!f.iter().any(|x| x.code == "language"), "babel sets the language");
        assert!(f.iter().any(|x| x.code == "tagging" && x.message.starts_with("Beamer")), "beamer is explained");
        assert_eq!(language(src), "de");
    }

    #[test]
    fn the_wrapper_puts_metadata_first_and_leaves_the_document_alone() {
        let w = wrapper("talk.tex", "en-US", false, "strict");
        let lines: Vec<&str> = w.lines().filter(|l| !l.starts_with('%')).collect();
        assert!(lines[0].starts_with("\\IfFormatAtLeastTF{2025-06-01}"));
        assert!(w.contains("tagging=on") && w.contains("testphase={phase-III}") && w.contains("lang=en-US"));
        assert!(w.contains("\\PassOptionsToPackage{strict}{accessgate}\n\\input{talk.tex}"));
        let own = wrapper("talk.tex", "en-US", true, "");
        assert!(!own.contains("DocumentMetadata{") && own.contains("\\input{talk.tex}"), "a document with its own metadata keeps it");
    }

    #[test]
    fn log_reports_are_read_by_their_codes() {
        let log = "Package accessgate Warning: [alt] `a.png' has no alt text. Add alt={...} describing it\n(accessgate)                or artifact on input line 12.\n\nPackage accessgate Warning: [placeholder] Video `c.mp4' has no poster yet on input line 20.\n";
        let r = log_reports(log);
        assert_eq!(r.iter().map(|(c, _)| c.as_str()).collect::<Vec<_>>(), vec!["alt", "placeholder"]);
    }

    /// A real tagged build with LaTeX, where it is installed: the wrapper, the gate and the PDF check.
    #[test]
    fn a_real_document_gets_tags_and_alt_text() {
        if crate::latex::find_tool("lualatex").is_none() && crate::latex::find_tool("pdflatex").is_none() {
            return;
        }
        let tex = crate::video_prep::tex_dir().unwrap();
        let _ = tex;
        let p = deck("build", "\\documentclass{article}\n\\usepackage{graphicx}\n\\usepackage{accessgate}\n\\title{A test}\n\\begin{document}\n\\section{One}\nText.\n\\includegraphics[width=2cm,alt={A grey test picture}]{example-image}\n\\AccFigure{A drawn box}{\\fbox{box}}\n\\end{document}\n");
        std::fs::write(p.parent().unwrap().join("accessgate.sty"), GATE_STY).unwrap();
        let b = build(&p, false);
        let Some(pdf) = b.pdf.clone() else { panic!("no tagged copy: {}", b.note) };
        assert!(pdf.ends_with("talk.acc.pdf"));
        let t = b.tags.clone().unwrap_or_else(|| tags(&pdf));
        assert!(t.tagged && t.figures >= 2 && t.figures_without_alt == 0, "{t:?} {}", b.note);
        assert_eq!(t.lang.as_deref(), Some("en-US"));
        assert!(p.parent().unwrap().join("talk.tex").is_file() && !p.parent().unwrap().join("talk.pdf").exists(), "the document and its PDF are left alone");
    }

    const BEAMER_DECK: &str = "\\documentclass[aspectratio=169]{beamer}\n\\usetheme[progressbar=frametitle]{Madrid}\n\\usecolortheme{beaver}\n\
\\setbeamertemplate{navigation symbols}{}\n\\setbeamertemplate{footline}{%\n  \\insertframenumber}\n\\setbeamertemplate{itemize items}[circle]\n\
\\setbeamercolor{title}{fg=red}\n\\beamertemplatenavigationsymbolsempty\n\\usepackage{amsmath,amssymb,graphicx}\n\\usepackage{appendixnumberbeamer}\n\
% \\usetheme{Warsaw} stays a comment\n\\title{Talk}\\author{Ann}\n\\AtBeginSection[]{\\begin{frame}{Outline}\\tableofcontents\\end{frame}}\n\
\\begin{document}\n\\begin{frame}\\titlepage\\end{frame}\n\\begin{frame}{Outline}\\tableofcontents\\end{frame}\n\\section{One}\n\\begin{frame}[label=pic,t]{A picture}\n\
\\includegraphics[width=2cm,alt={A grey test picture}]{example-image}\\note{Say hello.}\n\\end{frame}\n\
\\begin{frame}{Two columns}\\transfade\n\\begin{columns}[t]\n\\column{.5\\linewidth} left $\\mathbb{R}$\n\\column[c]{.5\\linewidth} right\n\\end{columns}\n\
\\begin{alertblock}{Careful}\\structure{x}\\end{alertblock}\n\\begin{theorem}t\\end{theorem}\n\\end{frame}\n\
\\begin{frame}[fragile]{Code}{In Python}\n\\begin{verbatim}\nx = 1 % not a comment\n\\end{verbatim}\n\\end{frame}\n\
\\appendix\n\\begin{frame}[label=refs]{References}\n\\begin{thebibliography}{9}\\bibitem{a} A. Book.\\end{thebibliography}\n\\end{frame}\n\\againframe{pic}\n\\end{document}\n";

    #[test]
    fn a_beamer_deck_becomes_an_ltx_talk_copy() {
        let c = talk_copy("talk.tex", BEAMER_DECK, "en-US", "strict", true, "ltx-talk", &[]);
        let body: String = c.lines().filter(|l| !l.trim_start().starts_with('%')).collect::<Vec<_>>().join("\n");
        assert!(c.starts_with("% Made by TeXora") && body.starts_with("\\DocumentMetadata{lang=en-US,pdfstandard=ua-2,tagging=on}\n\\PassOptionsToPackage{strict}{accessgate}\n\\documentclass{ltx-talk}\n"), "{c}");
        for gone in ["\\usetheme", "\\usecolortheme", "\\setbeamer", "\\beamertemplate", "\\transfade", "amssymb", "environment=", "{beamer}"] {
            assert!(!body.contains(gone), "{gone} is left out: {body}");
        }
        assert!(c.contains("% \\usetheme{Warsaw} stays a comment"), "comments are kept");
        assert!(body.contains("\\AccAppendix\n") && !body.contains("\n\\appendix"), "\\appendix becomes \\AccAppendix");
        assert!(body.contains("\\usepackage{amsmath,graphicx}") && body.contains("\\begin{frame}[label=pic,vertical-alignment=top]\\frametitle{A picture}") && body.contains("\\begin{frame}[label=refs]\\frametitle{References}"));
        assert!(body.contains("\\begin{frame*}\n\\frametitle{Code}\\framesubtitle{In Python}\n\\begin{verbatim}\nx = 1 % not a comment\n\\end{verbatim}\n\\end{frame*}"), "fragile frames take verbatim text: {body}");
        assert!(body.contains("\\begin{frame}\\titlepage\\end{frame}"), "a frame with no title has none");
        let cmd = talk_copy("talk.tex", "\\documentclass{beamer}\n\\frame<2>[label=n,t]{\\frametitle{Numbers} x}\n\\frame{\\fbox{y}}\\framebreak\n", "en-US", "", true, "ltx-talk", &[]);
        assert!(cmd.contains("\\begin{frame}<2>[label=n,vertical-alignment=top]\\relax\\frametitle{Numbers} x\\end{frame}\n\\begin{frame}\\relax\\fbox{y}\\end{frame}\\framebreak"), "the command form becomes a frame: {cmd}");
        assert!(body.contains("\\begin{columns}[t]\n\\begin{column}{.5\\linewidth} left $\\mathbb{R}$\n\\end{column}\\begin{column}[c]{.5\\linewidth} right\n\\end{column}\\end{columns}"), "{body}");
        assert!(body.contains("alt={A grey test picture}") && body.contains("\\note{Say hello.}") && body.contains("\\againframe{pic}"), "the content stays");
        assert!(body.contains("\\ProvideDocumentCommand\\note") && body.contains("\\title{Talk}"));
        // A custom class built on Beamer counts as Beamer.
        let p = deck("custom-class", "\\documentclass{acmedeck}\n\\begin{document}\\end{document}\n");
        let dir = p.parent().unwrap();
        assert!(!beamer_based(dir, "\\documentclass{acmedeck}"));
        std::fs::write(dir.join("acmedeck.cls"), "\\ProvidesClass{acmedeck}\n% \\LoadClass{article}\n\\LoadClass[aspectratio=169]{beamer}\n").unwrap();
        assert!(beamer_based(dir, "\\documentclass{acmedeck}") && beamer_based(dir, "\\documentclass[t]{beamer}") && !beamer_based(dir, "\\documentclass{article}"));
        let cls = talk_class("acmedeck", "\\NeedsTeXFormat{LaTeX2e}\n\\ProvidesClass{acmedeck}[2026/11/09 Acme]\n\\DeclareOption*{\\PassOptionsToClass{\\CurrentOption}{beamer}}\n\\ProcessOptions\\relax\n\\LoadClass[aspectratio=169]{beamer}\n\\usetheme{Acme}\n\\newenvironment{customframe}[1]{\\begin{frame}[label=c]{#1}}{\\end{frame}}\n", true, &[]);
        assert!(cls.contains("\\ProvidesClass{acmedeck.acc}") && cls.contains("\\DeclareOption*{}") && cls.contains("\\LoadClass{ltx-talk}\n"), "{cls}");
        assert!(!cls.contains("\\usetheme") && cls.contains("\\newenvironment{customframe}[1]{\\begin{frame}[label=c]\\frametitle{#1}}"), "{cls}");
        // A theme beside the presentation is loaded as its converted copy; others are left out.
        let local = vec!["beamerthemeAcme".to_string()];
        let c = talk_copy("talk.tex", "\\documentclass{beamer}\n\\usetheme[x]{Acme}\\usecolortheme{Acme}\n\\usepackage{beamerthemeAcme,beamerthemeWarsaw}\n\\usetheme{Madrid}\n", "en-US", "", true, "ltx-talk", &local);
        assert!(c.contains("\\usepackage{beamerthemeAcme.acc}\n\\usepackage{beamerthemeAcme.acc}\n\n") && !c.contains("Madrid") && !c.contains("Warsaw"), "{c}");
        let th = talk_theme("beamerthemeAcme", "\\mode<presentation>\n\\setbeamercolor{structure}{fg=blue}\n\\newenvironment{customframe}[1]{\\begin{frame}[environment=customframe]{#1}}{\\end{frame}}\n", true, &local);
        assert!(!th.contains("setbeamercolor") && th.contains("\\begin{frame}\\frametitle{#1}") && !th.contains("\\mode<presentation>"), "{th}");
        assert!(talk_theme("t", "\\mode<presentation>{\\usefonttheme{serif}}\\mode*\n", true, &[]).contains("\\mode<presentation>{}\n"), "the braced form stays");
        // pdfLaTeX keeps amssymb (the clash is LuaLaTeX's).
        assert!(talk_copy("talk.tex", BEAMER_DECK, "en-US", "", false, "ltx-talk", &[]).contains("\\usepackage{amsmath,amssymb,graphicx}"));
    }

    /// Every package TeXora ships names its author, licence, source and date, after the first line
    /// (`%% <name> -- `, which is how TeXora recognises its own copies beside a presentation); the date
    /// is the package's own \ProvidesPackage date.
    #[test]
    fn the_packages_name_author_licence_source_and_date() {
        for (name, text) in [("accessgate.sty", GATE_STY), ("texoravideo.sty", crate::video_prep::STY), ("texoraimage.sty", crate::image_prep::STY)] {
            assert!(text.starts_with(&format!("%% {name} -- ")), "{name}");
            for line in ["%% Author:    Khalid Alkhaldi", "%% Copyright: 2026 Khalid Alkhaldi", "%% License:   PolyForm Noncommercial 1.0.0", "%% SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0", "%% Source:    https://github.com/khalidt/TeXora"] {
                assert!(text.contains(line), "{name}: {line}");
            }
            let date = text.lines().find_map(|l| l.strip_prefix("%% Date:")).map(str::trim).expect("a date");
            let provides = text.split("\\ProvidesPackage{").nth(1).and_then(|r| r.split_once('[')).map(|(_, d)| &d[..10]).unwrap();
            assert_eq!(date, provides, "{name}: the header's date is the package's date");
        }
        assert!(TALK_SHIM.contains("% SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0"));
        let theme = talk_theme("beamerthemeAcme", "\\mode<presentation>\n", true, &[]);
        assert!(theme.starts_with("%% beamerthemeAcme.acc.sty -- generated by TeXora from beamerthemeAcme.sty") && theme.contains("https://github.com/khalidt/TeXora") && theme.contains(&format!("%% Date:      {}", today())), "{theme}");
        assert!(talk_class("acmedeck", "\\LoadClass{beamer}\n", true, &[]).starts_with("%% acmedeck.acc.cls -- generated by TeXora"));
        let t = today();
        assert!(t.len() == 10 && t[..4].parse::<u32>().unwrap() >= 2026 && &t[4..5] == "/", "{t}");
    }

    #[test]
    fn colours_are_measured_against_wcag() {
        assert!((contrast([0.0; 3], [1.0; 3]) - 21.0).abs() < 0.01);
        let defined = BTreeMap::from([("ink".to_string(), [20.0 / 255.0, 24.0 / 255.0, 31.0 / 255.0])]);
        assert_eq!(colour_value("white", &defined), Some([1.0; 3]));
        let mix = colour_value("black!40", &defined).unwrap();
        assert!((mix[0] - 0.6).abs() < 1e-9, "40% black with white");
        assert!(colour_value("ink!50!white", &defined).is_some() && colour_value("nosuch", &defined).is_none());
        let src = "\\documentclass{beamer}\n\\usepackage[colorlinks,urlcolor=pale]{hyperref}\n\\definecolor{pale}{RGB}{150,190,230}\n\\definecolor{dim}{HTML}{BBBBBB}\n\
\\setbeamercolor{block title}{fg=dim,bg=white}\n\\setbeamercolor{frametitle}{fg=white,bg=black}\n\\begin{document}\n{\\setbeamercolor{background canvas}{bg=black}\\begin{frame}x\\end{frame}}\n\\href{https://x.org}{the site}\n\\end{document}\n";
        let f = colour_findings(src);
        let codes: Vec<&str> = f.iter().map(|x| x.code).collect();
        assert!(f.iter().any(|x| x.code == "contrast" && x.message.contains("\"block title\"")), "{f:?}");
        assert!(!f.iter().any(|x| x.message.contains("frametitle")), "white on black passes");
        assert!(!f.iter().any(|x| x.message.contains("normal text")), "a background set inside the document is local: {f:?}");
        assert!(f.iter().any(|x| x.code == "linkcolour" && x.message.starts_with("urlcolor=pale")) && !f.iter().any(|x| x.message.starts_with("linkcolor")), "{codes:?}");
        let hidden = colour_findings("\\hypersetup{hidelinks}\n\\begin{document}\\href{https://x.org}{the site}\\end{document}");
        assert!(hidden.iter().any(|x| x.code == "linkcolour" && x.info), "hidelinks is a note");
    }

    #[test]
    fn the_losses_report_says_what_the_copy_changed() {
        let r = talk_losses("talk.tex", BEAMER_DECK, None, true, &[], Some("0.4.6"), false);
        for want in ["theme Madrid", "colour theme beaver", "colour, font and template settings", "1 speaker note", "1 slide transition", "the slide shown at each section", "\\againframe: the slide is not repeated", "the package appendixnumberbeamer", "amssymb is not loaded", "\\column{...} became", "fragile frames became frame*"] {
            assert!(r.contains(want), "{want}:\n{r}");
        }
        assert!(!r.contains("% \\usetheme{Warsaw}") && !r.contains("theme Warsaw"), "comments are not counted");
        assert!(talk_losses("talk.tex", BEAMER_DECK, None, true, &[], Some("0.6.12"), false).contains("\\againframe: repeated by ltx-talk 0.6.12"));
        let steps = talk_losses("talk.tex", "% !TEX program = xelatex\n\\documentclass{beamer}\n\\begin{document}\n\\begin{frame}{A}one\\pause two\\end{frame}\n\\begin{frame}{B}\\begin{tikzpicture}\\node[visible on=<2>]{x};\\end{tikzpicture}\\end{frame}\n\\end{document}\n", None, true, &[], None, false);
        assert!(steps.contains("1 frame with overlay steps") && steps.contains("TikZ with overlays") && steps.contains("instead of XeLaTeX"), "{steps}");
    }

    /// The Beamer deck above, built for real where TeX Live has ltx-talk (2026): tagged, every frame,
    /// the picture's alt text.
    #[test]
    fn a_real_beamer_deck_gets_a_tagged_copy() {
        if !ltx_talk() {
            return;
        }
        let p = deck("beamer-build", BEAMER_DECK);
        std::fs::write(p.parent().unwrap().join("accessgate.sty"), GATE_STY).unwrap();
        let b = build(&p, false);
        let Some(pdf) = b.pdf.clone() else { panic!("no tagged copy: {}", b.note) };
        let t = b.tags.clone().unwrap();
        assert!(t.tagged && t.figures == 1 && t.figures_without_alt == 0, "{t:?} {}", b.note);
        assert!(b.note.contains("ltx-talk"), "{}", b.note);
        let log = std::fs::read_to_string(pdf.with_extension("log")).unwrap();
        assert!(!log.contains("\n! "), "no LaTeX errors");
    }

    /// A custom class built on Beamer, with a theme beside it that defines a frame environment.
    #[test]
    fn a_real_custom_class_deck_gets_a_tagged_copy() {
        if !ltx_talk() {
            return;
        }
        let p = deck("custom-build", "\\documentclass{acmedeck}\n\\title{Review}\n\\begin{document}\n\\begin{frame}\\titlepage\\end{frame}\n\\begin{customframe}{Plan}\n\\begin{itemize}\\item One\\item<2> Two\\end{itemize}\n\\end{customframe}\n\\end{document}\n");
        let dir = p.parent().unwrap();
        std::fs::write(dir.join("acmedeck.cls"), "\\NeedsTeXFormat{LaTeX2e}\n\\ProvidesClass{acmedeck}[2026/11/09 Acme]\n\\LoadClass[aspectratio=169]{beamer}\n\\usetheme{Acme}\n").unwrap();
        std::fs::write(dir.join("beamerthemeAcme.sty"), "\\mode<presentation>\n\\definecolor{acmeblue}{RGB}{20,60,140}\n\\setbeamercolor{frametitle}{bg=acmeblue,fg=white}\n\\setbeamertemplate{footline}{\\hfill\\insertframenumber/\\inserttotalframenumber}\n\\newenvironment{customframe}[1]{\\begin{frame}[environment=customframe]{#1}}{\\end{frame}}\n\\mode<all>\n").unwrap();
        let b = build(&p, false);
        assert!(b.pdf.is_some() && b.ok, "{}", b.note);
        assert!(dir.join("acmedeck.acc.cls").is_file() && dir.join("beamerthemeAcme.acc.sty").is_file());
        assert!(std::fs::read_to_string(dir.join("acmedeck.cls")).unwrap().contains("{beamer}"), "the class itself is not changed");
    }
}
