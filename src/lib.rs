#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::MacroSolverApp;
pub use worker::Worker;

mod config;
mod utils;
mod widgets;
mod worker;
