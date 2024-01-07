use std::collections::HashSet;
use std::time::Instant;
use std::vec::Vec;

use crate::{
    config::Settings,
    game::{
        actions::{Action, QUAL_DENOM},
        conditions::Condition,
        state::{Completed, InProgress, State},
    },
};

static OPENERS: &[&[Action]] = &[&[Action::MuscleMemory], &[Action::Reflect]];

static ACTIONS: &[&[Action]] = &[
    // singles
    &[Action::CarefulSynthesis],
    &[Action::Groundwork],
    &[Action::PreparatoryTouch],
    &[Action::PrudentTouch],
    &[Action::TrainedFinesse],
    // combos
    &[
        Action::BasicTouch,
        Action::StandardTouch,
        Action::AdvancedTouch,
    ],
    &[Action::Observe, Action::FocusedSynthesis],
    &[Action::Observe, Action::FocusedTouch],
    // effects
    &[Action::MasterMend],
    &[Action::Manipulation],
    &[Action::WasteNot],
    &[Action::WasteNot2],
    &[Action::Innovation],
    &[Action::Veneration],
    // finisher
    &[Action::GreatStrides, Action::ByregotsBlessing],
    &[Action::ByregotsBlessing],
];

#[derive(Debug, Clone)]
struct Node {
    state: InProgress,
    parent_node_index: Option<usize>,
    last_action: Vec<Action>,
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

        let mut visited_states: HashSet<InProgress> = HashSet::new();
        let mut search_queue: Vec<Node> = Vec::new();

        visited_states.insert(state.clone());
        search_queue.push(Node {
            state,
            parent_node_index: None,
            last_action: Vec::new(),
        });

        let mut result: Option<MacroResult> = None;

        let mut i: usize = 0;
        while i < search_queue.len() {
            let current_node: Node = search_queue[i].clone();
            for actions in self.search_space(&current_node.state) {
                let use_action = self.use_actions(State::InProgress(current_node.state), actions);
                match use_action {
                    State::InProgress(new_state) => {
                        if !visited_states.contains(&new_state) {
                            visited_states.insert(new_state.clone());
                            search_queue.push(Node {
                                state: new_state,
                                parent_node_index: Some(i),
                                last_action: actions.to_vec(),
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
                                actions: self.trace_steps(&search_queue, i, actions),
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

    fn use_actions(&self, mut state: State, actions: &[Action]) -> State {
        for action in actions {
            match state {
                State::InProgress(in_progress) => {
                    state = in_progress.use_action(*action, Condition::Normal, &self.settings);
                }
                _ => return State::Invalid,
            }
        }
        state
    }

    fn search_space(&self, state: &InProgress) -> &[&[Action]] {
        if state.last_action.is_none() {
            OPENERS
        } else {
            ACTIONS
        }
    }

    fn trace_steps(&self, nodes: &Vec<Node>, index: usize, last_action: &[Action]) -> Vec<Action> {
        let mut steps: Vec<Action> = Vec::new();
        for action in last_action.iter().rev() {
            steps.push(*action);
        }

        let mut index: Option<usize> = nodes[index].parent_node_index;
        while let Some(i) = index {
            for action in nodes[i].last_action.iter().rev() {
                steps.push(*action);
            }
            index = nodes[i].parent_node_index;
        }

        steps.reverse();
        steps
    }
}
