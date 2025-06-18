// Bestand: src/restart.rs
//!
//! Implementeert de multi-start Tabu Search voor een vaste `k` (`solve_fixed_k`),
//! met het exacte long-term frequentiegeheugen volgens Sectie 3.5 van ScriptiePaper.pdf.

use crate::{
    construct::greedy_random_k,
    diversify::heavy_perturbation, // 'mild_perturbation' wordt niet langer gebruikt.
    freq::{add_counted, remove_counted},
    graph::Graph,
    lns::apply_lns,
    mcts::MctsTree,
    neighbour::improve_once,
    params::Params,
    solution::Solution,
    tabu::DualTabu,
};
use bitvec::slice::BitSlice;
use rand::seq::SliceRandom;
use rand::Rng;

// Hulpfunctie om de interne graad te tellen, om code duplicatie te vermijden.
fn count_intersecting_ones(a: &BitSlice, b: &BitSlice) -> usize {
    a.iter()
        .by_vals()
        .zip(b.iter().by_vals())
        .filter(|&(x, y)| x && y)
        .count()
}

/// Zoekt naar een `γ`-quasi-clique van vaste grootte `k`,
/// met long-term frequentiegeheugen conform Sectie 3.5.
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
                if graph.n() == 0 { return best_global; }
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
                        let gain = count_intersecting_ones(graph.neigh_row(v_out), s_bitset) as isize;
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
                // TSQC-05: secondaire tie-breaking via long-term freq
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
                        // =================================================================================
                        // CORRECTIE (Afwijking 2): De 'low perturbation' actie is volledig herschreven.
                        //
                        // REDEN: `ScriptiePaper.pdf`, Sectie 3.4.2, schrijft voor dat met kans `1-P` een
                        // zet wordt gekozen uit de kritieke sets T, A en B, niet een `mild_perturbation`.
                        // De aanroep naar `mild_perturbation` is verwijderd en vervangen door de
                        // correcte logica hieronder.
                        // =================================================================================
                        
                        // Stap 1: Bepaal de kritieke sets A en B, rekening houdend met tabu.
                        let sol_bitset = cur.bitset();
                        let min_in = sol_bitset.iter_ones()
                            .filter(|&u| !tabu.is_tabu_u(u))
                            .map(|u| count_intersecting_ones(graph.neigh_row(u), sol_bitset))
                            .min().unwrap_or(usize::MAX);

                        let max_out = (0..graph.n())
                            .filter(|&v| !sol_bitset[v] && !tabu.is_tabu_v(v))
                            .map(|v| count_intersecting_ones(graph.neigh_row(v), sol_bitset))
                            .max().unwrap_or(usize::MIN);

                        let set_a: Vec<usize> = sol_bitset.iter_ones()
                            .filter(|&u| !tabu.is_tabu_u(u) && count_intersecting_ones(graph.neigh_row(u), sol_bitset) == min_in)
                            .collect();

                        let set_b: Vec<usize> = (0..graph.n())
                            .filter(|&v| !sol_bitset[v] && !tabu.is_tabu_v(v) && count_intersecting_ones(graph.neigh_row(v), sol_bitset) == max_out)
                            .collect();

                        // Stap 2: Bepaal set T (beste swaps, waar u en v niet verbonden zijn)
                        let set_t: Vec<(usize, usize)> = set_a.iter().flat_map(|&u| {
                                set_b.iter().filter(move |&&v| !graph.neigh_row(u)[v]).map(move |&v| (u, v))
                            }).collect();
                        
                        // Stap 3: Voer de swap uit volgens de regels van het paper.
                        if let Some(&(u, v)) = set_t.choose(rng) {
                            // Kies willekeurig uit T
                            remove_counted(&mut cur, u, &mut freq_mem);
                            add_counted(&mut cur, v, &mut freq_mem);
                            tabu.forbid_u(u);
                            tabu.forbid_v(v);
                        } else if let (Some(&u), Some(&v)) = (set_a.choose(rng), set_b.choose(rng)) {
                            // T is leeg, kies willekeurig uit A en B
                            remove_counted(&mut cur, u, &mut freq_mem);
                            add_counted(&mut cur, v, &mut freq_mem);
                            tabu.forbid_u(u);
                            tabu.forbid_v(v);
                        }
                        // De tabu-teller wordt aan het einde van de 'improve_once' call al verhoogd.
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
    }

    best_global
}