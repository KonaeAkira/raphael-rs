use std::collections::HashMap;

use simulator::Action;

pub fn get_action_icons() -> HashMap<Action, image::RgbImage> {
    let mut images = std::collections::HashMap::new();
    let mut load_icon = |action, image_bytes| {
        images.insert(
            action,
            image::load_from_memory(image_bytes)
                .unwrap()
                .resize(30, 30, image::imageops::FilterType::Gaussian)
                .to_rgb8(),
        );
    };
    load_icon(
        Action::CarefulSynthesis,
        include_bytes!("../assets/actions/Careful Synthesis.webp"),
    );
    load_icon(
        Action::BasicSynthesis,
        include_bytes!("../assets/actions/Basic Synthesis-BSM.webp"),
    );
    load_icon(
        Action::BasicTouch,
        include_bytes!("../assets/actions/Basic Touch-BSM.webp"),
    );
    load_icon(
        Action::MasterMend,
        include_bytes!("../assets/actions/Master's Mend.webp"),
    );
    load_icon(
        Action::Observe,
        include_bytes!("../assets/actions/Observe.webp"),
    );
    load_icon(
        Action::TricksOfTheTrade,
        include_bytes!("../assets/actions/Tricks of the Trade.webp"),
    );
    load_icon(
        Action::WasteNot,
        include_bytes!("../assets/actions/Waste Not.webp"),
    );
    load_icon(
        Action::Veneration,
        include_bytes!("../assets/actions/Veneration.webp"),
    );
    load_icon(
        Action::StandardTouch,
        include_bytes!("../assets/actions/Standard Touch-BSM.webp"),
    );
    load_icon(
        Action::ComboStandardTouch,
        include_bytes!("../assets/actions/Standard Touch-BSM.webp"),
    );
    load_icon(
        Action::GreatStrides,
        include_bytes!("../assets/actions/Great Strides.webp"),
    );
    load_icon(
        Action::Innovation,
        include_bytes!("../assets/actions/Innovation.webp"),
    );
    load_icon(
        Action::WasteNot2,
        include_bytes!("../assets/actions/Waste Not II.webp"),
    );
    load_icon(
        Action::ByregotsBlessing,
        include_bytes!("../assets/actions/Byregot's Blessing.webp"),
    );
    load_icon(
        Action::PreciseTouch,
        include_bytes!("../assets/actions/Precise Touch-BSM.webp"),
    );
    load_icon(
        Action::MuscleMemory,
        include_bytes!("../assets/actions/Muscle Memory.webp"),
    );
    load_icon(
        Action::CarefulSynthesis,
        include_bytes!("../assets/actions/Careful Synthesis.webp"),
    );
    load_icon(
        Action::Manipulation,
        include_bytes!("../assets/actions/Manipulation.webp"),
    );
    load_icon(
        Action::PrudentTouch,
        include_bytes!("../assets/actions/Prudent Touch-BSM.webp"),
    );
    load_icon(
        Action::Reflect,
        include_bytes!("../assets/actions/Reflect.webp"),
    );
    load_icon(
        Action::PreparatoryTouch,
        include_bytes!("../assets/actions/Preparatory Touch-BSM.webp"),
    );
    load_icon(
        Action::Groundwork,
        include_bytes!("../assets/actions/Groundwork-BSM.webp"),
    );
    load_icon(
        Action::DelicateSynthesis,
        include_bytes!("../assets/actions/Delicate Synthesis-BSM.webp"),
    );
    load_icon(
        Action::IntensiveSynthesis,
        include_bytes!("../assets/actions/Intensive Synthesis-BSM.webp"),
    );
    load_icon(
        Action::AdvancedTouch,
        include_bytes!("../assets/actions/Advanced Touch-BSM.webp"),
    );
    load_icon(
        Action::ComboAdvancedTouch,
        include_bytes!("../assets/actions/Advanced Touch-BSM.webp"),
    );
    load_icon(
        Action::PrudentSynthesis,
        include_bytes!("../assets/actions/Prudent Synthesis-BSM.webp"),
    );
    load_icon(
        Action::TrainedFinesse,
        include_bytes!("../assets/actions/Trained Finesse.webp"),
    );
    images
}
