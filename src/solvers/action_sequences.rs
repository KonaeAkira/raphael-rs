use crate::game::Action;

pub type ActionSequence = &'static [Action];

pub const PROGRESS_ACTIONS: &[ActionSequence] = &[
    &[Action::MuscleMemory],
    // &[Action::BasicSynthesis], macro_solver doesn't support 0-cp actions yet
    &[Action::CarefulSynthesis],
    &[Action::Groundwork],
    &[Action::PrudentSynthesis],
    &[Action::Observe, Action::FocusedSynthesis],
    &[Action::Veneration],
];

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

pub const DURABILITY_ACTIONS: &[ActionSequence] = &[
    &[Action::MasterMend],
    &[Action::Manipulation],
    &[Action::WasteNot],
    &[Action::WasteNot2],
];
