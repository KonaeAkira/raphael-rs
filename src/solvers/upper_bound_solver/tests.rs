use crate::{
    game::{
        units::{Progress, Quality},
        Action, Condition, Settings, State,
    },
    solvers::UpperBoundSolver,
};

fn solve(settings: Settings, actions: &[Action]) -> f32 {
    let state = State::new(&settings).use_actions(actions, Condition::Normal, &settings);
    let result = UpperBoundSolver::new(settings)
        .quality_upper_bound(state.as_in_progress().unwrap())
        .into();
    dbg!(result);
    result
}

#[test]
fn test_01() {
    let settings = Settings {
        max_cp: 553,
        max_durability: 70,
        max_progress: Progress::from(2400.00),
        max_quality: Quality::from(20000.00),
    };
    let result = solve(
        settings,
        &[
            Action::MuscleMemory,
            Action::PrudentTouch,
            Action::Manipulation,
            Action::Veneration,
            Action::WasteNot2,
            Action::Groundwork,
            Action::Groundwork,
            Action::Groundwork,
            Action::PreparatoryTouch,
        ],
    );
    assert_eq!(result, 3455.00); // tightness test
    assert!(result >= 3352.50); // correctness test
}
