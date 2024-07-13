use simulator::{
    state::{InProgress, PrevActionDelta}, Action, ActionMask, ComboAction, Condition, Effects, Settings,
    SimulationState, SingleUse,
};

use rustc_hash::FxHashMap as HashMap;

use super::actions::{DURABILITY_ACTIONS, PROGRESS_ACTIONS};

const SEARCH_ACTIONS: ActionMask = PROGRESS_ACTIONS
    .union(DURABILITY_ACTIONS)
    .remove(Action::DelicateSynthesis);

#[bitfield_struct::bitfield(u16)]
#[derive(PartialEq, Eq, Hash)]
struct ReducedEffects {
    #[bits(4)]
    pub muscle_memory: u8,
    #[bits(4)]
    pub waste_not: u8,
    #[bits(4)]
    pub veneration: u8,
    #[bits(4)]
    pub manipulation: u8,
}

impl ReducedEffects {
    pub fn from_effects(effects: &Effects) -> ReducedEffects {
        Self::new()
            .with_muscle_memory(effects.muscle_memory())
            .with_waste_not(effects.waste_not())
            .with_veneration(effects.veneration())
            .with_manipulation(effects.manipulation())
    }

    pub fn to_effects(self) -> Effects {
        Effects::new()
            .with_waste_not(self.waste_not())
            .with_veneration(self.veneration())
            .with_muscle_memory(self.muscle_memory())
            .with_manipulation(self.manipulation())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ReducedState {
    durability: i8,
    cp: i16,
    effects: ReducedEffects,
    combo: Option<ComboAction>,
    trained_perfection: SingleUse,
}

impl ReducedState {
    pub fn from_state(state: &InProgress) -> ReducedState {
        ReducedState {
            durability: state.raw_state().durability,
            cp: state.raw_state().cp,
            effects: ReducedEffects::from_effects(&state.raw_state().effects),
            combo: state.raw_state().combo,
            trained_perfection: state.raw_state().effects.trained_perfection(),
        }
    }

    pub fn to_state(self) -> InProgress {
        let raw_state = SimulationState {
            durability: self.durability,
            cp: self.cp,
            missing_progress: u16::MAX,
            missing_quality: [0; 3],
            prev_deltas: [PrevActionDelta::default(); 2],
            effects: self
                .effects
                .to_effects()
                .with_trained_perfection(self.trained_perfection),
            combo: self.combo,
        };
        raw_state.try_into().unwrap()
    }
}

pub struct FinishSolver {
    settings: Settings,
    // maximum attainable progress for each state
    max_progress: HashMap<ReducedState, u16>,
}

impl FinishSolver {
    pub fn new(settings: Settings) -> FinishSolver {
        dbg!(std::mem::size_of::<ReducedState>());
        dbg!(std::mem::align_of::<ReducedState>());
        FinishSolver {
            settings,
            max_progress: HashMap::default(),
        }
    }

    pub fn can_finish(&mut self, state: &InProgress) -> bool {
        let max_progress = self.solve_max_progress(ReducedState::from_state(state));
        max_progress >= state.raw_state().missing_progress
    }

    fn solve_max_progress(&mut self, state: ReducedState) -> u16 {
        match self.max_progress.get(&state) {
            Some(max_progress) => *max_progress,
            None => {
                let mut max_progress = 0;
                for action in SEARCH_ACTIONS
                    .intersection(self.settings.allowed_actions)
                    .actions_iter()
                {
                    if let Ok(new_state) =
                        state
                            .to_state()
                            .use_action(action, Condition::Normal, &self.settings)
                    {
                        if let Ok(in_progress) = new_state.try_into() {
                            let child_progress =
                                self.solve_max_progress(ReducedState::from_state(&in_progress));
                            let action_progress =
                                u16::MAX - in_progress.raw_state().missing_progress;
                            max_progress =
                                std::cmp::max(max_progress, child_progress + action_progress);
                        } else {
                            let progress = u16::MAX - new_state.missing_progress;
                            max_progress = std::cmp::max(max_progress, progress);
                        }
                    }
                    if max_progress >= self.settings.max_progress {
                        // stop early if progress is already maxed out
                        // this optimization would work better with a better action ordering
                        max_progress = self.settings.max_progress;
                        break;
                    }
                }
                self.max_progress.insert(state, max_progress);
                max_progress
            }
        }
    }
}

impl Drop for FinishSolver {
    fn drop(&mut self) {
        dbg!(self.max_progress.len());
    }
}
