use std::time::Instant;
use std::vec::Vec;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    config::Settings,
    game::{
        actions::{Action, QUAL_DENOM},
        conditions::Condition,
        state::{Completed, InProgress, State},
    },
    util::pareto_front::ParetoFront,
};

#[derive(Debug, Clone)]
struct Node {
    state: InProgress,
    trace: Option<(usize, Sequence)>,
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
            for node in bucket.iter() {
                self.pareto_front.insert(&node.state);
            }
            for node in bucket {
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
}

impl MacroSolver {
    pub fn new(settings: Settings) -> MacroSolver {
        MacroSolver {
            settings: settings.clone(),
            search_queue: SearchQueue::new(settings),
            save: Vec::new(),
        }
    }

    pub fn solve(&mut self, state: State) -> Option<Action> {
        match state {
            State::InProgress(state) => {
                let result = self.do_solve(state);
                match result {
                    Some(result) => Some(result.actions[0]),
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
            for sequence in Sequence::iter() {
                if self.should_use(&current_node.state, sequence) {
                    let use_action =
                        self.use_actions(State::InProgress(current_node.state), sequence);
                    match use_action {
                        State::InProgress(new_state) => {
                            self.search_queue.push(Node {
                                state: new_state,
                                trace: Some((self.save.len() - 1, sequence)),
                            });
                        }
                        State::Completed(Completed { quality }) => {
                            let current_quality = match result {
                                None => -1,
                                Some(MacroResult { quality, .. }) => quality,
                            };
                            if current_quality < quality {
                                let new_result = MacroResult {
                                    quality,
                                    actions: self.trace_steps(sequence),
                                };
                                println!(
                                    "result ({}): {:?}",
                                    new_result.quality as f32 / QUAL_DENOM,
                                    new_result.actions
                                );
                                result = Some(new_result);
                            }
                        }
                        _ => (),
                    }
                }
            }
        }

        let time = timer.elapsed().as_secs_f32();
        let nodes = self.save.len() as f32;
        println!("Time elapsed: {}s", time);
        println!(
            "Searched nodes: {:+.2e} ({:+.2e} nodes/s)",
            nodes,
            nodes / time
        );

        result
    }

    fn should_use(&self, state: &InProgress, sequence: Sequence) -> bool {
        if state.last_action.is_none() {
            match sequence {
                Sequence::MuscleMemoryOpener | Sequence::ReflectOpener => true,
                _ => false,
            }
        } else {
            let is_pre_byregots_blessing: bool =
                state.effects.inner_quiet != 0 || state.quality == 0;
            let use_quality_increase: bool =
                state.effects.muscle_memory == 0 && is_pre_byregots_blessing;
            match sequence {
                Sequence::MuscleMemoryOpener => true,
                Sequence::ReflectOpener => true,
                Sequence::MasterMend => state.durability + 30 <= self.settings.max_durability,
                Sequence::CarefulSynthesis => state.effects.muscle_memory == 0,
                Sequence::Groundwork => {
                    state.effects.waste_not != 0 || state.effects.muscle_memory != 0
                }
                Sequence::PreparatoryTouch => state.effects.waste_not != 0 && use_quality_increase,
                Sequence::PrudentTouch => use_quality_increase,
                Sequence::TrainedFinesse => true,
                Sequence::AdvancedTouchCombo => {
                    use_quality_increase
                        && (state.effects.innovation >= 3 || state.effects.innovation == 0)
                }
                Sequence::FocusedSynthesisCombo => {
                    state.effects.muscle_memory == 0
                        && (state.effects.veneration >= 2 || state.effects.veneration == 0)
                }
                Sequence::FocusedTouchCombo => {
                    use_quality_increase
                        && (state.effects.innovation >= 2 || state.effects.innovation == 0)
                }
                Sequence::Manipulation => state.effects.manipulation == 0,
                Sequence::WasteNot => state.effects.waste_not == 0,
                Sequence::WasteNot2 => state.effects.waste_not == 0,
                Sequence::Innovation => {
                    state.effects.innovation == 0
                        && state.effects.muscle_memory == 0
                        && use_quality_increase
                }
                Sequence::Veneration => {
                    state.effects.veneration == 0
                        && (!use_quality_increase
                            || state.effects.muscle_memory != 0
                            || state.cp <= 96)
                    // 96 cp = veneration + 4 * groundwork
                }
                Sequence::ByresgotsBlessingCombo => state.effects.inner_quiet >= 4,
                Sequence::ByregotsBlessing => state.effects.inner_quiet >= 4,
            }
        }
    }

    fn use_actions(&self, mut state: State, sequence: Sequence) -> State {
        for action in sequence.to_slice() {
            match state {
                State::InProgress(in_progress) => {
                    state = in_progress.use_action(*action, Condition::Normal, &self.settings);
                }
                _ => return State::Invalid,
            }
        }
        state
    }

    fn trace_steps(&self, last_sequence: Sequence) -> Vec<Action> {
        let mut steps: Vec<Action> = Vec::new();
        for action in last_sequence.to_slice().iter().rev() {
            steps.push(*action);
        }

        let mut trace: Option<(usize, Sequence)> = self.save.last().unwrap().trace;
        while let Some((i, sequence)) = trace {
            for action in sequence.to_slice().iter().rev() {
                steps.push(*action);
            }
            trace = self.save[i].trace;
        }

        steps.reverse();
        steps
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumIter)]
enum Sequence {
    // opener
    MuscleMemoryOpener,
    ReflectOpener,
    // singles
    MasterMend,
    CarefulSynthesis,
    Groundwork,
    PreparatoryTouch,
    PrudentTouch,
    TrainedFinesse,
    // combos
    AdvancedTouchCombo,
    FocusedSynthesisCombo,
    FocusedTouchCombo,
    // effects
    Manipulation,
    WasteNot,
    WasteNot2,
    Innovation,
    Veneration,
    // finisher
    ByresgotsBlessingCombo,
    ByregotsBlessing,
}

impl Sequence {
    pub fn to_slice(&self) -> &[Action] {
        match *self {
            Sequence::CarefulSynthesis => &[Action::CarefulSynthesis],
            Sequence::Groundwork => &[Action::Groundwork],
            Sequence::PreparatoryTouch => &[Action::PreparatoryTouch],
            Sequence::PrudentTouch => &[Action::PrudentTouch],
            Sequence::TrainedFinesse => &[Action::TrainedFinesse],
            Sequence::AdvancedTouchCombo => &[
                Action::BasicTouch,
                Action::StandardTouch,
                Action::AdvancedTouch,
            ],
            Sequence::FocusedSynthesisCombo => &[Action::Observe, Action::FocusedSynthesis],
            Sequence::FocusedTouchCombo => &[Action::Observe, Action::FocusedTouch],
            Sequence::MasterMend => &[Action::MasterMend],
            Sequence::Manipulation => &[Action::Manipulation],
            Sequence::WasteNot => &[Action::WasteNot],
            Sequence::WasteNot2 => &[Action::WasteNot2],
            Sequence::Innovation => &[Action::Innovation],
            Sequence::Veneration => &[Action::Veneration],
            Sequence::ByresgotsBlessingCombo => &[Action::GreatStrides, Action::ByregotsBlessing],
            Sequence::ByregotsBlessing => &[Action::ByregotsBlessing],
            Sequence::MuscleMemoryOpener => &[Action::MuscleMemory],
            Sequence::ReflectOpener => &[Action::Reflect],
        }
    }
}
