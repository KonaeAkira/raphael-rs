use std::alloc::{self, Layout};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParetoValue<T, U> {
    pub first: T,
    pub second: U,
}

impl<T, U> ParetoValue<T, U> {
    pub const fn new(first: T, second: U) -> Self {
        Self { first, second }
    }
}

#[derive(Debug, Clone, Copy)]
struct Segment {
    pub offset: usize,
    pub length: usize,
}

pub struct ParetoFrontBuilder<T, U>
where
    T: num_traits::int::PrimInt,
    U: num_traits::int::PrimInt,
{
    buffer: *mut ParetoValue<T, U>,
    buffer_head: usize,
    buffer_capacity: usize,
    segments: Vec<Segment>,
    // cut-off values
    max_first: T,
    max_second: U,
    // variables used for profiling
    fronts_generated: usize,
    values_generated: usize,
}

impl<T, U> ParetoFrontBuilder<T, U>
where
    T: num_traits::int::PrimInt,
    U: num_traits::int::PrimInt,
{
    pub fn new(max_first: T, max_second: U) -> Self {
        const INITIAL_CAPACITY: usize = 1024;
        unsafe {
            let layout = alloc::Layout::from_size_align_unchecked(
                INITIAL_CAPACITY * std::mem::size_of::<ParetoValue<T, U>>(),
                std::mem::align_of::<ParetoValue<T, U>>(),
            );
            Self {
                buffer: alloc::alloc(layout) as *mut ParetoValue<T, U>,
                buffer_head: 0,
                buffer_capacity: INITIAL_CAPACITY,
                segments: Vec::new(),
                max_first,
                max_second,
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
        self.buffer_capacity * std::mem::size_of::<ParetoValue<T, U>>()
    }

    fn layout(&self) -> Layout {
        unsafe {
            alloc::Layout::from_size_align_unchecked(
                self.buffer_byte_size(),
                std::mem::align_of::<ParetoValue<T, U>>(),
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
                        as *mut ParetoValue<T, U>;
            }
        }
    }

    pub fn push_empty(&mut self) {
        self.segments.push(Segment {
            offset: self.buffer_head,
            length: 0,
        });
    }

    pub fn push(&mut self, values: &[ParetoValue<T, U>]) {
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

    pub fn add(&mut self, first: T, second: U) {
        let segment = self.segments.last().unwrap();
        let slice: &mut [ParetoValue<T, U>];
        unsafe {
            slice = std::slice::from_raw_parts_mut(self.buffer.add(segment.offset), segment.length);
        }
        for x in slice.iter_mut() {
            x.first = x.first + first;
            x.second = x.second + second;
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

        let mut rolling_max: Option<U> = None;
        let mut try_insert = |x: ParetoValue<T, U>| {
            if rolling_max.is_none() || x.second > rolling_max.unwrap() {
                rolling_max = Some(x.second);
                slice_c[tail_c] = x;
                tail_c += 1;
            }
        };

        while head_a < slice_a.len() && head_b < slice_b.len() {
            match slice_a[head_a].first.cmp(&slice_b[head_b].first) {
                std::cmp::Ordering::Less => {
                    try_insert(slice_b[head_b]);
                    head_b += 1;
                }
                std::cmp::Ordering::Equal => {
                    let first = slice_a[head_a].first;
                    let second = std::cmp::max(slice_a[head_a].second, slice_b[head_b].second);
                    try_insert(ParetoValue::new(first, second));
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

        // cut out values that are over max_first
        while head_c + 1 < tail_c && slice_c[head_c + 1].first >= self.max_first {
            head_c += 1;
        }

        // cut out values that are over max_second
        while head_c + 1 < tail_c && slice_c[tail_c - 2].second >= self.max_second {
            tail_c -= 1;
        }

        let segment_c = Segment {
            offset: offset_c + head_c,
            length: tail_c - head_c,
        };
        self.buffer_head = segment_c.offset + segment_c.length;
        self.segments.push(segment_c);
    }

    pub fn peek(&mut self) -> Option<Box<[ParetoValue<T, U>]>> {
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
                assert!(window[0].first > window[1].first);
                assert!(window[0].second < window[1].second);
            }
        }
    }
}

impl<T, U> Drop for ParetoFrontBuilder<T, U>
where
    T: num_traits::int::PrimInt,
    U: num_traits::int::PrimInt,
{
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

    use simulator::{ActionMask, Settings};

    use super::*;

    const SETTINGS: Settings = Settings {
        max_cp: 500,
        max_durability: 60,
        max_progress: 1000,
        max_quality: 2000,
        base_progress: 0,
        base_quality: 0,
        initial_quality: 0,
        job_level: 90,
        allowed_actions: ActionMask::none(),
    };

    const SAMPLE_FRONT_1: &[ParetoValue<u16, u16>] = &[
        ParetoValue::new(300, 100),
        ParetoValue::new(200, 200),
        ParetoValue::new(100, 300),
    ];

    const SAMPLE_FRONT_2: &[ParetoValue<u16, u16>] = &[
        ParetoValue::new(300, 50),
        ParetoValue::new(250, 150),
        ParetoValue::new(150, 250),
        ParetoValue::new(50, 270),
    ];

    #[test]
    fn test_merge_empty() {
        let mut builder: ParetoFrontBuilder<u16, u16> =
            ParetoFrontBuilder::new(SETTINGS.max_progress, SETTINGS.max_quality);
        builder.push_empty();
        builder.push_empty();
        builder.merge();
        let front = builder.peek().unwrap();
        assert!(front.as_ref().is_empty());
        builder.check_invariants();
    }

    #[test]
    fn test_value_shift() {
        let mut builder: ParetoFrontBuilder<u16, u16> =
            ParetoFrontBuilder::new(SETTINGS.max_progress, SETTINGS.max_quality);
        builder.push(SAMPLE_FRONT_1);
        builder.add(100, 100);
        let front = builder.peek().unwrap();
        assert_eq!(
            *front,
            [
                ParetoValue::new(400, 200),
                ParetoValue::new(300, 300),
                ParetoValue::new(200, 400),
            ]
        );
        builder.check_invariants();
    }

    #[test]
    fn test_merge() {
        let mut builder: ParetoFrontBuilder<u16, u16> =
            ParetoFrontBuilder::new(SETTINGS.max_progress, SETTINGS.max_quality);
        builder.push(SAMPLE_FRONT_1);
        builder.push(SAMPLE_FRONT_2);
        builder.merge();
        let front = builder.peek().unwrap();
        assert_eq!(
            *front,
            [
                ParetoValue::new(300, 100),
                ParetoValue::new(250, 150),
                ParetoValue::new(200, 200),
                ParetoValue::new(150, 250),
                ParetoValue::new(100, 300),
            ]
        );
        builder.check_invariants();
    }

    #[test]
    fn test_merge_truncated() {
        let mut builder: ParetoFrontBuilder<u16, u16> =
            ParetoFrontBuilder::new(SETTINGS.max_progress, SETTINGS.max_quality);
        builder.push(SAMPLE_FRONT_1);
        builder.add(SETTINGS.max_progress, SETTINGS.max_quality);
        builder.push(SAMPLE_FRONT_2);
        builder.add(SETTINGS.max_progress, SETTINGS.max_quality);
        builder.merge();
        let front = builder.peek().unwrap();
        assert_eq!(*front, [ParetoValue::new(1100, 2300)]);
        builder.check_invariants();
    }

    #[test]
    fn test_random_simulation() {
        let mut rng = rand::thread_rng();
        let mut builder: ParetoFrontBuilder<u16, u16> =
            ParetoFrontBuilder::new(SETTINGS.max_progress, SETTINGS.max_quality);
        let mut lut = [0; 5000];

        for _ in 0..200 {
            let cnt = rng.gen_range(1..200);
            for _ in 0..cnt {
                let progress: u16 = rng.gen_range(0..5000);
                let quality: u16 = rng.gen_range(0..10000);
                for i in 0..=progress as usize {
                    lut[i] = std::cmp::max(lut[i], quality);
                }
                builder.push(&[ParetoValue::new(progress, quality)]);
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
            assert_eq!(lut[value.first as usize], value.second);
        }

        builder.clear();
        builder.check_invariants();
    }
}
