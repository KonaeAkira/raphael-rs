use std::{
    sync::{Arc, Mutex},
    thread,
};

use godot::prelude::*;

use crate::{
    game::{
        units::{Durability, Progress, Quality, CP},
        Action, Condition, Settings, State,
    },
    solvers::MacroSolver,
};

#[derive(GodotClass)]
#[class(base=Node)]
struct MacroSolverInterface {
    base: Base<Node>,

    #[export]
    solver_busy: bool,
    solver_result: Arc<Mutex<Option<Vec<Action>>>>,

    #[export]
    configuration: Dictionary,
    #[export]
    simulation: Dictionary,
    #[export]
    macro_string: GString,
}

#[godot_api]
impl INode for MacroSolverInterface {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,
            solver_busy: false,
            solver_result: Arc::new(Mutex::new(None)),
            configuration: Dictionary::new(),
            simulation: dict! {
                "PROGRESS": 0.0,
                "QUALITY": 0.0,
                "DURABILITY": 0.0,
                "CP": 0.0,
            },
            macro_string: GString::new(),
        }
    }
}

#[godot_api]
impl MacroSolverInterface {
    #[signal]
    fn state_updated();
    fn emit_state_updated(&mut self) {
        self.base_mut().emit_signal("state_updated".into(), &[]);
    }

    fn get_settings(&self) -> Settings {
        let max_progress: f32 = self.configuration.get_or_nil("MAX_PROGRESS").to();
        let max_quality: f32 = self.configuration.get_or_nil("MAX_QUALITY").to();
        let base_progress: f32 = self.configuration.get_or_nil("PROGRESS_INCREASE").to();
        let base_quality: f32 = self.configuration.get_or_nil("QUALITY_INCREASE").to();
        Settings {
            max_cp: self.configuration.get_or_nil("MAX_CP").to::<f64>() as CP,
            max_durability: self.configuration.get_or_nil("MAX_DURABILITY").to::<f64>()
                as Durability,
            max_progress: Progress::from(100.0 * max_progress / base_progress),
            max_quality: Quality::from(100.0 * max_quality / base_quality),
        }
    }

    #[func]
    fn reset_result(&mut self) {
        self.simulation = dict! {
            "PROGRESS": 0.0,
            "QUALITY": 0.0,
            "DURABILITY": self.configuration.get_or_nil("MAX_DURABILITY"),
            "CP": self.configuration.get_or_nil("MAX_CP"),
        };
        self.macro_string = GString::new();
        self.emit_state_updated();
    }

    fn set_result(&mut self, actions: Vec<Action>) {
        let base_progress: f32 = self.configuration.get_or_nil("PROGRESS_INCREASE").to();
        let base_quality: f32 = self.configuration.get_or_nil("QUALITY_INCREASE").to();

        let settings = self.get_settings();

        // set simulation state
        let state = from_action_sequence(&settings, &actions[0..actions.len() - 1])
            .as_in_progress()
            .unwrap();
        let last_action = actions.last().unwrap();

        let missing_progress = state
            .missing_progress
            .saturating_sub(last_action.progress_increase(&state.effects, Condition::Normal));
        let progress = settings.max_progress.sub(missing_progress);

        let missing_quality = state
            .missing_quality
            .saturating_sub(last_action.quality_increase(&state.effects, Condition::Normal));
        let quality = settings.max_quality.sub(missing_quality);

        let durability =
            state.durability - last_action.durability_cost(&state.effects, Condition::Normal);
        let cp = state.cp - last_action.cp_cost(&state.effects, Condition::Normal);

        self.simulation = dict! {
            "PROGRESS": f32::from(progress) * base_progress / 100.0,
            "QUALITY": f32::from(quality) * base_quality / 100.0,
            "DURABILITY": durability,
            "CP": cp,
        };

        // set macro string
        let mut lines: Vec<String> = Vec::new();
        for action in actions {
            lines.push(format!(
                "/ac \"{}\" <wait.{}>",
                action.display_name(),
                action.time_cost()
            ))
        }
        self.macro_string = lines.join("\n").into();

        self.emit_state_updated();
    }

    #[func]
    fn check_result(&mut self) {
        if self.solver_busy {
            match self.solver_result.clone().try_lock() {
                Ok(mut lock_guard) => {
                    match lock_guard.as_ref() {
                        Some(actions) => self.set_result(actions.clone()),
                        None => (),
                    };
                    *lock_guard = None;
                    self.solver_busy = false;
                    self.emit_state_updated();
                }
                Err(_) => (),
            }
        }
    }

    #[func]
    fn solve(&mut self) {
        if !self.solver_busy {
            self.solver_busy = true;
            self.emit_state_updated();
            let mutex = self.solver_result.clone();
            let settings = self.get_settings();
            dbg!("spawning solver thread");
            thread::spawn(move || {
                Self::do_solve(mutex, settings);
            });
        }
    }

    fn do_solve(mutex: Arc<Mutex<Option<Vec<Action>>>>, settings: Settings) {
        let mut lock_guard = mutex.lock().unwrap();
        let state = State::new(&settings);
        *lock_guard = MacroSolver::new(settings).solve(state);
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
