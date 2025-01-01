use simulator::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolverAction {
    TricksOfTheTradeCombo,   // Heart and Soul + Tricks of the Trade
    IntensiveSynthesisCombo, // Heart and Soul + Intensive Synthesis
    PreciseTouchCombo,       // Heart and Soul + Precise Touch
    Single(Action),
}

impl SolverAction {
    #[inline(always)]
    pub fn actions(self) -> &'static [Action] {
        match self {
            SolverAction::TricksOfTheTradeCombo => {
                &[Action::HeartAndSoul, Action::TricksOfTheTrade]
            }
            SolverAction::IntensiveSynthesisCombo => {
                &[Action::HeartAndSoul, Action::IntensiveSynthesis]
            }
            SolverAction::PreciseTouchCombo => &[Action::HeartAndSoul, Action::PreciseTouch],
            SolverAction::Single(action) => match action {
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

    #[inline(always)]
    pub fn steps(self) -> u8 {
        self.actions().len() as u8
    }

    #[inline(always)]
    pub fn duration(self) -> u8 {
        self.actions().iter().map(|action| action.time_cost()).sum()
    }
}

pub const FULL_SEARCH_ACTIONS: &[SolverAction] = &[
    SolverAction::TricksOfTheTradeCombo,
    SolverAction::IntensiveSynthesisCombo,
    SolverAction::PreciseTouchCombo,
    // progress
    SolverAction::Single(Action::BasicSynthesis),
    SolverAction::Single(Action::Veneration),
    SolverAction::Single(Action::MuscleMemory),
    SolverAction::Single(Action::CarefulSynthesis),
    SolverAction::Single(Action::Groundwork),
    SolverAction::Single(Action::IntensiveSynthesis),
    SolverAction::Single(Action::PrudentSynthesis),
    // quality
    SolverAction::Single(Action::BasicTouch),
    SolverAction::Single(Action::StandardTouch),
    SolverAction::Single(Action::GreatStrides),
    SolverAction::Single(Action::Innovation),
    SolverAction::Single(Action::ByregotsBlessing),
    SolverAction::Single(Action::PreciseTouch),
    SolverAction::Single(Action::PrudentTouch),
    SolverAction::Single(Action::Reflect),
    SolverAction::Single(Action::PreparatoryTouch),
    SolverAction::Single(Action::AdvancedTouch),
    SolverAction::Single(Action::TrainedFinesse),
    SolverAction::Single(Action::RefinedTouch),
    SolverAction::Single(Action::TrainedEye),
    SolverAction::Single(Action::QuickInnovation),
    // durability
    SolverAction::Single(Action::MasterMend),
    SolverAction::Single(Action::WasteNot),
    SolverAction::Single(Action::WasteNot2),
    SolverAction::Single(Action::Manipulation),
    SolverAction::Single(Action::ImmaculateMend),
    SolverAction::Single(Action::TrainedPerfection),
    // misc
    SolverAction::Single(Action::DelicateSynthesis),
    SolverAction::Single(Action::Observe),
    SolverAction::Single(Action::TricksOfTheTrade),
];

pub const PROGRESS_ONLY_SEARCH_ACTIONS: &[SolverAction] = &[
    SolverAction::IntensiveSynthesisCombo,
    SolverAction::TricksOfTheTradeCombo,
    // progress
    SolverAction::Single(Action::BasicSynthesis),
    SolverAction::Single(Action::Veneration),
    SolverAction::Single(Action::MuscleMemory),
    SolverAction::Single(Action::CarefulSynthesis),
    SolverAction::Single(Action::Groundwork),
    SolverAction::Single(Action::IntensiveSynthesis),
    SolverAction::Single(Action::PrudentSynthesis),
    // durability
    SolverAction::Single(Action::MasterMend),
    SolverAction::Single(Action::WasteNot),
    SolverAction::Single(Action::WasteNot2),
    SolverAction::Single(Action::Manipulation),
    SolverAction::Single(Action::ImmaculateMend),
    SolverAction::Single(Action::TrainedPerfection),
    // misc
    SolverAction::Single(Action::TricksOfTheTrade),
];

pub const QUALITY_ONLY_SEARCH_ACTIONS: &[SolverAction] = &[
    SolverAction::TricksOfTheTradeCombo,
    SolverAction::PreciseTouchCombo,
    // quality
    SolverAction::Single(Action::BasicTouch),
    SolverAction::Single(Action::StandardTouch),
    SolverAction::Single(Action::GreatStrides),
    SolverAction::Single(Action::Innovation),
    SolverAction::Single(Action::ByregotsBlessing),
    SolverAction::Single(Action::PreciseTouch),
    SolverAction::Single(Action::PrudentTouch),
    SolverAction::Single(Action::Reflect),
    SolverAction::Single(Action::PreparatoryTouch),
    SolverAction::Single(Action::AdvancedTouch),
    SolverAction::Single(Action::TrainedFinesse),
    SolverAction::Single(Action::RefinedTouch),
    SolverAction::Single(Action::TrainedEye),
    SolverAction::Single(Action::QuickInnovation),
    // durability
    SolverAction::Single(Action::MasterMend),
    SolverAction::Single(Action::WasteNot),
    SolverAction::Single(Action::WasteNot2),
    SolverAction::Single(Action::Manipulation),
    SolverAction::Single(Action::ImmaculateMend),
    SolverAction::Single(Action::TrainedPerfection),
    // misc
    SolverAction::Single(Action::Observe),
    SolverAction::Single(Action::TricksOfTheTrade),
];

pub fn use_solver_action(
    settings: &Settings,
    mut state: SimulationState,
    action: SolverAction,
) -> Result<SimulationState, &'static str> {
    for action in action.actions().iter() {
        state = state.use_action(*action, Condition::Normal, settings)?;
    }
    Ok(state)
}
