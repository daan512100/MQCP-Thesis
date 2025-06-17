// Bestand: src/restart.rs
//!
///! Implementeert de multi-start Tabu Search voor een vaste `k` (`solve_fixed_k`),
///! met het exacte long-term frequentiegeheugen volgens Sectie 3.5 van ScriptiePaper.pdf.

use crate::{
    construct::greedy_random_k,
    diversify::{heavy_perturbation, mild_perturbation},
    freq::{add_counted},
    lns::apply_lns,
    mcts::MctsTree,
    neighbour::improve_once,
    params::Params,
    solution::Solution,
    tabu::DualTabu,
    graph::Graph,
};
use rand::seq::SliceRandom;
use rand::Rng;

/// Zoekt naar een `γ`-quasi-clique van vaste grootte `k`,
/// met long-term frequentiegeheugen conform Sectie 3.5:
/// elke move telt, en bij overflow (gₙ(v) > |S|) resetten alle tellers.
pub fn solve_fixed_k<'g, R>(
    graph: &'g Graph,
    k: usize,
    rng: &mut R,
    p: &Params,
) -> Solution<'g>
where
    R: Rng + ?Sized + Send + Sync,
{
    // 0. Pre-berekening van benodigde kanten.
    let max_possible_edges = if k > 1 { k * (k - 1) / 2 } else { 0 };
    let needed_edges = (p.gamma_target * max_possible_edges as f64).ceil() as usize;
    if max_possible_edges < needed_edges {
        return Solution::new(graph);
    }

    // Long-term frequency memory per vertex (gₙ(v)), init op nul.
    let mut freq_mem = vec![0usize; graph.n()];
    let mut best_global = Solution::new(graph);
    let mut best_global_rho = 0.0;
    let mut total_moves = 0usize;

    // Buitenste restart-lus
    while total_moves < p.max_iter {
        // 1. INITIALISATIE OPLOSSING
        let mut cur = if best_global.size() == 0 {
            // Eerste run: standaard greedy-random (frequenties nog niet gebruikt)
            greedy_random_k(graph, k, rng)
        } else {
            // Restart-strategie (§3.5): kies seed met minimaal gₙ(v)
            let min_f = *freq_mem.iter().min().unwrap_or(&0);
            let seed_candidates: Vec<usize> =
                (0..graph.n()).filter(|&v| freq_mem[v] == min_f).collect();

            let mut s = Solution::new(graph);
            if let Some(&seed_node) = seed_candidates.choose(rng) {
                add_counted(&mut s, seed_node, &mut freq_mem);
            } else {
                // Fallback
                let v = rng.gen_range(0..graph.n());
                add_counted(&mut s, v, &mut freq_mem);
            }

            // Greedy aanvulling met secundaire tie-break op freq_mem
            while s.size() < k {
                let mut best_gain = isize::MIN;
                let mut best_v_cand = Vec::new();
                let s_bitset = s.bitset();
                for v_out in 0..graph.n() {
                    if !s_bitset[v_out] {
                        let gain = graph
                            .neigh_row(v_out)
                            .iter()
                            .by_vals()
                            .zip(s_bitset.iter().by_vals())
                            .filter(|&(a, b)| a && b)
                            .count() as isize;
                        if gain > best_gain {
                            best_gain = gain;
                            best_v_cand.clear();
                            best_v_cand.push(v_out);
                        } else if gain == best_gain {
                            best_v_cand.push(v_out);
                        }
                    }
                }
                if best_v_cand.is_empty() {
                    break;
                }
                // TSQC-05: secondaires tie-breaking via long-term freq
                let min_freq = best_v_cand.iter().map(|&v| freq_mem[v]).min().unwrap_or(0);
                let filtered: Vec<usize> = best_v_cand
                    .into_iter()
                    .filter(|&v| freq_mem[v] == min_freq)
                    .collect();
                if let Some(&chosen) = filtered.choose(rng) {
                    add_counted(&mut s, chosen, &mut freq_mem);
                } else {
                    break;
                }
            }
            s
        };

        // 2. INITIALISATIE TABU
        let mut tabu = DualTabu::new(graph.n(), p.tenure_u, p.tenure_v);
        tabu.update_tenures(cur.size(), cur.edges(), p.gamma_target, rng);

        let mut best_run = cur.clone();
        let mut stagnation = 0usize;

        // 3. LOKALE ZOEKTOCHT (intensificatie/diversificatie)
        while stagnation < p.stagnation_iter && total_moves < p.max_iter {
            let moved = improve_once(&mut cur, &mut tabu, best_global_rho, &mut freq_mem, p, rng);
            total_moves += 1;
            if moved {
                stagnation = 0;
            } else {
                stagnation += 1;
            }

            if cur.density() > best_run.density() {
                best_run = cur.clone();
            }
            if best_run.is_gamma_feasible(p.gamma_target) {
                return best_run;
            }

            // 3b. Diversificatie bij stagnatie
            if stagnation >= p.stagnation_iter {
                if p.use_mcts {
                    let mut mcts_tree = MctsTree::new(&cur, graph, p);
                    let removal_seq = mcts_tree.run(rng);
                    cur = apply_lns(&cur, &removal_seq, p, rng);
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
            }
        }

        // 4. UPDATE GLOBALE BESTE OPLOSSING
        if best_run.density() > best_global.density() {
            best_global = best_run.clone();
            best_global_rho = best_global.density();
        }

        // uitsluitend via add_counted/remove_counted en perturbaties gehanteerd.
    }

    best_global
}
