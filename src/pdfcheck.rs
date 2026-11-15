//! The accessibility checker for any PDF, slides or not: what can be read from the PDF itself (its
//! tags, language, title, PDF/UA declaration, alt text, headings, table headers, links, annotations,
//! fonts, and the text, media and animations on each page), together with the source check when the
//! PDF has a `.tex` file, sorted under the WCAG 2.2 success criteria they concern. Each criterion is
//! passed, failed, to review by a person, or not applicable, with the issues behind it and what to do.
//!
//! This is an automated check: it finds what a program can find. It is not a PDF/UA validation (that
//! is veraPDF's job, which the panel can run when it is installed) and it cannot judge whether alt
//! text is right, the reading order sensible or the colours of pictures readable.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// What the PDF's own structure says.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Facts {
    pub tagged: bool,
    pub marked: bool,
    pub lang: Option<String>,
    pub title: Option<String>,
    pub display_title: bool,
    /// PDF/UA part declared in the metadata (1 or 2).
    pub ua_part: Option<u8>,
    pub encrypted: bool,
    /// Encrypted with the permission for assistive technology turned off.
    pub blocks_assistive: bool,
    pub images: usize,
    pub figures: usize,
    pub figures_without_alt: usize,
    pub headings: usize,
    pub tables: usize,
    pub header_cells: usize,
    pub lists: usize,
    pub struct_links: usize,
    pub link_annots: usize,
    pub link_annots_without_contents: usize,
    pub pages_with_annots_without_tabs: usize,
    pub screen_annots_without_contents: usize,
    pub fonts: usize,
    pub fonts_without_unicode: usize,
    /// Bookmarks (outline items).
    pub bookmarks: usize,
}

/// One page, as the PDF engine read it.
#[derive(Clone, Debug, Default)]
pub struct PageSummary {
    pub has_text: bool,
    pub links: usize,
    pub media: usize,
    /// Animations that start by themselves and loop, with no pause button of their own.
    pub moving_without_pause: usize,
    pub animations: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    Fail,
    Review,
    Pass,
    NotApplicable,
}

impl Status {
    pub fn word(self) -> &'static str {
        match self {
            Status::Fail => "Fails",
            Status::Review => "Check by hand",
            Status::Pass => "Passes",
            Status::NotApplicable => "Not applicable",
        }
    }
}

/// A place in the source: file, line, column, length of the text to select.
pub type SourceSpot = (PathBuf, usize, usize, usize);

/// Something found, with where it is (a page of the PDF, or a place in the source) and what to do.
#[derive(Clone, Debug, PartialEq)]
pub struct Issue {
    pub text: String,
    pub fix: String,
    /// Page index in the PDF (from 0).
    pub page: Option<usize>,
    /// The source finding.
    pub source: Option<SourceSpot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Criterion {
    /// `1.1.1`, or `PDF/UA` for what PDF/UA asks beyond WCAG.
    pub id: &'static str,
    pub name: &'static str,
    /// Perceivable, Operable, Understandable, Robust, or PDF.
    pub principle: &'static str,
    pub status: Status,
    pub summary: String,
    pub issues: Vec<Issue>,
}

#[derive(Clone, Debug, Default)]
pub struct Report {
    pub pdf: PathBuf,
    pub facts: Facts,
    pub criteria: Vec<Criterion>,
    /// The `.tex` file that was checked too, if any.
    pub source: Option<PathBuf>,
}

impl Report {
    pub fn count(&self, s: Status) -> usize {
        self.criteria.iter().filter(|c| c.status == s).count()
    }
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

/// The dictionary that contains position `k`: from the nearest `<<` before it to its `>>`.
fn dict_around(text: &str, k: usize) -> &str {
    let open = text[..k].rfind("<<").unwrap_or(0);
    &text[open..dict_end(text, open).max(k)]
}

/// A PDF string at the start of `s`: `(literal)` or `<hex>`, UTF-16 when it starts with a BOM.
fn pdf_string(s: &str) -> Option<String> {
    let s = s.trim_start();
    if let Some(rest) = s.strip_prefix('(') {
        let mut out = Vec::new();
        let (mut depth, mut chars) = (0i32, rest.bytes().peekable());
        while let Some(c) = chars.next() {
            match c {
                b'\\' => {
                    if let Some(n) = chars.next() {
                        out.push(match n {
                            b'n' => b'\n',
                            b'r' => b'\r',
                            b't' => b'\t',
                            other => other,
                        });
                    }
                }
                b'(' => {
                    depth += 1;
                    out.push(c);
                }
                b')' if depth == 0 => break,
                b')' => {
                    depth -= 1;
                    out.push(c);
                }
                _ => out.push(c),
            }
        }
        return Some(decode(&out));
    }
    if let Some(rest) = s.strip_prefix('<') {
        let hex: String = rest.chars().take_while(|c| *c != '>').filter(|c| c.is_ascii_hexdigit()).collect();
        let bytes: Vec<u8> = (0..hex.len() / 2).filter_map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).ok()).collect();
        return Some(decode(&bytes));
    }
    None
}

fn decode(b: &[u8]) -> String {
    if b.starts_with(&[0xFE, 0xFF]) {
        let units: Vec<u16> = b[2..].chunks(2).filter(|c| c.len() == 2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect();
        String::from_utf16_lossy(&units)
    } else {
        String::from_utf8_lossy(b).to_string()
    }
}

/// Reads the PDF's structure: every stream that is not a picture or a font program is opened
/// (Flate), so tags in object streams are found too.
pub fn facts(bytes: &[u8]) -> Facts {
    let mut all = bytes.to_vec();
    let mut i = 0;
    while let Some(p) = find(&bytes[i..], b"stream") {
        let kw = i + p;
        let mut start = kw + 6;
        if bytes.get(start) == Some(&b'\r') {
            start += 1;
        }
        if bytes.get(start) == Some(&b'\n') {
            start += 1;
        }
        let Some(e) = find(&bytes[start..], b"endstream") else { break };
        // The stream's dictionary is just before the keyword; pictures and fonts are skipped.
        let head = String::from_utf8_lossy(&bytes[kw.saturating_sub(400)..kw]);
        let dict = head.rsplit("obj").next().unwrap_or("");
        let skip = dict.contains("/Image") || dict.contains("/FontFile") || dict.contains("/Length1") || dict.contains("/Length2");
        if !skip {
            if let Ok(d) = miniz_oxide::inflate::decompress_to_vec_zlib(&bytes[start..start + e]) {
                all.extend_from_slice(&d);
                all.push(b'\n');
            }
        }
        i = start + e + 9;
    }
    let text = String::from_utf8_lossy(&all).to_string();
    // Names compared without the spaces PDF writers put between tokens.
    let compact = text.replace([' ', '\n', '\r'], "");
    let raw = String::from_utf8_lossy(bytes);
    let mut f = Facts {
        tagged: compact.contains("/StructTreeRoot"),
        marked: compact.contains("/Markedtrue"),
        display_title: compact.contains("/DisplayDocTitletrue"),
        encrypted: raw.contains("/Encrypt"),
        images: raw.matches("/Subtype/Image").count() + raw.matches("/Subtype /Image").count(),
        ..Default::default()
    };
    // The catalogue's language (structure elements may have their own /Lang too).
    if let Some(k) = text.find("/Type/Catalog").or_else(|| text.find("/Type /Catalog")) {
        let d = dict_around(&text, k);
        if let Some(l) = d.find("/Lang") {
            f.lang = pdf_string(&d[l + 5..]).filter(|s| !s.trim().is_empty());
        }
    }
    if f.lang.is_none() {
        if let Some(l) = text.find("/Lang") {
            f.lang = pdf_string(&text[l + 5..]).filter(|s| !s.trim().is_empty());
        }
    }
    // The title: the document information (the dictionary with the producer or dates; outline items
    // have titles too), else the XMP metadata.
    for (k, _) in text.match_indices("/Title") {
        let d = dict_around(&text, k);
        if d.contains("/Producer") || d.contains("/CreationDate") || d.contains("/Creator") {
            f.title = pdf_string(&text[k + 6..]).filter(|s| !s.trim().is_empty());
            if f.title.is_some() {
                break;
            }
        }
    }
    if f.title.is_none() {
        if let Some(k) = text.find("<dc:title>") {
            let rest = &text[k..];
            let inner = rest.split("</dc:title>").next().unwrap_or("");
            let clean: String = inner.split('<').map(|p| p.split_once('>').map(|(_, t)| t).unwrap_or("")).collect::<Vec<_>>().join("").trim().to_string();
            if !clean.is_empty() {
                f.title = Some(clean);
            }
        }
    }
    // PDF/UA part: <pdfuaid:part>2</pdfuaid:part> or pdfuaid:part="1".
    if let Some(k) = text.find("pdfuaid:part") {
        f.ua_part = text[k + 12..].chars().skip_while(|c| !c.is_ascii_digit()).take(1).collect::<String>().parse().ok();
    }
    if f.encrypted {
        if let Some(k) = raw.find("/P ").or_else(|| raw.find("/P-")) {
            let num: String = raw[k + 2..].trim_start().chars().take_while(|c| c.is_ascii_digit() || *c == '-').collect();
            if let Ok(p) = num.parse::<i64>() {
                f.blocks_assistive = p & 512 == 0;
            }
        }
    }
    // Role maps: the document's own structure names and the standard type each stands for
    // (`/frametitle /H4` in /RoleMap, `/frametitle [/H4 ...]` in /RoleMapNS).
    let standard = ["H", "H1", "H2", "H3", "H4", "H5", "H6", "Figure", "Table", "TH", "TD", "TR", "L", "LI", "Link", "P", "Part", "Sect", "Div", "Formula", "Caption"];
    // The maps are often objects of their own (`/RoleMapNS 16 0 R`), so the whole text is read for
    // their entries: a name followed by a standard type, directly (`/frametitle /H4`, a /RoleMap) or
    // in an array (`/section [/H2 11 0 R]`, a /RoleMapNS). Ordinary dictionary keys are not names of
    // structure types.
    let keys = ["S", "Type", "Subtype", "P", "Pg", "K", "Parent", "Role", "Alt", "ActualText", "Lang", "T", "ID", "NS", "Name", "N", "D", "Dest", "A", "O", "Owner", "BaseFont", "Encoding", "Filter", "First", "Last", "Next", "Prev", "Root", "Info"];
    let mut roles: BTreeMap<String, String> = BTreeMap::new();
    if text.contains("/RoleMap") {
        let spaced = text.replace('[', " [ ").replace(']', " ] ").replace('/', " /").replace(['<', '>'], " ");
        let toks: Vec<&str> = spaced.split_whitespace().collect();
        for i in 0..toks.len().saturating_sub(2) {
            let Some(name) = toks[i].strip_prefix('/') else { continue };
            let next = if toks[i + 1] == "[" { toks[i + 2] } else { toks[i + 1] };
            let Some(target) = next.strip_prefix('/') else { continue };
            if !keys.contains(&name) && !standard.contains(&name) && standard.contains(&target) {
                roles.entry(name.to_string()).or_insert(target.to_string());
            }
        }
    }
    let count_role = |targets: &[&str]| -> usize {
        let exact = |n: &str| {
            let pat = format!("/S/{n}");
            compact.match_indices(&pat).filter(|(k, _)| !compact[k + pat.len()..].starts_with(|c: char| c.is_ascii_alphanumeric() || c == '-')).count()
        };
        targets.iter().map(|t| exact(t)).sum::<usize>() + roles.iter().filter(|(_, t)| targets.contains(&t.as_str())).map(|(n, _)| exact(n)).sum::<usize>()
    };
    f.headings = count_role(&["H", "H1", "H2", "H3", "H4", "H5", "H6"]);
    f.tables = count_role(&["Table"]);
    f.header_cells = count_role(&["TH"]);
    f.lists = count_role(&["L"]);
    f.struct_links = count_role(&["Link"]);
    // Figures and their alternate text.
    for pat in ["/S/Figure", "/S /Figure"] {
        for (k, _) in text.match_indices(pat) {
            if text[k + pat.len()..].starts_with(|c: char| c.is_ascii_alphanumeric()) {
                continue;
            }
            f.figures += 1;
            if !dict_around(&text, k).contains("/Alt") {
                f.figures_without_alt += 1;
            }
        }
    }
    // Bookmarks: outline items have a title, a parent and a destination or action.
    for (k, _) in text.match_indices("/Title") {
        let d = dict_around(&text, k);
        if d.contains("/Parent") && (d.contains("/Dest") || d.contains("/A ") || d.contains("/A<<")) {
            f.bookmarks += 1;
        }
    }
    // Annotations.
    for (k, _) in compact.match_indices("/Subtype/Link") {
        f.link_annots += 1;
        if !dict_around(&compact, k).contains("/Contents") {
            f.link_annots_without_contents += 1;
        }
    }
    for (k, _) in compact.match_indices("/Subtype/Screen") {
        if !dict_around(&compact, k).contains("/Contents") {
            f.screen_annots_without_contents += 1;
        }
    }
    for (k, _) in compact.match_indices("/Type/Page") {
        if compact[k + 10..].starts_with('s') {
            continue;
        }
        let d = dict_around(&compact, k);
        if d.contains("/Annots") && !d.contains("/Tabs/S") {
            f.pages_with_annots_without_tabs += 1;
        }
    }
    // Fonts: a Unicode map makes the text readable aloud and copyable.
    for (k, _) in compact.match_indices("/Type/Font") {
        // Not /Type/FontDescriptor.
        if compact[k + 10..].starts_with(|c: char| c.is_ascii_alphabetic()) {
            continue;
        }
        let d = dict_around(&compact, k);
        if d.contains("/Subtype/CIDFontType") || d.contains("/Subtype/Type3") {
            continue;
        }
        f.fonts += 1;
        let standard_14 = d.contains("/Subtype/Type1") && !d.contains("/FontDescriptor");
        if !d.contains("/ToUnicode") && !standard_14 {
            f.fonts_without_unicode += 1;
        }
    }
    f
}

/// The pages of an open document, for the checks that need its text, media and animations.
pub fn page_summaries(doc: &crate::pdf::DocInfo) -> Vec<PageSummary> {
    doc.pages
        .iter()
        .map(|p| PageSummary {
            has_text: !p.text_full.trim().is_empty(),
            links: p.links.len(),
            media: p.media.len() + p.videos.len(),
            animations: p.animations.len(),
            moving_without_pause: p.animations.iter().filter(|a| a.autoplay && a.end != crate::pdf::AnimEnd::Stop && a.controls.is_none()).count(),
        })
        .collect()
}

fn issue(text: impl Into<String>, fix: impl Into<String>) -> Issue {
    Issue { text: text.into(), fix: fix.into(), page: None, source: None }
}

/// Pages, as "1, 3, 7" (from 1), shortened past a dozen.
fn page_list(pages: &[usize]) -> String {
    let mut s: Vec<String> = pages.iter().take(12).map(|p| (p + 1).to_string()).collect();
    if pages.len() > 12 {
        s.push(format!("and {} more", pages.len() - 12));
    }
    s.join(", ")
}

/// Checks `pdf` (with its pages as the engine read them, and how many bookmarks it has) and, when
/// there is one, its `.tex` source; returns every criterion, worst first within each principle.
pub fn evaluate(pdf: &Path, pages: &[PageSummary], bookmarks: usize, source: Option<&Path>) -> Report {
    let bytes = std::fs::read(pdf).unwrap_or_default();
    let f = facts(&bytes);
    let findings = source.map(crate::access::check).unwrap_or_default();
    let from_source = |codes: &[&str]| -> Vec<Issue> {
        findings
            .iter()
            .filter(|x| codes.contains(&x.code.as_str()) && x.severity >= crate::access::Severity::Warning)
            .map(|x| {
                let file = x.file.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                let what = if x.target.is_empty() { x.subject.clone() } else { x.target.clone() };
                Issue {
                    text: format!("{file}, line {}: {what}: {}", x.line, x.message),
                    fix: x.fix.clone(),
                    page: None,
                    source: Some((x.file.clone(), x.line, x.col, x.target.chars().count())),
                }
            })
            .collect()
    };
    let n = pages.len();
    let has_source = source.is_some();
    let rebuild = if has_source { "Fix it in the source, then Build tagged copy (in the studio's Accessibility tab)" } else { "Make the PDF again with tagging on (LaTeX: \\DocumentMetadata{...tagging=on}; Beamer slides: TeXora's Build tagged copy)" };
    let mut out: Vec<Criterion> = vec![];
    let mut add = |id: &'static str, name: &'static str, principle: &'static str, status: Status, summary: String, issues: Vec<Issue>| {
        out.push(Criterion { id, name, principle, status, summary, issues });
    };

    // Perceivable.
    {
        let mut issues = from_source(&["alt"]);
        let (status, summary) = if f.tagged {
            if f.figures_without_alt > 0 {
                issues.insert(0, issue(format!("{} of {} pictures in the PDF have no alternate text", f.figures_without_alt, f.figures), "give each picture alt={...} (or artifact if it is decoration), then make the PDF again"));
            }
            let status = if issues.is_empty() { Status::Pass } else { Status::Fail };
            let summary = if f.figures == 0 && f.images == 0 { "No pictures found.".into() } else { format!("{} tagged picture(s), {} with alternate text.", f.figures, f.figures - f.figures_without_alt) };
            (status, summary)
        } else if f.images > 0 || !issues.is_empty() {
            issues.insert(0, issue(format!("the PDF has {} picture(s) and no tags, so none has alternate text a screen reader can find", f.images), rebuild));
            (Status::Fail, "Pictures without alternate text (the PDF is not tagged).".into())
        } else {
            (Status::NotApplicable, "No pictures found.".into())
        };
        add("1.1.1", "Non-text content", "Perceivable", status, summary, issues);
    }
    {
        let mut issues = from_source(&["transcript"]);
        let media_pages: Vec<usize> = pages.iter().enumerate().filter(|(_, p)| p.media > 0).map(|(i, _)| i).collect();
        let status = if !issues.is_empty() {
            Status::Fail
        } else if media_pages.is_empty() {
            Status::NotApplicable
        } else {
            Status::Review
        };
        if !media_pages.is_empty() && issues.is_empty() {
            for &p in &media_pages {
                issues.push(Issue { page: Some(p), ..issue(format!("page {}: a video or sound clip", p + 1), "check it has a text alternative (caption or transcript), and synchronized captions when it has speech") });
            }
        }
        let summary = if media_pages.is_empty() && issues.is_empty() { "No video or audio found.".to_string() } else { format!("Video or audio on page(s) {}.", page_list(&media_pages)) };
        add("1.2.1–1.2.3", "Audio and video", "Perceivable", status, summary, issues);
    }
    {
        let mut issues = from_source(&["table"]);
        let mut status = Status::Pass;
        if !f.tagged {
            issues.insert(0, issue("the PDF is not tagged: headings, lists and tables are only visual, a screen reader cannot tell them apart", rebuild));
            status = Status::Fail;
        } else {
            if f.tables > 0 && f.header_cells == 0 {
                issues.push(issue(format!("{} table(s) without header cells", f.tables), "mark the first row as headers: \\AccHeaderRows{1} before the table (or \\tagpdfsetup{table/header-rows={1}})"));
                status = Status::Fail;
            }
            if f.headings == 0 && n > 1 {
                issues.push(issue("no headings in the tags", "give slides titles (\\frametitle) and documents sections, so a screen reader can move between them"));
                status = status.min(Status::Review);
            }
            if !issues.is_empty() && status == Status::Pass {
                status = Status::Review;
            }
        }
        let summary = if f.tagged { format!("Tagged: {} heading(s), {} list(s), {} table(s) with {} header cell(s).", f.headings, f.lists, f.tables, f.header_cells) } else { "Not tagged.".into() };
        add("1.3.1", "Info and relationships", "Perceivable", status, summary, issues);
    }
    add(
        "1.3.2",
        "Meaningful sequence",
        "Perceivable",
        if f.tagged { Status::Review } else { Status::Fail },
        if f.tagged { "Tags give the reading order; a program cannot tell whether it makes sense.".into() } else { "Not tagged: the reading order is whatever the PDF happens to hold.".into() },
        if f.tagged { vec![issue("listen to it with a screen reader, or look at the order in Acrobat's Reading Order tool", "if the order is wrong, change the order of the source")] } else { vec![issue("the PDF has no reading order", rebuild)] },
    );
    {
        let issues = from_source(&["linkcolour"]);
        let status = if !issues.is_empty() { Status::Fail } else { Status::Review };
        let summary = if has_source { "Link colours measured in the source; meaning carried by colour in the content is for a person to check.".into() } else { "Check that nothing is told by colour alone (\"the red items\"), and that links stand out by more than colour.".into() };
        add("1.4.1", "Use of colour", "Perceivable", status, summary, issues);
    }
    {
        let issues = from_source(&["contrast"]);
        let status = if !issues.is_empty() { Status::Fail } else { Status::Review };
        let summary = if has_source { "Colour pairs set in the source measured; text on pictures and theme colours are for a person to check.".into() } else { "Check that text has 4.5:1 contrast with its background (3:1 for large text).".into() };
        add("1.4.3", "Contrast (minimum)", "Perceivable", status, summary, issues);
    }
    {
        let without_text: Vec<usize> = pages.iter().enumerate().filter(|(_, p)| !p.has_text).map(|(i, _)| i).collect();
        let issues: Vec<Issue> = without_text
            .iter()
            .map(|&p| Issue { page: Some(p), ..issue(format!("page {} has no text: a picture of text, a scan, or a slide of pictures only", p + 1), "if it shows words, make them real text (or run OCR on a scan); a slide of pictures needs alt text") })
            .collect();
        let status = if without_text.is_empty() { Status::Pass } else { Status::Review };
        let summary = if without_text.is_empty() { format!("Every page has text ({n} page(s)).") } else { format!("{} page(s) without text: {}.", without_text.len(), page_list(&without_text)) };
        add("1.4.5", "Images of text", "Perceivable", status, summary, issues);
    }

    // Operable.
    {
        let mut issues = from_source(&["motion"]);
        let moving: Vec<usize> = pages.iter().enumerate().filter(|(_, p)| p.moving_without_pause > 0).map(|(i, _)| i).collect();
        for &p in &moving {
            issues.push(Issue { page: Some(p), ..issue(format!("page {}: an animation that starts by itself and repeats, with no pause button", p + 1), "add controls to the animation, or drop autoplay") });
        }
        let animated = pages.iter().any(|p| p.animations > 0 || p.media > 0);
        let status = if !issues.is_empty() { Status::Fail } else if animated { Status::Pass } else { Status::NotApplicable };
        let summary = if animated { "Animations and clips found; moving content needs a way to pause it.".into() } else { "Nothing moves.".into() };
        add("2.2.2", "Pause, stop, hide", "Operable", status, summary, issues);
    }
    {
        let mut issues = from_source(&["doctitle"]);
        let mut status = Status::Pass;
        if f.title.is_none() {
            issues.insert(0, issue("the PDF has no title", "set \\title{...} and pdftitle={...} in \\hypersetup; in other programs, the document properties' Title"));
            status = Status::Fail;
        } else if !f.display_title {
            issues.push(issue("the title is there, but the viewer shows the file name instead", "set pdfdisplaydoctitle=true in \\hypersetup (PDF/UA asks for it)"));
            status = Status::Fail;
        }
        let summary = match &f.title { Some(t) => format!("Title: \u{201c}{t}\u{201d}{}.", if f.display_title { "" } else { ", not shown in the window" }), None => "No title.".into() };
        add("2.4.2", "Page titled", "Operable", if issues.is_empty() { status } else { Status::Fail }, summary, issues);
    }
    {
        let mut issues = from_source(&["link"]);
        let links: usize = pages.iter().map(|p| p.links).sum();
        if f.link_annots_without_contents > 0 && f.tagged {
            issues.push(issue(format!("{} of {} links have no description (Contents) for assistive technology", f.link_annots_without_contents, f.link_annots), "LaTeX's tagging adds them; with tagging on and a current LaTeX this goes away"));
        }
        let status = if !issues.is_empty() { Status::Fail } else if links > 0 { Status::Review } else { Status::NotApplicable };
        let summary = if links > 0 { format!("{links} link(s); check that each link's text says where it goes.") } else { "No links.".into() };
        add("2.4.4", "Link purpose", "Operable", status, summary, issues);
    }
    {
        // The checked PDF's own bookmarks, or the open document's (the engine reads those).
        let bookmarks = bookmarks.max(f.bookmarks);
        let (status, summary, issues) = if n <= 3 {
            (Status::NotApplicable, "Short document.".to_string(), vec![])
        } else if bookmarks > 0 {
            (Status::Pass, format!("{bookmarks} bookmark(s) to move through the document."), vec![])
        } else {
            // WCAG asks for more than one way to find content; bookmarks are the usual one in a PDF,
            // a contents slide or page is another, so a person decides.
            (Status::Review, "No bookmarks.".to_string(), vec![issue(format!("{n} pages and no bookmarks (outline)"), "use \\section in the source (hyperref turns sections into bookmarks), or give a contents slide or page")])
        };
        add("2.4.5", "Multiple ways", "Operable", status, summary, issues);
    }
    {
        let issues = from_source(&["title"]);
        let status = if !issues.is_empty() { Status::Fail } else if f.tagged && f.headings > 0 { Status::Pass } else { Status::Review };
        let summary = if has_source { "Slide and section titles checked in the source.".into() } else { "Check that headings and slide titles describe their content.".into() };
        add("2.4.6", "Headings and labels", "Operable", status, summary, issues);
    }

    // Understandable.
    {
        let mut issues = from_source(&["language"]);
        if f.lang.is_none() {
            issues.insert(0, issue("the PDF does not say what language it is in", "set the language: \\DocumentMetadata{lang=en-US}, or babel with pdflang=en-US in \\hypersetup"));
        }
        let summary = match &f.lang { Some(l) => format!("Language: {l}."), None => "No language.".into() };
        add("3.1.1", "Language of page", "Understandable", if issues.is_empty() { Status::Pass } else { Status::Fail }, summary, issues);
    }

    // Robust.
    {
        let mut issues = vec![];
        if f.tagged && f.pages_with_annots_without_tabs > 0 {
            issues.push(issue(format!("{} page(s) with links or buttons have no tab order that follows the structure", f.pages_with_annots_without_tabs), "LaTeX's tagging sets it (/Tabs /S); make the PDF again with tagging on"));
        }
        if f.screen_annots_without_contents > 0 {
            issues.push(issue(format!("{} media or animation annotation(s) without a description", f.screen_annots_without_contents), "with accessgate loaded and tagging on, TeXora gives animate's annotations one; give clips alt={...}"));
        }
        let status = if !f.tagged { Status::Fail } else if issues.is_empty() { Status::Pass } else { Status::Fail };
        if !f.tagged {
            issues.insert(0, issue("not tagged: links, buttons and media have no role a screen reader can announce", rebuild));
        }
        add("4.1.2", "Name, role, value", "Robust", status, if f.tagged { "Annotations checked.".into() } else { "Not tagged.".into() }, issues);
    }

    // What PDF/UA asks of the file itself.
    {
        let mut issues = vec![];
        if !f.tagged {
            issues.push(issue("not tagged", rebuild));
        } else if !f.marked {
            issues.push(issue("tagged, but not marked as tagged (MarkInfo /Marked true)", "make the PDF again with a current LaTeX and tagging on"));
        }
        if f.ua_part.is_none() {
            issues.push(issue("does not declare PDF/UA", "\\DocumentMetadata{pdfstandard=ua-2,...} declares it (TeXora's tagged copy does)"));
        }
        if f.blocks_assistive {
            issues.push(issue("encrypted with the permission for assistive technology turned off", "remove the encryption, or allow access for accessibility"));
        } else if f.encrypted {
            issues.push(issue("encrypted: TeXora cannot read all of its structure", "check it unencrypted, or with veraPDF"));
        }
        if f.fonts_without_unicode > 0 {
            issues.push(issue(format!("{} of {} fonts have no Unicode map: words may be read out or copied wrongly", f.fonts_without_unicode, f.fonts), "LaTeX's tagging adds the maps; with pdfLaTeX, also \\input{glyphtounicode} and \\pdfgentounicode=1"));
        }
        let status = if !f.tagged || f.blocks_assistive { Status::Fail } else if issues.is_empty() { Status::Pass } else { Status::Review };
        let summary = format!(
            "{}{}.",
            if f.tagged { "Tagged" } else { "Not tagged" },
            f.ua_part.map(|p| format!(", declares PDF/UA-{p}; validate it with veraPDF")).unwrap_or_default()
        );
        add("PDF/UA", "Tagged PDF and PDF/UA", "PDF", status, summary, issues);
    }

    Report { pdf: pdf.to_path_buf(), facts: f, criteria: out, source: source.map(Path::to_path_buf) }
}

/// veraPDF, when it is installed: the command on the PATH, or where its installer puts it.
pub fn verapdf() -> Option<PathBuf> {
    if let Some(p) = crate::latex::find_tool("verapdf") {
        return Some(p);
    }
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    [
        PathBuf::from("/Applications/veraPDF/verapdf"),
        home.join("verapdf/verapdf"),
        home.join("veraPDF/verapdf"),
        PathBuf::from("C:\\Program Files\\veraPDF\\verapdf.bat"),
        PathBuf::from("/opt/verapdf/verapdf"),
    ]
    .into_iter()
    .find(|p| p.is_file())
}

/// One failed veraPDF rule.
#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    pub clause: String,
    pub test: String,
    pub description: String,
    pub failed: usize,
}

/// What veraPDF said: compliant or not, against which profile, and the failed rules.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Validation {
    pub profile: String,
    pub compliant: bool,
    pub rules: Vec<Rule>,
    pub error: Option<String>,
}

/// Reads a veraPDF machine-readable report (`--format mrr`).
pub fn parse_mrr(xml: &str) -> Validation {
    let attr = |s: &str, name: &str| -> Option<String> {
        let k = s.find(&format!("{name}=\""))? + name.len() + 2;
        Some(s[k..].split('"').next()?.to_string())
    };
    let mut v = Validation::default();
    if let Some(k) = xml.find("<validationReport") {
        let tag = &xml[k..xml[k..].find('>').map_or(xml.len(), |e| k + e)];
        v.profile = attr(tag, "profileName").unwrap_or_default();
        v.compliant = attr(tag, "isCompliant").as_deref() == Some("true");
    } else {
        v.error = Some(xml.split("<errorMessage>").nth(1).and_then(|e| e.split('<').next()).unwrap_or("veraPDF gave no report").to_string());
    }
    for part in xml.split("<rule ").skip(1) {
        let head = part.split('>').next().unwrap_or("");
        if attr(head, "status").as_deref() != Some("failed") {
            continue;
        }
        let description = part.split("<description>").nth(1).and_then(|d| d.split("</description>").next()).unwrap_or("").trim().replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&amp;", "&");
        v.rules.push(Rule {
            clause: attr(head, "clause").unwrap_or_default(),
            test: attr(head, "testNumber").unwrap_or_default(),
            description,
            failed: attr(head, "failedChecks").and_then(|n| n.parse().ok()).unwrap_or(0),
        });
    }
    v
}

/// Runs veraPDF on `pdf` against the PDF/UA part it declares (2 when it declares none).
pub fn validate(verapdf: &Path, pdf: &Path, ua_part: Option<u8>) -> Validation {
    let flavour = if ua_part == Some(1) { "ua1" } else { "ua2" };
    match std::process::Command::new(verapdf).args(["--flavour", flavour, "--format", "mrr"]).arg(pdf).output() {
        Ok(o) => parse_mrr(&String::from_utf8_lossy(&o.stdout)),
        Err(e) => Validation { error: Some(format!("veraPDF could not be run: {e}")), ..Default::default() },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pages(n: usize) -> Vec<PageSummary> {
        (0..n).map(|_| PageSummary { has_text: true, ..Default::default() }).collect()
    }

    #[test]
    fn strings_and_verapdf_reports_are_read() {
        assert_eq!(pdf_string("(A \\(test\\) title)"), Some("A (test) title".into()));
        assert_eq!(pdf_string("<FEFF00540065>"), Some("Te".into()));
        let xml = r#"<report><validationReport profileName="PDF/UA-2 + Tagged PDF validation profile" isCompliant="false"><details/><rule specification="ISO 14289-2:2024" clause="8.2.5.27" testNumber="1" status="failed" failedChecks="2"><description>When present, the Caption element shall be the first or the last child</description></rule><rule clause="5" testNumber="1" status="passed" passedChecks="1"></rule></validationReport></report>"#;
        let v = parse_mrr(xml);
        assert_eq!((v.profile.as_str(), v.compliant), ("PDF/UA-2 + Tagged PDF validation profile", false));
        assert_eq!(v.rules, vec![Rule { clause: "8.2.5.27".into(), test: "1".into(), description: "When present, the Caption element shall be the first or the last child".into(), failed: 2 }]);
    }

    /// A small hand-made PDF: untagged, one picture, no language, no title.
    #[test]
    fn an_untagged_pdf_fails_where_it_should() {
        let dir = std::env::temp_dir().join(format!("texora-pdfcheck-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("plain.pdf");
        std::fs::write(&p, "%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj\n3 0 obj<</Type/Page/Parent 2 0 R/Annots[5 0 R]>>endobj\n4 0 obj<</Type/XObject/Subtype/Image/Width 1/Height 1>>endobj\n5 0 obj<</Type/Annot/Subtype/Link/Rect[0 0 1 1]>>endobj\ntrailer<</Root 1 0 R>>\n%%EOF\n").unwrap();
        let mut pg = pages(5);
        pg[2].has_text = false;
        pg[1].links = 1;
        let r = evaluate(&p, &pg, 0, None);
        let st = |id: &str| r.criteria.iter().find(|c| c.id == id).map(|c| c.status).unwrap();
        assert!(!r.facts.tagged && r.facts.images == 1 && r.facts.link_annots == 1);
        assert_eq!(st("1.1.1"), Status::Fail);
        assert_eq!(st("1.3.1"), Status::Fail);
        assert_eq!(st("1.4.5"), Status::Review);
        assert_eq!(r.criteria.iter().find(|c| c.id == "1.4.5").unwrap().issues[0].page, Some(2), "the page without text is named");
        assert_eq!(st("2.4.2"), Status::Fail);
        assert_eq!(st("2.4.5"), Status::Review, "five pages and no bookmarks: a person decides");
        assert_eq!(st("3.1.1"), Status::Fail);
        assert_eq!(st("PDF/UA"), Status::Fail);
        assert_eq!(st("1.2.1–1.2.3"), Status::NotApplicable);
        assert!(r.criteria.iter().all(|c| c.status != Status::Fail || !c.issues.is_empty()), "every failure says what and how to fix");
    }

    /// A tagged article made by LaTeX, where it is installed: tags, language, title, PDF/UA, alt text,
    /// headings and table headers are read from the PDF.
    #[test]
    fn a_tagged_article_is_read() {
        let Some(lua) = crate::latex::find_tool("lualatex") else { return };
        let dir = std::env::temp_dir().join(format!("texora-pdfcheck-tagged-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let tex = dir.join("t.tex");
        std::fs::write(&tex, "\\DocumentMetadata{lang=en-US,pdfstandard=ua-2,tagging=on}\n\\documentclass{article}\n\\usepackage{graphicx,booktabs,hyperref}\n\\hypersetup{pdftitle={A test},pdfdisplaydoctitle=true}\n\\title{A test}\n\\begin{document}\n\\section{One}\nText.\n\\includegraphics[width=2cm,alt={A grey box}]{example-image}\n\\tagpdfsetup{table/header-rows={1}}\n\\begin{tabular}{ll}\\toprule A & B\\\\\\midrule 1 & 2\\\\\\bottomrule\\end{tabular}\n\\end{document}\n").unwrap();
        let ok = std::process::Command::new(lua).current_dir(&dir).args(["-interaction=nonstopmode", "t.tex"]).output().is_ok_and(|o| o.status.success());
        if !ok || !dir.join("t.pdf").exists() {
            return;
        }
        let f = facts(&std::fs::read(dir.join("t.pdf")).unwrap());
        assert!(f.tagged && f.marked, "{f:?}");
        assert_eq!(f.lang.as_deref(), Some("en-US"));
        assert_eq!(f.title.as_deref(), Some("A test"));
        assert!(f.display_title && f.ua_part == Some(2), "{f:?}");
        assert!(f.figures >= 1 && f.figures_without_alt == 0, "{f:?}");
        assert!(f.headings >= 1 && f.tables == 1 && f.header_cells >= 2, "{f:?}");
        let r = evaluate(&dir.join("t.pdf"), &pages(1), 0, None);
        for id in ["1.1.1", "2.4.2", "3.1.1", "1.3.1"] {
            assert_eq!(r.criteria.iter().find(|c| c.id == id).unwrap().status, Status::Pass, "{id}: {:?}", r.criteria.iter().find(|c| c.id == id));
        }
    }
}
