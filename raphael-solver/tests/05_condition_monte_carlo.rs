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
//! Every iteration logs its exact derived seed. Reproduce iteration `N` directly
//! by setting `RAPHAEL_MONTE_CARLO_SEED` to that logged decimal or `0x...` seed
//! and `RAPHAEL_MONTE_CARLO_ITERATIONS=1`.

use std::{collections::HashMap, time::Instant};

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
        quality: state.quality.min(SETTINGS.max_quality),
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

#[derive(Debug)]
struct StepLog {
    step: usize,
    condition: Condition,
    action: Action,
    solve_ms: u128,
    state_before: SimulationState,
    state_after: SimulationState,
}

fn print_steps(strategy: &str, steps: &[StepLog]) {
    println!("{strategy} actions:");
    for step in steps {
        println!(
            concat!(
                "  step={:02} condition={:?} action={:?} solve_ms={} ",
                "before={{cp:{}, durability:{}, progress:{}, quality:{}}} ",
                "after={{cp:{}, durability:{}, progress:{}, quality:{}}}"
            ),
            step.step,
            step.condition,
            step.action,
            step.solve_ms,
            step.state_before.cp,
            step.state_before.durability,
            step.state_before.progress,
            step.state_before.quality,
            step.state_after.cp,
            step.state_after.durability,
            step.state_after.progress,
            step.state_after.quality,
        );
    }
}

fn run_static(actions: &[Action], seed: u64) -> (RunResult, Vec<Action>, Vec<StepLog>) {
    let mut state = SimulationState::new(&SETTINGS);
    let mut conditions = ConditionRng::new(seed);
    let mut executed = Vec::new();
    let mut step_logs = Vec::new();
    for &action in actions {
        let state_before = state;
        let condition = conditions.current;
        let Ok(next_state) = state.use_action(action, condition, &SETTINGS) else {
            break;
        };
        state = next_state;
        executed.push(action);
        step_logs.push(StepLog {
            step: executed.len(),
            condition,
            action,
            solve_ms: 0,
            state_before,
            state_after: state,
        });
        if consumes_condition(action) {
            conditions.advance();
        }
        if state.is_final(&SETTINGS) {
            break;
        }
    }
    (
        finish_result(state, &conditions, &executed),
        executed,
        step_logs,
    )
}

fn run_dynamic(seed: u64) -> (RunResult, Vec<Action>, Vec<StepLog>) {
    let mut state = SimulationState::new(&SETTINGS);
    let mut conditions = ConditionRng::new(seed);
    let mut dynamic_solver = solver();
    let mut executed = Vec::new();
    let mut step_logs = Vec::new();
    while !state.is_final(&SETTINGS) && executed.len() < 100 {
        let state_before = state;
        let condition = conditions.current;
        println!(
            "  dynamic step={:02} condition={condition:?} solve=running state={{cp:{}, durability:{}, progress:{}, quality:{}}}",
            executed.len() + 1,
            state.cp,
            state.durability,
            state.progress,
            state.quality,
        );
        let solve_start = Instant::now();
        let solve_result = dynamic_solver.solve_from_state_with_condition(state, condition);
        let solve_ms = solve_start.elapsed().as_millis();
        let plan = match solve_result {
            Ok(plan) => plan,
            Err(error) => {
                println!(
                    "  dynamic step={:02} solve_ms={solve_ms} error={error:?}",
                    executed.len() + 1
                );
                break;
            }
        };
        let Some(&action) = plan.first() else {
            println!(
                "  dynamic step={:02} solve_ms={solve_ms} error=empty-plan",
                executed.len() + 1
            );
            break;
        };
        let Ok(next_state) = state.use_action(action, condition, &SETTINGS) else {
            println!(
                "  dynamic step={:02} solve_ms={solve_ms} action={action:?} error=invalid-action",
                executed.len() + 1
            );
            break;
        };
        state = next_state;
        executed.push(action);
        step_logs.push(StepLog {
            step: executed.len(),
            condition,
            action,
            solve_ms,
            state_before,
            state_after: state,
        });
        println!(
            "  dynamic step={:02} condition={condition:?} action={action:?} solve_ms={solve_ms} result={{cp:{}, durability:{}, progress:{}, quality:{}}}",
            executed.len(),
            state.cp,
            state.durability,
            state.progress,
            state.quality,
        );
        if consumes_condition(action) {
            conditions.advance();
        }
    }
    (
        finish_result(state, &conditions, &executed),
        executed,
        step_logs,
    )
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn parse_seed(value: &str) -> Option<u64> {
    value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .map_or_else(
            || value.parse().ok(),
            |hex| u64::from_str_radix(hex, 16).ok(),
        )
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .as_deref()
        .and_then(parse_seed)
        .unwrap_or(default)
}

#[test]
fn hexadecimal_and_decimal_seeds_are_reproducible() {
    assert_eq!(parse_seed("0x5eedc0de6178"), Some(DEFAULT_SEED));
    assert_eq!(parse_seed(&DEFAULT_SEED.to_string()), Some(DEFAULT_SEED));
}

#[test]
fn default_seed_condition_prefix_is_stable() {
    let mut conditions = ConditionRng::new(DEFAULT_SEED);
    let actual = (0..26)
        .map(|_| {
            let current = conditions.current;
            conditions.advance();
            current
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        [
            Condition::Normal,
            Condition::Normal,
            Condition::Good,
            Condition::Normal,
            Condition::Normal,
            Condition::Good,
            Condition::Normal,
            Condition::Normal,
            Condition::Normal,
            Condition::Normal,
            Condition::Normal,
            Condition::Normal,
            Condition::Normal,
            Condition::Normal,
            Condition::Normal,
            Condition::Excellent,
            Condition::Poor,
            Condition::Normal,
            Condition::Normal,
            Condition::Normal,
            Condition::Good,
            Condition::Normal,
            Condition::Normal,
            Condition::Good,
            Condition::Normal,
            Condition::Normal,
        ]
    );
}

#[test]
#[ignore = "long-running Monte Carlo experiment; run explicitly in release mode"]
fn compare_static_and_dynamic_over_random_conditions() {
    let iterations = env_usize("RAPHAEL_MONTE_CARLO_ITERATIONS", DEFAULT_ITERATIONS);
    let seed = env_u64("RAPHAEL_MONTE_CARLO_SEED", DEFAULT_SEED);
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
        let iteration_start = Instant::now();
        let run_seed = seed.wrapping_add(iteration as u64);
        println!("iteration={iteration} seed={run_seed:#x} start");
        let static_start = Instant::now();
        let static_actions = solver().solve().expect("static solve failed");
        let (static_run, static_executed, static_steps) = run_static(&static_actions, run_seed);
        let static_ms = static_start.elapsed().as_millis();
        println!(
            "iteration={iteration} seed={run_seed:#x} static={{quality:{}, success:{}, steps:{}, duration:{}, run_ms:{static_ms}}}",
            static_run.quality, static_run.success, static_run.steps, static_run.duration
        );
        print_steps("static", &static_steps);

        let dynamic_start = Instant::now();
        let (dynamic_run, dynamic_executed, dynamic_steps) = run_dynamic(run_seed);
        let dynamic_ms = dynamic_start.elapsed().as_millis();
        println!(
            "iteration={iteration} seed={run_seed:#x} dynamic={{quality:{}, success:{}, steps:{}, duration:{}, run_ms:{dynamic_ms}}}",
            dynamic_run.quality, dynamic_run.success, dynamic_run.steps, dynamic_run.duration
        );
        print_steps("dynamic", &dynamic_steps);
        println!(
            "iteration={iteration} seed={run_seed:#x} complete elapsed_ms={}",
            iteration_start.elapsed().as_millis()
        );
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
