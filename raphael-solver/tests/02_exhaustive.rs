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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
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
            finish_states: 88341,
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 51746,
                processed_nodes: 3719,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 946344,
                states_on_shards: 45503,
                values: 22751757,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 631128,
                states_on_shards: 0,
                values: 11674974,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
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
            finish_states: 81558,
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 72819,
                processed_nodes: 5240,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 808184,
                states_on_shards: 46099,
                values: 17736621,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 631128,
                states_on_shards: 0,
                values: 11652491,
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
            finish_states: 142854,
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 17646,
                processed_nodes: 1021,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 1668344,
                states_on_shards: 87012,
                values: 37644777,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 1178831,
                states_on_shards: 0,
                values: 23063578,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
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
            finish_states: 125704,
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 43698,
                processed_nodes: 6044,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 877264,
                states_on_shards: 46836,
                values: 23497998,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 693970,
                states_on_shards: 0,
                values: 14559312,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
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
            finish_states: 102529,
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 5408,
                processed_nodes: 342,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 960160,
                states_on_shards: 45425,
                values: 21397668,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 505459,
                states_on_shards: 0,
                values: 9590744,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: false,
    };
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
            finish_states: 308710,
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 13407968,
                processed_nodes: 1597673,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 932788,
                states_on_shards: 6315,
                values: 22061921,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 878246,
                states_on_shards: 362445,
                values: 26206132,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
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
            finish_states: 210965,
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 3886,
                processed_nodes: 180,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 685374,
                states_on_shards: 0,
                values: 10568081,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 366835,
                states_on_shards: 155845,
                values: 8852280,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
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
            finish_states: 392377,
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 4699,
                processed_nodes: 192,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 1464188,
                states_on_shards: 0,
                values: 24789603,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 606763,
                states_on_shards: 393761,
                values: 19074563,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
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
            finish_states: 210965,
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 4013,
                processed_nodes: 180,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 1405659,
                states_on_shards: 0,
                values: 21872304,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 378435,
                states_on_shards: 159328,
                values: 9141198,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
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
            finish_states: 386830,
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 376873,
                processed_nodes: 21476,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 709548,
                states_on_shards: 0,
                values: 11347845,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 482739,
                states_on_shards: 171952,
                values: 10811806,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
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
            finish_states: 18046,
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 37315,
                processed_nodes: 2226,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 414395,
                states_on_shards: 0,
                values: 2776768,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 36767,
                states_on_shards: 30175,
                values: 556755,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
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
            finish_states: 25429,
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 183400,
                processed_nodes: 8697,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 588992,
                states_on_shards: 0,
                values: 5111464,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 136211,
                states_on_shards: 72730,
                values: 1955153,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
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
            finish_states: 315896,
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 2314,
                processed_nodes: 109,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 615546,
                states_on_shards: 0,
                values: 8053602,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 266926,
                states_on_shards: 147362,
                values: 6060588,
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
                overflow_quality: 638,
            },
        )
    "#]];
    let expected_runtime_stats = expect![[r#"
        MacroSolverStats {
            finish_states: 304851,
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 171861,
                processed_nodes: 7939,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 579131,
                states_on_shards: 0,
                values: 6864670,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 266926,
                states_on_shards: 143595,
                values: 5457006,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
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
            finish_states: 379883,
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 209791,
                processed_nodes: 9797,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 812878,
                states_on_shards: 0,
                values: 15607520,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 569044,
                states_on_shards: 160051,
                values: 13406303,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: false,
    };
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
            finish_states: 37677,
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 18326484,
                processed_nodes: 2024194,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 969126,
                states_on_shards: 4229,
                values: 16039954,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 196205,
                states_on_shards: 67110,
                values: 4081029,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
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
            finish_states: 63256,
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 358137,
                processed_nodes: 28100,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 2397570,
                states_on_shards: 106158,
                values: 38833493,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 370080,
                states_on_shards: 0,
                values: 6335993,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
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
            finish_states: 446655,
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 575,
                processed_nodes: 30,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 428868,
                states_on_shards: 0,
                values: 428868,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 13059,
                states_on_shards: 47248,
                values: 60307,
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
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
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
            finish_states: 535900,
            search_queue_stats: SearchQueueStats {
                inserted_nodes: 663,
                processed_nodes: 398,
            },
            quality_ub_stats: QualityUbSolverStats {
                states_on_main: 951755,
                states_on_shards: 0,
                values: 2232866,
            },
            step_lb_stats: StepLbSolverStats {
                states_on_main: 450021,
                states_on_shards: 0,
                values: 1030226,
            },
        }
    "#]];
    test_with_settings(solver_settings, expected_score, expected_runtime_stats);
}
