use crate::game::units::{Progress, Quality};

#[derive(Debug, Clone, Copy, Default)]
pub struct ParetoValue {
    pub progress: Progress,
    pub quality: Quality,
}

impl ParetoValue {
    pub const fn zero() -> Self {
        Self {
            progress: Progress::new(0),
            quality: Quality::new(0),
        }
    }

    pub fn add(self, progress: Progress, quality: Quality) -> Self {
        ParetoValue {
            progress: self.progress.saturating_add(progress),
            quality: self.quality.saturating_add(quality),
        }
    }
}

const ZERO_FRONT: [ParetoValue; 1] = [ParetoValue::zero()];

pub struct ParetoFrontBuilder {
    buffer: [ParetoValue; 1024],
    merge_buffer: [ParetoValue; 1024],
    length: usize,
}

impl ParetoFrontBuilder {
    pub fn new() -> Self {
        Self {
            buffer: [Default::default(); 1024],
            merge_buffer: [Default::default(); 1024],
            length: 0,
        }
    }

    pub fn add(&mut self, mut new_values: &[ParetoValue], progress: Progress, quality: Quality) {
        let this_values = &self.buffer[0..self.length];
        if new_values.is_empty() {
            new_values = &ZERO_FRONT;
        }

        let mut i: usize = 0;
        let mut j: usize = 0;
        let mut n: usize = 0;

        while i < this_values.len() && j < new_values.len() {
            if this_values[i].progress > new_values[j].progress.saturating_add(progress) {
                n = Self::append_to_merge_buffer(&mut self.merge_buffer, n, this_values[i]);
                i += 1;
            } else {
                n = Self::append_to_merge_buffer(
                    &mut self.merge_buffer,
                    n,
                    new_values[j].add(progress, quality),
                );
                j += 1;
            }
        }

        while i < this_values.len() {
            n = Self::append_to_merge_buffer(&mut self.merge_buffer, n, this_values[i]);
            i += 1;
        }
        while j < new_values.len() {
            n = Self::append_to_merge_buffer(
                &mut self.merge_buffer,
                n,
                new_values[j].add(progress, quality),
            );
            j += 1;
        }

        if self.merge_buffer[n - 1].progress == Progress::new(0) {
            n -= 1;
        }

        self.length = n;
        self.buffer[0..self.length].copy_from_slice(&self.merge_buffer[0..n]);
    }

    pub fn finalize(&mut self) -> &[ParetoValue] {
        &self.buffer[0..self.length]
    }

    fn append_to_merge_buffer(buffer: &mut [ParetoValue], i: usize, v: ParetoValue) -> usize {
        if i == 0 {
            buffer[i] = v;
            i + 1
        } else {
            if buffer[i - 1].progress == v.progress {
                buffer[i - 1].quality = std::cmp::max(buffer[i - 1].quality, v.quality);
                i
            } else {
                buffer[i] = v;
                i + 1
            }
        }
    }
}
