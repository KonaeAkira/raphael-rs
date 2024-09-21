#[macro_use]
extern crate rust_i18n;

i18n!("assets/locales", fallback = "en");

mod app;
pub use app::MacroSolverApp;
pub use worker::Worker;

mod config;
mod widgets;
mod worker;
