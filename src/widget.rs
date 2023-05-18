use eframe::{
    egui::{self, Response, RichText, Ui},
    epaint::Color32,
};

pub fn error_button(ui: &mut Ui, text: impl Into<String>) -> Response {
    ui.add(egui::Button::new(RichText::new(text).color(Color32::WHITE)).fill(Color32::RED))
}

pub fn info_row(ui: &mut Ui, l: impl Into<String>, r: impl Into<String>) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(l).size(20.0));
        ui.label(RichText::new(r).size(20.0));
    });
}
