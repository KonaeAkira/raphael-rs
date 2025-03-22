use raphael_sim::*;

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
