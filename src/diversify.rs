//! src/diversify.rs
//!
//! Implementeert de adaptieve diversificatiemechanismen (zware en milde perturbaties)
//! voor TSQC, zoals beschreven in `ScriptiePaper.pdf`, Sectie 3.4.2.

use crate::{params::Params, solution::Solution, tabu::DualTabu};
use rand::seq::SliceRandom;
use rand::Rng;

/// Private helper-functie om de handmatige intersectie-telling uit te voeren.
/// Dit is de centrale oplossing voor de E0369-compilerfout.
fn count_intersecting_ones(a: &bitvec::slice::BitSlice, b: &bitvec::slice::BitSlice) -> usize {
    a.iter().by_vals().zip(b.iter().by_vals()).filter(|&(x, y)| x && y).count()
}

/// Zware perturbatie ("grote schok"):
/// 1. Verwijder een willekeurige knoop `u` ∈ S.
/// 2. Bereken de drempel `h` volgens de *gecorrigeerde* formule uit het paper.
/// 3. Verzamel buitenstaanders `v` ∉ S met `deg_in(v) < h`.
/// 4. Voeg een willekeurig gekozen `v` toe.
/// 5. Werk frequentiegeheugen en tabu-lijsten bij.
pub fn heavy_perturbation<'g, R>(
    sol: &mut Solution<'g>,
    tabu: &mut DualTabu,
    rng: &mut R,
    p: &Params,
    freq: &mut Vec<usize>,
) where
    R: Rng + ?Sized,
{
    let k = sol.size();
    if k < 1 {
        return;
    }

    // 1. Kies en verwijder een willekeurige u ∈ S.
    let u = *sol.bitset().iter_ones().collect::<Vec<_>>().choose(rng)
       .expect("Oplossing moet niet-leeg zijn");
    sol.remove(u);

    // 2. Bereken drempel `h` volgens de gecorrigeerde formule (oplossing voor TSQC-02).
    // Formule: h = ceil(0.85*gamma*k) als Dn <= 0.5, anders h = ceil(gamma*k).
    let graph = sol.graph();
    let dn = if graph.n() < 2 { 0.0 } else { 2.0 * graph.m() as f64 / (graph.n() * (graph.n() - 1)) as f64 };
    let h = if dn <= 0.5 {
        (0.85 * p.gamma_target * k as f64).ceil() as usize
    } else {
        (p.gamma_target * k as f64).ceil() as usize
    };

    // 3. Verzamel kandidaten voor toevoeging.
    let sol_bitset = sol.bitset();
    let mut candidates: Vec<usize> = (0..graph.n())
       .filter(|&v| !sol_bitset[v] && count_intersecting_ones(graph.neigh_row(v), sol_bitset) < h)
       .collect();

    // Fallback: als geen enkele knoop < h, neem dan de knopen met de minimale graad.
    if candidates.is_empty() {
        let min_deg_out = (0..graph.n())
           .filter(|&v| !sol_bitset[v])
           .map(|v| count_intersecting_ones(graph.neigh_row(v), sol_bitset))
           .min().unwrap_or(0);
        candidates = (0..graph.n())
           .filter(|&v| !sol_bitset[v] && count_intersecting_ones(graph.neigh_row(v), sol_bitset) == min_deg_out)
           .collect();
    }

    // 4. Voeg een willekeurige kandidaat `v` toe.
    if let Some(&v) = candidates.choose(rng) {
        sol.add(v);

        // 5. Werk frequentiegeheugen bij.
        freq[u] = freq[u].saturating_add(1);
        freq[v] = freq[v].saturating_add(1);
        // Frequentiegeheugen resetten (zoals in het originele paper)
        if freq.iter().any(|&f| f > k) {
            freq.fill(0);
        }
    } else {
        // Geen buitenstaanders gevonden, voeg u terug toe om de grootte te herstellen.
        sol.add(u);
    }

    // 6. Reset tabu en werk duren bij.
    tabu.reset();
    tabu.update_tenures(sol.size(), sol.edges(), p.gamma_target, rng);
}

/// Milde perturbatie ("kleine schok"):
/// 1. Bouw kritieke sets A (u ∈ S met minimale `deg_in`) en B (v ∉ S met maximale `deg_in`).
/// 2. Wissel een willekeurige `u` ∈ A en `v` ∈ B.
/// 3. Werk frequentiegeheugen en tabu-lijsten bij.
pub fn mild_perturbation<'g, R>(
    sol: &mut Solution<'g>,
    tabu: &mut DualTabu,
    rng: &mut R,
    p: &Params,
    freq: &mut Vec<usize>,
) where
    R: Rng + ?Sized,
{
    let k = sol.size();
    if k == 0 {
        return;
    }
    let graph = sol.graph();
    let sol_bitset = sol.bitset();

    // 1. Bouw kritieke sets A en B.
    let min_in = sol_bitset.iter_ones()
       .map(|u| count_intersecting_ones(graph.neigh_row(u), sol_bitset))
       .min().unwrap_or(0);
    let set_a: Vec<usize> = sol_bitset.iter_ones()
       .filter(|&u| count_intersecting_ones(graph.neigh_row(u), sol_bitset) == min_in)
       .collect();

    let max_out = (0..graph.n())
       .filter(|&v| !sol_bitset[v])
       .map(|v| count_intersecting_ones(graph.neigh_row(v), sol_bitset))
       .max().unwrap_or(0);
    let set_b: Vec<usize> = (0..graph.n())
       .filter(|&v| !sol_bitset[v] && count_intersecting_ones(graph.neigh_row(v), sol_bitset) == max_out)
       .collect();

    // 2. Wissel willekeurige u ∈ A en v ∈ B.
    if let (Some(&u), Some(&v)) = (set_a.choose(rng), set_b.choose(rng)) {
        sol.remove(u);
        sol.add(v);

        // 3. Werk frequentiegeheugen bij.
        freq[u] = freq[u].saturating_add(1);
        freq[v] = freq[v].saturating_add(1);
        if freq.iter().any(|&f| f > k) {
            freq.fill(0);
        }
    }

    // 4. Reset tabu en werk duren bij.
    tabu.reset();
    tabu.update_tenures(sol.size(), sol.edges(), p.gamma_target, rng);
}