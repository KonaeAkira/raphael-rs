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
                processed_nodes: 456,
                dropped_nodes: 4506,
                pareto_buckets_squared_size_sum: 2538,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 957930,
                sequential_states: 42813,
                pareto_values: 17006469,
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
            finish_states: 267200,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 4454,
                dropped_nodes: 32686,
                pareto_buckets_squared_size_sum: 241837,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 818130,
                sequential_states: 44927,
                pareto_values: 13132095,
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
                processed_nodes: 571,
                dropped_nodes: 6571,
                pareto_buckets_squared_size_sum: 5691,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 1690660,
                sequential_states: 84617,
                pareto_values: 27637851,
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
                processed_nodes: 1578,
                dropped_nodes: 4978,
                pareto_buckets_squared_size_sum: 37264,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 888030,
                sequential_states: 44397,
                pareto_values: 17385016,
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
            finish_states: 345769,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 3222,
                dropped_nodes: 18224,
                pareto_buckets_squared_size_sum: 96982,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 971910,
                sequential_states: 45972,
                pareto_values: 17456807,
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
            finish_states: 1190393,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 61930,
                dropped_nodes: 250888,
                pareto_buckets_squared_size_sum: 7195828,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 945752,
                sequential_states: 10702,
                pareto_values: 18106329,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 2318275,
                sequential_states: 0,
                pareto_values: 24996627,
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
            finish_states: 353410,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 3115,
                dropped_nodes: 47363,
                pareto_buckets_squared_size_sum: 42015,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 727703,
                sequential_states: 0,
                pareto_values: 9890579,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1493595,
                sequential_states: 0,
                pareto_values: 12299858,
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
            finish_states: 478840,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 3643,
                dropped_nodes: 62771,
                pareto_buckets_squared_size_sum: 48356,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 1547790,
                sequential_states: 0,
                pareto_values: 20676360,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 3361396,
                sequential_states: 0,
                pareto_values: 26035232,
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
            finish_states: 353411,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 3115,
                dropped_nodes: 48533,
                pareto_buckets_squared_size_sum: 42015,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 1477069,
                sequential_states: 0,
                pareto_values: 20284215,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 3071733,
                sequential_states: 0,
                pareto_values: 25567704,
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
                pareto_buckets_squared_size_sum: 783,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 750109,
                sequential_states: 0,
                pareto_values: 11255498,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1579065,
                sequential_states: 0,
                pareto_values: 13935076,
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
            finish_states: 131819,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 8093,
                dropped_nodes: 97927,
                pareto_buckets_squared_size_sum: 458666,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 448545,
                sequential_states: 20,
                pareto_values: 3013677,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 168003,
                sequential_states: 0,
                pareto_values: 928501,
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
            finish_states: 308214,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 26389,
                dropped_nodes: 407583,
                pareto_buckets_squared_size_sum: 2536899,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 632525,
                sequential_states: 39,
                pareto_values: 4877078,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 461630,
                sequential_states: 0,
                pareto_values: 2798356,
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
            finish_states: 395101,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 349,
                dropped_nodes: 4773,
                pareto_buckets_squared_size_sum: 889,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 654113,
                sequential_states: 0,
                pareto_values: 8411272,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1336246,
                sequential_states: 0,
                pareto_values: 10026036,
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
            finish_states: 457339,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 1194,
                dropped_nodes: 17919,
                pareto_buckets_squared_size_sum: 5727,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 620174,
                sequential_states: 0,
                pareto_values: 7334997,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1118641,
                sequential_states: 0,
                pareto_values: 8025952,
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
            finish_states: 1169189,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 16259,
                dropped_nodes: 246089,
                pareto_buckets_squared_size_sum: 610705,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 871349,
                sequential_states: 0,
                pareto_values: 14549573,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1556489,
                sequential_states: 0,
                pareto_values: 14722811,
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
            finish_states: 574570,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 151237,
                dropped_nodes: 672402,
                pareto_buckets_squared_size_sum: 35948055,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 994029,
                sequential_states: 5093,
                pareto_values: 12428188,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 361555,
                sequential_states: 0,
                pareto_values: 3147923,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}
