// src/restart.rs
//!
//! Implementeert de multi-start Tabu Search voor een vaste `k`.

use crate::{
    construct::greedy_random_k,
    diversify::{heavy_perturbation, mild_perturbation},
    freq::add_counted,
    graph::Graph,
    lns::apply_lns,
    mcts::MctsTree,
    neighbour::improve_once,
    params::Params,
    solution::Solution,
    tabu::DualTabu,
};
use rand::seq::SliceRandom;
use rand::Rng;
use std::time::Instant;

/// Helper-functie die in deze module wordt gebruikt.
fn count_intersecting_ones(a: &bitvec::slice::BitSlice, b: &bitvec::slice::BitSlice) -> usize {
    a.iter().by_vals().zip(b.iter().by_vals()).filter(|&(x, y)| x && y).count()
}

/// Zoekt naar een `γ`-quasi-clique van vaste grootte `k` en stopt zodra een haalbare oplossing is gevonden.
pub fn solve_fixed_k<'g, R>(
    graph: &'g Graph,
    k: usize,
    rng: &mut R,
    p: &Params,
) -> (Solution<'g>, bool) // Returns (gevonden_oplossing, is_timed_out)
where
    R: Rng + ?Sized + Send + Sync,
{
    // --- 0. Initialisatie ---
    let max_possible_edges = if k > 1 { k * (k - 1) / 2 } else { 0 };
    let needed_edges = (p.gamma_target * max_possible_edges as f64).ceil() as usize;

    if max_possible_edges < needed_edges {
        return (Solution::new(graph), false);
    }

    let start_time = Instant::now();
    let mut is_timed_out = false;

    let mut freq_mem = vec![0usize; graph.n()];
    let mut best_global = Solution::new(graph);
    let mut total_moves = 0usize;

    // --- 1. Hoofdlus met Restarts ---
    'restart_loop: while total_moves < p.max_iter {
        if p.max_time_seconds > 0.0 && start_time.elapsed().as_secs_f64() >= p.max_time_seconds {
            is_timed_out = true;
            break;
        }

        let mut cur = initialize_solution(graph, k, &mut freq_mem, &best_global, rng);
        let mut tabu = DualTabu::new(graph.n(), p.tenure_u, p.tenure_v);
        tabu.update_tenures(cur.size(), cur.edges(), p.gamma_target, rng);

        let mut best_run = cur.clone();
        
        // --- Controleer direct na initialisatie ---
        // Als de initiële oplossing al haalbaar is, zijn we direct klaar.
        if best_run.is_gamma_feasible(p.gamma_target) {
            return (best_run, false); // Gevonden, niet timed out
        }

        let mut stagnation = 0usize;

        // --- 2. Lokale Zoektocht ---
        while stagnation < p.stagnation_iter && total_moves < p.max_iter {
            if p.max_time_seconds > 0.0 && start_time.elapsed().as_secs_f64() >= p.max_time_seconds {
                is_timed_out = true;
                // Update de globale beste VOORDAT we stoppen
                if best_run.density() > best_global.density() {
                    best_global = best_run;
                }
                break 'restart_loop;
            }

            let moved = improve_once(&mut cur, &mut tabu, best_global.density(), &mut freq_mem, p, rng);
            total_moves += 1;
            
            if cur.density() > best_run.density() {
                best_run = cur.clone();
                stagnation = 0;
            } else if moved {
                stagnation = 0;
            } else {
                stagnation += 1;
            }

            // --- KRITIEKE LOGICA: EARLY EXIT ---
            // Controleer NA ELKE VERBETERING of we een haalbare oplossing hebben.
            if best_run.is_gamma_feasible(p.gamma_target) {
                // JA! Gevonden. Stop de zoektocht en retourneer dit resultaat.
                return (best_run, false); // Gevonden, niet timed out
            }

            // Diversificatie bij stagnatie
            if stagnation >= p.stagnation_iter {
                // ... (diversificatie logica blijft hetzelfde) ...
                if p.use_mcts {
                    let mut mcts_tree = MctsTree::new(&best_run, graph, p);
                    let removal_seq = mcts_tree.run(rng);
                    cur = apply_lns(&best_run, &removal_seq, p, rng);
                } else {
                    let i = needed_edges.saturating_sub(cur.edges()).min(10);
                    let p_heavy = ((i as f64 + 2.0) / (k as f64)).min(0.1);
                    if rng.gen_bool(p_heavy) {
                        heavy_perturbation(&mut cur, &mut tabu, rng, p, &mut freq_mem);
                    } else {
                        mild_perturbation(&mut cur, &mut tabu, rng, p, &mut freq_mem);
                    }
                }
                stagnation = 0;
                best_run = cur.clone();
            }
        }

        // Update de globale beste oplossing (voor het geval we nooit een haalbare vonden)
        if best_run.density() > best_global.density() {
            best_global = best_run;
        }
    }

    // Retourneer de beste oplossing die we hebben als de tijd om is of max_iter is bereikt
    // ZONDER een haalbare oplossing te vinden.
    (best_global, is_timed_out)
}


/// Helper voor het construeren van een initiële oplossing. (ongewijzigd)
fn initialize_solution<'g, R>(
    graph: &'g Graph,
    k: usize,
    freq_mem: &mut Vec<usize>,
    best_global: &Solution<'g>,
    rng: &mut R,
) -> Solution<'g>
where
    R: Rng + ?Sized,
{
    if best_global.size() == 0 {
        return greedy_random_k(graph, k, rng);
    }

    let min_f = *freq_mem.iter().min().unwrap_or(&0);
    let seed_candidates: Vec<usize> = (0..graph.n()).filter(|&v| freq_mem[v] == min_f).collect();
    
    let mut s = Solution::new(graph);
    if let Some(&seed_node) = seed_candidates.choose(rng) {
        add_counted(&mut s, seed_node, freq_mem);
    } else if graph.n() > 0 {
        let v = rng.gen_range(0..graph.n());
        add_counted(&mut s, v, freq_mem);
    } else {
        return s;
    }

    while s.size() < k {
        let s_bitset = s.bitset();
        let best_v_cand = (0..graph.n())
            .filter(|&v_out| !s_bitset[v_out])
            .max_by_key(|&v_out| {
                let gain = count_intersecting_ones(graph.neigh_row(v_out), s_bitset);
                let freq = freq_mem[v_out];
                (gain, -(freq as isize))
            });

        if let Some(chosen) = best_v_cand {
            add_counted(&mut s, chosen, freq_mem);
        } else {
            break;
        }
    }
    s
}