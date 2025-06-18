//! src/construct.rs
//!
//! Heuristieken voor het construeren van een initiële subset `S`.
//! Implementeert de "Greedy random heuristic" zoals beschreven in
//! `ScriptiePaper.pdf`, Sectie 3.3.
use crate::{graph::Graph, solution::Solution};
use rand::seq::SliceRandom;
use rand::Rng;

/// Creëert een initiële oplossing van grootte `k` met de greedy-random heuristiek.
/// 1. Selecteer een willekeurige knoop als startpunt.
/// 2. Voeg iteratief de knoop toe die de meeste buren heeft binnen de huidige set.
///    Bij een gelijke stand wordt een willekeurige kandidaat gekozen.
///
/// Dit volgt exact de procedure beschreven in Sectie 3.3 van het paper.
pub fn greedy_random_k<'g, R>(graph: &'g Graph, k: usize, rng: &mut R) -> Solution<'g>
where
    R: Rng + ?Sized,
{
    assert!(k > 0 && k <= graph.n(), "k moet binnen het bereik [1, n] liggen");
    let mut sol = Solution::new(graph);

    // 1. Selecteer een willekeurige startknoop.
    if graph.n() > 0 {
        sol.add(rng.gen_range(0..graph.n()));
    }

    // 2. Voeg iteratief de beste knoop toe tot grootte k is bereikt.
    while sol.size() < k {
        let mut best_edges = usize::MIN;
        let mut candidates = Vec::new();
        let sol_bitset = sol.bitset();

        for v in 0..graph.n() {
            if !sol_bitset[v] { // Alleen knopen buiten de oplossing overwegen
                
                // CORRECTIE (E0369): De `&`-operator wordt vervangen door een handmatige
                // en performante intersectie-telling via iterators. Dit omzeilt het
                // onverwachte compilerprobleem zonder prestatieverlies.
                let edges = graph
                    .neigh_row(v)
                    .iter()
                    .by_vals()
                    .zip(sol_bitset.iter().by_vals())
                    .filter(|&(a, b)| a && b)
                    .count();

                if edges > best_edges {
                    best_edges = edges;
                    candidates.clear();
                    candidates.push(v);
                } else if edges == best_edges {
                    candidates.push(v);
                }
            }
        }

        // Kies willekeurig uit de beste kandidaten (tie-breaking).
        if let Some(&chosen) = candidates.choose(rng) {
            sol.add(chosen);
        } else {
            // Dit zou niet moeten gebeuren in een graaf met k < n.
            // Als er geen kandidaten meer zijn, stop dan.
            break;
        }
    }
    sol
}