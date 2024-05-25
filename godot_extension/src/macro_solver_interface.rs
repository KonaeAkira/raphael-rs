use std::{
    sync::{Arc, Mutex},
    thread,
};

use game_data::{get_craftable_item_names, get_ingredients, ITEMS, ITEM_IDS, LEVELS, RECIPES};
use godot::prelude::*;

use simulator::{Action, ActionMask, Condition, Settings, State};
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

    #[var]
    ingredient_names: Array<GString>,
    #[var]
    ingredient_amounts: Array<i64>,
    #[var]
    ingredient_has_hq: Array<bool>,
    #[var]
    setting_hq_ingredient_amount: Array<i64>,

    #[export]
    simulation_progress: i64,
    #[export]
    simulation_max_progress: i64,
    #[export]
    simulation_base_progress: i64,
    #[export]
    simulation_initial_quality: i64,
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

            item_names: get_craftable_item_names().map(|s| s.to_godot()).collect(),

            setting_recipe: "".to_godot(),
            setting_craftsmanship: 0,
            setting_control: 0,
            setting_max_cp: 0,
            setting_job_level: 0,
            setting_manipulation_unlocked: false,

            #[rustfmt::skip]
            ingredient_names: array!["".into(), "".into(), "".into(), "".into(), "".into(), "".into()],
            ingredient_amounts: array![0, 0, 0, 0, 0, 0],
            ingredient_has_hq: array![true, true, true, true, true, true],
            setting_hq_ingredient_amount: array![0, 0, 0, 0, 0, 0],

            simulation_progress: 0,
            simulation_max_progress: 0,
            simulation_base_progress: 0,
            simulation_initial_quality: 0,
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
        Settings {
            max_cp: self.simulation_max_cp as i16,
            max_durability: self.simulation_max_durability as i16,
            max_progress: self.simulation_max_progress as u32,
            max_quality: (self.simulation_max_quality - self.simulation_initial_quality) as u32,
            base_progress: self.simulation_base_progress as u32,
            base_quality: self.simulation_base_quality as u32,
            job_level: self.setting_job_level as u8,
            allowed_actions: ActionMask::from_level(
                self.setting_job_level as u32,
                self.setting_manipulation_unlocked,
            ),
        }
    }

    #[func]
    fn reset_simulation_parameters(&mut self) {
        let item_id = ITEM_IDS
            .get(&self.setting_recipe.to_string())
            .expect("No such item name");
        let recipe = RECIPES.get(item_id).expect("No recipe for item");

        let mut base_progress: f64 =
            self.setting_craftsmanship as f64 * 10.0 / recipe.progress_div as f64 + 2.0;
        let mut base_quality: f64 =
            self.setting_control as f64 * 10.0 / recipe.quality_div as f64 + 35.0;
        if LEVELS[self.setting_job_level as usize - 1] <= recipe.recipe_level {
            base_progress = base_progress * recipe.progress_mod as f64 / 100.0;
            base_quality = base_quality * recipe.quality_mod as f64 / 100.0;
        }

        let ingredients = get_ingredients(self.setting_recipe.clone().into());
        self.ingredient_names = ingredients
            .iter()
            .map(|i| ITEMS.get(&i.item_id).unwrap().name.to_godot())
            .collect();
        self.ingredient_amounts = ingredients.iter().map(|i| i.amount as i64).collect();
        self.ingredient_has_hq = ingredients
            .iter()
            .map(|i| ITEMS.get(&i.item_id).unwrap().can_be_hq)
            .collect();

        // calculate initial quality
        let mut total_ilvl: i64 = 0;
        let mut selected_ilvl: i64 = 0;
        for (ingredient, selected_amount) in ingredients
            .iter()
            .zip(self.setting_hq_ingredient_amount.iter_shared())
        {
            let item = ITEMS.get(&ingredient.item_id).unwrap();
            if item.can_be_hq {
                total_ilvl += item.item_level as i64 * ingredient.amount as i64;
                selected_ilvl += item.item_level as i64 * selected_amount;
            }
        }
        self.simulation_initial_quality =
            self.simulation_max_quality * recipe.material_quality_factor as i64 * selected_ilvl
                / total_ilvl
                / 100;

        self.simulation_progress = 0;
        self.simulation_max_progress = recipe.progress as i64;
        self.simulation_base_progress = base_progress.floor() as i64;
        self.simulation_quality = self.simulation_initial_quality;
        self.simulation_max_quality = recipe.quality as i64;
        self.simulation_base_quality = base_quality.floor() as i64;
        self.simulation_durability = recipe.durability as i64;
        self.simulation_max_durability = recipe.durability as i64;
        self.simulation_cp = self.setting_max_cp;
        self.simulation_max_cp = self.setting_max_cp;

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
        self.simulation_quality = self.simulation_initial_quality + quality as i64;
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
