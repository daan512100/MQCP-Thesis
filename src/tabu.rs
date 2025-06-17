//! src/tabu.rs
//!
//! Implementeert de dubbele tabu-lijsten met adaptieve duur, zoals
//! beschreven in ScriptiePaper.pdf, Sectie 3.4.3.

use rand::Rng;

/// Beheert twee korte-termijn tabu-geheugens.
#[derive(Clone, Debug)]
pub struct DualTabu {
    /// `expiry_u[v]` is de iteratie waarin knoop `v` weer mag worden toegevoegd.
    expiry_u: Vec<usize>,
    /// `expiry_v[v]` is de iteratie waarin knoop `v` weer mag worden verwijderd.
    expiry_v: Vec<usize>,
    /// Huidige globale iteratieteller.
    iter: usize,
    /// Huidige tabu-duur voor het opnieuw toevoegen (Tu).
    tu: usize,
    /// Huidige tabu-duur voor het verwijderen (Tv).
    tv: usize,
}

impl DualTabu {
    /// Creëert een nieuw DualTabu-object voor `n` knopen met initiële duren.
    pub fn new(n: usize, initial_tu: usize, initial_tv: usize) -> Self {
        Self {
            expiry_u: vec![0; n],
            expiry_v: vec![0; n],
            iter: 0,
            tu: initial_tu.max(1),
            tv: initial_tv.max(1),
        }
    }

    /// Controleert of knoop `v` taboe is voor her-toevoegen.
    #[inline]
    pub fn is_tabu_u(&self, v: usize) -> bool {
        self.expiry_u[v] > self.iter
    }

    /// Controleert of knoop `v` taboe is voor verwijderen.
    #[inline]
    pub fn is_tabu_v(&self, v: usize) -> bool {
        self.expiry_v[v] > self.iter
    }

    /// Herberekent `Tu` en `Tv` op basis van de huidige staat van de oplossing.
    /// Dit implementeert de formules uit Sectie 3.4.3, waarmee de bug
    /// `TSQC-03` is opgelost.
    pub fn update_tenures<R: Rng + ?Sized>(
        &mut self,
        size_s: usize,
        edges: usize,
        gamma: f64,
        rng: &mut R,
    ) {
        if size_s < 2 {
            self.tu = 1;
            self.tv = 1;
            return;
        }

        let max_possible_edges = size_s * (size_s - 1) / 2;
        let needed_edges = (gamma * max_possible_edges as f64).ceil() as usize;
        let l = needed_edges.saturating_sub(edges).min(10);
        let c = (size_s / 40).max(6);
        // TSQC-03: Gebruik inclusieve Random(c) volgens de paper: [0..=c]
        let rand_u = if c > 1 { rng.gen_range(0..=c) } else { 0 };
        self.tu = (l + rand_u).max(1);

        let base_v = (0.6 * l as f64).floor() as usize;
        let c6 = (0.6 * c as f64).floor() as usize;
        // TSQC-03: Gebruik inclusieve Random(c6)
        let rand_v = if c6 > 1 { rng.gen_range(0..=c6) } else { 0 };
        self.tv = (base_v + rand_v).max(1);
    }

    /// Maakt knoop `v` taboe om te worden toegevoegd voor `tu` iteraties.
    #[inline]
    pub fn forbid_u(&mut self, v: usize) {
        self.expiry_u[v] = self.iter + self.tu;
    }

    /// Maakt knoop `v` taboe om te worden verwijderd voor `tv` iteraties.
    #[inline]
    pub fn forbid_v(&mut self, v: usize) {
        self.expiry_v[v] = self.iter + self.tv;
    }

    /// Verhoogt de globale iteratieteller.
    #[inline]
    pub fn step(&mut self) {
        self.iter += 1;
    }

    /// Reset alle tabu-markeringen.
    pub fn reset(&mut self) {
        self.expiry_u.fill(0);
        self.expiry_v.fill(0);
    }
}
