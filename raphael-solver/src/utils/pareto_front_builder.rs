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

pub struct ParetoFrontBuilder {
    cutoff: ParetoValue,
    result: Vec<ParetoValue>,
    merge_buffer: Vec<ParetoValue>,
}

impl ParetoFrontBuilder {
    pub fn new() -> Self {
        Self {
            cutoff: ParetoValue::new(u32::MAX, u32::MAX),
            result: Vec::new(),
            merge_buffer: Vec::new(),
        }
    }

    pub fn initialize_with_cutoff(&mut self, cutoff: ParetoValue) {
        self.cutoff = cutoff;
        self.result.clear();
    }

    pub fn push(&mut self, value: ParetoValue) {
        self.push_slice(std::iter::once(value));
    }

    pub fn push_slice(&mut self, values: impl Iterator<Item = ParetoValue>) {
        let (mut slice_a, mut slice_b) = {
            let slice_a_len = self.result.len();
            self.result.extend(values);
            self.result.split_at_mut(slice_a_len)
        };

        slice_b = trim_slice(slice_b, self.cutoff);

        loop {
            match (slice_a.first(), slice_b.first()) {
                (None, None) => break,
                (None, Some(_)) => {
                    let idx = self.merge_buffer.last().map_or(0, |merge_tail| {
                        slice_b.partition_point(|v| v.progress <= merge_tail.progress)
                    });
                    self.merge_buffer.extend_from_slice(&slice_b[idx..]);
                    break;
                }
                (Some(_), None) => {
                    let idx = self.merge_buffer.last().map_or(0, |merge_tail| {
                        slice_a.partition_point(|v| v.progress <= merge_tail.progress)
                    });
                    self.merge_buffer.extend_from_slice(&slice_a[idx..]);
                    break;
                }
                (Some(a), Some(b)) => {
                    if a.quality > b.quality || (a.quality == b.quality && a.progress >= b.progress)
                    {
                        if self
                            .merge_buffer
                            .last()
                            .is_none_or(|merge_tail| a.progress > merge_tail.progress)
                        {
                            self.merge_buffer.push(*a);
                        }
                        slice_a.split_off_first_mut();
                    } else {
                        if self
                            .merge_buffer
                            .last()
                            .is_none_or(|merge_tail| b.progress > merge_tail.progress)
                        {
                            self.merge_buffer.push(*b);
                        }
                        slice_b.split_off_first_mut();
                    }
                }
            }
        }

        std::mem::swap(&mut self.result, &mut self.merge_buffer);
        self.merge_buffer.clear();
    }

    pub fn is_maximal(&self, cutoff: ParetoValue) -> bool {
        self.result.first().is_some_and(|value| {
            value.progress >= cutoff.progress && value.quality >= cutoff.quality
        })
    }

    pub fn result(&mut self) -> Box<[ParetoValue]> {
        self.result.as_slice().into()
    }
}

fn trim_slice(mut slice: &mut [ParetoValue], cutoff: ParetoValue) -> &mut [ParetoValue] {
    if slice.len() >= 1 {
        if slice[0].quality > cutoff.quality {
            while slice.len() >= 2 && slice[1].quality >= cutoff.quality {
                slice.split_off_first_mut();
            }
            slice[0].quality = cutoff.quality;
        }
        if slice[slice.len() - 1].progress > cutoff.progress {
            while slice.len() >= 2 && slice[slice.len() - 2].progress >= cutoff.progress {
                slice.split_off_last_mut();
            }
            slice[slice.len() - 1].progress = cutoff.progress;
        }
    }
    slice
}
