// src/params.rs

use pyo3::prelude::*;

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
    // --- NIEUW ---
    // De alpha-parameter voor de Restricted Candidate List in de LNS-herstelfase.
    // 1.0 = puur hebzuchtig (greedy), < 1.0 introduceert willekeur.
    #[pyo3(get, set)]
    pub lns_rcl_alpha: f64,
    // --- EINDE NIEUW ---
    #[pyo3(get, set)]
    pub max_time_seconds: f64,
    #[pyo3(get, set)]
    pub k: Option<usize>,
    #[pyo3(get, set)]
    pub runs: usize,
    #[pyo3(get, set)]
    pub seed: u64,
}

#[pymethods]
impl Params {
    #[new]
    #[pyo3(signature = (
        gamma_target = 0.90,
        stagnation_iter = 1_000,
        max_iter = 100_000_000,
        tenure_u = 1,
        tenure_v = 1,
        use_mcts = false,
        mcts_budget = 100,
        mcts_exploration_const = 1.414,
        mcts_max_depth = 5,
        lns_repair_depth = 10,
        // --- NIEUW ---
        // Standaardwaarde 1.0 betekent puur hebzuchtig (het oude gedrag)
        lns_rcl_alpha = 1.0,
        // --- EINDE NIEUW ---
        max_time_seconds = 0.0,
        k = None,
        runs = 1,
        seed = 42,
    ))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
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
        // --- NIEUW ---
        lns_rcl_alpha: f64,
        // --- EINDE NIEUW ---
        max_time_seconds: f64,
        k: Option<usize>,
        runs: usize,
        seed: u64,
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
            // --- NIEUW ---
            lns_rcl_alpha,
            // --- EINDE NIEUW ---
            max_time_seconds,
            k,
            runs,
            seed,
        }
    }

    pub fn copy(&self) -> Self {
        self.clone()
    }
}

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
            // --- NIEUW ---
            lns_rcl_alpha: 1.0,
            // --- EINDE NIEUW ---
            max_time_seconds: 0.0,
            k: None,
            runs: 1,
            seed: 42,
        }
    }
}

// De enable_mcts functie hoeft niet aangepast te worden.
impl Params {
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