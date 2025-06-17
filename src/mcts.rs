//! src/mcts.rs
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

struct MctsNode {
    parent: Option<usize>,
    children: Vec<usize>,
    visits: u32,
    total_reward: f64,
    vertex_removed: Option<usize>,
    depth: usize,
}

pub struct MctsTree<'g> {
    nodes: Vec<MctsNode>,
    initial_solution: Solution<'g>,
    graph: &'g Graph,
    params: &'g Params,
}

impl<'g> MctsTree<'g> {
    pub fn new(initial_solution: &Solution<'g>, graph: &'g Graph, params: &'g Params) -> Self {
        MctsTree {
            nodes: vec![MctsNode {
                parent: None,
                children: Vec::new(),
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

    pub fn run<R: Rng + ?Sized + Send + Sync>(&mut self, rng: &mut R) -> Vec<usize> {
        #[cfg(feature = "parallel_mcts")]
        {
            let threads = rayon::current_num_threads();
            let budget_per_thread = self.params.mcts_budget.max(threads) / threads;

            if budget_per_thread == 0 {
                self.run_simulations(self.params.mcts_budget, rng);
                return self.extract_best_sequence();
            }

            let results: Vec<MctsTree> = (0..threads)
                .into_par_iter()
                .map(|_| {
                    let mut local_rng = rand::thread_rng();
                    let mut local_tree = MctsTree::new(&self.initial_solution, self.graph, self.params);
                    local_tree.run_simulations(budget_per_thread, &mut local_rng);
                    local_tree
                })
                .collect();

            for tree in results {
                self.merge_from(&tree);
            }

            return self.extract_best_sequence();
        }

        #[cfg(not(feature = "parallel_mcts"))]
        {
            self.run_simulations(self.params.mcts_budget, rng);
            self.extract_best_sequence()
        }
    }

    fn run_simulations<R: Rng + ?Sized>(&mut self, budget: usize, rng: &mut R) {
        for _ in 0..budget {
            let (leaf_idx, removal_path) = self.select();
            let new_node_idx = self.expand(leaf_idx, &removal_path, rng);
            let reward = self.rollout(&self.nodes[new_node_idx], rng);
            self.backpropagate(new_node_idx, reward);
        }
    }

    fn select(&self) -> (usize, Vec<usize>) {
        let mut current_idx = 0;
        let mut path = Vec::new();
        while !self.nodes[current_idx].children.is_empty() {
            let parent_visits = self.nodes[current_idx].visits;
            let best_child = *self.nodes[current_idx]
                .children
                .iter()
                .max_by(|&&a, &&b| {
                    let uct_a = self.uct_score(a, parent_visits);
                    let uct_b = self.uct_score(b, parent_visits);
                    uct_a.partial_cmp(&uct_b).unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap();

            path.push(self.nodes[best_child].vertex_removed.unwrap());
            current_idx = best_child;
            if self.nodes[current_idx].depth >= self.params.mcts_max_depth {
                break;
            }
        }
        (current_idx, path)
    }

    fn expand<R: Rng + ?Sized>(&mut self, node_idx: usize, path: &[usize], rng: &mut R) -> usize {
        if self.nodes[node_idx].visits == 0 || self.nodes[node_idx].depth >= self.params.mcts_max_depth {
            return node_idx;
        }

        let mut current_sol = self.initial_solution.clone();
        for &v in path {
            current_sol.remove(v);
        }

        let threshold = (self.params.gamma_target * (current_sol.size().saturating_sub(1)) as f64).floor() as usize;
        let sol_bitset = current_sol.bitset();
        
        // CORRECTIE (E0369): De `&`-operator wordt vervangen door de helper-functie.
        let mut critical_subset: Vec<usize> = sol_bitset
            .iter_ones()
            .filter(|&u| count_intersecting_ones(self.graph.neigh_row(u), sol_bitset) <= threshold)
            .collect();

        let tried_children: HashSet<usize> = self.nodes[node_idx].children.iter().map(|&c| self.nodes[c].vertex_removed.unwrap()).collect();
        critical_subset.retain(|v| !tried_children.contains(v));

        if critical_subset.is_empty() {
            critical_subset = sol_bitset.iter_ones().filter(|v| !tried_children.contains(v)).collect();
        }

        if let Some(&vertex_to_remove) = critical_subset.choose(rng) {
            let new_node = MctsNode {
                parent: Some(node_idx),
                children: Vec::new(),
                visits: 0,
                total_reward: 0.0,
                vertex_removed: Some(vertex_to_remove),
                depth: self.nodes[node_idx].depth + 1,
            };
            self.nodes.push(new_node);
            let new_node_idx = self.nodes.len() - 1;
            self.nodes[node_idx].children.push(new_node_idx);
            return new_node_idx;
        }

        node_idx
    }

    fn rollout<R: Rng + ?Sized>(&self, from_node: &MctsNode, rng: &mut R) -> f64 {
        let mut path = Vec::new();
        let mut current_opt = Some(from_node);
        while let Some(current) = current_opt {
            if let Some(v) = current.vertex_removed {
                path.push(v);
            }
            current_opt = current.parent.map(|idx| &self.nodes[idx]);
        }
        path.reverse();

        let repaired_sol = apply_lns(&self.initial_solution, &path, self.params, rng);
        repaired_sol.size() as f64
    }

    fn backpropagate(&mut self, start_idx: usize, reward: f64) {
        let mut current_idx = Some(start_idx);
        while let Some(idx) = current_idx {
            let node = &mut self.nodes[idx];
            node.visits += 1;
            node.total_reward += reward;
            current_idx = node.parent;
        }
    }

    fn uct_score(&self, node_idx: usize, parent_visits: u32) -> f64 {
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
        while !self.nodes[current_idx].children.is_empty() {
            let best_child_idx = *self.nodes[current_idx]
                .children
                .iter()
                .max_by(|&&a, &&b| {
                    let avg_reward_a = self.nodes[a].total_reward / self.nodes[a].visits.max(1) as f64;
                    let avg_reward_b = self.nodes[b].total_reward / self.nodes[b].visits.max(1) as f64;
                    avg_reward_a.partial_cmp(&avg_reward_b).unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap();
            seq.push(self.nodes[best_child_idx].vertex_removed.unwrap());
            current_idx = best_child_idx;
        }
        seq
    }

    fn merge_from(&mut self, other: &MctsTree) {
        if self.nodes.is_empty() || other.nodes.is_empty() {
            return;
        }

        let self_children_map: HashMap<usize, usize> = self.nodes[0]
            .children
            .iter()
            .map(|&idx| (self.nodes[idx].vertex_removed.unwrap(), idx))
            .collect();

        let other_root = &other.nodes[0];
        self.nodes[0].visits += other_root.visits;
        self.nodes[0].total_reward += other_root.total_reward;

        for &other_child_idx in &other_root.children {
            let other_child = &other.nodes[other_child_idx];
            let vertex = other_child.vertex_removed.unwrap();

            if let Some(&self_child_idx) = self_children_map.get(&vertex) {
                self.nodes[self_child_idx].visits += other_child.visits;
                self.nodes[self_child_idx].total_reward += other_child.total_reward;
            } else {
                let new_node = MctsNode {
                    parent: Some(0),
                    children: Vec::new(),
                    visits: other_child.visits,
                    total_reward: other_child.total_reward,
                    vertex_removed: other_child.vertex_removed,
                    depth: 1,
                };
                self.nodes.push(new_node);
                let new_idx = self.nodes.len() - 1;
                self.nodes[0].children.push(new_idx);
            }
        }
    }
}