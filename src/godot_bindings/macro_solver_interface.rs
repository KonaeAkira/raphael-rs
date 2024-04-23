use godot::prelude::*;

use crate::{
    game::{
        units::{Progress, Quality},
        Action, Condition, Settings, State,
    },
    solvers::MacroSolver,
};

struct GdExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GdExtension {}

#[derive(GodotClass)]
#[class(base=Node)]
struct MacroSolverInterface {
    base: Base<Node>,

    #[export]
    max_progress: u32,
    #[export]
    max_quality: u32,
    #[export]
    max_durability: i8,
    #[export]
    max_cp: i16,
}

#[godot_api]
impl INode for MacroSolverInterface {
    fn init(_base: Base<Self::Base>) -> Self {
        Self {
            base: _base,
            max_progress: 0,
            max_quality: 0,
            max_durability: 0,
            max_cp: 0,
        }
    }
}

#[godot_api]
impl MacroSolverInterface {
    #[func]
    fn get_number(&self) -> i64 {
        100
    }

    #[signal]
    fn macro_string_changed(value: GString);
    fn emit_macro_string_changed(&mut self, value: GString) {
        self.base_mut()
            .emit_signal("macro_string_changed".into(), &[value.to_variant()]);
    }

    fn get_settings(&self) -> Settings {
        Settings {
            max_cp: self.max_cp,
            max_durability: self.max_durability,
            max_progress: Progress::from(self.max_progress),
            max_quality: Quality::from(self.max_quality),
        }
    }

    #[func]
    fn reset_result(&mut self) {
        self.emit_macro_string_changed("".into());
    }

    fn set_result(&mut self, actions: Vec<Action>) {
        let state = from_action_sequence(&self.get_settings(), &actions);

        let mut lines: Vec<String> = Vec::new();
        for action in actions {
            lines.push(format!("/ac \"{}\" <wait.{}>", action.display_name(), action.duration()))
        }
        self.emit_macro_string_changed(lines.join("\n").into());
    }

    #[func]
    fn solve(&mut self) -> bool {
        let settings = self.get_settings();

        let state = State::new(&settings);
        let mut solver = MacroSolver::new(settings);

        match solver.solve(state) {
            Some(actions) => {
                self.set_result(actions);
                true
            }
            None => false,
        }
    }
}

fn from_action_sequence(settings: &Settings, actions: &[Action]) -> State {
    let mut state: State = State::new(&settings);
    for action in actions {
        state = state.as_in_progress().unwrap().use_action(
            action.clone(),
            Condition::Normal,
            &settings,
        );
    }
    return state;
}
