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

#[test]
fn unsolvable() {
    let simulator_settings = Settings {
        max_cp: 100,
        max_durability: 60,
        max_progress: 4000,
        max_quality: 1000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::regular(),
        adversarial: false,
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        None
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 2864,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 0,
                dropped_nodes: 0,
                pareto_buckets_squared_size_sum: 0,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 0,
                sequential_states: 0,
                pareto_values: 0,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 0,
                pareto_values: 0,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn zero_quality() {
    let simulator_settings = Settings {
        max_cp: 80,
        max_durability: 60,
        max_progress: 1920,
        max_quality: 1000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::regular(),
        adversarial: false,
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 0,
                steps: 5,
                duration: 14,
                overflow_quality: 0,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 1660,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 41,
                dropped_nodes: 12,
                pareto_buckets_squared_size_sum: 73,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 31147,
                sequential_states: 0,
                pareto_values: 109398,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 228416,
                pareto_values: 1548874,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn max_quality() {
    let simulator_settings = Settings {
        max_cp: 400,
        max_durability: 60,
        max_progress: 2000,
        max_quality: 1000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::regular(),
        adversarial: false,
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 1000,
                steps: 11,
                duration: 28,
                overflow_quality: 100,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 245618,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 5843,
                dropped_nodes: 68720,
                pareto_buckets_squared_size_sum: 36691,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 389796,
                sequential_states: 2516,
                pareto_values: 2988048,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 239199,
                pareto_values: 1636066,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn large_progress_quality_increase() {
    let simulator_settings = Settings {
        max_cp: 300,
        max_durability: 40,
        max_progress: 100,
        max_quality: 100,
        base_progress: 5000,
        base_quality: 5000,
        job_level: 100,
        allowed_actions: ActionMask::all(),
        adversarial: false,
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 100,
                steps: 1,
                duration: 3,
                overflow_quality: 4900,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 24,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 0,
                dropped_nodes: 23,
                pareto_buckets_squared_size_sum: 0,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 178982,
                sequential_states: 0,
                pareto_values: 178982,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 6336,
                pareto_values: 6336,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn backload_progress_single_delicate_synthesis() {
    let simulator_settings = Settings {
        max_cp: 100,
        max_durability: 20,
        max_progress: 100,
        max_quality: 100,
        base_progress: 100,
        base_quality: 100,
        job_level: 100,
        allowed_actions: ActionMask::regular(),
        adversarial: false,
        backload_progress: true,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 100,
                steps: 1,
                duration: 3,
                overflow_quality: 0,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 15,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 0,
                dropped_nodes: 14,
                pareto_buckets_squared_size_sum: 0,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 8965,
                sequential_states: 0,
                pareto_values: 8965,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1596,
                pareto_values: 1596,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}
