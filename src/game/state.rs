use crate::game::{units::*, Action, ComboAction, Condition, Effects, Settings};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    InProgress(InProgress),
    Completed(Completed),
    Failed,
    Invalid,
}

impl State {
    pub fn new(settings: &Settings) -> State {
        State::InProgress(InProgress::new(settings))
    }

    pub fn as_in_progress(self) -> Option<InProgress> {
        match self {
            State::InProgress(in_progress) => Some(in_progress),
            _ => None,
        }
    }

    pub fn as_completed(self) -> Option<Completed> {
        match self {
            State::Completed(completed) => Some(completed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Completed {
    pub quality: Quality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InProgress {
    pub cp: CP,
    pub durability: Durability,
    pub progress: Progress,
    pub quality: Quality,
    pub effects: Effects,
    pub combo: Option<ComboAction>,
}

impl InProgress {
    pub fn new(settings: &Settings) -> InProgress {
        InProgress {
            cp: settings.max_cp,
            durability: settings.max_durability,
            progress: Progress::from(0),
            quality: Quality::from(0),
            effects: Default::default(),
            combo: Some(ComboAction::SynthesisBegin),
        }
    }

    fn can_use_action(&self, action: Action, condition: Condition) -> bool {
        if action.cp_cost(&self.effects, condition) > self.cp {
            false
        } else if action.required_combo().is_some() && self.combo != action.required_combo() {
            false
        } else {
            match action {
                Action::ByregotsBlessing => self.effects.inner_quiet != 0,
                Action::PrudentSynthesis | Action::PrudentTouch => self.effects.waste_not == 0,
                Action::IntensiveSynthesis | Action::PreciseTouch | Action::TricksOfTheTrade => {
                    condition == Condition::Good || condition == Condition::Excellent
                }
                Action::Groundwork => {
                    self.durability >= action.durability_cost(&self.effects, condition)
                }
                Action::TrainedFinesse => self.effects.inner_quiet == 10,
                _ => true,
            }
        }
    }

    pub fn use_action(&self, action: Action, condition: Condition, settings: &Settings) -> State {
        if !self.can_use_action(action, condition) {
            return State::Invalid;
        }

        let cp_cost = action.cp_cost(&self.effects, condition);
        let durability_cost = action.durability_cost(&self.effects, condition);
        let progress_increase = action.progress_increase(&self.effects, condition);
        let quality_increase = action.quality_increase(&self.effects, condition);

        let mut new_state = self.clone();
        new_state.combo = action.to_combo();
        new_state.cp -= cp_cost;
        new_state.durability -= durability_cost;

        // reset muscle memory if progress increased
        if progress_increase > Progress::from(0) {
            new_state.progress += progress_increase;
            new_state.effects.muscle_memory = 0;
        }

        // reset great strides and increase inner quiet if quality increased
        if quality_increase > Quality::from(0) {
            new_state.quality += quality_increase;
            new_state.effects.great_strides = 0;
            new_state.effects.inner_quiet += match action {
                Action::Reflect => 2,
                Action::PreciseTouch => 2,
                Action::PreparatoryTouch => 2,
                _ => 1,
            };
            new_state.effects.inner_quiet = std::cmp::min(10, new_state.effects.inner_quiet);
        }

        new_state.quality = std::cmp::min(settings.max_quality, new_state.quality);
        if new_state.progress >= settings.max_progress {
            return State::Completed(Completed {
                quality: new_state.quality,
            });
        } else if new_state.durability <= 0 {
            return State::Failed;
        }

        // remove manipulation before it is triggered
        if action == Action::Manipulation {
            new_state.effects.manipulation = 0;
        }

        if new_state.effects.manipulation > 0 {
            new_state.durability = std::cmp::min(new_state.durability + 5, settings.max_durability);
        }
        new_state.effects.tick_down();

        // trigger special action effects
        let duration_bonus = if condition == Condition::Pliant { 2 } else { 0 };
        match action {
            Action::MuscleMemory => new_state.effects.muscle_memory = 5 + duration_bonus,
            Action::GreatStrides => new_state.effects.great_strides = 3 + duration_bonus,
            Action::Veneration => new_state.effects.veneration = 4 + duration_bonus,
            Action::Innovation => new_state.effects.innovation = 4 + duration_bonus,
            Action::WasteNot => new_state.effects.waste_not = 4 + duration_bonus,
            Action::WasteNot2 => new_state.effects.waste_not = 8 + duration_bonus,
            Action::Manipulation => new_state.effects.manipulation = 8 + duration_bonus,
            Action::MasterMend => {
                new_state.durability =
                    std::cmp::min(settings.max_durability, new_state.durability + 30)
            }
            Action::ByregotsBlessing => new_state.effects.inner_quiet = 0,
            Action::TricksOfTheTrade => {
                new_state.cp = std::cmp::min(settings.max_cp, new_state.cp + 20)
            }
            _ => (),
        }

        State::InProgress(new_state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SETTINGS: Settings = Settings {
        max_cp: 200,
        max_durability: 60,
        max_progress: Progress::from_const(2000),
        max_quality: Quality::from_const(40000),
    };

    #[test]
    fn test_initial_state() {
        let state = State::new(&SETTINGS);
        match state {
            State::InProgress(state) => {
                assert_eq!(state.combo, Some(ComboAction::SynthesisBegin));
                assert_eq!(state.cp, SETTINGS.max_cp);
                assert_eq!(state.durability, SETTINGS.max_durability);
                assert_eq!(state.progress, Progress::from(0));
                assert_eq!(state.quality, Quality::from(0));
                assert_eq!(state.effects.inner_quiet, 0);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_not_enough_cp() {
        let mut settings: Settings = SETTINGS;
        settings.max_cp = 10;
        let state = InProgress::new(&settings);
        let state = state.use_action(Action::Manipulation, Condition::Normal, &settings);
        assert!(matches!(state, State::Invalid));
    }
}
