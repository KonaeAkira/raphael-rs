use crate::{Action, ComboAction, Condition, Effects, Settings};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrainedPerfectionState {
    Available,
    Active,
    Used,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimulationState {
    pub cp: i16,
    pub durability: i8,
    pub missing_progress: u16,
    pub missing_quality: u16,
    pub effects: Effects,
    pub combo: Option<ComboAction>,
    pub trained_perfection: TrainedPerfectionState,
}

impl SimulationState {
    pub fn new(settings: &Settings) -> Self {
        Self {
            cp: settings.max_cp,
            durability: settings.max_durability,
            missing_progress: settings.max_progress,
            missing_quality: settings
                .max_quality
                .saturating_sub(settings.initial_quality),
            effects: Default::default(),
            combo: Some(ComboAction::SynthesisBegin),
            trained_perfection: TrainedPerfectionState::Available,
        }
    }

    pub fn from_macro(settings: &Settings, actions: &[Action]) -> Result<Self, &'static str> {
        let mut state = Self::new(settings);
        for action in actions {
            let in_progress: InProgress = state.try_into()?;
            state = in_progress.use_action(*action, Condition::Normal, settings)?;
        }
        Ok(state)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InProgress {
    state: SimulationState,
}

impl TryFrom<SimulationState> for InProgress {
    type Error = &'static str;

    fn try_from(value: SimulationState) -> Result<Self, Self::Error> {
        if value.missing_progress == 0 {
            return Err("Progress already at 100%");
        }
        if value.durability <= 0 {
            return Err("No remaining durability");
        }
        Ok(Self { state: value })
    }
}

impl InProgress {
    pub fn new(settings: &Settings) -> Self {
        Self {
            state: SimulationState::new(settings),
        }
    }

    pub fn raw_state(&self) -> &SimulationState {
        &self.state
    }

    pub fn can_use_action(&self, action: Action, condition: Condition) -> Result<(), &'static str> {
        if action.cp_cost(&self.state.effects, condition) > self.state.cp {
            return Err("Not enough CP");
        }
        if !action.combo_fulfilled(self.state.combo) {
            return Err("Combo requirement not fulfilled");
        }
        match action {
            Action::ByregotsBlessing if self.state.effects.inner_quiet == 0 => {
                Err("Need Inner Quiet to use Byregot's Blessing")
            }
            Action::PrudentSynthesis | Action::PrudentTouch
                if self.state.effects.waste_not != 0 =>
            {
                Err("Action cannot be used during Waste Not")
            }
            Action::IntensiveSynthesis | Action::PreciseTouch | Action::TricksOfTheTrade
                if condition != Condition::Good && condition != Condition::Excellent =>
            {
                Err("Requires condition to be Good or Excellent")
            }
            Action::Groundwork
                if self.state.durability
                    < action.durability_cost(&self.state.effects, condition) =>
            {
                Err("Not enough durability")
            }
            Action::TrainedFinesse if self.state.effects.inner_quiet < 10 => {
                Err("Requires 10 Inner Quiet")
            }
            Action::TrainedPerfection
                if !matches!(
                    self.state.trained_perfection,
                    TrainedPerfectionState::Available
                ) =>
            {
                Err("Action can only be used once per synthesis")
            }
            _ => Ok(()),
        }
    }

    pub fn use_action(
        self,
        action: Action,
        condition: Condition,
        settings: &Settings,
    ) -> Result<SimulationState, &'static str> {
        self.can_use_action(action, condition)?;
        let mut state = self.state;

        let cp_cost = action.cp_cost(&state.effects, condition);
        let durability_cost = match state.trained_perfection {
            TrainedPerfectionState::Active => 0,
            _ => action.durability_cost(&state.effects, condition),
        };
        let progress_increase = action.progress_increase(settings, &state.effects, condition);
        let quality_increase = action.quality_increase(settings, &state.effects, condition);

        state.combo = action.to_combo();
        state.cp -= cp_cost;
        state.durability -= durability_cost;
        match state.trained_perfection {
            TrainedPerfectionState::Active => {
                state.trained_perfection = TrainedPerfectionState::Used
            }
            TrainedPerfectionState::Available if action == Action::TrainedPerfection => {
                state.trained_perfection = TrainedPerfectionState::Active
            }
            _ => (),
        };

        // reset muscle memory if progress increased
        if progress_increase != 0 {
            state.missing_progress = state.missing_progress.saturating_sub(progress_increase);
            state.effects.muscle_memory = 0;
        }

        // reset great strides and increase inner quiet if quality increased
        if quality_increase != 0 {
            state.missing_quality = state.missing_quality.saturating_sub(quality_increase);
            state.effects.great_strides = 0;
            if settings.job_level >= 11 {
                state.effects.inner_quiet += match action {
                    Action::Reflect => 2,
                    Action::PreciseTouch => 2,
                    Action::PreparatoryTouch => 2,
                    Action::ComboRefinedTouch => 2,
                    _ => 1,
                };
                state.effects.inner_quiet = std::cmp::min(10, state.effects.inner_quiet);
            }
        }

        if state.missing_progress == 0 || state.durability <= 0 {
            return Ok(state);
        }

        // remove manipulation before it is triggered
        if action == Action::Manipulation {
            state.effects.manipulation = 0;
        }

        if state.effects.manipulation > 0 {
            state.durability = std::cmp::min(state.durability + 5, settings.max_durability);
        }
        state.effects.tick_down();

        // trigger special action effects
        let duration_bonus = if condition == Condition::Pliant { 2 } else { 0 };
        match action {
            Action::MuscleMemory => state.effects.muscle_memory = 5 + duration_bonus,
            Action::GreatStrides => state.effects.great_strides = 3 + duration_bonus,
            Action::Veneration => state.effects.veneration = 4 + duration_bonus,
            Action::Innovation => state.effects.innovation = 4 + duration_bonus,
            Action::WasteNot => state.effects.waste_not = 4 + duration_bonus,
            Action::WasteNot2 => state.effects.waste_not = 8 + duration_bonus,
            Action::Manipulation => state.effects.manipulation = 8 + duration_bonus,
            Action::MasterMend => {
                state.durability = std::cmp::min(settings.max_durability, state.durability + 30)
            }
            Action::ByregotsBlessing => state.effects.inner_quiet = 0,
            Action::TricksOfTheTrade => state.cp = std::cmp::min(settings.max_cp, state.cp + 20),
            Action::ImmaculateMend => state.durability = settings.max_durability,
            _ => (),
        }

        Ok(state)
    }
}
