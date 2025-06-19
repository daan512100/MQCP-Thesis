// src/neighbour.rs
//!
//! Implementeert de intensificatiestap (één-swap lokale zoektocht) voor TSQC.

use crate::{
    freq::{add_counted, remove_counted},
    graph::Graph,
    params::Params,
    solution::Solution,
    tabu::DualTabu,
};
use bitvec::slice::BitSlice;
use rand::seq::SliceRandom;
use rand::Rng;

/// Handmatige intersectie-telling voor verbindingen tussen v en S.
fn count_intersecting_ones(a: &BitSlice, b: &BitSlice) -> usize {
    a.iter().by_vals().zip(b.iter().by_vals()).filter(|&(x, y)| x && y).count()
}

/// Probeert één intensificatie-swap uit te voeren. Returns `true` als er geswapped is.
///
/// GEOPTIMALISEERDE IMPLEMENTATIE: Deze versie vermijdt het aanmaken van meerdere
/// vectoren met kandidaten binnen de hot loop. In plaats daarvan houdt het de
/// "beste tot nu toe" swap bij, wat heap-allocaties minimaliseert en de prestaties
/// aanzienlijk verbetert.
pub fn improve_once<'g, R>(
    sol: &mut Solution<'g>,
    tabu: &mut DualTabu,
    best_global_rho: f64,
    freq: &mut Vec<usize>,
    p: &Params,
    rng: &mut R,
) -> bool
where
    R: Rng +?Sized,
{
    let graph = sol.graph();
    let k = sol.size();
    if k == 0 || k == graph.n() {
        tabu.step();
        tabu.update_tenures(sol.size(), sol.edges(), p.gamma_target, rng);
        return false;
    }

    let (min_in, max_out) = calculate_critical_degrees(sol, tabu);
    if min_in == usize::MAX || max_out == usize::MIN {
        tabu.step();
        tabu.update_tenures(sol.size(), sol.edges(), p.gamma_target, rng);
        return false;
    }

    let (set_a, set_b) = build_critical_sets(sol, tabu, min_in, max_out);
    if set_a.is_empty() || set_b.is_empty() {
        tabu.step();
        tabu.update_tenures(sol.size(), sol.edges(), p.gamma_target, rng);
        return false;
    }

    let current_edges = sol.edges();
    let sol_bitset = sol.bitset();

    // --- GEOPTIMALISEERDE SWAP SELECTIE ---
    let mut best_aspirating_swap: Option<(isize, usize, usize)> = None;
    let mut best_non_tabu_delta: isize = -1;
    let mut best_non_tabu_candidates: Vec<(usize, usize)> = Vec::new();

    for &u in &set_a {
        let loss = count_intersecting_ones(graph.neigh_row(u), sol_bitset);
        for &v in &set_b {
            let gain = count_intersecting_ones(graph.neigh_row(v), sol_bitset);
            let e_uv = if graph.neigh_row(u)[v] { 1 } else { 0 };
            let delta = gain as isize - loss as isize - e_uv as isize;

            let is_tabu_move = tabu.is_tabu_u(u) || tabu.is_tabu_v(v);

            if is_tabu_move {
                // Aspiratiecriterium check
                let new_edges = (current_edges as isize + delta) as usize;
                let new_rho = if k < 2 { 0.0 } else { 2.0 * new_edges as f64 / (k * (k - 1)) as f64 };
                if new_rho > best_global_rho {
                    // Vergelijk met de huidige beste aspirerende swap
                    if best_aspirating_swap.is_none() || delta > best_aspirating_swap.unwrap().0 {
                        best_aspirating_swap = Some((delta, u, v));
                    }
                }
            } else {
                // Niet-taboe, niet-verslechterende moves
                if delta >= 0 {
                    if delta > best_non_tabu_delta {
                        // Dit is een strikt betere delta dan we eerder zagen.
                        best_non_tabu_delta = delta;
                        best_non_tabu_candidates.clear();
                        best_non_tabu_candidates.push((u, v));
                    } else if delta == best_non_tabu_delta {
                        // Deze delta is even goed als de beste, voeg toe voor tie-breaking.
                        best_non_tabu_candidates.push((u, v));
                    }
                }
            }
        }
    }

    // --- DEFINITIEVE KEUZE VAN DE SWAP ---
    let chosen_swap: Option<(usize, usize)> = if let Some(aspirating) = best_aspirating_swap {
        // Prioriteit 1: De beste aspirerende swap.
        Some((aspirating.1, aspirating.2))
    } else if!best_non_tabu_candidates.is_empty() {
        // Prioriteit 2: Een willekeurige van de beste niet-taboe, niet-verslechterende swaps.
        best_non_tabu_candidates.choose(rng).cloned()
    } else {
        // Geen geschikte swap gevonden.
        None
    };

    let mut did_swap = false;
    if let Some((u, v)) = chosen_swap {
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


/// Berekent MinInS en MaxOutS voor niet-taboe knopen.
fn calculate_critical_degrees(sol: &Solution, tabu: &DualTabu) -> (usize, usize) {
    let graph = sol.graph();
    let sol_bitset = sol.bitset();

    let min_in = sol_bitset.iter_ones()
       .filter(|&u|!tabu.is_tabu_u(u))
       .map(|u| count_intersecting_ones(graph.neigh_row(u), sol_bitset))
       .min().unwrap_or(usize::MAX);

    let max_out = (0..graph.n())
       .filter(|&v|!sol_bitset[v] &&!tabu.is_tabu_v(v))
       .map(|v| count_intersecting_ones(graph.neigh_row(v), sol_bitset))
       .max().unwrap_or(usize::MIN);

    (min_in, max_out)
}

/// Bouwt kritieke sets A (min_in) en B (max_out).
fn build_critical_sets(
    sol: &Solution,
    tabu: &DualTabu,
    min_in: usize,
    max_out: usize,
) -> (Vec<usize>, Vec<usize>) {
    let graph = sol.graph();
    let sol_bitset = sol.bitset();

    let set_a: Vec<usize> = sol_bitset.iter_ones()
       .filter(|&u|!tabu.is_tabu_u(u) && count_intersecting_ones(graph.neigh_row(u), sol_bitset) == min_in)
       .collect();

    let set_b: Vec<usize> = (0..graph.n())
       .filter(|&v|!sol_bitset[v] &&!tabu.is_tabu_v(v) && count_intersecting_ones(graph.neigh_row(v), sol_bitset) == max_out)
       .collect();

    (set_a, set_b)
}