# How to make an accessible PDF

Short recipes: how to add tags and make a PDF that works with screen readers, in TeXora or any LaTeX
editor. For the details behind each step (what is supported, what is checked, versions and limits),
see [Accessible slides and documents](Accessible.md).

<details open>
<summary><b>Recipes</b></summary>

1. [Make an accessible PDF of Beamer slides](#1-make-an-accessible-pdf-of-beamer-slides)
2. [Make an accessible PDF of a document](#2-make-an-accessible-pdf-of-a-document)
3. [Add tags to a document that has none](#3-add-tags-to-a-document-that-has-none)
4. [Describe each kind of content](#4-describe-each-kind-of-content)
5. [Find and fix what is missing](#5-find-and-fix-what-is-missing)
6. [Validate the PDF with veraPDF](#6-validate-the-pdf-with-verapdf)
7. [Work in another LaTeX editor](#7-work-in-another-latex-editor)
8. [Automate it](#8-automate-it)
9. [When something goes wrong](#9-when-something-goes-wrong)
10. [Checklist](#10-checklist)

</details>

## 1. Make an accessible PDF of Beamer slides

Beamer cannot be tagged itself, so TeXora makes a tagged copy of the slides beside them. Your `.tex`
file and its PDF stay as they are.

You need TeX Live 2026 (it includes the `ltx-talk` class the copy is made with).

1. Open the presentation in TeXora (or start from **New LaTeX presentation > Accessible Lecture**).
2. In the preamble, add the language, the title and the gate:
   ```latex
   \usepackage[english]{babel}
   \usepackage{accessgate}               % texoravideo and texoraimage load it already
   \hypersetup{pdflang=en-US,pdftitle={My Talk}}
   \title{My Talk}
   ```
3. Give every picture `alt={...}`, every slide a title, and every video a caption or transcript
   ([recipe 4](#4-describe-each-kind-of-content)).
4. **Tools > Check accessibility**. Fix every line under **To fix** ([recipe 5](#5-find-and-fix-what-is-missing)).
5. In the **Accessibility** tab, choose:
   - **One page per slide**, so a screen reader does not hear the same text again on every overlay
     step (recommended for a copy to read);
   - **Still pictures**, if the copy should show still pictures instead of animations.
6. Press **Build tagged copy**. When it is done, the tab says:
   *Tagged copy (not validated) made with LuaLaTeX and the ltx-talk class ...: talk.acc.pdf (... pictures,
   ... with alt text; language en-US).*
7. Press **Open the tagged copy** and look through it. Read `talk.acc-losses.txt`: it lists what the
   copy left out (the theme, speaker notes) and anything to check by hand.
8. Validate `talk.acc.pdf` ([recipe 6](#6-validate-the-pdf-with-verapdf)).
9. Present from `talk.pdf` as usual; give `talk.acc.pdf` to screen-reader users or publish it.

The copy has no theme: it is black text on white. Overlays, titles, pictures and their alt text,
videos, tables, maths and links are all in it.

## 2. Make an accessible PDF of a document

An article, report, thesis or handout can be tagged by LaTeX on every compile, in any editor.

1. Put these lines at the very top of the file, before `\documentclass`:
   ```latex
   \IfFormatAtLeastTF{2025-06-01}
     {\DocumentMetadata{lang=en-US,pdfstandard=ua-2,tagging=on}}
     {\DocumentMetadata{lang=en-US,pdfstandard=ua-1,testphase={phase-III}}}
   ```
   (The first line is used from LaTeX 2025-06 on, the second with older LaTeX.)
2. Add a title (`\title{...}`, and `pdftitle` in `\hypersetup`).
3. Describe the content ([recipe 4](#4-describe-each-kind-of-content)).
4. Compile with **LuaLaTeX**: put `% !TEX program = lualatex` on the first line in TeXora or TeXstudio,
   or choose it in your editor. pdfLaTeX also tags, but LuaLaTeX's tags are the most complete.
5. The PDF is now tagged. Check the source with **Tools > Check accessibility** and validate the PDF
   ([recipe 6](#6-validate-the-pdf-with-verapdf)).

The **Tagged Article** template (**New LaTeX presentation > Tagged Article**) is a finished example.

## 3. Add tags to a document that has none

Two ways, for documents that are not Beamer:

- **Without changing the file**: open it in TeXora and press **Build tagged copy**. TeXora writes
  `talk.acc.tex`, which adds `\DocumentMetadata` and then reads your file, and compiles it to
  `talk.acc.pdf`.
- **In the file itself**: add the `\DocumentMetadata` lines of [recipe 2](#2-make-an-accessible-pdf-of-a-document)
  at the top. From then on every compile, in any editor, is tagged.

To see the tags, run `show-pdf-tags talk.pdf` (it comes with TeX Live). It prints the structure:
headings, paragraphs, lists, figures with their alternate text, tables.

## 4. Describe each kind of content

Copy the line you need.

| Content | Write |
|---|---|
| A picture | `\includegraphics[width=5cm,alt={Bar chart: sales doubled from 2024 to 2026}]{chart.pdf}` |
| A picture in AVIF, WebP, HEIC or JPEG XL | `\includeimage[width=3cm,alt={Team photo, five people}]{team.heic}` |
| A decorative picture | `\includegraphics[artifact]{divider.png}` |
| Decoration that is not a picture | `\AccArtifact{\rule{2cm}{2pt}}` |
| A TikZ drawing or chart | `\AccFigure{A flow chart: Write, then Check, then Build}{\begin{tikzpicture}...\end{tikzpicture}}` |
| An animation | `\AccFigure{A pendulum swinging}{\begin{animateinline}[controls,loop,alttext={A pendulum swinging}]{16}...\end{animateinline}}` |
| An animation with a still version | `\AccMotion{\begin{animateinline}...\end{animateinline}}{\includegraphics[alt=...]{still.png}}` |
| A video | `\includevideo[width=6cm][alt={Play video: sorting demo, 40 s},caption={The list is sorted in three passes.}]{sort.mp4}` |
| A video with speech | `\includevideo[...][alt={Play video: ...},transcript={Every word that is said ...}]{talk.mp4}`, and `\AccTranscripts` on a last slide |
| A transcript in a file | `transcriptfile=media/talk-transcript.txt` in the video's second options |
| A looping GIF | `\includevideo[height=4cm][loop,alt={Play animation: ...},caption={...}]{loop.gif}` (no `autoplay`) |
| A table | `\AccHeaderRows{1}` before `\begin{tabular}`, and a rule (`\midrule`) under the first row |
| A link | `\href{https://example.org/results}{the full results table}` (text that says where it goes) |
| A slide title | `\begin{frame}{Results}` (Beamer) or `\frametitle{Results}` |
| Maths | `$a^2+b^2=c^2$`, `\[ ... \]`: write maths as maths, not as pictures |
| The language | `\usepackage[english]{babel}` and `\hypersetup{pdflang=en-US}`, or `lang=` in `\DocumentMetadata` |

Good alt text says what matters in one or two sentences: the finding of a chart, who is in a photo, what
clicking a video does. Do not start with "image of".

## 5. Find and fix what is missing

**Tools > Check accessibility** (or `TeXora --check-accessibility talk.tex`) lists, line by line, what a
screen-reader user would miss. Click a line in the **Accessibility** tab to go to it.

A slide before:

```latex
\documentclass{beamer}
\usepackage{graphicx}
\begin{document}
\begin{frame}
  \includegraphics[width=4cm]{example-image}
  More at \href{https://example.org/results}{here}.
\end{frame}
\end{document}
```

The check finds five things to fix. Each gives the file, line and column, what is wrong, and a `fix:`
line with what to do:

```text
before.tex:1:1  (document)  [language] no document language is set, so a screen reader may read it with the wrong voice  WCAG 3.1.1  (warning)
    fix: add \usepackage[english]{babel} (or your language) after \documentclass, and pdflang=en-US in \hypersetup; ...
before.tex:3:1  (document)  [doctitle] the document has no \title, so the PDF window shows the file name  WCAG 2.4.2  (warning)
    fix: add \title{...} before \begin{document} (and pdftitle={...} in \hypersetup)
before.tex:4:1  frame  [title] this slide has no title  WCAG 2.4.6  (warning)
    fix: add {Title} after \begin{frame}, or \frametitle{Title} inside the frame
before.tex:5:3  includegraphics  [alt] missing alt={...} (or artifact, decorative)  WCAG 1.1.1  (warning)
    fix: add alt={what it shows} to its options, or artifact if it is decoration only; ...
before.tex:6:46  href  [link] link text "here" does not say where it goes  WCAG 2.4.4  (warning)
    fix: write what it leads to, for example \href{https://example.org/results}{the full results table}
```

In the **Accessibility** tab each finding is a card: the line and column, the text it is about, the
source line with that text marked, **Problem** and **How to fix**. Click the card to go to the line;
the text is selected and marked in the editor for a few seconds. Findings about the whole document
point at the line that matters: a colour at its `\setbeamercolor` or `\hypersetup` key (the fix also
gives the line of its `\definecolor`, and a colour that passes), the language at `\documentclass`, a
missing title at `\begin{document}`.

The same slide after:

```latex
\documentclass{beamer}
\usepackage[english]{babel}
\usepackage{graphicx}
\usepackage{accessgate}
\hypersetup{pdflang=en-US,pdftitle={Results}}
\title{Results}
\begin{document}
\begin{frame}{Results}
  \includegraphics[width=4cm,alt={Bar chart: sales doubled from 2024 to 2026}]{example-image}
  More in \href{https://example.org/results}{the full results table}.
\end{frame}
\end{document}
```

The check now says *Nothing to fix for screen readers (1 note)*: the note says Beamer is tagged through
its copy. **Build tagged copy** makes `after.acc.pdf`, which passes veraPDF's PDF/UA-2 check.

What each code means:

| Code | Fix |
|---|---|
| `[alt]` | add `alt={...}`, or `artifact` if it is decoration; wrap drawings and animations in `\AccFigure` |
| `[transcript]` | add `caption=`, `transcript=` or `transcriptfile=` to the video |
| `[motion]` | drop `autoplay`, or add `controls` to the animation |
| `[title]` | give the slide a title |
| `[link]` | make the link text say where it goes |
| `[linkcolour]` | choose a link colour with 3:1 against the text and 4.5:1 against the background, or underline links |
| `[contrast]` | choose colours with 4.5:1 (3:1 for titles) |
| `[table]` | a rule under the header row, and `\AccHeaderRows{1}` before the table |
| `[language]` | set the language (babel, `pdflang`, or `lang=` in `\DocumentMetadata`) |
| `[doctitle]` | add `\title{...}` |
| `[placeholder]` | open the file in TeXora once, so it makes the video posters and converted pictures |

Notes (such as `[tagging]`) are information, not problems.

## 6. Validate the PDF with veraPDF

**In TeXora first**: the accessibility button (a person in a circle, next to Search, Analytics and
Settings) opens the WCAG checker for the open PDF. It lists what fails, what to check by hand and what
passes, criterion by criterion, with **Go to page** and **Go to line** for each issue. For Beamer slides
choose **Tagged copy** at its top. With veraPDF installed, its **Validate with veraPDF** button runs the
steps below for you.


TeXora checks that the PDF has tags, a language and alt text on every picture. Whether it meets the PDF/UA
standard is a validator's job. Use **veraPDF**, which is free and validates PDF/UA-2:

1. Download veraPDF from [verapdf.org/software](https://verapdf.org/software/) and install it (it needs
   Java 11 or later).
2. In a terminal, in the folder of the PDF:
   ```sh
   verapdf --flavour ua2 talk.acc.pdf
   ```
   Use `--flavour ua1` instead for a PDF made with LaTeX older than 2025-06 (it declares PDF/UA-1).
3. The report says `isCompliant="true"` when the PDF passes. Otherwise it names each failed rule and
   where it failed.

Tagged copies of the accessible templates pass this check. Then listen to
the PDF with a screen reader: VoiceOver on a Mac (**Cmd+F5**), NVDA on Windows, Orca on Linux.

## 7. Work in another LaTeX editor

The packages work the same in Overleaf, TeXstudio, TeXShop and VS Code:

1. Put `accessgate.sty` beside the `.tex` file, and `texoravideo.sty` and `texoraimage.sty` if you use
   videos or pictures in other formats. TeXora puts them there when it opens the file; they are also in
   TeXora's `assets/tex` folder.
2. Keep the `media/`, `texora-video/` and `texora-images/` folders with the file.
3. **Overleaf**: upload the `.tex` file, the `.sty` files and the folders; set **Menu > Compiler >
   LuaLaTeX**.
4. **TeXstudio, TeXShop, VS Code**: open the `.tex` file and compile; `% !TEX program = lualatex` on the
   first line selects LuaLaTeX.
5. Documents with `\DocumentMetadata` come out tagged. Beamer slides do not; make their tagged copy in
   TeXora ([recipe 1](#1-make-an-accessible-pdf-of-beamer-slides)).

Nothing stops with an error: a video TeXora has not prepared yet shows as a box with its name and a
`[placeholder]` warning in the log.

## 8. Automate it

```sh
TeXora --check-accessibility talk.tex --strict          # exit 1 when anything is to fix
TeXora --build-accessible talk.tex --handout            # tagged copy, one page per Beamer frame
TeXora --build-accessible talk.tex --still              # with still pictures instead of animations
verapdf --flavour ua2 talk.acc.pdf                     # validate the copy
```

In a CI job, fail the build when something is to fix:

```sh
TeXora --check-accessibility slides/talk.tex --strict || exit 1
TeXora --build-accessible slides/talk.tex && verapdf --flavour ua2 slides/talk.acc.pdf
```

## 9. When something goes wrong

| Message | What to do |
|---|---|
| *The beamer class is not compatible with \DocumentMetadata* | Do not add `\DocumentMetadata` to Beamer slides: use **Build tagged copy** ([recipe 1](#1-make-an-accessible-pdf-of-beamer-slides)). |
| *No tagged copy: ... the ltx-talk class ... is not installed* | Install TeX Live 2026, or run `tlmgr install ltx-talk`. |
| *The ltx-talk class requires LaTeX 2026-06-01* | `ltx-talk` was updated without LaTeX. Update both: `sudo tlmgr update --self --all`. |
| *Unknown color '...'* (only when tagged) | Put the `\definecolor` lines before the `\hypersetup` that uses them. |
| *Command \ulcorner already defined* | `amssymb` with `ltx-talk` and LuaLaTeX; TeXora's copy leaves it out. In your own `ltx-talk` slides, drop `\usepackage{amssymb}`. |
| *Package keyval Error: alt undefined* | LaTeX older than 2021-11: update TeX Live, or use `\includeimage` instead of `\includegraphics`. |
| *File `accessgate.sty' not found* (another editor) | Copy `accessgate.sty` beside the `.tex` file ([recipe 7](#7-work-in-another-latex-editor)). |
| The tagged copy stops with an error | Open `talk.acc.log` to see the line, and `talk.acc-losses.txt` for what the conversion changed. Simplify that slide (for example TikZ overlays), or report it. |
| In `ltx-talk` slides, a frame title appears as text | Write `\frametitle{...}` inside the frame, not `\begin{frame}{...}`. |
| In `ltx-talk` slides, verbatim text is lost | Use `frame*`, with `\end{frame*}` on a line of its own. |
| veraPDF: *8.2.2-1 Content ... shall be an artifact* | Compile with LuaLaTeX instead of pdfLaTeX. |
| veraPDF: a *.notdef glyph* | A character is missing from the font; in `ltx-talk` with bold sans, write `>` differently, or change the font. |
| A screen reader reads slide text again and again | Build the copy with **One page per slide**. |

## 10. Checklist

- [ ] Language set (babel and `pdflang`, or `lang=` in `\DocumentMetadata`)
- [ ] `\title{...}` and `pdftitle`
- [ ] Every slide has a title
- [ ] Every picture has `alt={...}`, or `artifact` if it is decoration
- [ ] Drawings and animations are wrapped in `\AccFigure{description}{...}`
- [ ] Every video has `alt`, and a `caption`, `transcript` or `transcriptfile`; videos with speech have captions in the video
- [ ] No `autoplay` with `loop` without `controls`; nothing flashes more than three times a second
- [ ] Tables have a header rule and `\AccHeaderRows{1}`
- [ ] Link text says where it goes; links stand out by more than colour
- [ ] Text has 4.5:1 contrast with its background (3:1 for titles)
- [ ] **Tools > Check accessibility** shows nothing to fix
- [ ] Beamer: **Build tagged copy** (with **One page per slide**), and `talk.acc-losses.txt` read
- [ ] `verapdf --flavour ua2` says compliant
- [ ] Listened to with a screen reader
