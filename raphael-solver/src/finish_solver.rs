use raphael_sim::*;
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    SolverException, SolverSettings,
    actions::{FULL_SEARCH_ACTIONS, PROGRESS_ONLY_SEARCH_ACTIONS, use_action_combo},
    macros::internal_error,
};

#[derive(Default)]
struct CpProgressBreakpoints {
    /// List of CP breakpoints and the associated achievable Progress.
    /// Sorted in order of ascending CP.
    breakpoints: Vec<(u16, u32)>,
    /// The maximum CP at which the state was solved.
    /// Querying the solution at a CP higher than this may give incorrect results.
    max_solved_cp: Option<u16>,
}

impl CpProgressBreakpoints {
    fn get_progress(&self, cp: u16) -> Option<u32> {
        if Some(cp) > self.max_solved_cp {
            return None;
        }
        let partition_idx = self.breakpoints.partition_point(|&v| v.0 <= cp);
        partition_idx
            .checked_sub(1)
            .map(|idx| self.breakpoints[idx].1)
            .or(Some(0))
    }

    /// Add a new (CP, Durability) breakpoint.
    /// Breakpoints must be added with strictly increasing CP, otherwise `get_progress` may return wrong results.
    /// If the new breakpoint does not have strictly better Progress than the previous breakpoint, it is ignored.
    fn add_breakpoint(&mut self, cp: u16, progress: u32) {
        self.max_solved_cp = Some(cp);
        if self.breakpoints.last().is_none_or(|last| last.1 < progress) {
            self.breakpoints.push((cp, progress));
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FinishSolverStats {
    pub states: usize,
    pub values: usize,
}

pub struct FinishSolver {
    settings: SolverSettings,
    solved_states: FxHashMap<(u16, Effects), CpProgressBreakpoints>,
}

impl FinishSolver {
    pub fn new(settings: SolverSettings) -> Self {
        Self {
            settings,
            solved_states: FxHashMap::default(),
        }
    }

    /// Calling this method before calling `FinishSolver::precompute` will return a `SolverException`.
    pub fn can_finish(&self, state: &SimulationState) -> Result<bool, SolverException> {
        let key = (state.durability, state.effects.strip_quality_effects());
        if !self.solved_states.contains_key(&key) {
            dbg!(key);
        }
        let breakpoints = self.solved_states.get(&key).ok_or_else(|| {
            internal_error!(
                "State not found in FinishSolver solved states.",
                self.settings,
                state
            )
        })?;
        let max_additional_progress = breakpoints.get_progress(state.cp).ok_or_else(|| {
            internal_error!(
                "State found in FinishSolver solved states but with not enough CP.",
                self.settings,
                state
            )
        })?;
        Ok(state.progress + max_additional_progress >= self.settings.max_progress())
    }

    pub fn precompute(&mut self) -> Result<(), SolverException> {
        let mut templates = generate_templates(&self.settings);
        while !templates.is_empty() {
            templates
                .par_iter_mut()
                .for_each(|template| self.solve_template(template));
            if !templates.iter().any(|t| t.current_max_progress.is_some()) {
                // At least one template must be solved for the precompute loop to make any progress.
                // No template solved in this iteration means that there also won't be any templates solved in the next iteration and so on.
                return Err(internal_error!(
                    "Infinite loop detected in FinishSolver precompute.",
                    self.settings
                ));
            }
            for template in &mut templates {
                if let Some(progress) = template.current_max_progress {
                    let key = (template.durability, template.effects);
                    let breakpoints = self.solved_states.entry(key).or_default();
                    breakpoints.add_breakpoint(template.current_cp, progress);
                    if progress >= self.settings.max_progress() {
                        breakpoints.max_solved_cp = Some(u16::MAX);
                    }
                    template.current_cp += 1;
                }
            }
            templates.retain(|template| {
                template.current_cp <= self.settings.max_cp()
                    && template.current_max_progress < Some(self.settings.max_progress())
            });
        }
        Ok(())
    }

    fn solve_template(&self, template: &mut Template) {
        let state = SimulationState {
            cp: template.current_cp,
            durability: template.durability,
            progress: 0,
            quality: 0,
            unreliable_quality: 0,
            effects: template.effects,
        };
        let mut result = 0;
        for action in PROGRESS_ONLY_SEARCH_ACTIONS {
            if let Ok(child_state) = use_action_combo(&self.settings, state, action) {
                let key = (child_state.durability, child_state.effects);
                if child_state.is_final(&self.settings.simulator_settings) {
                    result = std::cmp::max(result, child_state.progress);
                } else if let Some(child_breakpoints) = self.solved_states.get(&key)
                    && let Some(child_progress) = child_breakpoints.get_progress(child_state.cp)
                {
                    result = std::cmp::max(result, child_state.progress + child_progress);
                } else {
                    // Required child state has not been solved yet.
                    // Abort and try again in the next iteration.
                    return;
                }
            }
        }
        template.current_max_progress = Some(result);
    }

    pub fn runtime_stats(&self) -> FinishSolverStats {
        FinishSolverStats {
            states: self.solved_states.len(),
            values: self
                .solved_states
                .values()
                .map(|breakpoints| breakpoints.breakpoints.len())
                .sum(),
        }
    }
}

#[derive(Debug)]
struct Template {
    durability: u16,
    effects: Effects,
    current_cp: u16,
    current_max_progress: Option<u32>,
}

fn generate_templates(settings: &SolverSettings) -> Vec<Template> {
    let mut initial_state = SimulationState::new(&settings.simulator_settings);
    initial_state.effects = initial_state.effects.strip_quality_effects();
    let mut templates = FxHashSet::default();
    templates.insert((initial_state.durability, initial_state.effects));
    let mut stack = vec![initial_state];
    while let Some(mut state) = stack.pop() {
        state
            .effects
            .set_special_quality_state(SpecialQualityState::Normal);
        for action in FULL_SEARCH_ACTIONS {
            if let Ok(mut new_state) = use_action_combo(settings, state, action)
                && new_state.durability > 0
            {
                new_state.effects = new_state.effects.strip_quality_effects();
                new_state.progress = 0;
                new_state.cp = settings.max_cp();
                if templates.insert((new_state.durability, new_state.effects)) {
                    stack.push(new_state);
                }
            }
        }
    }
    templates
        .into_iter()
        .map(|(durability, effects)| Template {
            durability,
            effects,
            current_cp: 0,
            current_max_progress: None,
        })
        .collect()
}
