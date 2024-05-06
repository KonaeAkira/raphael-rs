use std::alloc::{self, Layout};

use crate::game::units::{Progress, Quality};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ParetoValue {
    pub progress: Progress,
    pub quality: Quality,
}

impl ParetoValue {
    pub const fn new(progress: Progress, quality: Quality) -> Self {
        Self { progress, quality }
    }
}

pub struct ParetoFrontBuilder {
    buffer: *mut ParetoValue,
    capacity: usize,
    fronts: Vec<usize>,
    length: usize,
}

impl ParetoFrontBuilder {
    pub fn new() -> Self {
        const INITIAL_CAPACITY: usize = 1024;
        unsafe {
            let layout = alloc::Layout::from_size_align_unchecked(
                INITIAL_CAPACITY * std::mem::size_of::<ParetoValue>(),
                std::mem::align_of::<ParetoValue>(),
            );
            Self {
                buffer: alloc::alloc(layout) as *mut ParetoValue,
                capacity: INITIAL_CAPACITY,
                fronts: Vec::new(),
                length: 0,
            }
        }
    }

    pub fn clear(&mut self) {
        self.fronts.clear();
        self.length = 0;
    }

    fn buffer_byte_size(&self) -> usize {
        self.capacity * std::mem::size_of::<ParetoValue>()
    }

    fn layout(&self) -> Layout {
        unsafe {
            alloc::Layout::from_size_align_unchecked(
                self.buffer_byte_size(),
                std::mem::align_of::<ParetoValue>(),
            )
        }
    }

    fn ensure_buffer_size(&mut self, min_size: usize) {
        if self.capacity < min_size {
            unsafe {
                let layout = self.layout();
                while self.capacity < min_size {
                    self.capacity *= 2;
                }
                self.buffer =
                    alloc::realloc(self.buffer as *mut u8, layout, self.buffer_byte_size())
                        as *mut ParetoValue;
            }
        }
    }

    pub fn start_new_front(&mut self) {
        self.fronts.push(self.length);
    }

    pub fn import_front(&mut self, front: &[ParetoValue]) {
        if front.is_empty() {
            self.ensure_buffer_size(self.length + 1);
            self.fronts.push(self.length);
            unsafe {
                *self.buffer.add(self.length) = ParetoValue::new(Progress::new(0), Quality::new(0));
            }
            self.length += 1;
        } else {
            self.ensure_buffer_size(self.length + front.len());
            self.fronts.push(self.length);
            unsafe {
                std::slice::from_raw_parts_mut(self.buffer.add(self.length), front.len())
                    .copy_from_slice(front);
            }
            self.length += front.len();
        }
    }

    pub fn shift_last_front_value(&mut self, progress: Progress, quality: Quality) {
        let head = *self.fronts.last().unwrap();
        let length = self.length - head;
        let slice: &mut [ParetoValue];
        unsafe {
            slice = std::slice::from_raw_parts_mut(self.buffer.add(head), length);
        }
        for x in slice.iter_mut() {
            x.progress = x.progress.saturating_add(progress);
            x.quality = x.quality.saturating_add(quality);
        }
    }

    pub fn merge_last_two_fronts(&mut self) {
        assert!(self.fronts.len() >= 2);

        let head_b = self.fronts.pop().unwrap();
        let head_a = *self.fronts.last().unwrap();

        let slice_a: &[ParetoValue];
        let slice_b: &[ParetoValue];
        let slice_c: &mut [ParetoValue];
        unsafe {
            let max_len_c = self.length - head_a;
            self.ensure_buffer_size(self.length + max_len_c);
            slice_a = std::slice::from_raw_parts(self.buffer.add(head_a), head_b - head_a);
            slice_b = std::slice::from_raw_parts(self.buffer.add(head_b), self.length - head_b);
            slice_c = std::slice::from_raw_parts_mut(self.buffer.add(self.length), max_len_c);
        }

        let mut i: usize = 0;
        let mut j: usize = 0;
        let mut k: usize = 0;

        let mut cur_quality: Option<Quality> = None;
        let mut try_insert = |x: ParetoValue| {
            if cur_quality.is_none() || x.quality > cur_quality.unwrap() {
                cur_quality = Some(x.quality);
                slice_c[k] = x;
                k += 1;
            }
        };

        while i < slice_a.len() && j < slice_b.len() {
            match slice_a[i].progress.cmp(&slice_b[j].progress) {
                std::cmp::Ordering::Less => {
                    try_insert(slice_b[j]);
                    j += 1;
                }
                std::cmp::Ordering::Equal => {
                    let progress = slice_a[i].progress;
                    let quality = std::cmp::max(slice_a[i].quality, slice_b[j].quality);
                    try_insert(ParetoValue { progress, quality });
                    i += 1;
                    j += 1;
                }
                std::cmp::Ordering::Greater => {
                    try_insert(slice_a[i]);
                    i += 1;
                }
            }
        }

        while i < slice_a.len() {
            try_insert(slice_a[i]);
            i += 1;
        }

        while j < slice_b.len() {
            try_insert(slice_b[j]);
            j += 1;
        }

        let slice_r: &mut [ParetoValue];
        unsafe {
            slice_r = std::slice::from_raw_parts_mut(self.buffer.add(head_a), k);
        }
        slice_r.copy_from_slice(&slice_c[0..k]);
        self.length = head_a + k;
    }

    pub fn finalize(&mut self) -> Box<[ParetoValue]> {
        unsafe {
            let head = *self.fronts.last().unwrap();
            let len = self.length - head;
            std::slice::from_raw_parts(self.buffer.add(head), len).into()
        }
    }
}

impl Drop for ParetoFrontBuilder {
    fn drop(&mut self) {
        unsafe {
            dbg!(self.layout());
            alloc::dealloc(self.buffer as *mut u8, self.layout());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_FRONT_1: &[ParetoValue] = &[
        ParetoValue::new(Progress::new(300), Quality::new(100)),
        ParetoValue::new(Progress::new(200), Quality::new(200)),
        ParetoValue::new(Progress::new(100), Quality::new(300)),
    ];

    #[test]
    fn test_empty_pareto_front() {
        let mut builder = ParetoFrontBuilder::new();
        builder.start_new_front();
        builder.start_new_front();
        builder.merge_last_two_fronts();
        let front = builder.finalize();
        assert!(front.as_ref().is_empty())
    }

    #[test]
    fn test_value_shift() {
        let mut builder = ParetoFrontBuilder::new();
        builder.import_front(SAMPLE_FRONT_1);
        builder.shift_last_front_value(Progress::new(100), Quality::new(100));
        let front = builder.finalize();
        assert_eq!(
            *front,
            [
                ParetoValue::new(Progress::new(400), Quality::new(200)),
                ParetoValue::new(Progress::new(300), Quality::new(300)),
                ParetoValue::new(Progress::new(200), Quality::new(400)),
            ]
        )
    }
}
