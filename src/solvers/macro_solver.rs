use std::collections::HashSet;
use std::vec::Vec;

use crate::{
    config::Settings,
    game::{
        actions::{Action, QUAL_DENOM},
        conditions::Condition,
        state::{Completed, InProgress, State}
    }
};

static MACRO_ACTIONS: [Action; 24] = [
    Action::BasicSynthesis,
    Action::BasicTouch,
    Action::MasterMend,
    Action::Observe,
    Action::WasteNot,
    Action::Veneration,
    Action::StandardTouch,
    Action::GreatStrides,
    Action::Innovation,
    Action::WasteNot2,
    Action::ByregotsBlessing,
    Action::MuscleMemory,
    Action::CarefulSynthesis,
    Action::Manipulation,
    Action::PrudentTouch,
    Action::FocusedSynthesis,
    Action::FocusedTouch,
    Action::Reflect,
    Action::PreparatoryTouch,
    Action::Groundwork,
    Action::DelicateSynthesis,
    Action::AdvancedTouch,
    Action::PrudentSynthesis,
    Action::TrainedFinesse,
];

#[derive(Debug, Clone)]
struct Node {
    state: InProgress,
    parent_node_index: Option<usize>,
    time_cost: i32,
    steps: i32,
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
        let mut visited_states: HashSet<InProgress> = HashSet::new();
        let mut search_queue: Vec<Node> = Vec::new();

        visited_states.insert(state.clone());
        search_queue.push(Node {
            state,
            parent_node_index: None,
            time_cost: 0,
            steps: 0,
        });

        let mut result: Option<MacroResult> = None;

        let mut i: usize = 0;
        while i < search_queue.len() {
            let current_node: Node = search_queue[i].clone();
            for action in MACRO_ACTIONS {
                let use_action =
                    current_node
                        .state
                        .use_action(action, Condition::Normal, &self.settings);
                match use_action {
                    State::InProgress(new_state) => {
                        if !visited_states.contains(&new_state) {
                            visited_states.insert(new_state.clone());
                            search_queue.push(Node {
                                state: new_state,
                                parent_node_index: Some(i),
                                time_cost: current_node.time_cost + action.time_cost(),
                                steps: current_node.steps + 1,
                            });
                        }
                    }
                    State::Completed(Completed { quality }) => {
                        let current_quality = match result {
                            None => -1,
                            Some(MacroResult { quality, .. }) => quality,
                        };
                        if current_quality < quality {
                            let mut new_result = MacroResult {
                                quality,
                                actions: Vec::new(),
                            };
                            new_result.actions.push(action);
                            let mut backtrack_index = i;
                            while backtrack_index != 0 {
                                let backtrack_node: &Node = &search_queue[backtrack_index];
                                new_result
                                    .actions
                                    .push(backtrack_node.state.last_action.unwrap());
                                backtrack_index = backtrack_node.parent_node_index.unwrap();
                            }
                            new_result.actions.reverse();
                            println!("new result ({}): {:?}", new_result.quality as f32 / QUAL_DENOM, new_result.actions);
                            result = Some(new_result);
                        }
                    }
                    _ => (),
                }
            }
            i += 1;
        }

        result
    }
}
