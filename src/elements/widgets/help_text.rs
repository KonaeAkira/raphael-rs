pub struct HelpText {
    text: egui::WidgetText,
}

impl HelpText {
    pub fn new(text: impl Into<egui::WidgetText>) -> Self {
        Self { text: text.into() }
    }
}

impl egui::Widget for HelpText {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.add(egui::Label::new(egui::RichText::new("( ? )")).sense(egui::Sense::hover()))
            .on_hover_cursor(egui::CursorIcon::Help)
            .on_hover_text(self.text)
    }
}
