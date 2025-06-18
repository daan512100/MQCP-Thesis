// Bestand: src/diversify.rs
//!
//! Implementeert de adaptieve diversificatiemechanismen (zware en milde perturbaties)
//! voor TSQC, zoals beschreven in `ScriptiePaper.pdf`, Sectie 3.4.2.
//! Gebruikt long-term frequentiegeheugen via `add_counted`/`remove_counted` (Sectie 3.5).
use crate::{
    params::Params,
    solution::Solution,
    tabu::DualTabu, // Importeren we DualTabu voor de tabu-checks
    freq::{add_counted, remove_counted},
    graph::Graph, // Graaf is nodig om de buren op te vragen
};
use rand::seq::SliceRandom;
use rand::Rng;
use bitvec::slice::BitSlice;

/// Handmatige intersectie-telling voor verbindingen tussen v en S.
fn count_intersecting_ones(a: &BitSlice, b: &BitSlice) -> usize {
    a.iter().by_vals()
        .zip(b.iter().by_vals())
        .filter(|&(x, y)| x && y)
        .count()
}

/// Zware perturbatie ("grote schok"):
/// 1. Verwijder willekeurige u ∈ S.
/// 2. Bereken drempel h = floor(0.85 * γ * k) als dn ≤ 0.5, anders floor(γ * k).
/// 3. Kies v ∉ S met |N(v) ∩ S| < h (strict kleiner dan).
/// 4. Voeg v toe.
/// 5. Update long-term freq via helper.
/// 6. Reset tabu en update tenures.
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
    let u = *sol
        .bitset()
        .iter_ones()
        .collect::<Vec<_>>()
        .choose(rng)
        .expect("Oplossing moet niet-leeg zijn");
    remove_counted(sol, u, freq);

    // 2. Bereken de drempel h volgens gepaste formule.
    let graph = sol.graph();
    let dn = if graph.n() < 2 {
        0.0
    } else {
        2.0 * graph.m() as f64 / ((graph.n() * (graph.n() - 1)) as f64)
    };
    let h = if dn <= 0.5 {
        (0.85 * p.gamma_target * (k as f64)).floor() as usize
    } else {
        (p.gamma_target * (k as f64)).floor() as usize
    };

    // 3. Verzamel kandidaten v ∉ S met |N(v) ∩ S| < h (strict kleiner dan, zoals in paper).
    let sol_bitset = sol.bitset();
    let mut candidates: Vec<usize> = (0..graph.n())
        .filter(|&v| !sol_bitset[v]
            && count_intersecting_ones(graph.neigh_row(v), sol_bitset) < h) // Correctie: van <= naar <
        .collect();

    // Fallback: als geen kandidaten voldoen aan strict <h, kies dan v met minimale out-degree.
    // Dit is een pragmatische toevoeging om te garanderen dat er een knoop wordt gevonden,
    // en lost het probleem op dat in het paper wordt genoemd (`d(v) < 0` onmogelijk).
    if candidates.is_empty() {
        let min_deg_out = (0..graph.n())
            .filter(|&v| !sol_bitset[v])
            .map(|v| count_intersecting_ones(graph.neigh_row(v), sol_bitset))
            .min()
            .unwrap_or(0);
        candidates = (0..graph.n())
            .filter(|&v| !sol_bitset[v]
                && count_intersecting_ones(graph.neigh_row(v), sol_bitset) == min_deg_out)
            .collect();
        // Als zelfs dan nog leeg (bv. lege graaf of k=n), kan er geen knoop worden toegevoegd
        if candidates.is_empty() {
            return; // Kan geen knoop toevoegen, perturbatie mislukt.
        }
    }

    // 4. Voeg willekeurige kandidaat v toe en update freq.
    if let Some(&v) = candidates.choose(rng) {
        add_counted(sol, v, freq);
    } else {
        // Dit zou niet moeten gebeuren door de fallback, maar voor de zekerheid
        return;
    }

    // 5. Reset tabu en update tenures.
    tabu.reset();
    tabu.update_tenures(sol.size(), sol.edges(), p.gamma_target, rng);
}

/// Milde perturbatie ("kleine schok"):
/// 1. Bereken MinInS en MaxOutS over S en V\S, rekening houdend met tabu-lijsten (conform 3.4.1).
/// 2. Kies u ∈ S met |N(u) ∩ S| = MinInS, v ∉ S met |N(v) ∩ S| = MaxOutS.
/// 3. Verwissel u en v.
/// 4. Update long-term freq.
/// 5. Reset tabu en update tenures.
pub fn mild_perturbation<'g, R>(
    sol: &mut Solution<'g>,
    tabu: &mut DualTabu,
    rng: &mut R,
    p: &Params,
    freq: &mut Vec<usize>,
) where
    R: Rng + ?Sized,
{
    let graph = sol.graph();
    let k = sol.size();
    if k < 2 { // Minimaal 2 knopen nodig voor een swap
        return;
    }

    let sol_bitset = sol.bitset();

    // Correctie: Bereken MinInS en MaxOutS voor niet-taboe knopen (conform Sectie 3.4.1)
    let min_in = sol_bitset
        .iter_ones()
        .filter(|&u| !tabu.is_tabu_u(u)) // Filter op niet-tabu
        .map(|u| count_intersecting_ones(graph.neigh_row(u), sol_bitset))
        .min()
        .unwrap_or(usize::MAX);
    
    let max_out = (0..graph.n())
        .filter(|&v| !sol_bitset[v] && !tabu.is_tabu_v(v)) // Filter op niet in S en niet-tabu
        .map(|v| count_intersecting_ones(graph.neigh_row(v), sol_bitset))
        .max()
        .unwrap_or(usize::MIN);

    // Als er geen geldige niet-taboe knopen zijn, kunnen we niet swappen
    if min_in == usize::MAX || max_out == usize::MIN {
        tabu.reset(); // Toch resetten van tabu lijsten als geen swap mogelijk
        tabu.update_tenures(sol.size(), sol.edges(), p.gamma_target, rng);
        return;
    }

    // 2. Bouw kritieke sets A en B (conform Sectie 3.4.1, nu met tabu-checks ingebouwd via min_in/max_out)
    let set_a: Vec<usize> = sol_bitset
        .iter_ones()
        .filter(|&u| !tabu.is_tabu_u(u) && count_intersecting_ones(graph.neigh_row(u), sol_bitset) == min_in)
        .collect();
    let set_b: Vec<usize> = (0..graph.n())
        .filter(|&v| !sol_bitset[v] && !tabu.is_tabu_v(v) && count_intersecting_ones(graph.neigh_row(v), sol_bitset) == max_out)
        .collect();
    
    // 3. Kies en verwissel.
    if let (Some(&u), Some(&v)) = (set_a.choose(rng), set_b.choose(rng)) {
        remove_counted(sol, u, freq);
        add_counted(sol, v, freq);
    } else {
        // Mocht er onverhoopt geen geschikte u of v gevonden worden na filtering
        tabu.reset(); // Toch resetten van tabu lijsten
        tabu.update_tenures(sol.size(), sol.edges(), p.gamma_target, rng);
        return;
    }

    // 4. Reset tabu en update tenures.
    tabu.reset();
    tabu.update_tenures(sol.size(), sol.edges(), p.gamma_target, rng);
}