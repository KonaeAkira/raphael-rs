use std::sync::{Arc, LazyLock, Mutex};

use log::Log;
use raphael_sim::{ActionMask, Settings};
use raphael_solver::{AtomicFlag, MacroSolver, SolverSettings};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SolveArgs {
    pub on_start: extern "C" fn(*mut bool),
    pub on_finish: extern "C" fn(*const Action, usize),
    pub on_suggest_solution: Option<extern "C" fn(*const Action, usize)>,
    pub on_progress: Option<extern "C" fn(usize)>,
    pub on_log: Option<extern "C" fn(*const u8, usize)>,
    pub log_level: LevelFilter,
    pub thread_count: u16,
    pub action_mask: u64,
    pub progress: u16,
    pub quality: u16,
    pub base_progress: u16,
    pub base_quality: u16,
    pub cp: u16,
    pub durability: u16,
    pub job_level: u8,
    pub adversarial: bool,
    pub backload_progress: bool,
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

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum LevelFilter {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LevelFilter> for log::LevelFilter {
    fn from(value: LevelFilter) -> Self {
        match value {
            LevelFilter::Off => log::LevelFilter::Off,
            LevelFilter::Error => log::LevelFilter::Error,
            LevelFilter::Warn => log::LevelFilter::Warn,
            LevelFilter::Info => log::LevelFilter::Info,
            LevelFilter::Debug => log::LevelFilter::Debug,
            LevelFilter::Trace => log::LevelFilter::Trace,
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
            backload_progress: value.backload_progress,
        };
        Self { simulator_settings }
    }
}

#[derive(Clone)]
struct CallbackLogger(Arc<Mutex<Option<CallbackLoggerImpl>>>);

struct CallbackLoggerImpl {
    on_log: extern "C" fn(*const u8, usize),
}

impl Log for CallbackLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        log::max_level() >= metadata.level()
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            if let Some(logger) = self.0.lock().unwrap().as_ref() {
                let message = format!("{} - {}", record.level(), record.args());
                let message = message.as_bytes();
                (logger.on_log)(message.as_ptr(), message.len());
            }
        }
    }

    fn flush(&self) {}
}

impl CallbackLogger {
    fn new() -> Self {
        let logger = Arc::new(Mutex::new(None));
        let logger = CallbackLogger(logger);
        log::set_max_level(log::LevelFilter::Off);
        log::set_boxed_logger(Box::new(logger.clone())).unwrap();
        logger
    }

    fn clear(&self) {
        log::set_max_level(log::LevelFilter::Off);
        self.0.lock().unwrap().take();
    }

    fn set_callback(&self, on_log: extern "C" fn(*const u8, usize), log_level: log::LevelFilter) {
        let mut logger = self.0.lock().unwrap();
        *logger = Some(CallbackLoggerImpl { on_log });
        log::set_max_level(log_level);
    }
}

static LOG: LazyLock<CallbackLogger> = LazyLock::new(|| CallbackLogger::new());

#[unsafe(no_mangle)]
pub extern "C" fn solve(args: &SolveArgs) {
    let logger = LOG.clone();

    if let Some(on_log) = args.on_log {
        let log_level = args.log_level.into();
        if log_level != log::LevelFilter::Off {
            logger.set_callback(on_log, log_level);
        }
    }

    rayon::ThreadPoolBuilder::new()
        .num_threads(args.thread_count.into())
        .build()
        .unwrap()
        .install(|| {
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

            let mut solver =
                MacroSolver::new(settings, solution_callback, progress_callback, flag.clone());
            let actions = solver.solve().unwrap_or_default();
            (args.on_finish)(actions.as_ptr() as *const Action, actions.len());
        });

    logger.clear();
}
