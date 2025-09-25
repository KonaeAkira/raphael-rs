use std::collections::BinaryHeap;

use nunny::NonEmpty;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ParetoValue {
    pub progress: u32,
    pub quality: u32,
}

impl ParetoValue {
    pub const fn new(progress: u32, quality: u32) -> Self {
        Self { progress, quality }
    }
}

impl std::ops::Add for ParetoValue {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.progress + rhs.progress, self.quality + rhs.quality)
    }
}

#[derive(PartialEq, Eq)]
struct Segment<'a> {
    head: ParetoValue,
    values: &'a [ParetoValue],
    offset: ParetoValue,
}

impl<'a> std::cmp::PartialOrd for Segment<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.head.quality.cmp(&other.head.quality))
    }
}

impl<'a> std::cmp::Ord for Segment<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.head.quality.cmp(&other.head.quality)
    }
}

pub struct ParetoFrontBuilder<'a> {
    segments: Vec<Segment<'a>>,
}

impl<'a> ParetoFrontBuilder<'a> {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn push(&mut self, progress: u32, quality: u32) {
        let segment = Segment {
            head: ParetoValue::new(progress, quality),
            values: &[],
            offset: ParetoValue::new(0, 0),
        };
        self.segments.push(segment);
    }

    pub fn push_slice(
        &mut self,
        values: &'a NonEmpty<[ParetoValue]>,
        progress_offset: u32,
        quality_offset: u32,
    ) {
        let (&head, values) = values.split_first();
        let offset = ParetoValue::new(progress_offset, quality_offset);
        self.segments.push(Segment {
            head: head + offset,
            values,
            offset,
        });
    }

    pub fn build(self, max_progress: u32, max_quality: u32) -> Box<[ParetoValue]> {
        let mut result = Vec::<ParetoValue>::new();
        let mut segments = BinaryHeap::from(self.segments);
        while let Some(mut segment) = segments.pop() {
            let value = ParetoValue::new(
                std::cmp::min(max_progress, segment.head.progress),
                std::cmp::min(max_quality, segment.head.quality),
            );
            while result.last().is_some_and(|last_value| {
                last_value.progress <= value.progress && last_value.quality <= value.quality
            }) {
                result.pop();
            }
            if result
                .last()
                .is_none_or(|last_value| last_value.progress < value.progress)
            {
                result.push(value);
            }
            if let Some(new_head) = segment.values.split_off_first() {
                segment.head = *new_head + segment.offset;
                segments.push(segment);
            }
        }
        result.into_boxed_slice()
    }
}
