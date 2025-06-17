//! src/maxk.rs
//!
//! Implementeert de buitenste `max-k` zoekstrategie voor TSQC.
//! Deze strategie zoekt incrementeel naar de grootste `k` waarvoor een
//! `gamma`-quasi-clique gevonden kan worden.

use crate::{
    construct::greedy_random_k, params::Params, restart::solve_fixed_k, solution::Solution, Graph,
};
use rand::Rng;

/// Zoekt naar de maximale `gamma`-quasi-clique door `solve_fixed_k`
/// iteratief aan te roepen voor oplopende waarden van `k`.
pub fn solve_maxk<'g, R>(graph: &'g Graph, rng: &mut R, p: &Params) -> Solution<'g>
where
    // `Send + Sync` is nodig om de RNG thread-safe te maken voor parallelle MCTS.
    R: Rng + ?Sized + Send + Sync,
{
    // Start met een ondergrens voor k, bv. 2 of een kleine gretig gevonden oplossing.
    let mut k_lb = 2.min(graph.n());
    if k_lb == 0 {
        return Solution::new(graph);
    }

    let mut best_sol = greedy_random_k(graph, k_lb, rng);
    if best_sol.is_gamma_feasible(p.gamma_target) {
        k_lb = best_sol.size();
    } else {
        // Als de initiële oplossing niet haalbaar is, starten we met een lege oplossing
        // en beginnen we de zoektocht vanaf k_lb + 1.
        best_sol = Solution::new(graph);
    }
    
    for k in (k_lb + 1)..=graph.n() {
        let sol_k = solve_fixed_k(graph, k, rng, p);
        if sol_k.is_gamma_feasible(p.gamma_target) {
            // We hebben een haalbare oplossing gevonden voor deze k, dus dit is onze nieuwe beste.
            best_sol = sol_k;
        } else {
            // Eerste `k` die faalt na een succesvolle `k-1` geeft aan dat we
            // waarschijnlijk de maximale grootte hebben gevonden.
            break;
        }
    }

    best_sol
}