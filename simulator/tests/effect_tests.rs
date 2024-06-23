// use simulator::{state::InProgress, Action, ActionMask, Condition, Settings, SimulationState};

// const SETTINGS: Settings = Settings {
//     max_cp: 200,
//     max_durability: 60,
//     max_progress: 2000,
//     max_quality: 40000,
//     base_progress: 100,
//     base_quality: 100,
//     job_level: 90,
//     allowed_actions: ActionMask::none(),
// };

// #[test]
// fn test_muscle_memory() {
//     let mut state = SimulationState::new(&SETTINGS);
//     state.effects.muscle_memory = 3;
//     let state = InProgress::try_from(state).unwrap();
//     let new_state = state
//         .use_action(Action::CarefulSynthesis, Condition::Normal, &SETTINGS)
//         .unwrap();
//     assert_eq!(
//         new_state.missing_progress,
//         SETTINGS.max_progress.saturating_sub(360)
//     );
//     assert_eq!(new_state.effects.muscle_memory, 0);
//     let new_state = state
//         .use_action(Action::BasicTouch, Condition::Normal, &SETTINGS)
//         .unwrap();
//     assert_eq!(new_state.effects.muscle_memory, 2);
// }

// #[test]
// fn test_veneration() {
//     let mut state = InProgress::new(&SETTINGS);
//     state.raw_state().effects.veneration = 3;
//     match state.use_action(Action::CarefulSynthesis, Condition::Normal, &SETTINGS) {
//         State::InProgress(state) => {
//             assert_eq!(
//                 state.missing_progress,
//                 SETTINGS.max_progress.saturating_sub(270)
//             );
//             assert_eq!(state.effects.veneration, 2);
//         }
//         _ => panic!(),
//     }
// }

// #[test]
// fn test_muscle_memory_veneration() {
//     let mut state = InProgress::new(&SETTINGS);
//     state.raw_state().effects.muscle_memory = 3;
//     state.raw_state().effects.veneration = 3;
//     match state.use_action(Action::CarefulSynthesis, Condition::Normal, &SETTINGS) {
//         State::InProgress(state) => {
//             assert_eq!(
//                 state.missing_progress,
//                 SETTINGS.max_progress.saturating_sub(450)
//             );
//             assert_eq!(state.effects.muscle_memory, 0);
//             assert_eq!(state.effects.veneration, 2);
//         }
//         _ => panic!(),
//     }
// }

// #[test]
// fn test_waste_not() {
//     let mut state = InProgress::new(&SETTINGS);
//     state.raw_state().effects.waste_not = 3;
//     match state.use_action(Action::CarefulSynthesis, Condition::Normal, &SETTINGS) {
//         State::InProgress(state) => {
//             assert_eq!(state.durability, 55);
//             assert_eq!(state.effects.waste_not, 2);
//         }
//         _ => panic!(),
//     }
//     match state.use_action(Action::PrudentTouch, Condition::Normal, &SETTINGS) {
//         State::Invalid => (),
//         _ => panic!(),
//     }
//     match state.use_action(Action::PrudentSynthesis, Condition::Normal, &SETTINGS) {
//         State::Invalid => (),
//         _ => panic!(),
//     }
// }

// #[test]
// fn test_manipulation() {
//     let mut state = InProgress::new(&SETTINGS);
//     state.raw_state().effects.manipulation = 3;
//     state.raw_state().durability = 30;
//     match state.use_action(Action::BasicSynthesis, Condition::Normal, &SETTINGS) {
//         State::InProgress(state) => {
//             assert_eq!(state.durability, 25);
//             assert_eq!(state.effects.manipulation, 2);
//         }
//         _ => panic!(),
//     }
//     match state.use_action(Action::Observe, Condition::Normal, &SETTINGS) {
//         State::InProgress(state) => {
//             assert_eq!(state.durability, 35);
//             assert_eq!(state.effects.manipulation, 2);
//         }
//         _ => panic!(),
//     }
//     match state.use_action(Action::Manipulation, Condition::Normal, &SETTINGS) {
//         State::InProgress(state) => {
//             assert_eq!(state.durability, 30);
//             assert_eq!(state.effects.manipulation, 8);
//         }
//         _ => panic!(),
//     }
//     match state.use_action(Action::MasterMend, Condition::Normal, &SETTINGS) {
//         State::InProgress(state) => {
//             assert_eq!(state.durability, 60);
//             assert_eq!(state.effects.manipulation, 2);
//         }
//         _ => panic!(),
//     }
// }

// #[test]
// fn test_great_strides() {
//     let mut state = InProgress::new(&SETTINGS);
//     state.raw_state().effects.great_strides = 3;
//     match state.use_action(Action::BasicTouch, Condition::Normal, &SETTINGS) {
//         State::InProgress(state) => {
//             assert_eq!(
//                 state.missing_quality,
//                 SETTINGS.max_quality.saturating_sub(200)
//             );
//             assert_eq!(state.effects.great_strides, 0);
//         }
//         _ => panic!(),
//     }
//     match state.use_action(Action::BasicSynthesis, Condition::Normal, &SETTINGS) {
//         State::InProgress(state) => {
//             assert_eq!(state.effects.great_strides, 2);
//         }
//         _ => panic!(),
//     }
// }

// #[test]
// fn test_innovation() {
//     let mut state = InProgress::new(&SETTINGS);
//     state.raw_state().effects.innovation = 3;
//     match state.use_action(Action::BasicTouch, Condition::Normal, &SETTINGS) {
//         State::InProgress(state) => {
//             assert_eq!(
//                 state.missing_quality,
//                 SETTINGS.max_quality.saturating_sub(150)
//             );
//             assert_eq!(state.effects.innovation, 2);
//         }
//         _ => panic!(),
//     }
// }

// #[test]
// fn test_great_strides_innovation() {
//     let mut state = InProgress::new(&SETTINGS);
//     state.raw_state().effects.great_strides = 3;
//     state.raw_state().effects.innovation = 3;
//     match state.use_action(Action::BasicTouch, Condition::Normal, &SETTINGS) {
//         State::InProgress(state) => {
//             assert_eq!(
//                 state.missing_quality,
//                 SETTINGS.max_quality.saturating_sub(250)
//             );
//             assert_eq!(state.effects.great_strides, 0);
//             assert_eq!(state.effects.innovation, 2);
//         }
//         _ => panic!(),
//     }
// }

// #[test]
// fn test_inner_quiet() {
//     let mut state = InProgress::new(&SETTINGS);
//     state.raw_state().effects.inner_quiet = 4;
//     match state.use_action(Action::BasicTouch, Condition::Normal, &SETTINGS) {
//         State::InProgress(state) => {
//             assert_eq!(
//                 state.missing_quality,
//                 SETTINGS.max_quality.saturating_sub(140)
//             );
//             assert_eq!(state.effects.inner_quiet, 5);
//         }
//         _ => panic!(),
//     }
// }

// #[test]
// fn test_innovation_inner_quiet() {
//     let mut state = InProgress::new(&SETTINGS);
//     state.raw_state().effects.innovation = 3;
//     state.raw_state().effects.inner_quiet = 4;
//     match state.use_action(Action::BasicTouch, Condition::Normal, &SETTINGS) {
//         State::InProgress(state) => {
//             assert_eq!(
//                 state.missing_quality,
//                 SETTINGS.max_quality.saturating_sub(210)
//             );
//             assert_eq!(state.effects.innovation, 2);
//             assert_eq!(state.effects.inner_quiet, 5);
//         }
//         _ => panic!(),
//     }
// }
