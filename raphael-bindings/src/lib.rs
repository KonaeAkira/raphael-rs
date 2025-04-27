use raphael_sim::{ActionMask, Settings};
use raphael_solver::{AtomicFlag, MacroSolver, SolverSettings};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SolveArgs {
    pub on_start: extern "C" fn(*mut bool),
    pub on_finish: extern "C" fn(*const Action, usize),
    pub on_suggest_solution: Option<extern "C" fn(*const Action, usize)>,
    pub on_progress: Option<extern "C" fn(usize)>,
    pub action_mask: u64,
    pub progress: u16,
    pub quality: u16,
    pub base_progress: u16,
    pub base_quality: u16,
    pub cp: i16,
    pub durability: i8,
    pub job_level: u8,
    pub adversarial: bool,
    pub backload_progress: bool,
    pub unsound_branch_pruning: bool,
}

// repr should be identical to raphael_sim::Action
#[repr(u8)]
pub enum Action {
    BasicSynthesis,
    BasicTouch,
    MasterMend,
    Observe,
    TricksOfTheTrade,
    WasteNot,
    Veneration,
    StandardTouch,
    GreatStrides,
    Innovation,
    WasteNot2,
    ByregotsBlessing,
    PreciseTouch,
    MuscleMemory,
    CarefulSynthesis,
    Manipulation,
    PrudentTouch,
    AdvancedTouch,
    Reflect,
    PreparatoryTouch,
    Groundwork,
    DelicateSynthesis,
    IntensiveSynthesis,
    TrainedEye,
    HeartAndSoul,
    PrudentSynthesis,
    TrainedFinesse,
    RefinedTouch,
    QuickInnovation,
    ImmaculateMend,
    TrainedPerfection,
}

// This should produce an error if raphael_sim::Action is changed
impl From<raphael_sim::Action> for Action {
    fn from(value: raphael_sim::Action) -> Self {
        match value {
            raphael_sim::Action::BasicSynthesis => Self::BasicSynthesis,
            raphael_sim::Action::BasicTouch => Self::BasicTouch,
            raphael_sim::Action::MasterMend => Self::MasterMend,
            raphael_sim::Action::Observe => Self::Observe,
            raphael_sim::Action::TricksOfTheTrade => Self::TricksOfTheTrade,
            raphael_sim::Action::WasteNot => Self::WasteNot,
            raphael_sim::Action::Veneration => Self::Veneration,
            raphael_sim::Action::StandardTouch => Self::StandardTouch,
            raphael_sim::Action::GreatStrides => Self::GreatStrides,
            raphael_sim::Action::Innovation => Self::Innovation,
            raphael_sim::Action::WasteNot2 => Self::WasteNot2,
            raphael_sim::Action::ByregotsBlessing => Self::ByregotsBlessing,
            raphael_sim::Action::PreciseTouch => Self::PreciseTouch,
            raphael_sim::Action::MuscleMemory => Self::MuscleMemory,
            raphael_sim::Action::CarefulSynthesis => Self::CarefulSynthesis,
            raphael_sim::Action::Manipulation => Self::Manipulation,
            raphael_sim::Action::PrudentTouch => Self::PrudentTouch,
            raphael_sim::Action::AdvancedTouch => Self::AdvancedTouch,
            raphael_sim::Action::Reflect => Self::Reflect,
            raphael_sim::Action::PreparatoryTouch => Self::PreparatoryTouch,
            raphael_sim::Action::Groundwork => Self::Groundwork,
            raphael_sim::Action::DelicateSynthesis => Self::DelicateSynthesis,
            raphael_sim::Action::IntensiveSynthesis => Self::IntensiveSynthesis,
            raphael_sim::Action::TrainedEye => Self::TrainedEye,
            raphael_sim::Action::HeartAndSoul => Self::HeartAndSoul,
            raphael_sim::Action::PrudentSynthesis => Self::PrudentSynthesis,
            raphael_sim::Action::TrainedFinesse => Self::TrainedFinesse,
            raphael_sim::Action::RefinedTouch => Self::RefinedTouch,
            raphael_sim::Action::QuickInnovation => Self::QuickInnovation,
            raphael_sim::Action::ImmaculateMend => Self::ImmaculateMend,
            raphael_sim::Action::TrainedPerfection => Self::TrainedPerfection,
        }
    }
}

impl From<SolveArgs> for SolverSettings {
    fn from(value: SolveArgs) -> Self {
        let simulator_settings = Settings {
            max_cp: value.cp,
            max_durability: value.durability,
            max_progress: value.progress,
            max_quality: value.quality,
            base_progress: value.base_progress,
            base_quality: value.base_quality,
            job_level: value.job_level,
            allowed_actions: ActionMask::from_bits(value.action_mask),
            adversarial: value.adversarial,
        };
        Self {
            simulator_settings,
            simulator_initial_state: None,
            backload_progress: value.backload_progress,
            allow_unsound_branch_pruning: value.unsound_branch_pruning,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn solve(args: &SolveArgs) {
    let flag = AtomicFlag::new();
    (args.on_start)(flag.as_ptr());

    let settings = SolverSettings::from(*args);
    let solution_callback: Box<dyn Fn(&[raphael_sim::Action])> =
        if let Some(cb) = args.on_suggest_solution {
            Box::new(move |actions| {
                cb(actions.as_ptr() as *const Action, actions.len());
            })
        } else {
            Box::new(|_| {})
        };
    let progress_callback: Box<dyn Fn(usize)> = if let Some(cb) = args.on_progress {
        Box::new(move |progress| {
            cb(progress);
        })
    } else {
        Box::new(|_| {})
    };

    let mut solver = MacroSolver::new(settings, solution_callback, progress_callback, flag.clone());
    let actions = solver.solve().unwrap_or_default();
    (args.on_finish)(actions.as_ptr() as *const Action, actions.len());
}
