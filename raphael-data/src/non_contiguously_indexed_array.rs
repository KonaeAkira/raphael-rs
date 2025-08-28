pub struct NciArray<V: 'static> {
    pub index_range_starting_indices: &'static [u32],
    pub index_range_skip_amounts: &'static [u32],
    pub values: &'static [V],
}

impl<V> std::ops::Index<u32> for NciArray<V> {
    type Output = V;

    fn index(&self, index: u32) -> &Self::Output {
        self.get(index).unwrap()
    }
}

pub struct NciArrayIndexIter {
    index_range_starting_indices: &'static [u32],
    index_range_skip_amounts: &'static [u32],
    index: u32,
    skipped: u32,
    next_range_index: usize,
    next_index_range_starting_index: Option<&'static u32>,
    next_index_range_skip_amount: Option<&'static u32>,
    true_index: usize, // MAYBE no longer needed
    value_count: usize,
}

impl NciArrayIndexIter {
    fn new(
        index_range_starting_indices: &'static [u32],
        index_range_skip_amounts: &'static [u32],
        value_count: usize,
    ) -> Self {
        if let (Some(initial_index), Some(initial_skip_amount)) = (
            index_range_starting_indices.get(0),
            index_range_skip_amounts.get(0),
        ) {
            let next_index_range_starting_index = index_range_starting_indices.get(1);
            let next_index_range_skip_amount = index_range_skip_amounts.get(1);

            Self {
                index_range_starting_indices,
                index_range_skip_amounts,
                index: *initial_index,
                skipped: *initial_skip_amount,
                next_range_index: 1,
                next_index_range_starting_index,
                next_index_range_skip_amount,
                true_index: 0,
                value_count,
            }
        } else {
            Self {
                index_range_starting_indices,
                index_range_skip_amounts,
                index: u32::MAX,
                skipped: u32::MAX,
                next_range_index: usize::MAX,
                next_index_range_starting_index: None,
                next_index_range_skip_amount: None,
                true_index: usize::MAX,
                value_count,
            }
        }
    }
}

impl Iterator for NciArrayIndexIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.true_index < self.value_count {
            let value = self.index;

            self.index += 1;
            if let (Some(next_index_range_starting_index), Some(next_index_range_skip_amount)) = (
                self.next_index_range_starting_index,
                self.next_index_range_skip_amount,
            ) {
                if next_index_range_starting_index - self.index
                    <= next_index_range_skip_amount - self.skipped
                {
                    self.index = *next_index_range_starting_index;
                    self.skipped = *next_index_range_skip_amount;

                    self.next_range_index += 1;
                    self.next_index_range_starting_index =
                        self.index_range_starting_indices.get(self.next_range_index);
                    self.next_index_range_skip_amount =
                        self.index_range_skip_amounts.get(self.next_range_index);
                }
            }

            self.true_index += 1;
            Some(value)
        } else {
            None
        }
    }
}

impl<V> NciArray<V> {
    pub fn values(&self) -> core::slice::Iter<'static, V> {
        self.values.iter()
    }

    pub fn indices(&self) -> NciArrayIndexIter {
        NciArrayIndexIter::new(
            self.index_range_starting_indices,
            self.index_range_skip_amounts,
            self.values.len(),
        )
    }

    pub fn entries(&self) -> std::iter::Zip<NciArrayIndexIter, core::slice::Iter<'static, V>> {
        self.indices().zip(self.values())
    }

    pub fn has_entry(&self, index: u32) -> bool {
        let range_index = match self.index_range_starting_indices.binary_search(&index) {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };

        let index_range_starting_index = self
            .index_range_starting_indices
            .get(range_index)
            .cloned()
            .unwrap_or_default();
        let index_range_skipped = self
            .index_range_skip_amounts
            .get(range_index)
            .cloned()
            .unwrap_or_default();

        let true_starting_index = index_range_starting_index - index_range_skipped;

        let index_range_end =
            if let (Some(next_index_range_starting_index), Some(next_index_range_skip_amount)) = (
                self.index_range_starting_indices.get(range_index + 1),
                self.index_range_skip_amounts.get(range_index + 1),
            ) {
                index_range_starting_index
                    + (next_index_range_starting_index - next_index_range_skip_amount)
                    - true_starting_index
            } else {
                index_range_starting_index + self.values.len() as u32 - true_starting_index
            };

        index >= index_range_starting_index && index < index_range_end
    }

    pub fn get(&self, index: u32) -> Option<&V> {
        let range_index = match self.index_range_starting_indices.binary_search(&index) {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };

        let index_range_starting_index = self
            .index_range_starting_indices
            .get(range_index)
            .cloned()
            .unwrap_or_default();
        let index_range_skipped = self
            .index_range_skip_amounts
            .get(range_index)
            .cloned()
            .unwrap_or_default();
        let slice_start = (index_range_starting_index - index_range_skipped) as usize;
        let slice_end =
            if let (Some(next_index_range_starting_index), Some(next_index_range_skip_amount)) = (
                self.index_range_starting_indices.get(range_index + 1),
                self.index_range_skip_amounts.get(range_index + 1),
            ) {
                (next_index_range_starting_index - next_index_range_skip_amount) as usize
            } else {
                self.values.len()
            };
        let slice: &[V] = &self.values[slice_start..slice_end];
        // Simply subtracting here is techically not correct for all NciArrays
        // Breaks if di := index - index_range_starting_index < 0 & slice_end >= usize::MAX + di
        // This should never be occur in our use case, so not checking di < 0 skips an unnecessary branch
        slice.get((index as usize).wrapping_sub(index_range_starting_index as usize))
    }
}
