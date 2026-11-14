//! Pictures LaTeX cannot read, for `\usepackage{texoraimage}` and `\includeimage{...}`: AVIF, WebP,
//! JPEG XL, GIF, TIFF, BMP and HEIC/HEIF.
//!
//! Before each compile TeXora finds every `\includeimage{...}`, and converts each such picture to an
//! ordinary JPEG (PNG when it has transparency) in `texora-images/<MD5>.jpg` beside the presentation,
//! where MD5 is that of the path as written (the package works out the same name). LaTeX then includes
//! that copy with `\includegraphics`, in every engine. A copy is made again only when its picture's size
//! or modification time changes (`texora-images/index.json`); copies no longer used are removed.
//!
//! The package (`assets/tex/texoraimage.sty`) is found through TEXINPUTS while TeXora compiles, and a
//! copy is also written beside the presentation, so the document compiles in any other LaTeX editor:
//! with the `texora-images` folder it shows every picture, without it a framed box and a warning.
//!
//! Formats that are free to use are decoded inside TeXora, in pure Rust: WebP, GIF, TIFF and BMP by the
//! `image` crate, JPEG XL by `jxl-oxide`, and AVIF by `avif-decode` with `rav1d` (the Rust port of dav1d,
//! built without its assembly). HEIC/HEIF pictures use HEVC, which is covered by active patents, so
//! TeXora contains no decoder for them and uses a converter already on the computer: `heif-dec`
//! (libheif), macOS's built-in `sips`, ImageMagick, or the ffmpeg set in Settings > Video. Those also
//! stand in when a built-in decoder cannot read an unusual file. Safety, as for videos: only files inside
//! the presentation's folder, only files whose first bytes really are that format, full paths only,
//! never a shell.
use crate::video_prep::{balanced, md5_upper, resolve, uncomment};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const STY: &str = include_str!("../assets/tex/texoraimage.sty");
pub const FOLDER: &str = "texora-images";
/// The first line of the package; a `texoraimage.sty` beside the presentation starting with it is
/// TeXora's own copy, which TeXora keeps up to date. Any other file of that name is left alone
/// (`video_prep::place_beside`).
#[cfg(test)]
const MARK: &str = "%% texoraimage.sty -- ";
/// Copies are at most this many pixels on their longer side: plenty for a slide, and a small PDF.
const MAX_SIDE: u32 = 2400;
const JPEG_QUALITY: u8 = 90;

/// The picture formats TeXora converts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Avif,
    Webp,
    Heic,
    Jxl,
    Gif,
    Tiff,
    Bmp,
}

/// Pictures larger than this many pixels are refused before decoding (a 20000 x 10000 picture is far
/// beyond any slide, and a crafted file could ask for much more memory).
const MAX_PIXELS: u64 = 200_000_000;

impl Format {
    pub fn of(path: &str) -> Option<Format> {
        let ext = Path::new(path).extension()?.to_string_lossy().to_lowercase();
        match ext.as_str() {
            "avif" => Some(Format::Avif),
            "webp" => Some(Format::Webp),
            "heic" | "heif" | "hif" => Some(Format::Heic),
            "jxl" => Some(Format::Jxl),
            "gif" => Some(Format::Gif),
            "tif" | "tiff" => Some(Format::Tiff),
            "bmp" | "dib" => Some(Format::Bmp),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Format::Avif => "AVIF",
            Format::Webp => "WebP",
            Format::Heic => "HEIC/HEIF",
            Format::Jxl => "JPEG XL",
            Format::Gif => "GIF",
            Format::Tiff => "TIFF",
            Format::Bmp => "BMP",
        }
    }

    /// Whether TeXora decodes this format itself. Not HEIC/HEIF: its HEVC pictures are covered by
    /// active patents, so those go to a converter on the computer.
    pub fn built_in(self) -> bool {
        self != Format::Heic
    }

    /// Whether a file's first bytes are this format's (its name does not count).
    pub fn matches(self, head: &[u8]) -> bool {
        let ftyp = head.len() >= 12 && &head[4..8] == b"ftyp";
        let brands = |list: &[&[u8; 4]]| ftyp && head[8..head.len().min(64)].chunks(4).any(|b| list.iter().any(|x| b == &x[..]));
        match self {
            Format::Webp => head.len() >= 12 && &head[0..4] == b"RIFF" && &head[8..12] == b"WEBP",
            Format::Avif => brands(&[b"avif", b"avis"]),
            Format::Heic => brands(&[b"heic", b"heix", b"hevc", b"hevx", b"heim", b"heis", b"hevm", b"hevs", b"mif1", b"msf1"]),
            Format::Jxl => head.starts_with(&[0xFF, 0x0A]) || head.starts_with(&[0, 0, 0, 0x0C, b'J', b'X', b'L', b' ', 0x0D, 0x0A, 0x87, 0x0A]),
            Format::Gif => head.starts_with(b"GIF87a") || head.starts_with(b"GIF89a"),
            Format::Tiff => head.starts_with(b"II*\0") || head.starts_with(b"MM\0*"),
            Format::Bmp => head.starts_with(b"BM"),
        }
    }
}

/// Extensions LaTeX reads itself; `\includeimage` passes these straight to `\includegraphics`.
pub fn is_native(path: &str) -> bool {
    let ext = Path::new(path).extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "pdf" | "eps")
}

/// One `\includeimage` in the source.
#[derive(Clone, Debug, PartialEq)]
pub struct Use {
    /// The path as written, with runs of spaces made one, as TeX reads it.
    pub path: String,
    pub line: usize,
    pub file: PathBuf,
}

/// Every `\includeimage` in one file's text.
pub fn uses_in(src: &str, file: &Path) -> Vec<Use> {
    let clean = uncomment(src);
    let b = clean.as_bytes();
    let name = "\\includeimage";
    let mut out = vec![];
    let mut pos = 0;
    while let Some(p) = clean[pos..].find(name) {
        let start = pos + p;
        let mut i = start + name.len();
        pos = i;
        if clean[i..].starts_with(|c: char| c.is_ascii_alphabetic()) {
            continue;
        }
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < b.len() && b[i] == b'[' {
            let Some((_, end)) = balanced(&clean, i) else { continue };
            i = end;
            while i < b.len() && b[i].is_ascii_whitespace() {
                i += 1;
            }
        }
        if i >= b.len() || b[i] != b'{' {
            continue;
        }
        let Some((arg, end)) = balanced(&clean, i) else { continue };
        pos = end;
        let path = arg.split_whitespace().collect::<Vec<_>>().join(" ");
        let line = clean[..start].matches('\n').count() + 1;
        out.push(Use { path, line, file: file.to_path_buf() });
    }
    out
}

/// Every `\includeimage` of a document: the main file and the files it `\input`s or `\include`s.
pub fn uses(root: &Path) -> (Vec<Use>, bool) {
    let dir = root.parent().unwrap_or(Path::new("."));
    let (mut out, mut wants) = (vec![], false);
    let mut seen: Vec<PathBuf> = vec![];
    let mut todo = vec![root.to_path_buf()];
    while let Some(f) = todo.pop() {
        if seen.contains(&f) || seen.len() > 200 {
            continue;
        }
        seen.push(f.clone());
        let Ok(src) = std::fs::read_to_string(&f) else { continue };
        out.extend(uses_in(&src, &f));
        let clean = uncomment(&src);
        wants |= uses_package(&clean);
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
    }
    (out, wants)
}

/// Whether comment-free source loads the package: `\usepackage{texoraimage}`, alone or in a list.
fn uses_package(clean: &str) -> bool {
    clean.match_indices("\\usepackage").any(|(k, _)| {
        let rest = &clean[k + "\\usepackage".len()..];
        let rest = rest.trim_start();
        let rest = match rest.strip_prefix('[') {
            Some(r) => r.split_once(']').map(|(_, a)| a).unwrap_or(""),
            None => rest,
        };
        rest.trim_start().strip_prefix('{').and_then(|r| r.split_once('}')).is_some_and(|(list, _)| list.split(',').any(|p| p.trim() == "texoraimage"))
    })
}

/// The copy's file name for a picture's path as written, without its extension (the package looks for
/// `.jpg`, then `.png`).
pub fn copy_stem(path: &str) -> String {
    md5_upper(path)
}

/// Whether a file in `texora-images/` is a copy TeXora made: an MD5 in capitals, `.jpg` or `.png`.
fn is_copy_file(name: &str) -> bool {
    let stem = name.strip_suffix(".jpg").or_else(|| name.strip_suffix(".png")).unwrap_or("");
    stem.len() == 32 && stem.bytes().all(|b| b.is_ascii_digit() || (b'A'..=b'F').contains(&b))
}

/// What `index.json` keeps about each copy.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Entry {
    pub path: String,
    pub size: u64,
    pub modified: u64,
    /// The copy's file name (`<MD5>.jpg` or `<MD5>.png`).
    pub copy: String,
    /// The converter that made it.
    pub tool: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Index {
    pub images: BTreeMap<String, Entry>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    pub made: usize,
    pub reused: usize,
    /// Each picture that could not be made ready: the file and line of its `\includeimage`, and why.
    pub problems: Vec<(PathBuf, usize, String)>,
}

fn stamp(p: &Path) -> (u64, u64) {
    let m = std::fs::metadata(p).ok();
    let size = m.as_ref().map(|m| m.len()).unwrap_or(0);
    let modified = m.and_then(|m| m.modified().ok()).and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
    (size, modified)
}

/// Writes TeXora's `texoraimage.sty` beside the presentation, unless a file of that name there is
/// someone else's.
fn place_package(dir: &Path) {
    crate::video_prep::place_beside(dir, "texoraimage.sty", STY);
}

/// A converter that is on this computer, and the arguments that turn `input` into the PNG `output`.
/// Full paths only, never through a shell.
pub fn converters(format: Format, input: &Path, output: &Path) -> Vec<(String, PathBuf, Vec<String>)> {
    let (i, o) = (crate::video::plain(input), crate::video::plain(output));
    let mut out: Vec<(String, PathBuf, Vec<String>)> = vec![];
    fn add(out: &mut Vec<(String, PathBuf, Vec<String>)>, name: &str, args: Vec<String>) {
        if let Some(p) = crate::latex::find_tool(name) {
            out.push((name.to_string(), p, args));
        }
    }
    // The format's own tool (libwebp, libavif, libjxl, libheif) first: the most faithful.
    match format {
        Format::Webp => add(&mut out, "dwebp", vec![i.clone(), "-o".into(), o.clone()]),
        Format::Avif => add(&mut out, "avifdec", vec![i.clone(), o.clone()]),
        Format::Jxl => add(&mut out, "djxl", vec![i.clone(), o.clone()]),
        Format::Heic => {
            add(&mut out, "heif-dec", vec![i.clone(), o.clone()]);
            add(&mut out, "heif-convert", vec![i.clone(), o.clone()]);
        }
        Format::Gif | Format::Tiff | Format::Bmp => {}
    }
    // Built into macOS.
    if cfg!(target_os = "macos") {
        out.push(("sips".into(), PathBuf::from("/usr/bin/sips"), vec!["-s".into(), "format".into(), "png".into(), i.clone(), "--out".into(), o.clone()]));
    }
    // ImageMagick, told the format so it uses only that reader.
    let coder = match format {
        Format::Avif => "AVIF",
        Format::Webp => "WEBP",
        Format::Heic => "HEIC",
        Format::Jxl => "JXL",
        Format::Gif => "GIF",
        Format::Tiff => "TIFF",
        Format::Bmp => "BMP",
    };
    add(&mut out, "magick", vec![format!("{coder}:{i}"), format!("PNG:{o}")]);
    // ffmpeg, as for videos: local files only, real picture formats only.
    if let Some(ff) = crate::video::tools().ffmpeg {
        let args = vec![
            "-nostdin".into(), "-hide_banner".into(), "-loglevel".into(), "error".into(),
            "-protocol_whitelist".into(), "file".into(),
            "-format_whitelist".into(), "image2,webp_pipe,jpegxl_pipe,gif,tiff_pipe,bmp_pipe,mov,mp4,heif".into(),
            "-i".into(), format!("file:{i}"), "-frames:v".into(), "1".into(), "-y".into(), format!("file:{o}"),
        ];
        out.push(("ffmpeg".into(), ff, args));
    }
    out
}

/// Decodes a picture inside TeXora, in pure Rust (every format but HEIC/HEIF). For a GIF, its first
/// frame.
pub fn decode_built_in(format: Format, input: &Path) -> Result<image::DynamicImage, String> {
    use image::DynamicImage as D;
    let too_big = |w: u64, h: u64| if w * h > MAX_PIXELS { Err(format!("the picture is too large ({w} x {h} pixels)")) } else { Ok(()) };
    let bad = || "the decoded picture has an unexpected size".to_string();
    match format {
        Format::Webp | Format::Gif | Format::Tiff | Format::Bmp => {
            let f = match format {
                Format::Webp => image::ImageFormat::WebP,
                Format::Gif => image::ImageFormat::Gif,
                Format::Tiff => image::ImageFormat::Tiff,
                _ => image::ImageFormat::Bmp,
            };
            let mut r = image::ImageReader::open(input).map_err(|e| e.to_string())?;
            r.set_format(f);
            let (w, h) = r.into_dimensions().map_err(|e| e.to_string())?;
            too_big(w as u64, h as u64)?;
            let mut r = image::ImageReader::open(input).map_err(|e| e.to_string())?;
            r.set_format(f);
            r.decode().map_err(|e| e.to_string())
        }
        Format::Jxl => {
            let mut img = jxl_oxide::JxlImage::builder().open(input).map_err(|e| e.to_string())?;
            too_big(img.width() as u64, img.height() as u64)?;
            // Ordinary sRGB, as LaTeX and every PDF viewer expect.
            img.request_color_encoding(jxl_oxide::EnumColourEncoding::srgb(jxl_oxide::RenderingIntent::Relative));
            let render = img.render_frame(0).map_err(|e| e.to_string())?;
            let mut stream = render.stream();
            let (w, h, c) = (stream.width(), stream.height(), stream.channels());
            let mut buf = vec![0u8; (w * h * c) as usize];
            stream.write_to_buffer(&mut buf);
            Ok(match c {
                1 => D::ImageLuma8(image::ImageBuffer::from_raw(w, h, buf).ok_or_else(bad)?),
                2 => D::ImageLumaA8(image::ImageBuffer::from_raw(w, h, buf).ok_or_else(bad)?),
                3 => D::ImageRgb8(image::ImageBuffer::from_raw(w, h, buf).ok_or_else(bad)?),
                4 => D::ImageRgba8(image::ImageBuffer::from_raw(w, h, buf).ok_or_else(bad)?),
                n => return Err(format!("{n} colour channels are not supported")),
            })
        }
        Format::Avif => {
            let bytes = std::fs::read(input).map_err(|e| e.to_string())?;
            let img = avif_decode::Decoder::from_avif(&bytes).map_err(|e| e.to_string())?.to_image().map_err(|e| e.to_string())?;
            use avif_decode::Image as A;
            let out = match img {
                A::Rgb8(i) => {
                    let (b, w, h) = i.into_contiguous_buf();
                    D::ImageRgb8(image::ImageBuffer::from_raw(w as u32, h as u32, b.iter().flat_map(|p| [p.r, p.g, p.b]).collect()).ok_or_else(bad)?)
                }
                A::Rgba8(i) => {
                    let (b, w, h) = i.into_contiguous_buf();
                    D::ImageRgba8(image::ImageBuffer::from_raw(w as u32, h as u32, b.iter().flat_map(|p| [p.r, p.g, p.b, p.a]).collect()).ok_or_else(bad)?)
                }
                A::Gray8(i) => {
                    let (b, w, h) = i.into_contiguous_buf();
                    D::ImageLuma8(image::ImageBuffer::from_raw(w as u32, h as u32, b.iter().map(|p| p.value()).collect()).ok_or_else(bad)?)
                }
                A::Rgb16(i) => {
                    let (b, w, h) = i.into_contiguous_buf();
                    D::ImageRgb16(image::ImageBuffer::from_raw(w as u32, h as u32, b.iter().flat_map(|p| [p.r, p.g, p.b]).collect()).ok_or_else(bad)?)
                }
                A::Rgba16(i) => {
                    let (b, w, h) = i.into_contiguous_buf();
                    D::ImageRgba16(image::ImageBuffer::from_raw(w as u32, h as u32, b.iter().flat_map(|p| [p.r, p.g, p.b, p.a]).collect()).ok_or_else(bad)?)
                }
                A::Gray16(i) => {
                    let (b, w, h) = i.into_contiguous_buf();
                    D::ImageLuma16(image::ImageBuffer::from_raw(w as u32, h as u32, b.iter().map(|p| p.value()).collect()).ok_or_else(bad)?)
                }
            };
            too_big(out.width() as u64, out.height() as u64)?;
            Ok(out)
        }
        Format::Heic => Err("HEIC/HEIF is not decoded inside TeXora".into()),
    }
}

/// Makes the copy of one picture, at most `MAX_SIDE` pixels on its longer side: a JPEG, or a PNG when
/// it has transparency. TeXora decodes it itself when it can; otherwise (HEIC/HEIF, or a file the
/// built-in decoder cannot read) a converter on the computer makes a PNG first. Returns the copy's file
/// name and what decoded it.
fn convert(format: Format, input: &Path, out_dir: &Path, stem: &str) -> Result<(String, String), String> {
    let mut built_in_error = None;
    if format.built_in() {
        match decode_built_in(format, input) {
            Ok(img) => return save_copy(img, out_dir, stem).map(|c| (c, "built-in".to_string())),
            Err(e) => built_in_error = Some(e),
        }
    }
    let tmp = out_dir.join(format!(".{stem}.tmp.png"));
    let tools = converters(format, input, &tmp);
    if tools.is_empty() {
        return Err(match built_in_error {
            Some(e) => format!("The {} picture could not be read ({e}).", format.name()),
            None => format!("No converter for {} pictures was found. {}", format.name(), install_hint()),
        });
    }
    let mut tried: Vec<String> = built_in_error.map(|e| format!("built-in: {e}")).into_iter().collect();
    for (name, program, args) in tools {
        let _ = std::fs::remove_file(&tmp);
        let mut c = crate::video::tool_command(&program);
        c.args(&args);
        match crate::video::run_for(c, Duration::from_secs(90)) {
            Ok((_, err)) if tmp.is_file() => {
                let made = finish(&tmp, out_dir, stem);
                let _ = std::fs::remove_file(&tmp);
                match made {
                    Ok(copy) => return Ok((copy, name)),
                    Err(e) => tried.push(format!("{name}: {e}")),
                }
                let _ = err;
            }
            Ok((_, err)) => tried.push(format!("{name}: {}", err.lines().find(|l| !l.trim().is_empty()).unwrap_or("it made no picture").trim())),
            Err(e) => tried.push(format!("{name}: {e}")),
        }
    }
    let _ = std::fs::remove_file(&tmp);
    let hint = if format.built_in() { "" } else { install_hint() };
    Err(format!("The {} picture could not be converted ({}). {hint}", format.name(), tried.join("; ")).trim_end().to_string())
}

/// Turns the converter's PNG into the copy LaTeX reads.
fn finish(png: &Path, out_dir: &Path, stem: &str) -> Result<String, String> {
    save_copy(image::open(png).map_err(|e| e.to_string())?, out_dir, stem)
}

/// Saves the copy LaTeX reads: a JPEG, or a PNG when the picture has transparency.
fn save_copy(img: image::DynamicImage, out_dir: &Path, stem: &str) -> Result<String, String> {
    let img = if img.width().max(img.height()) > MAX_SIDE { img.resize(MAX_SIDE, MAX_SIDE, image::imageops::FilterType::Lanczos3) } else { img };
    let see_through = img.color().has_alpha() && img.to_rgba8().pixels().any(|p| p.0[3] < 255);
    let (name, other) = if see_through { (format!("{stem}.png"), format!("{stem}.jpg")) } else { (format!("{stem}.jpg"), format!("{stem}.png")) };
    let dest = out_dir.join(&name);
    if see_through {
        img.save_with_format(&dest, image::ImageFormat::Png).map_err(|e| e.to_string())?;
    } else {
        let f = std::fs::File::create(&dest).map_err(|e| e.to_string())?;
        let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(std::io::BufWriter::new(f), JPEG_QUALITY);
        enc.encode_image(&img.to_rgb8()).map_err(|e| e.to_string())?;
    }
    // Only one copy per picture: the package would take a stale .jpg before a new .png.
    let _ = std::fs::remove_file(out_dir.join(other));
    Ok(name)
}

/// How to get a HEIC/HEIF converter, for this system (the other formats need nothing).
pub fn install_hint() -> &'static str {
    if cfg!(target_os = "macos") {
        "macOS reads HEIC pictures itself (sips, built in). If that fails, install ImageMagick or libheif with Homebrew."
    } else if cfg!(windows) {
        "Install ImageMagick (imagemagick.org), then compile again."
    } else {
        "Install libheif's tools (the libheif-examples package) or ImageMagick, then compile again."
    }
}

fn read_index(out_dir: &Path) -> Index {
    std::fs::read_to_string(out_dir.join("index.json")).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

/// Gets every `\includeimage` picture of the document at `root` ready for LaTeX.
pub fn prepare(root: &Path) -> Report {
    let (uses, wants) = uses(root);
    let mut report = Report::default();
    let dir = root.parent().unwrap_or(Path::new(".")).to_path_buf();
    if wants || !uses.is_empty() {
        place_package(&dir);
    }
    let todo: Vec<&Use> = uses.iter().filter(|u| !is_native(&u.path)).collect();
    let out_dir = dir.join(FOLDER);
    if todo.is_empty() {
        return report;
    }
    if std::fs::create_dir_all(&out_dir).is_err() {
        return report;
    }
    let old = read_index(&out_dir);
    let mut index = Index::default();
    let mut problem = |u: &Use, msg: String| report.problems.push((u.file.clone(), u.line, msg));
    let mut done: BTreeMap<String, Result<String, String>> = BTreeMap::new();
    for u in &todo {
        let stem = copy_stem(&u.path);
        if let Some(r) = done.get(&stem) {
            if let Err(e) = r {
                problem(u, e.clone());
            }
            continue;
        }
        let result = (|| {
            let Some(format) = Format::of(&u.path) else {
                return Err(format!("{}: this kind of picture is not supported. Use PNG, JPEG, PDF, AVIF, WebP, JPEG XL, GIF, TIFF, BMP or HEIC/HEIF.", u.path));
            };
            let file = resolve(&dir, &u.path).map_err(|p| match p {
                crate::video::Problem::Missing => format!("Picture not found: {}. Put it in the .tex file's folder or a folder inside it, and write that folder in the name ({{images/photo.heic}}).", u.path),
                crate::video::Problem::Refused(_) => format!("{}: only pictures inside the presentation's folder are used. Write the path from that folder.", u.path),
                other => format!("{}: {}", u.path, other.message(&u.path)),
            })?;
            let head = crate::video::head(&file).unwrap_or_default();
            if !format.matches(&head) {
                return Err(format!("{}: the file is not really a {} picture.", u.path, format.name()));
            }
            let (size, modified) = stamp(&file);
            if let Some(e) = old.images.get(&stem) {
                if e.size == size && e.modified == modified && e.path == u.path && out_dir.join(&e.copy).is_file() {
                    report.reused += 1;
                    index.images.insert(stem.clone(), e.clone());
                    return Ok(e.copy.clone());
                }
            }
            let (copy, tool) = convert(format, &file, &out_dir, &stem)?;
            report.made += 1;
            index.images.insert(stem.clone(), Entry { path: u.path.clone(), size, modified, copy: copy.clone(), tool });
            Ok(copy)
        })();
        if let Err(e) = &result {
            problem(u, e.clone());
        }
        done.insert(stem, result);
    }
    // Copies no longer used go.
    let keep: Vec<String> = index.images.values().map(|e| e.copy.clone()).collect();
    if let Ok(entries) = std::fs::read_dir(&out_dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if is_copy_file(&name) && !keep.contains(&name) {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    if let Ok(s) = serde_json::to_string_pretty(&index) {
        let _ = std::fs::write(out_dir.join("index.json"), s);
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_are_read_with_their_options() {
        let src = "\\usepackage{graphicx,texoraimage}\n% \\includeimage{commented.heic}\n\\includeimage[width=0.5\\linewidth]{photos/My  photo.heic}\n\\includeimage {a.webp} \\includeimagefoo{x}\n";
        let u = uses_in(src, Path::new("t.tex"));
        let got: Vec<(&str, usize)> = u.iter().map(|x| (x.path.as_str(), x.line)).collect();
        assert_eq!(got, vec![("photos/My photo.heic", 3), ("a.webp", 4)]);
        assert!(uses_package(&uncomment(src)));
        assert!(uses_package("\\usepackage[draft]{texoraimage}"));
        assert!(!uses_package("\\usepackage{texoravideo}"));
    }

    #[test]
    fn formats_are_told_by_name_and_by_their_first_bytes() {
        assert_eq!(Format::of("a/B.HEIC"), Some(Format::Heic));
        assert_eq!(Format::of("x.heif"), Some(Format::Heic));
        assert_eq!(Format::of("x.avif"), Some(Format::Avif));
        assert_eq!(Format::of("x.webp"), Some(Format::Webp));
        assert_eq!(Format::of("x.jxl"), Some(Format::Jxl));
        assert_eq!(Format::of("x.png"), None);
        assert_eq!(Format::of("x.GIF"), Some(Format::Gif));
        assert_eq!(Format::of("x.tif"), Some(Format::Tiff));
        assert_eq!(Format::of("x.bmp"), Some(Format::Bmp));
        assert!(Format::Gif.matches(b"GIF89a\x10\0"));
        assert!(Format::Tiff.matches(b"II*\0\x08\0\0\0") && Format::Tiff.matches(b"MM\0*\0\0\0\x08"));
        assert!(Format::Bmp.matches(b"BM\x36\0\0\0"));
        assert!(!Format::Gif.matches(b"BM\x36\0"));
        assert!(is_native("a.PNG") && is_native("b.jpeg") && is_native("c.pdf") && !is_native("d.webp"));
        assert!(Format::Webp.matches(b"RIFF\x10\0\0\0WEBPVP8 "));
        assert!(Format::Avif.matches(b"\0\0\0\x1cftypavif\0\0\0\0avifmif1"));
        assert!(Format::Heic.matches(b"\0\0\0\x18ftypheic\0\0\0\0mif1heic"));
        assert!(Format::Heic.matches(b"\0\0\0\x18ftypmif1\0\0\0\0mif1heic"));
        assert!(Format::Jxl.matches(&[0xFF, 0x0A, 1, 2]));
        assert!(Format::Jxl.matches(b"\0\0\0\x0cJXL \r\n\x87\n\0\0"));
        // A playlist or text file renamed is not a picture.
        assert!(!Format::Webp.matches(b"#EXTM3U\nhttp://x"));
        assert!(!Format::Avif.matches(b"\0\0\0\x18ftypisom\0\0\0\0isom"), "an MP4 is not an AVIF");
    }

    #[test]
    fn copies_are_named_as_the_package_names_them() {
        // \pdf@mdfivesum of the path as written, in capitals.
        assert_eq!(copy_stem("photo.heic"), md5_upper("photo.heic"));
        assert!(is_copy_file(&format!("{}.jpg", copy_stem("a.webp"))));
        assert!(is_copy_file(&format!("{}.png", copy_stem("a.webp"))));
        assert!(!is_copy_file("index.json") && !is_copy_file("mine.jpg"));
    }

    #[test]
    fn converters_get_full_paths_and_no_shell() {
        let d = std::env::temp_dir();
        let input = d.join("-x.webp");
        for (_, prog, args) in converters(Format::Webp, &input, &d.join("o.png")) {
            assert!(prog.is_absolute() || prog.is_file(), "{prog:?}");
            assert!(args.iter().all(|a| !a.starts_with("-x")), "a file name is never read as an option: {args:?}");
            assert!(!args.iter().any(|a| a.contains(';') || a.contains('|')), "{args:?}");
        }
    }

    #[test]
    fn the_package_is_placed_but_never_over_someone_elses() {
        let d = std::env::temp_dir().join(format!("texora-img-pkg-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let p = d.join("texoraimage.sty");
        let _ = std::fs::remove_file(&p);
        place_package(&d);
        assert_eq!(std::fs::read_to_string(&p).unwrap(), STY);
        std::fs::write(&p, format!("{MARK}an older TeXora copy\n")).unwrap();
        place_package(&d);
        assert_eq!(std::fs::read_to_string(&p).unwrap(), STY, "TeXora's own older copy is updated");
        std::fs::write(&p, "% my own package of that name\n").unwrap();
        place_package(&d);
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "% my own package of that name\n", "someone else's file is kept");
    }

    #[test]
    fn problems_are_reported_in_plain_words() {
        let d = std::env::temp_dir().join(format!("texora-img-probs-{}", std::process::id()));
        std::fs::create_dir_all(d.join("images")).unwrap();
        std::fs::write(d.join("images/fake.webp"), "#EXTM3U\nhttp://example.com/x\n").unwrap();
        std::fs::write(d.join("images/old.xcf"), "gimp xcf v011").unwrap();
        std::fs::write(d.join("images/ok.png"), "not read: LaTeX takes PNG itself").unwrap();
        let tex = d.join("p.tex");
        std::fs::write(&tex, "\\usepackage{texoraimage}\n\\includeimage{images/missing.heic}\n\\includeimage{images/fake.webp}\n\\includeimage{images/old.xcf}\n\\includeimage{../outside.avif}\n\\includeimage{images/ok.png}\n").unwrap();
        let r = prepare(&tex);
        let got: Vec<(usize, &str)> = r.problems.iter().map(|(_, l, m)| (*l, m.as_str())).collect();
        assert_eq!(got.len(), 4, "{got:?}");
        assert!(got[0].0 == 2 && got[0].1.starts_with("Picture not found: images/missing.heic"), "{got:?}");
        assert!(got[1].0 == 3 && got[1].1.contains("not really a WebP picture"), "{got:?}");
        assert!(got[2].0 == 4 && got[2].1.contains("not supported"), "{got:?}");
        assert!(got[3].0 == 5 && got[3].1.contains("only pictures inside the presentation's folder"), "{got:?}");
        assert!(d.join("texoraimage.sty").is_file(), "the package is placed beside the presentation");
        assert!(!d.join(FOLDER).join(format!("{}.jpg", copy_stem("images/ok.png"))).exists(), "PNG is not converted");
    }

    fn gradient() -> image::RgbaImage {
        image::RgbaImage::from_fn(64, 48, |x, y| image::Rgba([(x * 4) as u8, (y * 5) as u8, 120, 255]))
    }

    /// GIF, TIFF, BMP and WebP pictures, written here by the image crate, are decoded inside TeXora
    /// (no converter on the computer is used), and an opaque picture becomes a JPEG, a see-through one
    /// a PNG. Runs everywhere, CI included.
    #[test]
    fn free_formats_are_decoded_inside_texora() {
        let d = std::env::temp_dir().join(format!("texora-img-builtin-{}", std::process::id()));
        let out = d.join(FOLDER);
        std::fs::create_dir_all(&out).unwrap();
        let rgb = image::DynamicImage::ImageRgba8(gradient()).to_rgb8();
        let cases = [(Format::Gif, image::ImageFormat::Gif, "p.gif"), (Format::Tiff, image::ImageFormat::Tiff, "p.tiff"), (Format::Bmp, image::ImageFormat::Bmp, "p.bmp"), (Format::Webp, image::ImageFormat::WebP, "p.webp")];
        for (format, written, name) in cases {
            let file = d.join(name);
            if format == Format::Webp {
                image::DynamicImage::ImageRgba8(gradient()).save_with_format(&file, written).unwrap();
            } else {
                image::DynamicImage::ImageRgb8(rgb.clone()).save_with_format(&file, written).unwrap();
            }
            assert!(format.matches(&crate::video::head(&file).unwrap()), "{format:?} first bytes");
            let img = decode_built_in(format, &file).unwrap_or_else(|e| panic!("{format:?}: {e}"));
            assert_eq!((img.width(), img.height()), (64, 48), "{format:?}");
            let px = img.to_rgb8().get_pixel(10, 20).0;
            assert!(px[0].abs_diff(40) < 12 && px[1].abs_diff(100) < 12 && px[2].abs_diff(120) < 12, "{format:?} colour {px:?}");
            let (copy, by) = convert(format, &file, &out, &format!("{:032X}", format as u8)).unwrap();
            assert_eq!(by, "built-in", "{format:?} needs no converter on the computer");
            assert!(copy.ends_with(".jpg"), "an opaque picture becomes a JPEG");
        }
        // Transparency is kept: a see-through WebP becomes a PNG.
        let clear = image::RgbaImage::from_fn(32, 32, |x, _| image::Rgba([200, 30, 30, if x < 16 { 0 } else { 255 }]));
        let file = d.join("clear.webp");
        image::DynamicImage::ImageRgba8(clear).save_with_format(&file, image::ImageFormat::WebP).unwrap();
        let (copy, _) = convert(Format::Webp, &file, &out, "C1EA2C1EA2C1EA2C1EA2C1EA2C1EA2C1").unwrap();
        assert!(copy.ends_with(".png"), "{copy}");
        assert_eq!(image::open(out.join(copy)).unwrap().to_rgba8().get_pixel(2, 2).0[3], 0);
    }

    /// AVIF and JPEG XL pictures made by an encoder on this computer, if there is one, are decoded
    /// inside TeXora (avif-decode with rav1d, jxl-oxide). Skipped where there is no encoder.
    #[test]
    fn avif_and_jpeg_xl_are_decoded_inside_texora() {
        let d = std::env::temp_dir().join(format!("texora-img-av1-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let png = d.join("src.png");
        gradient().save(&png).unwrap();
        let makers: [(Format, &str, Vec<String>); 2] = [
            (Format::Avif, "avifenc", vec![crate::video::plain(&png), crate::video::plain(&d.join("p.avif"))]),
            (Format::Jxl, "cjxl", vec![crate::video::plain(&png), crate::video::plain(&d.join("p.jxl"))]),
        ];
        for (format, maker, args) in makers {
            let Some(tool) = crate::latex::find_tool(maker) else { continue };
            let mut c = crate::video::tool_command(&tool);
            c.args(&args);
            let _ = crate::video::run_for(c, Duration::from_secs(60));
            let file = PathBuf::from(args.last().unwrap());
            if !file.is_file() {
                continue;
            }
            assert!(format.matches(&crate::video::head(&file).unwrap()), "{format:?} first bytes");
            let img = decode_built_in(format, &file).unwrap_or_else(|e| panic!("{format:?}: {e}"));
            assert_eq!((img.width(), img.height()), (64, 48), "{format:?}");
            let px = img.to_rgb8().get_pixel(10, 20).0;
            assert!(px[0].abs_diff(40) < 16 && px[1].abs_diff(100) < 16 && px[2].abs_diff(120) < 16, "{format:?} colour {px:?}");
        }
    }

    /// HEIC/HEIF (HEVC, patented) is never decoded inside TeXora: only converters on the computer.
    #[test]
    fn heic_is_left_to_converters_on_the_computer() {
        assert!(!Format::Heic.built_in());
        assert!(decode_built_in(Format::Heic, Path::new("/x.heic")).is_err());
        assert!([Format::Avif, Format::Webp, Format::Jxl, Format::Gif, Format::Tiff, Format::Bmp].iter().all(|f| f.built_in()));
    }
}
