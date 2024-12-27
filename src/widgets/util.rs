/// Adds a "collapse" button to the UI and returns the collapsed state
pub fn collapse_button(ui: &mut egui::Ui, id: egui::Id) -> bool {
    let collapsed = ui.data_mut(|data| *data.get_persisted_mut_or_default(id));
    let button_text = match collapsed {
        true => "⏵",
        false => "⏷",
    };
    if ui.button(button_text).clicked() {
        ui.data_mut(|data| data.insert_persisted(id, !collapsed));
    }
    collapsed
}
