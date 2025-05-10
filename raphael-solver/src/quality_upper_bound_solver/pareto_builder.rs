use std::cmp::Reverse;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ParetoValue {
    progress: u32,
    quality: u32,
    steps: Reverse<u8>,
}

impl ParetoValue {
    #[inline]
    pub const fn new(progress: u32, quality: u32, steps: u8) -> Self {
        Self {
            progress,
            quality,
            steps: Reverse(steps),
        }
    }

    #[inline]
    pub const fn progress(&self) -> u32 {
        self.progress
    }

    #[inline]
    pub const fn quality(&self) -> u32 {
        self.quality
    }

    #[inline]
    pub const fn steps(&self) -> u8 {
        self.steps.0
    }

    #[inline]
    pub const fn dominates(&self, other: &Self) -> bool {
        self.progress() >= other.progress()
            && self.quality() >= other.quality()
            && self.steps() <= other.steps()
    }
}

pub struct ParetoBuilder {
    values: Vec<Option<ParetoValue>>,
}

impl ParetoBuilder {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn push(&mut self, value: ParetoValue) {
        self.values.push(Some(value));
    }

    pub fn extend(&mut self, values: impl Iterator<Item = ParetoValue>) {
        self.values.extend(values.map(|value| Some(value)));
    }

    pub fn build(mut self) -> Box<[ParetoValue]> {
        self.values.sort_unstable();
        filter_dominated(&mut self.values);
        self.values.into_iter().flatten().collect()
    }
}

fn filter_dominated(values: &mut [Option<ParetoValue>]) {
    if values.len() <= 1 {
        return;
    }
    let (lhs, rhs) = values.split_at_mut(values.len() / 2);
    filter_dominated(lhs);
    filter_dominated(rhs);
    lhs.iter_mut().for_each(|lhs_option| {
        if let Some(lhs_value) = lhs_option {
            let dominates_lhs_value = |value: &ParetoValue| value.dominates(lhs_value);
            if rhs.iter().flatten().any(dominates_lhs_value) {
                *lhs_option = None;
            }
        }
    });
}
