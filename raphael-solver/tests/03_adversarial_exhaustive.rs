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
            finish_states: 882590,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 79649,
                dropped_nodes: 1553519,
                pareto_buckets_squared_size_sum: 1190791,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 2271577,
                sequential_states: 6893,
                pareto_values: 39211597,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1414006,
                sequential_states: 35537,
                pareto_values: 21195116,
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
            finish_states: 1465125,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 3385746,
                dropped_nodes: 28572558,
                pareto_buckets_squared_size_sum: 298697843,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 2490785,
                sequential_states: 89951,
                pareto_values: 70479369,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 2227573,
                sequential_states: 47014,
                pareto_values: 43597095,
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
            finish_states: 76205,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 26562,
                dropped_nodes: 348147,
                pareto_buckets_squared_size_sum: 456620,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 1800446,
                sequential_states: 5317,
                pareto_values: 16383032,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 52408,
                sequential_states: 11357,
                pareto_values: 542930,
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
            finish_states: 512190,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 20339,
                dropped_nodes: 197983,
                pareto_buckets_squared_size_sum: 312904,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 2515306,
                sequential_states: 151863,
                pareto_values: 65076119,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 0,
                sequential_states: 0,
                pareto_values: 0,
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
            finish_states: 553896,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 947084,
                dropped_nodes: 2311281,
                pareto_buckets_squared_size_sum: 65503086,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 2623634,
                sequential_states: 79247,
                pareto_values: 78388861,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 420784,
                sequential_states: 31429,
                pareto_values: 8950199,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn issue_113() {
    // https://github.com/KonaeAkira/raphael-rs/issues/113
    // Ceremonial Gunblade
    // 5428/5236/645 + HQ Ceviche + HQ Cunning Tisane
    let simulator_settings = Settings {
        max_cp: 768,
        max_durability: 70,
        max_progress: 9000,
        max_quality: 18700,
        base_progress: 297,
        base_quality: 288,
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
                capped_quality: 14070,
                steps: 33,
                duration: 93,
                overflow_quality: 0,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 1959668,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 1267875,
                dropped_nodes: 17909160,
                pareto_buckets_squared_size_sum: 78126500,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 3043554,
                sequential_states: 94212,
                pareto_values: 121470665,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 0,
                sequential_states: 0,
                pareto_values: 0,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn issue_118() {
    // https://github.com/KonaeAkira/raphael-rs/issues/118
    let simulator_settings = Settings {
        max_cp: 614,
        max_durability: 20,
        max_progress: 2310,
        max_quality: 8400,
        base_progress: 205,
        base_quality: 240,
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
                capped_quality: 8400,
                steps: 19,
                duration: 52,
                overflow_quality: 84,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 571064,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 1214933,
                dropped_nodes: 14402683,
                pareto_buckets_squared_size_sum: 216063563,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 1931154,
                sequential_states: 59856,
                pareto_values: 25546575,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 258314,
                sequential_states: 17917,
                pareto_values: 2875845,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}
