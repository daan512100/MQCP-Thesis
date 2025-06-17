//! src/lns.rs
//!
//! Implementeert de Large Neighborhood Search (LNS) herstelheuristiek.
//! Dit volgt de twee-fasen aanpak uit de thesis-proposal:
//! 1. Greedy Completion: herstel de oplossingsgrootte.
//! 2. Mini-TSQC Refinement: verfijn de oplossing met lokale zoekstappen.

use crate::{
    neighbour::improve_once, params::Params, solution::Solution, tabu::DualTabu,
};
use rand::Rng;

/// Past LNS-herstel toe op een deels vernietigde oplossing.
///
/// - `initial_sol`: De oplossing vóór de vernietiging.
/// - `removals`: De sequentie van verwijderde knopen.
///
/// Geeft een herstelde oplossing terug.
pub fn apply_lns<'g, R>(
    initial_sol: &Solution<'g>,
    removals: &[usize],
    p: &Params,
    rng: &mut R,
) -> Solution<'g>
where
    R: Rng + ?Sized,
{
    let mut sol = initial_sol.clone();
    for &v in removals {
        sol.remove(v);
    }

    let target_k = initial_sol.size();
    let graph = initial_sol.graph();

    // --- Fase 1: Greedy Completion ---
    // Herstel de grootte van de oplossing naar `target_k`.
    while sol.size() < target_k {
        let mut best_gain = isize::MIN;
        let mut best_v = None;
        let sol_bitset = sol.bitset();

        for v in 0..graph.n() {
            if !sol_bitset[v] {
                // CORRECTIE (E0369): De `&`-operator wordt vervangen door een handmatige
                // en performante intersectie-telling via iterators.
                let gain = graph
                    .neigh_row(v)
                    .iter()
                    .by_vals()
                    .zip(sol_bitset.iter().by_vals())
                    .filter(|&(a, b)| a && b)
                    .count() as isize;

                if gain > best_gain {
                    best_gain = gain;
                    best_v = Some(v);
                }
            }
        }

        if let Some(v_to_add) = best_v {
            sol.add(v_to_add);
        } else {
            // Geen knopen meer om toe te voegen.
            break;
        }
    }

    // --- Fase 2: Mini-TSQC Refinement ---
    // Verfijn de oplossing met een korte lokale zoektocht.
    if p.lns_repair_depth > 0 {
        let mut tabu = DualTabu::new(graph.n(), p.tenure_u, p.tenure_v);
        let mut freq = vec![0; graph.n()]; // Tijdelijk frequentiegeheugen
        let best_rho = 0.0; // Aspiratie is niet relevant in deze korte verfijning.

        for _ in 0..p.lns_repair_depth {
            if !improve_once(&mut sol, &mut tabu, best_rho, &mut freq, p, rng) {
                // Geen verbetering meer mogelijk.
                break;
            }
        }
    }

    sol
}