// src/maxk.rs
//! Max-k zoekstrategie voor γ-quasi-cliques met bovengrens-pruning.
//! Uitgevoerd door solve_fixed_k herhaald voor oplopende k, stopt bij
//! ofwel geen γ-feasible oplossing meer, ofwel bij de bovengrens.

use crate::{graph::Graph, restart::solve_fixed_k, solution::Solution, params::Params};
use rand::Rng;
use std::time::Instant; // NIEUWE IMPORT: Voor het bijhouden van de tijd

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
pub fn solve_maxk<'g, R>(graph: &'g Graph, rng: &mut R, p: &Params) -> (Solution<'g>, bool) // AANGEPAST retourtype
where
    R: Rng + ?Sized + Send + Sync,
{
    let n = graph.n();
    let degree_prefix = compute_degree_prefix(graph);

    // Timer initialisatie
    let start_time = Instant::now();
    let mut is_timed_out_maxk = false;

    // Geen niet-triviale cliques mogelijk
    if n < 2 {
        return (Solution::new(graph), false); // Niet timed out
    }

    // Start met k = 2 voor een minimale γ-feasible basis
    let mut best_sol = Solution::new(graph);
    
    // Timeout check voordat de eerste solve_fixed_k wordt aangeroepen
    if p.max_time_seconds > 0.0 && start_time.elapsed().as_secs_f64() >= p.max_time_seconds {
        return (best_sol, true); // Timed out
    }

    let (mut sol, timed_out_fixed_k) = solve_fixed_k(graph, 2, rng, p);
    if timed_out_fixed_k {
        is_timed_out_maxk = true;
        // Als zelfs de k=2 run timed out, kunnen we hier stoppen of doorgaan.
        // Voor nu kiezen we ervoor om door te gaan, maar markeren de totale run als timed out.
    }

    if !sol.is_gamma_feasible(p.gamma_target) {
        // Geen enkele 2-clique voldoet, dus geen oplossing
        return (best_sol, is_timed_out_maxk); // Retourneer de best_sol die nog leeg is, en de timeout status
    }
    best_sol = sol;

    // Probeer voor elke volgende k, met bovengrenscontrole
    for k in 3..=n {
        // Timeout check aan het begin van elke k-iteratie
        if p.max_time_seconds > 0.0 && start_time.elapsed().as_secs_f64() >= p.max_time_seconds {
            is_timed_out_maxk = true;
            break; // Stop als timeout bereikt is
        }

        let ub = ub_edges(&degree_prefix, k);
        let required_edges = (p.gamma_target * ((k * (k - 1)) as f64) / 2.0).ceil() as usize;
        if ub < required_edges {
            // Pruning: zelfs in het beste geval te weinig randen
            break;
        }
        let (sol_k, timed_out_current_k) = solve_fixed_k(graph, k, rng, p);
        if timed_out_current_k {
            is_timed_out_maxk = true;
        }

        if sol_k.is_gamma_feasible(p.gamma_target) {
            // Belangrijk: Voor max-k is het primaire doel de grootte,
            // met dichtheid als tie-breaker.
            if sol_k.size() > best_sol.size()
                || (sol_k.size() == best_sol.size() && sol_k.density() > best_sol.density())
            {
                best_sol = sol_k;
            }
        } else {
            // Geen γ-quasi-clique van deze grootte → stoppen met zoeken naar grotere k
            break;
        }
    }

    (best_sol, is_timed_out_maxk) // Retourneer de beste oplossing en de timeout status
}