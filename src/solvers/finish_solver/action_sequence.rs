use crate::game::{units::*, Action, Condition, Settings, State};

use strum_macros::EnumIter;

#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumIter)]
pub enum ActionSequence {
    BasicSynthesis,
    MasterMend,
    CarefulSynthesis,
    Groundwork,
    FocusedSynthesisCombo,
    Manipulation,
    WasteNot,
    WasteNot2,
    Veneration,
}

impl ActionSequence {
    pub fn actions(&self) -> &[Action] {
        match *self {
            ActionSequence::CarefulSynthesis => &[Action::CarefulSynthesis],
            ActionSequence::Groundwork => &[Action::Groundwork],
            ActionSequence::FocusedSynthesisCombo => &[Action::Observe, Action::FocusedSynthesis],
            ActionSequence::MasterMend => &[Action::MasterMend],
            ActionSequence::Manipulation => &[Action::Manipulation],
            ActionSequence::WasteNot => &[Action::WasteNot],
            ActionSequence::WasteNot2 => &[Action::WasteNot2],
            ActionSequence::Veneration => &[Action::Veneration],
            ActionSequence::BasicSynthesis => &[Action::BasicSynthesis],
        }
    }

    pub fn base_cp_cost(&self) -> CP {
        let mut result = 0;
        for action in self.actions() {
            result += action.base_cp_cost();
        }
        result
    }

    pub fn apply(&self, mut state: State, settings: &Settings) -> State {
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
}
