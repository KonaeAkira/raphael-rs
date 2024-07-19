use radix_heap::RadixHeapMap;
use simulator::state::InProgress;
use simulator::{Action, ActionMask, Condition, Settings};

use super::pareto_set::ParetoSet;
use super::quick_search::quick_search;
use crate::actions::{DURABILITY_ACTIONS, PROGRESS_ACTIONS, QUALITY_ACTIONS};
use crate::macro_solver::fast_lower_bound::fast_lower_bound;
use crate::utils::{Backtracking, NamedTimer};
use crate::{FinishSolver, UpperBoundSolver};

use std::vec::Vec;

const FULL_SEARCH_ACTIONS: ActionMask = PROGRESS_ACTIONS
    .union(QUALITY_ACTIONS)
    .union(DURABILITY_ACTIONS);

const PROGRESS_SEARCH_ACTIONS: ActionMask = PROGRESS_ACTIONS
    .union(DURABILITY_ACTIONS)
    .remove(Action::DelicateSynthesis);

#[derive(Debug, Clone, Copy)]
struct SearchNode {
    state: InProgress,
    backtrack_index: u32,
}

type ProgressCallback<'a> = dyn Fn(&[Action]) + 'a;
pub struct MacroSolver<'a> {
    settings: Settings,
    finish_solver: FinishSolver,
    bound_solver: UpperBoundSolver,
    progress_callback: Box<ProgressCallback<'a>>,
}

impl<'a> MacroSolver<'a> {
    pub fn new<F>(settings: Settings, callback: F) -> MacroSolver<'a> 
    where F: Fn(&[Action]) + 'a {
        dbg!(std::mem::size_of::<SearchNode>());
        dbg!(std::mem::align_of::<SearchNode>());
        MacroSolver {
            settings,
            finish_solver: FinishSolver::new(settings),
            bound_solver: UpperBoundSolver::new(settings),
            progress_callback: Box::new(callback),
        }
    }

    /// Returns a list of Actions that maximizes Quality of the completed state.
    /// Returns `None` if the state cannot be completed (i.e. cannot max out Progress).
    /// The solver makes an effort to produce a short solution, but it is not (yet) guaranteed to be the shortest solution.
    pub fn solve(&mut self, state: InProgress, backload_progress: bool) -> Option<Vec<Action>> {
        let timer = NamedTimer::new("Finish solver");
        if !self.finish_solver.can_finish(&state) {
            return None;
        }
        drop(timer);

        if let Some(actions) = quick_search(
            state,
            &self.settings,
            &mut self.finish_solver,
            &mut self.bound_solver,
        ) {
            return Some(actions);
        }

        let _timer = NamedTimer::new("Full search");
        self.do_solve(state, backload_progress)
    }

    fn do_solve(&mut self, state: InProgress, backload_progress: bool) -> Option<Vec<Action>> {
        let mut pareto_dominated_nodes: usize = 0;
        let mut finish_solver_rejected_nodes: usize = 0;
        let mut upper_bound_solver_rejected_nodes: usize = 0;

        let mut pareto_set = ParetoSet::default();

        let mut search_queue: RadixHeapMap<Score, SearchNode> = RadixHeapMap::new();
        let mut backtracking: Backtracking<Action> = Backtracking::new();

        let mut quality_lower_bound = fast_lower_bound(
            state,
            &self.settings,
            &mut self.finish_solver,
            &mut self.bound_solver,
        );
        let mut solution: Option<(Score, u32)> = None; // (quality, trace_index)

        search_queue.push(
            Score::new(self.bound_solver.quality_upper_bound(state), 0, 0),
            SearchNode {
                state,
                backtrack_index: Backtracking::<Action>::SENTINEL,
            },
        );

        while let Some((score, node)) = search_queue.pop() {
            if score.quality < quality_lower_bound {
                break;
            }
            if solution.is_some() && score <= solution.unwrap().0 {
                break;
            }
            let search_actions = match backload_progress
                && node.state.raw_state().missing_progress != self.settings.max_progress
            {
                true => PROGRESS_SEARCH_ACTIONS.intersection(self.settings.allowed_actions),
                false => FULL_SEARCH_ACTIONS.intersection(self.settings.allowed_actions),
            };
            for action in search_actions.actions_iter() {
                if let Ok(state) = node
                    .state
                    .use_action(action, Condition::Normal, &self.settings)
                {
                    if let Ok(in_progress) = InProgress::try_from(state) {
                        // skip this state if it is impossible to max out Progress
                        if !self.finish_solver.can_finish(&in_progress) {
                            finish_solver_rejected_nodes += 1;
                            continue;
                        }
                        // skip this state if its Quality upper bound is not greater than the current best Quality
                        let quality_upper_bound =
                            self.bound_solver.quality_upper_bound(in_progress);
                        if quality_upper_bound < quality_lower_bound {
                            upper_bound_solver_rejected_nodes += 1;
                            continue;
                        }
                        // skip this state if it is Pareto-dominated
                        if !pareto_set.insert(state) {
                            pareto_dominated_nodes += 1;
                            continue;
                        }

                        let duration = score.duration + action.time_cost() as u8;
                        let backtrack_index = backtracking.push(action, node.backtrack_index);
                        search_queue.push(
                            Score::new(quality_upper_bound, duration, score.steps + 1),
                            SearchNode {
                                state: in_progress,
                                backtrack_index,
                            },
                        );

                        let quality = self.settings.max_quality - state.get_missing_quality();
                        if quality > quality_lower_bound {
                            quality_lower_bound = quality;
                        }
                    } else if state.missing_progress == 0 {
                        let final_score = Score::new(
                            self.settings.max_quality - state.get_missing_quality(),
                            score.duration + action.time_cost() as u8,
                            score.steps + 1,
                        );
                        if solution.is_none() || solution.unwrap().0 < final_score {
                            let backtrack_index = backtracking.push(action, node.backtrack_index);
                            solution = Some((final_score, backtrack_index));
                            let actions: Vec<Action> = backtracking.get(backtrack_index).collect();
                            (self.progress_callback)(&actions);
                        }
                    }
                }
            }
        }

        let actions = match solution {
            Some((score, trace_index)) => {
                dbg!(score);
                Some(backtracking.get(trace_index).collect())
            }
            None => None,
        };

        dbg!(
            finish_solver_rejected_nodes,
            upper_bound_solver_rejected_nodes,
            pareto_dominated_nodes,
        );

        dbg!(&actions);
        actions
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Score {
    quality: u16,
    duration: u8,
    steps: u8,
}

impl Score {
    fn new(quality: u16, duration: u8, steps: u8) -> Self {
        Self {
            quality,
            duration,
            steps,
        }
    }
}

impl std::cmp::PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            self.quality
                .cmp(&other.quality)
                .then(other.duration.cmp(&self.duration))
                .then(other.steps.cmp(&self.steps)),
        )
    }
}

impl std::cmp::Ord for Score {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.quality
            .cmp(&other.quality)
            .then(other.duration.cmp(&self.duration))
            .then(other.steps.cmp(&self.steps))
    }
}

impl radix_heap::Radix for Score {
    const RADIX_BITS: u32 = 32;
    fn radix_similarity(&self, other: &Self) -> u32 {
        if self.quality != other.quality {
            self.quality.radix_similarity(&other.quality)
        } else if self.duration != other.duration {
            self.duration.radix_similarity(&other.duration) + 16
        } else {
            self.steps.radix_similarity(&other.steps) + 24
        }
    }
}
