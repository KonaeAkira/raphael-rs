mod effect_pareto_front;
pub use effect_pareto_front::EffectParetoFront;

mod quality_pareto_front;
pub use quality_pareto_front::QualityParetoFront;

trait Dominate {
    fn dominate(&self, other: &Self) -> bool;
}

struct ParetoFront<T: Clone + Copy + Dominate> {
    values: Vec<T>,
}

impl<T: Clone + Copy + Dominate> ParetoFront<T> {
    fn is_dominated(&self, new_value: &T) -> bool {
        self.values.iter().any(|value| value.dominate(new_value))
    }

    pub fn insert(&mut self, new_value: T) -> bool {
        if !self.is_dominated(&new_value) {
            self.values.retain(|value| !new_value.dominate(value));
            self.values.push(new_value);
            true
        } else {
            false
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl<T: Clone + Copy + Dominate> Default for ParetoFront<T> {
    fn default() -> Self {
        Self { values: Vec::new() }
    }
}
