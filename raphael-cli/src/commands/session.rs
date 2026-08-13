use std::{
    collections::HashMap,
    io::{self, BufRead, Write},
};

use clap::Args;
use raphael_sim::{Action, Condition, Effects, SimulationState};
use raphael_solver::{AtomicFlag, MacroSolver, SolverSettings};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Args, Debug)]
pub struct SessionArgs {}

#[derive(Debug, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
enum Request {
    Create {
        #[serde(default)]
        request_id: Option<Value>,
        session_id: String,
        settings: SolverSettings,
    },
    Solve {
        #[serde(default)]
        request_id: Option<Value>,
        session_id: String,
        state: WireState,
        condition: Condition,
    },
    Close {
        #[serde(default)]
        request_id: Option<Value>,
        session_id: String,
    },
    Ping {
        #[serde(default)]
        request_id: Option<Value>,
    },
}

#[derive(Serialize)]
struct Response<T: Serialize> {
    request_id: Option<Value>,
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WireState {
    cp: u16,
    durability: u16,
    progress: u16,
    quality: u16,
    #[serde(default)]
    unreliable_quality: u16,
    effects: u64,
}

impl From<WireState> for SimulationState {
    fn from(state: WireState) -> Self {
        Self {
            cp: state.cp,
            durability: state.durability,
            progress: state.progress,
            quality: state.quality,
            unreliable_quality: state.unreliable_quality,
            effects: Effects::from_bits(state.effects),
        }
    }
}

#[derive(Serialize)]
struct SessionResult {
    session_id: String,
}

#[derive(Serialize)]
struct SolveResult {
    actions: Vec<Action>,
}

#[derive(Serialize)]
struct Pong {
    pong: bool,
}

fn solver(settings: SolverSettings) -> MacroSolver<'static> {
    MacroSolver::new(
        settings,
        Box::new(|_| {}),
        Box::new(|_| {}),
        AtomicFlag::new(),
    )
}

fn write_response<T: Serialize>(stdout: &mut impl Write, response: &Response<T>) -> io::Result<()> {
    serde_json::to_writer(&mut *stdout, response)?;
    stdout.write_all(b"\n")?;
    stdout.flush()
}

fn error_response(request_id: Option<Value>, error: impl ToString) -> Response<Value> {
    Response {
        request_id,
        ok: false,
        result: None,
        error: Some(error.to_string()),
    }
}

pub fn execute(_args: &SessionArgs) {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    let mut sessions: HashMap<String, MacroSolver<'static>> = HashMap::new();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                let _ = write_response(&mut stdout, &error_response(None, error));
                break;
            }
        };
        if line.trim().is_empty() {
            continue;
        }

        let request = match serde_json::from_str::<Request>(&line) {
            Ok(request) => request,
            Err(error) => {
                let _ = write_response(&mut stdout, &error_response(None, error));
                continue;
            }
        };

        match request {
            Request::Create {
                request_id,
                session_id,
                settings,
            } => {
                if sessions.contains_key(&session_id) {
                    let _ = write_response(
                        &mut stdout,
                        &error_response(request_id, "session already exists"),
                    );
                    continue;
                }
                sessions.insert(session_id.clone(), solver(settings));
                let _ = write_response(
                    &mut stdout,
                    &Response {
                        request_id,
                        ok: true,
                        result: Some(SessionResult { session_id }),
                        error: None,
                    },
                );
            }
            Request::Solve {
                request_id,
                session_id,
                state,
                condition,
            } => {
                let Some(session) = sessions.get_mut(&session_id) else {
                    let _ =
                        write_response(&mut stdout, &error_response(request_id, "unknown session"));
                    continue;
                };
                match session.solve_from_state_with_condition(state.into(), condition) {
                    Ok(actions) => {
                        let _ = write_response(
                            &mut stdout,
                            &Response {
                                request_id,
                                ok: true,
                                result: Some(SolveResult { actions }),
                                error: None,
                            },
                        );
                    }
                    Err(error) => {
                        let _ = write_response(
                            &mut stdout,
                            &error_response(request_id, format!("{error:?}")),
                        );
                    }
                }
            }
            Request::Close {
                request_id,
                session_id,
            } => {
                if sessions.remove(&session_id).is_none() {
                    let _ =
                        write_response(&mut stdout, &error_response(request_id, "unknown session"));
                    continue;
                }
                let _ = write_response(
                    &mut stdout,
                    &Response {
                        request_id,
                        ok: true,
                        result: Some(SessionResult { session_id }),
                        error: None,
                    },
                );
            }
            Request::Ping { request_id } => {
                let _ = write_response(
                    &mut stdout,
                    &Response {
                        request_id,
                        ok: true,
                        result: Some(Pong { pong: true }),
                        error: None,
                    },
                );
            }
        }
    }
}
