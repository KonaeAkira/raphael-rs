use std::collections::HashMap;

use simulator::Action;

pub fn get_action_icons() -> HashMap<Action, image::RgbImage> {
    let mut images = std::collections::HashMap::new();
    let mut load_icon = |action, image_bytes| {
        images.insert(
            action,
            image::load_from_memory(image_bytes).unwrap().to_rgb8(),
        );
    };
    load_icon(
        Action::CarefulSynthesis,
        include_bytes!("../assets/actions/Careful Synthesis.png"),
    );
    load_icon(
        Action::BasicSynthesis,
        include_bytes!("../assets/actions/Basic Synthesis.png"),
    );
    load_icon(
        Action::BasicTouch,
        include_bytes!("../assets/actions/Basic Touch.png"),
    );
    load_icon(
        Action::MasterMend,
        include_bytes!("../assets/actions/Master's Mend.png"),
    );
    load_icon(
        Action::Observe,
        include_bytes!("../assets/actions/Observe.png"),
    );
    load_icon(
        Action::TricksOfTheTrade,
        include_bytes!("../assets/actions/Tricks of the Trade.png"),
    );
    load_icon(
        Action::WasteNot,
        include_bytes!("../assets/actions/Waste Not.png"),
    );
    load_icon(
        Action::Veneration,
        include_bytes!("../assets/actions/Veneration.png"),
    );
    load_icon(
        Action::StandardTouch,
        include_bytes!("../assets/actions/Standard Touch.png"),
    );
    load_icon(
        Action::ComboStandardTouch,
        include_bytes!("../assets/actions/Standard Touch.png"),
    );
    load_icon(
        Action::GreatStrides,
        include_bytes!("../assets/actions/Great Strides.png"),
    );
    load_icon(
        Action::Innovation,
        include_bytes!("../assets/actions/Innovation.png"),
    );
    load_icon(
        Action::WasteNot2,
        include_bytes!("../assets/actions/Waste Not II.png"),
    );
    load_icon(
        Action::ByregotsBlessing,
        include_bytes!("../assets/actions/Byregot's Blessing.png"),
    );
    load_icon(
        Action::PreciseTouch,
        include_bytes!("../assets/actions/Precise Touch.png"),
    );
    load_icon(
        Action::MuscleMemory,
        include_bytes!("../assets/actions/Muscle Memory.png"),
    );
    load_icon(
        Action::CarefulSynthesis,
        include_bytes!("../assets/actions/Careful Synthesis.png"),
    );
    load_icon(
        Action::Manipulation,
        include_bytes!("../assets/actions/Manipulation.png"),
    );
    load_icon(
        Action::PrudentTouch,
        include_bytes!("../assets/actions/Prudent Touch.png"),
    );
    load_icon(
        Action::Reflect,
        include_bytes!("../assets/actions/Reflect.png"),
    );
    load_icon(
        Action::PreparatoryTouch,
        include_bytes!("../assets/actions/Preparatory Touch.png"),
    );
    load_icon(
        Action::Groundwork,
        include_bytes!("../assets/actions/Groundwork.png"),
    );
    load_icon(
        Action::DelicateSynthesis,
        include_bytes!("../assets/actions/Delicate Synthesis.png"),
    );
    load_icon(
        Action::IntensiveSynthesis,
        include_bytes!("../assets/actions/Intensive Synthesis.png"),
    );
    load_icon(
        Action::AdvancedTouch,
        include_bytes!("../assets/actions/Advanced Touch.png"),
    );
    load_icon(
        Action::ComboAdvancedTouch,
        include_bytes!("../assets/actions/Advanced Touch.png"),
    );
    load_icon(
        Action::PrudentSynthesis,
        include_bytes!("../assets/actions/Prudent Synthesis.png"),
    );
    load_icon(
        Action::TrainedFinesse,
        include_bytes!("../assets/actions/Trained Finesse.png"),
    );
    load_icon(
        Action::ComboRefinedTouch,
        include_bytes!("../assets/actions/Refined Touch.png"),
    );
    load_icon(
        Action::ImmaculateMend,
        include_bytes!("../assets/actions/Immaculate Mend.png"),
    );
    load_icon(
        Action::TrainedPerfection,
        include_bytes!("../assets/actions/Trained Perfection.png"),
    );
    images
}
