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
            finish_states: 886551,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 79712,
                dropped_nodes: 1501578,
                pareto_buckets_squared_size_sum: 1172823,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 4722074,
                pareto_values: 77746538,
            },
            step_lb_stats: StepLbSolverStats {
                states: 874432,
                pareto_values: 14960763,
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
            finish_states: 1376236,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 2332448,
                dropped_nodes: 512436,
                pareto_buckets_squared_size_sum: 143370684,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 5030738,
                pareto_values: 135274246,
            },
            step_lb_stats: StepLbSolverStats {
                states: 1412743,
                pareto_values: 31607366,
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
            finish_states: 75899,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 26294,
                dropped_nodes: 296454,
                pareto_buckets_squared_size_sum: 463009,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 3772842,
                pareto_values: 33011766,
            },
            step_lb_stats: StepLbSolverStats {
                states: 69197,
                pareto_values: 549500,
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
            finish_states: 449667,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 8953,
                dropped_nodes: 10680,
                pareto_buckets_squared_size_sum: 99526,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 5310554,
                pareto_values: 125659412,
            },
            step_lb_stats: StepLbSolverStats {
                states: 331716,
                pareto_values: 5726156,
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
            finish_states: 502169,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 553135,
                dropped_nodes: 0,
                pareto_buckets_squared_size_sum: 23891886,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 5298836,
                pareto_values: 151000601,
            },
            step_lb_stats: StepLbSolverStats {
                states: 324469,
                pareto_values: 6963158,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}
