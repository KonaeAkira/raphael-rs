use raphael_sim::*;

use crate::SolverSettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionCombo {
    TricksOfTheTrade,   // Heart and Soul + Tricks of the Trade
    IntensiveSynthesis, // Heart and Soul + Intensive Synthesis
    PreciseTouch,       // Heart and Soul + Precise Touch
    StandardTouch,      // Basic Touch + Standard Touch
    AdvancedTouch,      // Basic Touch + Standard Touch + Advanced Touch
    FocusedTouch,       // Observe + AdvancedTouch
    RefinedTouch,       // Basic Touch + Refined Touch
    Single(Action),
}

impl ActionCombo {
    pub const fn actions(self) -> &'static [Action] {
        match self {
            Self::TricksOfTheTrade => &[Action::HeartAndSoul, Action::TricksOfTheTrade],
            Self::IntensiveSynthesis => &[Action::HeartAndSoul, Action::IntensiveSynthesis],
            Self::PreciseTouch => &[Action::HeartAndSoul, Action::PreciseTouch],
            Self::StandardTouch => &[Action::BasicTouch, Action::StandardTouch],
            Self::AdvancedTouch => &[
                Action::BasicTouch,
                Action::StandardTouch,
                Action::AdvancedTouch,
            ],
            Self::FocusedTouch => &[Action::Observe, Action::AdvancedTouch],
            Self::RefinedTouch => &[Action::BasicTouch, Action::RefinedTouch],
            Self::Single(action) => match action {
                Action::BasicSynthesis => &[Action::BasicSynthesis],
                Action::BasicTouch => &[Action::BasicTouch],
                Action::MasterMend => &[Action::MasterMend],
                Action::Observe => &[Action::Observe],
                Action::TricksOfTheTrade => &[Action::TricksOfTheTrade],
                Action::WasteNot => &[Action::WasteNot],
                Action::Veneration => &[Action::Veneration],
                Action::StandardTouch => &[Action::StandardTouch],
                Action::GreatStrides => &[Action::GreatStrides],
                Action::Innovation => &[Action::Innovation],
                Action::WasteNot2 => &[Action::WasteNot2],
                Action::ByregotsBlessing => &[Action::ByregotsBlessing],
                Action::PreciseTouch => &[Action::PreciseTouch],
                Action::MuscleMemory => &[Action::MuscleMemory],
                Action::CarefulSynthesis => &[Action::CarefulSynthesis],
                Action::Manipulation => &[Action::Manipulation],
                Action::PrudentTouch => &[Action::PrudentTouch],
                Action::AdvancedTouch => &[Action::AdvancedTouch],
                Action::Reflect => &[Action::Reflect],
                Action::PreparatoryTouch => &[Action::PreparatoryTouch],
                Action::Groundwork => &[Action::Groundwork],
                Action::DelicateSynthesis => &[Action::DelicateSynthesis],
                Action::IntensiveSynthesis => &[Action::IntensiveSynthesis],
                Action::TrainedEye => &[Action::TrainedEye],
                Action::HeartAndSoul => &[Action::HeartAndSoul],
                Action::PrudentSynthesis => &[Action::PrudentSynthesis],
                Action::TrainedFinesse => &[Action::TrainedFinesse],
                Action::RefinedTouch => &[Action::RefinedTouch],
                Action::QuickInnovation => &[Action::QuickInnovation],
                Action::ImmaculateMend => &[Action::ImmaculateMend],
                Action::TrainedPerfection => &[Action::TrainedPerfection],
            },
        }
    }

    pub const fn steps(self) -> u8 {
        self.actions().len() as u8
    }

    pub fn duration(self) -> u8 {
        self.actions().iter().map(|action| action.time_cost()).sum()
    }
}

pub const FULL_SEARCH_ACTIONS: &[ActionCombo] = &[
    ActionCombo::AdvancedTouch,
    ActionCombo::TricksOfTheTrade,
    ActionCombo::IntensiveSynthesis,
    ActionCombo::PreciseTouch,
    ActionCombo::StandardTouch,
    ActionCombo::FocusedTouch,
    ActionCombo::RefinedTouch,
    // progress
    ActionCombo::Single(Action::BasicSynthesis),
    ActionCombo::Single(Action::Veneration),
    ActionCombo::Single(Action::MuscleMemory),
    ActionCombo::Single(Action::CarefulSynthesis),
    ActionCombo::Single(Action::Groundwork),
    ActionCombo::Single(Action::PrudentSynthesis),
    // quality
    ActionCombo::Single(Action::BasicTouch),
    ActionCombo::Single(Action::StandardTouch),
    ActionCombo::Single(Action::GreatStrides),
    ActionCombo::Single(Action::Innovation),
    ActionCombo::Single(Action::ByregotsBlessing),
    ActionCombo::Single(Action::PrudentTouch),
    ActionCombo::Single(Action::Reflect),
    ActionCombo::Single(Action::PreparatoryTouch),
    ActionCombo::Single(Action::AdvancedTouch),
    ActionCombo::Single(Action::TrainedFinesse),
    ActionCombo::Single(Action::TrainedEye),
    ActionCombo::Single(Action::QuickInnovation),
    // durability
    ActionCombo::Single(Action::MasterMend),
    ActionCombo::Single(Action::WasteNot),
    ActionCombo::Single(Action::WasteNot2),
    ActionCombo::Single(Action::Manipulation),
    ActionCombo::Single(Action::ImmaculateMend),
    ActionCombo::Single(Action::TrainedPerfection),
    // misc
    ActionCombo::Single(Action::DelicateSynthesis),
];

pub const PROGRESS_ONLY_SEARCH_ACTIONS: &[ActionCombo] = &[
    ActionCombo::IntensiveSynthesis,
    ActionCombo::TricksOfTheTrade,
    // progress
    ActionCombo::Single(Action::BasicSynthesis),
    ActionCombo::Single(Action::Veneration),
    ActionCombo::Single(Action::MuscleMemory),
    ActionCombo::Single(Action::CarefulSynthesis),
    ActionCombo::Single(Action::Groundwork),
    ActionCombo::Single(Action::PrudentSynthesis),
    // durability
    ActionCombo::Single(Action::MasterMend),
    ActionCombo::Single(Action::WasteNot),
    ActionCombo::Single(Action::WasteNot2),
    ActionCombo::Single(Action::Manipulation),
    ActionCombo::Single(Action::ImmaculateMend),
    ActionCombo::Single(Action::TrainedPerfection),
];

pub fn use_action_combo(
    settings: &SolverSettings,
    mut state: SimulationState,
    action_combo: ActionCombo,
) -> Result<SimulationState, &'static str> {
    for action in action_combo.actions() {
        state = state.use_action(*action, Condition::Normal, &settings.simulator_settings)?;
        if state.effects.allow_quality_actions() && state.quality >= settings.max_quality() {
            state.unreliable_quality = 0;
            state.effects = state.effects.strip_quality_effects();
        }
    }
    state.effects.set_combo(Combo::None);
    Ok(state)
}
