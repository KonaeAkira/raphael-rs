use raphael_data::{
    Consumable, CrafterStats, Locale, control_bonus, cp_bonus, craftsmanship_bonus,
};
use raphael_sim::*;
use raphael_translations::{t, t_format};

pub fn effect_string(
    consumable: Consumable,
    crater_stats: &CrafterStats,
    locale: Locale,
) -> String {
    let CrafterStats {
        craftsmanship,
        control,
        cp,
        ..
    } = crater_stats;
    let consumable_slice = &[Some(consumable)];

    let mut effect: String = String::new();
    if consumable.craft_rel != 0 {
        effect.push_str(&t_format!(
            locale,
            "Crafts. +{rel}% ({val}), ",
            rel = consumable.craft_rel,
            val = craftsmanship_bonus(*craftsmanship, consumable_slice)
        ));
    }
    if consumable.control_rel != 0 {
        effect.push_str(&t_format!(
            locale,
            "Control +{rel}% ({val}), ",
            rel = consumable.control_rel,
            val = control_bonus(*control, consumable_slice)
        ));
    }
    if consumable.cp_rel != 0 {
        effect.push_str(&t_format!(
            locale,
            "CP +{rel}% ({val}), ",
            rel = consumable.cp_rel,
            val = cp_bonus(*cp, consumable_slice)
        ));
    }
    effect.pop(); // This will potentially not work for JP & KR
    effect.pop();
    effect
}

pub fn text_width(ui: &egui::Ui, text: impl Into<String>) -> f32 {
    ui.fonts_mut(|fonts| {
        let galley = fonts.layout_no_wrap(
            text.into(),
            egui::FontId::default(),
            egui::Color32::default(),
        );
        galley.rect.width()
    })
}

pub fn max_text_width(ui: &egui::Ui, text_slice: &[impl ToString]) -> f32 {
    ui.fonts_mut(|fonts| {
        text_slice
            .iter()
            .map(|text| {
                let galley = fonts.layout_no_wrap(
                    text.to_string(),
                    egui::FontId::default(),
                    egui::Color32::default(),
                );
                galley.rect.width()
            })
            .fold(0.0, f32::max)
    })
}

pub enum TableColumnWidth {
    SelectButton,
    JobName,
    RelativeToRemainingClamped { scale: f32, min: f32, max: f32 },
    Remaining,
}

pub fn calculate_column_widths<const N: usize>(
    ui: &egui::Ui,
    desired_widths: [TableColumnWidth; N],
    locale: Locale,
) -> [f32; N] {
    let width_used_for_spacing = N.saturating_sub(1) as f32 * ui.spacing().item_spacing.x;
    let mut remaining_width = (ui.available_width() - width_used_for_spacing).max(0.0);
    desired_widths.map(|desired_width| {
        let exact_width = match desired_width {
            TableColumnWidth::SelectButton => {
                text_width(ui, t!(locale, "Select")) + 2.0 * ui.spacing().button_padding.x
            }
            TableColumnWidth::JobName => max_text_width(
                ui,
                match locale {
                    Locale::EN => &raphael_data::JOB_NAMES_EN,
                    Locale::DE => &raphael_data::JOB_NAMES_DE,
                    Locale::FR => &raphael_data::JOB_NAMES_FR,
                    Locale::JP => &raphael_data::JOB_NAMES_JP,
                    Locale::KR => &raphael_data::JOB_NAMES_KR,
                },
            ),
            TableColumnWidth::RelativeToRemainingClamped { scale, min, max } => {
                (remaining_width * scale).clamp(min, max)
            }
            TableColumnWidth::Remaining => remaining_width,
        };
        remaining_width = (remaining_width - exact_width).max(0.0);
        exact_width
    })
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

#[cfg(target_arch = "wasm32")]
pub fn get_action_icon(action: Action, job_id: u8) -> egui::Image<'static> {
    let image_path = format!(
        "{}/action-icons/{}/{}.webp",
        env!("BASE_URL"),
        raphael_data::get_job_name(job_id, raphael_data::Locale::EN),
        raphael_data::action_name(action, raphael_data::Locale::EN)
    );
    egui::Image::new(image_path)
}

#[cfg(not(target_arch = "wasm32"))]
macro_rules! action_icon {
    ( $name:literal, $job_id:expr ) => {
        match $job_id {
            0 => egui::include_image!(concat!("../../assets/action-icons/CRP/", $name, ".webp")),
            1 => egui::include_image!(concat!("../../assets/action-icons/BSM/", $name, ".webp")),
            2 => egui::include_image!(concat!("../../assets/action-icons/ARM/", $name, ".webp")),
            3 => egui::include_image!(concat!("../../assets/action-icons/GSM/", $name, ".webp")),
            4 => egui::include_image!(concat!("../../assets/action-icons/LTW/", $name, ".webp")),
            5 => egui::include_image!(concat!("../../assets/action-icons/WVR/", $name, ".webp")),
            6 => egui::include_image!(concat!("../../assets/action-icons/ALC/", $name, ".webp")),
            7 => egui::include_image!(concat!("../../assets/action-icons/CUL/", $name, ".webp")),
            _ => {
                log::warn!("Unknown job id {}. Falling back to job id 0.", $job_id);
                egui::include_image!(concat!("../../assets/action-icons/CRP/", $name, ".webp"))
            }
        }
    };
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_action_icon(action: Action, job_id: u8) -> egui::Image<'static> {
    egui::Image::new(match action {
        Action::BasicSynthesis => action_icon!("Basic Synthesis", job_id),
        Action::BasicTouch => action_icon!("Basic Touch", job_id),
        Action::MasterMend => action_icon!("Master's Mend", job_id),
        Action::Observe => action_icon!("Observe", job_id),
        Action::TricksOfTheTrade => action_icon!("Tricks of the Trade", job_id),
        Action::WasteNot => action_icon!("Waste Not", job_id),
        Action::Veneration => action_icon!("Veneration", job_id),
        Action::StandardTouch => action_icon!("Standard Touch", job_id),
        Action::GreatStrides => action_icon!("Great Strides", job_id),
        Action::Innovation => action_icon!("Innovation", job_id),
        Action::WasteNot2 => action_icon!("Waste Not II", job_id),
        Action::ByregotsBlessing => action_icon!("Byregot's Blessing", job_id),
        Action::PreciseTouch => action_icon!("Precise Touch", job_id),
        Action::MuscleMemory => action_icon!("Muscle Memory", job_id),
        Action::CarefulSynthesis => action_icon!("Careful Synthesis", job_id),
        Action::Manipulation => action_icon!("Manipulation", job_id),
        Action::PrudentTouch => action_icon!("Prudent Touch", job_id),
        Action::AdvancedTouch => action_icon!("Advanced Touch", job_id),
        Action::Reflect => action_icon!("Reflect", job_id),
        Action::PreparatoryTouch => action_icon!("Preparatory Touch", job_id),
        Action::Groundwork => action_icon!("Groundwork", job_id),
        Action::DelicateSynthesis => action_icon!("Delicate Synthesis", job_id),
        Action::IntensiveSynthesis => action_icon!("Intensive Synthesis", job_id),
        Action::TrainedEye => action_icon!("Trained Eye", job_id),
        Action::HeartAndSoul => action_icon!("Heart and Soul", job_id),
        Action::PrudentSynthesis => action_icon!("Prudent Synthesis", job_id),
        Action::TrainedFinesse => action_icon!("Trained Finesse", job_id),
        Action::RefinedTouch => action_icon!("Refined Touch", job_id),
        Action::QuickInnovation => action_icon!("Quick Innovation", job_id),
        Action::ImmaculateMend => action_icon!("Immaculate Mend", job_id),
        Action::TrainedPerfection => action_icon!("Trained Perfection", job_id),
    })
}
