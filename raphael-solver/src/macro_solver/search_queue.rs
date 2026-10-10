use std::collections::{BTreeSet, hash_map::Entry};
use std::sync::Mutex;

use raphael_sim::SimulationState;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use crate::{
    SolverException, SolverSettings,
    actions::{ActionCombo, use_action_combo},
};

use super::pareto_front::ParetoFront;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SearchScore {
    pub quality_upper_bound: u16,
    pub steps_lower_bound: u8,
    pub duration_lower_bound: u8,
    pub current_steps: u8,
    pub current_duration: u8,
}

impl SearchScore {
    pub const MIN: Self = Self {
        quality_upper_bound: 0,
        steps_lower_bound: u8::MAX,
        duration_lower_bound: u8::MAX,
        current_steps: u8::MAX,
        current_duration: u8::MAX,
    };

    pub const MAX: Self = Self {
        quality_upper_bound: u16::MAX,
        steps_lower_bound: 0,
        duration_lower_bound: 0,
        current_steps: 0,
        current_duration: 0,
    };

    /// Search score as a single numerical value for faster comparison.
    const fn ordinal(&self) -> u64 {
        (self.quality_upper_bound as u64) << 32
            | (!self.steps_lower_bound as u64) << 24
            | (!self.duration_lower_bound as u64) << 16
            | (!self.current_steps as u64) << 8
            | (!self.current_duration as u64)
    }
}

impl std::cmp::PartialOrd for SearchScore {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ord::cmp(self, other))
    }
}

impl std::cmp::Ord for SearchScore {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ordinal().cmp(&other.ordinal())
    }
}

#[cfg(target_pointer_width = "32")]
#[bitfield_struct::bitfield(u32)]
struct SearchNode {
    #[bits(26)]
    parent_idx: usize,
    #[bits(6)]
    action: ActionCombo,
}

#[cfg(target_pointer_width = "64")]
#[bitfield_struct::bitfield(u64)]
struct SearchNode {
    #[bits(58)]
    parent_idx: usize,
    #[bits(6)]
    action: ActionCombo,
}

#[derive(Debug, Clone, Copy)]
pub struct Candidate {
    pub score: SearchScore,
    node: SearchNode,
}

impl Candidate {
    pub fn try_new(
        score: SearchScore,
        action: ActionCombo,
        parent_idx: usize,
    ) -> Result<Self, SolverException> {
        let node = SearchNode::new()
            .with_parent_idx_checked(parent_idx)
            .map_err(|_| SolverException::SearchQueueCapacityExceeded)?
            .with_action(action);
        Ok(Self { score, node })
    }
}

/// A range of candidates with equal score inside one candidate list.
#[derive(Debug, Clone, Copy)]
struct CandidateRun {
    score: SearchScore,
    list_idx: u32,
    start: u32,
    end: u32,
}

impl CandidateRun {
    fn len(&self) -> usize {
        (self.end - self.start) as usize
    }

    fn range(&self) -> std::ops::Range<usize> {
        self.start as usize..self.end as usize
    }
}

#[derive(Debug)]
pub struct Batch {
    pub score: SearchScore,
    pub nodes: Vec<(SimulationState, usize)>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SearchQueueStats {
    pub inserted_nodes: usize,
    pub processed_nodes: usize,
}

pub struct SearchQueue {
    settings: SolverSettings,
    pareto_front: ParetoFront,
    batch_ordering: BTreeSet<SearchScore>,
    batches: FxHashMap<SearchScore, Mutex<Vec<SearchNode>>>,
    visited_nodes: Vec<SearchNode>,
    num_inserted_nodes: usize,
    initial_state: SimulationState,
}

impl SearchQueue {
    pub fn new(settings: SolverSettings, initial_state: SimulationState) -> Self {
        let mut search_queue = Self {
            settings,
            pareto_front: ParetoFront::default(),
            batch_ordering: BTreeSet::default(),
            batches: FxHashMap::default(),
            visited_nodes: Vec::new(),
            num_inserted_nodes: 0,
            initial_state,
        };
        if let Ok(root) = Candidate::try_new(SearchScore::MAX, ActionCombo::None, 0) {
            search_queue.push_sorted(&[&[root]]);
        }
        search_queue
    }

    /// Pushes the candidates of all lists into the queue.
    /// Each list is assumed to be already sorted by score ascending.
    pub fn push_sorted(&mut self, candidate_lists: &[&[Candidate]]) {
        // Cut each list into runs of equal score.
        let mut runs: Vec<CandidateRun> = candidate_lists
            .par_iter()
            .enumerate()
            .flat_map_iter(|(list_idx, candidates)| {
                let mut start = 0;
                candidates
                    .chunk_by(|lhs, rhs| lhs.score == rhs.score)
                    .map(move |group| {
                        let run = CandidateRun {
                            score: group[0].score,
                            list_idx: list_idx as u32,
                            start,
                            end: start + group.len() as u32,
                        };
                        start = run.end;
                        run
                    })
            })
            .collect();
        // Stable sort so runs with the same score stay in list order.
        runs.par_sort_by_key(|run| run.score);
        // Create and preallocate memory for the batches.
        for group in runs.chunk_by(|lhs, rhs| lhs.score == rhs.score) {
            let num_nodes = group.iter().map(CandidateRun::len).sum::<usize>();
            let batch = match self.batches.entry(group[0].score) {
                Entry::Occupied(occupied_entry) => occupied_entry.into_mut(),
                Entry::Vacant(vacant_entry) => {
                    self.batch_ordering.insert(group[0].score);
                    vacant_entry.insert(Mutex::default())
                }
            };
            batch.get_mut().unwrap().reserve(num_nodes);
            self.num_inserted_nodes += num_nodes;
        }
        // Every group writes to a different batch so we can copy batches in parallel.
        let batches = &self.batches;
        runs.par_chunk_by(|lhs, rhs| lhs.score == rhs.score)
            .for_each(|group| {
                let mut batch = batches[&group[0].score].lock().unwrap();
                for run in group {
                    let candidates = &candidate_lists[run.list_idx as usize][run.range()];
                    batch.extend(candidates.iter().map(|candidate| candidate.node));
                }
            });
    }

    pub fn drop_nodes_below_score(&mut self, min_score: SearchScore) {
        let mut dropped = 0;
        while let Some(&score) = self.batch_ordering.first()
            && score < min_score
        {
            self.batch_ordering.pop_first();
            dropped += self
                .batches
                .remove(&score)
                .map_or(0, |batch| batch.into_inner().unwrap().len());
        }
        if dropped != 0 {
            log::trace!("{dropped} nodes dropped ({min_score:?})");
        }
    }

    pub fn pop_batch(&mut self) -> Result<Option<Batch>, SolverException> {
        let Some(score) = self.batch_ordering.pop_last() else {
            return Ok(None);
        };
        let Some(batch) = self.batches.remove(&score) else {
            return Ok(None);
        };
        let batch = batch.into_inner().unwrap();
        // Replay actions from the initial state to get the current state.
        let expanded_nodes: Vec<(SearchNode, SimulationState)> = batch
            .into_par_iter()
            .map(|search_node| {
                let mut state = self.initial_state;
                let actions = self.get_actions_from_node_idx(search_node.parent_idx());
                for action in actions {
                    state = use_action_combo(&self.settings, state, action).unwrap();
                }
                state = use_action_combo(&self.settings, state, search_node.action()).unwrap();
                (search_node, state)
            })
            .collect();
        // Filter out Pareto-dominated nodes.
        let non_dominated_nodes = self
            .pareto_front
            .insert_batch(expanded_nodes, |expanded_node| &expanded_node.1)
            .collect::<Vec<_>>();
        // Push and return the remaining nodes.
        let first_idx = self.visited_nodes.len();
        let nodes = non_dominated_nodes
            .iter()
            .enumerate()
            .map(|(offset, expanded_node)| (expanded_node.1, first_idx + offset))
            .collect();
        self.visited_nodes.extend(
            non_dominated_nodes
                .into_iter()
                .map(|expanded_node| expanded_node.0),
        );
        Ok(Some(Batch { score, nodes }))
    }

    pub fn get_actions_from_node_idx(&self, mut idx: usize) -> SmallVec<[ActionCombo; 56]> {
        let mut actions = SmallVec::new();
        while idx > 0 {
            let search_node = self.visited_nodes[idx];
            actions.push(search_node.action());
            idx = search_node.parent_idx();
        }
        actions.reverse();
        actions
    }

    pub fn runtime_stats(&self) -> SearchQueueStats {
        SearchQueueStats {
            inserted_nodes: self.num_inserted_nodes,
            processed_nodes: self.visited_nodes.len(),
        }
    }
}
