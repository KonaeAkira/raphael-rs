use simulator::{ActionMask, Settings, SimulationState};
use solvers::{AtomicFlag, MacroSolver};

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

// repr should be identical to simulator::Action
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

// This should produce an error if simulator::Action is changed
impl From<simulator::Action> for Action {
    fn from(value: simulator::Action) -> Self {
        match value {
            simulator::Action::BasicSynthesis => Self::BasicSynthesis,
            simulator::Action::BasicTouch => Self::BasicTouch,
            simulator::Action::MasterMend => Self::MasterMend,
            simulator::Action::Observe => Self::Observe,
            simulator::Action::TricksOfTheTrade => Self::TricksOfTheTrade,
            simulator::Action::WasteNot => Self::WasteNot,
            simulator::Action::Veneration => Self::Veneration,
            simulator::Action::StandardTouch => Self::StandardTouch,
            simulator::Action::GreatStrides => Self::GreatStrides,
            simulator::Action::Innovation => Self::Innovation,
            simulator::Action::WasteNot2 => Self::WasteNot2,
            simulator::Action::ByregotsBlessing => Self::ByregotsBlessing,
            simulator::Action::PreciseTouch => Self::PreciseTouch,
            simulator::Action::MuscleMemory => Self::MuscleMemory,
            simulator::Action::CarefulSynthesis => Self::CarefulSynthesis,
            simulator::Action::Manipulation => Self::Manipulation,
            simulator::Action::PrudentTouch => Self::PrudentTouch,
            simulator::Action::AdvancedTouch => Self::AdvancedTouch,
            simulator::Action::Reflect => Self::Reflect,
            simulator::Action::PreparatoryTouch => Self::PreparatoryTouch,
            simulator::Action::Groundwork => Self::Groundwork,
            simulator::Action::DelicateSynthesis => Self::DelicateSynthesis,
            simulator::Action::IntensiveSynthesis => Self::IntensiveSynthesis,
            simulator::Action::TrainedEye => Self::TrainedEye,
            simulator::Action::HeartAndSoul => Self::HeartAndSoul,
            simulator::Action::PrudentSynthesis => Self::PrudentSynthesis,
            simulator::Action::TrainedFinesse => Self::TrainedFinesse,
            simulator::Action::RefinedTouch => Self::RefinedTouch,
            simulator::Action::QuickInnovation => Self::QuickInnovation,
            simulator::Action::ImmaculateMend => Self::ImmaculateMend,
            simulator::Action::TrainedPerfection => Self::TrainedPerfection,
        }
    }
}

impl From<SolveArgs> for Settings {
    fn from(value: SolveArgs) -> Self {
        Self {
            max_cp: value.cp,
            max_durability: value.durability,
            max_progress: value.progress,
            max_quality: value.quality,
            base_progress: value.base_progress,
            base_quality: value.base_quality,
            job_level: value.job_level,
            allowed_actions: ActionMask::from_bits(value.action_mask),
            adversarial: value.adversarial,
        }
    }
}

#[no_mangle]
pub extern "C" fn solve(args: &SolveArgs) {
    let flag = AtomicFlag::new();
    (args.on_start)(flag.as_ptr());

    let settings = Settings::from(*args);
    let solution_callback: Box<dyn Fn(&[simulator::Action])> =
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

    let state = SimulationState::new(&settings);

    let mut solver = MacroSolver::new(
        settings,
        args.backload_progress,
        args.unsound_branch_pruning,
        solution_callback,
        progress_callback,
        flag.clone(),
    );

    let actions = solver.solve(state).unwrap_or_default();
    (args.on_finish)(actions.as_ptr() as *const Action, actions.len());
}
