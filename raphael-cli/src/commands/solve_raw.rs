use clap::Args;
use raphael_sim::{Action, ActionMask, Combo, Condition, Effects, Settings, SimulationState};
use raphael_solver::{AtomicFlag, MacroSolver, SolverSettings};

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum ConditionArgument {
    Normal,
    Good,
    Excellent,
    Poor,
    // Next Condition is Good
    GoodOmen,
    // Half Durability Loss
    Sturdy,
    // Increase Success of RNG Skills by 25%
    Centered,
    // CP Cost reduced by 50%
    Pliant,
    // Buff lasts 2 steps longer
    Primed,
    // Progress + 50%
    Malleable,
}

impl Into<Condition> for ConditionArgument {
    fn into(self) -> Condition {
        match self {
            Self::Normal => Condition::Normal,
            Self::Good => Condition::Good,
            Self::Excellent => Condition::Excellent,
            Self::Poor => Condition::Poor,
            Self::GoodOmen => Condition::GoodOmen,
            Self::Sturdy => Condition::Sturdy,
            Self::Centered => Condition::Centered,
            Self::Pliant => Condition::Pliant,
            Self::Primed => Condition::Primed,
            Self::Malleable => Condition::Malleable,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum ComboArg {
    None,
    SynthesisBegin,
    BasicTouch,
    StandardTouch,
}

impl Into<Combo> for ComboArg {
    fn into(self) -> Combo {
        match self {
            Self::None => Combo::None,
            Self::SynthesisBegin => Combo::SynthesisBegin,
            Self::BasicTouch => Combo::BasicTouch,
            Self::StandardTouch => Combo::StandardTouch,
        }
    }
}

#[derive(Args, Debug)]
pub struct RawSolveArgs {
    /// Maximum progress of the craft
    #[arg(long)]
    pub max_progress: u16,

    /// Maximum quality of the craft
    #[arg(long)]
    pub max_quality: u16,

    /// Maximum durability of the craft
    #[arg(long)]
    pub max_durability: u16,

    /// Progress per 100%. can be derived from craftsmanship and the level of the recipe normally. Must already include food and potion.
    #[arg(short = 'p', long)]
    pub base_progress: u16,

    /// Quality per 100%. Can be derived from control and the level of the recipe normally. Must already include food and potion.
    #[arg(short = 'q', long)]
    pub base_quality: u16,

    /// Maximum crafting points. Must already include food and potion.
    #[arg(short = 'c', long)]
    pub max_cp: u16,

    /// Current progress of the recipe. It will be truncated to the maximum progress, if more than the maximum progress.
    #[arg(long, default_value_t = 0)]
    pub current_progress: u32,

    /// Current quality of the recipe. It will be truncated to the maximum quality, if more than the maximum quality.
    #[arg(long, default_value_t = 0)]
    pub current_quality: u32,

    /// Current durability of the recipe. It will be truncated to the maximum durability, if more than the maximum durability.
    #[arg(long, default_value = None)]
    pub current_durability: Option<u16>,

    /// Currently remaining crafting points. It will be truncated to the maximum cp, if more than the maximum cp.
    #[arg(long, default_value = None)]
    pub current_cp: Option<u16>,

    /// Number of remaining steps the great strides buff is still active
    #[arg(long, default_value_t = 0)]
    pub great_strides_steps: u8,

    /// Indicates that the heart and soul buff is currently active
    #[arg(long, default_value_t = false)]
    pub heart_and_soul_active: bool,

    /// Indicates that the heart and soul is not longer available
    #[arg(long, default_value_t = false)]
    pub heart_and_soul_unavailable: bool,

    /// Number of remaining steps the innovation buff is still active
    #[arg(long, default_value_t = 0)]
    pub innovation_steps: u8,

    /// Number of inner quiet stacks
    #[arg(long, default_value_t = 0)]
    pub inner_quiet_stacks: u8,

    /// Number of remaining steps manipulation buff is still active
    #[arg(long, default_value_t = 0)]
    pub manipulation_steps: u8,

    /// Number of remaining steps the muscle memory buff is still active
    #[arg(long, default_value_t = 0)]
    pub muscle_memory_steps: u8,

    /// Indicates that quick innovation is not longer available
    #[arg(long, default_value_t = false)]
    pub quick_innovation_unavailable: bool,

    /// Indicates that trained perfection buff is currently active
    #[arg(long, default_value_t = false)]
    pub trained_perfection_active: bool,

    /// Indicates that trained perfection is not longer available
    #[arg(long, default_value_t = false)]
    pub trained_perfection_unavailable: bool,

    /// Number of remaining steps veneration buff is still active
    #[arg(long, default_value_t = 0)]
    pub veneration_steps: u8,

    /// Number of remaining steps waste not buff is still active
    #[arg(long, default_value_t = 0)]
    pub waste_not_steps: u8,

    /// Current combo of the craft. synthesis-begin is a special condition that is only available at the start of the craft. Indicates that Muscle Memory or Reflect may be used.
    #[arg(long, value_enum, default_value_t = ComboArg::SynthesisBegin)]
    pub combo: ComboArg,

    /// Current condition of the craft.
    #[arg(long, value_enum, default_value_t = ConditionArgument::Normal)]
    pub condition: ConditionArgument,

    /// Crafter level
    #[arg(short, long, default_value_t = 100)]
    pub level: u8,

    /// Enable Manipulation
    #[arg(short, long, default_value_t = false)]
    pub manipulation: bool,

    /// Enable Heart and Soul
    #[arg(long, default_value_t = false)]
    pub heart_and_soul: bool,

    /// Enable Quick Innovation
    #[arg(long, default_value_t = false)]
    pub quick_innovation: bool,

    /// Enable Trained Eye
    #[arg(long, default_value_t = false)]
    pub trained_eye: bool,

    /// Enable adversarial simulator (ensure 100% reliability)
    #[arg(long, default_value_t = false)]
    pub adversarial: bool,

    /// Only use Progress-increasing actions at the end of the macro
    #[arg(long, default_value_t = false)]
    pub backload_progress: bool,

    /// Enable unsound branch pruning
    #[arg(long, default_value_t = false)]
    pub unsound: bool,

    /// Output the provided list of variables. The output is deliminated by the output-field-separator
    ///
    /// <IDENTIFIER> can be any of the following: `item_id`, `recipe`, `food`, `potion`, `craftsmanship`, `control`, `cp`, `crafter_stats`, `settings`, `initial_quality`, `target_quality`, `recipe_max_quality`, `actions`, `final_state`, `state_quality`, `final_quality`, `steps`, `duration`.
    /// While the output is mainly intended for generating CSVs, some output can contain `,` inside brackets that are not deliminating columns. For this reason they are wrapped in double quotes and the argument `output-field-separator` can be used to override the delimiter to something that is easier to parse and process
    #[arg(long, num_args = 1.., value_name = "IDENTIFIER")]
    pub output_variables: Vec<String>,

    /// The delimiter the output specified with the argument `output-format` uses to separate identifiers
    #[arg(long, alias = "OFS", default_value = ",", env = "OFS")]
    output_field_separator: String,
}

pub fn execute(args: &RawSolveArgs) {
    let mut allowed_actions = ActionMask::all();
    if !args.manipulation {
        allowed_actions = allowed_actions.remove(Action::Manipulation);
    }
    if !args.heart_and_soul {
        allowed_actions = allowed_actions.remove(Action::HeartAndSoul);
    }
    if !args.quick_innovation_unavailable {
        allowed_actions = allowed_actions.remove(Action::QuickInnovation);
    }
    if !args.trained_eye {
        allowed_actions = allowed_actions.remove(Action::TrainedEye);
    }

    let settings = Settings {
        max_cp: args.max_cp,
        max_durability: args.max_durability,
        max_progress: args.max_progress,
        max_quality: args.max_quality,
        base_progress: args.base_progress,
        base_quality: args.base_quality,
        job_level: args.level,
        allowed_actions,
        adversarial: args.adversarial,
    };

    let effects = Effects::new()
        .with_combo(args.combo.into())
        .with_condition(args.condition.into())
        .with_heart_and_soul_active(args.heart_and_soul_active)
        .with_heart_and_soul_available(!args.heart_and_soul_unavailable)
        .with_great_strides(args.great_strides_steps)
        .with_inner_quiet(args.inner_quiet_stacks)
        .with_innovation(args.innovation_steps)
        .with_manipulation(args.manipulation_steps)
        .with_muscle_memory(args.muscle_memory_steps)
        .with_quick_innovation_available(!args.quick_innovation_unavailable)
        .with_trained_perfection_active(args.trained_perfection_active)
        .with_trained_perfection_available(!args.trained_perfection_unavailable)
        .with_waste_not(args.waste_not_steps)
        .with_veneration(args.veneration_steps);

    let initial_state = SimulationState {
        cp: args.current_cp.unwrap_or(args.max_cp),
        durability: args.current_durability.unwrap_or(args.max_durability),
        progress: args.current_progress,
        quality: args.current_quality,
        unreliable_quality: args.current_quality,
        effects,
    };

    let solver_settings = SolverSettings {
        simulator_settings: settings,
        simulator_initial_state: Some(initial_state),
        backload_progress: args.backload_progress,
        allow_unsound_branch_pruning: args.unsound,
    };

    let mut solver = MacroSolver::new(
        solver_settings,
        Box::new(|_| {}),
        Box::new(|_| {}),
        AtomicFlag::new(),
    );
    let actions = solver.solve().expect("Failed to solve");

    let final_state =
        SimulationState::from_macro(&settings, &actions, Some(initial_state)).unwrap();
    let state_quality = final_state.quality;
    let final_quality = state_quality;
    let steps = actions.len();
    let duration: u8 = actions.iter().map(|action| action.time_cost()).sum();

    if args.output_variables.is_empty() {
        println!("Quality: {}/{}", final_quality, settings.max_quality);
        println!(
            "Progress: {}/{}",
            final_state.progress, settings.max_progress
        );
        println!("Steps: {}", steps);
        println!("Duration: {} seconds", duration);
        println!("\nActions:");
        for action in actions {
            println!("{:?}", action);
        }
    } else {
        let mut output_string = "".to_owned();

        //let output_format = args.output_variables.clone().unwrap();
        //let segments: Vec<&str> = args.output_variables;
        for identifier in &args.output_variables {
            let map_to_debug_str = |actions: Vec<raphael_sim::Action>| match &*(*identifier) {
                "settings" => format!("\"{:?}\"", settings),
                "actions" => format!("\"{:?}\"", actions),
                "final_state" => format!("\"{:?}\"", final_state),
                "state_quality" => format!("{:?}", state_quality),
                "steps" => format!("{:?}", steps),
                "duration" => format!("{:?}", duration),
                _ => "Undefined".to_owned(),
            };

            output_string += &(map_to_debug_str(actions.clone()) + &args.output_field_separator);
        }

        println!(
            "{}",
            output_string.trim_end_matches(&args.output_field_separator)
        );
    }
}
