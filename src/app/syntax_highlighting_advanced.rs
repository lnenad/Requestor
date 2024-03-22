//! Syntax highlighting for code.
//!
//! Turn on the `syntect` feature for great syntax highlighting of any language.
//! Otherwise, a very simple fallback will be used, that works okish for C, C++, Rust, and Python.

#![allow(clippy::mem_forget)] // False positive from enum_map macro

use std::cmp::{max, min};

use egui::{text::LayoutJob, Event};

/// View some code with syntax highlighting and selection.
pub fn code_view_ui(
    ui: &mut egui::Ui,
    theme: &CodeTheme,
    code: &str,
    language: &str,
    scroll_state: &mut i64,
    last_scroll_state: &mut i64,
    layout_job: &mut LayoutJob,
    _wrap_text: &mut bool,
) -> egui::Response {
    let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
        let mut layout_job: egui::text::LayoutJob = Highlighter::default()
            .highlight(
                theme,
                code,
                language,
                scroll_state,
                last_scroll_state,
                layout_job,
            )
            .unwrap();
        // let mut layout_job = LayoutJob::simple(
        //     code.to_owned(),
        //     egui::FontId::monospace(12.0),
        //     egui::Color32::LIGHT_GRAY,
        //     wrap_width,
        // );
        layout_job.wrap.max_width = wrap_width;
        ui.fonts(|f| f.layout_job(layout_job))
    };
    let response = ui.add_sized(
        ui.available_size(),
        egui::TextEdit::multiline(&mut code.to_owned()).layouter(&mut layouter),
    );

    response.ctx.input(|input_state| {
        for ev in &input_state.raw.events {
            if let Event::Scroll(vec) = ev {
                *scroll_state += vec.y as i64;
                if *scroll_state > 0 {
                    *scroll_state = 0;
                }
                println!("{:?}", scroll_state);
            }
        }
    });
    response
    // ui.add(
    //     egui::Label::new(layout_job)
    //         .wrap(*wrap_text)
    //         .selectable(true),
    // )
}

#[cfg(feature = "syntect")]
#[derive(Clone, Copy, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
enum SyntectTheme {
    Base16EightiesDark,
    Base16MochaDark,
    Base16OceanDark,
    Base16OceanLight,
    InspiredGitHub,
    SolarizedDark,
    SolarizedLight,
}

#[cfg(feature = "syntect")]
impl SyntectTheme {
    fn all() -> impl ExactSizeIterator<Item = Self> {
        [
            Self::Base16EightiesDark,
            Self::Base16MochaDark,
            Self::Base16OceanDark,
            Self::Base16OceanLight,
            Self::InspiredGitHub,
            Self::SolarizedDark,
            Self::SolarizedLight,
        ]
        .iter()
        .copied()
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Base16EightiesDark => "Base16 Eighties (dark)",
            Self::Base16MochaDark => "Base16 Mocha (dark)",
            Self::Base16OceanDark => "Base16 Ocean (dark)",
            Self::Base16OceanLight => "Base16 Ocean (light)",
            Self::InspiredGitHub => "InspiredGitHub (light)",
            Self::SolarizedDark => "Solarized (dark)",
            Self::SolarizedLight => "Solarized (light)",
        }
    }

    fn syntect_key_name(&self) -> &'static str {
        match self {
            Self::Base16EightiesDark => "base16-eighties.dark",
            Self::Base16MochaDark => "base16-mocha.dark",
            Self::Base16OceanDark => "base16-ocean.dark",
            Self::Base16OceanLight => "base16-ocean.light",
            Self::InspiredGitHub => "InspiredGitHub",
            Self::SolarizedDark => "Solarized (dark)",
            Self::SolarizedLight => "Solarized (light)",
        }
    }

    pub fn is_dark(&self) -> bool {
        match self {
            Self::Base16EightiesDark
            | Self::Base16MochaDark
            | Self::Base16OceanDark
            | Self::SolarizedDark => true,

            Self::Base16OceanLight | Self::InspiredGitHub | Self::SolarizedLight => false,
        }
    }
}

/// A selected color theme.
#[derive(Clone, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct CodeTheme {
    dark_mode: bool,

    syntect_theme: SyntectTheme,
}

impl Default for CodeTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl CodeTheme {
    /// Selects either dark or light theme based on the given style.
    #[allow(dead_code)]
    pub fn from_style(style: &egui::Style) -> Self {
        if style.visuals.dark_mode {
            Self::dark()
        } else {
            Self::light()
        }
    }

    /// Load code theme from egui memory.
    ///
    /// There is one dark and one light theme stored at any one time.
    #[allow(dead_code)]
    pub fn from_memory(ctx: &egui::Context) -> Self {
        if ctx.style().visuals.dark_mode {
            ctx.data_mut(|d| {
                d.get_persisted(egui::Id::new("dark"))
                    .unwrap_or_else(Self::dark)
            })
        } else {
            ctx.data_mut(|d| {
                d.get_persisted(egui::Id::new("light"))
                    .unwrap_or_else(Self::light)
            })
        }
    }

    /// Store theme to egui memory.
    ///
    /// There is one dark and one light theme stored at any one time.
    #[allow(dead_code)]
    pub fn store_in_memory(self, ctx: &egui::Context) {
        if self.dark_mode {
            ctx.data_mut(|d| d.insert_persisted(egui::Id::new("dark"), self));
        } else {
            ctx.data_mut(|d| d.insert_persisted(egui::Id::new("light"), self));
        }
    }
}

#[cfg(feature = "syntect")]
impl CodeTheme {
    pub fn dark() -> Self {
        Self {
            dark_mode: true,
            syntect_theme: SyntectTheme::Base16MochaDark,
        }
    }

    #[allow(dead_code)]
    pub fn light() -> Self {
        Self {
            dark_mode: false,
            syntect_theme: SyntectTheme::SolarizedLight,
        }
    }

    /// Show UI for changing the color theme.
    #[allow(dead_code)]
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        egui::widgets::global_dark_light_mode_buttons(ui);

        for theme in SyntectTheme::all() {
            if theme.is_dark() == self.dark_mode {
                ui.radio_value(&mut self.syntect_theme, theme, theme.name());
            }
        }
    }
}

#[cfg(feature = "syntect")]
struct Highlighter {
    ps: syntect::parsing::SyntaxSet,
    ts: syntect::highlighting::ThemeSet,
}

#[cfg(feature = "syntect")]
impl Default for Highlighter {
    fn default() -> Self {
        Self {
            ps: syntect::parsing::SyntaxSet::load_defaults_newlines(),
            ts: syntect::highlighting::ThemeSet::load_defaults(),
        }
    }
}

impl Highlighter {
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn highlight(
        &self,
        theme: &CodeTheme,
        text: &str,
        language: &str,
        scroll_state: &i64,
        last_scroll_state: &mut i64,
        layout_job: &mut LayoutJob,
    ) -> Option<LayoutJob> {
        println!(
            "Diff: {}",
            scroll_state.clone().abs_diff(last_scroll_state.clone())
        );
        if (scroll_state.clone().abs_diff(last_scroll_state.clone()) < 1000)
            && layout_job.sections.len() != 0
        {
            return Some(layout_job.clone());
        }
        *last_scroll_state = *scroll_state;
        use syntect::easy::HighlightLines;
        use syntect::highlighting::FontStyle;
        use syntect::util::LinesWithEndings;

        let syntax = self
            .ps
            .find_syntax_by_name(language)
            .or_else(|| self.ps.find_syntax_by_extension(language))?;

        let theme = theme.syntect_theme.syntect_key_name();
        let mut h = HighlightLines::new(syntax, &self.ts.themes[theme]);

        use egui::text::{LayoutSection, TextFormat};

        let mut job = LayoutJob {
            text: text.into(),
            ..Default::default()
        };

        let trail = 20000;
        let start = max(0, scroll_state.abs() - trail) as usize; // Starting character index for syntax highlighting
        let end = min(text.len(), (scroll_state.abs() + trail) as usize); // Ending character index for syntax highlighting
        job.sections.push(LayoutSection {
            leading_space: 0.0,
            byte_range: 0..start,
            format: TextFormat::simple(egui::FontId::monospace(12.0), egui::Color32::LIGHT_GRAY),
        });
        println!("Invoked {} {} {} {}", scroll_state, start, end, text.len());
        for line in LinesWithEndings::from(&text[start..end]) {
            for (style, range) in h.highlight_line(line, &self.ps).ok()? {
                let brange = as_byte_range(text, range);
                if !(start..end).contains(&brange.start) {
                    continue;
                }
                let fg = style.foreground;
                let text_color = egui::Color32::from_rgb(fg.r, fg.g, fg.b);
                let italics = style.font_style.contains(FontStyle::ITALIC);
                let underline = style.font_style.contains(FontStyle::ITALIC);
                let underline = if underline {
                    egui::Stroke::new(1.0, text_color)
                } else {
                    egui::Stroke::NONE
                };
                job.sections.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: brange,
                    format: TextFormat::simple(egui::FontId::monospace(12.0), text_color),
                });
            }
        }
        job.sections.push(LayoutSection {
            leading_space: 0.0,
            byte_range: end..text.len(),
            format: TextFormat::simple(egui::FontId::monospace(12.0), egui::Color32::LIGHT_GRAY),
        });

        *layout_job = job.clone();

        Some(job)

        // .unwrap_or_else(|| {
        //     // Fallback:
        //     LayoutJob::simple(
        //         code.into(),
        //         egui::FontId::monospace(12.0),
        //         if theme.dark_mode {
        //             egui::Color32::LIGHT_GRAY
        //         } else {
        //             egui::Color32::DARK_GRAY
        //         },
        //         f32::INFINITY,
        //     )
        // })
    }
}

#[cfg(feature = "syntect")]
fn as_byte_range(whole: &str, range: &str) -> std::ops::Range<usize> {
    let whole_start = whole.as_ptr() as usize;
    let range_start = range.as_ptr() as usize;
    assert!(whole_start <= range_start);
    assert!(range_start + range.len() <= whole_start + whole.len());
    let offset = range_start - whole_start;
    offset..(offset + range.len())
}

pub fn get_type_from_mime(mime: &str) -> &str {
    match mime {
        x if x.contains("application/json") => "json",
        x if x.contains("application/postscript") => "postscript",
        x if x.contains("application/mathml+xml") => "xml",
        x if x.contains("application/vnd.mozilla.xul+xml") => "xml",
        x if x.contains("application/xhtml+xml") => "xml",
        x if x.contains("application/xslt+xml") => "xml",
        x if x.contains("application/xml") => "xml",
        x if x.contains("application/xml-dtd") => "xml",
        x if x.contains("text/plain") => "text",
        x if x.contains("text/richtext") => "rtf",
        x if x.contains("text/rtf") => "rtf",
        x if x.contains("text/html") => "html",
        x if x.contains("text/calendar") => "calendar",
        x if x.contains("text/css") => "css",
        x if x.contains("text/sgml") => "sgml",
        x if x.contains("text/tab-separated-values") => "csv",
        _ => "",
    }
}
