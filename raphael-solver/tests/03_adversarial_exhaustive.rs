use expect_test::expect;
use raphael_sim::*;
use raphael_solver::{AtomicFlag, MacroSolver, SolverSettings};

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct SolutionScore {
    pub capped_quality: u32,
    pub steps: u8,
    pub duration: u8,
    pub overflow_quality: u32,
}

fn is_progress_backloaded(settings: &SolverSettings, actions: &[Action]) -> bool {
    let mut state = SimulationState::new(&settings.simulator_settings);
    let mut quality_lock = None;
    for action in actions {
        state = state
            .use_action(*action, Condition::Normal, &settings.simulator_settings)
            .unwrap();
        if state.progress != 0 && quality_lock.is_none() {
            quality_lock = Some(state.quality);
        }
    }
    quality_lock.is_none_or(|quality| state.quality == quality)
}

fn test_with_settings(
    settings: SolverSettings,
    expected_score: expect_test::Expect,
    expected_runtime_stats: expect_test::Expect,
) {
    let mut solver = MacroSolver::new(
        settings,
        Box::new(|_| {}),
        Box::new(|_| {}),
        AtomicFlag::new(),
    );
    let result = solver.solve();
    let score = result.map_or(None, |actions| {
        let final_state =
            SimulationState::from_macro(&settings.simulator_settings, &actions).unwrap();
        assert!(final_state.progress >= settings.max_progress());
        if settings.simulator_settings.backload_progress {
            assert!(is_progress_backloaded(&settings, &actions));
        }
        Some(SolutionScore {
            capped_quality: std::cmp::min(final_state.quality, settings.max_quality()),
            steps: actions.len() as u8,
            duration: actions.iter().map(|action| action.time_cost()).sum(),
            overflow_quality: final_state.quality.saturating_sub(settings.max_quality()),
        })
    });
    expected_score.assert_debug_eq(&score);
    expected_runtime_stats.assert_debug_eq(&solver.runtime_stats());
}

const SETTINGS: Settings = Settings {
    max_cp: 370,
    max_durability: 60,
    max_progress: 2000,
    max_quality: 40000,
    base_progress: 100,
    base_quality: 100,
    job_level: 100,
    allowed_actions: ActionMask::all()
        .remove(Action::TrainedEye)
        .remove(Action::HeartAndSoul)
        .remove(Action::QuickInnovation),
    adversarial: true,
    backload_progress: false,
};

#[test]
fn stuffed_peppers() {
    // lv99 Rarefied Stuffed Peppers
    // 4785 CMS, 4758 Ctrl, 646 CP
    let simulator_settings = Settings {
        max_cp: 646,
        max_durability: 80,
        max_progress: 6300,
        max_quality: 11400,
        base_progress: 289,
        base_quality: 360,
        ..SETTINGS
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 11400,
                steps: 16,
                duration: 45,
                overflow_quality: 282,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 922126,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 87724,
                dropped_nodes: 1638093,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 4716375,
                pareto_values: 77743046,
            },
            step_lb_stats: StepLbSolverStats {
                states: 851371,
                pareto_values: 14949004,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn test_rare_tacos_2() {
    // lv100 Rarefied Tacos de Carne Asada
    // 4785 CMS, 4758 Ctrl, 646 CP
    let simulator_settings = Settings {
        max_cp: 646,
        max_durability: 80,
        max_progress: 6600,
        max_quality: 12000,
        base_progress: 256,
        base_quality: 265,
        job_level: 100,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 12000,
                steps: 32,
                duration: 91,
                overflow_quality: 138,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 1376491,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 2400710,
                dropped_nodes: 526101,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 5030014,
                pareto_values: 135253994,
            },
            step_lb_stats: StepLbSolverStats {
                states: 1388898,
                pareto_values: 31625687,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn test_mountain_chromite_ingot_no_manipulation() {
    // Mountain Chromite Ingot
    // 3076 Craftsmanship, 3106 Control, Level 90, HQ Tsai Tou Vonou
    let simulator_settings = Settings {
        max_cp: 616,
        max_durability: 40,
        max_progress: 2000,
        max_quality: 8200,
        base_progress: 217,
        base_quality: 293,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::Manipulation)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 8200,
                steps: 14,
                duration: 38,
                overflow_quality: 32,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 79090,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 30335,
                dropped_nodes: 330886,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 3768983,
                pareto_values: 33009543,
            },
            step_lb_stats: StepLbSolverStats {
                states: 58508,
                pareto_values: 520357,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn test_indagator_3858_4057() {
    let simulator_settings = Settings {
        max_cp: 687,
        max_durability: 70,
        max_progress: 5720,
        max_quality: 12900,
        base_progress: 239,
        base_quality: 271,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 10686,
                steps: 26,
                duration: 71,
                overflow_quality: 0,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 449841,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 8982,
                dropped_nodes: 10698,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 5297705,
                pareto_values: 125105998,
            },
            step_lb_stats: StepLbSolverStats {
                states: 675136,
                pareto_values: 13435389,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn test_rare_tacos_4628_4410() {
    let simulator_settings = Settings {
        max_cp: 675,
        max_durability: 80,
        max_progress: 6600,
        max_quality: 12000,
        base_progress: 246,
        base_quality: 246,
        job_level: 100,
        allowed_actions: ActionMask::all()
            .remove(Action::Manipulation)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 11748,
                steps: 31,
                duration: 88,
                overflow_quality: 0,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 502537,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 563469,
                dropped_nodes: 0,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 5297719,
                pareto_values: 150961450,
            },
            step_lb_stats: StepLbSolverStats {
                states: 319260,
                pareto_values: 6969709,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}
