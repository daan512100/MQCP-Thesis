//! src/lib.rs
//!
//! Hoofdbestand van de Rust-bibliotheek. Definieert de publieke modules
//! en de PyO3-bindings om de Rust-functionaliteit toegankelijk te maken
//! vanuit Python.

// Publieke modules voor gebruik binnen de Rust-crate
pub mod construct;
pub mod diversify;
pub mod graph;
pub mod lns;
pub mod maxk;
pub mod mcts;
pub mod neighbour;
pub mod params;
pub mod restart;
pub mod solution;
pub mod tabu;

// Her-exporteer de belangrijkste types voor Rust-gebruikers
pub use graph::Graph;
pub use params::Params;
pub use solution::Solution;

use pyo3::prelude::*;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::fs::File;
use std::io::BufReader;

/// Python-binding voor de fixed-k oplosser.
#[pyfunction]
#[pyo3(signature = (
    instance_path, k, gamma, seed, runs=1,
    use_mcts=false, mcts_budget=100, mcts_uct=1.414, mcts_depth=5, lns_repair=10
))]
fn solve_k_py(
    instance_path: String,
    k: usize,
    gamma: f64,
    seed: u64,
    runs: usize,
    use_mcts: bool,
    mcts_budget: usize,
    mcts_uct: f64,
    mcts_depth: usize,
    lns_repair: usize,
) -> PyResult<(usize, usize, f64)> {
    let file = File::open(&instance_path)
       .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    let graph = Graph::parse_dimacs(BufReader::new(file))
       .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    let mut p = Params::default();
    p.gamma_target = gamma;
    if use_mcts {
        p.enable_mcts(mcts_budget, mcts_uct, mcts_depth, lns_repair);
    }

    let mut best_sol_overall = Solution::new(&graph);

    for i in 0..runs {
        let mut rng = StdRng::seed_from_u64(seed + i as u64);
        let sol = restart::solve_fixed_k(&graph, k, &mut rng, &p);
        if sol.density() > best_sol_overall.density() {
            best_sol_overall = sol;
        }
    }

    Ok((
        best_sol_overall.size(),
        best_sol_overall.edges(),
        best_sol_overall.density(),
    ))
}

/// Python-binding voor de max-k oplosser.
#[pyfunction]
#[pyo3(signature = (
    instance_path, gamma, seed, runs=1,
    use_mcts=false, mcts_budget=100, mcts_uct=1.414, mcts_depth=5, lns_repair=10
))]
fn solve_max_py(
    instance_path: String,
    gamma: f64,
    seed: u64,
    runs: usize,
    use_mcts: bool,
    mcts_budget: usize,
    mcts_uct: f64,
    mcts_depth: usize,
    lns_repair: usize,
) -> PyResult<(usize, usize, f64)> {
    let file = File::open(&instance_path)
       .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    let graph = Graph::parse_dimacs(BufReader::new(file))
       .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    let mut p = Params::default();
    p.gamma_target = gamma;
    if use_mcts {
        p.enable_mcts(mcts_budget, mcts_uct, mcts_depth, lns_repair);
    }
    
    let mut best_sol_overall = Solution::new(&graph);
    for i in 0..runs {
        let mut rng = StdRng::seed_from_u64(seed + i as u64);
        let sol = maxk::solve_maxk(&graph, &mut rng, &p);
        // Voor max-k is het primaire doel de grootte, met dichtheid als tie-breaker.
        if sol.size() > best_sol_overall.size()|| (sol.size() == best_sol_overall.size() && sol.density() > best_sol_overall.density())
        {
            best_sol_overall = sol;
        }
    }

    Ok((
        best_sol_overall.size(),
        best_sol_overall.edges(),
        best_sol_overall.density(),
    ))
}

/// Helperfunctie om een DIMACS-bestand te parsen en (n, m) terug te geven.
#[pyfunction]
fn parse_dimacs_py(instance_path: String) -> PyResult<(usize, usize)> {
    let file = File::open(&instance_path)
       .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    let graph = Graph::parse_dimacs(BufReader::new(file))
       .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    Ok((graph.n(), graph.m()))
}


/// Definieert de Python-module `_native`.
#[pymodule]
fn _native(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(solve_k_py, m)?)?;
    m.add_function(wrap_pyfunction!(solve_max_py, m)?)?;
    m.add_function(wrap_pyfunction!(parse_dimacs_py, m)?)?;
    Ok(())
}