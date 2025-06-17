//! src/neighbour.rs
//!
//! Implementeert de intensificatiestap (één-swap lokale zoektocht) voor TSQC,
//! zoals beschreven in `ScriptiePaper.pdf`, Sectie 3.4.1.

use crate::{params::Params, solution::Solution, tabu::DualTabu};
use rand::Rng;

/// Private helper-functie om de handmatige intersectie-telling uit te voeren.
/// Dit is de centrale oplossing voor de E0369-compilerfout.
fn count_intersecting_ones(a: &bitvec::slice::BitSlice, b: &bitvec::slice::BitSlice) -> usize {
    a.iter().by_vals().zip(b.iter().by_vals()).filter(|&(x, y)| x && y).count()
}

/// Probeert een enkele intensificatiezet uit te voeren.
///
/// Deze functie implementeert de gecorrigeerde en volledige logica:
/// 1. Berekent `MinInS` en `MaxOutS` op basis van *niet-taboe* knopen (oplossing voor `TSQC-05`).
/// 2. Bouwt kritieke sets A en B.
/// 3. Evalueert alle swaps (u∈A, v∈B) en selecteert de beste volgens de hiërarchie
///    uit het paper: eerst verbeterend, dan niet-verslechterend, dan aspiratie.
/// 4. Voert de swap uit, werkt frequentiegeheugen en tabu-lijsten bij.
///
/// - `best_global_rho`: beste dichtheid tot nu toe (voor aspiratiecriterium).
/// - `freq`: lange-termijn frequentiegeheugen.
///
/// Geeft `true` terug als er een swap is uitgevoerd.
pub fn improve_once<'g, R>(
    sol: &mut Solution<'g>,
    tabu: &mut DualTabu,
    best_global_rho: f64,
    freq: &mut Vec<usize>,
    p: &Params,
    rng: &mut R,
) -> bool
where
    R: Rng + ?Sized,
{
    let graph = sol.graph();
    let k = sol.size();
    if k == 0 || k == graph.n() {
        tabu.step();
        tabu.update_tenures(sol.size(), sol.edges(), p.gamma_target, rng);
        return false;
    }

    // --- Correctie voor TSQC-05: Bereken MinInS en MaxOutS over niet-taboe knopen ---
    let (min_in, max_out) = calculate_critical_degrees(sol, tabu);
    if min_in == usize::MAX || max_out == usize::MIN {
        // Geen geldige zetten mogelijk
        tabu.step();
        tabu.update_tenures(sol.size(), sol.edges(), p.gamma_target, rng);
        return false;
    }

    // Bouw kritieke sets A en B op basis van de correct berekende graden
    let (set_a, set_b) = build_critical_sets(sol, tabu, min_in, max_out);
    
    // --- Zoek de beste swap volgens de hiërarchie van het paper ---
    let mut best_allowed: Option<(isize, usize, usize)> = None; // (delta, u, v)
    let mut best_aspire: Option<(isize, usize, usize)> = None;

    let current_edges = sol.edges();
    let sol_bitset = sol.bitset();
    for &u in &set_a {
        // De verandering in het aantal kanten is `gain - loss`.
        // `loss` is het aantal buren van `u` binnen de huidige oplossing `S`.
        let loss = count_intersecting_ones(graph.neigh_row(u), sol_bitset);
        for &v in &set_b {
            // `gain` is het aantal buren van `v` binnen de huidige oplossing `S`.
            let gain = count_intersecting_ones(graph.neigh_row(v), sol_bitset);
            
            // De totale verandering in kanten (delta) bij het swappen van u en v is
            // `gain - loss`. Als u en v buren zijn, wordt die kant niet meegeteld
            // in de interne graden, maar de kant gaat ook niet verloren, dus de
            // formule is correct.
            let delta = gain as isize - loss as isize;

            let is_tabu = tabu.is_tabu_u(u) || tabu.is_tabu_v(v);

            if !is_tabu {
                // Vergelijk met de beste toegestane zet tot nu toe
                if best_allowed.is_none() || delta >= best_allowed.unwrap().0 {
                    best_allowed = Some((delta, u, v));
                }
            } else {
                // Controleer aspiratiecriterium: leidt de zet tot een betere oplossing dan de globale beste?
                let new_edges = (current_edges as isize + delta) as usize;
                let new_rho = Solution::calculate_density(k, new_edges);
                if new_rho > best_global_rho {
                    // Vergelijk met de beste aspiratiezet tot nu toe
                    if best_aspire.is_none() || delta >= best_aspire.unwrap().0 {
                        best_aspire = Some((delta, u, v));
                    }
                }
            }
        }
    }

    // Prioriteer een toegestane zet boven een aspiratiezet
    let chosen_swap = best_allowed.or(best_aspire);
    let did_swap = if let Some((_, u, v)) = chosen_swap {
        // Voer de swap uit
        sol.remove(u);
        sol.add(v);

        // Werk frequentiegeheugen bij
        freq[u] = freq[u].saturating_add(1);
        freq[v] = freq[v].saturating_add(1);
        // Markeer als taboe
        tabu.forbid_u(u);
        tabu.forbid_v(v);
        true
    } else {
        false
    };
    
    // Verhoog altijd de tabu-teller en werk de duren bij
    tabu.step();
    tabu.update_tenures(sol.size(), sol.edges(), p.gamma_target, rng);
    did_swap
}

// Hulpfunctie om de graden van de kritieke sets te berekenen.
fn calculate_critical_degrees(sol: &Solution, tabu: &DualTabu) -> (usize, usize) {
    let graph = sol.graph();
    let sol_bitset = sol.bitset();
    let min_in = sol_bitset.iter_ones()
       .filter(|&u| !tabu.is_tabu_u(u))
       .map(|u| count_intersecting_ones(graph.neigh_row(u), sol_bitset))
       .min().unwrap_or(usize::MAX);

    let max_out = (0..graph.n())
       .filter(|&v| !sol_bitset[v] && !tabu.is_tabu_v(v))
       .map(|v| count_intersecting_ones(graph.neigh_row(v), sol_bitset))
       .max().unwrap_or(usize::MIN);
    (min_in, max_out)
}

// Hulpfunctie om de kritieke sets A en B te bouwen.
fn build_critical_sets(sol: &Solution, tabu: &DualTabu, min_in: usize, max_out: usize) -> (Vec<usize>, Vec<usize>) {
    let graph = sol.graph();
    let sol_bitset = sol.bitset();

    let set_a: Vec<usize> = sol_bitset.iter_ones()
       .filter(|&u| !tabu.is_tabu_u(u) && count_intersecting_ones(graph.neigh_row(u), sol_bitset) == min_in)
       .collect();
    let set_b: Vec<usize> = (0..graph.n())
       .filter(|&v| !sol_bitset[v] && !tabu.is_tabu_v(v) && count_intersecting_ones(graph.neigh_row(v), sol_bitset) == max_out)
       .collect();
    (set_a, set_b)
}

// Statische hulpfunctie binnen de module om dichtheid te berekenen zonder een Solution-instantie.
// Dit is nodig voor het aspiratiecriterium.
impl<'g> Solution<'g> {
    fn calculate_density(size: usize, edges: usize) -> f64 {
        if size < 2 {
            0.0
        } else {
            2.0 * edges as f64 / (size * (size - 1)) as f64
        }
    }
}