use crate::{Action, Condition, Settings, SimulationState};

const fn condition_probabilities(current_condition: Condition) -> &'static [(Condition, f32)] {
    match current_condition {
        Condition::Normal => &[
            (Condition::Normal, 0.80),
            (Condition::Good, 0.15),
            (Condition::Excellent, 0.05),
        ],
        Condition::Good => &[(Condition::Normal, 1.00)],
        Condition::Excellent => &[(Condition::Poor, 1.00)],
        Condition::Poor => &[(Condition::Normal, 1.00)],
    }
}

pub fn quality_probability_distribution(
    settings: Settings,
    actions: impl Into<Box<[Action]>>,
) -> Vec<f32> {
    let initial_state = SimulationState::new(&settings);
    let mut solver = QualityDistributionSolver::new(settings, actions.into());
    solver.solve(initial_state, Condition::Normal, 0);
    let compact_distribution = solver
        .memoization
        .get(&(initial_state, Condition::Normal, 0))
        .expect("State not in memoization even after solving");
    let mut distribution = vec![0.0; settings.max_quality as usize + 1];
    for element in compact_distribution {
        distribution[std::cmp::min(element.quality, settings.max_quality) as usize] +=
            element.probability
    }
    distribution
}

#[derive(Debug, Clone, Copy)]
struct Element {
    quality: u16,
    probability: f32,
}

struct QualityDistributionSolver {
    settings: Settings,
    actions: Box<[Action]>,
    memoization: std::collections::HashMap<(SimulationState, Condition, usize), Vec<Element>>,
}

impl QualityDistributionSolver {
    const ZERO_DISTRIBUTION: &[Element] = &[Element {
        quality: 0,
        probability: 1.0,
    }];

    fn new(settings: Settings, actions: Box<[Action]>) -> Self {
        Self {
            settings: Settings {
                adversarial: false,
                ..settings
            },
            actions,
            memoization: Default::default(),
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
            let next_state = match action_result {
                Ok(next_state) => next_state,
                Err(_) => state,
            };
            let action_quality = next_state.quality;
            let next_distribution = match step + 1 == self.actions.len() {
                true => Self::ZERO_DISTRIBUTION,
                false => {
                    self.solve(next_state, *condition, step + 1);
                    self.memoization
                        .get(&(Self::normalize_state(next_state), *condition, step + 1))
                        .expect("State not in memoization even after solving")
                }
            }
            .iter()
            .map(|element| Element {
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
        lhs_iter: impl Iterator<Item = Element>,
        rhs_iter: impl Iterator<Item = Element>,
    ) -> Vec<Element> {
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
                    result.push(Element {
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
