#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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
pub struct Segment {
    offset: usize,
    length: usize,
}

pub type ParetoFrontId = Segment;

pub struct ParetoFrontBuilder<T, U>
where
    T: Copy + std::cmp::Ord + std::default::Default + std::fmt::Debug,
    U: Copy + std::cmp::Ord + std::default::Default + std::fmt::Debug,
{
    storage: Vec<ParetoValue<T, U>>,
    buffer: Vec<ParetoValue<T, U>>,
    segments: Vec<Segment>,
    // cut-off values
    max_first: T,
    max_second: U,
    // used for profiling
    fronts_generated: usize,
}

impl<T, U> ParetoFrontBuilder<T, U>
where
    T: Copy + std::cmp::Ord + std::default::Default + std::fmt::Debug,
    U: Copy + std::cmp::Ord + std::default::Default + std::fmt::Debug,
{
    pub fn new(max_first: T, max_second: U) -> Self {
        Self {
            storage: Vec::with_capacity(1 << 18),
            buffer: Vec::with_capacity(1 << 12),
            segments: Vec::with_capacity(1 << 12),
            max_first,
            max_second,
            fronts_generated: 0,
        }
    }

    pub fn clear(&mut self) {
        self.segments.clear();
        self.buffer.clear();
    }

    pub fn push_empty(&mut self) {
        self.segments.push(Segment {
            offset: self.buffer.len(),
            length: 0,
        });
    }

    pub fn push_from_slice(&mut self, values: &[ParetoValue<T, U>]) {
        let segment = Segment {
            offset: self.buffer.len(),
            length: values.len(),
        };
        self.segments.push(segment);
        self.buffer.extend_from_slice(values);
    }

    pub fn push_from_id(&mut self, id: ParetoFrontId) {
        let slice = &self.storage[id.offset..id.offset + id.length];
        self.segments.push(Segment {
            offset: self.buffer.len(),
            length: id.length,
        });
        self.buffer.extend_from_slice(slice);
    }

    /// Modifies each element of the last segment in-place.
    /// Panics in case there are no segments.
    pub fn map<F>(&mut self, f: F)
    where
        F: Fn(&mut ParetoValue<T, U>),
    {
        let segment = self.segments.last().unwrap();
        let slice = &mut self.buffer[segment.offset..segment.offset + segment.length];
        slice.iter_mut().for_each(f);
    }

    /// Merges the last two segments into one.
    /// Panics in case there are fewer than two segments.
    pub fn merge(&mut self) {
        assert!(self.segments.len() >= 2);
        let segment_b = self.segments.pop().unwrap();
        let segment_a = self.segments.pop().unwrap();

        let (slice_a, slice_b, slice_c) =
            Self::create_merge_slices(&mut self.buffer, segment_a, segment_b);
        let slice_c = Self::merge_mixed(slice_a, slice_b, slice_c);
        let slice_c = Self::trim_slice(slice_c, self.max_first, self.max_second);

        // SAFETY: slice_c and slice_a come from the same object (self.buffer) and outlive the pointers
        let offset_c =
            segment_a.offset + unsafe { slice_c.as_ptr().offset_from(slice_a.as_ptr()) as usize };
        let segment_c = Segment {
            offset: offset_c,
            length: slice_c.len(),
        };

        self.segments.push(segment_c);
        self.buffer.truncate(segment_c.offset + segment_c.length);
    }

    #[inline(always)]
    fn create_merge_slices(
        buffer: &mut Vec<ParetoValue<T, U>>,
        segment_a: Segment,
        segment_b: Segment,
    ) -> (
        &mut [ParetoValue<T, U>],
        &mut [ParetoValue<T, U>],
        &mut [ParetoValue<T, U>],
    ) {
        assert!(
            buffer.len() >= segment_b.offset + segment_b.length
                && segment_b.offset >= segment_a.offset + segment_a.length
        );
        let required_length = segment_a.length + segment_b.length;
        let available_length = segment_b.offset - (segment_a.offset + segment_a.length);
        if required_length <= available_length {
            // sandwich merge segment between parent segments
            let (_, buffer) = buffer.split_at_mut(segment_a.offset);
            let (slice_a, buffer) = buffer.split_at_mut(segment_a.length);
            let (slice_c, buffer) = buffer.split_at_mut(available_length);
            let (slice_b, _) = buffer.split_at_mut(segment_b.length);
            (slice_a, slice_b, slice_c)
        } else {
            // allocate merge segment at the end of the buffer
            buffer.resize(
                segment_b.offset + segment_b.length + required_length,
                Default::default(),
            );
            let (_, buffer) = buffer.split_at_mut(segment_a.offset);
            let (slice_a, buffer) = buffer.split_at_mut(segment_a.length);
            let (_, buffer) = buffer.split_at_mut(available_length);
            let (slice_b, slice_c) = buffer.split_at_mut(segment_b.length);
            (slice_a, slice_b, slice_c)
        }
    }

    #[inline(always)]
    fn merge_mixed<'a>(
        // slice_a and slice_b are marked &mut to tell the compiler that they are disjoint
        slice_a: &mut [ParetoValue<T, U>],
        slice_b: &mut [ParetoValue<T, U>],
        slice_c: &'a mut [ParetoValue<T, U>],
    ) -> &'a [ParetoValue<T, U>] {
        assert!(slice_a.len() + slice_b.len() <= slice_c.len());

        let mut idx_a = 0;
        let mut idx_b = 0;
        let mut idx_c = 0;

        let mut rolling_max = None;
        let mut try_insert = |x: ParetoValue<T, U>| {
            if rolling_max.is_none() || rolling_max.unwrap() < x.first {
                rolling_max = Some(x.first);
                unsafe {
                    // SAFETY: the number of elements added to slice_c is not greater than the total number of elements in slice_a and slice_b
                    *slice_c.get_unchecked_mut(idx_c) = x;
                }
                idx_c += 1;
            }
        };

        while idx_a < slice_a.len() && idx_b < slice_b.len() {
            let a = slice_a[idx_a];
            let b = slice_b[idx_b];
            match (a.first.cmp(&b.first), a.second.cmp(&b.second)) {
                (_, std::cmp::Ordering::Greater) => {
                    try_insert(a);
                    idx_a += 1;
                }
                (std::cmp::Ordering::Greater, std::cmp::Ordering::Equal) => {
                    try_insert(a);
                    idx_a += 1;
                    idx_b += 1;
                }
                (_, std::cmp::Ordering::Equal) => {
                    try_insert(b);
                    idx_a += 1;
                    idx_b += 1;
                }
                _ => {
                    try_insert(b);
                    idx_b += 1;
                }
            }
        }

        while idx_a < slice_a.len() {
            try_insert(slice_a[idx_a]);
            idx_a += 1;
        }

        while idx_b < slice_b.len() {
            try_insert(slice_b[idx_b]);
            idx_b += 1;
        }

        &slice_c[0..idx_c]
    }

    #[inline(always)]
    fn trim_slice<'a>(
        mut slice: &'a [ParetoValue<T, U>],
        max_first: T,
        max_second: U,
    ) -> &'a [ParetoValue<T, U>] {
        while slice.len() > 1 && slice[1].second >= max_second {
            (_, slice) = slice.split_first().unwrap();
        }
        while slice.len() > 1 && slice[slice.len() - 2].first >= max_first {
            (_, slice) = slice.split_last().unwrap();
        }
        slice
    }

    /// Saves the last segment to storage and returns an identifier to retrieve the segment
    pub fn save(&mut self) -> Option<ParetoFrontId> {
        match self.segments.last() {
            Some(segment) => {
                self.fronts_generated += 1;
                let slice = &self.buffer[segment.offset..segment.offset + segment.length];
                let segment = Segment {
                    offset: self.storage.len(),
                    length: segment.length,
                };
                self.storage.extend_from_slice(slice);
                Some(segment)
            }
            None => None,
        }
    }

    pub fn peek(&self) -> Option<&[ParetoValue<T, U>]> {
        match self.segments.last() {
            Some(segment) => Some(&self.buffer[segment.offset..segment.offset + segment.length]),
            None => None,
        }
    }

    /// Retrieves a Pareto front from storage
    pub fn retrieve(&self, id: ParetoFrontId) -> &[ParetoValue<T, U>] {
        &self.storage[id.offset..id.offset + id.length]
    }

    pub fn is_max(&self) -> bool {
        match self.segments.last() {
            Some(segment) if segment.length == 1 => {
                let element = self.buffer[segment.offset];
                element.first >= self.max_first && element.second >= self.max_second
            }
            _ => false,
        }
    }

    #[cfg(test)]
    fn check_invariants(&self) {
        for window in self.segments.windows(2) {
            // segments musn't overlap and must have left-to-right ordering
            assert!(window[0].offset + window[0].length <= window[1].offset);
        }
        for segment in self.segments.iter() {
            // each segment must lie entirely in the buffer
            assert!(segment.offset + segment.length <= self.buffer.len());
            // each segment must form a valid pareto front:
            // - first value strictly increasing
            // - second value strictly decreasing
            let slice = &self.buffer[segment.offset..segment.offset + segment.length];
            for window in slice.windows(2) {
                assert!(window[0].first < window[1].first);
                assert!(window[0].second > window[1].second);
            }
        }
        // The buffer must end right after the last segment
        match self.segments.last() {
            Some(segment) => assert_eq!(self.buffer.len(), segment.offset + segment.length),
            None => assert_eq!(self.buffer.len(), 0),
        }
    }
}

impl<T, U> Drop for ParetoFrontBuilder<T, U>
where
    T: Copy + std::cmp::Ord + std::default::Default + std::fmt::Debug,
    U: Copy + std::cmp::Ord + std::default::Default + std::fmt::Debug,
{
    fn drop(&mut self) {
        log::debug!(
            "ParetoFrontBuilder - buffer_size: {}, fronts_generated: {}, storage_size: {}",
            self.buffer.capacity(),
            self.fronts_generated,
            self.storage.len()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    const SAMPLE_FRONT_1: &[ParetoValue<u16, u16>] = &[
        ParetoValue::new(100, 300),
        ParetoValue::new(200, 200),
        ParetoValue::new(300, 100),
    ];

    const SAMPLE_FRONT_2: &[ParetoValue<u16, u16>] = &[
        ParetoValue::new(50, 270),
        ParetoValue::new(150, 250),
        ParetoValue::new(250, 150),
        ParetoValue::new(300, 50),
    ];

    #[test]
    fn test_merge_empty() {
        let mut builder: ParetoFrontBuilder<u16, u16> = ParetoFrontBuilder::new(1000, 2000);
        builder.push_empty();
        builder.push_empty();
        builder.merge();
        let id = builder.save().unwrap();
        let front = builder.retrieve(id);
        assert!(front.as_ref().is_empty());
        builder.check_invariants();
    }

    #[test]
    fn test_value_shift() {
        let mut builder: ParetoFrontBuilder<u16, u16> = ParetoFrontBuilder::new(1000, 2000);
        builder.push_from_slice(SAMPLE_FRONT_1);
        builder.map(move |value| {
            value.first += 100;
            value.second += 100;
        });
        let id = builder.save().unwrap();
        let front = builder.retrieve(id);
        assert_eq!(
            *front,
            [
                ParetoValue::new(200, 400),
                ParetoValue::new(300, 300),
                ParetoValue::new(400, 200),
            ]
        );
        builder.check_invariants();
    }

    #[test]
    fn test_merge() {
        let mut builder: ParetoFrontBuilder<u16, u16> = ParetoFrontBuilder::new(1000, 2000);
        builder.push_from_slice(SAMPLE_FRONT_1);
        builder.push_from_slice(SAMPLE_FRONT_2);
        builder.merge();
        let id = builder.save().unwrap();
        let front = builder.retrieve(id);
        assert_eq!(
            *front,
            [
                ParetoValue::new(100, 300),
                ParetoValue::new(150, 250),
                ParetoValue::new(200, 200),
                ParetoValue::new(250, 150),
                ParetoValue::new(300, 100),
            ]
        );
        builder.check_invariants();
    }

    #[test]
    fn test_merge_truncated() {
        let mut builder: ParetoFrontBuilder<u16, u16> = ParetoFrontBuilder::new(1000, 2000);
        builder.push_from_slice(SAMPLE_FRONT_1);
        builder.map(|value| {
            value.first += 1000;
            value.second += 2000;
        });
        builder.push_from_slice(SAMPLE_FRONT_2);
        builder.map(|value| {
            value.first += 1000;
            value.second += 2000;
        });
        builder.merge();
        let id = builder.save().unwrap();
        let front = builder.retrieve(id);
        assert_eq!(*front, [ParetoValue::new(1300, 2100)]);
        builder.check_invariants();
    }

    #[test]
    fn test_fuzz() {
        let mut rng = rand::thread_rng();
        let mut builder: ParetoFrontBuilder<u16, u16> = ParetoFrontBuilder::new(1000, 2000);
        let mut lut = [0; 5000];

        for _ in 0..200 {
            let cnt = rng.gen_range(1..200);
            for _ in 0..cnt {
                let progress: u16 = rng.gen_range(0..5000);
                let quality: u16 = rng.gen_range(0..10000);
                for i in 0..=progress as usize {
                    lut[i] = std::cmp::max(lut[i], quality);
                }
                builder.push_from_slice(&[ParetoValue::new(progress, quality)]);
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

        let id = builder.save().unwrap();
        let front = builder.retrieve(id);
        for value in front.iter() {
            assert_eq!(lut[value.first as usize], value.second);
        }

        builder.clear();
        builder.check_invariants();
    }
}
