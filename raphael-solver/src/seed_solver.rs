//! Warm-start seeding for the macro solver.

use raphael_sim::{Action, SimulationState};
use rayon::prelude::*;
use rustc_hash::FxHashMap;

use crate::actions::{
    ActionCombo, FULL_SEARCH_ACTIONS, PROGRESS_ONLY_SEARCH_ACTIONS, use_action_combo,
};
use crate::{FinishSolver, SolverSettings};

// Arbitrary. On M4 MBA this takes ~300ms. Might be worth benchmarking,
// as weaker hardware might experience too much additional time during
// this search to be worth the savings that only show up in harder crafts
const BEAM_WIDTH: usize = 4096;
const MAX_DEPTH: usize = 40;
/// Only the best `PRE_RANK_WIDTH` candidates (by held resources) of each
/// depth receive a close-out rollout
const PRE_RANK_WIDTH: usize = 4 * BEAM_WIDTH;

/// Cheap state score used for pre-ranking and tie-breaking.
fn resource_score(state: &SimulationState) -> u32 {
    u32::from(state.quality) + 30 * u32::from(state.cp) + 55 * u32::from(state.durability)
}

/// Quality actions that can cash in a freshly applied buff.
/// The rollout only casts a buff if at least one of these is usable afterwards.
const CASH_ACTIONS: [Action; 4] = [
    Action::ByregotsBlessing,
    Action::TrainedFinesse,
    Action::PrudentTouch,
    Action::BasicTouch,
];

fn use_single(
    settings: &SolverSettings,
    state: SimulationState,
    action: Action,
) -> Option<SimulationState> {
    use_action_combo(settings, state, ActionCombo::Single(action)).ok()
}

/// Greedy close-out rollout: spends CP on Quality for as long as the finish
/// solver confirms the craft remains finishable, then completes Progress by
/// walking the finish-solver table.
/// Returns the capped Quality of the finished craft and, if `RECORD` is set,
/// the used actions (otherwise the returned `Vec` is empty).
fn close_out<const RECORD: bool>(
    settings: &SolverSettings,
    finish_solver: &FinishSolver,
    mut state: SimulationState,
) -> Option<(u16, Vec<ActionCombo>)> {
    let finishable = |state: &SimulationState| {
        state.durability > 0 && finish_solver.can_finish(state).unwrap_or(false)
    };
    if !finishable(&state) {
        return None;
    }
    let mut actions = Vec::new();
    // Quality phase: greedy priority ladder. Every step must keep the craft
    // finishable, and a buff must be cashable by at least one follow-up touch.
    'quality: for _ in 0..24 {
        if state.quality >= settings.max_quality() {
            break;
        }
        let effects = state.effects;
        let mut candidates: Vec<Action> = Vec::new();
        if effects.inner_quiet() >= 8 && effects.great_strides() > 0 && effects.innovation() > 0 {
            candidates.push(Action::ByregotsBlessing);
        }
        if effects.innovation() == 0 {
            if effects.quick_innovation_available() {
                candidates.push(Action::QuickInnovation);
            } else {
                candidates.push(Action::Innovation);
            }
        }
        if effects.inner_quiet() == 10 && effects.great_strides() == 0 && effects.innovation() > 1 {
            candidates.push(Action::GreatStrides);
        }
        if effects.inner_quiet() == 10 {
            candidates.push(Action::TrainedFinesse);
        }
        candidates.push(Action::PrudentTouch);
        candidates.push(Action::BasicTouch);
        if effects.inner_quiet() > 0 {
            candidates.push(Action::ByregotsBlessing); // last call before Progress
        }
        for action in candidates {
            let Some(child) = use_single(settings, state, action) else {
                continue;
            };
            if !finishable(&child) {
                continue;
            }
            let is_buff = matches!(
                action,
                Action::Innovation | Action::QuickInnovation | Action::GreatStrides
            );
            if is_buff
                && !CASH_ACTIONS.iter().any(|&touch| {
                    use_single(settings, child, touch).is_some_and(|gc| finishable(&gc))
                })
            {
                continue;
            }
            if RECORD {
                actions.push(ActionCombo::Single(action));
            }
            state = child;
            continue 'quality;
        }
        break; // no quality action keeps the craft finishable
    }
    // Finish phase: extract a finishing line from the finish-solver table.
    // A finishable child always exists here (Bellman property of the table);
    // the refresh guard below can rarely hide it, in which case the rollout
    // is simply discarded.
    for _ in 0..20 {
        if state.progress >= settings.max_progress() {
            return Some((std::cmp::min(state.quality, settings.max_quality()), actions));
        }
        let mut chosen: Option<(SimulationState, ActionCombo)> = None;
        for &combo in &PROGRESS_ONLY_SEARCH_ACTIONS {
            // Refreshing an already-active timer never brings the craft closer
            // to finishing; without this guard the greedy pick below can spin
            // on re-casting buffs (each re-cast is still "finishable").
            let is_pointless_refresh = match combo {
                ActionCombo::Single(Action::Veneration) => state.effects.veneration() > 0,
                ActionCombo::Single(Action::WasteNot | Action::WasteNot2) => {
                    state.effects.waste_not() > 0
                }
                ActionCombo::Single(Action::Manipulation) => state.effects.manipulation() > 0,
                ActionCombo::Single(Action::StellarSteadyHand) => {
                    state.effects.stellar_steady_hand() > 0
                }
                _ => false,
            };
            if is_pointless_refresh {
                continue;
            }
            let Ok(child) = use_action_combo(settings, state, combo) else {
                continue;
            };
            // Rank by progress gained, then durability (so a mend is chosen
            // exactly when it heals and no progress action is viable).
            if (child.progress >= settings.max_progress() || finishable(&child))
                && chosen.as_ref().is_none_or(|(best, _)| {
                    (child.progress, child.durability) > (best.progress, best.durability)
                })
            {
                chosen = Some((child, combo));
            }
        }
        let (child, combo) = chosen?;
        if RECORD {
            actions.push(combo);
        }
        state = child;
    }
    None
}

/// Produces a complete rotation to be used as the macro solver's initial
/// solution, or `None` if no complete rotation was found.
pub fn beam_seed(
    settings: &SolverSettings,
    finish_solver: &FinishSolver,
) -> Option<Vec<ActionCombo>> {
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
                if best
                    .as_ref()
                    .is_none_or(|(best_quality, _)| quality > *best_quality)
                {
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

        let mut candidates: Vec<(SimulationState, usize, ActionCombo)> = dedup
            .into_iter()
            .map(|(child, (parent_index, combo))| (child, parent_index, combo))
            .collect();
        if candidates.len() > PRE_RANK_WIDTH {
            candidates.par_sort_unstable_by_key(|&(state, ..)| {
                (
                    std::cmp::Reverse(resource_score(&state)),
                    state.effects.into_bits(),
                    state.cp,
                    state.durability,
                    state.progress,
                    state.quality,
                )
            });
            candidates.truncate(PRE_RANK_WIDTH);
        }

        // Rank candidates by the quality of their close-out rollout
        let mut scored: Vec<(u16, u32, SimulationState, usize, ActionCombo)> = candidates
            .into_par_iter()
            .filter_map(|(child, parent_index, combo)| {
                let (rollout_quality, _) = close_out::<false>(settings, finish_solver, child)?;
                Some((
                    rollout_quality,
                    resource_score(&child),
                    child,
                    parent_index,
                    combo,
                ))
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

        if let Some(&(rollout_quality, _, state, parent_index, combo)) = scored.first()
            && best
                .as_ref()
                .is_none_or(|(best_quality, _)| rollout_quality > *best_quality)
            && let Some((quality, rollout)) = close_out::<true>(settings, finish_solver, state)
        {
            let mut combos = rotation_of(&arena, parent_index);
            combos.push(combo);
            combos.extend(rollout);
            best = Some((quality, combos));
        }

        frontier = scored
            .into_iter()
            .map(|(_, _, child, parent_index, combo)| {
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
