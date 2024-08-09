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
struct Segment {
    pub offset: usize,
    pub length: usize,
}

pub struct ParetoFrontBuilder<T, U>
where
    T: Copy + std::cmp::Ord + std::default::Default,
    U: Copy + std::cmp::Ord + std::default::Default,
{
    buffer: Vec<ParetoValue<T, U>>,
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
    T: Copy + std::cmp::Ord + std::default::Default,
    U: Copy + std::cmp::Ord + std::default::Default,
{
    pub fn new(max_first: T, max_second: U) -> Self {
        Self {
            buffer: Vec::new(),
            segments: Vec::new(),
            max_first,
            max_second,
            fronts_generated: 0,
            values_generated: 0,
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

    pub fn push(&mut self, values: &[ParetoValue<T, U>]) {
        let segment = Segment {
            offset: self.buffer.len(),
            length: values.len(),
        };
        self.segments.push(segment);
        self.buffer.extend_from_slice(values);
    }

    /// Modify each element of the last segment in-place.
    /// Panics in case there are no segments.
    pub fn map<F>(&mut self, f: F)
    where
        F: Fn(&mut ParetoValue<T, U>),
    {
        let segment = self.segments.last().unwrap();
        let slice = &mut self.buffer[segment.offset..segment.offset + segment.length];
        slice.iter_mut().for_each(f);
    }

    /// Merge the last two segments into one.
    /// Panics in case there are fewer than two segments.
    pub fn merge(&mut self) {
        assert!(self.segments.len() >= 2);
        let segment_b = self.segments.pop().unwrap();
        let segment_a = self.segments.pop().unwrap();

        #[cfg(test)]
        assert_eq!(self.buffer.len(), segment_b.offset + segment_b.length);

        let length_c = segment_a.length + segment_b.length;
        let offset_c = if segment_a.offset + segment_a.length + length_c <= segment_b.offset {
            // sandwich C between A and B
            segment_a.offset + segment_a.length
        } else {
            // allocate C after B
            self.buffer
                .resize(self.buffer.len() + length_c, Default::default());
            segment_b.offset + segment_b.length
        };

        let (slice_l, slice_r) = self.buffer.split_at_mut(offset_c);
        let (slice_c, slice_r) = slice_r.split_at_mut(length_c);
        let slice_a = &slice_l[segment_a.offset..segment_a.offset + segment_a.length];
        let slice_b = if segment_b.offset <= offset_c {
            &slice_l[segment_b.offset..segment_b.offset + segment_b.length]
        } else {
            let offset = segment_b.offset - offset_c - length_c;
            &slice_r[offset..offset + segment_b.length]
        };

        let mut head_a: usize = 0;
        let mut head_b: usize = 0;
        let mut head_c: usize = 0;
        let mut tail_c: usize = 0;

        let mut rolling_max: Option<T> = None;
        let mut try_insert = |x: ParetoValue<T, U>| {
            if rolling_max.is_none() || x.first > rolling_max.unwrap() {
                rolling_max = Some(x.first);
                slice_c[tail_c] = x;
                tail_c += 1;
            }
        };

        while head_a < slice_a.len() && head_b < slice_b.len() {
            match slice_a[head_a].second.cmp(&slice_b[head_b].second) {
                std::cmp::Ordering::Less => {
                    try_insert(slice_b[head_b]);
                    head_b += 1;
                }
                std::cmp::Ordering::Equal => {
                    let first = std::cmp::max(slice_a[head_a].first, slice_b[head_b].first);
                    let second = slice_a[head_a].second;
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

        // cut out values in front that are over max_second
        while head_c + 1 < tail_c && slice_c[head_c + 1].second >= self.max_second {
            head_c += 1;
        }

        // cut out values in the back that are over max_first
        while head_c + 1 < tail_c && slice_c[tail_c - 2].first >= self.max_first {
            tail_c -= 1;
        }

        let segment_c = Segment {
            offset: offset_c + head_c,
            length: tail_c - head_c,
        };

        self.segments.push(segment_c);
        self.buffer.truncate(segment_c.offset + segment_c.length);
    }

    pub fn peek(&mut self) -> Option<Box<[ParetoValue<T, U>]>> {
        match self.segments.last() {
            Some(segment) => {
                self.fronts_generated += 1;
                self.values_generated += segment.length;
                let slice = &self.buffer[segment.offset..segment.offset + segment.length];
                Some(slice.into())
            }
            None => None,
        }
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
            // each segment must form a valid pareto front
            let slice = &self.buffer[segment.offset..segment.offset + segment.length];
            for window in slice.windows(2) {
                assert!(window[0].first < window[1].first);
                assert!(window[0].second > window[1].second);
            }
        }
    }
}

impl<T, U> Drop for ParetoFrontBuilder<T, U>
where
    T: Copy + std::cmp::Ord + std::default::Default,
    U: Copy + std::cmp::Ord + std::default::Default,
{
    fn drop(&mut self) {
        dbg!(
            self.buffer.capacity(),
            self.fronts_generated,
            self.values_generated
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
        let front = builder.peek().unwrap();
        assert!(front.as_ref().is_empty());
        builder.check_invariants();
    }

    #[test]
    fn test_value_shift() {
        let mut builder: ParetoFrontBuilder<u16, u16> = ParetoFrontBuilder::new(1000, 2000);
        builder.push(SAMPLE_FRONT_1);
        builder.map(move |value| {
            value.first += 100;
            value.second += 100;
        });
        let front = builder.peek().unwrap();
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
        builder.push(SAMPLE_FRONT_1);
        builder.push(SAMPLE_FRONT_2);
        builder.merge();
        let front = builder.peek().unwrap();
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
        builder.push(SAMPLE_FRONT_1);
        builder.map(|value| {
            value.first += 1000;
            value.second += 2000;
        });
        builder.push(SAMPLE_FRONT_2);
        builder.map(|value| {
            value.first += 1000;
            value.second += 2000;
        });
        builder.merge();
        let front = builder.peek().unwrap();
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
