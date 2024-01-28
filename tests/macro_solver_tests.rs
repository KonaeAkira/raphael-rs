use raphael_rs::{
    config::Settings,
    game::{
        actions::{Action, PROG_DENOM, QUAL_DENOM},
        conditions::Condition,
        state::State,
    },
    progress, quality,
    solvers::macro_solver::MacroSolver,
};

fn solve(settings: &Settings) -> Option<Vec<Action>> {
    MacroSolver::new(settings.clone()).solve(State::new(settings))
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

#[test]
fn test_01() {
    let settings = Settings {
        max_cp: 240,
        max_durability: 60,
        max_progress: progress!(2000),
        max_quality: quality!(40000),
    };
    let actions = solve(&settings).unwrap();
    let final_state = from_action_sequence(&settings, &actions)
        .as_completed()
        .unwrap();
    assert_eq!(final_state.quality, quality!(667.5));
}

#[test]
fn test_02() {
    let settings = Settings {
        max_cp: 320,
        max_durability: 70,
        max_progress: progress!(2400),
        max_quality: quality!(20000),
    };
    let actions = solve(&settings).unwrap();
    let final_state = from_action_sequence(&settings, &actions)
        .as_completed()
        .unwrap();
    assert_eq!(final_state.quality, quality!(1068));
}