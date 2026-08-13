//! Monte Carlo comparison of a fixed macro and the condition-aware online solver.
//!
//! This experiment is ignored by default because the dynamic strategy invokes the
//! exact solver after every action. Run a small sample with:
//!
//! ```text
//! RAPHAEL_MONTE_CARLO_ITERATIONS=10 cargo test --release -p raphael-solver \
//!   --test 05_condition_monte_carlo -- --ignored --nocapture
//! ```
//!
//! Increase `RAPHAEL_MONTE_CARLO_ITERATIONS` for long-running experiments.

use std::collections::HashMap;

use rand::{Rng, SeedableRng, rngs::StdRng};
use raphael_sim::{Action, ActionMask, Condition, Settings, SimulationState};
use raphael_solver::{AtomicFlag, MacroSolver, SolverSettings};

const DEFAULT_ITERATIONS: usize = 10;
const DEFAULT_SEED: u64 = 0x5eed_c0de_6178;

// Claro Walnut Sandals of Gathering, recipe 6178. The crafter configuration and
// target were taken from raphael_solve_events.jsonl:
// craftsmanship=4292, control=3931, CP=508, level=100, target quality=12000.
// get_game_settings resolves those inputs to the base progress/quality below.
const SETTINGS: Settings = Settings {
    max_cp: 508,
    max_durability: 80,
    max_progress: 6600,
    max_quality: 12_000,
    base_progress: 229,
    base_quality: 222,
    job_level: 100,
    allowed_actions: ActionMask::all()
        .remove(Action::Manipulation)
        .remove(Action::TrainedEye)
        .remove(Action::HeartAndSoul)
        .remove(Action::QuickInnovation),
    adversarial: false,
    backload_progress: false,
    stellar_steady_hand_charges: 0,
};

// Standard non-expert recipes roll from Normal to Excellent 4% of the time and
// to Good 25% of the time. Excellent -> Poor and Good/Poor -> Normal are forced.
// Source: https://ffxiv.consolegameswiki.com/wiki/Crafting#Crafting_Basics
const EXCELLENT_CHANCE: f64 = 0.04;
const GOOD_CHANCE: f64 = 0.25;

#[derive(Clone, Copy, Debug)]
struct RunResult {
    success: bool,
    quality: u16,
    steps: usize,
    duration: u16,
    good: usize,
    excellent: usize,
    poor: usize,
}

#[derive(Debug, Default)]
struct Totals {
    successes: usize,
    hq: usize,
    quality: u64,
    steps: u64,
    duration: u64,
    good: u64,
    excellent: u64,
    poor: u64,
    action_counts: HashMap<Action, usize>,
}

impl Totals {
    fn add(&mut self, run: RunResult, actions: &[Action]) {
        self.successes += usize::from(run.success);
        self.hq += usize::from(run.quality >= SETTINGS.max_quality);
        self.quality += u64::from(run.quality);
        self.steps += run.steps as u64;
        self.duration += u64::from(run.duration);
        self.good += run.good as u64;
        self.excellent += run.excellent as u64;
        self.poor += run.poor as u64;
        for &action in actions {
            *self.action_counts.entry(action).or_default() += 1;
        }
    }

    fn print(&self, name: &str, iterations: usize) {
        println!(
            "{name:8} success={:6.2}% max-quality={:6.2}% avg-quality={:8.1} avg-steps={:5.2} avg-duration={:5.2} avg-conditions(G/E/P)={:.2}/{:.2}/{:.2}",
            self.successes as f64 * 100.0 / iterations as f64,
            self.hq as f64 * 100.0 / iterations as f64,
            self.quality as f64 / iterations as f64,
            self.steps as f64 / iterations as f64,
            self.duration as f64 / iterations as f64,
            self.good as f64 / iterations as f64,
            self.excellent as f64 / iterations as f64,
            self.poor as f64 / iterations as f64,
        );
    }
}

#[derive(Debug)]
struct ConditionRng {
    rng: StdRng,
    current: Condition,
    good: usize,
    excellent: usize,
    poor: usize,
}

impl ConditionRng {
    fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            current: Condition::Normal,
            good: 0,
            excellent: 0,
            poor: 0,
        }
    }

    fn advance(&mut self) {
        self.current = match self.current {
            Condition::Excellent => Condition::Poor,
            Condition::Good | Condition::Poor => Condition::Normal,
            Condition::Normal => {
                let roll = self.rng.random::<f64>();
                if roll < EXCELLENT_CHANCE {
                    Condition::Excellent
                } else if roll < EXCELLENT_CHANCE + GOOD_CHANCE {
                    Condition::Good
                } else {
                    Condition::Normal
                }
            }
        };
        match self.current {
            Condition::Good => self.good += 1,
            Condition::Excellent => self.excellent += 1,
            Condition::Poor => self.poor += 1,
            Condition::Normal => {}
        }
    }
}

fn solver() -> MacroSolver<'static> {
    MacroSolver::new(
        SolverSettings {
            simulator_settings: SETTINGS,
            allow_non_max_quality_solutions: true,
        },
        Box::new(|_| {}),
        Box::new(|_| {}),
        AtomicFlag::new(),
    )
}

fn consumes_condition(action: Action) -> bool {
    !matches!(action, Action::HeartAndSoul | Action::QuickInnovation)
}

fn finish_result(
    state: SimulationState,
    conditions: &ConditionRng,
    actions: &[Action],
) -> RunResult {
    RunResult {
        success: state.progress >= SETTINGS.max_progress,
        quality: state.quality,
        steps: actions.len(),
        duration: actions
            .iter()
            .map(|action| u16::from(action.time_cost()))
            .sum(),
        good: conditions.good,
        excellent: conditions.excellent,
        poor: conditions.poor,
    }
}

fn run_static(actions: &[Action], seed: u64) -> (RunResult, Vec<Action>) {
    let mut state = SimulationState::new(&SETTINGS);
    let mut conditions = ConditionRng::new(seed);
    let mut executed = Vec::new();
    for &action in actions {
        let Ok(next_state) = state.use_action(action, conditions.current, &SETTINGS) else {
            break;
        };
        state = next_state;
        executed.push(action);
        if consumes_condition(action) {
            conditions.advance();
        }
        if state.is_final(&SETTINGS) {
            break;
        }
    }
    (finish_result(state, &conditions, &executed), executed)
}

fn run_dynamic(seed: u64) -> (RunResult, Vec<Action>) {
    let mut state = SimulationState::new(&SETTINGS);
    let mut conditions = ConditionRng::new(seed);
    let mut executed = Vec::new();
    while !state.is_final(&SETTINGS) && executed.len() < 100 {
        let Ok(plan) = solver().solve_from_state_with_condition(state, conditions.current) else {
            break;
        };
        let Some(&action) = plan.first() else {
            break;
        };
        let Ok(next_state) = state.use_action(action, conditions.current, &SETTINGS) else {
            break;
        };
        state = next_state;
        executed.push(action);
        if consumes_condition(action) {
            conditions.advance();
        }
    }
    (finish_result(state, &conditions, &executed), executed)
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

#[test]
#[ignore = "long-running Monte Carlo experiment; run explicitly in release mode"]
fn compare_static_and_dynamic_over_random_conditions() {
    let iterations = env_usize("RAPHAEL_MONTE_CARLO_ITERATIONS", DEFAULT_ITERATIONS);
    let seed = env_u64("RAPHAEL_MONTE_CARLO_SEED", DEFAULT_SEED);
    let static_actions = solver().solve().expect("static solve failed");
    let mut static_totals = Totals::default();
    let mut dynamic_totals = Totals::default();
    let mut dynamic_wins = 0;
    let mut ties = 0;
    let mut static_wins = 0;

    println!(
        "recipe=6178 iterations={iterations} seed={seed:#x} stats=4292/3931/508 condition probabilities: Good={:.0}% Excellent={:.0}%",
        GOOD_CHANCE * 100.0,
        EXCELLENT_CHANCE * 100.0,
    );

    for iteration in 0..iterations {
        let run_seed = seed.wrapping_add(iteration as u64);
        let (static_run, static_executed) = run_static(&static_actions, run_seed);
        let (dynamic_run, dynamic_executed) = run_dynamic(run_seed);
        match dynamic_run.quality.cmp(&static_run.quality) {
            std::cmp::Ordering::Greater => dynamic_wins += 1,
            std::cmp::Ordering::Equal => ties += 1,
            std::cmp::Ordering::Less => static_wins += 1,
        }
        static_totals.add(static_run, &static_executed);
        dynamic_totals.add(dynamic_run, &dynamic_executed);
        if (iteration + 1) % 10 == 0 || iteration + 1 == iterations {
            println!("completed {}/{}", iteration + 1, iterations);
        }
    }

    static_totals.print("static", iterations);
    dynamic_totals.print("dynamic", iterations);
    println!(
        "paired quality outcomes: dynamic wins={dynamic_wins}, ties={ties}, static wins={static_wins}"
    );
    println!(
        "delta    max-quality={:+.2}pp avg-quality={:+.1} avg-steps={:+.2}",
        (dynamic_totals.hq as f64 - static_totals.hq as f64) * 100.0 / iterations as f64,
        (dynamic_totals.quality as f64 - static_totals.quality as f64) / iterations as f64,
        (dynamic_totals.steps as f64 - static_totals.steps as f64) / iterations as f64,
    );

    assert_eq!(static_totals.successes, iterations);
    assert_eq!(dynamic_totals.successes, iterations);
}
