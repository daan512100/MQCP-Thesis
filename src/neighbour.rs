// Bestand: src/neighbour.rs
//!
//! Implementeert de intensificatiestap (één-swap lokale zoektocht) voor TSQC,
//! met Δ_uv-correctie (d(v) − d(u) − e_uv) en long-term frequentiegeheugen (Secties 3.4.1 & 3.5).

use crate::{
    graph::Graph,
    params::Params,
    solution::Solution,
    tabu::DualTabu,
    freq::{remove_counted, add_counted},
};
use rand::Rng;
use bitvec::slice::BitSlice;

/// Handmatige intersectie-telling voor verbindingen tussen v en S.
/// Dit omzeilt de E0369-compilerfout.
fn count_intersecting_ones(a: &BitSlice, b: &BitSlice) -> usize {
    a.iter().by_vals()
        .zip(b.iter().by_vals())
        .filter(|&(x, y)| x && y)
        .count()
}

/// Probeert één intensificatie-swap (u ∈ A, v ∈ B) uit te voeren.
/// Returns `true` als er geswapped is.
pub fn improve_once<'g, R>(
    sol: &mut Solution<'g>,
    tabu: &mut DualTabu,
    best_global_rho: f64,
    freq: &mut Vec<usize>,
    p: &Params,
    rng: &mut R,
) -> bool
where
    R: Rng + ?Sized,
{
    let graph = sol.graph();
    let k = sol.size();
    if k == 0 || k == graph.n() {
        tabu.step();
        tabu.update_tenures(sol.size(), sol.edges(), p.gamma_target, rng);
        return false;
    }

    // Bereken MinInS en MaxOutS over niet-taboe knopen
    let (min_in, max_out) = calculate_critical_degrees(sol, tabu);
    if min_in == usize::MAX || max_out == usize::MIN {
        tabu.step();
        tabu.update_tenures(sol.size(), sol.edges(), p.gamma_target, rng);
        return false;
    }

    // Bouw kritieke sets A en B
    let (set_a, set_b) = build_critical_sets(sol, tabu, min_in, max_out);

    let current_edges = sol.edges();
    let sol_bitset = sol.bitset();

    let mut best_allowed: Option<(isize, usize, usize)> = None;
    let mut best_aspire: Option<(isize, usize, usize)> = None;

    for &u in &set_a {
        let loss = count_intersecting_ones(graph.neigh_row(u), sol_bitset);
        for &v in &set_b {
            let gain = count_intersecting_ones(graph.neigh_row(v), sol_bitset);
            // Δ_uv = d(v) − d(u) − e_uv
            let e_uv = if graph.neigh_row(u)[v] { 1 } else { 0 };
            let delta = gain as isize - loss as isize - e_uv as isize;

            let is_tabu = tabu.is_tabu_u(u) || tabu.is_tabu_v(v);
            if !is_tabu {
                if best_allowed.is_none() || delta > best_allowed.unwrap().0 {
                    best_allowed = Some((delta, u, v));
                }
            } else {
                // Aspiratie: accepteer taboe-move als dichtheid verbetert
                let new_edges = (current_edges as isize + delta) as usize;
                let new_rho = if k < 2 {
                    0.0
                } else {
                    2.0 * (new_edges as f64) / ((k * (k - 1)) as f64)
                };
                if new_rho > best_global_rho {
                    if best_aspire.is_none() || delta > best_aspire.unwrap().0 {
                        best_aspire = Some((delta, u, v));
                    }
                }
            }
        }
    }

    let mut did_swap = false;
    if let Some((_, u, v)) = best_allowed.or(best_aspire) {
        // Gebruik long-term freq-helpers voor remove/add mét reset
        remove_counted(sol, u, freq);
        add_counted(sol, v, freq);
        tabu.forbid_u(u);
        tabu.forbid_v(v);
        did_swap = true;
    }

    tabu.step();
    tabu.update_tenures(sol.size(), sol.edges(), p.gamma_target, rng);
    did_swap
}

/// Berekent MinInS en MaxOutS voor niet-taboe knopen (Sectie 3.4.1).
fn calculate_critical_degrees(sol: &Solution, tabu: &DualTabu) -> (usize, usize) {
    let graph = sol.graph();
    let sol_bitset = sol.bitset();
    let min_in = sol_bitset.iter_ones()
        .filter(|&u| !tabu.is_tabu_u(u))
        .map(|u| count_intersecting_ones(graph.neigh_row(u), sol_bitset))
        .min().unwrap_or(usize::MAX);
    let max_out = (0..graph.n())
        .filter(|&v| !sol_bitset[v] && !tabu.is_tabu_v(v))
        .map(|v| count_intersecting_ones(graph.neigh_row(v), sol_bitset))
        .max().unwrap_or(usize::MIN);
    (min_in, max_out)
}

/// Bouwt kritieke sets A (min_in) en B (max_out) (Sectie 3.4.1).
fn build_critical_sets(
    sol: &Solution,
    tabu: &DualTabu,
    min_in: usize,
    max_out: usize,
) -> (Vec<usize>, Vec<usize>) {
    let graph = sol.graph();
    let sol_bitset = sol.bitset();
    let set_a: Vec<usize> = sol_bitset.iter_ones()
        .filter(|&u| !tabu.is_tabu_u(u)
            && count_intersecting_ones(graph.neigh_row(u), sol_bitset) == min_in)
        .collect();
    let set_b: Vec<usize> = (0..graph.n())
        .filter(|&v| !sol_bitset[v]
            && !tabu.is_tabu_v(v)
            && count_intersecting_ones(graph.neigh_row(v), sol_bitset) == max_out)
        .collect();
    (set_a, set_b)
}
