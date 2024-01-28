use crate::{
    config::Settings,
    game::{actions::Action, conditions::Condition, effects::Effects},
};

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
    pub quality: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InProgress {
    pub last_action: Option<Action>,
    pub cp: i32,
    pub durability: i32,
    pub progress: i32,
    pub quality: i32,
    pub effects: Effects,
}

impl InProgress {
    pub fn new(settings: &Settings) -> InProgress {
        InProgress {
            last_action: None,
            cp: settings.max_cp,
            durability: settings.max_durability,
            progress: 0,
            quality: 0,
            effects: Default::default(),
        }
    }

    pub fn use_action(&self, action: Action, condition: Condition, settings: &Settings) -> State {
        let cp_cost = action.cp_cost(&self.effects, condition);
        let durability_cost = action.durability_cost(&self.effects, condition);
        let progress_increase = action.progress_increase(&self.effects, condition);
        let quality_increase = action.quality_increase(&self.effects, condition);

        if cp_cost > self.cp {
            return State::Invalid;
        }

        if !match action {
            Action::MuscleMemory | Action::Reflect => self.last_action.is_none(),
            Action::ByregotsBlessing => self.effects.inner_quiet != 0,
            Action::PrudentSynthesis | Action::PrudentTouch => self.effects.waste_not == 0,
            Action::IntensiveSynthesis | Action::PreciseTouch | Action::TricksOfTheTrade => {
                condition == Condition::Good || condition == Condition::Excellent
            }
            Action::Groundwork => self.durability >= durability_cost,
            Action::TrainedFinesse => self.effects.inner_quiet == 10,
            _ => true,
        } {
            return State::Invalid;
        }

        // last action must match required combo action
        match action.combo_action() {
            Some(req_action) => {
                if self.last_action != Some(req_action) {
                    return State::Invalid;
                }
            }
            None => (),
        }

        let mut new_state = self.clone();
        new_state.last_action = Some(action);
        new_state.cp -= cp_cost;
        new_state.durability -= durability_cost;

        // reset muscle memory if progress increased
        if progress_increase > 0 {
            new_state.progress += progress_increase;
            new_state.effects.muscle_memory = 0;
        }

        // reset great strides and increase inner quiet if quality increased
        if quality_increase > 0 {
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

        return State::InProgress(new_state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        game::actions::{PROG_DENOM, QUAL_DENOM},
        progress, quality,
    };

    const SETTINGS: Settings = Settings {
        max_cp: 200,
        max_durability: 60,
        max_progress: progress!(2000),
        max_quality: quality!(40000),
    };

    #[test]
    fn test_initial_state() {
        let state = State::new(&SETTINGS);
        match state {
            State::InProgress(state) => {
                assert_eq!(state.last_action, None);
                assert_eq!(state.cp, SETTINGS.max_cp);
                assert_eq!(state.durability, SETTINGS.max_durability);
                assert_eq!(state.progress, 0);
                assert_eq!(state.quality, 0);
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
