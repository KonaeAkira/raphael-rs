use egui::{Align, Layout, Widget};
use simulator::Action;

pub struct MacroTextView {
    text: String,
}

impl MacroTextView {
    pub fn new(actions: &[Action], include_delay: bool) -> Self {
        let lines: Vec<_> = actions
            .into_iter()
            .map(|action| {
                if include_delay {
                    format!(
                        "/ac \"{}\" <wait.{}>",
                        action.display_name(),
                        action.time_cost()
                    )
                } else {
                    format!("/ac \"{}\"", action.display_name())
                }
            })
            .collect();
        Self {
            text: lines.join("\r\n"),
        }
    }
}

impl Widget for MacroTextView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.group(|ui| {
            ui.horizontal_top(|ui| {
                ui.monospace(&self.text);
                ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                    if ui.button("Copy").clicked() {
                        ui.output_mut(|output| output.copied_text = self.text);
                    }
                });
            });
        })
        .response
    }
}
