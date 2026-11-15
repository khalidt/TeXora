# Accessible slides and documents

A guide to making presentations and documents that work with screen readers and other assistive
technology, in TeXora and in any other LaTeX editor: what is supported, how each kind of content is
written, how tagging works, what TeXora checks and what it does not, and how to validate the result.
For short step-by-step recipes, see [How to make an accessible PDF](how-to.md).

It applies to TeX Live 2024 to 2026, with `ltx-talk` 0.4.6 (as TeX Live 2026 ships it) or 0.6.12.

<details open>
<summary><b>Contents</b></summary>

1. [Does my PDF have tags?](#1-does-my-pdf-have-tags)
2. [What tagging is](#2-what-tagging-is)
3. [What is supported](#3-what-is-supported)
4. [Setting up](#4-setting-up)
5. [Beamer slides in TeXora, step by step](#5-beamer-slides-in-texora-step-by-step)
6. [Writing each kind of content](#6-writing-each-kind-of-content)
7. [Checking the source](#7-checking-the-source)
8. [The tagged copy](#8-the-tagged-copy)
9. [Validating the PDF](#9-validating-the-pdf)
10. [Documents that are not slides](#10-documents-that-are-not-slides)
11. [Slides tagged directly: ltx-talk](#11-slides-tagged-directly-ltx-talk)
12. [Other LaTeX editors](#12-other-latex-editors)
13. [Command line and CI](#13-command-line-and-ci)
14. [Your own macros](#14-your-own-macros)
15. [What is not checked](#15-what-is-not-checked)
16. [Limits](#16-limits)
17. [Reference](#17-reference)

</details>

## 1. Does my PDF have tags?

| What you compile | Where | Tags in the PDF |
|---|---|---|
| Beamer slides, with **Compile** (or in any editor) | anywhere | **No.** Beamer refuses LaTeX's tagging. Alt text, captions and transcripts stay in the source; captions and transcripts are visible, and all of it is used by the tagged copy. |
| Beamer slides, with **Build tagged copy** | TeXora, TeX Live 2026 | **Yes**, in a second file, `talk.acc.pdf`, made through the `ltx-talk` class. `talk.pdf` stays as it is. |
| An article, report or book with `\DocumentMetadata{...tagging=on}` | any editor | **Yes**, on every compile. |
| An article without `\DocumentMetadata`, with **Build tagged copy** | TeXora | **Yes**, in `talk.acc.pdf`. |
| Slides made with `\documentclass{ltx-talk}` and `\DocumentMetadata` | any editor, TeX Live 2026 | **Yes**, on every compile. |

"Tagged" is not "validated": TeXora checks that the copy has tags, a language and alt text on every
picture, not that it meets PDF/UA. Validate it with veraPDF ([section 9](#9-validating-the-pdf)).

Present from the normal Beamer PDF, and give out (or publish) the tagged copy to anyone who uses a
screen reader. Documents with `\DocumentMetadata` need no second file.

## 2. What tagging is

A PDF without tags is a picture of text: a screen reader can guess at the words, but not which line is
a heading, where a list starts, what a picture shows or in which order to read. A **tagged** PDF holds,
besides the pages, a structure tree:

```text
Document
├─ Sect (a slide)
│  ├─ H4  "Describe every picture"
│  ├─ Figure  Alternate text: "A television test card: eight vertical colour bars, ..."
│  └─ L (list)
│     ├─ LI  "Say what matters, not 'image of'."
│     └─ LI  "One or two sentences; put details in the text."
└─ Sect
   ├─ H4  "Tables with a header row"
   └─ Table
      ├─ TR  TH "Content" | TH "What to give it" | TH "WCAG"
      └─ TR  TD "Picture" | TD "alt={...} or artifact" | TD "1.1.1"
```

(The start of the tagged copy of the **Accessible Lecture** template, as `show-pdf-tags` prints it.)
It also holds the document's **language** and **title**, and marks decoration as **artifact** so it is
skipped. A PDF that does all this and says so follows **PDF/UA** (ISO 14289): LaTeX writes PDF/UA-2
(ISO 14289-2, on PDF 2.0) from LaTeX 2025-06, and PDF/UA-1 with the test phase before.

Slide titles are H4 in `ltx-talk` with no H1 to H3 above them. PDF/UA-2 allows that; PDF/UA-1 has a rule against skipped heading levels (7.4.2), which is one reason to
validate a file against the standard it declares.

LaTeX makes the tags itself when the document starts with `\DocumentMetadata{...}`. TeXora adds three
things around that, each of which works on its own:

| Layer | What it is | Works without |
|---|---|---|
| 1. Registry | `accessgate-registry.toml`: data saying which commands carry structure and what they need | LaTeX, TeXora |
| 2. Source | `accessgate.sty`: adds tags where tagging is on, typesets as before where it is off | tagging, TeXora |
| 3. Checking | the source check, the tagged copy and the check of its tags | tagging |

## 3. What is supported

| Content | Write it as | In the tagged PDF | TeXora checks |
|---|---|---|---|
| Slide titles | `\begin{frame}{Title}` (Beamer), `\frametitle{...}` | heading (H4 in `ltx-talk`) | every slide has one |
| Sections (documents) | `\section`, `\subsection`, ... | headings, in sections | |
| Paragraphs | plain text | paragraphs | |
| Lists | `itemize`, `enumerate`, `description` | lists with items and labels | |
| Pictures | `\includegraphics[alt={...}]`, `\includeimage[alt={...}]` | Figure with alternate text | alt or artifact given |
| Decoration | `[artifact]`, `\AccArtifact{...}` | artifact (skipped) | |
| Drawings (TikZ, PGFPlots) | `\AccFigure{description}{...}` | Figure with alternate text | |
| Animations (`animate`) | `\AccFigure{...}{...}`, `alttext=`, `controls` | Figure with alternate text | wrapped; a way to pause when looping |
| Reduced motion | `\AccMotion{animation}{still}`, option `still` | the still picture | |
| Videos and GIFs | `\includevideo[...][alt=..., caption=...]` | Figure with alternate text (the poster is a link), the caption as a paragraph, links to transcripts | alt; caption or transcript; autoplay with loop |
| Transcripts | `transcript={...}`, `transcriptfile=...`, `\AccTranscripts` | text and links | |
| Tables | `tabular` with a rule under the header, `\AccHeaderRows{1}` | table, rows, header cells (TH), cells | header row marked |
| Maths | `$...$`, `\[...\]`, `equation` | Formula (with LuaLaTeX also MathML) | |
| Links | `\href{url}{text that says where}` | Link | link text; link colour |
| Footnotes, references | `\footnote`, `\cite`, `thebibliography` | notes and references, linked | |
| Colours | `\definecolor`, `\setbeamercolor`, `\hypersetup` | (not in the tags) | contrast of the pairs the source sets |
| Language | `\DocumentMetadata{lang=...}`, babel, polyglossia, `pdflang` | the PDF's language | set |
| Title | `\title{...}`, `pdftitle` | the PDF's title | set |

Engines: **LuaLaTeX** gives the most complete tags and MathML for maths; use it for PDF/UA-2.
**pdfLaTeX** tags too, but can leave text neither tagged nor marked as artifact, which fails PDF/UA-2
(veraPDF rule 8.2.2-1) where the same document built with LuaLaTeX passes.
**XeLaTeX**'s tags are incomplete. TeXora's tagged copy uses LuaLaTeX.

## 4. Setting up

**In TeXora** nothing needs to be installed: `accessgate`, `texoravideo` and `texoraimage` are built in
and found when TeXora compiles. Only LaTeX itself is needed:

| LaTeX | What works |
|---|---|
| any | the source check; `\AccFigure`, `\AccArtifact`, `\AccMotion` and the `\includevideo` keys compile everywhere |
| 2021-11 or later (TeX Live 2022) | `alt=` on `\includegraphics` is accepted; without tagging it is ignored, without error, with all three engines |
| 2023 or later (TeX Live 2023) | tagging in the test phase (`testphase={phase-III}`), PDF/UA-1 |
| 2025-06 or later (TeX Live 2025) | `tagging=on`, PDF/UA-2, `\AccHeaderRows` |
| TeX Live 2026 with `ltx-talk` | tagged copies of Beamer slides; `ltx-talk` slides |

**ltx-talk versions.** TeX Live 2026 shipped `ltx-talk` 0.4.6 (2026-02-18, needs LaTeX 2025-11-01).
Version 0.6.12 (2026-11-12) needs **LaTeX 2026-06-01**. Updating
`ltx-talk` alone (`tlmgr update ltx-talk`) therefore leaves a class that refuses to load ("The
ltx-talk class requires LaTeX 2026-06-01"); update LaTeX with it:

```sh
sudo tlmgr update --self --all     # MacTeX and other system-wide TeX Live; without sudo where it is yours
```

TeXora works with both: it reads the installed version, names it in the result, and uses each
version's own commands where they work.

| | 0.4.6 | 0.6.12 |
|---|---|---|
| Contents page | breaks the tag structure; TeXora lists the contents plainly instead | its own |
| `\againframe` | not there; repeats nothing in the copy | repeats the frame |
| `label=` on a frame | not there; TeXora makes it do nothing | used by `\againframe` |
| `\appendix` | breaks the tag structure; left out | its own |
| `amssymb` with LuaLaTeX | clashes ("Command \ulcorner already defined"); TeXora leaves it out, its symbols come with `ltx-talk` | the same |
| `\begin{frame}{Title}` | the title is typeset as text | the same: use `\frametitle` |

To see what is installed: `kpsewhich ltx-talk.cls` (a path when it is there) and
`tlmgr info ltx-talk`.

**The quickest start**: **New LaTeX presentation** has four accessible slide templates and two tagged
documents. Each is a guide: every slide or section shows one technique and uses it, and the source
check finds nothing to fix in them.

| Template | Shows |
|---|---|
| **Accessible Lecture** | alt text, decoration, described drawings, a table with header rows, links with enough contrast, maths, language |
| **Accessible Media** | a video and a GIF with alt text, a caption, a transcript in the slides and one in its own file, a loop that starts on a click |
| **Accessible Motion** | animations that are described and can be paused, click-by-click steps, decoration, a still version |
| **Tagged Slides** | slides tagged directly by LaTeX with `ltx-talk` |
| **Tagged Article** (not slides) | a document tagged by LaTeX alone: headings, lists, figure, table, maths, footnote, references |
| **Media Handout** (not slides) | a tagged document with a video, a GIF, a picture and an animation, for any editor |

Their tagged versions validate as PDF/UA-2 with veraPDF (Tagged Article and Tagged Slides when built
with LuaLaTeX).

## 5. Beamer slides in TeXora, step by step

1. **Open your presentation**, or create one from **Accessible Lecture**.
2. **Load the gate** in the preamble. `texoravideo` and `texoraimage` load it already; otherwise:
   ```latex
   \usepackage{accessgate}
   ```
3. **Set the language and the title**:
   ```latex
   \usepackage[english]{babel}                 % or \usepackage[ngerman]{babel}, polyglossia, ...
   \hypersetup{pdflang=en-US,pdftitle={My Talk}}
   \title{My Talk}
   ```
4. **Describe the content**, kind by kind: [section 6](#6-writing-each-kind-of-content).
5. **Check**: **Tools > Check accessibility**. The **Accessibility** tab of the studio's message panel
   lists what is missing, line by line; click a line to go there. Fix until nothing is left to fix.
6. **Compile and present** as usual. This PDF has no tags (Beamer), and does not need them on the
   projector.
7. **Build tagged copy** (in the Accessibility tab, or **Tools**). Two choices next to the button:
   - **One page per slide**: each frame on one page with all its steps, instead of one page per step
     ([overlays](#text-slide-titles-and-overlays)).
   - **Still pictures**: every `\AccMotion` shows its still picture instead of the animation.

   TeXora writes `talk.acc.pdf`, checks its tags and says, for example: *Tagged copy (not validated)
   made with LuaLaTeX and the ltx-talk class 0.4.6, without Beamer's theme: talk.acc.pdf (2 pictures,
   2 with alt text; language en-US). What the copy changed: talk.acc-losses.txt.* **Open the tagged
   copy** shows it.
8. **Read `talk.acc-losses.txt`** and look through the copy: the conversion is source to source
   ([section 8](#8-the-tagged-copy)).
9. **Validate** it with veraPDF against PDF/UA-2 ([section 9](#9-validating-the-pdf)).
10. **Give out `talk.acc.pdf`**, or publish it beside the slides.

## 6. Writing each kind of content

### Text, slide titles and overlays

Give **every** frame a title: a screen reader moves from slide to slide by them. The title slide
counts as titled when it shows `\titlepage`, `\maketitle` or `\inserttitle`.

```latex
\begin{frame}{Results}            % Beamer; in ltx-talk: \frametitle{Results} inside the frame
  Plain text, lists and blocks are read in the order they are written.
\end{frame}
```

**Overlays.** A frame with steps (`\pause`, `\only<2>`, `\item<2->`) becomes several pages, one per
step, in the Beamer PDF and in the tagged copy alike. Content that stays on screen is on every one of
those pages, so a screen reader reads it again on each step. For a copy to read through, choose
**One page per slide** (`--handout`): each frame becomes one page with all its steps (the `handout`
mode of `ltx-talk`; the sample lecture goes from 23 pages to 14). Beamer's own `handout` class option
does the same for the Beamer PDF.

### Lists

`itemize`, `enumerate` and `description` become lists with items; nothing to add. Keep bullets as
lists rather than lines that start with a dash.

### Pictures

```latex
\includegraphics[width=5cm,alt={Bar chart: sales doubled from 2024 to 2026}]{chart.pdf}
\includeimage[width=3cm,alt={Team photo, five people}]{team.heic}   % AVIF, WebP, HEIC, JPEG XL
\includegraphics[artifact]{divider.png}                             % decoration
```

- Say what matters: the finding of a chart, who is in a photo. Not "image of".
- One or two sentences; longer explanations belong in the text.
- `alt=` on `\includegraphics` is known to LaTeX from 2021-11 (TeX Live 2022): there it is accepted
  everywhere and simply unused where the document is not tagged. With an older LaTeX it stops with an
  unknown-key error; use `\includeimage`, or `\AccIncludeGraphics`, which pass `alt` on only where
  LaTeX knows it.

### Drawings

TikZ, PGFPlots and anything else without an `alt` key:

```latex
\AccFigure{A flow chart: Write, then Check, then Build; Check can go back to Write}{%
  \begin{tikzpicture} ... \end{tikzpicture}}
```

### Decoration

Lines, background shapes, a spinner: anything that carries no meaning.

```latex
\AccArtifact{\textcolor{teal}{\rule{2cm}{2pt}}}
```

A logo that names an organisation is not decoration: give it alt text.

### Animations

```latex
\AccFigure{A pendulum swinging left and right, about once a second}{%
  \AccMotion{%
    \begin{animateinline}[controls,loop,alttext={A pendulum swinging}]{16}
      \multiframe{17}{rA=-32+4}{\pendulum{\rA}}
    \end{animateinline}}%
  {\pendulum{0}}}                              % the still picture
```

- `\AccFigure` makes the animation one described Figure; its frames are not read out.
- `alttext=` is animate's own key: the description on its play button (animate's default is
  "animation by animate"). `accessgate` also gives animate's hidden helper annotation the
  description PDF/UA-2 asks of every Screen annotation (veraPDF rule 8.9.2.4.12-1), where tagging is on.
- `controls` puts play and pause buttons under it. WCAG 2.2.2 asks for a way to pause any movement
  that starts by itself and lasts over 5 seconds, looping or not; the check reports `autoplay`
  together with `loop` without `controls` (it cannot tell how long an animation runs).
- **Steps instead of movement** are often clearer, and print well:
  `\only<1>{\bars{1}}\only<2>{\bars{3}}\only<3>{\bars{5}}` inside one `\AccFigure`.
- **Reduced motion**: `\AccMotion{animation}{still}` shows the animation, or, with
  `\usepackage[still]{accessgate}` (or `\AccStill`), the still picture. **Still pictures** in
  TeXora (`--still`) does the same for the tagged copy without changing the file. `\AccMotion` is
  safe with animation code that has conditionals of its own; a hand-made
  `\ifmotion#1\else#2\fi` is not.
- Never flash more than three times a second (WCAG 2.3.1). TeXora does not check this.
- Decorative movement: `\AccArtifact{...}` around it, still with `controls`. Its buttons remain
  reachable by keyboard and screen readers; validate the PDF with veraPDF.

### Videos and GIFs

```latex
\includevideo[width=6cm][alt={Play video: sorting demo, 40 s, no narration},
                         caption={The list is sorted in three passes.}]{media/sort.mp4}
```

| Key | Use it for |
|---|---|
| `alt={...}` | what the video is and what clicking does. The poster is a link, so a screen reader names the link by it: *Play video: ..., 40 s*. Reported when missing (a warning; an error with `strict`). |
| `caption={...}` | a short line under the video, a figure caption for everyone. **Not** captions in the WCAG sense (subtitles in time with the sound). |
| `transcript={...}` | the full text: every word, the sounds, and what is seen. A *Transcript of ...* link under the video leads to it; it is printed where `\AccTranscripts` is. |
| `transcriptfile=<file>` | a transcript in its own file, linked under the video: `.txt` or `.pdf` for reading. A `.vtt` file is timed cues for a player, not a readable transcript; link it as well if you have one. |

One of `caption`, `transcript` or `transcriptfile` is required by the check. What WCAG asks depends
on the video:

| Prerecorded video | WCAG 2.2 | What to give |
|---|---|---|
| no sound | 1.2.1 | `caption` (what it shows) or a transcript |
| speech or sound | 1.2.2 (captions) | **synchronized captions in the video itself** (burned in, or a subtitle track the player shows); a transcript alone does not meet 1.2.2 |
| with meaning in the picture | 1.2.3 / 1.2.5 | a transcript that describes what is seen, or audio description |

TeXora cannot add synchronized captions to a video: make them with the video (for example in your
video editor or with a `.vtt` track), and give the transcript as well. Print the transcripts on a last
slide:

```latex
\begin{frame}[allowframebreaks]{Transcripts}
  \AccTranscripts
\end{frame}
```

A clip with `autoplay` **and** `loop` is reported (WCAG 2.2.2): keep `loop` and drop `autoplay`, so it
starts and stops on a click. Autoplay without loop is also covered by 2.2.2 when the clip runs over 5
seconds; the check does not know clip lengths, so look at those yourself. GIFs take the same keys.

In the tagged PDF the poster is a Figure with the alt text inside a link, the caption a paragraph
right after it, and the transcript links are links.

### Tables

```latex
\AccHeaderRows{1}               % the first row holds headers (TH) in the tagged PDF
\begin{tabular}{@{}lll@{}}
  \toprule
  Engine & Tags & Maths \\
  \midrule
  LuaLaTeX & complete & also MathML \\
  \bottomrule
\end{tabular}
```

- The rule under the first row shows the header to the eye; `\AccHeaderRows{1}` marks it in the tags
  (from LaTeX 2025-06; it does nothing in an untagged build, so it is safe in any document). The check
  notes a table without it.
- Keep tables simple: one header row, no merged cells, no empty cells used for spacing.
- A table used only for layout: use `columns` instead.

### Maths

Write maths as maths: `$a^2+b^2=c^2$`, `\[ ... \]`, `equation`. Each formula is tagged as Formula;
with LuaLaTeX the PDF also holds it as MathML, which screen readers can speak (LuaLaTeX writes a
`<name>-luamml-mathml.html` file as well). A formula drawn as a picture needs alt text.

### Links

```latex
\href{https://www.w3.org/WAI/WCAG22/quickref/}{the WCAG 2.2 quick reference}
```

- Link text must say where it goes. "here", "click here", "link", "more" and a bare web address are
  reported (WCAG 2.4.4).
- Links must stand out by more than colour (WCAG 1.4.1). Coloured links (`colorlinks`) need a contrast
  of 3:1 with the text around them, and 4.5:1 with the background, or an underline. The check measures
  `urlcolor`, `linkcolor` and `citecolor` (for the kinds of link the document has) against the text
  colour and the background. With `hidelinks`, links in a sentence look like the text around them; the
  check notes that.
- Example: with black text on white, `RGB 0,110,200` meets both (4.1:1 and 5.2:1); `RGB 0,82,155`
  does not (2.7:1 with black text).

### Colour and contrast

- Text needs a contrast of at least 4.5:1 with its background, 3:1 for large text such as titles
  (WCAG 1.4.3).
- The check measures the colours the source sets: each `\setbeamercolor` that gives both `fg` and
  `bg`, and normal text against the slide background, from the `\definecolor` lines (RGB, rgb, HTML,
  gray, xcolor's names and mixes such as `teal!80!black`). It does not see a theme's own colours, or a
  colour that only sets `fg` on a background drawn elsewhere: the measurement covers what is written,
  and the rest is advice.
- **The tagged copy drops colours and themes**: it is black text on white (`ltx-talk`'s default), so
  the contrast advice is for the Beamer slides and the projector, not for what a screen-reader user
  receives.
- Never use colour as the only signal: "the red items" means nothing without colour. Add a word, a
  symbol or bold. TeXora cannot check meaning carried by colour.

### Language and title

`\DocumentMetadata{lang=...}`, babel or polyglossia set the language; in a Beamer deck also
`\hypersetup{pdflang=...}`. The tagged copy reads the language from them (en-US when none is set).
`\title` and `pdftitle` give the title shown in the PDF viewer's window.

## 7. Checking the source

**Tools > Check accessibility** reads the presentation and the files it `\input`s against the
registry. It needs no LaTeX and works for any class and engine. Each finding names the file, line and
column, the code, what is wrong and the WCAG 2.2 success criterion, with a `fix:` line under it:

```text
talk.tex:42:3  includegraphics  [alt] missing alt={...} (or artifact, decorative)  WCAG 1.1.1
    fix: add alt={what it shows} to its options, or artifact if it is decoration only; ...
talk.tex:57:3  includevideo  [motion] starts by itself (autoplay) and loops, with no way to pause it  WCAG 2.2.2
    fix: drop autoplay, or add controls: movement over 5 seconds needs a way to pause it ...
talk.tex:63:28  href  [link] link text "here" does not say where it goes  WCAG 2.4.4
    fix: write what it leads to, for example \href{https://example.org}{the full results table}
talk.tex:70:1  frame  [title] this slide has no title  WCAG 2.4.6
    fix: add {Title} after \begin{frame}, or \frametitle{Title} inside the frame
talk.tex:12:1  (document)  [contrast] the text of "block title" has a contrast of 2.9:1 with its background; 3.0:1 is needed for large text  WCAG 1.4.3
    fix: make its fg darker or lighter: \definecolor{dim}{RGB}{...} meets it; or change its bg (\definecolor{dim} is on line 9)
talk.tex:31:43  (document)  [linkcolour] urlcolor=linkblue: links are told apart from the text by colour alone, and the colour has a contrast of 2.6:1 with the text around them (3:1 is needed)  WCAG 1.4.1
    fix: redefine the colour: \definecolor{linkblue}{RGB}{13,91,160} meets both; or underline the links (\definecolor{linkblue} is on line 29)
talk.tex:1:1  (document)  [language] no document language is set, so a screen reader may read it with the wrong voice  WCAG 3.1.1
    fix: add \usepackage[english]{babel} (or your language) after \documentclass, ...
```

The line and column (`talk.tex:31:43`) are the format compilers use, so editors that read compiler
output can jump there. Findings about the whole document point at the line that matters: a colour at
its `\setbeamercolor` or `\hypersetup` key, the language at `\documentclass`, a missing title at
`\begin{document}`. For a colour, the fix gives one that passes and the line of its `\definecolor`.

In the studio's **Accessibility** tab each finding is a card: the line and column, the text it is
about, the source line with that text marked, **Problem** and **How to fix**. Click the card to go to
the line; the text is selected and marked in the editor for a few seconds.

| Code | Means | WCAG |
|---|---|---|
| `alt` | a picture, drawing or animation without a text alternative | 1.1.1 |
| `transcript` | a video without a caption or transcript | 1.2.1-1.2.3 |
| `table` | a table whose header row is not marked: no rule, or no `\AccHeaderRows` (a note) | 1.3.1 |
| `linkcolour` | links told apart by colour alone, with too little contrast; `hidelinks` (a note) | 1.4.1 |
| `contrast` | a colour pair the source sets, below 4.5:1 (3:1 for titles) | 1.4.3 |
| `motion` | starts by itself and loops, with no way to pause | 2.2.2 |
| `doctitle` | no `\title` | 2.4.2 |
| `link` | link text that does not say where it goes | 2.4.4 |
| `title` | a slide without a title | 2.4.6 |
| `language` | no document language | 3.1.1 |
| `tagging` | notes about tagging: not tagged, Beamer, XeLaTeX | 4.1.2 |
| `placeholder` | a video poster or converted picture is not there yet | |

Errors are listed first, then **To fix** (warnings), then **Notes**. Findings are warnings by default;
`[policy] default = "error"` in a registry file makes them errors, and `--strict` on the command line
fails on any finding to fix.

## 8. The tagged copy

**Build tagged copy** leaves `talk.tex` and `talk.pdf` alone and writes beside them:

| File | What it is |
|---|---|
| `talk.acc.tex` | what is compiled: a wrapper (documents) or a converted copy (Beamer) |
| `talk.acc.pdf` | the tagged PDF (not validated) |
| `talk.acc.log` | LaTeX's log, with the `[code]` reports of `accessgate` |
| `talk.acc-losses.txt` | Beamer: what the conversion left out, changed, or could not be sure of |
| `<class>.acc.cls`, `<theme>.acc.sty` | Beamer: converted copies of a custom class or theme beside the presentation |
| `talk.acc-luamml-mathml.html` | LuaLaTeX: the MathML of the formulas, when there is maths |

**For a document** (not Beamer), `talk.acc.tex` is a wrapper that puts the right `\DocumentMetadata`
for the installed LaTeX first, then reads the document unchanged:

```latex
\IfFormatAtLeastTF{2025-06-01}
  {\DocumentMetadata{lang=en-US,pdfstandard=ua-2,tagging=on}}
  {\DocumentMetadata{lang=en-US,pdfstandard=ua-1,testphase={phase-III}}}
\input{talk.tex}
```

A document that has its own `\DocumentMetadata` keeps it.

**For Beamer slides**, Beamer stops with "The beamer class is not compatible with \DocumentMetadata",
so `talk.acc.tex` is a copy converted to `ltx-talk`. The conversion is **source to source**: it reads
the `.tex` file and rewrites what it recognises.

| Kept | Changed | Left out |
|---|---|---|
| every frame and overlay step | `\begin{frame}{Title}` becomes `\frametitle{Title}` (a heading) | themes, colours, fonts, templates |
| text, lists, blocks, columns, theorems | `[fragile]` frames become `frame*` | `\note{...}` (for the presenter) |
| pictures with their alt text, videos, animations | `\column{w}` becomes the `column` environment | transitions, `\logo`, `\titlegraphic`, section pages |
| maths, TikZ, tables, links | `\frame{...}` becomes a frame | frame options that set the look (`plain`, `shrink`, `squeeze`, ...) |
| the contents and the bibliography | `allowframebreaks` becomes `auto-break`; `t`/`c`/`b` become `vertical-alignment` | `amssymb` with LuaLaTeX (its symbols come with `ltx-talk`) |
| `label=` and `\againframe` (0.6.12) | a custom class or theme beside the file is converted with it | Beamer-only packages (`beamertheme...`, `appendixnumberbeamer`) |

What can fail or come out wrong, and is listed under *Check by hand* in `talk.acc-losses.txt`:

- TikZ with overlays (`visible on=<2>`, `\only` or `\pause` inside a picture): may show every step at
  once.
- `minted` (needs `-shell-escape`), `pgfpages`, `beamerposter`, `tcolorbox`, `multimedia`, `media9`,
  `animate` and other packages made for Beamer or for interactive PDF.
- Your own macros that call Beamer's internals.
- A presentation set for **XeLaTeX** is built with LuaLaTeX (XeLaTeX's tags are incomplete): fonts set
  with `fontspec` are kept, others may look different.

Where the copy stops at something `ltx-talk` lacks, TeXora says so and `talk.acc.log` shows the line;
the presentation is never changed.

After building, TeXora reads the PDF: whether it is tagged, its language, and every Figure's alt text.
Any picture without alt text in the copy is listed. That is not validation.

**Strict builds**: `\usepackage[strict]{accessgate}` (or `\AccStrict`) turns every report into a LaTeX
error, for a build that must not pass with anything missing; `--build-accessible talk.tex --strict`
does the same.

## 9. Validating the PDF

**The WCAG checker.** The accessibility button (a person in a circle), next to Search, Analytics and
Settings, opens a sidebar that checks the open PDF, any PDF, slides or not, against WCAG 2.2 and
PDF/UA. It reads the PDF itself (tags, language, title, PDF/UA declaration, alt text, headings, table
headers, links, annotations, fonts, bookmarks, and the text, video and animations on each page) and,
when the PDF has a `.tex` file, the source check too. Each criterion is marked:

| Mark | Means |
|---|---|
| ✗ Fails | something a program can see is missing; the issues below it say what, and how to fix it |
| ? Check by hand | a program cannot decide (reading order, colour, a page without text): a person looks |
| ✓ Passes | what can be checked automatically is there |
| – Not applicable | nothing of that kind in the PDF (no video, no pictures) |

Each issue has **Go to page** (the page in the PDF) or **Go to line** (the line in the source, with the
text marked). For Beamer slides, choose **Tagged copy** at the top to check what screen-reader users
receive: the slides' own PDF is never tagged. When veraPDF is installed, **Validate with veraPDF**
runs it on the same file and lists the failed rules.


Validate against the standard the file declares. LaTeX 2025-06 and later (the first line of the
wrapper) writes **PDF/UA-2** on PDF 2.0; an older LaTeX writes PDF/UA-1. Checking a PDF/UA-2 file
against PDF/UA-1 fails on the declared part and the PDF version alone, whatever its tags.

| Validator | PDF/UA-1 | PDF/UA-2 |
|---|---|---|
| [**veraPDF**](https://verapdf.org) (free) | yes | **yes**: the "PDF/UA-2 + Tagged PDF" profile |
| [**PAC**](https://pac.pdf-accessibility.org) (free, Windows) | yes | not stated: PDF 2.0 is "supported to a limited extent" |

So: **veraPDF for PDF/UA-2**, PAC for PDF/UA-1 files (and for its screen-reader preview, with care on
PDF 2.0 files).

```sh
verapdf --flavour ua2 talk.acc.pdf     # PDF/UA-2 (LaTeX 2025-06 and later)
verapdf --flavour ua1 talk.acc.pdf     # PDF/UA-1 (older LaTeX)
```

The tagged copies TeXora makes of Beamer slides (with `ltx-talk` 0.4.6 or 0.6.12), and the accessible
templates built with LuaLaTeX, validate as PDF/UA-2. Built with pdfLaTeX, Tagged Article and Tagged
Slides fail rule 8.2.2-1 (text neither tagged nor marked as artifact): build them with LuaLaTeX.
Validation still matters for your own files: it sees what no source check can.

Other tools:

- `show-pdf-tags talk.acc.pdf` (in TeX Live) prints the tag tree, as in [section 2](#2-what-tagging-is).
- **Adobe Acrobat**: *Accessibility > Reading Order* and the *Tags* panel.
- A screen reader: VoiceOver (macOS, Cmd+F5), NVDA (Windows), Orca (Linux). Nothing replaces
  listening to the result.

## 10. Documents that are not slides

Articles, reports, books, handouts and theses can be tagged by LaTeX on every compile, in any editor.
Put this before `\documentclass`:

```latex
\IfFormatAtLeastTF{2025-06-01}
  {\DocumentMetadata{lang=en-US,pdfstandard=ua-2,tagging=on}}
  {\DocumentMetadata{lang=en-US,pdfstandard=ua-1,testphase={phase-III}}}
\documentclass{article}
```

Everything in [section 6](#6-writing-each-kind-of-content) applies. `\AccFigure`, `\AccArtifact`,
`\AccMotion`, `\AccHeaderRows` and the video and picture keys work in documents too
(`\usepackage{accessgate}`, or `texoravideo` / `texoraimage`). Without `accessgate`, LaTeX's own
command for table headers is `\tagpdfsetup{table/header-rows={1}}` (LaTeX 2025-06 and later).

Set colours before `\hypersetup` uses them: with tagging on, hyperref looks a colour up at once and
stops with "Unknown color" if it is defined later.

The **Tagged Article** template uses LaTeX alone: compile it anywhere and look at its tags. The
**Media Handout** template adds a video, a GIF, a picture in any format and an animation.

Use LuaLaTeX (`% !TEX program = lualatex` at the top; in Overleaf, *Menu > Compiler > LuaLaTeX*).

## 11. Slides tagged directly: ltx-talk

With TeX Live 2026, slides can be made with `ltx-talk` from the start; they are then tagged on every
compile, in any editor, with no second file:

```latex
\DocumentMetadata{lang=en-US,pdfstandard=ua-2,tagging=on}
\documentclass{ltx-talk}
\title{My Talk}
\begin{document}
\begin{frame}
  \frametitle{Key ideas}
  \begin{itemize}
    \item<1-> Overlays work as in Beamer.
    \item<2-> Titles are headings, pictures keep their alt text.
  \end{itemize}
\end{frame}
\begin{frame*}
  \frametitle{Code}
\begin{verbatim}
print("frame* holds verbatim text")
\end{verbatim}
\end{frame*}
\end{document}
```

- The title goes in `\frametitle`. `\begin{frame}{Key ideas}` does **not** set a title in `ltx-talk`
  (0.4.6 and 0.6.12): the words are typeset as the first line of text, and no heading is tagged.
- Verbatim needs `frame*`. Its text starts on the line after `\begin{frame*}`, and `\end{frame*}`
  must be on a line of its own: anything after it on that line is lost.
- There are no themes and no `\note` (TeXora keeps notes in its notes file). For one page per frame,
  `\PassOptionsToClass{handout}{ltx-talk}` after `\DocumentMetadata`.
- Fonts: with LuaLaTeX, `ltx-talk` uses New Computer Modern, which lacks some characters in bold sans;
  a missing one becomes a `.notdef` glyph that PDF/UA-2 rejects (a literal `>` in `\textbf` did).
- `ltx-talk` is new and still changing. The **Tagged Slides** template is a starting point.

## 12. Other LaTeX editors

The packages and the tagging work the same in Overleaf, TeXstudio, TeXShop, VS Code with LaTeX
Workshop, or on the command line:

1. Put `accessgate.sty` beside the `.tex` file, and `texoravideo.sty` and `texoraimage.sty` if the
   document uses them. TeXora places them there when it opens or compiles the file; they are also in
   its `assets/tex` folder. In Overleaf, upload them with the `.tex` file.
2. Keep `media/`, `texora-video/` (video posters) and `texora-images/` (converted pictures) with the
   file.
3. Compile. Nothing stops with an error: a video not prepared by TeXora yet shows as a box with its
   name, with a `[placeholder]` warning.
4. Reports are LaTeX warnings that start with their code, `Package accessgate Warning: [alt] ...`, so
   any editor's log viewer, or `grep '\[alt\]' talk.log`, finds them.
5. Documents with `\DocumentMetadata` come out tagged. Beamer slides do not; make their tagged copy in
   TeXora, or with `TeXora --build-accessible talk.tex`.

## 13. Command line and CI

```sh
TeXora --check-accessibility talk.tex             # the source check; exit 0 unless an error is found
TeXora --check-accessibility talk.tex --strict    # exit 1 when anything is to fix
TeXora --build-accessible talk.tex                # the tagged copy, then every finding
TeXora --build-accessible talk.tex --handout      # Beamer: one page per frame
TeXora --build-accessible talk.tex --still        # every \AccMotion as its still picture
TeXora --build-accessible talk.tex --strict       # every report is an error
```

Exit codes: 0 nothing to fix (notes allowed), 1 something to fix with `--strict` or an error, 2 the
file cannot be read. The output is the same for every engine and class, one finding per line, so a CI
job can fail a pull request on it, then validate:

```sh
TeXora --check-accessibility slides/talk.tex --strict || exit 1
TeXora --build-accessible slides/talk.tex && verapdf --flavour ua2 slides/talk.acc.pdf
```

## 14. Your own macros

Put an `accessgate-registry.toml` beside the `.tex` file; its entries are added to TeXora's:

```toml
[command.photo]                       # \photo[alt={...}]{file}
role = "Figure"
requires = ["alt = alt | artifact"]   # code = keys; any one satisfies it

[environment.figurebox]
checks = ["figure-wrapper"]           # must be inside \AccFigure (or \AccArtifact)

[policy]
default = "error"                     # every finding is an error
```

Named checks: `motion`, `descriptive-text`, `title`, `header-row`, `figure-wrapper`.

## 15. What is not checked

The source check and the check of the copy's tags look for what can be read from the source and the
tag tree. They do **not** check:

- whether the **alt text is any good** (it is there, not whether it is right);
- **heading order** in documents, and **reading order** on the page;
- **contrast** beyond the colour pairs written in the source (theme colours, pictures, text on
  pictures), and **meaning carried by colour** alone;
- **flashing** (WCAG 2.3.1) and the length of an autoplaying clip (2.2.2);
- **captions inside a video** (synchronized captions, 1.2.2) and audio description;
- the **tag quality** of pdfLaTeX and XeLaTeX builds, which can differ from LuaLaTeX's;
- that the tagged Beamer copy shows **every slide as intended**: compare it with the slides;
- PDF/UA conformance: that is what veraPDF is for.

## 16. Limits

- **Beamer is not tagged** by itself; the tagged copy is a converted one, black on white, without the
  theme, and the conversion can miss what it does not recognise (see `talk.acc-losses.txt`).
- **TeX Live 2026 is needed** for tagged Beamer copies and `ltx-talk` slides. `ltx-talk` 0.6 needs
  LaTeX 2026-06-01: update LaTeX with it.
- **XeLaTeX** makes incomplete tags; Beamer decks set for XeLaTeX are copied with LuaLaTeX.
- **Animations and videos play** in TeXora (videos with VLC or ffmpeg installed) and animations in
  Adobe Acrobat; other viewers show the first frame or the poster. Their descriptions are in every PDF.
- **Table headers** are marked with `\AccHeaderRows` (or `\tagpdfsetup`), from LaTeX 2025-06; a rule
  alone only shows them.
- **Video captions** in the WCAG sense cannot be added by TeXora; they belong in the video.
- TeXora's check is not a PDF/UA validator. PAC does not say it supports PDF/UA-2; use veraPDF.

## 17. Reference

| Command | Does |
|---|---|
| `\usepackage{accessgate}` | the gate (loaded by `texoravideo` and `texoraimage`) |
| `\usepackage[strict]{accessgate}`, `\AccStrict` | every report is a LaTeX error |
| `\usepackage[still]{accessgate}`, `\AccStill` | every `\AccMotion` shows its still picture |
| `\AccFigure{description}{content}` | content described as one figure |
| `\AccArtifact{content}` | decoration, skipped by screen readers |
| `\AccMotion{animation}{still}` | the animation, or with `still` the still picture |
| `\AccHeaderRows{n}` | the first *n* rows of the following tables are headers |
| `\AccTranscripts` | prints the transcripts given with `transcript=`, each linked from its video |
| `\AccIncludeGraphics{\opts}{\alt}{file}` | `\includegraphics`, with `alt` only where LaTeX knows the key |
| `\AccReport{code}{message}` | a report of your own, `[code] message`, in the log |

| Key | On | Does |
|---|---|---|
| `alt={...}` | `\includegraphics`, `\includeimage`, `\includevideo` | text alternative |
| `artifact`, `decorative` | `\includegraphics`, `\includeimage` | decoration |
| `caption={...}` | `\includevideo` | a line under the video (not synchronized captions) |
| `transcript={...}` | `\includevideo` | full text, printed by `\AccTranscripts` |
| `transcriptfile=<file>` | `\includevideo` | a linked transcript file (`.txt`, `.pdf`; `.vtt` is for players) |
| `controls` | `animateinline`, `\animategraphics` | play and pause buttons |
| `alttext={...}` | `animateinline`, `\animategraphics` | animate's description of its play button |

| Option | `TeXora --build-accessible` | In TeXora |
|---|---|---|
| one page per frame | `--handout` | **One page per slide** |
| still pictures | `--still` | **Still pictures** |
| reports are errors | `--strict` | (`\usepackage[strict]{accessgate}`) |
