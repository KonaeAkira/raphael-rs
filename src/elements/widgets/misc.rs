pub fn add_sized_labeled_widget<'a>(
    ui: &mut egui::Ui,
    label: impl Into<egui::Atom<'a>>,
    size: impl Into<egui::Vec2>,
    widget: impl egui::Widget,
) {
    let custom_atom_id = ui.next_auto_id();
    let response = egui::AtomLayout::new((label.into(), egui::Atom::custom(custom_atom_id, size)))
        .allocate(ui)
        .paint(ui);
    if let Some(rect) = response.rect(custom_atom_id) {
        ui.put(rect, widget);
    }
}

pub fn collapse_persisted(ui: &mut egui::Ui, id: egui::Id, collapsed: &mut bool) {
    *collapsed = ui.data_mut(|data| *data.get_persisted_mut_or(id, *collapsed));
    let button_text = match collapsed {
        true => "⏵",
        false => "⏷",
    };
    if ui.button(button_text).clicked() {
        ui.data_mut(|data| data.insert_persisted(id, !*collapsed));
    }
}

pub fn collapse_temporary(ui: &mut egui::Ui, id: egui::Id, collapsed: &mut bool) {
    *collapsed = ui.data_mut(|data| *data.get_temp_mut_or(id, *collapsed));
    let button_text = match collapsed {
        true => "⏵",
        false => "⏷",
    };
    if ui.button(button_text).clicked() {
        ui.data_mut(|data| data.insert_temp(id, !*collapsed));
    }
}
