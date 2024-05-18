use std::{
    sync::{Arc, Mutex},
    thread,
};

use godot::prelude::*;

use crate::{
    game::{
        units::{Durability, Progress, Quality, CP},
        Action, ActionMask, Condition, Settings, State,
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
    setting_max_progress: f64,
    #[export]
    setting_max_quality: f64,
    #[export]
    setting_max_durability: i64,
    #[export]
    setting_max_cp: i64,
    #[export]
    setting_base_progress: f64,
    #[export]
    setting_base_quality: f64,
    #[export]
    setting_job_level: i64,
    #[export]
    setting_manipulation_unlocked: bool,

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
            simulation: dict! {"PROGRESS":0.0,"QUALITY":0.0,"DURABILITY":0.0,"CP":0.0,},
            macro_string: GString::new(),

            setting_max_progress: 0.0,
            setting_max_quality: 0.0,
            setting_max_durability: 0,
            setting_max_cp: 0,
            setting_base_progress: 0.0,
            setting_base_quality: 0.0,
            setting_job_level: 0,
            setting_manipulation_unlocked: false,
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
        Settings {
            max_cp: self.setting_max_cp as CP,
            max_durability: self.setting_max_durability as Durability,
            max_progress: self.setting_max_progress as Progress,
            max_quality: self.setting_max_quality as Quality,
            base_progress: self.setting_base_progress as Progress,
            base_quality: self.setting_base_quality as Quality,
            job_level: self.setting_job_level as u8,
            allowed_actions: ActionMask::from_level(
                self.setting_job_level as u32,
                self.setting_manipulation_unlocked,
            ),
        }
    }

    #[func]
    fn reset_result(&mut self) {
        self.simulation = dict! {
            "PROGRESS": 0.0,
            "QUALITY": 0.0,
            "DURABILITY": self.setting_max_durability,
            "CP": self.setting_max_cp,
        };
        self.macro_string = GString::new();
        self.emit_state_updated();
    }

    fn set_result(&mut self, actions: Vec<Action>) {
        let settings = self.get_settings();

        // set simulation state
        let state = from_action_sequence(&settings, &actions[0..actions.len() - 1])
            .as_in_progress()
            .unwrap();
        let last_action = actions.last().unwrap();

        let missing_progress =
            state
                .missing_progress
                .saturating_sub(last_action.progress_increase(
                    &settings,
                    &state.effects,
                    Condition::Normal,
                ));
        let progress = settings.max_progress - missing_progress;

        let missing_quality = state
            .missing_quality
            .saturating_sub(last_action.quality_increase(
                &settings,
                &state.effects,
                Condition::Normal,
            ));
        let quality = settings.max_quality - missing_quality;

        let durability =
            state.durability - last_action.durability_cost(&state.effects, Condition::Normal);
        let cp = state.cp - last_action.cp_cost(&state.effects, Condition::Normal);

        self.simulation = dict! {
            "PROGRESS": progress,
            "QUALITY": quality,
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
