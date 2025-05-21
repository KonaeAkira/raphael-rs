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
            finish_states: 885035,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 79719,
                dropped_nodes: 1553283,
                pareto_buckets_squared_size_sum: 1172750,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 2490788,
                sequential_states: 9422,
                pareto_values: 39433322,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1414006,
                sequential_states: 35545,
                pareto_values: 21195115,
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
            finish_states: 1464478,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 3422528,
                dropped_nodes: 28744543,
                pareto_buckets_squared_size_sum: 256757713,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 2490788,
                sequential_states: 90229,
                pareto_values: 70483459,
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
            finish_states: 75996,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 26352,
                dropped_nodes: 344794,
                pareto_buckets_squared_size_sum: 463755,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 1839894,
                sequential_states: 50564,
                pareto_values: 16815732,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 52311,
                sequential_states: 11349,
                pareto_values: 542814,
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
            finish_states: 512167,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 20339,
                dropped_nodes: 198021,
                pareto_buckets_squared_size_sum: 313143,
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
            finish_states: 553908,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 957801,
                dropped_nodes: 2312288,
                pareto_buckets_squared_size_sum: 56259629,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 2623634,
                sequential_states: 79243,
                pareto_values: 78388759,
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
            finish_states: 1960135,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 1282661,
                dropped_nodes: 18120915,
                pareto_buckets_squared_size_sum: 65182040,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 3043554,
                sequential_states: 94213,
                pareto_values: 121470710,
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
            finish_states: 571554,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 1223706,
                dropped_nodes: 14462624,
                pareto_buckets_squared_size_sum: 248245795,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 1931356,
                sequential_states: 70697,
                pareto_values: 25707745,
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
