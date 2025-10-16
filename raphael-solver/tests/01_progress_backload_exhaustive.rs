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
        backload_progress: true,
    };
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
    let expected_score = expect![[r#"
        Ok(
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
            finish_states: 212167,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 456,
                dropped_nodes: 4523,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 957930,
                sequential_states: 42813,
                pareto_values: 16783500,
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
        backload_progress: true,
    };
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
    let expected_score = expect![[r#"
        Ok(
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
            finish_states: 267531,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 4630,
                dropped_nodes: 34331,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 818130,
                sequential_states: 44933,
                pareto_values: 13132176,
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
        backload_progress: true,
    };
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
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
            finish_states: 248396,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 577,
                dropped_nodes: 6649,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 1690660,
                sequential_states: 84617,
                pareto_values: 27637851,
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
        backload_progress: true,
    };
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
    let expected_score = expect![[r#"
        Ok(
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
            finish_states: 426778,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 1626,
                dropped_nodes: 5140,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 888030,
                sequential_states: 44397,
                pareto_values: 17344815,
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
        backload_progress: true,
    };
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
    let expected_score = expect![[r#"
        Ok(
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
            finish_states: 346005,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 3359,
                dropped_nodes: 19200,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 971910,
                sequential_states: 45972,
                pareto_values: 16209295,
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
        backload_progress: true,
    };
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: false,
    };
    let expected_score = expect![[r#"
        Ok(
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
            finish_states: 1192691,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 64249,
                dropped_nodes: 258813,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 945752,
                sequential_states: 12499,
                pareto_values: 16494243,
            },
            step_lb_stats: StepLbSolverStats {
                states: 1153291,
                pareto_values: 12957118,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
    let expected_score = expect![[r#"
        Ok(
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
            finish_states: 332904,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 2824,
                dropped_nodes: 42948,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 727703,
                sequential_states: 0,
                pareto_values: 8685113,
            },
            step_lb_stats: StepLbSolverStats {
                states: 568886,
                pareto_values: 5321421,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
    let expected_score = expect![[r#"
        Ok(
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
            finish_states: 448794,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 3248,
                dropped_nodes: 55897,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 1547790,
                sequential_states: 0,
                pareto_values: 20676360,
            },
            step_lb_stats: StepLbSolverStats {
                states: 1043176,
                pareto_values: 11604155,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
    let expected_score = expect![[r#"
        Ok(
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
            finish_states: 332905,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 2824,
                dropped_nodes: 43945,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 1477069,
                sequential_states: 0,
                pareto_values: 17827198,
            },
            step_lb_stats: StepLbSolverStats {
                states: 1062965,
                pareto_values: 10406670,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
    let expected_score = expect![[r#"
        Ok(
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
            finish_states: 592766,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 260,
                dropped_nodes: 3188,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 750109,
                sequential_states: 1,
                pareto_values: 9640072,
            },
            step_lb_stats: StepLbSolverStats {
                states: 560914,
                pareto_values: 5598938,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
    let expected_score = expect![[r#"
        Ok(
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
            finish_states: 127127,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 8158,
                dropped_nodes: 100430,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 448545,
                sequential_states: 20,
                pareto_values: 2531249,
            },
            step_lb_stats: StepLbSolverStats {
                states: 99751,
                pareto_values: 543891,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
    let expected_score = expect![[r#"
        Ok(
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
            finish_states: 297239,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 25196,
                dropped_nodes: 393113,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 632525,
                sequential_states: 39,
                pareto_values: 4421953,
            },
            step_lb_stats: StepLbSolverStats {
                states: 257154,
                pareto_values: 1637498,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
    let expected_score = expect![[r#"
        Ok(
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
            finish_states: 373265,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 306,
                dropped_nodes: 4190,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 654113,
                sequential_states: 0,
                pareto_values: 7088630,
            },
            step_lb_stats: StepLbSolverStats {
                states: 449455,
                pareto_values: 3802071,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
    let expected_score = expect![[r#"
        Ok(
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
            finish_states: 421555,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 955,
                dropped_nodes: 14218,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 620174,
                sequential_states: 0,
                pareto_values: 6152904,
            },
            step_lb_stats: StepLbSolverStats {
                states: 335609,
                pareto_values: 2765482,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
    let expected_score = expect![[r#"
        Ok(
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
            finish_states: 1141195,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 15021,
                dropped_nodes: 229298,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 871349,
                sequential_states: 0,
                pareto_values: 12661815,
            },
            step_lb_stats: StepLbSolverStats {
                states: 753949,
                pareto_values: 7397099,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
    let expected_score = expect![[r#"
        Ok(
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
            finish_states: 582393,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 169988,
                dropped_nodes: 761577,
            },
            quality_ub_stats: QualityUbSolverStats {
                parallel_states: 994029,
                sequential_states: 9767,
                pareto_values: 11580525,
            },
            step_lb_stats: StepLbSolverStats {
                states: 319667,
                pareto_values: 2762461,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}
