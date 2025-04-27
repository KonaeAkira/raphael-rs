use crate::actions::*;
use crate::effects::*;
use crate::{Condition, Settings};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimulationState {
    pub cp: i16,
    pub durability: i16,
    pub progress: u32,
    pub quality: u32,            // previous unguarded action = Poor
    pub unreliable_quality: u32, // previous unguarded action = Normal, diff with quality
    pub effects: Effects,
}

impl SimulationState {
    pub fn new(settings: &Settings) -> Self {
        Self {
            cp: settings.max_cp,
            durability: i16::from(settings.max_durability),
            progress: 0,
            quality: 0,
            unreliable_quality: 0,
            effects: Effects::initial(settings),
        }
    }

    pub fn from_macro(
        settings: &Settings,
        actions: &[Action],
        initial_state: Option<SimulationState>,
    ) -> Result<Self, &'static str> {
        let mut state = initial_state.unwrap_or_else(|| SimulationState::new(settings));
        for action in actions {
            state = state.use_action(*action, settings)?;
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
            state = match state.use_action(*action, settings) {
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
        self.durability <= 0 || self.progress >= u32::from(settings.max_progress)
    }

    fn check_common_preconditions<A: ActionImpl>(
        &self,
        settings: &Settings,
    ) -> Result<(), &'static str> {
        if settings.job_level < A::LEVEL_REQUIREMENT {
            Err("Level not high enough")
        } else if !settings.allowed_actions.has_mask(A::ACTION_MASK) {
            Err("Action disabled by action mask")
        } else if self.is_final(settings) {
            // println!("{:?}", self);
            Err("State is final")
        } else if A::cp_cost(self, settings, self.effects.condition()) > self.cp {
            Err("Not enough CP")
        } else {
            Ok(())
        }
    }

    pub fn use_action_impl<A: ActionImpl>(
        &self,
        settings: &Settings,
    ) -> Result<Self, &'static str> {
        self.check_common_preconditions::<A>(settings)?;
        let condition = self.effects.condition();
        A::precondition(self, settings, condition)?;

        let mut state = *self;

        A::transform_pre(&mut state, settings, condition);

        if A::base_durability_cost(&state, settings) != 0 {
            state.durability -= A::durability_cost(self, settings, condition);
            state.effects.set_trained_perfection_active(false);
        }

        state.cp -= A::cp_cost(self, settings, condition);

        let progress_increase = A::progress_increase(self, settings, condition);
        state.progress += progress_increase;
        if progress_increase != 0 {
            state.effects.set_muscle_memory(0);
        }

        let quality_increase = A::quality_increase(self, settings, condition);
        if settings.adversarial {
            let adversarial_quality_increase = if state.effects.guard() != 0 {
                quality_increase
            } else {
                A::quality_increase(self, settings, Condition::Poor)
            };
            if state.effects.guard() == 0 && adversarial_quality_increase == 0 {
                state.unreliable_quality = 0;
            } else if state.effects.guard() != 0 && adversarial_quality_increase != 0 {
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

        if state.is_final(settings) {
            return Ok(state);
        }

        if A::TICK_EFFECTS {
            if state.effects.manipulation() != 0 {
                state.durability =
                    std::cmp::min(i16::from(settings.max_durability), state.durability + 5);
            }
            state.effects.tick_down();
        }

        if settings.adversarial && quality_increase != 0 {
            state.effects.set_guard(1);
        }

        A::transform_post(&mut state, settings, condition);

        state
            .effects
            .set_combo(A::combo(&state, settings, condition));

        state
            .effects
            .set_condition(self.effects.condition().follow_up_condition());

        Ok(state)
    }

    pub fn use_action(&self, action: Action, settings: &Settings) -> Result<Self, &'static str> {
        match action {
            Action::BasicSynthesis => self.use_action_impl::<BasicSynthesis>(settings),
            Action::BasicTouch => self.use_action_impl::<BasicTouch>(settings),
            Action::MasterMend => self.use_action_impl::<MasterMend>(settings),
            Action::Observe => self.use_action_impl::<Observe>(settings),
            Action::TricksOfTheTrade => self.use_action_impl::<TricksOfTheTrade>(settings),
            Action::WasteNot => self.use_action_impl::<WasteNot>(settings),
            Action::Veneration => self.use_action_impl::<Veneration>(settings),
            Action::StandardTouch => self.use_action_impl::<StandardTouch>(settings),
            Action::GreatStrides => self.use_action_impl::<GreatStrides>(settings),
            Action::Innovation => self.use_action_impl::<Innovation>(settings),
            Action::WasteNot2 => self.use_action_impl::<WasteNot2>(settings),
            Action::ByregotsBlessing => self.use_action_impl::<ByregotsBlessing>(settings),
            Action::PreciseTouch => self.use_action_impl::<PreciseTouch>(settings),
            Action::MuscleMemory => self.use_action_impl::<MuscleMemory>(settings),
            Action::CarefulSynthesis => self.use_action_impl::<CarefulSynthesis>(settings),
            Action::Manipulation => self.use_action_impl::<Manipulation>(settings),
            Action::PrudentTouch => self.use_action_impl::<PrudentTouch>(settings),
            Action::AdvancedTouch => self.use_action_impl::<AdvancedTouch>(settings),
            Action::Reflect => self.use_action_impl::<Reflect>(settings),
            Action::PreparatoryTouch => self.use_action_impl::<PreparatoryTouch>(settings),
            Action::Groundwork => self.use_action_impl::<Groundwork>(settings),
            Action::DelicateSynthesis => self.use_action_impl::<DelicateSynthesis>(settings),
            Action::IntensiveSynthesis => self.use_action_impl::<IntensiveSynthesis>(settings),
            Action::TrainedEye => self.use_action_impl::<TrainedEye>(settings),
            Action::HeartAndSoul => self.use_action_impl::<HeartAndSoul>(settings),
            Action::PrudentSynthesis => self.use_action_impl::<PrudentSynthesis>(settings),
            Action::TrainedFinesse => self.use_action_impl::<TrainedFinesse>(settings),
            Action::RefinedTouch => self.use_action_impl::<RefinedTouch>(settings),
            Action::QuickInnovation => self.use_action_impl::<QuickInnovation>(settings),
            Action::ImmaculateMend => self.use_action_impl::<ImmaculateMend>(settings),
            Action::TrainedPerfection => self.use_action_impl::<TrainedPerfection>(settings),
        }
    }
}
