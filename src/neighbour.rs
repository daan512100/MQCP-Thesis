// Bestand: src/neighbour.rs
//!
//! Implementeert de intensificatiestap (één-swap lokale zoektocht) voor TSQC,
//! met Δ_uv-correctie (d(v) − d(u) − e_uv) en long-term frequentiegeheugen (Secties 3.4.1 & 3.5).
use crate::{
    graph::Graph,
    params::Params,
    solution::Solution,
    tabu::DualTabu,
    freq::{remove_counted, add_counted},
};
use rand::seq::SliceRandom;
use rand::Rng;
use bitvec::slice::BitSlice;

/// Handmatige intersectie-telling voor verbindingen tussen v en S.
/// Dit omzeilt de E0369-compilerfout.
fn count_intersecting_ones(a: &BitSlice, b: &BitSlice) -> usize {
    a.iter().by_vals()
        .zip(b.iter().by_vals())
        .filter(|&(x, y)| x && y)
        .count()
}

/// Probeert één intensificatie-swap (u ∈ A, v ∈ B) uit te voeren.
/// Returns `true` als er geswapped is.
///
/// Deze implementatie volgt de logica uit Sectie 3.4.1 van ScriptiePaper.pdf nauwkeuriger,
/// inclusief de twee gevallen voor het selecteren van de beste swap en willekeurige tie-breaking.
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

    // Bereken MinInS en MaxOutS over niet-taboe knopen
    let (min_in, max_out) = calculate_critical_degrees(sol, tabu);
    if min_in == usize::MAX || max_out == usize::MIN {
        // Geen geldige knopen in A of B die niet tabu zijn, dus geen mogelijke swap.
        tabu.step();
        tabu.update_tenures(sol.size(), sol.edges(), p.gamma_target, rng);
        return false;
    }

    // Bouw kritieke sets A en B (respecteert tabu, zoals gedefinieerd in 3.4.1)
    let (set_a, set_b) = build_critical_sets(sol, tabu, min_in, max_out);
    let current_edges = sol.edges();
    let sol_bitset = sol.bitset();

    // Opties voor de beste swap, inclusief aspiratie
    let mut best_non_tabu_swap: Option<(isize, usize, usize)> = None;
    let mut aspirating_swaps: Vec<(isize, usize, usize)> = Vec::new(); // Meerdere aspirerende kunnen dezelfde delta hebben

    // Kandidaten voor de "beste" swaps (MaxOutS - MinInS, met e_uv = 0)
    let mut best_delta_zero_eu_v_swaps: Vec<(isize, usize, usize)> = Vec::new();
    // Kandidaten voor de "tweede beste" swaps (MaxOutS - MinInS - 1, met e_uv = 1) of andere >= 0
    let mut other_non_deteriorating_swaps: Vec<(isize, usize, usize)> = Vec::new();


    for &u in &set_a {
        let loss = count_intersecting_ones(graph.neigh_row(u), sol_bitset);
        for &v in &set_b {
            let gain = count_intersecting_ones(graph.neigh_row(v), sol_bitset);
            let e_uv = if graph.neigh_row(u)[v] { 1 } else { 0 };
            let delta = gain as isize - loss as isize - e_uv as isize;

            let is_tabu_move = tabu.is_tabu_u(u) || tabu.is_tabu_v(v);

            // Bereken de dichtheid na de potentiële swap
            let new_edges = (current_edges as isize + delta) as usize;
            let new_rho = if k < 2 {
                0.0
            } else {
                2.0 * (new_edges as f64) / ((k * (k - 1)) as f64)
            };

            // Aspiratiecriterium: accepteer taboe-move als dichtheid verbetert best_global_rho
            if is_tabu_move && new_rho > best_global_rho {
                aspirating_swaps.push((delta, u, v));
            }

            // Alleen niet-tabu en niet-verslechterende moves overwegen voor de "beste" move sets
            if !is_tabu_move && delta >= 0 {
                // Check voor het specifieke "MaxOutS - MinInS" type swap (e_uv = 0)
                if e_uv == 0 && delta == (max_out as isize - min_in as isize) {
                    best_delta_zero_eu_v_swaps.push((delta, u, v));
                } else {
                    // Alle andere niet-verslechterende swaps
                    other_non_deteriorating_swaps.push((delta, u, v));
                }
            }
        }
    }

    // Selecteer de swap volgens de paper logica
    let mut chosen_swap: Option<(isize, usize, usize)> = None;

    // Prioriteit 1: Aspirating moves (ongeacht hun delta in vergelijking met andere niet-tabu moves)
    // De beste aspirerende move is die met de hoogste delta.
    if let Some(&(delta, u, v)) = aspirating_swaps.iter().max_by_key(|(d, _, _)| *d) {
        chosen_swap = Some((delta, u, v));
    }


    // Prioriteit 2: Zoek de "beste swap" zoals gedefinieerd in het paper:
    // "MaxOutS - MinInS" en (u,v) niet verbonden (e_uv=0) met delta >= 0
    // Als er meerdere zijn, kies er willekeurig een.
    if chosen_swap.is_none() {
        if !best_delta_zero_eu_v_swaps.is_empty() {
             // Paper: "TSQ select a pair of vertices (u, v) randomly from T"
            chosen_swap = best_delta_zero_eu_v_swaps.choose(rng).cloned();
        } else {
            // Prioriteit 3: Anders, als "MaxOutS - MinInS - 1 >= 0"
            // Paper: "select a vertex u from A randomly and a vertex v from B randomly and swap u and v."
            // Dit impliceert dat we een willekeurige niet-verslechterende swap willen vinden.
            // De oorspronkelijke code zocht de algemene beste, maar het paper suggereert willekeur.
            // We verzamelen alle niet-tabu, niet-verslechterende swaps die geen "beste delta" waren.
            if !other_non_deteriorating_swaps.is_empty() {
                 chosen_swap = other_non_deteriorating_swaps.choose(rng).cloned();
            } else {
                // Als er geen positieve delta swaps zijn, kan er geen intensificatie plaatsvinden.
                // Het paper zegt "Otherwise it doesn't exist a swap (u, v) such as Δ_uv >= 0".
                // De `stagnation` counter in `restart.rs` zal dit dan afhandelen.
                chosen_swap = None;
            }
        }
    }
    
    let mut did_swap = false;
    if let Some((_, u, v)) = chosen_swap {
        // Gebruik long-term freq-helpers voor remove/add mét reset
        remove_counted(sol, u, freq);
        add_counted(sol, v, freq);
        tabu.forbid_u(u);
        tabu.forbid_v(v);
        did_swap = true;
    }

    tabu.step();
    tabu.update_tenures(sol.size(), sol.edges(), p.gamma_target, rng);
    did_swap
}

/// Berekent MinInS en MaxOutS voor niet-taboe knopen (Sectie 3.4.1).
/// Dit is al correct geïmplementeerd en hoeft niet gewijzigd te worden.
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

/// Bouwt kritieke sets A (min_in) en B (max_out) (Sectie 3.4.1).
/// Dit is al correct geïmplementeerd en hoeft niet gewijzigd te worden.
/// De `&mut DualTabu` in de signatuur was een typefout, zou `&DualTabu` moeten zijn.
/// Aangezien deze functie geen mutatie op `tabu` uitvoert, is de `&` referentie correct.
fn build_critical_sets(
    sol: &Solution,
    tabu: &DualTabu, // Corrigeeerd: was &mut DualTabu
    min_in: usize,
    max_out: usize,
) -> (Vec<usize>, Vec<usize>) {
    let graph = sol.graph();
    let sol_bitset = sol.bitset();
    let set_a: Vec<usize> = sol_bitset.iter_ones()
        .filter(|&u| !tabu.is_tabu_u(u)
            && count_intersecting_ones(graph.neigh_row(u), sol_bitset) == min_in)
        .collect();
    let set_b: Vec<usize> = (0..graph.n())
        .filter(|&v| !sol_bitset[v]
            && !tabu.is_tabu_v(v)
            && count_intersecting_ones(graph.neigh_row(v), sol_bitset) == max_out)
        .collect();
    (set_a, set_b)
}