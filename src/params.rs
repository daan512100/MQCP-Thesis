// src/params.rs
//! src/params.rs
//!
//! Bundelt alle afstembare parameters voor de TSQC-oplosser.

use pyo3::prelude::*;

/// Alle afstembare besturingselementen voor TSQC en de MCTS-LNS uitbreiding.
#[pyclass]
#[derive(Clone, Debug)]
pub struct Params {
    #[pyo3(get, set)]
    pub gamma_target: f64,
    #[pyo3(get, set)]
    pub stagnation_iter: usize,
    #[pyo3(get, set)]
    pub max_iter: usize,
    #[pyo3(get, set)]
    pub tenure_u: usize,
    #[pyo3(get, set)]
    pub tenure_v: usize,
    #[pyo3(get, set)]
    pub use_mcts: bool,
    #[pyo3(get, set)]
    pub mcts_budget: usize,
    #[pyo3(get, set)]
    pub mcts_exploration_const: f64,
    #[pyo3(get, set)]
    pub mcts_max_depth: usize,
    #[pyo3(get, set)]
    pub lns_repair_depth: usize,
    // NIEUW VELD: Maximale looptijd in seconden voor een enkele run. 0.0 betekent geen limiet.
    #[pyo3(get, set)]
    pub max_time_seconds: f64,
}

#[pymethods]
impl Params {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(
        gamma_target: f64,
        stagnation_iter: usize,
        max_iter: usize,
        tenure_u: usize,
        tenure_v: usize,
        use_mcts: bool,
        mcts_budget: usize,
        mcts_exploration_const: f64,
        mcts_max_depth: usize,
        lns_repair_depth: usize,
        // NIEUWE PARAMETER VOOR CONSTRUCTOR
        max_time_seconds: f64, 
    ) -> Self {
        Self {
            gamma_target,
            stagnation_iter,
            max_iter,
            tenure_u,
            tenure_v,
            use_mcts,
            mcts_budget,
            mcts_exploration_const,
            mcts_max_depth,
            lns_repair_depth,
            // INITIALISATIE VAN NIEUWE PARAMETER
            max_time_seconds,
        }
    }
}

// CORRECTIE: De 'impl Default' is teruggezet voor intern Rust-gebruik,
// zoals in `lib.rs`.
impl Default for Params {
    fn default() -> Self {
        Params {
            gamma_target: 0.90,
            stagnation_iter: 1_000,
            max_iter: 100_000_000,
            tenure_u: 1,
            tenure_v: 1,
            use_mcts: false,
            mcts_budget: 100,
            mcts_exploration_const: 1.414,
            mcts_max_depth: 5,
            lns_repair_depth: 10,
            // NIEUWE DEFAULT WAARDE: 0.0 (geen timeout)
            max_time_seconds: 0.0, 
        }
    }
}

impl Params {
    /// Schakelt MCTS-gestuurde LNS-diversificatie in met de opgegeven parameters.
    pub fn enable_mcts(
        &mut self,
        budget: usize,
        exploration_const: f64,
        max_depth: usize,
        repair_depth: usize,
    ) -> &mut Self {
        self.use_mcts = true;
        self.mcts_budget = budget;
        self.mcts_exploration_const = exploration_const;
        self.mcts_max_depth = max_depth;
        self.lns_repair_depth = repair_depth;
        self
    }
}