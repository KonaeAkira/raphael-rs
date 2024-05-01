use crate::game::{state::InProgress, Action, ComboAction, Condition, Settings, State};

use strum_macros::EnumIter;

#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumIter)]
pub enum ActionSequence {
    MuscleMemory,
    Reflect,
    CarefulSynthesis,
    Groundwork,
    PreparatoryTouch,
    PrudentTouch,
    TrainedFinesse,
    AdvancedTouchCombo,
    FocusedSynthesisCombo,
    FocusedTouchCombo,
    Manipulation,
    WasteNot,
    WasteNot2,
    Innovation,
    Veneration,
    GreatStrides,
    ByregotsBlessing,
}

impl ActionSequence {
    pub const fn actions(self) -> &'static [Action] {
        match self {
            ActionSequence::CarefulSynthesis => &[Action::CarefulSynthesis],
            ActionSequence::Groundwork => &[Action::Groundwork],
            ActionSequence::PreparatoryTouch => &[Action::PreparatoryTouch],
            ActionSequence::PrudentTouch => &[Action::PrudentTouch],
            ActionSequence::TrainedFinesse => &[Action::TrainedFinesse],
            ActionSequence::AdvancedTouchCombo => &[
                Action::BasicTouch,
                Action::StandardTouch,
                Action::AdvancedTouch,
            ],
            ActionSequence::FocusedSynthesisCombo => &[Action::Observe, Action::FocusedSynthesis],
            ActionSequence::FocusedTouchCombo => &[Action::Observe, Action::FocusedTouch],
            ActionSequence::Manipulation => &[Action::Manipulation],
            ActionSequence::WasteNot => &[Action::WasteNot],
            ActionSequence::WasteNot2 => &[Action::WasteNot2],
            ActionSequence::Innovation => &[Action::Innovation],
            ActionSequence::Veneration => &[Action::Veneration],
            ActionSequence::GreatStrides => &[Action::GreatStrides],
            ActionSequence::ByregotsBlessing => &[Action::ByregotsBlessing],
            ActionSequence::MuscleMemory => &[Action::MuscleMemory],
            ActionSequence::Reflect => &[Action::Reflect],
        }
    }

    pub fn apply(self, mut state: State, settings: &Settings) -> State {
        for action in self.actions() {
            match state {
                State::InProgress(in_progress) => {
                    state = in_progress.use_action(*action, Condition::Normal, settings);
                }
                _ => return State::Invalid,
            }
        }
        state
    }

    pub fn should_use(self, state: &InProgress, settings: &Settings) -> bool {
        if state.combo == Some(ComboAction::SynthesisBegin) {
            return matches!(self, ActionSequence::MuscleMemory | ActionSequence::Reflect);
        }
        if state.effects.inner_quiet == 0 && state.missing_quality != settings.max_quality {
            return false; // don't do anything after Byregot's Blessing
        }
        let use_progress_increase: bool =
            state.effects.muscle_memory != 0 || state.effects.veneration != 0;
        let use_quality_increase: bool =
            state.effects.muscle_memory == 0 && state.effects.veneration <= 1;
        match self {
            ActionSequence::MuscleMemory => false,
            ActionSequence::Reflect => false,
            ActionSequence::CarefulSynthesis => {
                use_progress_increase && state.effects.muscle_memory == 0
            }
            ActionSequence::Groundwork => use_progress_increase,
            ActionSequence::PreparatoryTouch => {
                use_quality_increase && state.effects.waste_not != 0
            }
            ActionSequence::PrudentTouch => state.effects.great_strides == 0,
            ActionSequence::TrainedFinesse => {
                state.effects.inner_quiet == 10 && state.effects.great_strides == 0
            }
            ActionSequence::AdvancedTouchCombo => {
                use_quality_increase
                    && (state.effects.innovation >= 3 || state.effects.innovation == 0)
                    && state.effects.great_strides == 0
            }
            ActionSequence::FocusedSynthesisCombo => {
                use_progress_increase
                    && state.effects.muscle_memory == 0
                    && (state.effects.veneration >= 2 || state.effects.veneration == 0)
            }
            ActionSequence::FocusedTouchCombo => {
                use_quality_increase
                    && (state.effects.innovation >= 2 || state.effects.innovation == 0)
            }
            ActionSequence::Manipulation => {
                state.effects.manipulation == 0 && state.effects.waste_not == 0
            }
            ActionSequence::WasteNot => {
                state.effects.waste_not == 0 && state.effects.inner_quiet <= 1
            }
            ActionSequence::WasteNot2 => {
                state.effects.waste_not == 0 && state.effects.inner_quiet <= 1
            }
            ActionSequence::Innovation => use_quality_increase && state.effects.innovation == 0,
            ActionSequence::Veneration => {
                state.effects.muscle_memory != 0 && state.effects.veneration == 0
            }
            ActionSequence::GreatStrides => {
                state.effects.inner_quiet >= 6 && state.effects.great_strides == 0
            }
            ActionSequence::ByregotsBlessing => state.effects.inner_quiet >= 4,
        }
    }
}
