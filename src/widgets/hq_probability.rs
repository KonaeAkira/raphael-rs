use std::collections::HashMap;

use raphael_data::hq_percentage;
use raphael_sim::*;

const DEFAULT_CONDITION_PROBABILITIES: [(Condition, u8); 3] = [
    (Condition::Normal, 80),
    (Condition::Good, 15),
    (Condition::Excellent, 5),
];

pub struct HqDistributionWidget {
    simulator_settings: Settings,
    actions: Box<[Action]>,
}

impl HqDistributionWidget {
    pub fn new(mut simulator_settings: Settings, actions: Box<[Action]>) -> Self {
        simulator_settings.adversarial = false;
        Self {
            simulator_settings,
            actions,
        }
    }
}

impl egui::Widget for HqDistributionWidget {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let solve_args = HqDistributionSolveArgs {
            simulator_settings: self.simulator_settings,
            condition_probabilities: DEFAULT_CONDITION_PROBABILITIES.into(),
            actions: self.actions,
        };

        type HqDistributionSolverCache =
            egui::util::cache::FrameCache<HqDistribution, HqDistributionSolver>;
        let hq_distribution = ui.ctx().memory_mut(|mem| {
            mem.caches
                .cache::<HqDistributionSolverCache>()
                .get(&solve_args)
        });

        ui.label(format!(
            "avg. {:.4}% HQ ({}% - {}%)",
            hq_distribution.avg_hq_percentage,
            hq_distribution.min_hq_percentage,
            hq_distribution.max_hq_percentage,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct HqDistributionSolveArgs {
    simulator_settings: Settings,
    condition_probabilities: Box<[(Condition, u8)]>, // whole percentage points
    actions: Box<[Action]>,
}

#[derive(Default)]
struct HqDistributionSolver {}

impl egui::util::cache::ComputerMut<&HqDistributionSolveArgs, HqDistribution>
    for HqDistributionSolver
{
    fn compute(&mut self, solve_args: &HqDistributionSolveArgs) -> HqDistribution {
        let initial_state = SimulationState::new(&solve_args.simulator_settings);
        let mut quality_distribution = solve(
            solve_args,
            &mut HashMap::default(),
            initial_state,
            Condition::Normal,
            0,
        );
        // The returned distribution contains the distribution of additional quality gained, so
        // to get the total quality we need to add the starting quality.
        quality_distribution.add_quality(initial_state.quality);
        quality_distribution.hq_percentage_distribution(&solve_args.simulator_settings)
    }
}

fn solve(
    solve_args: &HqDistributionSolveArgs,
    memoization: &mut HashMap<(SimulationState, Condition, usize), QualityDistribution>,
    state: SimulationState,
    condition: Condition,
    step: usize,
) -> QualityDistribution {
    let state = normalize_state(state);
    if let Some(distribution) = memoization.get(&(state, condition, step)) {
        return distribution.clone();
    }

    let next_condition_probabilities = match condition {
        Condition::Normal => solve_args.condition_probabilities.as_ref(),
        Condition::Good | Condition::Poor => &[(Condition::Normal, 100)],
        Condition::Excellent => &[(Condition::Poor, 100)],
    };

    let mut distribution = QualityDistribution::empty();
    for &(condition, condition_probability) in next_condition_probabilities {
        let action_result = state.use_action(
            solve_args.actions[step],
            condition,
            &solve_args.simulator_settings,
        );
        let next_state = match action_result {
            Ok(next_state) => next_state,
            Err(_) => state,
        };
        let action_quality = next_state.quality;
        let mut next_distribution = if step + 1 >= solve_args.actions.len() {
            if next_state.progress >= u32::from(solve_args.simulator_settings.max_progress) {
                QualityDistribution::zero()
            } else {
                QualityDistribution::empty()
            }
        } else {
            solve(solve_args, memoization, next_state, condition, step + 1)
        };
        next_distribution.add_quality(action_quality);
        next_distribution.mul_probability(f32::from(condition_probability) / 100.0);
        distribution = QualityDistribution::combine(distribution, next_distribution);
    }
    memoization.insert((state, condition, step), distribution.clone());
    distribution
}

fn normalize_state(mut state: SimulationState) -> SimulationState {
    state.quality = 0;
    state.unreliable_quality = 0;
    state
}

#[derive(Debug, Clone, Copy)]
struct HqDistribution {
    min_hq_percentage: u8,
    max_hq_percentage: u8,
    avg_hq_percentage: f32,
}

#[derive(Debug, Clone)]
struct QualityDistribution {
    data_points: Vec<(u32, f32)>,
}

impl QualityDistribution {
    /// Creates a new quality distribution with no data points.
    fn empty() -> Self {
        Self {
            data_points: Vec::new(),
        }
    }

    /// Creates a new quality distrubtion with 100% chance of 0 quality.
    fn zero() -> Self {
        Self {
            data_points: vec![(0, 1.0)],
        }
    }

    fn add_quality(&mut self, q: u32) {
        for (quality, _) in &mut self.data_points {
            *quality = quality.saturating_add(q);
        }
    }

    fn mul_probability(&mut self, p: f32) {
        for (_, probability) in &mut self.data_points {
            *probability *= p;
        }
    }

    fn combine(lhs: Self, rhs: Self) -> Self {
        let mut new_data_points = Vec::new();
        let mut lhs_iter = lhs.data_points.into_iter().peekable();
        let mut rhs_iter = rhs.data_points.into_iter().peekable();
        while let (Some(lhs), Some(rhs)) = (lhs_iter.peek(), rhs_iter.peek()) {
            match lhs.0.cmp(&rhs.0) {
                std::cmp::Ordering::Less => {
                    new_data_points.push(*lhs);
                    lhs_iter.next();
                }
                std::cmp::Ordering::Equal => {
                    new_data_points.push((lhs.0, lhs.1 + rhs.1));
                    lhs_iter.next();
                    rhs_iter.next();
                }
                std::cmp::Ordering::Greater => {
                    new_data_points.push(*rhs);
                    rhs_iter.next();
                }
            }
        }
        new_data_points.extend(lhs_iter);
        new_data_points.extend(rhs_iter);
        Self {
            data_points: new_data_points,
        }
    }

    fn hq_percentage_distribution(&self, simulator_settings: &Settings) -> HqDistribution {
        let mut distribution = [0.0; 101];
        for &(quality, probability) in &self.data_points {
            let hq_percentage = hq_percentage(quality, simulator_settings.max_quality).unwrap_or(0);
            distribution[usize::from(hq_percentage)] += probability;
        }
        let min_hq_percentage = distribution.iter().position(|&p| p > 0.0).unwrap_or(0) as u8;
        let max_hq_percentage = distribution.iter().rposition(|&p| p > 0.0).unwrap_or(0) as u8;
        let mut avg_hq_percentage = 0.0;
        for i in 0..101 {
            avg_hq_percentage += i as f32 * distribution[i];
        }
        HqDistribution {
            min_hq_percentage,
            max_hq_percentage,
            avg_hq_percentage: avg_hq_percentage.min(100.0),
        }
    }
}
