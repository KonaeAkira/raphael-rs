use crate::game::Action;

pub type ActionSequence = &'static [Action];

// actions that increase only Progress
pub const PROGRESS_ACTIONS: &[ActionSequence] = &[
    &[Action::MuscleMemory],
    &[Action::BasicSynthesis],
    &[Action::CarefulSynthesis],
    &[Action::Groundwork],
    &[Action::PrudentSynthesis],
    &[Action::Observe, Action::FocusedSynthesis],
    &[Action::Veneration],
];

// actions that increase only Quality
pub const QUALITY_ACTIONS: &[ActionSequence] = &[
    &[Action::Reflect],
    &[Action::PrudentTouch],
    &[
        Action::BasicTouch,
        Action::StandardTouch,
        Action::AdvancedTouch,
    ],
    &[Action::PreparatoryTouch],
    &[Action::Observe, Action::FocusedTouch],
    &[Action::TrainedFinesse],
    &[Action::ByregotsBlessing],
    &[Action::Innovation],
    &[Action::GreatStrides],
];

// actions that that increase Progress and Quality
pub const MIXED_ACTIONS: &[ActionSequence] = &[&[Action::DelicateSynthesis]];

pub const DURABILITY_ACTIONS: &[ActionSequence] = &[
    &[Action::MasterMend],
    &[Action::Manipulation],
    &[Action::WasteNot],
    &[Action::WasteNot2],
];
