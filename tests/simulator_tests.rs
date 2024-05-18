use raphael::game::{
    units::{Progress, Quality},
    Action, ActionMask, Condition, Settings, State,
};

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
fn test_3ba90d3a() {
    let settings = Settings {
        max_cp: 200,
        max_durability: 60,
        max_progress: Progress::new(2000),
        max_quality: Quality::new(40000),
        job_level: 81,
        allowed_actions: ActionMask::none(),
    };
    let state = from_action_sequence(
        &settings,
        &[
            Action::Veneration,
            Action::CarefulSynthesis,
            Action::CarefulSynthesis,
        ],
    );
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 168);
            assert_eq!(state.durability, 40);
            assert_eq!(
                state.missing_progress,
                settings.max_progress.saturating_sub(Progress::from(450.00))
            );
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
}
