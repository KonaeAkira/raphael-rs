use simulator::{state::InProgress, Action, ActionMask, Condition, Settings, SimulationState};
use solvers::MacroSolver;

fn solve(settings: &Settings, backload_progress: bool) -> Option<Vec<Action>> {
    MacroSolver::new(settings.clone()).solve(InProgress::new(settings), backload_progress)
}

fn get_quality(settings: &Settings, actions: &[Action]) -> u16 {
    let mut state = SimulationState::new(&settings);
    for action in actions {
        state = InProgress::try_from(state)
            .unwrap()
            .use_action(action.clone(), Condition::Normal, &settings)
            .unwrap();
    }
    assert_eq!(state.missing_progress, 0);
    settings.max_quality - state.get_missing_quality()
}

fn get_duration(actions: &[Action]) -> i16 {
  actions.into_iter().map(|action| action.time_cost()).sum()
}

#[test]
fn adv_test_random_0f93c79f() {
    let settings = Settings {
        max_cp: 370,
        max_durability: 60,
        max_progress: 2000,
        max_quality: 40000,
        base_progress: 100,
        base_quality: 100,
        initial_quality: 0,
        job_level: 100,
        allowed_actions: ActionMask::from_level(100, true, false),
        adversarial: true,
    };
    let actions = solve(&settings, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 2046);
    assert_eq!(get_duration(&actions), 54);
    assert_eq!(actions.len(), 19);
}


// This test takes a long time to run right now. 
// I'm letting this run overnight, I'll patch it in later if it works 
//#[test]
fn adv_test_simul_29b0c876() {
    // lv100 Rarefied Tacos de Carne Asada
    // 4785 CMS, 4758 Ctrl, 646 CP
    let settings = Settings {
        max_cp: 646,
        max_durability: 80,
        max_progress: 6600,
        max_quality: 12000,
        base_progress: 256,
        base_quality: 265,
        initial_quality: 0,
        job_level: 100,
        allowed_actions: ActionMask::from_level(100, true, false),
        adversarial: true,
    };
    let actions = solve(&settings, false).unwrap();
    dbg!(actions.clone());
    assert_eq!(get_quality(&settings, &actions), 2046);
    assert_eq!(get_duration(&actions), 54);
    assert_eq!(actions.len(), 19);
}

#[test]
fn adv_test_simul_b6e93c9a() {
  // lv100 Rarefied Stuffed Peppers
  // 4785 CMS, 4758 Ctrl, 646 CP
    let settings = Settings {
        max_cp: 646,
        max_durability: 80,
        max_progress: 6600,
        max_quality: 12000,
        base_progress: 289,
        base_quality: 360,
        initial_quality: 0,
        job_level: 100,
        allowed_actions: ActionMask::from_level(100, true, false),
        adversarial: true,
    };
    let actions = solve(&settings, false).unwrap();
    dbg!(actions.clone());
    assert_eq!(get_quality(&settings, &actions), 11400);
    assert_eq!(get_duration(&actions), 47);
    assert_eq!(actions.len(), 17);
}