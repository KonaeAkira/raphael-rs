mod atomic_flag;
mod pareto_front_builder;

pub use atomic_flag::AtomicFlag;
pub use pareto_front_builder::{ParetoFrontBuilder, ParetoValue};

pub struct ScopedTimer {
    name: &'static str,
    timer: web_time::Instant,
}

impl ScopedTimer {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            timer: web_time::Instant::now(),
        }
    }
}

impl Drop for ScopedTimer {
    fn drop(&mut self) {
        log::info!(
            "Timer \"{}\" elapsed: {} seconds",
            self.name,
            self.timer.elapsed().as_secs_f32()
        );
    }
}

struct Entry<T> {
    item: T,
    depth: u8,
    parent_index: usize,
}

pub struct Backtracking<T: Copy> {
    entries: Vec<Entry<T>>,
}

impl<T: Copy> Backtracking<T> {
    pub const SENTINEL: usize = usize::MAX;

    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get_items(&self, mut index: usize) -> impl Iterator<Item = T> {
        let mut items = Vec::new();
        while index != Self::SENTINEL {
            items.push(self.entries[index].item);
            index = self.entries[index].parent_index;
        }
        items.into_iter().rev()
    }

    pub fn push(&mut self, item: T, parent_index: usize) -> usize {
        let depth = if parent_index == Self::SENTINEL {
            1
        } else {
            self.entries[parent_index].depth + 1
        };
        self.entries.push(Entry {
            item,
            depth,
            parent_index,
        });
        self.entries.len() - 1
    }
}

impl<T: Copy> Drop for Backtracking<T> {
    fn drop(&mut self) {
        log::debug!("Backtracking - nodes: {}", self.entries.len());
    }
}
