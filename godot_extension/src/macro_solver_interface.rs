use std::{
    sync::{Arc, Mutex},
    thread,
};

use game_data::{get_item_names, get_simulator_settings};
use godot::prelude::*;

use simulator::{Action, Condition, Settings, State};
use solvers::MacroSolver;

#[derive(GodotClass)]
#[class(base=Node)]
struct MacroSolverInterface {
    base: Base<Node>,

    #[export]
    solver_busy: bool,
    solver_result: Arc<Mutex<Option<Vec<Action>>>>,

    #[var]
    item_names: Array<GString>,

    #[export]
    setting_recipe: GString,
    #[export]
    setting_craftsmanship: i64,
    #[export]
    setting_control: i64,
    #[export]
    setting_max_cp: i64,
    #[export]
    setting_job_level: i64,
    #[export]
    setting_manipulation_unlocked: bool,

    #[export]
    simulation_progress: i64,
    #[export]
    simulation_max_progress: i64,
    #[export]
    simulation_base_progress: i64,
    #[export]
    simulation_quality: i64,
    #[export]
    simulation_max_quality: i64,
    #[export]
    simulation_base_quality: i64,
    #[export]
    simulation_durability: i64,
    #[export]
    simulation_max_durability: i64,
    #[export]
    simulation_cp: i64,
    #[export]
    simulation_max_cp: i64,

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
            macro_string: GString::new(),

            item_names: get_item_names().map(|s| s.to_godot()).collect(),

            setting_recipe: "".to_godot(),
            setting_craftsmanship: 0,
            setting_control: 0,
            setting_max_cp: 0,
            setting_job_level: 0,
            setting_manipulation_unlocked: false,

            simulation_progress: 0,
            simulation_max_progress: 0,
            simulation_base_progress: 0,
            simulation_quality: 0,
            simulation_max_quality: 0,
            simulation_base_quality: 0,
            simulation_durability: 0,
            simulation_max_durability: 0,
            simulation_cp: 0,
            simulation_max_cp: 0,
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
        get_simulator_settings(
            self.setting_recipe.clone().into(),
            self.setting_craftsmanship as u32,
            self.setting_control as u32,
            self.setting_max_cp as u32,
            self.setting_job_level as u32,
            self.setting_manipulation_unlocked,
        )
        .expect("Failed to get simulator settings")
    }

    #[func]
    fn reset_simulation(&mut self) {
        self.macro_string = GString::new();

        let settings = self.get_settings();
        self.simulation_progress = 0;
        self.simulation_max_progress = settings.max_progress as i64;
        self.simulation_base_progress = settings.base_progress as i64;
        self.simulation_quality = 0;
        self.simulation_max_quality = settings.max_quality as i64;
        self.simulation_base_quality = settings.base_quality as i64;
        self.simulation_durability = settings.max_durability as i64;
        self.simulation_max_durability = settings.max_durability as i64;
        self.simulation_cp = settings.max_cp as i64;
        self.simulation_max_cp = settings.max_cp as i64;

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

        self.simulation_progress = progress as i64;
        self.simulation_quality = quality as i64;
        self.simulation_durability = durability as i64;
        self.simulation_cp = cp as i64;

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
            if let Ok(mut lock_guard) = self.solver_result.clone().try_lock() {
                if let Some(actions) = lock_guard.as_ref() {
                    self.set_result(actions.clone());
                }
                *lock_guard = None;
                self.solver_busy = false;
                self.emit_state_updated();
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
    let mut state: State = State::new(settings);
    for action in actions {
        state = state
            .as_in_progress()
            .unwrap()
            .use_action(*action, Condition::Normal, settings);
    }
    state
}
