use std::time::Duration;

use egui::Image;

#[derive(Clone, Debug)]
pub struct Resource {
    /// HTTP response
    pub response: ehttp::Response,
    pub timing: Duration,
    pub text: Option<String>,

    /// If set, the response was an image.
    pub image: Option<Image<'static>>,

    /// If set, the response was text with some supported syntax highlighting (e.g. ".rs" or ".md").
    pub colored_text: Option<ColoredText>,
}

impl Resource {
    pub fn from_response(
        ctx: &egui::Context,
        response: ehttp::Response,
        elapsed: Duration,
    ) -> Self {
        let content_type = response.content_type().unwrap_or_default();
        if content_type.starts_with("image/") {
            ctx.include_bytes(response.url.clone(), response.bytes.clone());
            let image = Image::from_uri(response.url.clone());

            Self {
                response,
                timing: elapsed,
                text: None,
                colored_text: None,
                image: Some(image),
            }
        } else {
            let text = response.text();
            let colored_text = text.and_then(|text| syntax_highlighting(ctx, &response, text));
            let text = text.map(|text| text.to_owned());

            Self {
                response,
                timing: elapsed,
                text,
                colored_text,
                image: None,
            }
        }
    }
}

// ----------------------------------------------------------------------------
// Syntax highlighting:

fn syntax_highlighting(
    ctx: &egui::Context,
    response: &ehttp::Response,
    text: &str,
) -> Option<ColoredText> {
    let extension_and_rest: Vec<&str> = response.url.rsplitn(2, '.').collect();
    let extension = extension_and_rest.first()?;
    let theme = egui_extras::syntax_highlighting::CodeTheme::from_style(&ctx.style());
    Some(ColoredText(egui_extras::syntax_highlighting::highlight(
        ctx, &theme, text, extension,
    )))
}

#[derive(Clone, Debug)]
pub struct ColoredText(egui::text::LayoutJob);

impl ColoredText {
    pub fn ui(&self, ui: &mut egui::Ui) -> egui::Response {
        let mut job = self.0.clone();
        job.wrap.max_width = ui.available_width();
        let galley = ui.fonts(|f| f.layout_job(job));
        ui.add(egui::Label::new(galley).selectable(true))
    }
}
