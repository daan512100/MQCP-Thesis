// src/mcts.rs
//!
//! Implementeert de Monte Carlo Tree Search.

use crate::{graph::Graph, lns::apply_lns, params::Params, solution::Solution};
use bitvec::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::{HashMap, HashSet};

#[cfg(feature = "parallel_mcts")]
use rayon::prelude::*;

/// Private helper-functie om de handmatige intersectie-telling uit te voeren.
fn count_intersecting_ones(a: &BitSlice, b: &BitSlice) -> usize {
    a.iter().by_vals().zip(b.iter().by_vals()).filter(|&(x, y)| x && y).count()
}

// VERBETERD: MctsNode is nu publiek voor gebruik in unit tests en de merge-logica.
#[derive(Clone)]
pub struct MctsNode {
    parent: Option<usize>,
    children: HashMap<usize, usize>, // VERBETERD: HashMap<vertex_removed, node_idx> voor snelle lookups.
    visits: u32,
    total_reward: f64,
    vertex_removed: Option<usize>,
    depth: usize,
}

pub struct MctsTree<'g> {
    // VERBETERD: nodes is nu publiek voor gebruik in unit tests.
    pub nodes: Vec<MctsNode>,
    initial_solution: Solution<'g>,
    graph: &'g Graph,
    params: &'g Params,
}

impl<'g> MctsTree<'g> {
    pub fn new(initial_solution: &Solution<'g>, graph: &'g Graph, params: &'g Params) -> Self {
        MctsTree {
            nodes: vec![MctsNode {
                parent: None,
                children: HashMap::new(), // Gebruik HashMap
                visits: 0,
                total_reward: 0.0,
                vertex_removed: None,
                depth: 0,
            }],
            initial_solution: initial_solution.clone(),
            graph,
            params,
        }
    }

    pub fn run<R: Rng +?Sized + Send + Sync>(&mut self, rng: &mut R) -> Vec<usize> {
        #[cfg(feature = "parallel_mcts")]
        {
            let threads = rayon::current_num_threads().max(1);
            let budget_per_thread = self.params.mcts_budget.max(threads) / threads;

            if budget_per_thread < 1 {
                // Fallback voor een zeer klein budget
                self.run_simulations(self.params.mcts_budget, rng);
                return self.extract_best_sequence();
            }

            let results: Vec<MctsTree> = (0..threads)
               .into_par_iter()
               .map(|_| {
                    // Elke thread krijgt zijn eigen RNG en boom
                    let mut local_rng = rand::thread_rng();
                    let mut local_tree = MctsTree::new(&self.initial_solution, self.graph, self.params);
                    local_tree.run_simulations(budget_per_thread, &mut local_rng);
                    local_tree
                })
               .collect();

            // Voeg alle resultaten samen in de hoofdboom
            for other_tree in results {
                self.merge_from(&other_tree);
            }

            return self.extract_best_sequence();
        }

        #[cfg(not(feature = "parallel_mcts"))]
        {
            self.run_simulations(self.params.mcts_budget, rng);
            self.extract_best_sequence()
        }
    }

    fn run_simulations<R: Rng +?Sized>(&mut self, budget: usize, rng: &mut R) {
        for _ in 0..budget {
            let (leaf_idx, removal_path) = self.select();
            let new_node_idx = self.expand(leaf_idx, &removal_path, rng);
            // De rollout wordt nu uitgevoerd vanaf de *nieuwe* of *bestaande* leaf node.
            let reward = self.rollout(new_node_idx, rng);
            self.backpropagate(new_node_idx, reward);
        }
    }

    fn select(&self) -> (usize, Vec<usize>) {
        let mut current_idx = 0;
        let mut path = Vec::new();
        
        loop {
            let current_node = &self.nodes[current_idx];
            if current_node.children.is_empty() || current_node.depth >= self.params.mcts_max_depth {
                break;
            }

            let parent_visits = current_node.visits;
            // De iteratie over children.values() is nu correct omdat children een HashMap is.
            let best_child = *current_node
               .children
               .values()
               .max_by(|&&a, &&b| {
                    let uct_a = self.uct_score(a, parent_visits);
                    let uct_b = self.uct_score(b, parent_visits);
                    uct_a.partial_cmp(&uct_b).unwrap_or(std::cmp::Ordering::Equal)
                })
               .unwrap(); //.unwrap() is veilig omdat we checken op.is_empty()

            path.push(self.nodes[best_child].vertex_removed.unwrap());
            current_idx = best_child;
        }
        (current_idx, path)
    }

    fn expand<R: Rng +?Sized>(&mut self, node_idx: usize, path: &[usize], rng: &mut R) -> usize {
        // Expansie is alleen zinvol als de node al is bezocht en de max diepte niet is bereikt.
        if self.nodes[node_idx].visits == 0 || self.nodes[node_idx].depth >= self.params.mcts_max_depth {
            return node_idx;
        }

        let mut current_sol = self.initial_solution.clone();
        for &v in path {
            current_sol.remove(v);
        }
        
        if current_sol.size() == 0 {
            return node_idx; // Kan niet verder uitbreiden als de oplossing leeg is
        }

        let threshold = (self.params.gamma_target * (current_sol.size().saturating_sub(1)) as f64).floor() as usize;
        let sol_bitset = current_sol.bitset();

        let mut critical_subset: Vec<usize> = sol_bitset
           .iter_ones()
           .filter(|&u| count_intersecting_ones(self.graph.neigh_row(u), sol_bitset) <= threshold)
           .collect();
        
        // Filter knopen die al als kind zijn geprobeerd.
        let tried_children_vertices: HashSet<usize> = self.nodes[node_idx].children.keys().cloned().collect();
        critical_subset.retain(|v|!tried_children_vertices.contains(v));

        if critical_subset.is_empty() {
            // Fallback: als er geen kritieke kandidaten meer zijn, neem dan alle nog niet geprobeerde knopen.
            critical_subset = sol_bitset.iter_ones().filter(|v|!tried_children_vertices.contains(v)).collect();
        }

        if let Some(&vertex_to_remove) = critical_subset.choose(rng) {
            let new_node = MctsNode {
                parent: Some(node_idx),
                children: HashMap::new(),
                visits: 0,
                total_reward: 0.0,
                vertex_removed: Some(vertex_to_remove),
                depth: self.nodes[node_idx].depth + 1,
            };
            self.nodes.push(new_node);
            let new_node_idx = self.nodes.len() - 1;
            // Voeg het nieuwe kind toe aan de ouder.
            self.nodes[node_idx].children.insert(vertex_to_remove, new_node_idx);
            return new_node_idx;
        }

        node_idx // Geen nieuwe knoop om uit te breiden, retourneer de huidige.
    }

    fn rollout<R: Rng +?Sized>(&self, from_node_idx: usize, rng: &mut R) -> f64 {
        // KRITIEKE WIJZIGING: De beloning is nu gebaseerd op KWALITEIT (dichtheid), niet op grootte.
        
        // 1. Reconstrueer het pad van verwijderingen dat naar deze knoop leidt.
        let mut path = Vec::new();
        let mut current_idx_opt = Some(from_node_idx);
        while let Some(current_idx) = current_idx_opt {
            let current_node = &self.nodes[current_idx];
            if let Some(v) = current_node.vertex_removed {
                path.push(v);
            }
            current_idx_opt = current_node.parent;
        }
        path.reverse();

        // 2. Pas LNS toe om de oplossing te herstellen.
        let repaired_sol = apply_lns(&self.initial_solution, &path, self.params, rng);

        // 3. Bereken een betekenisvolle, samengestelde beloning.
        let density = repaired_sol.density();
        let is_feasible = repaired_sol.is_gamma_feasible(self.params.gamma_target);
        
        // Een beloning > 1.0 voor haalbare oplossingen, en < 1.0 voor onhaalbare.
        // Dit creëert een sterk signaal voor de MCTS om haalbaarheid te prioriteren.
        let reward = if is_feasible {
            1.0 + density 
        } else {
            density
        };
        
        reward
    }

    fn backpropagate(&mut self, start_idx: usize, reward: f64) {
        let mut current_idx_opt = Some(start_idx);
        while let Some(idx) = current_idx_opt {
            let node = &mut self.nodes[idx];
            node.visits += 1;
            node.total_reward += reward;
            current_idx_opt = node.parent;
        }
    }

    // VERBETERD: uct_score is nu publiek voor gebruik in unit tests.
    pub fn uct_score(&self, node_idx: usize, parent_visits: u32) -> f64 {
        let node = &self.nodes[node_idx];
        if node.visits == 0 {
            return f64::INFINITY;
        }
        let exploitation = node.total_reward / node.visits as f64;
        let exploration = self.params.mcts_exploration_const * ((parent_visits as f64).ln() / node.visits as f64).sqrt();
        exploitation + exploration
    }

    fn extract_best_sequence(&self) -> Vec<usize> {
        let mut seq = Vec::new();
        let mut current_idx = 0;
        
        while!self.nodes[current_idx].children.is_empty() {
            // Kies het kind met de hoogste *gemiddelde beloning* (pure exploitatie), niet UCT.
            let best_child_idx = *self.nodes[current_idx]
               .children
               .values()
               .max_by(|&&a, &&b| {
                    let node_a = &self.nodes[a];
                    let node_b = &self.nodes[b];
                    let avg_reward_a = node_a.total_reward / node_a.visits.max(1) as f64;
                    let avg_reward_b = node_b.total_reward / node_b.visits.max(1) as f64;
                    avg_reward_a.partial_cmp(&avg_reward_b).unwrap_or(std::cmp::Ordering::Equal)
                })
               .unwrap();

            seq.push(self.nodes[best_child_idx].vertex_removed.unwrap());
            current_idx = best_child_idx;
        }
        seq
    }

    fn merge_from(&mut self, other: &MctsTree) {
        // VERBETERDE, RECURSIEVE MERGE-LOGICA
        if other.nodes.is_empty() { return; }
        self.recursive_merge(0, other, 0);
    }
    
    fn recursive_merge(&mut self, self_node_idx: usize, other_tree: &MctsTree, other_node_idx: usize) {
        let other_node = &other_tree.nodes[other_node_idx];
        
        // Update de statistieken van de huidige knoop
        self.nodes[self_node_idx].visits += other_node.visits;
        self.nodes[self_node_idx].total_reward += other_node.total_reward;

        // Doorloop de kinderen van de 'other' knoop
        for (&other_vertex, &other_child_idx) in &other_node.children {
            // Kijk of dit kind (geïdentificeerd door de verwijderde vertex) al bestaat in de 'self' boom
            if let Some(&self_child_idx) = self.nodes[self_node_idx].children.get(&other_vertex) {
                // Ja, het bestaat: roep de merge recursief aan voor deze sub-boom
                self.recursive_merge(self_child_idx, other_tree, other_child_idx);
            } else {
                // Nee, het bestaat niet: kopieer de hele sub-boom van 'other' naar 'self'
                self.copy_subtree(Some(self_node_idx), other_tree, other_child_idx);
            }
        }
    }

    fn copy_subtree(&mut self, new_parent_idx: Option<usize>, other_tree: &MctsTree, other_node_idx: usize) -> usize {
        let other_node = &other_tree.nodes[other_node_idx];
        
        // Maak een diepe kopie van de knoop
        let mut new_node = other_node.clone();
        new_node.parent = new_parent_idx;
        new_node.children = HashMap::new(); // Kinderen worden recursief toegevoegd

        self.nodes.push(new_node);
        let new_node_idx = self.nodes.len() - 1;

        // Voeg de nieuwe knoop toe aan de kinderen van zijn nieuwe ouder
        if let Some(parent_idx) = new_parent_idx {
            let vertex = other_node.vertex_removed.unwrap();
            self.nodes[parent_idx].children.insert(vertex, new_node_idx);
        }
        
        // Roep recursief aan voor alle kinderen van de gekopieerde knoop
        for &other_child_idx in other_node.children.values() {
            self.copy_subtree(Some(new_node_idx), other_tree, other_child_idx);
        }
        
        new_node_idx
    }
}