//! Warm-start seeding for the macro solver.
//!
//! `beam_seed` runs a small beam search over the solver's action alphabet to
//! produce a complete, valid rotation before the main search starts. The macro
//! solver replays the seed and installs it as the initial solution, giving the
//! search a non-trivial score lower-bound from the first batch instead of
//! discovering one gradually. This only reduces the number of nodes inserted
//! into the search queue; nodes required to prove optimality are unaffected,
//! so the final solution is unchanged.
//!
//! Beam states are ranked by the quality of a greedy "close-out" rollout
//! (finishing the craft from that state immediately), not by their held
//! resources: a partial rotation is only worth what it can be converted into.
//! The rollout of the best-ranked state doubles as the seed candidate.

use raphael_sim::{Action, Condition, Settings, SimulationState};
use rayon::prelude::*;
use rustc_hash::FxHashMap;

use crate::SolverSettings;
use crate::actions::{ActionCombo, FULL_SEARCH_ACTIONS, use_action_combo};

// Arbitrary. On M4 MBA this takes ~300ms. Might be worth benchmarking,
// as weaker hardware might experience too much additional time during
// this search to be worth the savings that only show up in harder crafts
const BEAM_WIDTH: usize = 4096;
const MAX_DEPTH: usize = 40;

const PROGRESS_ACTIONS: [Action; 6] = [
    Action::RapidSynthesis,
    Action::Groundwork,
    Action::PrudentSynthesis,
    Action::CarefulSynthesis,
    Action::DelicateSynthesis,
    Action::BasicSynthesis,
];

/// Greedily completes Progress from `state`.
/// Returns the capped Quality of the finished craft and the used actions.
fn close_progress(
    settings: &Settings,
    mut state: SimulationState,
    mut actions: Vec<Action>,
) -> Option<(u16, Vec<Action>)> {
    for _ in 0..20 {
        if state.progress >= settings.max_progress {
            return Some((std::cmp::min(state.quality, settings.max_quality), actions));
        }
        // Any single action that finishes the craft right now, cheapest CP first.
        let mut finisher: Option<(u16, Action, SimulationState)> = None;
        for action in PROGRESS_ACTIONS {
            if let Ok(child) = state.use_action(action, Condition::Normal, settings)
                && child.progress >= settings.max_progress
            {
                let cp_cost = state.cp - child.cp;
                if finisher.as_ref().is_none_or(|(cost, _, _)| cp_cost < *cost) {
                    finisher = Some((cp_cost, action, child));
                }
            }
        }
        if let Some((_, action, child)) = finisher {
            actions.push(action);
            state = child;
            continue;
        }
        // Build-up: Stellar Steady Hand, Veneration, a mend if durability-starved,
        // then the strongest affordable Progress action.
        let mut candidates: Vec<Action> = Vec::new();
        if state.effects.stellar_steady_hand_charges() > 0
            && state.effects.stellar_steady_hand() == 0
        {
            candidates.push(Action::StellarSteadyHand);
        }
        if state.effects.veneration() == 0 && state.cp >= 18 + 7 {
            candidates.push(Action::Veneration);
        }
        if state.durability <= 10 {
            if state.cp >= 112 && settings.max_durability - state.durability > 30 {
                candidates.push(Action::ImmaculateMend);
            }
            if state.cp >= 88 {
                candidates.push(Action::MasterMend);
            }
        }
        candidates.extend(PROGRESS_ACTIONS);
        let mut advanced = false;
        for action in candidates {
            if let Ok(child) = state.use_action(action, Condition::Normal, settings) {
                let is_progress_action = !matches!(
                    action,
                    Action::StellarSteadyHand
                        | Action::Veneration
                        | Action::ImmaculateMend
                        | Action::MasterMend
                );
                if is_progress_action
                    && child.durability == 0
                    && child.progress < settings.max_progress
                {
                    continue; // would deadlock the rollout
                }
                actions.push(action);
                state = child;
                advanced = true;
                break;
            }
        }
        if !advanced {
            return None;
        }
    }
    None
}

/// Greedy close-out rollout: spends CP on Quality while reserving enough
/// resources to complete Progress, then completes Progress.
/// Returns the capped Quality of the finished craft and the used actions.
fn close_out(settings: &Settings, mut state: SimulationState) -> Option<(u16, Vec<Action>)> {
    let mut actions = Vec::new();
    // Resources reserved for the Progress phase of the rollout.
    let (reserve_cp, reserve_durability): (u16, u16) =
        if state.effects.stellar_steady_hand_charges() > 0
            || state.effects.stellar_steady_hand() >= 3
        {
            // Veneration + guaranteed Rapid Synthesis spam
            (18, 30)
        } else {
            // Veneration + Careful Synthesis spam
            (18 + 9 * 7, 90)
        };
    for _ in 0..24 {
        if state.quality >= settings.max_quality {
            break;
        }
        let effects = state.effects;
        let mut budget = state.cp.saturating_sub(reserve_cp);
        if state.durability < reserve_durability {
            budget = budget.saturating_sub(88); // price in a Master's Mend
        }
        let action = if budget >= 24
            && effects.inner_quiet() >= 8
            && effects.great_strides() > 0
            && effects.innovation() > 0
        {
            Some(Action::ByregotsBlessing)
        } else if budget >= 50 && effects.innovation() == 0 {
            if effects.quick_innovation_available() {
                Some(Action::QuickInnovation)
            } else {
                Some(Action::Innovation)
            }
        } else if budget >= 56
            && effects.inner_quiet() == 10
            && effects.great_strides() == 0
            && effects.innovation() > 1
        {
            Some(Action::GreatStrides)
        } else if budget >= 32 && effects.inner_quiet() == 10 {
            Some(Action::TrainedFinesse)
        } else if budget >= 25 && state.durability > 5 && effects.waste_not() == 0 {
            Some(Action::PrudentTouch)
        } else if budget >= 18 && state.durability > 10 {
            Some(Action::BasicTouch)
        } else if budget >= 24 && effects.inner_quiet() > 0 {
            Some(Action::ByregotsBlessing)
        } else {
            None
        };
        let Some(action) = action else { break };
        let Ok(child) = state.use_action(action, Condition::Normal, settings) else {
            break;
        };
        if child.durability == 0 {
            break;
        }
        actions.push(action);
        state = child;
    }
    close_progress(settings, state, actions)
}

/// Produces a complete rotation to be used as the macro solver's initial
/// solution, or `None` if no complete rotation was found.
pub fn beam_seed(settings: &SolverSettings) -> Option<Vec<ActionCombo>> {
    let sim_settings = &settings.simulator_settings;
    let initial_state = SimulationState::new(sim_settings);

    // Arena of (parent index, combo). Rotations are reconstructed by walking
    // parent links; index 0 is the root sentinel.
    let mut arena: Vec<(usize, ActionCombo)> = vec![(usize::MAX, ActionCombo::None)];
    let mut frontier: Vec<(SimulationState, usize)> = vec![(initial_state, 0)];
    let mut best: Option<(u16, Vec<ActionCombo>)> = None;

    let rotation_of = |arena: &[(usize, ActionCombo)], mut index: usize| -> Vec<ActionCombo> {
        let mut combos = Vec::new();
        while index != 0 {
            let (parent, combo) = arena[index];
            combos.push(combo);
            index = parent;
        }
        combos.reverse();
        combos
    };

    for _ in 0..MAX_DEPTH {
        let expanded: Vec<(SimulationState, usize, ActionCombo)> = frontier
            .par_iter()
            .flat_map_iter(|&(state, parent_index)| {
                FULL_SEARCH_ACTIONS.iter().filter_map(move |&combo| {
                    use_action_combo(settings, state, combo)
                        .ok()
                        .map(|child| (child, parent_index, combo))
                })
            })
            .collect();

        let mut dedup: FxHashMap<SimulationState, (usize, ActionCombo)> = FxHashMap::default();
        for (child, parent_index, combo) in expanded {
            if child.progress >= sim_settings.max_progress {
                // Completed within the beam itself.
                let quality = std::cmp::min(child.quality, sim_settings.max_quality);
                if best.as_ref().is_none_or(|(best_quality, _)| quality > *best_quality) {
                    let mut combos = rotation_of(&arena, parent_index);
                    combos.push(combo);
                    best = Some((quality, combos));
                }
            } else if child.durability != 0 {
                dedup.entry(child).or_insert((parent_index, combo));
            }
        }
        if dedup.is_empty() {
            break;
        }

        // Rank candidates by the quality of their close-out rollout.
        let mut scored: Vec<(u16, u32, SimulationState, usize, ActionCombo, Vec<Action>)> = dedup
            .into_par_iter()
            .filter_map(|(child, (parent_index, combo))| {
                let (rollout_quality, rollout) = close_out(sim_settings, child)?;
                let tie_break = u32::from(child.quality)
                    + 30 * u32::from(child.cp)
                    + 55 * u32::from(child.durability);
                Some((rollout_quality, tie_break, child, parent_index, combo, rollout))
            })
            .collect();
        // The final state-derived key makes truncation deterministic across runs.
        scored.par_sort_unstable_by_key(|&(quality, tie_break, state, ..)| {
            (
                std::cmp::Reverse(quality),
                std::cmp::Reverse(tie_break),
                state.effects.into_bits(),
                state.cp,
                state.durability,
                state.progress,
                state.quality,
            )
        });
        scored.truncate(BEAM_WIDTH);

        if let Some((rollout_quality, _, _, parent_index, combo, rollout)) = scored.first()
            && best.as_ref().is_none_or(|(best_quality, _)| *rollout_quality > *best_quality)
        {
            let mut combos = rotation_of(&arena, *parent_index);
            combos.push(*combo);
            combos.extend(rollout.iter().copied().map(ActionCombo::Single));
            best = Some((*rollout_quality, combos));
        }

        frontier = scored
            .into_iter()
            .map(|(_, _, child, parent_index, combo, _)| {
                arena.push((parent_index, combo));
                (child, arena.len() - 1)
            })
            .collect();

        if best
            .as_ref()
            .is_some_and(|(quality, _)| *quality >= sim_settings.max_quality)
        {
            break;
        }
    }
    best.map(|(_, combos)| combos)
}
