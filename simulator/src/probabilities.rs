use std::collections::HashMap;

use crate::{Action, Condition, Settings, SimulationState};

const fn condition_probabilities(current_condition: Condition) -> &'static [(Condition, f32)] {
    match current_condition {
        Condition::Normal => &[
            (Condition::Normal, 0.86),
            (Condition::Good, 0.12),
            (Condition::Excellent, 0.02),
        ],
        Condition::Good => &[(Condition::Normal, 1.00)],
        Condition::Excellent => &[(Condition::Poor, 1.00)],
        Condition::Poor => &[(Condition::Normal, 1.00)],
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Value {
    pub quality: u16,
    pub probability: f32,
}

impl Value {
    const fn zero() -> Self {
        Self {
            quality: 0,
            probability: 1.0,
        }
    }
}

pub struct QualityDistribution {
    distribution: Vec<Value>,
}

impl QualityDistribution {
    fn zero() -> Self {
        Self {
            distribution: vec![Value::zero()],
        }
    }
}

impl IntoIterator for QualityDistribution {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.distribution.into_iter()
    }
}

impl QualityDistribution {
    pub fn at_least(&self, quality: u16) -> f32 {
        let mut result = 0.0;
        for value in &self.distribution {
            if value.quality >= quality {
                result += value.probability;
            }
        }
        result.clamp(0.0, 1.0)
    }

    pub fn exactly(&self, quality: u16) -> f32 {
        let mut result = 0.0;
        for value in &self.distribution {
            if value.quality == quality {
                result += value.probability;
            }
        }
        result.clamp(0.0, 1.0)
    }
}

pub fn quality_probability_distribution(
    settings: Settings,
    actions: impl Into<Box<[Action]>>,
    initial_quality: u16,
) -> QualityDistribution {
    let initial_state = SimulationState::new(&settings);
    let actions: Box<[Action]> = actions.into();
    match actions.len() {
        0 => QualityDistribution::zero(),
        _ => {
            let mut solver = QualityDistributionSolver::new(settings, actions);
            solver.solve(initial_state, Condition::Normal, 0);
            let distribution = solver
                .memoization
                .get(&(initial_state, Condition::Normal, 0))
                .expect("State not in memoization even after solving")
                .iter()
                .map(|value| Value {
                    quality: initial_quality + value.quality,
                    probability: value.probability,
                });
            QualityDistribution {
                distribution: distribution.collect(),
            }
        }
    }
}

struct QualityDistributionSolver {
    settings: Settings,
    actions: Box<[Action]>,
    memoization: std::collections::HashMap<(SimulationState, Condition, usize), Vec<Value>>,
}

impl QualityDistributionSolver {
    const ZERO_DISTRIBUTION: &[Value] = &[Value::zero()];
    const EMPTY_DISTRIBUTION: &[Value] = &[];

    fn new(settings: Settings, actions: Box<[Action]>) -> Self {
        Self {
            settings: Settings {
                adversarial: false,
                ..settings
            },
            actions,
            memoization: HashMap::default(),
        }
    }

    fn solve(&mut self, state: SimulationState, condition: Condition, step: usize) {
        let state = Self::normalize_state(state);
        if self.memoization.contains_key(&(state, condition, step)) {
            return;
        }
        let mut distribution = Vec::new();
        for (condition, condition_probability) in condition_probabilities(condition) {
            let action_result = state.use_action(self.actions[step], *condition, &self.settings);
            let next_state = action_result.unwrap_or(state);
            let action_quality = next_state.quality;
            let next_distribution = match step + 1 == self.actions.len() {
                true => match next_state.progress >= self.settings.max_progress {
                    true => Self::ZERO_DISTRIBUTION,
                    false => Self::EMPTY_DISTRIBUTION,
                },
                false => {
                    self.solve(next_state, *condition, step + 1);
                    self.memoization
                        .get(&(Self::normalize_state(next_state), *condition, step + 1))
                        .expect("State not in memoization even after solving")
                }
            }
            .iter()
            .map(|element| Value {
                quality: element.quality.saturating_add(action_quality),
                probability: element.probability * condition_probability,
            });
            distribution = Self::merge_distributions(distribution.into_iter(), next_distribution);
        }
        self.memoization
            .insert((state, condition, step), distribution);
    }

    fn normalize_state(state: SimulationState) -> SimulationState {
        SimulationState {
            quality: 0,
            ..state
        }
    }

    fn merge_distributions(
        lhs_iter: impl Iterator<Item = Value>,
        rhs_iter: impl Iterator<Item = Value>,
    ) -> Vec<Value> {
        let mut result = Vec::new();
        let mut lhs_iter = lhs_iter.peekable();
        let mut rhs_iter = rhs_iter.peekable();
        while let (Some(lhs), Some(rhs)) = (lhs_iter.peek(), rhs_iter.peek()) {
            match lhs.quality.cmp(&rhs.quality) {
                std::cmp::Ordering::Less => {
                    result.push(*lhs);
                    lhs_iter.next();
                }
                std::cmp::Ordering::Equal => {
                    result.push(Value {
                        quality: lhs.quality,
                        probability: lhs.probability + rhs.probability,
                    });
                    lhs_iter.next();
                    rhs_iter.next();
                }
                std::cmp::Ordering::Greater => {
                    result.push(*rhs);
                    rhs_iter.next();
                }
            }
        }
        result.extend(lhs_iter);
        result.extend(rhs_iter);
        result
    }
}
