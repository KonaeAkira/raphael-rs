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
        Some(self.cmp(other))
    }
}

impl<'a> std::cmp::Ord for Segment<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.head
            .quality
            .cmp(&other.head.quality)
            .then(self.head.progress.cmp(&other.head.progress))
    }
}

pub struct ParetoFrontBuilder<'a> {
    segments: Vec<Segment<'a>>,
    cutoff: ParetoValue,
}

impl<'a> ParetoFrontBuilder<'a> {
    pub fn new(progress_cutoff: u32, quality_cutoff: u32) -> Self {
        Self {
            segments: Vec::new(),
            cutoff: ParetoValue::new(progress_cutoff, quality_cutoff),
        }
    }

    pub fn push(&mut self, progress: u32, quality: u32) {
        let segment = Segment {
            head: ParetoValue::new(
                std::cmp::min(self.cutoff.progress, progress),
                std::cmp::min(self.cutoff.quality, quality),
            ),
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
        let (mut head, mut values) = values.split_first();
        while let Some((next_head, next_values)) = values.split_first()
            && next_head.quality + quality_offset >= self.cutoff.quality
        {
            head = next_head;
            values = next_values;
        }
        let head = ParetoValue::new(
            std::cmp::min(self.cutoff.progress, head.progress + progress_offset),
            std::cmp::min(self.cutoff.quality, head.quality + quality_offset),
        );
        self.segments.push(Segment {
            head,
            values,
            offset: ParetoValue::new(progress_offset, quality_offset),
        });
    }

    pub fn build(self) -> Box<[ParetoValue]> {
        let mut result = Vec::<ParetoValue>::new();
        let mut segments = BinaryHeap::from(self.segments);
        while let Some(mut segment) = segments.pop() {
            if result
                .last()
                .is_none_or(|last_value| last_value.progress < segment.head.progress)
            {
                result.push(segment.head);
            }
            if segment.head.progress < self.cutoff.progress
                && let Some(new_head) = segment.values.split_off_first()
            {
                segment.head = ParetoValue::new(
                    std::cmp::min(
                        self.cutoff.progress,
                        new_head.progress + segment.offset.progress,
                    ),
                    std::cmp::min(
                        self.cutoff.quality,
                        new_head.quality + segment.offset.quality,
                    ),
                );
                segments.push(segment);
            }
        }
        result.into_boxed_slice()
    }
}
