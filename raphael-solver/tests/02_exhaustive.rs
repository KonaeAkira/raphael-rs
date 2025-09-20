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
    let score = result.map(|actions| {
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
        Ok(
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
            finish_states: 321209,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 3718,
                dropped_nodes: 47099,
                pareto_buckets_squared_size_sum: 82625,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 946344,
                sequential_states: 45503,
                pareto_values: 22751757,
            },
            step_lb_stats: StepLbSolverStats {
                states: 0,
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
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Ok(
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
            finish_states: 298477,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 5239,
                dropped_nodes: 65490,
                pareto_buckets_squared_size_sum: 220159,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 808184,
                sequential_states: 46099,
                pareto_values: 17736621,
            },
            step_lb_stats: StepLbSolverStats {
                states: 0,
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
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Ok(
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
                pareto_buckets_squared_size_sum: 19732,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 1668344,
                sequential_states: 87012,
                pareto_values: 37644777,
            },
            step_lb_stats: StepLbSolverStats {
                states: 0,
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
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Ok(
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
            finish_states: 522737,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 6043,
                dropped_nodes: 36050,
                pareto_buckets_squared_size_sum: 147900,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 877264,
                sequential_states: 46836,
                pareto_values: 23497998,
            },
            step_lb_stats: StepLbSolverStats {
                states: 0,
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
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Ok(
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
                pareto_buckets_squared_size_sum: 1113,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 960160,
                sequential_states: 45425,
                pareto_values: 21397668,
            },
            step_lb_stats: StepLbSolverStats {
                states: 0,
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
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Ok(
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
            finish_states: 2386785,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 1463009,
                dropped_nodes: 9964883,
                pareto_buckets_squared_size_sum: 441665830,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 932788,
                sequential_states: 6186,
                pareto_values: 22061401,
            },
            step_lb_stats: StepLbSolverStats {
                states: 1750683,
                pareto_values: 35133297,
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
        Ok(
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
            finish_states: 113181,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 158,
                dropped_nodes: 3253,
                pareto_buckets_squared_size_sum: 522,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 685374,
                sequential_states: 0,
                pareto_values: 10568081,
            },
            step_lb_stats: StepLbSolverStats {
                states: 716016,
                pareto_values: 12192698,
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
        Ok(
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
            finish_states: 135715,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 168,
                dropped_nodes: 3953,
                pareto_buckets_squared_size_sum: 562,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 1464188,
                sequential_states: 0,
                pareto_values: 24789603,
            },
            step_lb_stats: StepLbSolverStats {
                states: 1386414,
                pareto_values: 26436486,
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
        Ok(
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
            finish_states: 113182,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 158,
                dropped_nodes: 3376,
                pareto_buckets_squared_size_sum: 522,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 1405659,
                sequential_states: 0,
                pareto_values: 21872304,
            },
            step_lb_stats: StepLbSolverStats {
                states: 1414156,
                pareto_values: 24372058,
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
        Ok(
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
            finish_states: 1209636,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 20524,
                dropped_nodes: 330165,
                pareto_buckets_squared_size_sum: 789821,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 709548,
                sequential_states: 0,
                pareto_values: 11347845,
            },
            step_lb_stats: StepLbSolverStats {
                states: 902038,
                pareto_values: 14638942,
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
        Ok(
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
            finish_states: 49527,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 1727,
                dropped_nodes: 26236,
                pareto_buckets_squared_size_sum: 35394,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 414395,
                sequential_states: 0,
                pareto_values: 2776768,
            },
            step_lb_stats: StepLbSolverStats {
                states: 78431,
                pareto_values: 665881,
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
        Ok(
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
            finish_states: 121980,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 7804,
                dropped_nodes: 151505,
                pareto_buckets_squared_size_sum: 194855,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 588992,
                sequential_states: 0,
                pareto_values: 5111464,
            },
            step_lb_stats: StepLbSolverStats {
                states: 269309,
                pareto_values: 2610911,
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
        Ok(
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
                pareto_buckets_squared_size_sum: 122,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 615546,
                sequential_states: 0,
                pareto_values: 8053602,
            },
            step_lb_stats: StepLbSolverStats {
                states: 575636,
                pareto_values: 8434830,
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
        Ok(
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
            finish_states: 666973,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 6369,
                dropped_nodes: 128343,
                pareto_buckets_squared_size_sum: 90959,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 579131,
                sequential_states: 0,
                pareto_values: 6864670,
            },
            step_lb_stats: StepLbSolverStats {
                states: 550832,
                pareto_values: 7352920,
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
        Ok(
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
            finish_states: 865500,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 7627,
                dropped_nodes: 151949,
                pareto_buckets_squared_size_sum: 110095,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 812878,
                sequential_states: 0,
                pareto_values: 15607520,
            },
            step_lb_stats: StepLbSolverStats {
                states: 998131,
                pareto_values: 17952228,
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
        Ok(
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
            finish_states: 859311,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 1778459,
                dropped_nodes: 12974129,
                pareto_buckets_squared_size_sum: 434154578,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 969126,
                sequential_states: 4016,
                pareto_values: 16038900,
            },
            step_lb_stats: StepLbSolverStats {
                states: 341307,
                pareto_values: 4977992,
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
        Ok(
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
            finish_states: 270368,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 28099,
                dropped_nodes: 315380,
                pareto_buckets_squared_size_sum: 586714,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 2397570,
                sequential_states: 106158,
                pareto_values: 38833493,
            },
            step_lb_stats: StepLbSolverStats {
                states: 0,
                pareto_values: 0,
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
        Ok(
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
                pareto_buckets_squared_size_sum: 43,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 428868,
                sequential_states: 0,
                pareto_values: 428868,
            },
            step_lb_stats: StepLbSolverStats {
                states: 76276,
                pareto_values: 76276,
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
        Ok(
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
                pareto_buckets_squared_size_sum: 1033,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 951755,
                sequential_states: 0,
                pareto_values: 2232866,
            },
            step_lb_stats: StepLbSolverStats {
                states: 0,
                pareto_values: 0,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}
