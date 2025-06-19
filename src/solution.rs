// src/solution.rs

use crate::graph::Graph;
use bitvec::prelude::*;

#[derive(Clone, Debug)]
pub struct Solution<'g> {
    graph: &'g Graph,
    vertices: BitVec,
    edge_count: usize,
    size: usize,
}

impl<'g> Solution<'g> {
    /*────────── Constructors ──────────*/
    pub fn new(graph: &'g Graph) -> Self {
        Self {
            graph,
            vertices: bitvec![0; graph.n()],
            edge_count: 0,
            size: 0,
        }
    }

    /*────────── Queries ──────────*/

    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    #[inline]
    pub fn edges(&self) -> usize {
        self.edge_count
    }

    #[inline]
    pub fn bitset(&self) -> &BitSlice {
        &self.vertices
    }
    
    #[inline]
    pub fn graph(&self) -> &'g Graph {
        self.graph
    }

    pub fn density(&self) -> f64 {
        if self.size < 2 {
            0.0
        } else {
            2.0 * self.edge_count as f64 / (self.size * (self.size - 1)) as f64
        }
    }

    pub fn is_gamma_feasible(&self, gamma: f64) -> bool {
        self.density() + 1e-9 >= gamma
    }

    // --- DEFINITIEVE CORRECTIE HIERONDER ---
    // Deze versie gebruikt de iterator-methode die de compiler wel begrijpt
    // en die je al succesvol in andere bestanden gebruikte.
    pub fn count_connections(&self, v: usize) -> usize {
        self.graph.neigh_row(v)
            .iter()
            .by_vals()
            .zip(self.bitset().iter().by_vals())
            .filter(|&(a, b)| a && b)
            .count()
    }
    // --- EINDE CORRECTIE ---

    /*────────── Mutators ──────────*/
    
    pub fn add(&mut self, v: usize) {
        if self.vertices[v] {
            return;
        }
        
        let added_edges = self.count_connections(v);
        
        self.vertices.set(v, true);
        self.size += 1;
        self.edge_count += added_edges;
    }

    pub fn remove(&mut self, v: usize) {
        if !self.vertices[v] {
            return;
        }
        
        let removed_edges = self.count_connections(v);
        
        self.vertices.set(v, false);
        self.size -= 1;
        self.edge_count -= removed_edges;
    }
}