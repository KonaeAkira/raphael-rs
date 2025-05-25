use std::collections::VecDeque;

use raphael_sim::*;

use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    SolverSettings,
    actions::{PROGRESS_ONLY_SEARCH_ACTIONS, use_action_combo},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TemplateData {
    durability: u16,
    effects: Effects,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Template {
    data: TemplateData,
    max_cp: u16,
}

impl Template {
    pub fn instantiate(self, cp: u16) -> Option<ReducedState> {
        if cp > self.max_cp {
            return None;
        }
        Some(ReducedState {
            durability: self.data.durability,
            cp,
            effects: self.data.effects,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ReducedState {
    durability: u16,
    cp: u16,
    effects: Effects,
}

impl ReducedState {
    fn from_state(state: &SimulationState) -> Self {
        Self {
            durability: state.durability,
            cp: state.cp,
            effects: state.effects.strip_quality_effects(),
        }
    }

    fn to_state(self) -> SimulationState {
        SimulationState {
            durability: self.durability,
            cp: self.cp,
            progress: 0,
            quality: 0,
            unreliable_quality: 0,
            effects: self.effects,
        }
    }
}

pub struct FinishSolver {
    settings: SolverSettings,
    // maximum attainable progress for each state
    solved_states: FxHashMap<ReducedState, u32>,
}

impl FinishSolver {
    pub fn new(settings: SolverSettings) -> Self {
        Self {
            settings,
            solved_states: FxHashMap::default(),
        }
    }

    fn generate_precompute_templates(&self) -> Vec<Template> {
        let seed_template = Template {
            max_cp: u16::MAX,
            data: TemplateData {
                durability: self.settings.max_durability(),
                effects: Effects::initial(&self.settings.simulator_settings)
                    .strip_quality_effects(),
            },
        };

        let mut templates = FxHashSet::default();
        let mut queue = VecDeque::new();
        templates.insert(seed_template);
        queue.push_back(seed_template);

        while let Some(template) = queue.pop_front() {
            let full_state = template.instantiate(300).unwrap().to_state();
            for &action in PROGRESS_ONLY_SEARCH_ACTIONS {
                if let Ok(new_full_state) = use_action_combo(&self.settings, full_state, action) {
                    let new_reduced_state = ReducedState::from_state(&new_full_state);
                    let new_template = Template {
                        max_cp: u16::MAX,
                        data: TemplateData {
                            durability: new_reduced_state.durability,
                            effects: new_reduced_state.effects,
                        },
                    };
                    if templates.insert(new_template) {
                        queue.push_back(new_template);
                    }
                }
            }
        }

        templates.into_iter().collect()
    }

    pub fn precompute(&mut self) {
        let mut templates = self.generate_precompute_templates();
        for cp in 0..=self.settings.max_cp() + 10 {
            let solved_states = templates
                .par_iter_mut()
                .filter_map(|template| {
                    let mut cp_offset =
                        (self.settings.max_durability() - template.data.durability) / 10;
                    if template.data.effects.trained_
                    template.instantiate(cp).map(|state| (template, state));
                    todo!()
                })
                .map(|(template, state)| {
                    let solution = self.solve_precompute_state(state);
                    if solution >= self.settings.max_progress() {
                        template.max_cp = cp;
                    }
                    (state, solution)
                })
                .collect_vec_list();
            self.solved_states
                .extend(solved_states.into_iter().flatten());
        }
    }

    fn solve_precompute_state(&self, state: ReducedState) -> u32 {
        let mut best_progress = 0;
        for &action in PROGRESS_ONLY_SEARCH_ACTIONS {
            if let Ok(new_state) = use_action_combo(&self.settings, state.to_state(), action) {
                let action_progress = new_state.progress;
                if new_state.is_final(&self.settings.simulator_settings) {
                    best_progress = best_progress.max(action_progress);
                } else {
                    let new_reduced_state = ReducedState::from_state(&new_state);
                    let child_best_progress = *self.solved_states.get(&new_reduced_state).unwrap();
                    best_progress = best_progress.max(child_best_progress + action_progress);
                }
            }
        }
        best_progress
    }

    pub fn can_finish(&mut self, state: &SimulationState) -> bool {
        let reduced_state = ReducedState::from_state(state);
        self.solved_states
            .get(&reduced_state)
            .is_none_or(|&best_progress| {
                state.progress + best_progress >= self.settings.max_progress()
            })
    }

    pub fn num_states(&self) -> usize {
        self.solved_states.len()
    }
}

impl Drop for FinishSolver {
    fn drop(&mut self) {
        log::debug!("FinishSolver - states: {}", self.solved_states.len());
    }
}
