use simulator::{action_mask, Action, ActionMask};

pub const PROGRESS_ACTIONS: ActionMask = action_mask!(
    Action::BasicSynthesis,
    Action::Observe,
    Action::Veneration,
    Action::MuscleMemory,
    Action::CarefulSynthesis,
    Action::Groundwork,
    // Action::IntensiveSynthesis,
    Action::PrudentSynthesis
);

pub const QUALITY_ACTIONS: ActionMask = action_mask!(
    Action::BasicTouch,
    Action::Observe,
    Action::StandardTouch,
    Action::ComboStandardTouch,
    Action::GreatStrides,
    Action::Innovation,
    Action::ByregotsBlessing,
    // Action::PreciseTouch,
    Action::PrudentTouch,
    Action::Reflect,
    Action::PreparatoryTouch,
    Action::AdvancedTouch,
    Action::ComboAdvancedTouch,
    Action::TrainedFinesse
);

pub const MIXED_ACTIONS: ActionMask = action_mask!(Action::DelicateSynthesis);

pub const DURABILITY_ACTIONS: ActionMask = action_mask!(
    Action::MasterMend,
    Action::WasteNot,
    Action::WasteNot2,
    Action::Manipulation
);
