use crate::{
    config::Settings,
    game::{actions::Action, conditions::Condition, state::State},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ActionSequence {
    // opener
    MuscleMemoryOpener,
    ReflectOpener,
    // singles
    BasicSynthesis,
    MasterMend,
    CarefulSynthesis,
    Groundwork,
    PreparatoryTouch,
    PrudentTouch,
    TrainedFinesse,
    // combos
    AdvancedTouchCombo,
    FocusedSynthesisCombo,
    FocusedTouchCombo,
    // effects
    Manipulation,
    WasteNot,
    WasteNot2,
    Innovation,
    Veneration,
    // finisher
    ByresgotsBlessingCombo,
    ByregotsBlessing,
}

impl ActionSequence {
    pub fn actions(&self) -> &[Action] {
        match *self {
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
            ActionSequence::MasterMend => &[Action::MasterMend],
            ActionSequence::Manipulation => &[Action::Manipulation],
            ActionSequence::WasteNot => &[Action::WasteNot],
            ActionSequence::WasteNot2 => &[Action::WasteNot2],
            ActionSequence::Innovation => &[Action::Innovation],
            ActionSequence::Veneration => &[Action::Veneration],
            ActionSequence::ByresgotsBlessingCombo => {
                &[Action::GreatStrides, Action::ByregotsBlessing]
            }
            ActionSequence::ByregotsBlessing => &[Action::ByregotsBlessing],
            ActionSequence::MuscleMemoryOpener => &[Action::MuscleMemory],
            ActionSequence::ReflectOpener => &[Action::Reflect],
            ActionSequence::BasicSynthesis => &[Action::BasicSynthesis],
        }
    }

    pub fn base_cp_cost(&self) -> i32 {
        let mut result: i32 = 0;
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
