// src/lib.rs

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
pub mod freq;

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
#[pyo3(signature = (instance_path, py_params))]
fn solve_k_py(
    instance_path: String,
    py_params: Py<Params>,
) -> PyResult<(usize, usize, f64, bool)> {
    let file = File::open(&instance_path)
       .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    let graph = Graph::parse_dimacs(BufReader::new(file))
       .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    
    let p = Python::with_gil(|py| {
        let p_ref = py_params.borrow(py);
        Params::new(
            p_ref.gamma_target,
            p_ref.stagnation_iter,
            p_ref.max_iter,
            p_ref.tenure_u,
            p_ref.tenure_v,
            p_ref.use_mcts,
            p_ref.mcts_budget,
            p_ref.mcts_exploration_const,
            p_ref.mcts_max_depth,
            p_ref.lns_repair_depth,
            // --- NIEUW ---
            p_ref.lns_rcl_alpha,
            // --- EINDE NIEUW ---
            p_ref.max_time_seconds,
            p_ref.k,
            p_ref.runs,
            p_ref.seed,
        )
    });
    
    let k_val = p.k.expect("Fixed-k mode requires a 'k' value in Params.");

    let mut best_sol_overall = Solution::new(&graph);
    let mut is_timed_out_overall = false;

    for i in 0..p.runs {
        let mut rng = StdRng::seed_from_u64(p.seed + i as u64);
        let (sol, timed_out_run) = restart::solve_fixed_k(&graph, k_val, &mut rng, &p);
        if sol.density() > best_sol_overall.density() {
            best_sol_overall = sol;
        }
        if timed_out_run {
            is_timed_out_overall = true;
        }
    }

    Ok((
        best_sol_overall.size(),
        best_sol_overall.edges(),
        best_sol_overall.density(),
        is_timed_out_overall,
    ))
}

/// Python-binding voor de max-k oplosser.
#[pyfunction]
#[pyo3(signature = (instance_path, py_params))]
fn solve_max_py(
    instance_path: String,
    py_params: Py<Params>,
) -> PyResult<(usize, usize, f64, bool)> {
    let file = File::open(&instance_path)
       .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    let graph = Graph::parse_dimacs(BufReader::new(file))
       .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    let p = Python::with_gil(|py| {
        let p_ref = py_params.borrow(py);
        Params::new(
            p_ref.gamma_target,
            p_ref.stagnation_iter,
            p_ref.max_iter,
            p_ref.tenure_u,
            p_ref.tenure_v,
            p_ref.use_mcts,
            p_ref.mcts_budget,
            p_ref.mcts_exploration_const,
            p_ref.mcts_max_depth,
            p_ref.lns_repair_depth,
            // --- NIEUW ---
            p_ref.lns_rcl_alpha,
            // --- EINDE NIEUW ---
            p_ref.max_time_seconds,
            p_ref.k,
            p_ref.runs,
            p_ref.seed,
        )
    });

    let mut best_sol_overall = Solution::new(&graph);
    let mut is_timed_out_overall = false;

    for i in 0..p.runs {
        let mut rng = StdRng::seed_from_u64(p.seed + i as u64);
        let (sol, timed_out_run) = maxk::solve_maxk(&graph, &mut rng, &p);
        if sol.size() > best_sol_overall.size()
            || (sol.size() == best_sol_overall.size() && sol.density() > best_sol_overall.density())
        {
            best_sol_overall = sol;
        }
        if timed_out_run {
            is_timed_out_overall = true;
        }
    }

    Ok((
        best_sol_overall.size(),
        best_sol_overall.edges(),
        best_sol_overall.density(),
        is_timed_out_overall,
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
    m.add_class::<Params>()?;
    m.add_function(wrap_pyfunction!(solve_k_py, m)?)?;
    m.add_function(wrap_pyfunction!(solve_max_py, m)?)?;
    m.add_function(wrap_pyfunction!(parse_dimacs_py, m)?)?;
    Ok(())
}