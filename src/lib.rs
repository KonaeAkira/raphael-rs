pub mod config;

pub mod game {
    pub mod actions;
    pub mod conditions;
    pub mod effects;
    pub mod state;
    pub mod units {
        pub mod progress;
        pub mod quality;
    }
}

pub mod solvers {
    pub mod util {
        pub mod action_sequence;
        pub mod pareto_front;
    }
    pub mod finish_solver;
    pub mod macro_solver;
}
