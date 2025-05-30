use crate::actions::*;
use crate::effects::*;
use crate::{Condition, Settings};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimulationState {
    pub cp: u16,
    pub durability: u16,
    pub progress: u32,
    pub quality: u32,            // previous unguarded action = Poor
    pub unreliable_quality: u32, // previous unguarded action = Normal, diff with quality
    pub effects: Effects,
}

impl SimulationState {
    pub fn new(settings: &Settings) -> Self {
        Self {
            cp: settings.max_cp,
            durability: settings.max_durability,
            progress: 0,
            quality: 0,
            unreliable_quality: 0,
            effects: Effects::initial(settings),
        }
    }

    pub fn from_macro(settings: &Settings, actions: &[Action]) -> Result<Self, &'static str> {
        let mut state = Self::new(settings);
        for action in actions {
            state = state.use_action(*action, Condition::Normal, settings)?;
        }
        Ok(state)
    }

    pub fn from_macro_continue_on_error(
        settings: &Settings,
        actions: &[Action],
    ) -> (Self, Vec<Result<(), &'static str>>) {
        let mut state = Self::new(settings);
        let mut errors = Vec::new();
        for action in actions {
            state = match state.use_action(*action, Condition::Normal, settings) {
                Ok(new_state) => {
                    errors.push(Ok(()));
                    new_state
                }
                Err(err) => {
                    errors.push(Err(err));
                    state
                }
            };
        }
        (state, errors)
    }

    pub fn is_final(&self, settings: &Settings) -> bool {
        self.durability == 0 || self.progress >= u32::from(settings.max_progress)
    }

    fn check_common_preconditions<A: ActionImpl>(
        &self,
        settings: &Settings,
        condition: Condition,
    ) -> Result<(), &'static str> {
        if settings.job_level < A::LEVEL_REQUIREMENT {
            Err("Level not high enough")
        } else if !settings.allowed_actions.has_mask(A::ACTION_MASK) {
            Err("Action disabled by action mask")
        } else if self.is_final(settings) {
            Err("State is final")
        } else if A::cp_cost(self, settings, condition) > self.cp {
            Err("Not enough CP")
        } else {
            Ok(())
        }
    }

    pub fn use_action_impl<A: ActionImpl>(
        &self,
        settings: &Settings,
        condition: Condition,
    ) -> Result<Self, &'static str> {
        self.check_common_preconditions::<A>(settings, condition)?;
        A::precondition(self, settings, condition)?;

        let mut state = *self;

        A::transform_pre(&mut state, settings, condition);

        if A::base_durability_cost(&state, settings) != 0 {
            state.durability = state
                .durability
                .saturating_sub(A::durability_cost(self, settings, condition));
            state.effects.set_trained_perfection_active(false);
        }

        state.cp -= A::cp_cost(self, settings, condition);

        let quality_increase = A::quality_increase(self, settings, condition);
        if !state.effects.allow_quality_actions() && quality_increase != 0 {
            return Err("Forbidden by backload_progress setting");
        }
        if settings.adversarial {
            let adversarial_quality_increase = if state.effects.adversarial_guard() {
                quality_increase
            } else {
                A::quality_increase(self, settings, Condition::Poor)
            };
            if !state.effects.adversarial_guard() && adversarial_quality_increase == 0 {
                state.unreliable_quality = 0;
            } else if state.effects.adversarial_guard() && adversarial_quality_increase != 0 {
                state.quality += adversarial_quality_increase;
                state.unreliable_quality = 0;
            } else if adversarial_quality_increase != 0 {
                let quality_diff = quality_increase - adversarial_quality_increase;
                state.quality += adversarial_quality_increase
                    + std::cmp::min(state.unreliable_quality, quality_diff);
                state.unreliable_quality = quality_diff.saturating_sub(state.unreliable_quality);
            }
        } else {
            state.quality += quality_increase;
        }
        if quality_increase != 0 && settings.job_level >= 11 {
            state.effects.set_great_strides(0);
            state
                .effects
                .set_inner_quiet(std::cmp::min(10, state.effects.inner_quiet() + 1));
        }

        let progress_increase = A::progress_increase(self, settings);
        state.progress += progress_increase;
        if progress_increase != 0 && state.effects.muscle_memory() != 0 {
            state.effects.set_muscle_memory(0);
        }

        if progress_increase != 0 && settings.backload_progress {
            state.effects.set_allow_quality_actions(false);
        }

        if state.is_final(settings) {
            return Ok(state);
        }

        if A::TICK_EFFECTS {
            if state.effects.manipulation() != 0 {
                state.durability = std::cmp::min(settings.max_durability, state.durability + 5);
            }
            state.effects = state.effects.tick_down();
        }

        if settings.adversarial && quality_increase != 0 {
            state.effects.set_adversarial_guard(true);
        }

        A::transform_post(&mut state, settings, condition);

        state
            .effects
            .set_combo(A::combo(&state, settings, condition));

        if !state.effects.allow_quality_actions() {
            state.unreliable_quality = 0;
            state.effects = state.effects.strip_quality_effects();
        }

        Ok(state)
    }

    pub fn use_action(
        &self,
        action: Action,
        condition: Condition,
        settings: &Settings,
    ) -> Result<Self, &'static str> {
        match action {
            Action::BasicSynthesis => self.use_action_impl::<BasicSynthesis>(settings, condition),
            Action::BasicTouch => self.use_action_impl::<BasicTouch>(settings, condition),
            Action::MasterMend => self.use_action_impl::<MasterMend>(settings, condition),
            Action::Observe => self.use_action_impl::<Observe>(settings, condition),
            Action::TricksOfTheTrade => {
                self.use_action_impl::<TricksOfTheTrade>(settings, condition)
            }
            Action::WasteNot => self.use_action_impl::<WasteNot>(settings, condition),
            Action::Veneration => self.use_action_impl::<Veneration>(settings, condition),
            Action::StandardTouch => self.use_action_impl::<StandardTouch>(settings, condition),
            Action::GreatStrides => self.use_action_impl::<GreatStrides>(settings, condition),
            Action::Innovation => self.use_action_impl::<Innovation>(settings, condition),
            Action::WasteNot2 => self.use_action_impl::<WasteNot2>(settings, condition),
            Action::ByregotsBlessing => {
                self.use_action_impl::<ByregotsBlessing>(settings, condition)
            }
            Action::PreciseTouch => self.use_action_impl::<PreciseTouch>(settings, condition),
            Action::MuscleMemory => self.use_action_impl::<MuscleMemory>(settings, condition),
            Action::CarefulSynthesis => {
                self.use_action_impl::<CarefulSynthesis>(settings, condition)
            }
            Action::Manipulation => self.use_action_impl::<Manipulation>(settings, condition),
            Action::PrudentTouch => self.use_action_impl::<PrudentTouch>(settings, condition),
            Action::AdvancedTouch => self.use_action_impl::<AdvancedTouch>(settings, condition),
            Action::Reflect => self.use_action_impl::<Reflect>(settings, condition),
            Action::PreparatoryTouch => {
                self.use_action_impl::<PreparatoryTouch>(settings, condition)
            }
            Action::Groundwork => self.use_action_impl::<Groundwork>(settings, condition),
            Action::DelicateSynthesis => {
                self.use_action_impl::<DelicateSynthesis>(settings, condition)
            }
            Action::IntensiveSynthesis => {
                self.use_action_impl::<IntensiveSynthesis>(settings, condition)
            }
            Action::TrainedEye => self.use_action_impl::<TrainedEye>(settings, condition),
            Action::HeartAndSoul => self.use_action_impl::<HeartAndSoul>(settings, condition),
            Action::PrudentSynthesis => {
                self.use_action_impl::<PrudentSynthesis>(settings, condition)
            }
            Action::TrainedFinesse => self.use_action_impl::<TrainedFinesse>(settings, condition),
            Action::RefinedTouch => self.use_action_impl::<RefinedTouch>(settings, condition),
            Action::QuickInnovation => self.use_action_impl::<QuickInnovation>(settings, condition),
            Action::ImmaculateMend => self.use_action_impl::<ImmaculateMend>(settings, condition),
            Action::TrainedPerfection => {
                self.use_action_impl::<TrainedPerfection>(settings, condition)
            }
        }
    }
}
