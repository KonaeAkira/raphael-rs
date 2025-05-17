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

pub struct ParetoFrontBuilder<T, U>
where
    T: Copy + std::cmp::Ord + std::default::Default + std::fmt::Debug,
    U: Copy + std::cmp::Ord + std::default::Default + std::fmt::Debug,
{
    segments: Vec<usize>, // indices to the beginning of each segment
    buffer: Vec<ParetoValue<T, U>>,
    merge_buffer: [ParetoValue<T, U>; 512],
    // cut-off values
    max_first: T,
    max_second: U,
}

impl<T, U> ParetoFrontBuilder<T, U>
where
    T: Copy + std::cmp::Ord + std::default::Default + std::fmt::Debug,
    U: Copy + std::cmp::Ord + std::default::Default + std::fmt::Debug,
{
    pub fn new(max_first: T, max_second: U) -> Self {
        Self {
            segments: Vec::new(),
            buffer: Vec::new(),
            merge_buffer: [ParetoValue::default(); 512],
            max_first,
            max_second,
        }
    }

    pub fn clear(&mut self) {
        self.segments.clear();
        self.buffer.clear();
    }

    pub fn push_empty(&mut self) {
        self.segments.push(self.buffer.len());
    }

    pub fn push_slice(&mut self, values: &[ParetoValue<T, U>]) {
        self.segments.push(self.buffer.len());
        self.buffer.extend_from_slice(values);
    }

    /// Merges the last two segments into one.
    /// Panics in case there are fewer than two segments.
    pub fn merge(&mut self) {
        assert!(self.segments.len() >= 2);
        let begin_b = self.segments.pop().unwrap();
        let begin_a = self.segments.last().copied().unwrap();

        let mut begin_c = 0;
        let mut end_c = {
            assert!(begin_a <= begin_b && begin_b <= self.buffer.len());
            let (buffer, slice_b) = self.buffer.split_at_mut(begin_b);
            let (_buffer, slice_a) = buffer.split_at_mut(begin_a);

            // look for the first non-dominated element in slice_b
            // slice_a[..idx_a] + slice_b[idx_b] form the first elements in the merged segment
            // slice_b[..idx_b] are dominated by elements in slice_a and should be discarded
            let (idx_a, idx_b) = Self::find_first_non_dominated(slice_a, slice_b);

            if idx_b >= slice_b.len() {
                // slice_a fully dominates slice_b
                self.buffer.truncate(begin_b);
                return;
            }

            if idx_a >= slice_a.len() {
                // merge result is all of slice_a and slice_b[idx_b..]
                let cnt_a = slice_a.len();
                self.merge_buffer[..cnt_a].copy_from_slice(slice_a);
                let cnt_b = slice_b.len() - idx_b;
                self.merge_buffer[cnt_a..cnt_a + cnt_b].copy_from_slice(&slice_b[idx_b..]);
                cnt_a + cnt_b
            } else {
                // normal merge case
                let (merged, slice_a) = slice_a.split_at_mut(idx_a);
                self.merge_buffer[..idx_a].copy_from_slice(merged);
                let (_discarded, slice_b) = slice_b.split_at_mut(idx_b);
                let (merged, slice_b) = slice_b.split_first_mut().unwrap();
                self.merge_buffer[idx_a] = *merged;
                Self::merge_mixed(
                    slice_a,
                    slice_b,
                    &mut self.merge_buffer,
                    idx_a + 1,
                    merged.first,
                )
            }
        };

        assert!(end_c <= self.merge_buffer.len());
        while begin_c + 1 < end_c && self.merge_buffer[begin_c + 1].second >= self.max_second {
            begin_c += 1;
        }
        while begin_c + 1 < end_c && self.merge_buffer[end_c - 2].first >= self.max_first {
            end_c -= 1;
        }

        let length_c = end_c - begin_c;
        self.buffer.truncate(begin_a + length_c);
        self.buffer[begin_a..].copy_from_slice(&self.merge_buffer[begin_c..end_c]);
    }

    /// Find the first element of slice_b that is not dominated by slice_a
    #[inline(always)]
    fn find_first_non_dominated(
        // slice_a and slice_b are marked &mut to tell the compiler that they are disjoint
        slice_a: &mut [ParetoValue<T, U>],
        slice_b: &mut [ParetoValue<T, U>],
    ) -> (usize, usize) {
        if slice_b.is_empty() {
            return (slice_a.len(), slice_b.len());
        }
        let mut idx_a = 0;
        let mut idx_b = 0;
        while idx_a < slice_a.len() {
            let a = slice_a[idx_a];
            loop {
                let b = slice_b[idx_b];
                if a.first >= b.first && a.second >= b.second {
                    idx_b += 1;
                    if idx_b >= slice_b.len() {
                        return (slice_a.len(), slice_b.len());
                    }
                } else if b.second >= a.second {
                    return (idx_a, idx_b);
                } else {
                    break;
                }
            }
            idx_a += 1;
        }
        (slice_a.len(), idx_b)
    }

    #[inline(always)]
    fn merge_mixed(
        // slices are marked &mut to tell the compiler that they are disjoint
        slice_a: &mut [ParetoValue<T, U>],
        slice_b: &mut [ParetoValue<T, U>],
        slice_c: &mut [ParetoValue<T, U>],
        mut idx_c: usize,
        mut rolling_max: T,
    ) -> usize {
        assert!(slice_a.len() + slice_b.len() <= slice_c.len());

        let mut idx_a = 0;
        let mut idx_b = 0;

        let mut try_insert = |x: ParetoValue<T, U>| {
            if rolling_max < x.first {
                rolling_max = x.first;
                unsafe {
                    #[cfg(test)]
                    assert!(idx_c < slice_c.len());
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

        idx_c
    }

    pub fn peek(&self) -> Option<&[ParetoValue<T, U>]> {
        self.segments
            .last()
            .map(|&segment_begin| &self.buffer[segment_begin..])
    }

    pub fn peek_mut(&mut self) -> Option<&mut [ParetoValue<T, U>]> {
        self.segments
            .last()
            .map(|&segment_begin| &mut self.buffer[segment_begin..])
    }

    pub fn is_max(&self) -> bool {
        match self.segments.last().copied() {
            Some(segment_begin) if segment_begin + 1 == self.buffer.len() => {
                let element = self.buffer.last().unwrap();
                element.first >= self.max_first && element.second >= self.max_second
            }
            _ => false,
        }
    }

    #[cfg(test)]
    fn check_invariants(&self) {
        for window in self.segments.windows(2) {
            // segments must have left-to-right ordering
            assert!(window[0] <= window[1]);
        }
        let mut segment_end = self.buffer.len();
        for segment_begin in self.segments.iter().rev().copied() {
            assert!(segment_begin <= segment_end);
            // each segment must form a valid pareto front:
            // - first value strictly increasing
            // - second value strictly decreasing
            let slice = &self.buffer[segment_begin..segment_end];
            for window in slice.windows(2) {
                assert!(window[0].first < window[1].first);
                assert!(window[0].second > window[1].second);
            }
            segment_end = segment_begin;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::seq::SliceRandom;

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
    fn test_merge() {
        let mut builder: ParetoFrontBuilder<u16, u16> = ParetoFrontBuilder::new(1000, 2000);
        builder.push_slice(SAMPLE_FRONT_1);
        builder.push_slice(SAMPLE_FRONT_2);
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
    fn test_merge_truncate() {
        let mut builder: ParetoFrontBuilder<u16, u16> = ParetoFrontBuilder::new(1000, 2000);
        builder.push_slice(&[
            ParetoValue::new(1100, 2300),
            ParetoValue::new(1200, 2200),
            ParetoValue::new(1300, 2100),
        ]);
        builder.push_slice(&[
            ParetoValue::new(1050, 2270),
            ParetoValue::new(1150, 2250),
            ParetoValue::new(1250, 2150),
            ParetoValue::new(1300, 2050),
        ]);
        builder.merge();
        let front = builder.peek().unwrap();
        assert_eq!(*front, [ParetoValue::new(1300, 2100)]);
        builder.check_invariants();
    }

    #[test]
    fn test_merge_fuzz() {
        let mut rng = rand::rng();
        let mut values_first: Vec<usize> = (1..100).collect();
        let mut values_second: Vec<usize> = (1..100).collect();
        let mut random_values = |n: usize| -> Vec<ParetoValue<_, _>> {
            values_first.shuffle(&mut rng);
            values_second.shuffle(&mut rng);
            let mut values_first: Vec<_> = values_first.iter().copied().take(n).collect();
            let mut values_second: Vec<_> = values_second.iter().copied().take(n).collect();
            values_first.sort();
            values_second.sort_by_key(|x| std::cmp::Reverse(*x));
            values_first
                .iter()
                .zip(values_second.iter())
                .map(|(x, y)| ParetoValue::new(*x, *y))
                .collect()
        };

        for _ in 0..1000 {
            let values_a = random_values(10);
            let values_b = random_values(10);

            let mut lut = [0; 101];
            let mut expected_result = Vec::new();
            for a in values_a.iter().copied() {
                lut[a.first] = std::cmp::max(lut[a.first], a.second);
            }
            for b in values_b.iter().copied() {
                lut[b.first] = std::cmp::max(lut[b.first], b.second);
            }
            for i in (0..100).rev() {
                lut[i] = std::cmp::max(lut[i], lut[i + 1]);
            }
            for i in 0..100 {
                if lut[i] != lut[i + 1] {
                    expected_result.push(ParetoValue::new(i, lut[i]));
                }
            }

            let mut builder = ParetoFrontBuilder::new(usize::MAX, usize::MAX);
            builder.push_slice(&values_a);
            builder.check_invariants();
            builder.push_slice(&values_b);
            builder.check_invariants();
            builder.merge();
            builder.check_invariants();

            let result = builder.peek().unwrap();
            assert_eq!(result, &expected_result);
        }
    }
}
