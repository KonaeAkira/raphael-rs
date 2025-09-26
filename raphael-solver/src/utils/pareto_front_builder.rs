use std::collections::BinaryHeap;

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
            head: ParetoValue::new(progress, std::cmp::min(self.cutoff.quality, quality)),
            values: &[],
            offset: ParetoValue::new(0, 0),
        };
        self.segments.push(segment);
    }

    pub fn push_slice(
        &mut self,
        values: &'a nunny::Slice<ParetoValue>,
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
            head.progress + progress_offset,
            std::cmp::min(self.cutoff.quality, head.quality + quality_offset),
        );
        self.segments.push(Segment {
            head,
            values,
            offset: ParetoValue::new(progress_offset, quality_offset),
        });
    }

    pub fn build(self) -> Box<[ParetoValue]> {
        let mut segments = BinaryHeap::from(self.segments);
        if let Some(mut first_segment) = segments.pop() {
            if first_segment.head.progress >= self.cutoff.progress {
                return Box::new([first_segment.head]);
            }
            let mut result = nunny::vec![first_segment.head];
            if let Some(head) = first_segment.values.split_off_first() {
                first_segment.head = *head + first_segment.offset;
                segments.push(first_segment);
            }
            while let Some(mut segment) = segments.pop() {
                if segment.head.progress > result.last().progress {
                    result.push(segment.head);
                    if segment.head.progress >= self.cutoff.progress {
                        break;
                    }
                    if let Some(head) = segment.values.split_off_first() {
                        segment.head = *head + segment.offset;
                        segments.push(segment);
                    }
                } else {
                    while let Some(head) = segment.values.split_off_first() {
                        if head.progress + segment.offset.progress > result.last().progress {
                            segment.head = *head + segment.offset;
                            segments.push(segment);
                            break;
                        }
                    }
                }
            }
            result.into_boxed_slice().into()
        } else {
            Box::new([])
        }
    }
}
