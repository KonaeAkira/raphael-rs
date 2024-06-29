use radix_heap::RadixHeapMap;
use simulator::state::InProgress;
use simulator::{Action, ActionMask, Condition, Settings};

use super::pareto_set::ParetoSet;
use super::quick_search::quick_search;
use crate::actions::{DURABILITY_ACTIONS, PROGRESS_ACTIONS, QUALITY_ACTIONS};
use crate::utils::{Backtracking, NamedTimer};
use crate::{FinishSolver, UpperBoundSolver};

use std::vec::Vec;

const FULL_SEARCH_ACTIONS: ActionMask = PROGRESS_ACTIONS
    .union(QUALITY_ACTIONS)
    .union(DURABILITY_ACTIONS);

#[derive(Debug, Clone, Copy)]
struct SearchNode {
    state: InProgress,
    backtrack_index: u32,
}

pub struct MacroSolver {
    settings: Settings,
    finish_solver: FinishSolver,
    bound_solver: UpperBoundSolver,
}

impl MacroSolver {
    pub fn new(settings: Settings) -> MacroSolver {
        dbg!(std::mem::size_of::<SearchNode>());
        dbg!(std::mem::align_of::<SearchNode>());
        MacroSolver {
            settings,
            finish_solver: FinishSolver::new(settings),
            bound_solver: UpperBoundSolver::new(settings),
        }
    }

    /// Returns a list of Actions that maximizes Quality of the completed state.
    /// Returns `None` if the state cannot be completed (i.e. cannot max out Progress).
    /// The solver makes an effort to produce a short solution, but it is not (yet) guaranteed to be the shortest solution.
    pub fn solve(&mut self, state: InProgress) -> Option<Vec<Action>> {
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
        match self.do_solve(state) {
            Some(solution) => Some(solution),
            None => self.finish_solver.get_finish_sequence(state),
        }
    }

    fn do_solve(&mut self, state: InProgress) -> Option<Vec<Action>> {
        let mut pareto_dominated_nodes: usize = 0;
        let mut finish_solver_rejected_nodes: usize = 0;
        let mut upper_bound_solver_rejected_nodes: usize = 0;

        let mut pareto_set = ParetoSet::default();

        let mut search_queue: RadixHeapMap<Score, SearchNode> = RadixHeapMap::new();
        let mut backtracking: Backtracking<Action> = Backtracking::new();

        let mut quality_lower_bound = 0;
        let mut solution: Option<(Score, u32)> = None; // (quality, trace_index)

        pareto_set.insert(*state.raw_state());
        search_queue.push(
            Score::new(self.bound_solver.quality_upper_bound(state), 0),
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
            for action in FULL_SEARCH_ACTIONS
                .intersection(self.settings.allowed_actions)
                .actions_iter()
            {
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

                        let backtrack_index = backtracking.push(action, node.backtrack_index);
                        search_queue.push(
                            Score::new(
                                quality_upper_bound,
                                score.duration + action.time_cost() as u8,
                            ),
                            SearchNode {
                                state: in_progress,
                                backtrack_index,
                            },
                        );

                        let quality = self.settings.max_quality - state.missing_quality;
                        if quality > quality_lower_bound {
                            quality_lower_bound = quality;
                        }
                    } else if state.missing_progress == 0 {
                        let final_score = Score::new(
                            self.settings.max_quality - state.missing_quality,
                            score.duration + action.time_cost() as u8,
                        );
                        if solution.is_none() || solution.unwrap().0 < final_score {
                            let backtrack_index = backtracking.push(action, node.backtrack_index);
                            solution = Some((final_score, backtrack_index));
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
}

impl Score {
    fn new(quality: u16, duration: u8) -> Self {
        Self { quality, duration }
    }
}

impl std::cmp::PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            self.quality
                .cmp(&other.quality)
                .then(other.duration.cmp(&self.duration)),
        )
    }
}

impl std::cmp::Ord for Score {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.quality
            .cmp(&other.quality)
            .then(other.duration.cmp(&self.duration))
    }
}

impl radix_heap::Radix for Score {
    const RADIX_BITS: u32 = 24;
    fn radix_similarity(&self, other: &Self) -> u32 {
        if self.quality != other.quality {
            self.quality.radix_similarity(&other.quality)
        } else {
            self.duration.radix_similarity(&other.duration) + 16
        }
    }
}
