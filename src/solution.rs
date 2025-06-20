//! src/solution.rs
//!
//! Representeert een kandidaat-oplossing: een subset van knopen 'S' met
//! gecachte waarden voor de grootte '|S|' en het aantal kanten 'f(S)'.
//! Dit ontwerp biedt O(1) toegang tot de grootte en het aantal kanten, en
//! efficiënte O(n/64) updates per toevoeg- of verwijderoperatie.

use crate::graph::Graph;
use bitvec::prelude::*;

/// Een veranderlijke quasi-clique kandidaat, gebonden aan een specifieke `Graph`.
#[derive(Clone, Debug)]
pub struct Solution<'g> {
    graph: &'g Graph,
    vertices: BitVec,
    edge_count: usize,
    size: usize,
}

impl<'g> Solution<'g> {
    /*────────── Constructors ──────────*/

    /// Creëert een nieuwe, lege oplossing voor de gegeven graaf.
    pub fn new(graph: &'g Graph) -> Self {
        Self {
            graph,
            vertices: bitvec![0; graph.n()],
            edge_count: 0,
            size: 0,
        }
    }

    /*────────── Queries ──────────*/

    /// Geeft de grootte van de oplossing `|S|` terug.
    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Geeft het aantal kanten in de geïnduceerde subgraaf `f(S)` terug.
    #[inline]
    pub fn edges(&self) -> usize {
        self.edge_count
    }

    /// Geeft een onveranderlijke slice van de bitset die de knopen in `S` representeert.
    #[inline]
    pub fn bitset(&self) -> &BitSlice {
        &self.vertices
    }

    /// Geeft een referentie naar de onderliggende graaf.
    #[inline]
    pub fn graph(&self) -> &'g Graph {
        self.graph
    }

    /// Berekent de dichtheid `2 * f(S) / (|S| * (|S| - 1))`.
    /// Geeft 0.0 terug als `|S| < 2`.
    pub fn density(&self) -> f64 {
        if self.size < 2 {
            0.0
        } else {
            2.0 * self.edge_count as f64 / (self.size * (self.size - 1)) as f64
        }
    }

    /// Controleert of de oplossing voldoet aan de `gamma`-drempel.
    /// Een kleine epsilon wordt gebruikt om floating-point onnauwkeurigheden te ondervangen.
    pub fn is_gamma_feasible(&self, gamma: f64) -> bool {
        self.density() + 1e-9 >= gamma
    }

    /*────────── Mutators ──────────*/

    /// Voegt knoop `v` toe aan de oplossing. Negeert de operatie als `v` al aanwezig is.
    /// Werkt de gecachte `size` en `edge_count` efficiënt bij.
    pub fn add(&mut self, v: usize) {
        if self.vertices[v] {
            return;
        }
        
        // CORRECTIE (E0369): Vervang de `&`-operator door de handmatige intersectie-telling.
        let added_edges = self.graph.neigh_row(v)
            .iter()
            .by_vals()
            .zip(self.bitset().iter().by_vals())
            .filter(|&(a, b)| a && b)
            .count();

        self.vertices.set(v, true);
        self.size += 1;
        self.edge_count += added_edges;
    }

    /// Verwijdert knoop `v` uit de oplossing. Negeert de operatie als `v` niet aanwezig is.
    /// Werkt de gecachte `size` en `edge_count` efficiënt bij.
    pub fn remove(&mut self, v: usize) {
        if !self.vertices[v] {
            return;
        }
        
        // CORRECTIE (E0369): Pas dezelfde fix toe voor de `remove`-methode.
        // We tellen de verbindingen van `v` met de *huidige* oplossing, voordat `v` wordt verwijderd.
        let removed_edges = self.graph.neigh_row(v)
            .iter()
            .by_vals()
            .zip(self.bitset().iter().by_vals())
            .filter(|&(a, b)| a && b)
            .count();

        self.vertices.set(v, false);
        self.size -= 1;
        self.edge_count -= removed_edges;
    }
}