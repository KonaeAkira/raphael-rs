use std::alloc::{self, Layout};

use crate::game::{
    units::{Progress, Quality},
    Settings,
};

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

#[derive(Debug, Clone, Copy)]
struct Segment {
    pub offset: usize,
    pub length: usize,
}

pub struct ParetoFrontBuilder {
    settings: Settings,
    buffer: *mut ParetoValue,
    buffer_head: usize,
    buffer_capacity: usize,
    segments: Vec<Segment>,
    // variables used for profiling
    fronts_generated: usize,
    values_generated: usize,
}

impl ParetoFrontBuilder {
    pub fn new(settings: Settings) -> Self {
        const INITIAL_CAPACITY: usize = 1024;
        unsafe {
            let layout = alloc::Layout::from_size_align_unchecked(
                INITIAL_CAPACITY * std::mem::size_of::<ParetoValue>(),
                std::mem::align_of::<ParetoValue>(),
            );
            Self {
                settings,
                buffer: alloc::alloc(layout) as *mut ParetoValue,
                buffer_head: 0,
                buffer_capacity: INITIAL_CAPACITY,
                segments: Vec::new(),
                fronts_generated: 0,
                values_generated: 0,
            }
        }
    }

    pub fn clear(&mut self) {
        self.segments.clear();
        self.buffer_head = 0;
    }

    fn buffer_byte_size(&self) -> usize {
        self.buffer_capacity * std::mem::size_of::<ParetoValue>()
    }

    fn layout(&self) -> Layout {
        unsafe {
            alloc::Layout::from_size_align_unchecked(
                self.buffer_byte_size(),
                std::mem::align_of::<ParetoValue>(),
            )
        }
    }

    fn ensure_buffer_size(&mut self, min_buffer_capacity: usize) {
        if self.buffer_capacity < min_buffer_capacity {
            unsafe {
                let layout = self.layout();
                while self.buffer_capacity < min_buffer_capacity {
                    self.buffer_capacity *= 2;
                }
                self.buffer =
                    alloc::realloc(self.buffer as *mut u8, layout, self.buffer_byte_size())
                        as *mut ParetoValue;
            }
        }
    }

    pub fn push_empty(&mut self) {
        self.segments.push(Segment {
            offset: self.buffer_head,
            length: 0,
        });
    }

    pub fn push(&mut self, values: &[ParetoValue]) {
        let segment = Segment {
            offset: self.buffer_head,
            length: values.len(),
        };
        self.ensure_buffer_size(segment.offset + segment.length);
        unsafe {
            std::slice::from_raw_parts_mut(self.buffer.add(segment.offset), segment.length)
                .copy_from_slice(values);
        }
        self.buffer_head += segment.length;
        self.segments.push(segment);
    }

    pub fn add(&mut self, progress: Progress, quality: Quality) {
        let segment = self.segments.last().unwrap();
        let slice: &mut [ParetoValue];
        unsafe {
            slice = std::slice::from_raw_parts_mut(self.buffer.add(segment.offset), segment.length);
        }
        for x in slice.iter_mut() {
            x.progress = x.progress.add(progress);
            x.quality = x.quality.add(quality);
        }
    }

    pub fn merge(&mut self) {
        assert!(self.segments.len() >= 2);
        let segment_b = self.segments.pop().unwrap();
        let segment_a = self.segments.pop().unwrap();

        let length_c = segment_a.length + segment_b.length;
        let offset_c = if segment_a.offset + segment_a.length + length_c <= segment_b.offset {
            // sandwich C between A and B
            segment_a.offset + segment_a.length
        } else {
            // allocate C after B
            self.ensure_buffer_size(self.buffer_head + length_c);
            self.buffer_head
        };

        let slice_a = unsafe {
            std::slice::from_raw_parts(self.buffer.add(segment_a.offset), segment_a.length)
        };
        let slice_b = unsafe {
            std::slice::from_raw_parts(self.buffer.add(segment_b.offset), segment_b.length)
        };
        let slice_c =
            unsafe { std::slice::from_raw_parts_mut(self.buffer.add(offset_c), length_c) };

        let mut head_a: usize = 0;
        let mut head_b: usize = 0;
        let mut head_c: usize = 0;
        let mut tail_c: usize = 0;

        let mut cur_quality: Option<Quality> = None;
        let mut try_insert = |x: ParetoValue| {
            if cur_quality.is_none() || x.quality > cur_quality.unwrap() {
                cur_quality = Some(x.quality);
                slice_c[tail_c] = x;
                tail_c += 1;
            }
        };

        while head_a < slice_a.len() && head_b < slice_b.len() {
            match slice_a[head_a].progress.cmp(&slice_b[head_b].progress) {
                std::cmp::Ordering::Less => {
                    try_insert(slice_b[head_b]);
                    head_b += 1;
                }
                std::cmp::Ordering::Equal => {
                    let progress = slice_a[head_a].progress;
                    let quality = std::cmp::max(slice_a[head_a].quality, slice_b[head_b].quality);
                    try_insert(ParetoValue { progress, quality });
                    head_a += 1;
                    head_b += 1;
                }
                std::cmp::Ordering::Greater => {
                    try_insert(slice_a[head_a]);
                    head_a += 1;
                }
            }
        }

        while head_a < slice_a.len() {
            try_insert(slice_a[head_a]);
            head_a += 1;
        }

        while head_b < slice_b.len() {
            try_insert(slice_b[head_b]);
            head_b += 1;
        }

        // cut out values that are over max_progress
        while head_c + 1 < tail_c && slice_c[head_c + 1].progress >= self.settings.max_progress {
            head_c += 1;
        }

        let segment_c = Segment {
            offset: offset_c + head_c,
            length: tail_c - head_c,
        };
        self.buffer_head = segment_c.offset + segment_c.length;
        self.segments.push(segment_c);
    }

    pub fn peek(&mut self) -> Option<Box<[ParetoValue]>> {
        match self.segments.last() {
            Some(segment) => {
                self.fronts_generated += 1;
                self.values_generated += segment.length;
                unsafe {
                    let slice =
                        std::slice::from_raw_parts(self.buffer.add(segment.offset), segment.length);
                    Some(slice.into())
                }
            }
            None => None,
        }
    }

    #[cfg(test)]
    fn check_invariants(&self) {
        for window in self.segments.windows(2) {
            // segments musn't overlap and must have left-to-right ordering
            assert!(window[0].offset + window[0].length <= window[1].offset);
        }
        match self.segments.last() {
            Some(segment) => {
                // buffer head must point to the element right after the last segment
                assert_eq!(segment.offset + segment.length, self.buffer_head);
                // buffer head must remain within buffer capacity
                assert!(self.buffer_head <= self.buffer_capacity);
            }
            None => assert_eq!(self.buffer_head, 0),
        };
        for segment in self.segments.iter() {
            // each segment must form a valid pareto front
            let slice = unsafe {
                std::slice::from_raw_parts(self.buffer.add(segment.offset), segment.length)
            };
            for window in slice.windows(2) {
                assert!(window[0].progress > window[1].progress);
                assert!(window[0].quality < window[1].quality);
            }
        }
    }
}

impl Drop for ParetoFrontBuilder {
    fn drop(&mut self) {
        let buffer_byte_size = self.layout().size();
        dbg!(
            buffer_byte_size,
            self.fronts_generated,
            self.values_generated
        );
        unsafe {
            alloc::dealloc(self.buffer as *mut u8, self.layout());
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;

    const SETTINGS: Settings = Settings {
        max_cp: 500,
        max_durability: 60,
        max_progress: Progress::new(1000),
        max_quality: Quality::new(2000),
    };

    const SAMPLE_FRONT_1: &[ParetoValue] = &[
        ParetoValue::new(Progress::new(300), Quality::new(100)),
        ParetoValue::new(Progress::new(200), Quality::new(200)),
        ParetoValue::new(Progress::new(100), Quality::new(300)),
    ];

    const SAMPLE_FRONT_2: &[ParetoValue] = &[
        ParetoValue::new(Progress::new(300), Quality::new(50)),
        ParetoValue::new(Progress::new(250), Quality::new(150)),
        ParetoValue::new(Progress::new(150), Quality::new(250)),
        ParetoValue::new(Progress::new(50), Quality::new(270)),
    ];

    #[test]
    fn test_merge_empty() {
        let mut builder = ParetoFrontBuilder::new(SETTINGS);
        builder.push_empty();
        builder.push_empty();
        builder.merge();
        let front = builder.peek().unwrap();
        assert!(front.as_ref().is_empty());
        builder.check_invariants();
    }

    #[test]
    fn test_value_shift() {
        let mut builder = ParetoFrontBuilder::new(SETTINGS);
        builder.push(SAMPLE_FRONT_1);
        builder.add(Progress::new(100), Quality::new(100));
        let front = builder.peek().unwrap();
        assert_eq!(
            *front,
            [
                ParetoValue::new(Progress::new(400), Quality::new(200)),
                ParetoValue::new(Progress::new(300), Quality::new(300)),
                ParetoValue::new(Progress::new(200), Quality::new(400)),
            ]
        );
        builder.check_invariants();
    }

    #[test]
    fn test_merge() {
        let mut builder = ParetoFrontBuilder::new(SETTINGS);
        builder.push(SAMPLE_FRONT_1);
        builder.push(SAMPLE_FRONT_2);
        builder.merge();
        let front = builder.peek().unwrap();
        assert_eq!(
            *front,
            [
                ParetoValue::new(Progress::new(300), Quality::new(100)),
                ParetoValue::new(Progress::new(250), Quality::new(150)),
                ParetoValue::new(Progress::new(200), Quality::new(200)),
                ParetoValue::new(Progress::new(150), Quality::new(250)),
                ParetoValue::new(Progress::new(100), Quality::new(300)),
            ]
        );
        builder.check_invariants();
    }

    #[test]
    fn test_merge_truncated() {
        let mut builder = ParetoFrontBuilder::new(SETTINGS);
        builder.push(SAMPLE_FRONT_1);
        builder.add(SETTINGS.max_progress, SETTINGS.max_quality);
        builder.push(SAMPLE_FRONT_2);
        builder.add(SETTINGS.max_progress, SETTINGS.max_quality);
        builder.merge();
        let front = builder.peek().unwrap();
        assert_eq!(
            *front,
            [ParetoValue::new(Progress::new(1100), Quality::new(2300))]
        );
        builder.check_invariants();
    }

    #[test]
    fn test_random_simulation() {
        let mut rng = rand::thread_rng();
        let mut builder = ParetoFrontBuilder::new(SETTINGS);
        let mut lut = [Quality::new(0); 5000];

        for _ in 0..200 {
            let cnt = rng.gen_range(1..200);
            for _ in 0..cnt {
                let progress: u32 = rng.gen_range(0..5000);
                let quality: u32 = rng.gen_range(0..10000);
                for i in 0..=progress as usize {
                    lut[i] = std::cmp::max(lut[i], Quality::new(quality));
                }
                builder.push(&[ParetoValue {
                    progress: Progress::new(progress),
                    quality: Quality::new(quality),
                }]);
                builder.check_invariants();
            }
            for _ in 1..cnt {
                builder.merge();
                builder.check_invariants();
            }
        }
        for _ in 1..200 {
            builder.merge();
            builder.check_invariants();
        }

        let front = builder.peek().unwrap();
        for value in front.iter() {
            assert_eq!(lut[f32::from(value.progress) as usize], value.quality);
        }

        builder.clear();
        builder.check_invariants();
    }
}
