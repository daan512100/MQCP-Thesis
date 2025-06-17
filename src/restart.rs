//! src/restart.rs
//!
//! Implementeert de multi-start Tabu Search voor een vaste `k` (`solve_fixed_k`),
//! wat de kern is van het TSQC-algoritme (Algoritmes 1 & 2 in ScriptiePaper.pdf).
use crate::{
    construct::greedy_random_k,
    diversify::{heavy_perturbation, mild_perturbation},
    lns::apply_lns,
    mcts::MctsTree,
    neighbour::improve_once,
    params::Params,
    solution::Solution,
    tabu::DualTabu,
    Graph,
};
use rand::seq::SliceRandom;
use rand::Rng;

/// Zoekt naar een `gamma`-quasi-clique van vaste grootte `k`.
pub fn solve_fixed_k<'g, R>(
    graph: &'g Graph,
    k: usize,
    rng: &mut R,
    p: &Params,
) -> Solution<'g>
where
    // `Send + Sync` is nodig om de RNG thread-safe te maken voor parallelle MCTS.
    R: Rng + ?Sized + Send + Sync,
{
    // 0. Pre-berekening van benodigde kanten voor haalbaarheid.
    let max_possible_edges = if k > 1 { k * (k - 1) / 2 } else { 0 };
    let needed_edges = (p.gamma_target * max_possible_edges as f64).ceil() as usize;
    if max_possible_edges < needed_edges {
        return Solution::new(graph); // Onmogelijk om doel te bereiken.
    }

    let mut freq_mem = vec![0usize; graph.n()];
    let mut best_global = Solution::new(graph);
    let mut best_global_rho = 0.0;
    let mut total_moves = 0usize;

    // Buitenste restart-lus
    while total_moves < p.max_iter {
        // 1. INITIALISATIE OPLOSSING
        let mut cur = if best_global.size() == 0 {
            // Eerste run: start met een greedy-random oplossing.
            greedy_random_k(graph, k, rng)
        } else {
            // Restart-strategie (§3.5): start vanaf minst gebruikte knoop.
            let min_f = *freq_mem.iter().min().unwrap_or(&0);
            let candidates: Vec<usize> = (0..graph.n()).filter(|&v| freq_mem[v] == min_f).collect();
            
            let mut s = Solution::new(graph);
            if let Some(&seed_node) = candidates.choose(rng) {
                s.add(seed_node);
            } else if graph.n() > 0 {
                // Fallback als candidates leeg is (zeldzaam).
                s.add(rng.gen_range(0..graph.n()));
            } else {
                return best_global; // Lege graaf.
            }

            // Vul aan met greedy toevoegingen tot grootte k.
            while s.size() < k {
                let mut best_gain = isize::MIN;
                let mut best_v_cand = Vec::new();
                let s_bitset = s.bitset();
                for v_out in 0..graph.n() {
                    if !s_bitset[v_out] {
                        
                        // CORRECTIE (E0369): De `&`-operator wordt vervangen door een handmatige
                        // en performante intersectie-telling via iterators.
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
                if let Some(&chosen) = best_v_cand.choose(rng) {
                    s.add(chosen);
                } else { break; }
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

            // Update beste oplossing van deze run
            if cur.density() > best_run.density() {
                best_run = cur.clone();
            }

            // Vroege uitstap als een haalbare oplossing is gevonden
            if best_run.is_gamma_feasible(p.gamma_target) {
                return best_run;
            }
            
            // 3b. DIVERSIFICATIE bij stagnatie
            if stagnation >= p.stagnation_iter {
                if p.use_mcts {
                    // MCTS-LNS diversificatie
                    let mut mcts_tree = MctsTree::new(&cur, graph, p);
                    let removal_seq = mcts_tree.run(rng);
                    cur = apply_lns(&cur, &removal_seq, p, rng);
                } else {
                    // Standaard TSQC diversificatie
                    // Correctie voor TSQC-01: gebruik absolute, afgetopte deficit `I`.
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

        // TSQC-04: Documenteer de elitistische update van freq_mem.
        // Deze stap, die alle knopen in de beste oplossing van de run versterkt,
        // is een afwijking van het paper maar een potentieel nuttige
        // heuristische verbetering die hier behouden blijft.
        for v in best_run.bitset().iter_ones() {
            freq_mem[v] = freq_mem[v].saturating_add(1);
        }
    }

    best_global
}