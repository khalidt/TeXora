//! The Accessibility checker, a sidebar beside Search, Analytics and Settings: it checks the open PDF
//! (any PDF, slides or not) and, when there is one, its LaTeX source, against WCAG 2.2, and lists
//! what fails, what a person must check, and what passes, each with what to do and a way to get there:
//! the page in the PDF, or the line in the source. It can also check the tagged copy, build it, and
//! run veraPDF when that is installed.
use crate::app::*;
use crate::pdfcheck::{self, Report, SourceSpot, Status, Validation};
use egui::*;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

/// The checker's state, kept in the app.
#[derive(Default)]
pub struct Panel {
    pub report: Option<Report>,
    rx: Option<Receiver<Report>>,
    /// The PDF the report (or the running check) is for.
    for_path: Option<PathBuf>,
    pub validation: Option<Validation>,
    vera_rx: Option<Receiver<Validation>>,
    show_passed: bool,
    /// Check the tagged copy (`<name>.acc.pdf`) instead of the open PDF.
    copy: bool,
}

fn status_color(s: Status) -> Color32 {
    match s {
        Status::Fail => BAD,
        Status::Review => WARN,
        Status::Pass => LEAF,
        Status::NotApplicable => MUTED,
    }
}

fn status_mark(s: Status) -> &'static str {
    match s {
        Status::Fail => "✗",
        Status::Review => "?",
        Status::Pass => "✓",
        Status::NotApplicable => "–",
    }
}

impl App {
    /// The tagged copy beside the open document, when it has been built.
    fn tagged_copy(&self) -> Option<PathBuf> {
        let base = self.src.as_ref().map(|s| s.path.clone()).or_else(|| self.pdf_path.clone())?;
        let stem = base.file_stem()?.to_string_lossy().to_string();
        let p = base.with_file_name(format!("{stem}.acc.pdf"));
        p.is_file().then_some(p)
    }

    /// The PDF to check: the open one, or its tagged copy.
    fn check_target(&self) -> Option<PathBuf> {
        if self.access_panel.copy {
            if let Some(c) = self.tagged_copy() {
                return Some(c);
            }
        }
        self.pdf_path.clone()
    }

    fn start_pdf_check(&mut self, ctx: &Context) {
        let Some(pdf) = self.check_target() else { return };
        let pages = self.doc.as_ref().map(pdfcheck::page_summaries).unwrap_or_default();
        let bookmarks = self.doc.as_ref().map(|d| d.outline.len()).unwrap_or(0);
        // The source is the open .tex, or one beside the PDF with the same name.
        let source = self.src.as_ref().map(|s| s.path.clone()).or_else(|| {
            let t = self.pdf_path.as_ref()?.with_extension("tex");
            t.is_file().then_some(t)
        });
        let (tx, rx) = std::sync::mpsc::channel();
        self.access_panel.rx = Some(rx);
        self.access_panel.for_path = Some(pdf.clone());
        self.access_panel.validation = None;
        let wake = ctx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(pdfcheck::evaluate(&pdf, &pages, bookmarks, source.as_deref()));
            wake.request_repaint();
        });
    }

    fn start_validation(&mut self, ctx: &Context) {
        let (Some(vera), Some(r)) = (pdfcheck::verapdf(), self.access_panel.report.as_ref()) else { return };
        let (pdf, part) = (r.pdf.clone(), r.facts.ua_part);
        let (tx, rx) = std::sync::mpsc::channel();
        self.access_panel.vera_rx = Some(rx);
        let wake = ctx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(pdfcheck::validate(&vera, &pdf, part));
            wake.request_repaint();
        });
    }

    /// Goes to the place an issue is about: the page in the PDF, or the line in the source.
    fn goto_issue(&mut self, page: Option<usize>, source: Option<SourceSpot>) {
        if let Some((file, line, col, len)) = source {
            if self.src.as_ref().is_some_and(|s| s.path == file) {
                self.view = View::Studio;
                self.log_tab = LogTab::Access;
                self.log_open = true;
                if let Some(s) = &mut self.src {
                    s.goto_line = Some(line);
                    s.goto_select = (col > 0 && len > 0).then_some((col, len));
                }
            } else {
                self.open_path(&file);
            }
        } else if let Some(p) = page {
            if let Some(slide) = self.deck.slides.iter().position(|s| s.page == p) {
                self.jump(slide);
            }
        }
    }

    pub fn accessibility_tab(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        // Results from the background.
        if let Some(rx) = &self.access_panel.rx {
            if let Ok(r) = rx.try_recv() {
                self.access_panel.report = Some(r);
                self.access_panel.rx = None;
            }
        }
        if let Some(rx) = &self.access_panel.vera_rx {
            if let Ok(v) = rx.try_recv() {
                self.access_panel.validation = Some(v);
                self.access_panel.vera_rx = None;
            }
        }
        let Some(target) = self.check_target() else {
            ui.label(RichText::new("Open a PDF or a LaTeX presentation to check it.").color(MUTED));
            return;
        };
        // A new document (or the other choice of file) is checked by itself.
        if self.access_panel.for_path.as_ref() != Some(&target) && self.access_panel.rx.is_none() {
            self.start_pdf_check(&ctx);
        }
        let busy = self.access_panel.rx.is_some();
        let copy = self.tagged_copy();
        ScrollArea::vertical().id_salt("access-check").auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 5.0;
            ui.label(RichText::new("Checks this PDF against WCAG 2.2 and PDF/UA, and its LaTeX source when it has one. Automated: a person still checks what the list marks \u{201c}check by hand\u{201d}.").size(12.5).color(MUTED));
            // Which file.
            if let Some(c) = &copy {
                ui.horizontal_wrapped(|ui| {
                    let name = c.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                    if ui.radio(!self.access_panel.copy, "This PDF").clicked() {
                        self.access_panel.copy = false;
                    }
                    if ui.radio(self.access_panel.copy, format!("Tagged copy ({name})")).on_hover_text("The tagged copy Build tagged copy made; page checks use the slides' own pages").clicked() {
                        self.access_panel.copy = true;
                    }
                });
            }
            let name = target.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            ui.label(RichText::new(name).size(13.5).strong().color(INK));
            ui.horizontal_wrapped(|ui| {
                if ui.add_enabled(!busy, Button::new("Check again")).clicked() {
                    self.start_pdf_check(&ctx);
                }
                if self.src.is_some() {
                    if ui.button("Build tagged copy").on_hover_text("A tagged copy of the slides for screen readers (not validated); the source and its PDF stay as they are").clicked() {
                        self.start_access_build(&ctx);
                    }
                    if ui.button("Open the source check").on_hover_text("The studio's Accessibility tab: every finding in the source, line by line").clicked() {
                        self.open_accessibility();
                    }
                }
            });
            if busy {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(RichText::new("Checking…").color(MUTED));
                });
            }
            let Some(report) = self.access_panel.report.clone() else { return };
            // The summary.
            ui.add_space(2.0);
            ui.horizontal_wrapped(|ui| {
                for s in [Status::Fail, Status::Review, Status::Pass, Status::NotApplicable] {
                    let n = report.count(s);
                    let (r, _) = ui.allocate_exact_size(vec2(84.0, 42.0), Sense::hover());
                    ui.painter().rect_filled(r, 6.0, status_color(s).gamma_multiply(0.12));
                    ui.painter().text(r.center_top() + vec2(0.0, 6.0), Align2::CENTER_TOP, n.to_string(), FontId::proportional(17.0), status_color(s));
                    ui.painter().text(r.center_bottom() - vec2(0.0, 5.0), Align2::CENTER_BOTTOM, s.word(), FontId::proportional(10.5), status_color(s));
                }
            });
            // Beamer slides are never tagged themselves: say so, and point at the copy that is.
            let beamer = report.source.as_ref().and_then(|s| std::fs::read_to_string(s).ok()).is_some_and(|t| crate::access::beamer_based(report.source.as_ref().and_then(|s| s.parent()).unwrap_or(std::path::Path::new(".")), &t));
            if beamer && !report.facts.tagged {
                let text = if copy.is_some() {
                    "These are Beamer slides, which cannot be tagged themselves: present this PDF, and give screen-reader users the tagged copy. Choose \u{201c}Tagged copy\u{201d} above to check it."
                } else {
                    "These are Beamer slides, which cannot be tagged themselves: present this PDF, and give screen-reader users a tagged copy. Build tagged copy makes one; then check it here."
                };
                Frame::new().fill(LEAF_SOFT).corner_radius(5).inner_margin(Margin::symmetric(8, 6)).show(ui, |ui| {
                    ui.label(RichText::new(text).size(12.5).color(LEAF_DARK));
                });
            }
            if let Some(src) = &report.source {
                ui.label(RichText::new(format!("Source checked too: {}", src.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default())).size(12.0).color(MUTED));
            }
            // veraPDF.
            match pdfcheck::verapdf() {
                Some(_) => {
                    let part = report.facts.ua_part.unwrap_or(2);
                    ui.horizontal_wrapped(|ui| {
                        let running = self.access_panel.vera_rx.is_some();
                        if ui.add_enabled(!running, Button::new(format!("Validate with veraPDF (PDF/UA-{part})"))).clicked() {
                            self.start_validation(&ctx);
                        }
                        if running {
                            ui.spinner();
                        }
                    });
                    if let Some(v) = &self.access_panel.validation {
                        if let Some(e) = &v.error {
                            ui.label(RichText::new(e).size(12.5).color(BAD));
                        } else if v.compliant {
                            ui.label(RichText::new(format!("✓ veraPDF: compliant ({})", v.profile)).size(12.5).color(LEAF_DARK));
                        } else {
                            ui.label(RichText::new(format!("✗ veraPDF: not compliant ({}), {} rule(s) failed:", v.profile, v.rules.len())).size(12.5).color(BAD));
                            for r in &v.rules {
                                ui.label(RichText::new(format!("{}-{} (×{}): {}", r.clause, r.test, r.failed, r.description)).size(12.0).color(INK));
                            }
                        }
                    }
                }
                None => {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("For a full PDF/UA validation, install").size(12.0).color(MUTED));
                        ui.hyperlink_to(RichText::new("veraPDF").size(12.0), "https://verapdf.org/software/");
                        ui.label(RichText::new("(free); TeXora then runs it here.").size(12.0).color(MUTED));
                    });
                }
            }
            ui.checkbox(&mut self.access_panel.show_passed, "Show what passes");
            ui.separator();
            // The criteria, by principle; failures first.
            let mut go: Option<(Option<usize>, Option<SourceSpot>)> = None;
            let checking_open_pdf = !self.access_panel.copy || copy.is_none();
            for principle in ["Perceivable", "Operable", "Understandable", "Robust", "PDF"] {
                let mut list: Vec<_> = report.criteria.iter().filter(|c| c.principle == principle).filter(|c| self.access_panel.show_passed || matches!(c.status, Status::Fail | Status::Review)).collect();
                if list.is_empty() {
                    continue;
                }
                list.sort_by_key(|c| c.status);
                let heading = if principle == "PDF" { "The PDF file (PDF/UA)" } else { principle };
                ui.label(RichText::new(heading).size(13.5).strong().color(INK));
                for c in list {
                    let color = status_color(c.status);
                    let head = RichText::new(format!("{}  {}  {}", status_mark(c.status), c.id, c.name)).size(13.0).color(color);
                    CollapsingHeader::new(head).id_salt(("access-criterion", c.id)).default_open(c.status == Status::Fail).show(ui, |ui| {
                        ui.label(RichText::new(format!("{}. {}", c.status.word(), c.summary)).size(12.5).color(MUTED));
                        for i in c.issues.iter() {
                            Frame::new().fill(color.gamma_multiply(0.06)).corner_radius(5).inner_margin(Margin::symmetric(8, 5)).show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(RichText::new(&i.text).size(12.5).color(INK));
                                if !i.fix.is_empty() {
                                    ui.horizontal_wrapped(|ui| {
                                        ui.spacing_mut().item_spacing.x = 4.0;
                                        ui.label(RichText::new("How to fix:").size(12.5).strong().color(LEAF_DARK));
                                        ui.label(RichText::new(&i.fix).size(12.5).color(INK));
                                    });
                                }
                                ui.horizontal(|ui| {
                                    if let Some((_, line, _, _)) = &i.source {
                                        if ui.small_button(format!("Go to line {line}")).clicked() {
                                            go = Some((None, i.source.clone()));
                                        }
                                    }
                                    if let (Some(p), true) = (i.page, checking_open_pdf) {
                                        if ui.small_button(format!("Go to page {}", p + 1)).clicked() {
                                            go = Some((Some(p), None));
                                        }
                                    }
                                });
                            });
                        }
                    });
                }
                ui.add_space(4.0);
            }
            if let Some((page, source)) = go {
                self.goto_issue(page, source);
            }
            ui.add_space(6.0);
        });
    }
}
