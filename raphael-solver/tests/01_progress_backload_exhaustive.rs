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
fn rinascita_3700_3280() {
    let simulator_settings = Settings {
        max_cp: 680,
        max_durability: 70,
        max_progress: 5060,
        max_quality: 12628,
        base_progress: 229,
        base_quality: 224,
        job_level: 90,
        allowed_actions: ActionMask::regular(),
        adversarial: false,
        backload_progress: true,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 10492,
                steps: 25,
                duration: 66,
                overflow_quality: 0,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 212417,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 459,
                dropped_nodes: 4545,
                pareto_buckets_squared_size_sum: 1781,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 957930,
                sequential_states: 42813,
                pareto_values: 17006469,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1182529,
                pareto_values: 12830718,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn pactmaker_3240_3130() {
    let simulator_settings = Settings {
        max_cp: 600,
        max_durability: 70,
        max_progress: 4300,
        max_quality: 12800,
        base_progress: 200,
        base_quality: 215,
        job_level: 90,
        allowed_actions: ActionMask::regular(),
        adversarial: false,
        backload_progress: true,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 8801,
                steps: 24,
                duration: 65,
                overflow_quality: 0,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 267636,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 4623,
                dropped_nodes: 34199,
                pareto_buckets_squared_size_sum: 171614,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 818130,
                sequential_states: 44933,
                pareto_values: 13132245,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1178119,
                pareto_values: 12523354,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn pactmaker_3240_3130_heart_and_soul() {
    let simulator_settings = Settings {
        max_cp: 600,
        max_durability: 70,
        max_progress: 4300,
        max_quality: 12800,
        base_progress: 200,
        base_quality: 215,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::QuickInnovation),
        adversarial: false,
        backload_progress: true,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 9608,
                steps: 24,
                duration: 65,
                overflow_quality: 0,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 248396,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 577,
                dropped_nodes: 6649,
                pareto_buckets_squared_size_sum: 3863,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 1690660,
                sequential_states: 84617,
                pareto_values: 27637851,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 2628018,
                pareto_values: 26945403,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn diadochos_4021_3660() {
    let simulator_settings = Settings {
        max_cp: 640,
        max_durability: 70,
        max_progress: 6600,
        max_quality: 14040,
        base_progress: 249,
        base_quality: 247,
        job_level: 90,
        allowed_actions: ActionMask::regular(),
        adversarial: false,
        backload_progress: true,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 9580,
                steps: 23,
                duration: 61,
                overflow_quality: 0,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 426813,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 1630,
                dropped_nodes: 5157,
                pareto_buckets_squared_size_sum: 24480,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 888030,
                sequential_states: 44397,
                pareto_values: 17385016,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1314510,
                pareto_values: 16561809,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn indagator_3858_4057() {
    let simulator_settings = Settings {
        max_cp: 687,
        max_durability: 70,
        max_progress: 5720,
        max_quality: 12900,
        base_progress: 239,
        base_quality: 271,
        job_level: 90,
        allowed_actions: ActionMask::regular(),
        adversarial: false,
        backload_progress: true,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 12313,
                steps: 27,
                duration: 72,
                overflow_quality: 0,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 346009,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 3359,
                dropped_nodes: 19188,
                pareto_buckets_squared_size_sum: 65123,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 971910,
                sequential_states: 45972,
                pareto_values: 17456807,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1037399,
                pareto_values: 11619982,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn rarefied_tacos_de_carne_asada_4785_4758() {
    let simulator_settings = Settings {
        max_cp: 646,
        max_durability: 80,
        max_progress: 6600,
        max_quality: 12000,
        base_progress: 256,
        base_quality: 265,
        job_level: 100,
        allowed_actions: ActionMask::regular(),
        adversarial: false,
        backload_progress: true,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 12000,
                steps: 22,
                duration: 58,
                overflow_quality: 82,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 1197808,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 65094,
                dropped_nodes: 265540,
                pareto_buckets_squared_size_sum: 4726300,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 945752,
                sequential_states: 10989,
                pareto_values: 18108944,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 2288175,
                pareto_values: 24966527,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn stuffed_peppers_2() {
    // lv99 Rarefied Stuffed Peppers
    // 4785 CMS, 4758 Ctrl, 646 CP
    let simulator_settings = Settings {
        max_cp: 646,
        max_durability: 80,
        max_progress: 6300,
        max_quality: 11400,
        base_progress: 289,
        base_quality: 360,
        job_level: 100,
        allowed_actions: ActionMask::regular(),
        adversarial: false,
        backload_progress: true,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 11400,
                steps: 16,
                duration: 44,
                overflow_quality: 984,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 354023,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 3145,
                dropped_nodes: 47888,
                pareto_buckets_squared_size_sum: 31195,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 727703,
                sequential_states: 0,
                pareto_values: 9890579,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1471490,
                pareto_values: 12277753,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn stuffed_peppers_2_heart_and_soul() {
    // lv99 Rarefied Stuffed Peppers
    // 4785 CMS, 4758 Ctrl, 646 CP
    let simulator_settings = Settings {
        max_cp: 646,
        max_durability: 80,
        max_progress: 6300,
        max_quality: 11400,
        base_progress: 289,
        base_quality: 360,
        job_level: 100,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::QuickInnovation),
        adversarial: false,
        backload_progress: true,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 11400,
                steps: 16,
                duration: 43,
                overflow_quality: 624,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 479880,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 3679,
                dropped_nodes: 63489,
                pareto_buckets_squared_size_sum: 36165,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 1547790,
                sequential_states: 0,
                pareto_values: 20676360,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 3316813,
                pareto_values: 25990649,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn stuffed_peppers_2_quick_innovation() {
    // lv99 Rarefied Stuffed Peppers
    // 4785 CMS, 4758 Ctrl, 646 CP
    let simulator_settings = Settings {
        max_cp: 646,
        max_durability: 80,
        max_progress: 6300,
        max_quality: 11400,
        base_progress: 289,
        base_quality: 360,
        job_level: 100,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul),
        adversarial: false,
        backload_progress: true,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 11400,
                steps: 16,
                duration: 44,
                overflow_quality: 984,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 354024,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 3145,
                dropped_nodes: 49074,
                pareto_buckets_squared_size_sum: 31195,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 1477069,
                sequential_states: 0,
                pareto_values: 20284215,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 3049851,
                pareto_values: 25545822,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn rakaznar_lapidary_hammer_4462_4391() {
    let simulator_settings = Settings {
        max_cp: 569,
        max_durability: 80,
        max_progress: 6600,
        max_quality: 6500, // full HQ mats, 12500 custom target
        base_progress: 237,
        base_quality: 245,
        job_level: 100,
        allowed_actions: ActionMask::regular(),
        adversarial: false,
        backload_progress: true,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 6500,
                steps: 16,
                duration: 45,
                overflow_quality: 556,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 605814,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 292,
                dropped_nodes: 3564,
                pareto_buckets_squared_size_sum: 644,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 750109,
                sequential_states: 0,
                pareto_values: 11255498,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1560786,
                pareto_values: 13916797,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn black_star_4048_3997() {
    let simulator_settings = Settings {
        max_cp: 596,
        max_durability: 40,
        max_progress: 3000,
        max_quality: 5500, // full HQ mats
        base_progress: 250,
        base_quality: 312,
        job_level: 90,
        allowed_actions: ActionMask::regular(),
        adversarial: false,
        backload_progress: true,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 5500,
                steps: 12,
                duration: 31,
                overflow_quality: 926,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 136859,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 9079,
                dropped_nodes: 111869,
                pareto_buckets_squared_size_sum: 356414,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 448545,
                sequential_states: 20,
                pareto_values: 3013677,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 163956,
                pareto_values: 924454,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn claro_walnut_lumber_4900_4800() {
    let simulator_settings = Settings {
        max_cp: 620,
        max_durability: 40,
        max_progress: 3000,
        max_quality: 11000,
        base_progress: 300,
        base_quality: 368,
        job_level: 100,
        allowed_actions: ActionMask::regular(),
        adversarial: false,
        backload_progress: true,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 11000,
                steps: 14,
                duration: 35,
                overflow_quality: 517,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 315154,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 27817,
                dropped_nodes: 430405,
                pareto_buckets_squared_size_sum: 1595686,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 632525,
                sequential_states: 39,
                pareto_values: 4877078,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 452635,
                pareto_values: 2789361,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn rakaznar_lapidary_hammer_4900_4800() {
    let simulator_settings = Settings {
        max_cp: 620,
        max_durability: 80,
        max_progress: 6600,
        max_quality: 6000, // full hq-mats
        base_progress: 261,
        base_quality: 266,
        job_level: 100,
        allowed_actions: ActionMask::regular(),
        adversarial: false,
        backload_progress: true,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 6000,
                steps: 15,
                duration: 41,
                overflow_quality: 15,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 395204,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 357,
                dropped_nodes: 4875,
                pareto_buckets_squared_size_sum: 679,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 654113,
                sequential_states: 0,
                pareto_values: 8411272,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1317961,
                pareto_values: 10007751,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn rarefied_tacos_de_carne_asada_4966_4817() {
    let simulator_settings = Settings {
        max_cp: 626,
        max_durability: 80,
        max_progress: 6600,
        max_quality: 5400, // full hq-mats, 95% target
        base_progress: 264,
        base_quality: 267,
        job_level: 100,
        allowed_actions: ActionMask::regular(),
        adversarial: false,
        backload_progress: true,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 5400,
                steps: 14,
                duration: 38,
                overflow_quality: 317,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 457491,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 1202,
                dropped_nodes: 18056,
                pareto_buckets_squared_size_sum: 4650,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 620174,
                sequential_states: 0,
                pareto_values: 7334997,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1101885,
                pareto_values: 8009196,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn archeo_kingdom_broadsword_4966_4914() {
    let simulator_settings = Settings {
        max_cp: 745,
        max_durability: 70,
        max_progress: 7500,
        max_quality: 8250, // full hq-mats
        base_progress: 264,
        base_quality: 271,
        job_level: 100,
        allowed_actions: ActionMask::regular(),
        adversarial: false,
        backload_progress: true,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 8250,
                steps: 18,
                duration: 49,
                overflow_quality: 799,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 1171247,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 16606,
                dropped_nodes: 251621,
                pareto_buckets_squared_size_sum: 433633,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 871349,
                sequential_states: 0,
                pareto_values: 14549573,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1539116,
                pareto_values: 14705438,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn hardened_survey_plank_5558_5216() {
    let simulator_settings = Settings {
        max_cp: 753,
        max_durability: 20,
        max_progress: 4700,
        max_quality: 14900,
        base_progress: 310,
        base_quality: 324,
        job_level: 100,
        allowed_actions: ActionMask::regular(),
        adversarial: false,
        backload_progress: true,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 14900,
                steps: 22,
                duration: 56,
                overflow_quality: 439,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 584023,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 161594,
                dropped_nodes: 720978,
                pareto_buckets_squared_size_sum: 27623705,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 994029,
                sequential_states: 5457,
                pareto_values: 12431117,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 355665,
                pareto_values: 3142033,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}
