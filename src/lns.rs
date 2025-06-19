// src/lns.rs

use crate::{
    graph::Graph, // --- TOEGEVOEGD: expliciete import voor count_connections ---
    neighbour::improve_once, 
    params::Params, 
    solution::Solution, 
    tabu::DualTabu,
};
use rand::seq::SliceRandom; // --- TOEGEVOEGD: voor .choose() op de RCL ---
use rand::Rng;

pub fn apply_lns<'g, R>(
    initial_sol: &Solution<'g>,
    removals: &[usize],
    p: &Params,
    rng: &mut R,
) -> Solution<'g>
where
    R: Rng + ?Sized,
{
    let mut sol = initial_sol.clone();
    for &v in removals {
        sol.remove(v);
    }

    let target_k = initial_sol.size();
    let graph = initial_sol.graph();

    // --- FASE 1: Gerandomiseerde Greedy Completion (GRASP) ---
    // Deze hele `while`-lus is de nieuwe, slimmere logica.
    while sol.size() < target_k {
        let sol_bitset = sol.bitset();
        
        // Stap 1: Verzamel alle mogelijke kandidaten buiten de oplossing en hun 'gain'.
        let candidates: Vec<(usize, isize)> = (0..graph.n())
            .filter(|&v| !sol_bitset[v])
            .map(|v| {
                // Gebruik de `count_connections` methode van Solution, die al geoptimaliseerd is.
                let gain = sol.count_connections(v) as isize;
                (v, gain)
            })
            .collect();

        // Als er geen kandidaten meer zijn, kunnen we niet verder.
        if candidates.is_empty() {
            break;
        }

        // Stap 2: Bepaal de hoogst mogelijke gain van alle kandidaten.
        let best_gain = match candidates.iter().map(|&(_, g)| g).max() {
            Some(g) => g,
            None => break, // Veiligheid: stop als de lijst leeg zou zijn.
        };
        
        // Stap 3: Bouw de Restricted Candidate List (RCL).
        // Bepaal de drempel op basis van de beste gain en de nieuwe alpha-parameter.
        let rcl_threshold = (best_gain as f64 * p.lns_rcl_alpha).floor() as isize;
        let rcl: Vec<usize> = candidates
            .into_iter()
            .filter(|&(_, g)| g >= rcl_threshold) // Alle kandidaten die 'goed genoeg' zijn.
            .map(|(v, _)| v)
            .collect();

        // Stap 4: Kies een WILLEKEURIGE knoop uit de lijst van goede kandidaten en voeg toe.
        if let Some(&chosen) = rcl.choose(rng) {
            sol.add(chosen);
        } else {
            // Als de RCL om een of andere reden leeg is, kunnen we niet verder.
            break;
        }
    }
    // --- EINDE NIEUWE LOGICA ---


    // --- Fase 2: Mini-TSQC Refinement ---
    // Dit deel blijft ongewijzigd. Na de (nu slimmere) reconstructie,
    // proberen we de oplossing lokaal nog wat te verbeteren.
    if p.lns_repair_depth > 0 {
        let mut tabu = DualTabu::new(graph.n(), p.tenure_u, p.tenure_v);
        let mut freq = vec![0; graph.n()]; 
        let best_rho = 0.0;

        for _ in 0..p.lns_repair_depth {
            if !improve_once(&mut sol, &mut tabu, best_rho, &mut freq, p, rng) {
                break;
            }
        }
    }

    sol
}