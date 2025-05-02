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
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
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
            finish_states: 285758,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 1159,
                dropped_nodes: 1011,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 1982938,
                pareto_values: 45486067,
            },
            step_lb_stats: StepLbSolverStats {
                states: 795538,
                pareto_values: 15316958,
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
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
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
            finish_states: 291621,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 3023,
                dropped_nodes: 1529,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 1707402,
                pareto_values: 34644852,
            },
            step_lb_stats: StepLbSolverStats {
                states: 829035,
                pareto_values: 15439769,
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
            finish_states: 213985,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 219,
                dropped_nodes: 0,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 3455899,
                pareto_values: 73184655,
            },
            step_lb_stats: StepLbSolverStats {
                states: 1582899,
                pareto_values: 29777152,
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
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
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
            finish_states: 463963,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 2022,
                dropped_nodes: 1038,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 1847123,
                pareto_values: 46078953,
            },
            step_lb_stats: StepLbSolverStats {
                states: 958195,
                pareto_values: 20582553,
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
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
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
            finish_states: 365439,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 230,
                dropped_nodes: 812,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 2008872,
                pareto_values: 46441910,
            },
            step_lb_stats: StepLbSolverStats {
                states: 675136,
                pareto_values: 13435389,
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
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
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
            finish_states: 2235346,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 1222641,
                dropped_nodes: 7658550,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 1901560,
                pareto_values: 48784449,
            },
            step_lb_stats: StepLbSolverStats {
                states: 1622394,
                pareto_values: 35839860,
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
        max_quality: 40000,
        base_progress: 289,
        base_quality: 360,
        job_level: 100,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    let expected_score = expect![[r#"
        Some(
            SolutionScore {
                capped_quality: 20177,
                steps: 31,
                duration: 85,
                overflow_quality: 0,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 832985,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 328812,
                dropped_nodes: 1313016,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 1902638,
                pareto_values: 48583224,
            },
            step_lb_stats: StepLbSolverStats {
                states: 2132960,
                pareto_values: 49259139,
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
        max_quality: 40000,
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
                capped_quality: 21536,
                steps: 30,
                duration: 83,
                overflow_quality: 0,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 745899,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 61079,
                dropped_nodes: 182960,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 3867407,
                pareto_values: 101951274,
            },
            step_lb_stats: StepLbSolverStats {
                states: 4208974,
                pareto_values: 97339814,
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
        max_quality: 40000,
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
                capped_quality: 20502,
                steps: 28,
                duration: 77,
                overflow_quality: 0,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 868507,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 603602,
                dropped_nodes: 2876614,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 3944524,
                pareto_values: 100829366,
            },
            step_lb_stats: StepLbSolverStats {
                states: 4335937,
                pareto_values: 100272946,
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
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
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
            finish_states: 1210688,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 18982,
                dropped_nodes: 277204,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 1525676,
                pareto_values: 27015425,
            },
            step_lb_stats: StepLbSolverStats {
                states: 749510,
                pareto_values: 13760404,
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
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
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
            finish_states: 44568,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 1443,
                dropped_nodes: 21507,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 1344217,
                pareto_values: 7815321,
            },
            step_lb_stats: StepLbSolverStats {
                states: 84652,
                pareto_values: 850801,
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
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
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
            finish_states: 106424,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 7115,
                dropped_nodes: 139169,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 1522551,
                pareto_values: 12435109,
            },
            step_lb_stats: StepLbSolverStats {
                states: 225872,
                pareto_values: 2235160,
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
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
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
            finish_states: 285458,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 71,
                dropped_nodes: 1434,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 1694421,
                pareto_values: 20248583,
            },
            step_lb_stats: StepLbSolverStats {
                states: 331102,
                pareto_values: 5695344,
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
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
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
            finish_states: 664547,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 6149,
                dropped_nodes: 117817,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 1711194,
                pareto_values: 17518103,
            },
            step_lb_stats: StepLbSolverStats {
                states: 433569,
                pareto_values: 6585102,
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
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
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
            finish_states: 862184,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 7124,
                dropped_nodes: 141310,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 2107358,
                pareto_values: 37993603,
            },
            step_lb_stats: StepLbSolverStats {
                states: 794894,
                pareto_values: 15973671,
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
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
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
            finish_states: 864640,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 1855660,
                dropped_nodes: 12311239,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 2041117,
                pareto_values: 34763925,
            },
            step_lb_stats: StepLbSolverStats {
                states: 343345,
                pareto_values: 5335105,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}

#[test]
fn ceviche_4900_4800_no_quality() {
    let simulator_settings = Settings {
        max_cp: 620,
        max_durability: 70,
        max_progress: 8050,
        max_quality: 0, // 0% quality target
        base_progress: 261,
        base_quality: 266,
        job_level: 100,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
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
            finish_states: 1577600,
            search_queue_stats: SearchQueueStats {
                processed_nodes: 106671,
                dropped_nodes: 445477,
            },
            quality_ub_stats: QualityUbSolverStats {
                states: 24753,
                pareto_values: 24713,
            },
            step_lb_stats: StepLbSolverStats {
                states: 52,
                pareto_values: 52,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}
