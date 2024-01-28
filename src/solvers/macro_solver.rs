use std::time::Instant;
use std::vec::Vec;

use crate::{
    config::Settings,
    game::{
        actions::{Action, QUAL_DENOM},
        state::{InProgress, State},
    },
    solvers::{
        finish_solver::FinishSolver,
        util::{action_sequence::ActionSequence, pareto_front::ParetoFront},
    },
};

#[derive(Debug, Clone)]
struct Node {
    state: InProgress,
    trace: Option<(usize, ActionSequence)>,
}

struct SearchQueue {
    seed: Vec<Node>,
    buckets: Vec<Vec<Node>>,
    pareto_front: ParetoFront,
}

impl SearchQueue {
    pub fn new(settings: Settings) -> SearchQueue {
        SearchQueue {
            seed: Vec::new(),
            buckets: vec![Vec::new(); (settings.max_cp + 1) as usize],
            pareto_front: ParetoFront::new(),
        }
    }

    pub fn push_seed(&mut self, node: Node) {
        self.seed.push(node);
    }

    pub fn push(&mut self, node: Node) {
        self.buckets[node.state.cp as usize].push(node);
    }

    pub fn pop(&mut self) -> Option<Node> {
        if let Some(node) = self.seed.pop() {
            return Some(node);
        } else if self.pop_bucket() {
            return self.pop();
        } else {
            return None;
        }
    }

    fn pop_bucket(&mut self) -> bool {
        if let Some(bucket) = self.buckets.pop() {
            let mut unique: Vec<Node> = Vec::new();
            for node in bucket {
                if self.pareto_front.insert(&node.state) {
                    unique.push(node);
                }
            }
            for node in unique {
                if self.pareto_front.has(&node.state) {
                    self.seed.push(node);
                }
            }
            return true;
        } else {
            return false;
        }
    }
}

struct MacroResult {
    quality: i32,
    actions: Vec<Action>,
}

pub struct MacroSolver {
    settings: Settings,
    search_queue: SearchQueue,
    save: Vec<Node>,
    finish_solver: FinishSolver,
}

impl MacroSolver {
    pub fn new(settings: Settings) -> MacroSolver {
        MacroSolver {
            settings: settings.clone(),
            search_queue: SearchQueue::new(settings.clone()),
            save: Vec::new(),
            finish_solver: FinishSolver::new(settings),
        }
    }

    pub fn solve(&mut self, state: State) -> Option<Vec<Action>> {
        match state {
            State::InProgress(state) => {
                let result = self.do_solve(state);
                match result {
                    Some(result) => Some(result.actions),
                    None => None,
                }
            }
            _ => None,
        }
    }

    fn do_solve(&mut self, state: InProgress) -> Option<MacroResult> {
        let timer = Instant::now();

        self.search_queue.push_seed(Node { state, trace: None });

        let mut result: Option<MacroResult> = None;

        while let Some(current_node) = self.search_queue.pop() {
            self.save.push(current_node.clone());
            for sequence in ACTION_SEQUENCES {
                if self.should_use(&current_node.state, sequence) {
                    let use_action = sequence.apply(
                        State::InProgress(current_node.state.clone()),
                        &self.settings,
                    );
                    match use_action {
                        State::InProgress(state) => {
                            let best_quality = match result {
                                None => -1,
                                Some(MacroResult { quality, .. }) => quality,
                            };
                            if state.quality > best_quality
                                && state.effects.muscle_memory == 0
                                && self.finish_solver.can_finish(&state)
                            {
                                let mut new_result = MacroResult {
                                    quality: state.quality,
                                    actions: self.trace_steps(sequence),
                                };
                                new_result.actions.append(
                                    &mut self.finish_solver.get_finish_sequence(&state).unwrap(),
                                );
                                log::trace!(
                                    "result ({}): {:?}",
                                    new_result.quality as f32 / QUAL_DENOM as f32,
                                    new_result.actions
                                );
                                result = Some(new_result);
                            }
                            self.search_queue.push(Node {
                                state,
                                trace: Some((self.save.len() - 1, sequence)),
                            });
                        }
                        _ => (),
                    }
                }
            }
        }

        let time = timer.elapsed().as_secs_f32();
        let nodes = self.save.len() as f32;
        log::debug!("Time elapsed: {}s", time);
        log::debug!(
            "Searched nodes: {:+.2e} ({:+.2e} nodes/s)",
            nodes,
            nodes / time
        );

        result
    }

    fn should_use(&self, state: &InProgress, sequence: ActionSequence) -> bool {
        if state.last_action.is_none() {
            match sequence {
                ActionSequence::MuscleMemoryOpener | ActionSequence::ReflectOpener => true,
                _ => false,
            }
        } else if state.effects.inner_quiet == 0 && state.quality != 0 {
            false // don't do anything after Byregot's Blessing
        } else {
            let use_progress_increase: bool =
                state.effects.muscle_memory != 0 || state.effects.veneration != 0;
            let use_quality_increase: bool =
                state.effects.muscle_memory == 0 && state.effects.veneration <= 1;
            match sequence {
                ActionSequence::MuscleMemoryOpener => false,
                ActionSequence::ReflectOpener => false,
                ActionSequence::MasterMend => state.durability + 30 <= self.settings.max_durability,
                ActionSequence::CarefulSynthesis => {
                    use_progress_increase && state.effects.muscle_memory == 0
                }
                ActionSequence::Groundwork => use_progress_increase,
                ActionSequence::PreparatoryTouch => {
                    use_quality_increase && state.effects.waste_not != 0
                }
                ActionSequence::PrudentTouch => use_quality_increase,
                ActionSequence::TrainedFinesse => state.effects.inner_quiet == 10,
                ActionSequence::AdvancedTouchCombo => {
                    use_quality_increase
                        && (state.effects.innovation >= 3 || state.effects.innovation == 0)
                }
                ActionSequence::FocusedSynthesisCombo => {
                    use_progress_increase
                        && state.effects.muscle_memory == 0
                        && (state.effects.veneration >= 2 || state.effects.veneration == 0)
                }
                ActionSequence::FocusedTouchCombo => {
                    use_quality_increase
                        && (state.effects.innovation >= 2 || state.effects.innovation == 0)
                }
                ActionSequence::Manipulation => state.effects.manipulation == 0,
                ActionSequence::WasteNot => state.effects.waste_not == 0,
                ActionSequence::WasteNot2 => state.effects.waste_not == 0,
                ActionSequence::Innovation => use_quality_increase && state.effects.innovation == 0,
                ActionSequence::Veneration => state.effects.muscle_memory != 0,
                ActionSequence::ByresgotsBlessingCombo => state.effects.inner_quiet >= 4,
                ActionSequence::ByregotsBlessing => state.effects.inner_quiet >= 3,
            }
        }
    }

    fn trace_steps(&self, last_sequence: ActionSequence) -> Vec<Action> {
        let mut steps: Vec<Action> = Vec::new();
        for action in last_sequence.actions().iter().rev() {
            steps.push(*action);
        }

        let mut trace: Option<(usize, ActionSequence)> = self.save.last().unwrap().trace;
        while let Some((i, sequence)) = trace {
            for action in sequence.actions().iter().rev() {
                steps.push(*action);
            }
            trace = self.save[i].trace;
        }

        steps.reverse();
        steps
    }
}

const ACTION_SEQUENCES: [ActionSequence; 18] = [
    // opener
    ActionSequence::MuscleMemoryOpener,
    ActionSequence::ReflectOpener,
    // singles
    ActionSequence::MasterMend,
    ActionSequence::CarefulSynthesis,
    ActionSequence::Groundwork,
    ActionSequence::PreparatoryTouch,
    ActionSequence::PrudentTouch,
    ActionSequence::TrainedFinesse,
    // combos
    ActionSequence::AdvancedTouchCombo,
    ActionSequence::FocusedSynthesisCombo,
    ActionSequence::FocusedTouchCombo,
    // effects
    ActionSequence::Manipulation,
    ActionSequence::WasteNot,
    ActionSequence::WasteNot2,
    ActionSequence::Innovation,
    ActionSequence::Veneration,
    // finisher
    ActionSequence::ByresgotsBlessingCombo,
    ActionSequence::ByregotsBlessing,
];
