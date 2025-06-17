// Bestand: src/maxk.rs
//! Max-k zoekstrategie voor γ-quasi-cliques met bovengrens-pruning.
//! Uitgevoerd door solve_fixed_k herhaald voor oplopende k, stopt bij
//! ofwel geen γ-feasible oplossing meer, ofwel bij de bovengrens.

use crate::{graph::Graph, restart::solve_fixed_k, solution::Solution, params::Params};
use rand::Rng;

/// Bereken de prefix-som van de knoopgraden, gesorteerd in aflopende volgorde.
/// prefix[i] = som van de i hoogste graden.
fn compute_degree_prefix(graph: &Graph) -> Vec<usize> {
    let mut degrees: Vec<usize> = (0..graph.n()).map(|u| graph.degree(u)).collect();
    degrees.sort_unstable_by(|a, b| b.cmp(a));
    let mut prefix = Vec::with_capacity(degrees.len() + 1);
    prefix.push(0);
    for d in degrees {
        prefix.push(prefix.last().unwrap() + d);
    }
    prefix
}

/// Bovengrens op het aantal randen in een subgraaf van grootte `k`,
/// gebaseerd op de prefix-som. (Elke rand telt dubbel mee in de graden.)
fn ub_edges(prefix: &[usize], k: usize) -> usize {
    prefix[k] / 2
}

/// Zoekt naar de maximale γ-quasi-clique door solve_fixed_k
/// herhaald voor k = 2..n en stopt bij eerste mislukking of bij pruning.
pub fn solve_maxk<'g, R>(graph: &'g Graph, rng: &mut R, p: &Params) -> Solution<'g>
where
    R: Rng + ?Sized + Send + Sync,
{
    let n = graph.n();
    let degree_prefix = compute_degree_prefix(graph);

    // Geen niet-triviale cliques mogelijk
    if n < 2 {
        return Solution::new(graph);
    }

    // Start met k = 2 voor een minimale γ-feasible basis
    let mut best_sol = Solution::new(graph);
    let mut sol = solve_fixed_k(graph, 2, rng, p);
    if !sol.is_gamma_feasible(p.gamma_target) {
        // Geen enkele 2-clique voldoet, dus geen oplossing
        return best_sol;
    }
    best_sol = sol;

    // Probeer voor elke volgende k, met bovengrenscontrole
    for k in 3..=n {
        let ub = ub_edges(&degree_prefix, k);
        let required_edges = (p.gamma_target * ((k * (k - 1)) as f64) / 2.0).ceil() as usize;
        if ub < required_edges {
            // Pruning: zelfs in het beste geval te weinig randen
            break;
        }
        let sol_k = solve_fixed_k(graph, k, rng, p);
        if sol_k.is_gamma_feasible(p.gamma_target) {
            best_sol = sol_k;
        } else {
            // Geen γ-quasi-clique van deze grootte → stoppen
            break;
        }
    }

    best_sol
}
