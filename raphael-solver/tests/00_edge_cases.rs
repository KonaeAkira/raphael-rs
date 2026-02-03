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
) -> Vec<Action> {
    let mut solver = MacroSolver::new(
        settings,
        Box::new(|_| {}),
        Box::new(|_| {}),
        AtomicFlag::new(),
    );
    let result = solver.solve();
    let score = result.clone().map(|actions| {
        let final_state =
            SimulationState::from_macro(&settings.simulator_settings, &actions).unwrap();
        assert!(final_state.progress >= settings.max_progress());
        if settings.simulator_settings.backload_progress {
            assert!(is_progress_backloaded(&settings, &actions));
        }
        SolutionScore {
            capped_quality: std::cmp::min(final_state.quality, settings.max_quality()),
            steps: actions.len() as u8,
            duration: actions.iter().map(|action| action.time_cost()).sum(),
            overflow_quality: final_state.quality.saturating_sub(settings.max_quality()),
        }
    });
    expected_score.assert_debug_eq(&score);
    expected_runtime_stats.assert_debug_eq(&solver.runtime_stats());
    result.unwrap_or_default()
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
        stellar_steady_hand_charges: 0,
    };
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
    let expected_score = expect![[r#"
        Err(
            NoSolution,
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 0,
                processed_nodes: 0,
            },
            finish_solver_stats: FinishSolverStats {
                states: 4659,
                values: 81186,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 0,
                states_on_shards: 0,
                values: 0,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 0,
                states_on_shards: 0,
                values: 0,
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
        stellar_steady_hand_charges: 0,
    };
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
    let expected_score = expect![[r#"
        Ok(
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
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 64,
                processed_nodes: 42,
            },
            finish_solver_stats: FinishSolverStats {
                states: 4659,
                values: 49135,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 31147,
                states_on_shards: 0,
                values: 109398,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 26538,
                states_on_shards: 0,
                values: 194222,
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
        stellar_steady_hand_charges: 0,
    };
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
    let expected_score = expect![[r#"
        Ok(
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
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 97351,
                processed_nodes: 6915,
            },
            finish_solver_stats: FinishSolverStats {
                states: 4659,
                values: 66696,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 389796,
                states_on_shards: 514,
                values: 2236380,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 26538,
                states_on_shards: 50910,
                values: 486877,
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
        stellar_steady_hand_charges: 0,
    };
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
    let expected_score = expect![[r#"
        Ok(
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
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 1,
                processed_nodes: 1,
            },
            finish_solver_stats: FinishSolverStats {
                states: 21848,
                values: 21848,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 178982,
                states_on_shards: 5,
                values: 178987,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 1,
                states_on_shards: 13,
                values: 14,
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
        stellar_steady_hand_charges: 0,
    };
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
    let expected_score = expect![[r#"
        Ok(
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
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 1,
                processed_nodes: 1,
            },
            finish_solver_stats: FinishSolverStats {
                states: 3478,
                values: 3478,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 3243,
                states_on_shards: 0,
                values: 3243,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 1,
                states_on_shards: 9,
                values: 10,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
/// https://github.com/KonaeAkira/raphael-rs/issues/216
fn issue_216_steplbsolver_crash() {
    let simulator_settings = Settings {
        max_cp: 649,
        max_durability: 40,
        max_progress: 2125,
        max_quality: 8600,
        base_progress: 400,
        base_quality: 468,
        job_level: 100,
        allowed_actions: ActionMask::regular(),
        adversarial: false,
        backload_progress: false,
        stellar_steady_hand_charges: 0,
    };
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
    let expected_score = expect![[r#"
        Ok(
            SolutionScore {
                capped_quality: 8600,
                steps: 10,
                duration: 25,
                overflow_quality: 596,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 436353,
                processed_nodes: 21746,
            },
            finish_solver_stats: FinishSolverStats {
                states: 9851,
                values: 12801,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 318520,
                states_on_shards: 0,
                values: 1267763,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 36739,
                states_on_shards: 29009,
                values: 289165,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
/// https://github.com/KonaeAkira/raphael-rs/issues/312
fn issue_312_quick_innovation_reflect() {
    let simulator_settings = Settings {
        max_cp: 18,
        max_durability: 55,
        max_progress: 550,
        max_quality: 1399,
        base_progress: 306,
        base_quality: 311,
        job_level: 100,
        allowed_actions: ActionMask::regular().add(Action::QuickInnovation),
        adversarial: false,
        backload_progress: false,
        stellar_steady_hand_charges: 0,
    };
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
    let expected_score = expect![[r#"
        Ok(
            SolutionScore {
                capped_quality: 1399,
                steps: 3,
                duration: 9,
                overflow_quality: 0,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 17,
                processed_nodes: 4,
            },
            finish_solver_stats: FinishSolverStats {
                states: 10142,
                values: 10228,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 28849,
                states_on_shards: 1,
                values: 33547,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 46,
                states_on_shards: 75,
                values: 194,
            },
        }
    "#]];
    let actions = test_with_settings(solver_settings, expected_score, expected_runtime_stats);
    assert_eq!(
        actions,
        [
            Action::QuickInnovation,
            Action::Reflect,
            Action::CarefulSynthesis
        ]
    );
}

#[test]
/// The Hasty Touch > Daring Touch "combo" is not actually a combo, as Daring Touch is enabled
/// by the Expedience effect, which means that actions that don't tick effects such as Quick Innovation
/// may be used inbetween Hasty Touch and Daring Touch.
///
/// This is a specially constructed case where using Hasty Touch > Quick Innovation > Daring Touch
/// is optimal to test that the solver doesn't accidentally "optimize away" this particular case.
fn daring_touch_interrupted_combo() {
    let simulator_settings = Settings {
        max_cp: 0,
        max_durability: 30,
        max_progress: 500,
        max_quality: 347,
        base_progress: 100,
        base_quality: 100,
        job_level: 100,
        allowed_actions: ActionMask::regular().add(Action::QuickInnovation),
        adversarial: false,
        backload_progress: false,
        stellar_steady_hand_charges: 1,
    };
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
    let expected_score = expect![[r#"
        Ok(
            SolutionScore {
                capped_quality: 347,
                steps: 5,
                duration: 14,
                overflow_quality: 0,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 29,
                processed_nodes: 10,
            },
            finish_solver_stats: FinishSolverStats {
                states: 24476,
                values: 24476,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 22280,
                states_on_shards: 43,
                values: 26453,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 403,
                states_on_shards: 1482,
                values: 4815,
            },
        }
    "#]];
    let actions = test_with_settings(solver_settings, expected_score, expected_runtime_stats);
    assert_eq!(
        actions,
        [
            Action::StellarSteadyHand,
            Action::HastyTouch,
            Action::QuickInnovation,
            Action::DaringTouch,
            Action::RapidSynthesis
        ]
    );
}
