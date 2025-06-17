//! src/params.rs
//!
//! Bundelt alle afstembare parameters voor de TSQC-oplosser.

/// Alle afstembare besturingselementen voor TSQC en de MCTS-LNS uitbreiding.
#[derive(Clone, Debug)]
pub struct Params {
    // --- Algemene TSQC-parameters ---
    /// Doeldichtheid `gamma` in (0,1] die een gamma-quasi-clique definieert.
    pub gamma_target: f64,

    /// `L`: maximaal aantal opeenvolgende niet-verbeterende swaps
    /// voordat een diversificatiestap wordt geactiveerd (Sectie 3.1).
    pub stagnation_iter: usize,

    /// `It_max`: harde limiet op het totale aantal TSQC-iteraties
    /// over alle restarts heen (Sectie 3.1).
    pub max_iter: usize,

    // --- Tabu-parameters ---
    /// Basisduur voor het verbieden van recent verwijderde knopen (Tu).
    /// *Opmerking:* Dit is slechts een veiligheidsminimum; de werkelijke Tu
    /// wordt elke iteratie adaptief herberekend (§3.4.3).
    pub tenure_u: usize,

    /// Basisduur voor het verbieden van recent toegevoegde knopen (Tv).
    /// *Opmerking:* De werkelijke Tv is eveneens adaptief.
    pub tenure_v: usize,

    // --- MCTS-LNS Diversificatieparameters ---
    /// Vlag om MCTS-gestuurde LNS-diversificatie in te schakelen
    /// in plaats van de standaard 'shake'-perturbaties.
    pub use_mcts: bool,

    /// MCTS-budget: aantal gesimuleerde playouts voor de geleide diversificatie.
    pub mcts_budget: usize,

    /// Exploratieconstante (UCT) voor MCTS-knoopselectie.
    pub mcts_exploration_const: f64,

    /// Maximale verwijderingsdiepte (lengte van de sequentie) in MCTS.
    pub mcts_max_depth: usize,

    /// Aantal lokale hersteliteraties (mini-TSQC) na de MCTS-verwijdering.
    pub lns_repair_depth: usize,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            // TSQC defaults
            gamma_target: 0.90,
            stagnation_iter: 1_000,   // L = 1000, per 
            max_iter: 100_000_000, // It_max = 10^8, per 
            tenure_u: 1,           // Veiligheidsminimum
            tenure_v: 1,           // Veiligheidsminimum

            // MCTS-LNS defaults (standaard uitgeschakeld)
            use_mcts: false,
            mcts_budget: 100,
            mcts_exploration_const: 1.414, // sqrt(2)
            mcts_max_depth: 5,
            lns_repair_depth: 10,
        }
    }
}

impl Params {
    /// Schakelt MCTS-gestuurde LNS-diversificatie in met de opgegeven parameters.
    pub fn enable_mcts(
        &mut self,
        budget: usize,
        exploration_const: f64,
        max_depth: usize,
        repair_depth: usize,
    ) -> &mut Self {
        self.use_mcts = true;
        self.mcts_budget = budget;
        self.mcts_exploration_const = exploration_const;
        self.mcts_max_depth = max_depth;
        self.lns_repair_depth = repair_depth;
        self
    }
}