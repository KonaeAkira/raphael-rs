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

struct MacroResult {
    quality: i32,
    actions: Vec<Action>,
}

pub struct MacroSolver {
    settings: Settings,
}

impl MacroSolver {
    pub fn new(settings: Settings) -> MacroSolver {
        MacroSolver { settings }
    }

    pub fn solve(&self, state: State) -> Option<Action> {
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

    fn do_solve(&self, state: InProgress) -> Option<MacroResult> {
        let timer = Instant::now();

        let mut pareto_front: ParetoFront = ParetoFront::new();
        let mut search_queue: Vec<Node> = Vec::new();

        pareto_front.insert(&state);
        search_queue.push(Node { state, trace: None });

        let mut result: Option<MacroResult> = None;

        let mut i: usize = 0;
        while i < search_queue.len() {
            let current_node: Node = search_queue[i].clone();
            for sequence in Sequence::iter() {
                if self.should_use(&current_node.state, sequence) {
                    let use_action =
                        self.use_actions(State::InProgress(current_node.state), sequence);
                    match use_action {
                        State::InProgress(new_state) => {
                            if pareto_front.insert(&new_state) {
                                search_queue.push(Node {
                                    state: new_state,
                                    trace: Some((i, sequence)),
                                });
                            }
                        }
                        State::Completed(Completed { quality }) => {
                            let current_quality = match result {
                                None => -1,
                                Some(MacroResult { quality, .. }) => quality,
                            };
                            if current_quality < quality {
                                let new_result = MacroResult {
                                    quality,
                                    actions: self.trace_steps(&search_queue, i, sequence),
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
            i += 1;
        }

        let time = timer.elapsed().as_secs_f32();
        let nodes = search_queue.len() as f32;
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
            match sequence {
                Sequence::Groundwork | Sequence::PreparatoryTouch => state.effects.waste_not != 0,
                // don't waste effects
                Sequence::Innovation => {
                    state.effects.innovation == 0 && state.effects.muscle_memory == 0
                }
                Sequence::Veneration => state.effects.veneration == 0,
                Sequence::WasteNot => state.effects.waste_not == 0,
                Sequence::WasteNot2 => state.effects.waste_not == 0,
                Sequence::Manipulation => state.effects.manipulation == 0,
                // don't waste recovered durability
                Sequence::MasterMend => state.durability + 30 <= self.settings.max_durability,
                _ => true,
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

    fn trace_steps(&self, nodes: &Vec<Node>, index: usize, last_sequence: Sequence) -> Vec<Action> {
        let mut steps: Vec<Action> = Vec::new();
        for action in last_sequence.to_slice().iter().rev() {
            steps.push(*action);
        }

        let mut trace: Option<(usize, Sequence)> = nodes[index].trace;
        while let Some((i, sequence)) = trace {
            for action in sequence.to_slice().iter().rev() {
                steps.push(*action);
            }
            trace = nodes[i].trace;
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
