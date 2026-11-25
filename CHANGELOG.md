# Changelog

## 1.8.2

### New

- **Audience screen settings** (Settings > Audience screen, also under Present > Audience screen). **Detect displays automatically** (on at first) finds a projector or TV plugged in or out while TeXora is open, and **Look every** sets how often, from 1 to 30 seconds; **Look for displays now** checks at once. When the slide's shape differs from the screen's, **Fit** shows the whole slide with bands at the sides, **Fill** covers the screen and trims the slide's edges, and **Stretch** stretches the slide to the screen. **High resolution** (on at first) draws the slide with the screen's own pixels, up to 4K.

### Fixed

- **A projector or TV plugged in while TeXora is open is now found.** Before, TeXora only saw the displays connected when it started; it now makes a newly connected display the audience screen and says so, and notices one being unplugged.
- **The teleprompter no longer lingers between slides.** When a slide's notes end, the script moves quickly past the blank space and the next slide's heading to the next slide's first line, then goes on at the reading speed, so the reading band never waits on empty space. The space between slides is also smaller. **Band follows lines** (off at first) puts the band on the line being read, gliding from line to line, instead of at a fixed reading line.
- **The current slide in the presenter view is again at least as large as its automatic fit.** A divider saved earlier, or in another window size, could leave the current slide smaller than the next one; it now fills its column and grows with the window.

## 1.8.1

### Fixed

- **Presenting on two displays on macOS.** With "Displays have separate Spaces" turned off, opening the audience window hid the presenter, with its notes, on the other display. The audience window now fills its display without a fullscreen Space of its own, above the menu bar, and the presenter stays visible. **F11** switches it to a normal window and back.
- **Present on macOS 27.** Opening the audience window could stop TeXora with "Unable to create Texture object". On macOS, TeXora now draws with Metal instead of OpenGL; Windows and Linux are unchanged.

## 1.8.0

### New

- **Reading guide for the notes.** **Down** marks the next line of the speaker notes and **Up** the one before, with a soft band and a bar at its left, so you can follow where you are while you talk. It works on written notes and on Beamer notes pages, where it marks the note's own lines and skips the header and the small picture of the slide. It never changes the slide, keeps its place through a slide's overlay steps, and starts afresh on the next slide. In the overview, Down and Up still move through the slides. Both keys can be changed in **Settings > Keyboard shortcuts**.
- **Teleprompter.** A view of its own, from its button beside the presenter's or **View > Teleprompter**: the whole talk's notes as one script in large text that scrolls smoothly and continuously by itself, with the current and next slide beside it and the sections and timer below. **Play/Pause** (**K**) starts and stops it, and **Speed** (1 to 10, or **[** and **]**) matches your reading pace and is remembered. A band marks the reading line. When the script reaches the next slide's notes, that slide comes on show, for the audience too (**Follow slides**, on at first); moving to another slide brings its notes to the reading line. **Dark** shows light text on a dark background, and **Background** and **Text** pick any colours. The wheel, or Down and Up, move it by hand; Cmd/Ctrl with the wheel sets the text size. Notes come from the source, the `.note.md` file or the text of Beamer notes pages. `--view teleprompter` opens it.

### Changed

- **Right**, **Left**, **Space**, **Page Up** and **Page Down** move through the slides; **Down** and **Up** now move the reading guide. Shortcuts you saved before move over once; a key you gave to something else stays yours.

### Fixed

- After writing a note in the presenter, the notes area showed the plain text from then on, even once the slides had rebuilt with the note on their notes page. It now shows the notes page again as soon as the rebuild is done, with the new note on it.
- `\par` in a `\note` starts a new paragraph in the notes area instead of showing as text.

## 1.7.0

### New

- **Accessibility, in three layers that work alone.** A **registry** (`accessgate-registry.toml`: data saying which commands carry structure and what they need; add your own macros in a file of that name beside your `.tex`), a **source layer** (`accessgate.sty`, kernel only: it adds tags only where tagging is on and otherwise typesets as before, so documents never break) and **checking**.
- **Alt text and transcripts.** `\includeimage` takes `alt` and `artifact`; `\includevideo` takes `alt`, `caption`, `transcript` and `transcriptfile`; `\AccFigure{description}{...}` describes anything else, such as `animate` animations. Placeholders are tagged as Figures (or artifacts) instead of being read out as stray text.
- **Tools > Check accessibility**: an Accessibility tab in the studio listing missing alt text, captions or transcripts, autoplaying loops, vague links ("here"), untitled slides, unmarked table headers, the document's language and title, each with its line and WCAG 2.2 criterion. `TeXora --check-accessibility talk.tex [--strict]` prints the same, for any editor or CI.
- **Build tagged copy**: a tagged copy, `talk.acc.pdf`, made through a wrapper file so `talk.tex` and `talk.pdf` are untouched; LuaLaTeX, the right `\DocumentMetadata` for the installed LaTeX, and the document's language. The copy's tags are then checked: structure, language and every picture's alt text. `TeXora --build-accessible talk.tex` does the same.
- **The WCAG checker**, a new sidebar beside Search, Analytics and Settings (the accessibility button): it checks the open PDF, any PDF, slides or not, against WCAG 2.2 and PDF/UA, from the PDF itself (tags, language, title, PDF/UA declaration, alt text, headings, table headers, links, annotations, fonts, bookmarks, and the text, video and animations on each page) and from the LaTeX source when there is one. Each criterion fails, needs checking by hand, passes or does not apply, with the issues, how to fix them, and Go to page or Go to line. It can check the tagged copy of Beamer slides, build it, and run veraPDF when that is installed.
- **Findings you can act on.** Each accessibility finding gives the line and column (`talk.tex:31:43`, as compilers do), the text it is about (`urlcolor=linkblue`), the problem, and how to fix it. In the Accessibility tab a finding is a card with the source line and that text marked; a click goes to the line and selects and marks the text. Findings about the whole document point at their line (a colour at its `\setbeamercolor` or `\hypersetup` key, the language at `\documentclass`) instead of line 1, and colour fixes give a colour that passes and the line of its `\definecolor`.
- **A how-to, [how-to.md](https://github.com/khalidt/TeXora/blob/main/how-to.md)**: recipes for Beamer slides and documents, adding tags, describing each kind of content, a slide before and after fixing it, validating with veraPDF, other editors, automation, the messages you may meet, and a checklist.
- **Validated with veraPDF.** The tagged copies TeXora makes of Beamer slides, and the accessible templates, validate as PDF/UA-2.
- **ltx-talk 0.4.6 and 0.6.12** (0.6.12 needs LaTeX 2026-06-01). With 0.6.12 its own contents page and `\appendix` are used, frame labels are kept and `\againframe` repeats the frame. When `ltx-talk` was updated without LaTeX, the message says to update both.
- **Build tagged copy options**: **One page per slide** (`--handout`: each frame on one page, so a screen reader does not hear text again on every overlay step) and **Still pictures** (`--still`). Each Beamer copy comes with `<name>.acc-losses.txt`, what the conversion left out, changed, or could not be sure of.
- **Reduced motion in accessgate**: `\AccMotion{animation}{still}`, the `still` option and `\AccStill`.
- **Colour checks**: `[contrast]` (WCAG 1.4.3) for the colour pairs the source sets, and `[linkcolour]` (1.4.1) for links told apart by colour alone.
- **A guide, [Accessible.md](https://github.com/khalidt/TeXora/blob/main/Accessible.md)**: what is supported and how tagging works in TeXora and other editors, content by content (text, lists, pictures, drawings, animations, videos, tables, maths, links), how Beamer slides get a tagged PDF, and how to check its tags.
- **`\AccHeaderRows{n}`** marks the first *n* rows of the following tables as headers (TH) in the tags; untagged, it does nothing.
- **Six new templates.** Four accessible slide templates: **Accessible Lecture** (alt text, decoration, described drawings, tables, links, maths, language, contrast), **Accessible Media** (a video and a GIF with alt text, a caption, a transcript in the slides and one in its own file), **Accessible Motion** (described animations with pause buttons, click-by-click steps, a one-line still version) and **Tagged Slides** (an `ltx-talk` deck tagged by LaTeX itself). Two documents that are not slides, to test in any LaTeX editor: **Tagged Article** (LaTeX only) and **Media Handout** (a video, a GIF, a picture and an animation, with TeXora's packages beside it). The accessibility check finds nothing to fix in any of them.
- **Tagged Beamer slides** (TeX Live 2026). Beamer refuses LaTeX's tagging, so the accessible build converts the presentation to `ltx-talk`, the LaTeX project's tagging-ready slide class. Frames, overlays, frame titles (tagged as headings), pictures with their alt text, columns, blocks, theorems, verbatim frames and the contents stay; themes, transitions and speaker notes are left out. A custom class or theme beside the presentation is converted with it. Without `ltx-talk` no copy is made, and the check still applies.

### Fixed

- `texoravideo` loaded hyperref with no options, so a document loading hyperref later with its own options stopped with "Option clash". It now loads hyperref only when nothing has by the end of the preamble.
- Video options are read brace-aware: a transcript with commas or brackets no longer confuses `posterat`.
- An error message LaTeX broke over two log lines is shown whole in Problems.

### Changed

- The Video Guide template gives every clip alt text and a caption, has a slide on screen readers and sets its language.

## 1.6.1

### New

- **AVIF, WebP, JPEG XL, GIF, TIFF, BMP and HEIC/HEIF pictures.** Write `\usepackage{texoraimage}` and `\includeimage[width=0.5\linewidth]{photo.avif}`; the options are those of `\includegraphics`. pdfLaTeX, XeLaTeX and LuaLaTeX cannot read these formats, so before LaTeX runs TeXora converts each one to an ordinary JPEG (PNG when it has transparency, which is kept) in a `texora-images` folder beside the presentation, and LaTeX includes that copy. A copy is made again only when its picture changes. PNG, JPEG, PDF and EPS pictures go straight to `\includegraphics`, so `\includeimage` works for any picture.
- **Works in other LaTeX editors.** TeXora puts `texoraimage.sty` beside the presentation. With it and the `texora-images` folder, the document compiles in any LaTeX editor (tested with pdfLaTeX, XeLaTeX and LuaLaTeX) and shows every picture. Without the folder it still compiles, with a framed box naming each picture instead of an error.

### Fixed

- **Videos in other LaTeX editors.** A presentation using `\usepackage{texoravideo}` stopped with "File `texoravideo.sty' not found" in other editors, because the package lived only inside TeXora. TeXora now puts `texoravideo.sty` beside the presentation too (keeping it up to date, and leaving alone any file of that name that is not TeXora's). With the `texora-video` folder every video shows its poster; a video without a poster yet is a dark box with a play sign and its name, drawn by LaTeX, instead of a missing-file error. See "Using another LaTeX editor" in the README.
- **Built-in decoders for royalty-free formats, in pure Rust.** WebP, GIF, TIFF and BMP are decoded by the `image` crate, JPEG XL by `jxl-oxide`, and AVIF by `avif-decode` with `rav1d` (the memory-safe Rust port of the dav1d AV1 decoder, built without its assembly). Nothing needs to be installed for these.
- **HEIC/HEIF stays outside TeXora.** HEIC pictures use HEVC, which is covered by active patents (like H.264), so TeXora contains no decoder for it and uses one already on the computer: `heif-dec` (libheif), macOS's built-in `sips`, ImageMagick, or the ffmpeg set in Settings > Video. These also stand in if a built-in decoder cannot read an unusual file. The release check now also stops any HEVC decoder (libheif, libde265) from entering a package.
- Missing pictures, files that are not really that format, unsupported formats and paths outside the presentation's folder are listed as warnings under Problems, with the line; the slides still build.

## 1.5.3

### Fixed

- **Custom Beamer classes.** Frames written as `\frame{…}` or with a frame environment of their own (defined in the file, or in a class or theme file beside it) now get their notes, titles, folding and slide-to-source links. Before, only `\begin{frame}` was read, so such a deck could show a note on the wrong slide.
- **`\againframe`** is understood: the repeated slide shows the note of the frame it repeats, and a note written on it goes into that frame. Before, every note after an `\againframe` moved one slide, and a note written there could go into the next frame.
- When the source's frames cannot be matched to the slides one for one (frames in separate files or made by a macro), notes from the source are not used and edited notes go into the `.note.md` file, so no note lands on the wrong slide. The notes panel says why.
- The top row says "notes found" for notes written as `\note{}` in the source too.

### Changed

- The README shows one animated tour of TeXora instead of separate screenshots, a short demo of a video, a GIF and `animate` animations playing inside slides, and explains custom classes and themes.

## 1.5.2

### New

- **Code folding in the LaTeX studio.** An arrow beside the line numbers folds a block to its first line: an environment (`\begin{…}` to `\end{…}`), a section up to the next heading of its level, the preamble, or a `{…}` group over several lines such as a long `\note`. A folded block shows how many lines it holds; click the arrow or that label to open it. Moving the cursor or jumping to a line inside a folded block opens it. Folding only changes the view; the file keeps every line.
- **Block guide.** A green bar beside the line numbers marks the block the cursor is in, and its first and last lines are highlighted, so the matching `\begin` and `\end` are easy to see.

### Changed

- **The version is shown** at the right end of the top row (for example `v1.5.2`; a click opens About) and in the window title.
- **Errors and warnings are told apart.** The Problems tab says "Problems: 8 warnings" or "Problems: 1 error, 8 warnings", and lists errors first, then warnings, each under its own heading and with an Error or Warning label. Warnings do not stop the build; errors do. A successful build says "no errors, 8 warnings", and lines with a warning have orange numbers in the editor (errors red).

## 1.5.1

### Fixed

- **Editing in LaTeX Studio.** After a click on Save, Compile, a checkbox or the slide, the editor lost the keyboard, so the next letters ran presenter shortcuts instead of typing: **B** blacked out the slide and the arrows changed slides and moved the cursor. The keyboard now stays with the editor; another text box (such as Search) still gets its own typing.
- **No more rebuilding as you type.** The **Live** box is gone: typing never saves or rebuilds by itself. Press **Compile** (Cmd/Ctrl+B), or **Save** (Cmd/Ctrl+S) with **Rebuild on save** on.
- A file opened from a terminal with a short path (`texora talk.tex`) now compiles and checks its packages; before, both failed with "No such file or directory".
- Videos play with their sound. The sound comes from ffplay, which is part of every ffmpeg install (TeXora finds it beside your ffmpeg), and follows the picture: it starts, pauses and starts again from the same place, starts again with a `loop` clip, and stops when you show another slide. TeXora still contains no decoder of its own.
- **Settings > Video > Play sound** turns video sound on or off (on by default), and the `mute` option of `\includevideo` now keeps one clip silent. GIFs have no sound.
- ffplay runs with the same safety as ffmpeg: no window, no shell, local files only and real video formats only.
- The Video Guide template's sample video has a soft chord, so you can hear sound working.
- **Sound with VLC only.** Without ffmpeg, a clip opened in VLC plays its sound in VLC (ffplay is not needed). The `mute` option and **Settings > Video > Play sound** off now also keep VLC silent, and Settings > Video says where the sound plays.
- **Picture and sound in sync.** The picture now follows the sound's own clock (ffplay reports it), so it waits the moment ffplay takes to start instead of running ahead. Measured with a clip that flashes and beeps each second: the picture was 730 ms ahead of the sound, now within about 5 ms, also after pause and play. After a pause the sound starts again exactly where the picture is (before, it could start seconds early, from the last key frame).

## 1.5.0

### Security

- ffmpeg opens a clip with `-nostdin -protocol_whitelist file` and a list of real video formats only, so a file that is really a playlist or a concat list cannot make ffmpeg read other files or web addresses. Tested with such files: plain ffmpeg follows them, TeXora's ffmpeg refuses them.
- VLC gets the clip's full path after `--`, so a file name starting with `-` is never read as an option, and only files that really are videos (from their first bytes, not their name): never a playlist.
- Which player runs, and where it is, comes only from your own Settings. A presentation, its `.tex` file, a PDF or anything in its folder can never set the program or add arguments; this is tested.

### New

- Videos and GIFs in your slides: `\usepackage{texoravideo}` and `\includevideo[width=0.5\textwidth]{clip.mp4}`. TeXora ships the LaTeX package and finds it by itself when it compiles. A second optional argument takes `autoplay`, `loop`, `posterat=<percent>` and `poster=<image>`.
- **GIFs play by themselves.** **MP4 playback uses VLC or ffmpeg installed on your computer**: with ffmpeg the clip plays inside the slide, with VLC in VLC's own window. TeXora contains no video decoder and never downloads or installs one.
- Settings > Video chooses the player (Auto, ffmpeg, VLC or Off), with a path and a **Test** button for each.
- A video that needs a player when none is installed says "This video needs VLC or ffmpeg installed on your computer.", with **Get VLC**, **Get ffmpeg** and **Copy install command**. TeXora only opens the pages and copies the command; it never runs it.
- Before compiling, TeXora makes each clip's poster in a `texora-video` folder (again only when the clip changes; a video's poster needs ffmpeg), so every PDF viewer shows the poster; a click in TeXora plays or pauses the clip, and another slide stops it and its ffmpeg.
- A missing file, a file that is not a video or a path outside the presentation's folder shows a message over the poster and under Problems.
- A video can be in the `.tex` file's folder or in any folder inside it; a video in a folder is written with the folder name, as in `\includevideo{videos/clip.mp4}`. "Video not found" says this in plain words.
- One poster per video, however often it is used; posters no longer used are removed.
- A new template, **Video Guide**, explains `\includevideo` slide by slide: what you need, a video, a looping GIF, every option, posters, overlays and what to do when a clip does not play. Its sample video and GIF are saved in a `media` folder beside the new file.

### Fixed

- Check packages finds packages, classes, themes and TikZ libraries of your own that sit beside the `.tex` file (or in a folder you name, as in `\usepackage{styles/house}`), as LaTeX does, and no longer reports them as missing or suggests `tlmgr install` for them (#3).
- The control bar under an animation (first, previous, play/pause, next, last) works again: its buttons now get the click, instead of the slide underneath taking it.

## 1.4.1

### Fixed

- In the LaTeX studio, the slide and notes preview keeps the width you drag it to, instead of jumping back when you let go.
- The black line at the right edge of the notes page is gone, in the presenter screen and the LaTeX studio; a faint frame shows the page's edge instead.
- The presenter screen no longer leaves an empty grey strip beside the notes page: the Notes column fits the page, and the slides take the rest.
- In the LaTeX studio, a header wider than the preview no longer pushes the slide past the window's edge.

### Improved

- Double-click the edge between the source and the preview to split them half and half again.
- The LaTeX studio's preview grows with its width: on a wide panel the notes move beside the slide, so the slide can use the full height.
- The notes page in the studio gets a box of its own shape, so it fills it with no empty margin.
- In the presenter screen, widening the slide column makes the current slide bigger; in the large slide view, the notes page and the next slide share their column at the same width.

### New

- **Wrap text** in the LaTeX studio wraps long lines at the editor's width; line numbers still count real lines.

## 1.4.0

### Improved

- A new look for the presenter screen, with the slide, notes and next slide on cards.
- A top row shows the open files, whether you are ready to present, and a **Present** button.
- The drawing tools sit beside the slide in the large slide view.
- The timer, then previous and next buttons with the slide number and overlay step, sit under the main panel.
- The talk's sections are shown as a coloured line with your place and your bookmarks; click one to go there.
- The time of day sits in the top row, beside **Present**; the pace shows under the sections.
- A note saved with Edit Note shows at once, without waiting for LaTeX to rebuild the slides.
- For a `.tex` presentation, **Also save in the .tex file** chooses whether notes go into the source as `\note{...}` or into the `.note.md` file.

### New

- When you fall behind, a button offers to skip slides to catch up.
- **New LaTeX presentation** on Home and in the File menu: choose one of eight templates.
- Three animated templates: Motion Lab, Flipbook and Media Studio, with animations, GIF frames and a video clip.
- The sample lecture opens with a 20-minute duration, a bookmark and example points in its notes.

## 1.3.0

### New

- Comment out or uncomment lines in the LaTeX studio with **Cmd/Ctrl+/**.
- With several lines selected, all of them are commented or uncommented at once.
- **Cmd/Ctrl+]** comments and **Cmd/Ctrl+[** uncomments; all three keys can be changed in Settings.
- Animations made with the LaTeX `animate` package now play, as in Adobe Acrobat.
- Animations start by themselves when set to (`autoplay`), loop or play back and forth, at their own frame rate.
- Click an animation to play or pause it; the presenter view has first, previous, play/pause, next and last frame buttons.
- Video clips (`\movie` or media9) show a play button and open in your video player; clips set to start by themselves do so during the talk.

## 1.2.0

### New

- Added LaTeX studio.
- Edit the `.tex` source beside a live preview of the slide and its notes.
- Slides rebuild shortly after you stop typing, or when you save.
- Problems with their line numbers, required packages and the full log in one panel.
- Double-click a slide to jump to its source; the slide follows the cursor as you move through the source.
- Open it with the studio button, **View > LaTeX studio** or **Cmd/Ctrl+E**; save with **Cmd/Ctrl+S**.
- **Check packages** opens the package list in the studio.

### Fixed

- Notes now stay the same in the presenter and the LaTeX studio.
- For a `.tex` presentation, a note edited in the presenter is saved into that slide's `\note{...}` in the source.
- A `\note` edited in the studio shows in the presenter, even when a `.note.md` file already exists.
- After a rebuild, the presenter stays on the slide you were on.
- A note being edited stays with its own presentation when you switch tabs.

## 1.1.0

### New

- Added Edit Note feature.
- Edit the notes of any slide in the notes panel: press **E** or click the edit button.
- Save with the tick button or **Cmd/Ctrl+Enter**; cancel with the cross or **Esc**.
- Notes file is saved in the same folder as the opened presentation, using the presentation's filename followed by `.note.md`.
- **Write notes** button on slides that have no notes yet.
- Notes can be written in any language: Arabic, Chinese, Japanese, Korean and more.
- Arabic notes read right to left, with words, numbers and punctuation in the right order.
- Typing Arabic in the note editor runs from the right edge, with words in reading order.
- Emoji show in colour.

### Fixed

- Text in Arabic, Chinese, Japanese, Korean and emoji no longer shows as empty boxes.
- Notes files saved as UTF-16 (for example by Windows Notepad) or in older encodings now open correctly.

## 1.0.1

### New

- Update notice appears by itself when a new version is out.
- The notice offers **See what's new**, **Later** and **Skip this version**.
- The update box offers the download for your computer.
- TeXora checks for updates at start and every few hours while open.

### Fixed

- The bin button in **Recent files** now removes the file at once, without flickering.

### Changed

- **New LaTeX presentation** on Home is switched off.

## 1.0.0

- First public release.
- Presenter and audience views, LaTeX Beamer compiling and speaker notes.
- Drawing tools, laser pointer, zoom, bookmarks, contents, search and analytics.
- Installers for macOS, Windows and Linux.
