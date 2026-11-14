//! Makes sure the PDF engine (PDFium, a prebuilt library) sits next to the program.
//! It is fetched once for the current platform if it is not already in vendor/.
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn platform() -> Option<(&'static str, &'static str)> {
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    match (os.as_str(), arch.as_str()) {
        ("macos", "aarch64") => Some(("mac-arm64", "libpdfium.dylib")),
        ("macos", "x86_64") => Some(("mac-x64", "libpdfium.dylib")),
        ("linux", "x86_64") => Some(("linux-x64", "libpdfium.so")),
        ("linux", "aarch64") => Some(("linux-arm64", "libpdfium.so")),
        ("windows", "x86_64") => Some(("win-x64", "pdfium.dll")),
        ("windows", "aarch64") => Some(("win-arm64", "pdfium.dll")),
        _ => None,
    }
}

fn find_lib(dir: &Path, name: &str) -> Option<PathBuf> {
    [dir.join("lib").join(name), dir.join("bin").join(name)].into_iter().find(|p| p.exists())
}

/// Puts the icon inside the Windows program, so Explorer and the taskbar show it.
fn windows_icon() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=assets/icon.rc");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("icon.res");
    // Cross-building from a Mac uses Zig's resource compiler; on Windows the normal one is used.
    let zig = Command::new("zig").arg("rc").arg("/fo").arg(&out).arg("icon.rc").current_dir("assets").status();
    if zig.map(|s| s.success()).unwrap_or(false) {
        println!("cargo:rustc-link-arg-bins={}", out.display());
    } else if embed_resource::compile("assets/icon.rc", embed_resource::NONE).manifest_optional().is_err() {
        println!("cargo:warning=The Windows icon could not be added to the program.");
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        windows_icon();
    }
    println!("cargo:rerun-if-changed=vendor/pdfium");
    let Some((plat, name)) = platform() else { return };
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let dir = root.join("vendor/pdfium").join(plat);
    if find_lib(&dir, name).is_none() {
        let _ = fs::create_dir_all(&dir);
        let url = format!("https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-{plat}.tgz");
        let tgz = dir.join("pdfium.tgz");
        let ok = Command::new("curl")
            .args(["-sSfL", "-o"])
            .arg(&tgz)
            .arg(&url)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ok {
            let _ = Command::new("tar").arg("xzf").arg(&tgz).arg("-C").arg(&dir).status();
        }
        if find_lib(&dir, name).is_none() {
            println!("cargo:warning=PDF engine could not be fetched for {plat}; put {name} next to the program.");
            return;
        }
    }
    let lib = find_lib(&dir, name).unwrap();
    // The program goes in the folder holding `build`: OUT_DIR is target/<profile>/build/texora-xxx/out
    // up to Rust 1.99, and target/<profile>/build/texora/xxx/out from Rust 1.100. Counting folders up
    // from OUT_DIR put the engine in the wrong place with 1.100, so the nearest `build` is looked for.
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let Some(profile_dir) = out.ancestors().find(|p| p.file_name().is_some_and(|n| n == "build")).and_then(Path::parent) else {
        panic!("cannot tell where the program goes from OUT_DIR {}; put {name} next to the program", out.display());
    };
    // A package without the engine cannot open any PDF, so a failed copy stops the build. The same
    // file already there is left alone (Windows refuses to replace a DLL a running TeXora has open).
    let dest = profile_dir.join(name);
    let len = |p: &Path| fs::metadata(p).map(|m| m.len()).ok();
    if len(&dest).is_some() && len(&dest) == len(&lib) {
        return;
    }
    if let Err(e) = fs::copy(&lib, &dest) {
        panic!("could not put {} next to the program in {}: {e}", lib.display(), profile_dir.display());
    }
}
