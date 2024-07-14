pub struct HelpText<'a> {
    text: &'a str,
}

impl<'a> HelpText<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }
}

impl<'a> egui::Widget for HelpText<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.add(egui::Label::new(egui::RichText::new("( ? )")).sense(egui::Sense::hover()))
            .on_hover_cursor(egui::CursorIcon::Help)
            .on_hover_text(self.text)
    }
}
