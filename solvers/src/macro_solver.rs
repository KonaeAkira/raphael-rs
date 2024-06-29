use pareto_front::{Dominate, ParetoFront};
use radix_heap::RadixHeapMap;
use rustc_hash::FxHashMap;
use simulator::state::InProgress;

use crate::actions::{DURABILITY_ACTIONS, MIXED_ACTIONS, PROGRESS_ACTIONS, QUALITY_ACTIONS};
use crate::utils::NamedTimer;
use crate::{FinishSolver, UpperBoundSolver};
use simulator::{Action, ActionMask, ComboAction, Condition, Effects, Settings, SimulationState};

use std::vec::Vec;

const SEARCH_ACTIONS: ActionMask = PROGRESS_ACTIONS
    .union(QUALITY_ACTIONS)
    .union(MIXED_ACTIONS)
    .union(DURABILITY_ACTIONS);

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
        match self.do_solve(state) {
            Some(solution) => Some(solution),
            None => self.finish_solver.get_finish_sequence(state),
        }
    }

    fn do_solve(&mut self, state: InProgress) -> Option<Vec<Action>> {
        let mut pareto_dominated_nodes: usize = 0;
        let mut finish_solver_rejected_nodes: usize = 0;
        let mut upper_bound_solver_rejected_nodes: usize = 0;

        let mut pareto_fronts = FxHashMap::default();

        // priority queue based on quality upper bound
        let mut search_queue = RadixHeapMap::new();

        // backtracking data
        let mut traces: Vec<Option<SearchTrace>> = Vec::new();

        let mut best_quality = 0;
        let mut best_state = None;
        let mut best_trace = 0;

        pareto_fronts
            .entry(ParetoKey::from(*state.raw_state()))
            .or_insert(ParetoFront::new())
            .push(ParetoValue::from(*state.raw_state()));
        search_queue.push(
            {
                let _timer = NamedTimer::new("Quality upper bound");
                self.bound_solver.quality_upper_bound(state)
            },
            SearchNode {
                state,
                backtrack_index: 0,
            },
        );
        traces.push(None);

        let _timer = NamedTimer::new("A* search");

        while let Some((quality_bound, node)) = search_queue.pop() {
            if best_quality == self.settings.max_quality || quality_bound <= best_quality {
                continue;
            }
            for action in SEARCH_ACTIONS
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
                        let quality_bound = self.bound_solver.quality_upper_bound(in_progress);
                        if quality_bound <= best_quality {
                            upper_bound_solver_rejected_nodes += 1;
                            continue;
                        }
                        // skip this state if it is Pareto-dominated
                        if !pareto_fronts
                            .entry(ParetoKey::from(state))
                            .or_insert(ParetoFront::new())
                            .push(ParetoValue::from(state))
                        {
                            pareto_dominated_nodes += 1;
                            continue;
                        }

                        if quality_bound > search_queue.top().unwrap() {
                            dbg!(quality_bound, search_queue.top().unwrap());
                            dbg!(&node, &in_progress);
                        }

                        search_queue.push(
                            quality_bound,
                            SearchNode {
                                state: in_progress,
                                backtrack_index: traces.len() as _,
                            },
                        );
                        traces.push(Some(SearchTrace {
                            parent: node.backtrack_index as _,
                            action,
                        }));

                        let quality = self.settings.max_quality - state.missing_quality;
                        if quality > best_quality {
                            best_quality = quality;
                            best_state = Some(state);
                            best_trace = traces.len() - 1;
                        }
                    }
                }
            }
        }

        let best_actions = match best_state {
            Some(best_state) => {
                let trace_actions = get_actions(&traces, best_trace);
                let finish_actions = self
                    .finish_solver
                    .get_finish_sequence(best_state.try_into().unwrap())
                    .unwrap();
                Some(trace_actions.chain(finish_actions).collect())
            }
            None => None,
        };

        dbg!(
            traces.len(),
            finish_solver_rejected_nodes,
            upper_bound_solver_rejected_nodes,
            pareto_dominated_nodes,
        );

        dbg!(best_quality, &best_actions);
        best_actions
    }
}

#[derive(Debug, Clone)]
struct SearchNode {
    pub state: InProgress,
    pub backtrack_index: u32,
}

#[derive(Debug, Clone, Copy)]
struct SearchTrace {
    pub parent: u32,
    pub action: Action,
}

fn get_actions(traces: &[Option<SearchTrace>], mut index: usize) -> impl Iterator<Item = Action> {
    let mut actions = Vec::new();
    while let Some(trace) = traces[index] {
        actions.push(trace.action);
        index = trace.parent as usize;
    }
    actions.into_iter().rev()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ParetoKey {
    durability: i8,
    effects: Effects,
    combo: Option<ComboAction>,
}

impl std::convert::From<SimulationState> for ParetoKey {
    fn from(state: SimulationState) -> Self {
        Self {
            durability: state.durability,
            effects: state.effects,
            combo: state.combo,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ParetoValue {
    cp: i16,
    missing_progress: u16,
    missing_quality: u16,
}

impl std::convert::From<SimulationState> for ParetoValue {
    fn from(state: SimulationState) -> Self {
        Self {
            cp: state.cp,
            missing_progress: state.missing_progress,
            missing_quality: state.missing_quality,
        }
    }
}

impl Dominate for ParetoValue {
    fn dominate(&self, x: &Self) -> bool {
        self.cp >= x.cp
            && self.missing_progress <= x.missing_progress
            && self.missing_quality <= x.missing_quality
    }
}
