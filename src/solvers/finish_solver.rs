use crate::game::{
    state::InProgress, units::*, Action, ActionMask, ComboAction, Condition, Effects, Settings,
    State,
};

use rustc_hash::FxHashMap as HashMap;

use super::actions::{DURABILITY_ACTIONS, PROGRESS_ACTIONS};

const INF_PROGRESS: Progress = Progress::new(100000);
const SEARCH_ACTIONS: ActionMask = PROGRESS_ACTIONS.union(DURABILITY_ACTIONS);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ReducedEffects {
    pub muscle_memory: u8,
    pub waste_not: u8,
    pub veneration: u8,
    pub manipulation: u8,
}

impl ReducedEffects {
    pub fn from_effects(effects: &Effects) -> ReducedEffects {
        ReducedEffects {
            muscle_memory: effects.muscle_memory,
            waste_not: effects.waste_not,
            veneration: effects.veneration,
            manipulation: effects.manipulation,
        }
    }

    pub fn to_effects(self) -> Effects {
        Effects {
            inner_quiet: 0,
            waste_not: self.waste_not,
            innovation: 0,
            veneration: self.veneration,
            great_strides: 0,
            muscle_memory: self.muscle_memory,
            manipulation: self.manipulation,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ReducedState {
    durability: Durability,
    cp: CP,
    effects: ReducedEffects,
    combo: Option<ComboAction>,
}

impl ReducedState {
    pub fn from_state(state: &InProgress) -> ReducedState {
        ReducedState {
            durability: state.durability,
            cp: state.cp,
            effects: ReducedEffects::from_effects(&state.effects),
            combo: state.combo,
        }
    }

    pub fn to_state(self) -> InProgress {
        InProgress {
            durability: self.durability,
            cp: self.cp,
            missing_progress: INF_PROGRESS,
            missing_quality: Quality::new(0),
            effects: self.effects.to_effects(),
            combo: self.combo,
        }
    }
}

#[derive(Debug)]
pub struct FinishSolver {
    settings: Settings,
    memoization: HashMap<ReducedState, Vec<Progress>>,
}

impl FinishSolver {
    pub fn new(settings: Settings) -> FinishSolver {
        FinishSolver {
            settings,
            memoization: HashMap::default(),
        }
    }

    pub fn get_finish_sequence(&mut self, mut state: InProgress) -> Option<Vec<Action>> {
        if !self.can_finish(&state) {
            return None;
        }
        let mut finish_sequence: Vec<Action> = Vec::new();
        loop {
            let mut best_time = 100000;
            let mut best_action = Action::BasicSynthesis;
            for action in SEARCH_ACTIONS
                .intersection(self.settings.allowed_actions)
                .actions_iter()
            {
                let new_state = state.use_action(action, Condition::Normal, &self.settings);
                match new_state {
                    State::InProgress(new_state) => {
                        let time = self.time_to_finish(&new_state);
                        if time.is_some() && time.unwrap() + action.time_cost() < best_time {
                            best_time = time.unwrap() + action.time_cost();
                            best_action = action;
                        }
                    }
                    State::Completed { .. } => {
                        if action.time_cost() < best_time {
                            best_time = action.time_cost();
                            best_action = action;
                        }
                    }
                    _ => (),
                }
            }

            finish_sequence.push(best_action);

            let new_state = state.use_action(best_action, Condition::Normal, &self.settings);
            match new_state {
                State::InProgress(new_state) => state = new_state,
                State::Completed { .. } => return Some(finish_sequence),
                _ => (),
            }
        }
    }

    pub fn can_finish(&mut self, state: &InProgress) -> bool {
        *self.solve(ReducedState::from_state(state)).last().unwrap() >= state.missing_progress
    }

    pub fn time_to_finish(&mut self, state: &InProgress) -> Option<u32> {
        let result = self.solve(ReducedState::from_state(state));
        for (time, progress) in result.into_iter().enumerate() {
            if progress >= state.missing_progress {
                return Some(time as u32);
            }
        }
        None
    }

    fn solve(&mut self, state: ReducedState) -> Vec<Progress> {
        match self.memoization.get(&state) {
            Some(result) => result.clone(),
            None => {
                let mut result = Vec::new();
                for action in SEARCH_ACTIONS
                    .intersection(self.settings.allowed_actions)
                    .actions_iter()
                {
                    let new_state =
                        state
                            .to_state()
                            .use_action(action, Condition::Normal, &self.settings);
                    match new_state {
                        State::InProgress(new_state) => {
                            let progress = INF_PROGRESS.sub(new_state.missing_progress);
                            let sub_result = self.solve(ReducedState::from_state(&new_state));
                            Self::update_result(
                                &mut result,
                                &sub_result,
                                action.time_cost(),
                                progress,
                            );
                        }
                        State::Failed { missing_progress } => {
                            let progress = INF_PROGRESS.sub(missing_progress);
                            Self::update_result(
                                &mut result,
                                &[Progress::new(0)],
                                action.time_cost(),
                                progress,
                            );
                        }
                        _ => (),
                    }
                }
                self.canonicalize_result(&mut result);
                self.memoization.insert(state, result.clone());
                result
            }
        }
    }

    fn update_result(
        result: &mut Vec<Progress>,
        sub_result: &[Progress],
        time_offset: u32,
        progress_increase: Progress,
    ) {
        if result.len() < sub_result.len() + time_offset as usize {
            result.resize(sub_result.len() + time_offset as usize, Progress::new(0));
        }
        let result: &mut [Progress] =
            &mut result[time_offset as usize..time_offset as usize + sub_result.len()];
        for (a, b) in result.iter_mut().zip(sub_result.iter()) {
            *a = std::cmp::max(*a, (*b).add(progress_increase));
        }
    }

    fn canonicalize_result(&self, result: &mut Vec<Progress>) {
        let mut rolling_max = Progress::new(0);
        for a in result.iter_mut() {
            rolling_max = std::cmp::max(rolling_max, *a);
            *a = rolling_max;
        }
        while result.len() >= 2 && result[result.len() - 2] >= self.settings.max_progress {
            result.pop();
        }
    }
}

impl Drop for FinishSolver {
    fn drop(&mut self) {
        let finish_solver_states = self.memoization.len();
        dbg!(finish_solver_states);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solve(settings: Settings, actions: &[Action]) -> Vec<Action> {
        let state = State::new(&settings).use_actions(actions, Condition::Normal, &settings);
        let result = FinishSolver::new(settings)
            .get_finish_sequence(state.as_in_progress().unwrap())
            .unwrap();
        dbg!(&result);
        result
    }

    #[test]
    fn test_01() {
        let settings = Settings {
            max_cp: 100,
            max_durability: 30,
            max_progress: Progress::from(360.00),
            max_quality: Quality::from(20000.00),
            allowed_actions: ActionMask::from_level(90, true),
        };
        let actions = solve(settings, &[]);
        assert_eq!(actions, [Action::Groundwork]);
    }

    #[test]
    fn test_02() {
        let settings = Settings {
            max_cp: 100,
            max_durability: 60,
            max_progress: Progress::from(2100.00),
            max_quality: Quality::from(20000.00),
            allowed_actions: ActionMask::from_level(90, true),
        };
        let actions = solve(settings, &[]);
        assert_eq!(
            actions,
            [
                Action::MuscleMemory,
                Action::Veneration,
                Action::Groundwork,
                Action::Groundwork,
                Action::PrudentSynthesis,
                Action::BasicSynthesis,
            ]
        );
    }
}
