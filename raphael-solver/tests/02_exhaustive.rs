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
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 10623,
                steps: 26,
                duration: 70,
                overflow_quality: 0,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 321264,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 3720,
                dropped_nodes: 47101,
                pareto_buckets_squared_size_sum: 20840,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 946344,
                sequential_states: 45503,
                pareto_values: 23103012,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1196836,
                pareto_values: 19843366,
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
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 8912,
                steps: 21,
                duration: 55,
                overflow_quality: 0,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 298565,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 5246,
                dropped_nodes: 65584,
                pareto_buckets_squared_size_sum: 54474,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 808184,
                sequential_states: 46099,
                pareto_values: 17736698,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1227258,
                pareto_values: 20087913,
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
        backload_progress: false,
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
            finish_states: 273192,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 1020,
                dropped_nodes: 16380,
                pareto_buckets_squared_size_sum: 4700,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 1668344,
                sequential_states: 87012,
                pareto_values: 37644777,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 2736850,
                pareto_values: 41300056,
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
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 9688,
                steps: 25,
                duration: 68,
                overflow_quality: 0,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 522802,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 6058,
                dropped_nodes: 36110,
                pareto_buckets_squared_size_sum: 36854,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 877264,
                sequential_states: 46836,
                pareto_values: 23560138,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1312498,
                pareto_values: 24311098,
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
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 12793,
                steps: 27,
                duration: 72,
                overflow_quality: 0,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 244964,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 341,
                dropped_nodes: 5021,
                pareto_buckets_squared_size_sum: 559,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 960160,
                sequential_states: 45425,
                pareto_values: 23318404,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1062339,
                pareto_values: 17769091,
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
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 12000,
                steps: 21,
                duration: 56,
                overflow_quality: 123,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 2411529,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 1591900,
                dropped_nodes: 10803132,
                pareto_buckets_squared_size_sum: 64404105,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 932788,
                sequential_states: 3433,
                pareto_values: 24353529,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 2227636,
                pareto_values: 42541800,
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
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 11400,
                steps: 15,
                duration: 42,
                overflow_quality: 336,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 113261,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 160,
                dropped_nodes: 3297,
                pareto_buckets_squared_size_sum: 256,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 685374,
                sequential_states: 0,
                pareto_values: 12109118,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1414016,
                pareto_values: 20622466,
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
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 11400,
                steps: 15,
                duration: 42,
                overflow_quality: 336,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 135817,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 170,
                dropped_nodes: 4003,
                pareto_buckets_squared_size_sum: 272,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 1464188,
                sequential_states: 0,
                pareto_values: 24789603,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 3248970,
                pareto_values: 42114462,
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
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 11400,
                steps: 15,
                duration: 42,
                overflow_quality: 336,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 113262,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 160,
                dropped_nodes: 3422,
                pareto_buckets_squared_size_sum: 256,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 1405659,
                sequential_states: 0,
                pareto_values: 25010269,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 2932622,
                pareto_values: 42760190,
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
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 6500,
                steps: 16,
                duration: 43,
                overflow_quality: 369,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 1224118,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 22404,
                dropped_nodes: 359646,
                pareto_buckets_squared_size_sum: 187676,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 709548,
                sequential_states: 0,
                pareto_values: 13383321,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1514211,
                pareto_values: 21948277,
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
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 5500,
                steps: 11,
                duration: 29,
                overflow_quality: 302,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 49606,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 1739,
                dropped_nodes: 26431,
                pareto_buckets_squared_size_sum: 15755,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 414395,
                sequential_states: 0,
                pareto_values: 3311964,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 161906,
                pareto_values: 1332463,
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
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 11000,
                steps: 13,
                duration: 35,
                overflow_quality: 627,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 127733,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 8296,
                dropped_nodes: 161278,
                pareto_buckets_squared_size_sum: 79064,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 588992,
                sequential_states: 0,
                pareto_values: 5653305,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 457074,
                pareto_values: 4024634,
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
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 6000,
                steps: 14,
                duration: 40,
                overflow_quality: 455,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 281651,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 74,
                dropped_nodes: 1486,
                pareto_buckets_squared_size_sum: 90,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 615546,
                sequential_states: 0,
                pareto_values: 9634708,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1229915,
                pareto_values: 15572440,
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
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 5400,
                steps: 14,
                duration: 38,
                overflow_quality: 638,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 690212,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 7153,
                dropped_nodes: 144383,
                pareto_buckets_squared_size_sum: 27336,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 579131,
                sequential_states: 0,
                pareto_values: 8283565,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1108933,
                pareto_values: 13018517,
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
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 8250,
                steps: 17,
                duration: 46,
                overflow_quality: 339,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 870640,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 7820,
                dropped_nodes: 155839,
                pareto_buckets_squared_size_sum: 37628,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 812878,
                sequential_states: 0,
                pareto_values: 18128867,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1517881,
                pareto_values: 24731805,
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
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 14900,
                steps: 21,
                duration: 53,
                overflow_quality: 439,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 867357,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 1889237,
                dropped_nodes: 13613015,
                pareto_buckets_squared_size_sum: 142776403,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 969126,
                sequential_states: 1305,
                pareto_values: 17429437,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 386898,
                pareto_values: 5750109,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn hardened_survey_plank_5558_5216_heart_and_soul_quick_innovation() {
    let simulator_settings = Settings {
        max_cp: 500,
        max_durability: 20,
        max_progress: 4700,
        max_quality: 14900,
        base_progress: 310,
        base_quality: 324,
        job_level: 100,
        allowed_actions: ActionMask::all().remove(Action::TrainedEye),
        adversarial: false,
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 11378,
                steps: 23,
                duration: 63,
                overflow_quality: 0,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 270612,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 28159,
                dropped_nodes: 316039,
                pareto_buckets_squared_size_sum: 310984,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 2397570,
                sequential_states: 106175,
                pareto_values: 38834009,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 1775230,
                pareto_values: 24477318,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn ceviche_4900_4800_no_quality() {
    // https://github.com/KonaeAkira/raphael-rs/issues/149
    let simulator_settings = Settings {
        max_cp: 620,
        max_durability: 70,
        max_progress: 8050,
        max_quality: 0, // 0% quality target
        base_progress: 261,
        base_quality: 266,
        job_level: 100,
        allowed_actions: ActionMask::regular(),
        adversarial: false,
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 0,
                steps: 8,
                duration: 22,
                overflow_quality: 0,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 517559,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 29,
                dropped_nodes: 552,
                pareto_buckets_squared_size_sum: 31,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 428868,
                sequential_states: 0,
                pareto_values: 428868,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 664576,
                pareto_values: 664576,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn ce_high_progress_zero_achieved_quality() {
    // Researcher's Water-resistant Leather
    // 5386/5425/628/100 + HQ Rroneek Steak
    let simulator_settings = Settings {
        max_cp: 720,
        max_durability: 25,
        max_progress: 19800,
        max_quality: 1100,
        base_progress: 286,
        base_quality: 293,
        job_level: 100,
        allowed_actions: ActionMask::regular(),
        adversarial: false,
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 0,
                steps: 30,
                duration: 80,
                overflow_quality: 0,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 1751541,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 397,
                dropped_nodes: 0,
                pareto_buckets_squared_size_sum: 635,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 951755,
                sequential_states: 0,
                pareto_values: 3925107,
            },
            step_lb_stats: StepLbSolverStats {
                parallel_states: 861822,
                pareto_values: 3119837,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}
