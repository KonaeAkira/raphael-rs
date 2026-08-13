use raphael_sim::{Action, ActionMask, Condition, Settings, SimulationState};
use raphael_solver::{AtomicFlag, MacroSolver, SolverSettings};

const SETTINGS: Settings = Settings {
    // Competent Craftsman's Tisane (recipe 5667) with the live test character's stats.
    max_cp: 455,
    max_durability: 80,
    max_progress: 5100,
    max_quality: 9800,
    base_progress: 198,
    base_quality: 263,
    job_level: 90,
    allowed_actions: ActionMask::all()
        .remove(Action::Manipulation)
        .remove(Action::TrainedEye)
        .remove(Action::HeartAndSoul)
        .remove(Action::QuickInnovation),
    adversarial: false,
    backload_progress: false,
    stellar_steady_hand_charges: 0,
};

#[derive(Debug)]
struct RunResult {
    quality: u16,
    actions: Vec<Action>,
    decisions: Vec<(usize, Condition, Action)>,
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

fn condition_at(conditions: &[Condition], step: usize) -> Condition {
    conditions
        .get(step - 1)
        .copied()
        .unwrap_or(Condition::Normal)
}

fn run_static(conditions: &[Condition]) -> RunResult {
    let actions = solver().solve().unwrap();
    let mut state = SimulationState::new(&SETTINGS);
    let mut decisions = Vec::new();
    for (index, action) in actions.iter().copied().enumerate() {
        let step = index + 1;
        let condition = condition_at(conditions, step);
        decisions.push((step, condition, action));
        state = state.use_action(action, condition, &SETTINGS).unwrap();
    }
    assert!(state.progress >= SETTINGS.max_progress);
    RunResult {
        quality: state.quality,
        actions,
        decisions,
    }
}

fn run_dynamic(conditions: &[Condition]) -> RunResult {
    let mut state = SimulationState::new(&SETTINGS);
    let mut actions = Vec::new();
    let mut decisions = Vec::new();
    let mut step = 1;

    while !state.is_final(&SETTINGS) {
        let condition = condition_at(conditions, step);
        let action = solver()
            .solve_from_state_with_condition(state, condition)
            .unwrap()[0];
        decisions.push((step, condition, action));
        actions.push(action);
        state = state.use_action(action, condition, &SETTINGS).unwrap();
        step += 1;
    }

    assert!(state.progress >= SETTINGS.max_progress);
    RunResult {
        quality: state.quality,
        actions,
        decisions,
    }
}

fn print_comparison(name: &str, static_run: &RunResult, dynamic_run: &RunResult) {
    println!("\n{name}");
    println!("strategy | quality | steps");
    println!(
        "static   | {:7} | {}",
        static_run.quality,
        static_run.actions.len()
    );
    println!(
        "dynamic  | {:7} | {}",
        dynamic_run.quality,
        dynamic_run.actions.len()
    );
    println!(
        "delta    | {:+7} | {:+}",
        i32::from(dynamic_run.quality) - i32::from(static_run.quality),
        dynamic_run.actions.len() as isize - static_run.actions.len() as isize,
    );
    println!("non-Normal decisions:");
    let max_steps = static_run.decisions.len().max(dynamic_run.decisions.len());
    for step in 1..=max_steps {
        let static_decision = static_run.decisions.get(step - 1);
        let dynamic_decision = dynamic_run.decisions.get(step - 1);
        let condition = static_decision
            .map(|entry| entry.1)
            .or_else(|| dynamic_decision.map(|entry| entry.1))
            .unwrap_or(Condition::Normal);
        if condition != Condition::Normal {
            println!(
                "step {step:2} {condition:?}: static={:?}, dynamic={:?}",
                static_decision.map(|entry| entry.2),
                dynamic_decision.map(|entry| entry.2),
            );
        }
    }
}

fn valid_timeline(non_normal: &[(usize, Condition)]) -> Vec<Condition> {
    let mut result = vec![Condition::Normal; 32];
    for &(step, condition) in non_normal {
        result[step - 1] = condition;
    }
    // Normal crafting's forced transitions.
    for index in 0..result.len() - 1 {
        match result[index] {
            Condition::Good | Condition::Poor => result[index + 1] = Condition::Normal,
            Condition::Excellent => result[index + 1] = Condition::Poor,
            Condition::Normal => {}
        }
    }
    result
}

#[test]
fn static_rotation_does_not_replan_for_good_conditions() {
    let all_normal = run_static(&valid_timeline(&[]));
    let with_good = run_static(&valid_timeline(&[
        (3, Condition::Good),
        (7, Condition::Good),
        (12, Condition::Good),
        (17, Condition::Good),
    ]));

    // A static macro receives the condition multipliers, but its decisions cannot
    // change. In particular it cannot insert Precise Touch or Tricks of the Trade.
    assert_eq!(all_normal.actions, with_good.actions);
    assert_eq!(all_normal.quality, 5839);
    assert_eq!(with_good.quality, 6520);
    assert!(!with_good.actions.contains(&Action::PreciseTouch));
    assert!(!with_good.actions.contains(&Action::TricksOfTheTrade));
}

#[test]
fn dynamic_replans_on_hardcoded_good_steps_and_beats_static() {
    let conditions = valid_timeline(&[
        (3, Condition::Good),
        (7, Condition::Good),
        (12, Condition::Good),
        (17, Condition::Good),
    ]);
    let static_run = run_static(&conditions);
    let dynamic_run = run_dynamic(&conditions);
    print_comparison("four Good procs", &static_run, &dynamic_run);

    // Same stats, recipe and exact condition timeline. The only difference is
    // whether the solver observes each condition and replans.
    assert_eq!(static_run.quality, 6520);
    assert_eq!(dynamic_run.quality, 8876);
    assert_eq!(dynamic_run.quality - static_run.quality, 2356);

    assert_eq!(static_run.decisions[11].2, Action::BasicTouch);
    assert_eq!(dynamic_run.decisions[11].2, Action::PreciseTouch);
    assert_eq!(static_run.decisions[16].2, Action::AdvancedTouch);
    assert_eq!(dynamic_run.decisions[16].2, Action::PreciseTouch);
}

#[test]
fn dynamic_exploits_excellent_and_avoids_poor() {
    let conditions = valid_timeline(&[
        (5, Condition::Excellent), // forces Poor on step 6
        (11, Condition::Good),
        (16, Condition::Excellent), // forces Poor on step 17
    ]);
    let static_run = run_static(&conditions);
    let dynamic_run = run_dynamic(&conditions);
    print_comparison("Excellent -> Poor chains", &static_run, &dynamic_run);

    assert!(dynamic_run.quality > static_run.quality);

    // The dynamic solver must plan through the forced Excellent -> Poor edge,
    // not merely react when the test harness reaches the following step.
    assert_eq!(dynamic_run.decisions[4].1, Condition::Excellent);
    assert_eq!(dynamic_run.decisions[5].1, Condition::Poor);
    assert_eq!(dynamic_run.decisions[5].2, Action::Groundwork);
    assert_eq!(dynamic_run.decisions[15].1, Condition::Excellent);
    assert_eq!(dynamic_run.decisions[16].1, Condition::Poor);
    assert_eq!(dynamic_run.decisions[16].2, Action::Veneration);

    let quality_actions = [
        Action::BasicTouch,
        Action::StandardTouch,
        Action::ByregotsBlessing,
        Action::PreciseTouch,
        Action::PrudentTouch,
        Action::AdvancedTouch,
        Action::Reflect,
        Action::PreparatoryTouch,
        Action::DelicateSynthesis,
        Action::TrainedFinesse,
        Action::HastyTouch,
        Action::DaringTouch,
    ];
    for (_, condition, action) in &dynamic_run.decisions {
        if *condition == Condition::Poor {
            assert!(
                !quality_actions.contains(action),
                "dynamic solver spent Poor on quality action {action:?}"
            );
        }
    }
}

/// Regression for Monte Carlo seed 0x5eedc0de6178, iteration 0.
/// The first conditions are Normal, Normal, Good, so the live state immediately
/// before step 3 is deterministic and can be reproduced without the Monte Carlo
/// harness.
#[test]
fn live_good_state_after_muscle_memory_and_trained_perfection_has_a_solution() {
    let settings = Settings {
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
    let mut state = SimulationState::new(&settings);
    state = state
        .use_action(Action::MuscleMemory, Condition::Normal, &settings)
        .unwrap();
    state = state
        .use_action(Action::TrainedPerfection, Condition::Normal, &settings)
        .unwrap();

    assert_eq!(state.cp, 502);
    assert_eq!(state.durability, 70);
    assert_eq!(state.progress, 687);
    assert_eq!(state.quality, 0);

    let mut solver = MacroSolver::new(
        SolverSettings {
            simulator_settings: settings,
            allow_non_max_quality_solutions: true,
        },
        Box::new(|_| {}),
        Box::new(|_| {}),
        AtomicFlag::new(),
    );
    let actions = solver
        .solve_from_state_with_condition(state, Condition::Good)
        .expect("the live Good state must remain solvable");
    assert_eq!(actions.first(), Some(&Action::PreciseTouch));

    let mut condition = Condition::Good;
    let final_state = actions
        .iter()
        .try_fold(state, |state, &action| {
            let state = state.use_action(action, condition, &settings)?;
            if consumes_condition(action) {
                condition = condition.next_after_step();
            }
            Ok::<_, raphael_sim::ActionError>(state)
        })
        .unwrap();
    assert!(final_state.progress >= settings.max_progress);
}
