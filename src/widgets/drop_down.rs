use crate::widgets::util::max_text_width;

pub struct DropDown<'a, T, const N: usize, Formatter> {
    id: egui::Id,
    current_value: &'a mut T,
    selectable_values: [T; N],
    value_formatter: Formatter,
}

impl<'a, T, const N: usize, Formatter> DropDown<'a, T, N, Formatter> {
    pub fn new(
        id: impl std::hash::Hash,
        current_value: &'a mut T,
        selectable_values: [T; N],
        value_formatter: Formatter,
    ) -> Self {
        Self {
            id: egui::Id::new(id),
            current_value,
            selectable_values,
            value_formatter,
        }
    }
}

impl<'a, T, const N: usize, Formatter> egui::Widget for DropDown<'a, T, N, Formatter>
where
    T: Copy + PartialEq,
    Formatter: Copy + Fn(T) -> &'static str,
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let value_labels = self.selectable_values.map(self.value_formatter);

        // Calculate the maximum width beforehand to make sure the drop down does not resize
        // when selecting a new value.
        let max_label_width = max_text_width(ui, value_labels, egui::TextStyle::Button);
        let combo_box_width = max_label_width
            + ui.spacing().icon_spacing
            + ui.spacing().icon_width
            + 2.0 * ui.spacing().button_padding.x;

        egui::ComboBox::from_id_salt(self.id)
            .width(combo_box_width)
            .selected_text((self.value_formatter)(*self.current_value))
            .show_ui(ui, |ui| {
                for (value, value_label) in self.selectable_values.iter().zip(value_labels) {
                    ui.selectable_value(self.current_value, *value, value_label);
                }
            })
            .response
    }
}
