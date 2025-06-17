//! src/graph.rs
//!
//! Representeert een simpele, ongerichte graaf met behulp van een 'BitVec' per rij
//! voor de adjacency matrix. Dit biedt efficiënte operaties voor het controleren
//! van buren en het berekenen van de interne graad, wat cruciaal is voor de
//! TSQC- en MCTS-algoritmes. Ondersteunt het parsen van het DIMACS *.clq-formaat.

use bitvec::prelude::*;
use std::io::{self, BufRead, Read};

/// Een ongerichte graaf, opgeslagen als een row-major adjacency matrix.
#[derive(Clone, Debug)]
pub struct Graph {
    /// Adjacency matrix; `adj[i][j]` is 1 als er een kant (i,j) bestaat, met j!= i.
    adj: Vec<BitVec>,
}

impl Graph {
    /*────────── Constructors ──────────*/

    /// Creëert een lege graaf met `n` geïsoleerde knopen.
    pub fn with_vertices(n: usize) -> Self {
        let mut rows = Vec::with_capacity(n);
        for _ in 0..n {
            rows.push(bitvec![0; n]);
        }
        Self { adj: rows }
    }

    /// Bouwt een graaf op basis van een expliciete lijst van kanten (0-gebaseerde indices).
    pub fn from_edge_list(n: usize, edges: &[(usize, usize)]) -> Self {
        let mut g = Self::with_vertices(n);
        for &(u, v) in edges {
            // Boundary check om panics te voorkomen bij ongeldige edge lists.
            if u < n && v < n {
                g.add_edge(u, v);
            }
        }
        g
    }

    /// Parset het DIMACS *.clq formaat vanuit een gebufferde reader.
    pub fn parse_dimacs<R: Read>(reader: R) -> io::Result<Self> {
        let mut n = 0usize;
        let mut edges: Vec<(usize, usize)> = Vec::new();
        let mut header_found = false;

        for line_result in io::BufReader::new(reader).lines() {
            let line = line_result?;
            let line = line.trim();

            if line.is_empty() || line.starts_with('c') {
                continue;
            }

            let parts: Vec<_> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "p" if parts.len() >= 4 && parts[1] == "edge" => {
                    // CORRECTIE: Parse de individuele onderdelen (parts[2], parts[3])
                    // van de vector, niet de vector zelf.
                    n = parts[2].parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                    let m_expected: usize = parts[3].parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                    edges.reserve(m_expected);
                    header_found = true;
                }
                "e" if parts.len() >= 3 => {
                    if !header_found {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Edge line 'e' found before problem line 'p'"));
                    }
                    // CORRECTIE: Parse de individuele onderdelen (parts[1], parts[2]).
                    let u: usize = parts[1].parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                    let v: usize = parts[2].parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

                    if u > 0 && v > 0 && u <= n && v <= n {
                        edges.push((u - 1, v - 1)); // DIMACS is 1-based, wij zijn 0-based
                    } else {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Edge ({}, {}) out of bounds for n={}", u, v, n)));
                    }
                }
                _ => { /* Negeer andere of ongeldige regels */ }
            }
        }
        Ok(Self::from_edge_list(n, &edges))
    }

    /*────────── Getters ──────────*/

    /// Geeft het aantal knopen (vertices) in de graaf terug.
    #[inline]
    pub fn n(&self) -> usize {
        self.adj.len()
    }

    /// Geeft het aantal kanten (edges) in de graaf terug (elke kant eenmaal geteld).
    pub fn m(&self) -> usize {
        self.adj.iter().map(|row| row.count_ones()).sum::<usize>() / 2
    }

    /// Geeft de graad (degree) van knoop `v` terug.
    #[inline]
    pub fn degree(&self, v: usize) -> usize {
        self.adj[v].count_ones()
    }

    /// Geeft een onveranderlijke slice van de adjacency-rij voor knoop `v`.
    #[inline]
    pub fn neigh_row(&self, v: usize) -> &bitvec::slice::BitSlice {
        &self.adj[v]
    }

    /*────────── Mutators ──────────*/

    /// Voegt een ongerichte kant toe tussen knopen `u` en `v`.
    #[inline]
    pub fn add_edge(&mut self, u: usize, v: usize) {
        assert!(u < self.n() && v < self.n() && u != v, "Knoopindex buiten bereik of zelf-lus");
        self.adj[u].set(v, true);
        self.adj[v].set(u, true);
    }
}