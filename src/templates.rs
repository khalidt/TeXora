//! The templates offered for a new presentation: Beamer slides, accessible slides, and tagged
//! documents (not slides) to test in any editor. Each is a complete .tex file in
//! assets/templates/, with two preview pictures made from it (its title slide and a content slide).
//! To change one, edit its .tex file and run scripts/make-template-previews.sh.
//! The animated templates also bring files of their own (pictures and video clips), written into a
//! `media` folder beside the new presentation.

pub struct Template {
    pub id: &'static str,
    pub name: &'static str,
    pub about: &'static str,
    /// Shape and look, in a few words.
    pub tags: &'static str,
    pub tex: &'static str,
    pub title_png: &'static [u8],
    pub content_png: &'static [u8],
    /// Files the presentation needs, by path relative to it, such as `media/clip.mp4`.
    pub files: &'static [(&'static str, &'static [u8])],
}

macro_rules! template {
    ($id:literal, $name:literal, $about:literal, $tags:literal) => {
        template!($id, $name, $about, $tags, &[])
    };
    ($id:literal, $name:literal, $about:literal, $tags:literal, $files:expr) => {
        Template {
            id: $id,
            name: $name,
            about: $about,
            tags: $tags,
            tex: include_str!(concat!("../assets/templates/", $id, ".tex")),
            title_png: include_bytes!(concat!("../assets/templates/", $id, "-1.png")),
            content_png: include_bytes!(concat!("../assets/templates/", $id, "-5.png")),
            files: $files,
        }
    };
}

/// The pictures of the Flipbook template: the frames of a GIF, played as one animation.
const FRAMES: &[(&str, &[u8])] = &[
    ("media/frame-1.png", include_bytes!("../assets/templates/media/frame-1.png")),
    ("media/frame-2.png", include_bytes!("../assets/templates/media/frame-2.png")),
    ("media/frame-3.png", include_bytes!("../assets/templates/media/frame-3.png")),
    ("media/frame-4.png", include_bytes!("../assets/templates/media/frame-4.png")),
    ("media/frame-5.png", include_bytes!("../assets/templates/media/frame-5.png")),
    ("media/frame-6.png", include_bytes!("../assets/templates/media/frame-6.png")),
    ("media/frame-7.png", include_bytes!("../assets/templates/media/frame-7.png")),
    ("media/frame-8.png", include_bytes!("../assets/templates/media/frame-8.png")),
    ("media/frame-9.png", include_bytes!("../assets/templates/media/frame-9.png")),
    ("media/frame-10.png", include_bytes!("../assets/templates/media/frame-10.png")),
    ("media/frame-11.png", include_bytes!("../assets/templates/media/frame-11.png")),
    ("media/frame-12.png", include_bytes!("../assets/templates/media/frame-12.png")),
    ("media/frame-13.png", include_bytes!("../assets/templates/media/frame-13.png")),
    ("media/frame-14.png", include_bytes!("../assets/templates/media/frame-14.png")),
    ("media/frame-15.png", include_bytes!("../assets/templates/media/frame-15.png")),
    ("media/frame-16.png", include_bytes!("../assets/templates/media/frame-16.png")),
    ("media/frame-17.png", include_bytes!("../assets/templates/media/frame-17.png")),
    ("media/frame-18.png", include_bytes!("../assets/templates/media/frame-18.png")),
    ("media/frame-19.png", include_bytes!("../assets/templates/media/frame-19.png")),
    ("media/frame-20.png", include_bytes!("../assets/templates/media/frame-20.png")),
    ("media/frame-21.png", include_bytes!("../assets/templates/media/frame-21.png")),
    ("media/frame-22.png", include_bytes!("../assets/templates/media/frame-22.png")),
    ("media/frame-23.png", include_bytes!("../assets/templates/media/frame-23.png")),
    ("media/frame-24.png", include_bytes!("../assets/templates/media/frame-24.png")),
    ("media/frame-25.png", include_bytes!("../assets/templates/media/frame-25.png")),
    ("media/frame-26.png", include_bytes!("../assets/templates/media/frame-26.png")),
    ("media/frame-27.png", include_bytes!("../assets/templates/media/frame-27.png")),
    ("media/frame-28.png", include_bytes!("../assets/templates/media/frame-28.png")),
    ("media/frame-29.png", include_bytes!("../assets/templates/media/frame-29.png")),
    ("media/frame-30.png", include_bytes!("../assets/templates/media/frame-30.png")),
];

/// The clips of the Video Guide template, played inside the slide by the texoravideo package.
const VIDEOS: &[(&str, &[u8])] = &[
    ("media/sample-video.mp4", include_bytes!("../assets/templates/video/sample-video.mp4")),
    ("media/sample-loop.gif", include_bytes!("../assets/templates/video/sample-loop.gif")),
];

/// The video clip of the Media Studio template.
const CLIP: &[(&str, &[u8])] = &[("media/clip.mp4", include_bytes!("../assets/templates/media/clip.mp4"))];

const TEST_CARD: (&str, &[u8]) = ("media/test-card.png", include_bytes!("../assets/templates/media/frame-1.png"));
const TRANSCRIPT: (&str, &[u8]) = ("media/sample-video-transcript.txt", include_bytes!("../assets/templates/video/sample-video-transcript.txt"));
const VIDEO: (&str, &[u8]) = ("media/sample-video.mp4", include_bytes!("../assets/templates/video/sample-video.mp4"));
const GIF: (&str, &[u8]) = ("media/sample-loop.gif", include_bytes!("../assets/templates/video/sample-loop.gif"));
/// TeXora's packages, beside a document meant for any editor (TeXora itself finds them by itself).
const GATE: (&str, &[u8]) = ("accessgate.sty", include_bytes!("../assets/tex/accessgate.sty"));
const VIDEO_STY: (&str, &[u8]) = ("texoravideo.sty", include_bytes!("../assets/tex/texoravideo.sty"));
const IMAGE_STY: (&str, &[u8]) = ("texoraimage.sty", include_bytes!("../assets/tex/texoraimage.sty"));

const LECTURE_FILES: &[(&str, &[u8])] = &[TEST_CARD];
const ACCESS_MEDIA_FILES: &[(&str, &[u8])] = &[VIDEO, GIF, TRANSCRIPT];
const HANDOUT_FILES: &[(&str, &[u8])] = &[VIDEO, GIF, TRANSCRIPT, TEST_CARD, GATE, VIDEO_STY, IMAGE_STY];

pub const TEMPLATES: [Template; 15] = [
    template!("classic", "Classic Green", "Madrid theme with a calm green palette and a full information bar.", "16:9 · light · green"),
    template!("midnight", "Midnight", "Dark slides with amber highlights, easy on the eyes in a dim room.", "16:9 · dark · amber"),
    template!("minimal", "Minimal White", "Clean white slides, dark text and a single blue accent.", "16:9 · light · blue"),
    template!("conference", "Conference", "Berlin theme in deep wine red, with section progress along the top.", "16:9 · light · red"),
    template!("sunrise", "Sunrise", "Warm coral and teal on a soft cream page, in the classic 4:3 shape.", "4:3 · warm · coral"),
    template!("motion", "Motion Lab", "Dark slides with an orbit, a pendulum and a countdown drawn with TikZ: looping, back and forth, on a click and at changing speeds.", "16:9 · dark · animated"),
    template!("flipbook", "Flipbook", "Paper-white slides that play the frames of a GIF and bar charts that grow. The pictures come with it.", "4:3 · light · animated", FRAMES),
    template!("media", "Media Studio", "Violet slides with a video clip, a clip packed into the PDF, and charts that draw themselves. The clip comes with it.", "16:9 · light · video", CLIP),
    template!("video", "Video Guide", "Navy and gold slides that show and explain videos and GIFs playing inside the slide, with \\includevideo. A video and a GIF come with it.", "16:9 · light · video", VIDEOS),
    template!("access-lecture", "Accessible Lecture", "High-contrast slides, one technique each: alt text, decoration, described drawings, tables, links, maths and the language.", "16:9 · high contrast · accessible", LECTURE_FILES),
    template!("access-media", "Accessible Media", "A video and a GIF with alt text, a caption, a transcript in the slides and one in its own file, and a loop you can stop.", "16:9 · light · accessible video", ACCESS_MEDIA_FILES),
    template!("access-motion", "Accessible Motion", "Animations that are described, can be paused, come as click-by-click steps and switch to still pictures with one line.", "16:9 · light · accessible animation"),
    template!("access-talk", "Tagged Slides", "Slides that LaTeX tags directly, made with the ltx-talk class instead of Beamer. Needs TeX Live 2026.", "16:9 · ltx-talk · tagged", LECTURE_FILES),
    template!("access-article", "Tagged Article", "Not slides: an article that LaTeX tags for screen readers, with nothing from TeXora in it, to test in any editor.", "page · tagged · any editor", LECTURE_FILES),
    template!("access-handout", "Media Handout", "Not slides: a tagged handout with a video, a GIF, a picture and an animation, with TeXora's packages beside it for any editor.", "page · tagged · video · any editor", HANDOUT_FILES),
];

/// Where the animated templates start in the list.
pub const FIRST_ANIMATED: usize = 5;
/// Where the accessible slides start.
pub const FIRST_ACCESSIBLE: usize = 9;
/// Where the documents (not slides) start.
pub const FIRST_DOCUMENT: usize = 13;

impl Template {
    /// A page document (an article), not slides.
    pub fn is_document(&self) -> bool {
        self.tags.starts_with("page")
    }
}

/// Writes the files template `t` needs into `dir`, leaving any that are already there alone.
/// Returns the names of those it could not write.
pub fn write_files(dir: &std::path::Path, t: &Template) -> Vec<String> {
    let mut failed = vec![];
    for (name, bytes) in t.files {
        let to = dir.join(name);
        if to.exists() {
            continue;
        }
        let ok = to.parent().map(|p| std::fs::create_dir_all(p).is_ok()).unwrap_or(true) && std::fs::write(&to, bytes).is_ok();
        if !ok {
            failed.push(name.to_string());
        }
    }
    failed
}

/// The template with this id, or the first one.
pub fn by_id(id: &str) -> usize {
    TEMPLATES.iter().position(|t| t.id == id).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn different_templates_with_previews() {
        let mut themes = std::collections::HashSet::new();
        for t in &TEMPLATES {
            assert!(t.tex.contains("\\begin{document}") && t.tex.contains("\\end{document}"), "{}", t.id);
            if t.tex.contains("{beamer}") {
                assert!(t.tex.contains("\\note{"), "{} has speaker notes", t.id);
            }
            let theme = t.tex.lines().find(|l| l.starts_with("\\usetheme")).unwrap_or("");
            let colours: Vec<&str> = t.tex.lines().filter(|l| l.starts_with("\\definecolor")).collect();
            assert!(!colours.is_empty(), "{} sets its colours", t.id);
            themes.insert(format!("{theme}{}", colours.join("")));
            for png in [t.title_png, t.content_png] {
                let p = image::load_from_memory_with_format(png, image::ImageFormat::Png).expect("preview reads");
                assert_eq!(p.width(), 400, "{}", t.id);
            }
        }
        assert_eq!(themes.len(), TEMPLATES.len(), "every template looks different");
        for t in &TEMPLATES[FIRST_ANIMATED..FIRST_ACCESSIBLE] {
            assert!(t.tex.contains("\\usepackage{animate}") || t.tex.contains("\\usepackage{texoravideo}"), "{} is animated", t.id);
            for (name, bytes) in t.files {
                let stem = name.split('-').next().unwrap_or(name);
                assert!(t.tex.contains(stem), "{} uses {name}", t.id);
                assert!(!bytes.is_empty(), "{name}");
            }
        }
        assert_eq!(by_id("sunrise"), 4);
        assert_eq!(by_id("nope"), 0);
        assert!(TEMPLATES[FIRST_DOCUMENT..].iter().all(Template::is_document) && !TEMPLATES[..FIRST_DOCUMENT].iter().any(Template::is_document));
    }

    /// The accessible Beamer templates get a tagged copy (through ltx-talk, where TeX Live has it),
    /// with every picture's alt text.
    #[test]
    fn accessible_beamer_templates_get_a_tagged_copy() {
        if !crate::access::ltx_talk() {
            return;
        }
        for t in TEMPLATES[FIRST_ACCESSIBLE..FIRST_DOCUMENT].iter().filter(|t| t.tex.contains("{beamer}")) {
            let dir = std::env::temp_dir().join(format!("texora-template-tagged-{}-{}", std::process::id(), t.id));
            std::fs::create_dir_all(&dir).unwrap();
            let tex = dir.join("talk.tex");
            std::fs::write(&tex, t.tex).unwrap();
            assert!(write_files(&dir, t).is_empty());
            std::fs::write(dir.join("accessgate.sty"), crate::access::GATE_STY).unwrap();
            let b = crate::access::build(&tex, false);
            let tags = b.tags.clone().unwrap_or_default();
            assert!(b.ok && tags.tagged && tags.figures_without_alt == 0, "{}: {} {tags:?}", t.id, b.note);
        }
    }

    /// The accessible templates and documents follow their own advice: written out with their files,
    /// the accessibility check finds nothing to fix, and every file they name comes with them.
    #[test]
    fn accessible_templates_have_nothing_to_fix() {
        for t in &TEMPLATES[FIRST_ACCESSIBLE..] {
            let dir = std::env::temp_dir().join(format!("texora-template-access-{}-{}", std::process::id(), t.id));
            std::fs::create_dir_all(&dir).unwrap();
            let tex = dir.join("talk.tex");
            std::fs::write(&tex, t.tex).unwrap();
            assert!(write_files(&dir, t).is_empty(), "{} files", t.id);
            let f: Vec<_> = crate::access::check(&tex).into_iter().filter(|f| f.severity >= crate::access::Severity::Warning).collect();
            assert!(f.is_empty(), "{}: {:?}", t.id, f.iter().map(|f| f.line_text(&dir)).collect::<Vec<_>>());
            for (name, _) in t.files {
                if !name.ends_with(".sty") {
                    assert!(t.tex.contains(name), "{} uses {name}", t.id);
                }
            }
            for used in ["media/test-card.png", "media/sample-video.mp4", "media/sample-loop.gif", "media/sample-video-transcript.txt"] {
                if t.tex.contains(&format!("{{{used}}}")) || t.tex.contains(&format!("={used}")) {
                    assert!(dir.join(used).is_file(), "{} brings {used}", t.id);
                }
            }
        }
    }

    #[test]
    fn every_template_compiles() {
        if crate::latex::find_tool("pdflatex").is_none() {
            return;
        }
        let ltx_talk = crate::access::ltx_talk();
        for t in &TEMPLATES {
            if t.tex.contains("{ltx-talk}") && !ltx_talk {
                continue;
            }
            let dir = std::env::temp_dir().join(format!("texora-template-test-{}-{}", std::process::id(), t.id));
            std::fs::create_dir_all(&dir).unwrap();
            let tex = dir.join("talk.tex");
            std::fs::write(&tex, t.tex).unwrap();
            assert!(write_files(&dir, t).is_empty(), "{} files", t.id);
            let r = crate::latex::compile(&tex, crate::latex::Engine::PdfLatex);
            assert!(r.ok, "{} did not compile: {:?}", t.id, r.problems);
            assert!(r.problems.iter().all(|p| !p.is_error), "{}", t.id);
            let _ = std::fs::remove_dir_all(&dir);
        }
    }
}
