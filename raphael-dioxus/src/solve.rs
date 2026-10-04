use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use dioxus::prelude::*;
use raphael_data::{Consumable, CrafterStats};
use raphael_sim::{Action, Settings};
use raphael_solver::SolverException;

use crate::{
    config::RecipeConfiguration,
    context::{AppContext, SolverConfig},
    platform,
    saved_rotations::Rotation,
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

#[derive(Debug, Clone)]
pub struct RunningSolveInfo {
    pub recipe_config: RecipeConfiguration,
    pub food: Option<Consumable>,
    pub potion: Option<Consumable>,
    pub stats: CrafterStats,

    pub solve_params: SolveParameters,
    pub start_time: web_time::Instant,
    pub solver_progress: usize, // Number of nodes visited
}

#[derive(Debug, Default)]
pub enum SolveStatus {
    #[default]
    Idle,
    /// Waiting for the thread pool to be initialized.
    Starting,
    Solving {
        info: Box<RunningSolveInfo>,
        interrupt: raphael_solver::AtomicFlag,
    },
    Failed {
        error: SolverException,
    },
}

#[derive(Debug, Clone)]
pub struct LastSolveInfo {
    pub solve_params: SolveParameters,
    pub duration: web_time::Duration,
    pub loaded_from_history: bool,
}

#[derive(Debug, Default)]
pub struct SolveState {
    pub status: SolveStatus,
    pub actions: Vec<Action>,
    pub last_solve_info: Option<LastSolveInfo>,
}

impl SolveState {
    pub fn is_busy(&self) -> bool {
        matches!(
            self.status,
            SolveStatus::Starting | SolveStatus::Solving { .. }
        )
    }

    pub fn running_solve_info(&self) -> Option<&RunningSolveInfo> {
        match &self.status {
            SolveStatus::Solving { info, .. } => Some(info),
            _ => None,
        }
    }

    pub fn interrupted(&self) -> bool {
        match &self.status {
            SolveStatus::Solving { interrupt, .. } => interrupt.is_set(),
            _ => false,
        }
    }

    pub fn interrupt(&self) {
        if let SolveStatus::Solving { interrupt, .. } = &self.status {
            interrupt.set();
        }
    }

    pub fn resolve_error(&mut self) {
        if let SolveStatus::Failed { .. } = &self.status {
            self.status = SolveStatus::Idle;
        }
    }

    /// Whether the configuration has changed since the last solve.
    pub fn config_changed(&self, app_context: &AppContext) -> bool {
        !self.is_busy()
            && self
                .last_solve_info
                .as_ref()
                .is_some_and(|info| info.solve_params != SolveParameters::from(app_context))
    }
}

/// Whether the active crafter stats (including consumables) meet the recipe's requirements.
pub fn meets_stat_requirements(app_context: &AppContext) -> bool {
    let recipe = app_context.recipe_config.recipe();
    let stats = app_context.active_stats();
    let consumables = [app_context.selected_food, app_context.selected_potion];
    let craftsmanship =
        stats.craftsmanship + raphael_data::craftsmanship_bonus(stats.craftsmanship, &consumables);
    let control = stats.control + raphael_data::control_bonus(stats.control, &consumables);
    craftsmanship >= recipe.req_craftsmanship && control >= recipe.req_control
}

/// Starts solving the current configuration. Progress and results are written to `solve_state`.
pub async fn solve(mut app_context: Signal<AppContext>, mut solve_state: Signal<SolveState>) {
    if solve_state.peek().is_busy() {
        return;
    }
    if !thread_pool::is_initialized() {
        solve_state.write().status = SolveStatus::Starting;
        let num_threads = app_context.peek().app_config.num_threads;
        thread_pool::ensure_initialized(num_threads).await;
    }

    let (mut game_settings, initial_quality, solver_config) = {
        let ctx = app_context.peek();
        (
            ctx.game_settings(),
            ctx.initial_quality(),
            ctx.solver_config,
        )
    };

    let saved_actions = {
        let ctx = app_context.peek();
        ctx.saved_rotations_config
            .load_from_saved_rotations
            .then(|| {
                ctx.saved_rotations_data.find_solved_rotation(
                    &game_settings,
                    initial_quality,
                    &solver_config,
                )
            })
            .flatten()
    };
    if let Some(actions) = saved_actions {
        let mut state = solve_state.write();
        state.actions = actions;
        state.last_solve_info = Some(LastSolveInfo {
            solve_params: SolveParameters::from(&*app_context.peek()),
            duration: web_time::Duration::default(),
            loaded_from_history: true,
        });
        state.status = SolveStatus::Idle;
        return;
    }

    let target_quality = solver_config
        .quality_target
        .get_target(game_settings.max_quality);
    game_settings.max_quality = target_quality.saturating_sub(initial_quality);
    let solver_settings = raphael_solver::SolverSettings {
        simulator_settings: game_settings,
        allow_non_max_quality_solutions: !solver_config.must_reach_target_quality,
    };

    let info = {
        let ctx = app_context.peek();
        RunningSolveInfo {
            recipe_config: ctx.recipe_config,
            food: ctx.selected_food,
            potion: ctx.selected_potion,
            stats: *ctx.active_stats(),
            solve_params: SolveParameters::from(&*ctx),
            start_time: web_time::Instant::now(),
            solver_progress: 0,
        }
    };
    let interrupt = raphael_solver::AtomicFlag::new();
    {
        let mut state = solve_state.write();
        state.actions = Vec::new();
        state.status = SolveStatus::Solving {
            info: Box::new(info),
            interrupt: interrupt.clone(),
        };
    }

    let events = Arc::new(Mutex::new(VecDeque::new()));
    spawn_solver(solver_settings, events.clone(), interrupt);

    // The solver runs on the thread pool. Poll its events from the UI thread.
    loop {
        platform::sleep(web_time::Duration::from_millis(50)).await;
        #[cfg(target_arch = "wasm32")]
        if crate::OOM_PANIC_OCCURED.load(std::sync::atomic::Ordering::Relaxed) {
            solve_state.write().status = SolveStatus::Failed {
                error: SolverException::InternalError(
                    "Out of memory. Reload the page to continue.".to_string(),
                ),
            };
            return;
        }
        let drained: Vec<SolverEvent> = events.lock().unwrap().drain(..).collect();
        for event in drained {
            let mut state = solve_state.write();
            let SolveState {
                status, actions, ..
            } = &mut *state;
            let SolveStatus::Solving { info, .. } = status else {
                return;
            };
            match event {
                SolverEvent::NodesVisited(count) => info.solver_progress = count,
                SolverEvent::Actions(new_actions) => *actions = new_actions,
                SolverEvent::Finished(exception) => {
                    let info = (**info).clone();
                    state.last_solve_info = Some(LastSolveInfo {
                        solve_params: info.solve_params.clone(),
                        duration: info.start_time.elapsed(),
                        loaded_from_history: false,
                    });
                    state.status = match exception {
                        Some(SolverException::Interrupted) => SolveStatus::Idle,
                        Some(error) => SolveStatus::Failed { error },
                        None => {
                            let mut ctx = app_context.write();
                            let rotation = Rotation::new(&info, state.actions.clone(), ctx.locale);
                            let AppContext {
                                saved_rotations_data,
                                saved_rotations_config,
                                ..
                            } = &mut *ctx;
                            saved_rotations_data
                                .add_solved_rotation(rotation, saved_rotations_config);
                            SolveStatus::Idle
                        }
                    };
                    return;
                }
            }
        }
    }
}

fn spawn_solver(
    solver_settings: raphael_solver::SolverSettings,
    events: Arc<Mutex<VecDeque<SolverEvent>>>,
    interrupt: raphael_solver::AtomicFlag,
) {
    let solution_events = events.clone();
    let solution_callback = move |actions: &[Action]| {
        let event = SolverEvent::Actions(actions.to_vec());
        solution_events.lock().unwrap().push_back(event);
    };
    let progress_events = events.clone();
    let progress_callback = move |progress: usize| {
        let mut events = progress_events.lock().unwrap();
        // Only the latest progress value is relevant.
        if let Some(SolverEvent::NodesVisited(count)) = events.back_mut() {
            *count = progress;
        } else {
            events.push_back(SolverEvent::NodesVisited(progress));
        }
    };
    rayon::spawn(move || {
        log::debug!("Spawning solver: {solver_settings:?}");
        let mut macro_solver = raphael_solver::MacroSolver::new(
            solver_settings,
            Box::new(solution_callback),
            Box::new(progress_callback),
            interrupt,
        );
        let result = macro_solver.solve();
        let mut events = events.lock().unwrap();
        match result {
            Ok(actions) => {
                events.push_back(SolverEvent::Actions(actions));
                events.push_back(SolverEvent::Finished(None));
            }
            Err(exception) => events.push_back(SolverEvent::Finished(Some(exception))),
        }
    });
}
