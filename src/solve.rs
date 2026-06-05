use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use raphael_data::{Consumable, CrafterStats};
use raphael_sim::{Action, Settings};
use raphael_solver::SolverException;

use crate::{
    config::RecipeConfiguration,
    context::{AppContext, SolverConfig},
    elements::panels::Rotation,
    thread_pool,
};

#[derive(Debug)]
enum SolverEvent {
    NodesVisited(usize),
    Actions(Vec<Action>),
    Finished(Option<SolverException>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolveParameters {
    pub settings: Settings,
    pub initial_quality: u16,
    pub solver_config: SolverConfig,
}

impl From<&AppContext> for SolveParameters {
    fn from(app_context: &AppContext) -> Self {
        Self {
            settings: app_context.game_settings(),
            initial_quality: app_context.initial_quality(),
            solver_config: app_context.solver_config,
        }
    }
}

#[derive(Debug)]
pub struct RunningSolveInfo {
    pub recipe_config: RecipeConfiguration,
    pub food: Option<Consumable>,
    pub potion: Option<Consumable>,
    pub stats: CrafterStats,

    pub solve_params: SolveParameters,
    pub start_time: web_time::Instant,
    pub solver_progress: usize, // Number of nodes visited
}

// Lint is only triggered for the web version. Additionally, there (currently)
// is only one `SolveStatus` in use at a time so the difference is not important
#[cfg_attr(target_arch = "wasm32", expect(clippy::large_enum_variant))]
#[derive(Debug, Default)]
pub enum SolveStatus {
    #[default]
    Idle,
    Pending,
    Solving {
        info: RunningSolveInfo,
    },
    Failed {
        error: SolverException,
    },
}

#[derive(Debug)]
pub struct LastSolveInfo {
    pub solve_params: SolveParameters,
    pub duration: web_time::Duration,
    pub loaded_from_history: bool,
}

#[derive(Debug, Default)]
pub struct SolveState {
    status: SolveStatus,
    actions: Vec<Action>,

    last_solve_info: Option<LastSolveInfo>,

    solver_events: Arc<Mutex<VecDeque<SolverEvent>>>,
    solver_interrupt: raphael_solver::AtomicFlag,
}

impl SolveState {
    pub fn reset_actions(&mut self) {
        self.actions.clear();
    }

    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    pub fn actions_mut(&mut self) -> &mut Vec<Action> {
        &mut self.actions
    }

    pub fn running_solve_info(&self) -> Option<&RunningSolveInfo> {
        match &self.status {
            SolveStatus::Solving { info } => Some(info),
            _ => None,
        }
    }

    pub fn pending(&self) -> bool {
        matches!(self.status, SolveStatus::Pending)
    }

    pub fn solving(&self) -> bool {
        matches!(self.status, SolveStatus::Solving { .. })
    }

    pub fn interrupted(&self) -> bool {
        self.solver_interrupt.is_set()
    }

    pub fn interrupt(&mut self) {
        self.solver_interrupt.set();
    }

    pub fn solver_error(&self) -> Option<&SolverException> {
        match &self.status {
            SolveStatus::Failed { error } => Some(error),
            _ => None,
        }
    }

    pub fn resolve_error(&mut self) {
        if let SolveStatus::Failed { .. } = &self.status {
            self.status = SolveStatus::Idle
        }
    }

    pub fn last_solve_info(&self) -> Option<&LastSolveInfo> {
        self.last_solve_info.as_ref()
    }

    pub fn process_solver_events(&mut self, app_context: &mut AppContext) {
        let mut solver_events = self.solver_events.lock().unwrap();
        if let SolveStatus::Solving { info } = &mut self.status {
            while let Some(event) = solver_events.pop_front() {
                match event {
                    SolverEvent::NodesVisited(count) => info.solver_progress = count,
                    SolverEvent::Actions(actions) => self.actions = actions,
                    SolverEvent::Finished(exception) => {
                        self.solver_interrupt.clear();

                        let last_solve_info = LastSolveInfo {
                            solve_params: info.solve_params.clone(),
                            duration: info.start_time.elapsed(),
                            loaded_from_history: false,
                        };

                        match exception {
                            Some(exception) => match exception {
                                SolverException::Interrupted => {
                                    self.status = SolveStatus::Idle;
                                    break;
                                }
                                _ => self.status = SolveStatus::Failed { error: exception },
                            },
                            None => {
                                let new_rotation =
                                    Rotation::new(info, self.actions.clone(), app_context.locale);
                                app_context.saved_rotations_data.add_solved_rotation(
                                    new_rotation,
                                    &app_context.saved_rotations_config,
                                );
                                self.status = SolveStatus::Idle;
                            }
                        }
                        self.last_solve_info = Some(last_solve_info);
                        break;
                    }
                }
            }
        }
    }

    pub fn solve(&mut self, app_context: &AppContext) {
        if !thread_pool::is_initialized() {
            thread_pool::attempt_initialization(app_context.app_config.num_threads);

            if !thread_pool::is_initialized() {
                self.status = SolveStatus::Pending;
                return;
            }
        }

        self.solver_interrupt.clear();

        let mut game_settings = app_context.game_settings();
        let initial_quality = app_context.initial_quality();
        let solver_config = app_context.solver_config;

        if app_context.saved_rotations_config.load_from_saved_rotations
            && let Some(actions) = app_context.saved_rotations_data.find_solved_rotation(
                &game_settings,
                initial_quality,
                &solver_config,
            )
        {
            self.actions = actions;
            self.last_solve_info = Some(LastSolveInfo {
                solve_params: SolveParameters::from(app_context),
                duration: web_time::Duration::default(),
                loaded_from_history: true,
            });
            self.status = SolveStatus::Idle;
        } else {
            let target_quality = app_context
                .solver_config
                .quality_target
                .get_target(game_settings.max_quality);
            game_settings.max_quality = target_quality.saturating_sub(initial_quality);
            self.actions = Vec::new();
            let solver_progress = 0;
            let start_time = web_time::Instant::now();
            let solver_settings = raphael_solver::SolverSettings {
                simulator_settings: game_settings,
                allow_non_max_quality_solutions: !solver_config.must_reach_target_quality,
            };
            self.status = SolveStatus::Solving {
                info: RunningSolveInfo {
                    recipe_config: app_context.recipe_config,
                    food: app_context.selected_food,
                    potion: app_context.selected_potion,
                    stats: *app_context.active_stats(),
                    solve_params: SolveParameters::from(app_context),
                    start_time,
                    solver_progress,
                },
            };
            self.spawn_solver(solver_settings);
        }
    }

    fn spawn_solver(&mut self, solver_settings: raphael_solver::SolverSettings) {
        let events = self.solver_events.clone();
        let solution_callback = move |actions: &[raphael_sim::Action]| {
            let event = SolverEvent::Actions(actions.to_vec());
            events.lock().unwrap().push_back(event);
        };
        let events = self.solver_events.clone();
        let progress_callback = move |progress: usize| {
            let event = SolverEvent::NodesVisited(progress);
            events.lock().unwrap().push_back(event);
        };
        let solver_events = self.solver_events.clone();
        let solver_interrupt = self.solver_interrupt.clone();
        rayon::spawn(move || {
            log::debug!("Spawning solver: {solver_settings:?}");
            let mut macro_solver = raphael_solver::MacroSolver::new(
                solver_settings,
                Box::new(solution_callback),
                Box::new(progress_callback),
                solver_interrupt,
            );
            match macro_solver.solve() {
                Ok(actions) => {
                    let mut solver_events = solver_events.lock().unwrap();
                    solver_events.push_back(SolverEvent::Actions(actions));
                    solver_events.push_back(SolverEvent::Finished(None));
                }
                Err(exception) => solver_events
                    .lock()
                    .unwrap()
                    .push_back(SolverEvent::Finished(Some(exception))),
            }
        });
    }
}
